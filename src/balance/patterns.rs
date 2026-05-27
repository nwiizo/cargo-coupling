use std::collections::HashMap;

use crate::metrics::dimensions::{Distance, Subdomain, Visibility};
use crate::metrics::project::ProjectMetrics;

use super::action::RefactoringAction;
use super::coupling::is_entrypoint_module;
use super::issue::CouplingIssue;
use super::issue_type::IssueType;
use super::labels::extract_type_name;
use super::score::IssueThresholds;
use super::severity::Severity;
use super::subdomain::build_target_subdomain_map;

pub(crate) fn analyze_module_coupling(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
) -> Vec<CouplingIssue> {
    let mut issues = Vec::new();
    let target_subdomains = build_target_subdomain_map(metrics);

    // Count outgoing (efferent) and incoming (afferent) couplings per module
    // Only count INTERNAL dependencies (within workspace), not external crates
    let mut efferent: HashMap<&str, usize> = HashMap::new();
    let mut afferent: HashMap<&str, usize> = HashMap::new();

    for coupling in &metrics.couplings {
        // Skip external crate dependencies entirely
        if coupling.distance == Distance::DifferentCrate {
            continue;
        }

        *efferent.entry(&coupling.source).or_insert(0) += 1;
        *afferent.entry(&coupling.target).or_insert(0) += 1;
    }

    // Check for high efferent coupling (depends on too many things)
    for (module, count) in &efferent {
        if *count > thresholds.max_dependencies {
            // A binary entrypoint must wire the whole application together, so its
            // fan-out is expected by design — report it as a Low informational note
            // rather than a High-severity action item.
            let entrypoint = is_entrypoint_module(module);
            let severity = if entrypoint {
                Severity::Low
            } else if *count > thresholds.max_dependencies * 2 {
                Severity::High
            } else {
                Severity::Medium
            };
            let description = if entrypoint {
                format!(
                    "Entrypoint {} wires {} components — expected for a binary entrypoint, not a defect",
                    module, count
                )
            } else {
                format!(
                    "Module {} depends on {} other components (threshold: {})",
                    module, count, thresholds.max_dependencies
                )
            };
            let refactoring = if entrypoint {
                RefactoringAction::General {
                    action: "No action needed: entrypoints are expected to have wide fan-out"
                        .to_string(),
                }
            } else {
                RefactoringAction::SplitModule {
                    suggested_modules: vec![
                        format!("{}_core", module),
                        format!("{}_integration", module),
                    ],
                }
            };
            issues.push(CouplingIssue {
                issue_type: IssueType::HighEfferentCoupling,
                severity,
                source: module.to_string(),
                target: format!("{} dependencies", count),
                description,
                refactoring,
                balance_score: 1.0
                    - (*count as f64 / (thresholds.max_dependencies * 3) as f64).min(1.0),
            });
        }
    }

    // Check for high afferent coupling (too many things depend on this)
    // Only internal modules are counted (external crates already filtered above)
    for (module, count) in &afferent {
        if *count > thresholds.max_dependents {
            // A widely-depended-on module is only risky if it is VOLATILE; a STABLE
            // central abstraction with many dependents is good design (the balance
            // rule: strong + far + low volatility = Acceptable). Scale severity by
            // the module's essential volatility.
            let short = module.rsplit("::").next().unwrap_or(module);
            let subdomain = target_subdomains.get(short).copied().flatten();
            let base_severity = if *count > thresholds.max_dependents * 2 {
                Severity::High
            } else {
                Severity::Medium
            };
            let (severity, description, refactoring) = match subdomain {
                // Stable central abstraction: high afferent is acceptable, not a defect.
                Some(Subdomain::Supporting) | Some(Subdomain::Generic) => (
                    Severity::Low,
                    format!(
                        "Module {} is a stable central abstraction with {} dependents — acceptable; only risky if it becomes volatile",
                        module, count
                    ),
                    RefactoringAction::General {
                        action: "No action needed while this module stays stable".to_string(),
                    },
                ),
                // Volatile core: many dependents AND it genuinely evolves -> changes cascade widely.
                Some(Subdomain::Core) => (
                    base_severity,
                    format!(
                        "Core module {} is depended on by {} components AND genuinely evolves — changes cascade widely; stabilize its public contract",
                        module, count
                    ),
                    RefactoringAction::IntroduceTrait {
                        suggested_name: format!("{}Interface", extract_type_name(module)),
                        methods: vec!["// Define a stable public contract".to_string()],
                    },
                ),
                // Unclassified: keep prior behavior.
                None => (
                    base_severity,
                    format!(
                        "Module {} is depended on by {} other components (threshold: {})",
                        module, count, thresholds.max_dependents
                    ),
                    RefactoringAction::IntroduceTrait {
                        suggested_name: format!("{}Interface", extract_type_name(module)),
                        methods: vec!["// Define stable public API".to_string()],
                    },
                ),
            };
            issues.push(CouplingIssue {
                issue_type: IssueType::HighAfferentCoupling,
                severity,
                source: format!("{} dependents", count),
                target: module.to_string(),
                description,
                refactoring,
                balance_score: 1.0
                    - (*count as f64 / (thresholds.max_dependents * 3) as f64).min(1.0),
            });
        }
    }

    issues
}

/// Analyze Rust-specific patterns (God Module, Public Field Exposure, Primitive Obsession)
pub(crate) fn analyze_rust_patterns(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
) -> Vec<CouplingIssue> {
    let mut issues = Vec::new();

    // God Module detection
    for (module_name, module) in &metrics.modules {
        // Calculate function count, excluding test functions if configured
        let func_count = if thresholds.exclude_tests {
            module
                .function_count()
                .saturating_sub(module.test_function_count)
        } else {
            module.function_count()
        };
        let type_count = module.type_definitions.len();
        let impl_count = module.trait_impl_count + module.inherent_impl_count;

        // Check if module exceeds thresholds (with test exclusion applied)
        let is_god_module = func_count > thresholds.max_functions
            || type_count > thresholds.max_types
            || impl_count > thresholds.max_impls;

        if is_god_module {
            issues.push(CouplingIssue {
                issue_type: IssueType::GodModule,
                severity: if func_count > thresholds.max_functions * 2
                    || type_count > thresholds.max_types * 2
                {
                    Severity::High
                } else {
                    Severity::Medium
                },
                source: module_name.clone(),
                target: format!(
                    "{} functions, {} types, {} impls",
                    func_count, type_count, impl_count
                ),
                description: format!(
                    "Module {} has too many responsibilities (functions: {}/{}, types: {}/{}, impls: {}/{})",
                    module_name,
                    func_count, thresholds.max_functions,
                    type_count, thresholds.max_types,
                    impl_count, thresholds.max_impls,
                ),
                refactoring: RefactoringAction::SplitModule {
                    suggested_modules: vec![
                        format!("{}_core", module_name),
                        format!("{}_helpers", module_name),
                    ],
                },
                balance_score: 0.5,
            });
        }

        // Public Field Exposure detection
        for type_def in module.type_definitions.values() {
            if type_def.public_field_count > 0
                && !type_def.is_trait
                && type_def.visibility == Visibility::Public
            {
                issues.push(CouplingIssue {
                    issue_type: IssueType::PublicFieldExposure,
                    severity: Severity::Low,
                    source: format!("{}::{}", module_name, type_def.name),
                    target: format!("{} public fields", type_def.public_field_count),
                    description: format!(
                        "Type {} has {} public field(s). Consider using getter methods.",
                        type_def.name, type_def.public_field_count
                    ),
                    refactoring: RefactoringAction::AddGetters {
                        fields: vec!["// Add getter methods".to_string()],
                    },
                    balance_score: 0.7,
                });
            }
        }

        // Primitive Obsession detection
        for func_def in module.function_definitions.values() {
            if func_def.primitive_param_count >= thresholds.min_primitive_params
                && func_def.param_count >= thresholds.min_primitive_params
            {
                let ratio = func_def.primitive_param_count as f64 / func_def.param_count as f64;
                if ratio >= 0.6 {
                    issues.push(CouplingIssue {
                        issue_type: IssueType::PrimitiveObsession,
                        severity: Severity::Low,
                        source: format!("{}::{}", module_name, func_def.name),
                        target: format!(
                            "{}/{} primitive params",
                            func_def.primitive_param_count, func_def.param_count
                        ),
                        description: format!(
                            "Function {} has {} primitive parameters. Consider newtype pattern.",
                            func_def.name, func_def.primitive_param_count
                        ),
                        refactoring: RefactoringAction::IntroduceNewtype {
                            suggested_name: format!("{}Params", capitalize_first(&func_def.name)),
                            wrapped_type: "// Group related parameters".to_string(),
                        },
                        balance_score: 0.7,
                    });
                }
            }
        }
    }

    issues
}

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
