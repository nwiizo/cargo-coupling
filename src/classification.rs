//! Coupling classification: given a detected dependency, decide its target
//! module (including re-exported bare type names via the type registry), its
//! structural distance (module-tree adjacency), and its integration strength
//! (Rust-faithful: published data-model access is Model, unpublished access is
//! Intrusive). Extracted from `analyzer` so parsing and classification stay
//! separately cohesive.

use std::collections::HashSet;

use crate::analyzer::{Dependency, UsageContext};
use crate::discovery::join_module_path;
use crate::metrics::dimensions::{Distance, IntegrationStrength, Visibility};
use crate::metrics::project::ProjectMetrics;
use crate::workspace::WorkspaceInfo;

// ===== Dependency Resolution =====

/// Calculate distance using workspace information.
pub(crate) fn calculate_distance_with_workspace(
    source_module: &str,
    target_module: &str,
    target_is_known_internal_module: bool,
    current_crate: &str,
    resolved_crate: Option<&str>,
    workspace: &WorkspaceInfo,
) -> Distance {
    let Some(target_crate) = resolved_crate else {
        return Distance::DifferentCrate;
    };

    if target_crate != current_crate {
        return if workspace.is_workspace_member(target_crate) {
            Distance::DifferentModule
        } else {
            Distance::DifferentCrate
        };
    }

    calculate_distance(
        source_module,
        target_module,
        target_is_known_internal_module,
    )
}

/// Structurally adjacent modules are scored as local cohesion:
/// - same module, or any ancestor/descendant line, is `SameModule`;
/// - siblings under one non-root parent package are `SameModule`;
/// - root-level siblings stay `DifferentModule` because the crate root is a
///   publication boundary, not evidence of a cohesive package.
pub(crate) fn is_adjacent_module(source_module: &str, target_module: &str) -> bool {
    let source_segments: Vec<&str> = source_module.split("::").collect();
    let target_segments: Vec<&str> = target_module.split("::").collect();

    if source_segments == target_segments {
        return true;
    }

    let shorter_len = source_segments.len().min(target_segments.len());
    let source_prefix = &source_segments[..shorter_len];
    let target_prefix = &target_segments[..shorter_len];
    if source_prefix == target_prefix {
        return true;
    }

    let Some(source_parent) = source_segments.split_last().map(|(_, parent)| parent) else {
        return false;
    };
    let Some(target_parent) = target_segments.split_last().map(|(_, parent)| parent) else {
        return false;
    };

    !source_parent.is_empty() && source_parent == target_parent
}

pub(crate) fn target_type_name(path: &str) -> Option<&str> {
    path.trim_end_matches("::*")
        .rsplit("::")
        .next()
        .filter(|segment| !segment.is_empty() && *segment != "*")
}

pub(crate) fn strength_for_dependency(
    dep: &Dependency,
    target_visibility: Option<Visibility>,
) -> IntegrationStrength {
    match dep.usage {
        UsageContext::FieldAccess | UsageContext::StructConstruction => {
            if target_visibility == Some(Visibility::Public) {
                IntegrationStrength::Model
            } else {
                IntegrationStrength::Intrusive
            }
        }
        _ => dep.usage.to_strength(),
    }
}

pub(crate) fn visibility_for_dependency(
    dep: &Dependency,
    target_visibility: Option<Visibility>,
) -> Visibility {
    target_visibility.unwrap_or(match dep.usage {
        UsageContext::FieldAccess | UsageContext::StructConstruction => Visibility::Private,
        _ => Visibility::Public,
    })
}

/// Extract target module name from a path
pub(crate) fn extract_target_module(path: &str) -> String {
    // Remove common prefixes and get the module name
    let cleaned = path
        .trim_start_matches("crate::")
        .trim_start_matches("super::")
        .trim_start_matches("::");

    // Get first significant segment
    cleaned.split("::").next().unwrap_or(path).to_string()
}

pub(crate) fn resolve_target_module(
    path: &str,
    source_module: &str,
    known_modules: &HashSet<String>,
    project: &ProjectMetrics,
) -> String {
    let resolved = resolve_relative_module_path(path, source_module);
    let segments: Vec<&str> = resolved
        .split("::")
        .filter(|segment| !segment.is_empty())
        .collect();

    for len in (1..=segments.len()).rev() {
        let candidate = segments[..len].join("::");
        if known_modules.contains(&candidate) {
            return candidate;
        }
    }

    if should_resolve_bare_type(path)
        && let Some(type_name) = target_type_name(&resolved)
        && let Some(module_name) = project.get_type_module(type_name)
    {
        return module_name.to_string();
    }

    extract_target_module(path)
}

pub(crate) fn should_resolve_bare_type(path: &str) -> bool {
    !path.contains("::")
        || path.starts_with("crate::")
        || path.starts_with("self::")
        || path.starts_with("super::")
        || path.starts_with("::")
}

pub(crate) fn resolve_relative_module_path(path: &str, source_module: &str) -> String {
    if let Some(rest) = path.strip_prefix("crate::") {
        return rest.to_string();
    }
    if let Some(rest) = path.strip_prefix("self::") {
        return join_module_path(source_module, rest);
    }

    let mut rest = path;
    let mut parent_levels = 0;
    while let Some(next) = rest.strip_prefix("super::") {
        parent_levels += 1;
        rest = next;
    }

    if parent_levels == 0 {
        return path.trim_start_matches("::").to_string();
    }

    let mut base: Vec<&str> = source_module.split("::").collect();
    for _ in 0..parent_levels {
        base.pop();
    }
    let prefix = base.join("::");
    join_module_path(&prefix, rest)
}

/// Check if a path looks like a valid module/type reference (not a local variable)
pub(crate) fn is_valid_dependency_path(path: &str) -> bool {
    // Skip empty paths
    if path.is_empty() {
        return false;
    }

    // Skip Self references
    if path == "Self" || path.starts_with("Self::") {
        return false;
    }

    let segments: Vec<&str> = path.split("::").collect();

    // Skip short single-segment lowercase names (likely local variables)
    if segments.len() == 1 {
        let name = segments[0];
        if name.len() <= 8 && name.chars().all(|c| c.is_lowercase() || c == '_') {
            return false;
        }
    }

    // Skip patterns where last two segments are the same (likely module::type patterns from variables)
    if segments.len() >= 2 {
        let last = segments.last().unwrap();
        let second_last = segments.get(segments.len() - 2).unwrap();
        if last == second_last {
            return false;
        }
    }

    // Skip common patterns that look like local variable accesses
    let last_segment = segments.last().unwrap_or(&path);
    let common_locals = [
        "request",
        "response",
        "result",
        "content",
        "config",
        "proto",
        "domain",
        "info",
        "data",
        "item",
        "value",
        "error",
        "message",
        "expected",
        "actual",
        "status",
        "state",
        "context",
        "params",
        "args",
        "options",
        "settings",
        "violation",
        "page_token",
    ];
    if common_locals.contains(last_segment) && segments.len() <= 2 {
        return false;
    }

    true
}

/// Calculate same-crate structural distance after target resolution.
pub(crate) fn calculate_distance(
    source_module: &str,
    target_module: &str,
    target_is_known_internal_module: bool,
) -> Distance {
    if !target_is_known_internal_module {
        return Distance::DifferentCrate;
    }

    if is_adjacent_module(source_module, target_module) {
        Distance::SameModule
    } else {
        Distance::DifferentModule
    }
}
