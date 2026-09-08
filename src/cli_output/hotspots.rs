//! Refactoring priorities and their explanations.

use crate::balance::project::analyze_project_balance_with_thresholds;
use crate::{CouplingIssue, Distance, IssueThresholds, ProjectMetrics, Severity};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
};

/// A hotspot module that needs attention
#[derive(Debug, Clone, Serialize)]
pub struct Hotspot {
    /// Module name
    pub module: String,
    /// Hotspot score (higher = more urgent)
    pub score: u32,
    /// Issues found in this module
    pub issues: Vec<HotspotIssue>,
    /// Suggested fix action
    pub suggestion: String,
    /// File path if available
    pub file_path: Option<String>,
    /// Whether this module is in a circular dependency
    pub in_cycle: bool,
}

/// An issue contributing to a hotspot
#[derive(Debug, Clone, Serialize)]
pub struct HotspotIssue {
    pub severity: String,
    pub issue_type: String,
    pub description: String,
}

// ============================================================================
// Beginner-friendly explanations
// ============================================================================

/// Get a beginner-friendly explanation for an issue type
pub fn get_issue_explanation(issue_type: &str) -> IssueExplanation {
    match issue_type {
        "High Efferent Coupling" => IssueExplanation {
            what_it_means: "This module depends on too many other modules",
            why_its_bad: vec![
                "Changes elsewhere may break this module",
                "Testing requires many mocks/stubs",
                "Hard to understand in isolation",
            ],
            how_to_fix: "Split into smaller modules with clear responsibilities",
            example: Some("e.g., Split main.rs into cli.rs, config.rs, runner.rs"),
        },
        "High Afferent Coupling" => IssueExplanation {
            what_it_means: "Too many other modules depend on this one",
            why_its_bad: vec![
                "Changes here may break many other modules",
                "Fear of changing leads to technical debt",
                "Wide blast radius for bugs",
            ],
            how_to_fix: "Define a stable interface (trait) to hide implementation details",
            example: Some("e.g., pub struct -> pub trait + impl for abstraction"),
        },
        "Circular Dependency" | "CircularDependency" => IssueExplanation {
            what_it_means: "Modules depend on each other in a cycle (A -> B -> A)",
            why_its_bad: vec![
                "Can't understand one without the other",
                "Unit testing is difficult (need both)",
                "May cause compilation order issues",
            ],
            how_to_fix: "Extract shared types to a common module, or use traits to invert dependencies",
            example: Some("e.g., A -> B -> A becomes A -> Common <- B"),
        },
        "Global Complexity" => IssueExplanation {
            what_it_means: "Strong coupling to a distant module",
            why_its_bad: vec![
                "Hard to trace code flow",
                "Changes have unpredictable effects",
                "Module is not self-contained",
            ],
            how_to_fix: "Move the dependency closer, or use an interface for loose coupling",
            example: None,
        },
        "Cascading Change Risk" => IssueExplanation {
            what_it_means: "Strongly coupled to a frequently-changing module",
            why_its_bad: vec![
                "Every change there requires changes here",
                "Bugs propagate through the chain",
                "Constant rework needed",
            ],
            how_to_fix: "Depend on a stable interface instead of implementation",
            example: None,
        },
        "Scattered External Coupling" => IssueExplanation {
            what_it_means: "A third-party crate is used directly from many modules",
            why_its_bad: vec![
                "Crate API changes have a wide edit surface",
                "Upgrade risk is spread across unrelated modules",
                "Harder to replace or mock the dependency",
            ],
            how_to_fix: "Introduce a facade or wrapper module around the crate",
            example: Some("e.g., reqwest calls go through http_client.rs"),
        },
        "Inappropriate Intimacy" | "InappropriateIntimacy" => IssueExplanation {
            what_it_means: "Directly accessing another module's internal details",
            why_its_bad: vec![
                "Breaks encapsulation",
                "Internal changes affect external code",
                "Unclear module boundaries",
            ],
            how_to_fix: "Access through public methods or traits instead",
            example: Some("e.g., foo.internal_field -> foo.get_value()"),
        },
        _ => IssueExplanation {
            what_it_means: "A coupling-related issue was detected",
            why_its_bad: vec![
                "May reduce code maintainability",
                "May increase change impact",
            ],
            how_to_fix: "Review the module dependencies",
            example: None,
        },
    }
}

/// Beginner-friendly explanation for an issue
pub struct IssueExplanation {
    /// What this issue means in simple terms
    pub what_it_means: &'static str,
    /// Why this is problematic
    pub why_its_bad: Vec<&'static str>,
    /// How to fix it
    pub how_to_fix: &'static str,
    /// Optional example
    pub example: Option<&'static str>,
}

/// Calculate hotspots from project metrics
pub fn calculate_hotspots(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    limit: usize,
) -> Vec<Hotspot> {
    let circular_deps = metrics.detect_circular_dependencies();
    calculate_hotspots_with_cycles(metrics, thresholds, limit, &circular_deps)
}

pub(super) fn calculate_hotspots_with_cycles(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    limit: usize,
    circular_deps: &[Vec<String>],
) -> Vec<Hotspot> {
    let report = analyze_project_balance_with_thresholds(metrics, thresholds);
    let names = crate::design::graph::DependencyGraph {
        nodes: metrics.modules.keys().cloned().collect(),
        ..Default::default()
    };
    let canonical = |name: &str| names.resolve(name).unwrap_or_else(|| name.to_string());
    let cycle_modules: HashSet<String> = circular_deps
        .iter()
        .flatten()
        .map(|name| canonical(name))
        .collect();

    // Group issues by source module
    let mut module_issues: HashMap<String, Vec<&CouplingIssue>> = HashMap::new();
    for issue in &report.issues {
        module_issues
            .entry(canonical(&issue.source))
            .or_default()
            .push(issue);
    }

    // Calculate coupling counts per module
    let mut couplings_out: HashMap<String, usize> = HashMap::new();
    let mut couplings_in: HashMap<String, usize> = HashMap::new();
    for coupling in &metrics.couplings {
        if coupling.distance != Distance::DifferentCrate {
            *couplings_out
                .entry(canonical(&coupling.source))
                .or_default() += 1;
            *couplings_in.entry(canonical(&coupling.target)).or_default() += 1;
        }
    }

    // Build hotspots
    let mut hotspots: Vec<Hotspot> = Vec::new();

    for (module, issues) in &module_issues {
        let mut score: u32 = 0;

        // Base score from issue count and severity
        for issue in issues {
            score += match issue.severity {
                Severity::Critical => 50,
                Severity::High => 30,
                Severity::Medium => 15,
                Severity::Low => 5,
            };
        }

        // Bonus for circular dependencies
        let in_cycle = cycle_modules.contains(module);
        if in_cycle {
            score += 40;
        }

        // Bonus for high coupling count
        let out_count = couplings_out.get(module).copied().unwrap_or(0);
        let in_count = couplings_in.get(module).copied().unwrap_or(0);
        score += (out_count + in_count) as u32 * 2;

        // Determine primary issue type for suggestion
        let primary_issue = issues.iter().max_by_key(|i| i.severity);
        let suggestion = if in_cycle {
            "Break circular dependency by extracting shared types or inverting with traits".into()
        } else if let Some(issue) = primary_issue {
            format!("{}", issue.refactoring)
        } else {
            "Review module coupling".into()
        };

        // Get file path
        let file_path = metrics
            .modules
            .get(module)
            .map(|m| m.path.display().to_string());

        hotspots.push(Hotspot {
            module: module.clone(),
            score,
            issues: issues
                .iter()
                .map(|i| HotspotIssue {
                    severity: format!("{}", i.severity),
                    issue_type: format!("{}", i.issue_type),
                    description: i.description.clone(),
                })
                .collect(),
            suggestion,
            file_path,
            in_cycle,
        });
    }

    // Also add modules in cycles that don't have other issues
    for module in &cycle_modules {
        if !module_issues.contains_key(module) {
            let file_path = metrics
                .modules
                .get(module)
                .map(|m| m.path.display().to_string());

            hotspots.push(Hotspot {
                module: module.clone(),
                score: 40,
                issues: vec![HotspotIssue {
                    severity: "Critical".into(),
                    issue_type: "CircularDependency".into(),
                    description: "Part of a circular dependency cycle".into(),
                }],
                suggestion:
                    "Break circular dependency by extracting shared types or inverting with traits"
                        .into(),
                file_path,
                in_cycle: true,
            });
        }
    }

    // Sort by score descending, then module name for deterministic ties.
    hotspots.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.module.cmp(&b.module)));
    hotspots.truncate(limit);

    hotspots
}

/// Generate hotspots output to writer
pub fn generate_hotspots_output<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    limit: usize,
    verbose: bool,
    writer: &mut W,
) -> io::Result<()> {
    let hotspots = calculate_hotspots(metrics, thresholds, limit);

    writeln!(writer, "Top {} Refactoring Targets", limit)?;
    writeln!(
        writer,
        "═══════════════════════════════════════════════════════════"
    )?;

    if hotspots.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "✅ No significant hotspots detected.")?;
        writeln!(writer, "   Your codebase has good coupling balance.")?;
        return Ok(());
    }

    writeln!(writer)?;

    for (i, hotspot) in hotspots.iter().enumerate() {
        // Header with rank and score
        writeln!(
            writer,
            "#{} {} (Score: {})",
            i + 1,
            hotspot.module,
            hotspot.score
        )?;

        // File path if available
        if let Some(path) = &hotspot.file_path {
            writeln!(writer, "   📁 {}", path)?;
        }

        // Issues with optional verbose explanations
        for issue in &hotspot.issues {
            let icon = match issue.severity.as_str() {
                "Critical" => "🔴",
                "High" => "🟠",
                "Medium" => "🟡",
                _ => "⚪",
            };
            writeln!(
                writer,
                "   {} {}: {}",
                icon, issue.severity, issue.issue_type
            )?;

            // Show beginner-friendly explanation in verbose mode
            if verbose {
                let explanation = get_issue_explanation(&issue.issue_type);
                writeln!(writer)?;
                writeln!(writer, "   💡 What it means:")?;
                writeln!(writer, "      {}", explanation.what_it_means)?;
                writeln!(writer)?;
                writeln!(writer, "   ⚠️  Why it's a problem:")?;
                for reason in &explanation.why_its_bad {
                    writeln!(writer, "      • {}", reason)?;
                }
                writeln!(writer)?;
                writeln!(writer, "   🔧 How to fix:")?;
                writeln!(writer, "      {}", explanation.how_to_fix)?;
                if let Some(example) = explanation.example {
                    writeln!(writer, "      {}", example)?;
                }
                writeln!(writer)?;
            }
        }

        // Suggestion (only if not verbose, since verbose already shows how_to_fix)
        if !verbose {
            writeln!(writer, "   → Fix: {}", hotspot.suggestion)?;
        }
        writeln!(writer)?;
    }

    Ok(())
}
