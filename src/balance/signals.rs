use std::collections::{HashMap, HashSet};

use crate::metrics::dimensions::Subdomain;
use crate::metrics::project::ProjectMetrics;

use super::action::RefactoringAction;
use super::issue::CouplingIssue;
use super::issue_type::IssueType;
use super::severity::Severity;

pub(crate) fn analyze_hidden_temporal_coupling(metrics: &ProjectMetrics) -> Vec<CouplingIssue> {
    let file_to_module = build_file_to_module_map(metrics);
    let mut seen = HashSet::new();
    let mut issues = Vec::new();

    for temporal in metrics
        .temporal_couplings
        .iter()
        .filter(|tc| tc.is_strong())
    {
        let Some(source) = module_for_file(&temporal.file_a, &file_to_module) else {
            continue;
        };
        let Some(target) = module_for_file(&temporal.file_b, &file_to_module) else {
            continue;
        };

        if source == target || has_code_coupling(metrics, &source, &target) {
            continue;
        }

        let (stable_source, stable_target) = ordered_pair(&source, &target);
        if !seen.insert((stable_source.clone(), stable_target.clone())) {
            continue;
        }

        let ratio_pct = temporal.coupling_ratio * 100.0;
        issues.push(CouplingIssue {
            issue_type: IssueType::HiddenCoupling,
            severity: if temporal.coupling_ratio >= 0.8 {
                Severity::High
            } else {
                Severity::Medium
            },
            source: stable_source,
            target: stable_target,
            description: format!(
                "Strong temporal co-change without code dependency ({:.0}% ratio, {} co-changes)",
                ratio_pct, temporal.co_change_count
            ),
            refactoring: RefactoringAction::General {
                action: "Extract a shared abstraction or make the dependency explicit".to_string(),
            },
            balance_score: 1.0 - temporal.coupling_ratio,
        });
    }

    issues
}

/// Analyze modules whose git churn contradicts their expected subdomain volatility.
pub(crate) fn analyze_accidental_volatility(metrics: &ProjectMetrics) -> Vec<CouplingIssue> {
    if metrics.file_changes.len() < 4 {
        return Vec::new();
    }

    let threshold = top_quartile_change_threshold(&metrics.file_changes);
    let mut issues = Vec::new();

    for (module_name, module) in &metrics.modules {
        let Some(subdomain @ (Subdomain::Supporting | Subdomain::Generic)) = module.subdomain
        else {
            continue;
        };
        let change_count = change_count_for_path(&module.path, &metrics.file_changes);
        if change_count < threshold {
            continue;
        }

        issues.push(CouplingIssue {
            issue_type: IssueType::AccidentalVolatility,
            severity: Severity::Medium,
            source: module_name.clone(),
            target: module_name.clone(),
            description: format!(
                "Accidental volatility: {} is {} but changed {} times (top-quartile threshold: {})",
                module_name, subdomain, change_count, threshold
            ),
            refactoring: RefactoringAction::General {
                action: "Separate volatile policy from stable supporting/generic implementation"
                    .to_string(),
            },
            balance_score: 0.5,
        });
    }

    issues
}

fn build_file_to_module_map(metrics: &ProjectMetrics) -> Vec<(String, String)> {
    metrics
        .modules
        .iter()
        .map(|(name, module)| (normalize_path_string(&module.path), name.clone()))
        .collect()
}

fn module_for_file(file_path: &str, file_to_module: &[(String, String)]) -> Option<String> {
    let normalized_file = normalize_path_str(file_path);
    file_to_module
        .iter()
        .find(|(module_path, _)| paths_match(module_path, &normalized_file))
        .map(|(_, module_name)| module_name.clone())
}

/// Checks explicit couplings by assuming coupling source/target names end with
/// the short module name stored in `ProjectMetrics::modules`.
fn has_code_coupling(metrics: &ProjectMetrics, module_a: &str, module_b: &str) -> bool {
    metrics.couplings.iter().any(|coupling| {
        module_names_match(&coupling.source, module_a)
            && module_names_match(&coupling.target, module_b)
            || module_names_match(&coupling.source, module_b)
                && module_names_match(&coupling.target, module_a)
    })
}

fn module_names_match(coupling_module: &str, module_name: &str) -> bool {
    coupling_module == module_name
        || coupling_module
            .strip_suffix(module_name)
            .is_some_and(|prefix| prefix.ends_with("::"))
}

fn ordered_pair(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

fn top_quartile_change_threshold(file_changes: &HashMap<String, usize>) -> usize {
    let mut counts: Vec<_> = file_changes.values().copied().collect();
    counts.sort_unstable();
    let idx = counts.len() * 3 / 4;
    counts[idx.min(counts.len() - 1)].max(3)
}

fn change_count_for_path(path: &std::path::Path, file_changes: &HashMap<String, usize>) -> usize {
    let module_path = normalize_path_string(path);
    file_changes
        .iter()
        .filter(|(file_path, _)| paths_match(&module_path, &normalize_path_str(file_path)))
        .map(|(_, changes)| *changes)
        .max()
        .unwrap_or(0)
}

fn paths_match(module_path: &str, git_path: &str) -> bool {
    module_path == git_path
        || module_path.ends_with(&format!("/{git_path}"))
        || git_path.ends_with(&format!("/{module_path}"))
}

fn normalize_path_string(path: &std::path::Path) -> String {
    normalize_path_str(&path.to_string_lossy())
}

fn normalize_path_str(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}
