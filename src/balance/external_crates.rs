// ===== External Crate Heuristics =====

use crate::external::ExternalDependencyUsage;
use crate::{CouplingIssue, IssueType, RefactoringAction, Severity};

/// Number of internal modules above which direct third-party usage is considered scattered.
pub const SCATTERED_EXTERNAL_BREADTH_THRESHOLD: usize = 3;

/// Crate stability classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrateStability {
    /// Rust language fundamentals (std, core, alloc) - always ignore
    Fundamental,
    /// Highly stable, ubiquitous crates (serde, thiserror) - low concern
    Stable,
    /// Infrastructure crates (tokio, tracing) - medium concern
    Infrastructure,
    /// Regular external crate - normal analysis
    Normal,
}

/// Check the stability classification of a crate
pub fn classify_crate_stability(crate_name: &str) -> CrateStability {
    // Extract the base crate name (before ::)
    let base_name = crate_name.split("::").next().unwrap_or(crate_name).trim();

    match base_name {
        // Rust fundamentals - always safe to depend on
        "std" | "core" | "alloc" => CrateStability::Fundamental,

        // Highly stable, de-facto standard crates
        "serde" | "serde_json" | "serde_yaml" | "toml" |  // Serialization
        "thiserror" | "anyhow" |                           // Error handling
        "log" |                                            // Logging trait
        "chrono" | "time" |                                // Date/time
        "uuid" |                                           // UUIDs
        "regex" |                                          // Regex
        "lazy_static" | "once_cell" |                      // Statics
        "bytes" | "memchr" |                               // Byte utilities
        "itertools" |                                      // Iterator utilities
        "derive_more" | "strum"                            // Derive macros
        => CrateStability::Stable,

        // Infrastructure crates - stable but architectural decisions
        "tokio" | "async-std" | "smol" |                   // Async runtimes
        "async-trait" |                                    // Async traits
        "futures" | "futures-util" |                       // Futures
        "tracing" | "tracing-subscriber" |                 // Tracing
        "tracing-opentelemetry" | "opentelemetry" |        // Observability
        "opentelemetry-otlp" | "opentelemetry_sdk" |
        "hyper" | "reqwest" | "http" |                     // HTTP
        "tonic" | "prost" |                                // gRPC
        "sqlx" | "diesel" | "sea-orm" |                    // Database
        "clap" | "structopt"                               // CLI
        => CrateStability::Infrastructure,

        // Everything else
        _ => CrateStability::Normal,
    }
}

/// Check if a crate should be excluded from issue detection
pub fn should_skip_crate(crate_name: &str) -> bool {
    matches!(
        classify_crate_stability(crate_name),
        CrateStability::Fundamental
    )
}

/// Check if a crate should have reduced severity
pub fn should_reduce_severity(crate_name: &str) -> bool {
    matches!(
        classify_crate_stability(crate_name),
        CrateStability::Stable | CrateStability::Infrastructure
    )
}

/// Check if this is an external crate (not part of the workspace)
/// External crates are identified by not containing "::" or starting with known external patterns
pub fn is_external_crate(target: &str, source: &str) -> bool {
    // If target doesn't have ::, it might be external
    // But we need to check if it's the same workspace member

    // Extract the crate/module prefix
    let target_prefix = target.split("::").next().unwrap_or(target);
    let source_prefix = source.split("::").next().unwrap_or(source);

    // If they have the same prefix, it's internal
    if target_prefix == source_prefix {
        return false;
    }

    // If target looks like an external crate pattern (no workspace prefix match)
    // Check if it's a known stable/infrastructure crate
    let stability = classify_crate_stability(target);
    matches!(
        stability,
        CrateStability::Fundamental | CrateStability::Stable | CrateStability::Infrastructure
    )
}

/// Detect external crates used directly from many internal modules.
pub fn detect_scattered_external_coupling(
    dependencies: &[ExternalDependencyUsage],
) -> Vec<CouplingIssue> {
    dependencies
        .iter()
        .filter(|dependency| dependency.breadth > SCATTERED_EXTERNAL_BREADTH_THRESHOLD)
        .map(|dependency| {
            let severity = scattered_severity(dependency.breadth);
            CouplingIssue {
                issue_type: IssueType::ScatteredExternalCoupling,
                severity,
                source: format!("{} internal modules", dependency.breadth),
                target: dependency.crate_name.clone(),
                description: format!(
                    "{} is directly used from {} internal modules ({} references). Third-party upgrade risk is spread across the codebase.",
                    dependency.crate_name, dependency.breadth, dependency.total_references
                ),
                refactoring: RefactoringAction::General {
                    action: format!(
                        "Introduce a `{}` facade/wrapper module and route direct crate usage through it",
                        facade_module_name(&dependency.crate_name)
                    ),
                },
                balance_score: scattered_balance_score(dependency.breadth),
            }
        })
        .collect()
}

fn scattered_severity(breadth: usize) -> Severity {
    if breadth >= 10 {
        Severity::Critical
    } else if breadth >= 6 {
        Severity::High
    } else {
        Severity::Medium
    }
}

fn scattered_balance_score(breadth: usize) -> f64 {
    1.0 - (breadth as f64 / 12.0).min(1.0)
}

fn facade_module_name(crate_name: &str) -> String {
    format!("{}_facade", crate_name.replace('-', "_"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scattered_coupling_is_flagged_above_threshold() {
        let dependency = ExternalDependencyUsage {
            crate_name: "reqwest".to_string(),
            versions: vec![],
            breadth: 4,
            total_references: 9,
            dominant_strength: "Functional".to_string(),
            source_modules: vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
        };

        let issues = detect_scattered_external_coupling(&[dependency]);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, IssueType::ScatteredExternalCoupling);
        assert_eq!(issues[0].severity, Severity::Medium);
        assert_eq!(issues[0].target, "reqwest");
        assert!(format!("{}", issues[0].refactoring).contains("facade"));
    }
}
