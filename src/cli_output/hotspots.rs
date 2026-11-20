//! Refactoring priorities and their explanations.

use crate::balance::project::analyze_project_balance_with_thresholds;
use crate::{IssueThresholds, ProjectMetrics};
use serde::Serialize;
use std::{
    collections::HashSet,
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
            how_to_fix: "Check for unrelated responsibilities before splitting; orchestration can legitimately have high fan-out",
            example: Some("e.g., Split main.rs into cli.rs, config.rs, runner.rs"),
        },
        "High Afferent Coupling" => IssueExplanation {
            what_it_means: "Too many other modules depend on this one",
            why_its_bad: vec![
                "Changes here may break many other modules",
                "Fear of changing leads to technical debt",
                "Wide blast radius for bugs",
            ],
            how_to_fix: "Check whether consumers depend on volatile details; retain a stable shared abstraction when appropriate",
            example: Some("e.g., pub struct -> pub trait + impl for abstraction"),
        },
        "Circular Dependency" | "CircularDependency" => IssueExplanation {
            what_it_means: "Modules depend on each other in a cycle (A -> B -> A)",
            why_its_bad: vec![
                "Can't understand one without the other",
                "Unit testing is difficult (need both)",
                "May cause compilation order issues",
            ],
            how_to_fix: "Confirm whether the cycle crosses responsibilities that must evolve independently before changing its boundaries",
            example: Some("e.g., A -> B -> A becomes A -> Common <- B"),
        },
        "Global Complexity" => IssueExplanation {
            what_it_means: "Strong coupling to a distant module",
            why_its_bad: vec![
                "Hard to trace code flow",
                "Changes have unpredictable effects",
                "Module is not self-contained",
            ],
            how_to_fix: "Compare retaining a stable boundary, moving related responsibilities closer, and hiding shared knowledge",
            example: None,
        },
        "Cascading Change Risk" => IssueExplanation {
            what_it_means: "Strongly coupled to a frequently-changing module",
            why_its_bad: vec![
                "Every change there requires changes here",
                "Bugs propagate through the chain",
                "Constant rework needed",
            ],
            how_to_fix: "Confirm the shared knowledge and expected changes before introducing an integration-specific interface",
            example: None,
        },
        "Scattered External Coupling" => IssueExplanation {
            what_it_means: "A third-party crate is used directly from many modules",
            why_its_bad: vec![
                "Crate API changes have a wide edit surface",
                "Upgrade risk is spread across unrelated modules",
                "Harder to replace or mock the dependency",
            ],
            how_to_fix: "Check actual replacement needs and public API exposure before paying the maintenance cost of a wrapper",
            example: Some("e.g., reqwest calls go through http_client.rs"),
        },
        "Inappropriate Intimacy" | "InappropriateIntimacy" => IssueExplanation {
            what_it_means: "Directly accessing another module's internal details",
            why_its_bad: vec![
                "Breaks encapsulation",
                "Internal changes affect external code",
                "Unclear module boundaries",
            ],
            how_to_fix: "Check whether access exposes unintended knowledge; a getter that exposes the same representation may not help",
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
    let names = crate::design::graph::DependencyGraph::from_metrics(metrics);
    let canonical = |name: &str| names.resolve(name).unwrap_or_else(|| name.to_string());
    let cycle_modules: HashSet<String> = circular_deps
        .iter()
        .flatten()
        .map(|name| canonical(name))
        .collect();

    // Use the design report's baseline ordering. Context may add explicit
    // business weights in --design, but presentation never invents another score.
    crate::design::planning::priorities(
        &names, &crate::design::context::DesignContext::default(), &report.issues, &[],
    ).into_iter().take(limit).map(|priority| {
        let module = priority.module;
        let issues = report.issues.iter().filter(|issue|
            canonical(&issue.source) == module || canonical(&issue.target) == module
        ).map(|issue| HotspotIssue {
            severity: issue.severity.to_string(),
            issue_type: issue.issue_type.to_string(),
            description: issue.description.clone(),
        }).collect();
        Hotspot {
            score: priority.priority_score as u32,
            file_path: metrics.modules.get(&module).map(|m| m.path.display().to_string()),
            in_cycle: cycle_modules.contains(&module),
            module,
            issues,
            suggestion: "Confirm the shared reason for change and expected future changes; compare retaining the boundary, hiding knowledge, and moving related responsibilities. A cycle alone does not prescribe a refactoring.".into(),
        }
    }).collect()
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
        writeln!(
            writer,
            "   No matching findings in the analyzed scope; check coverage before concluding safety."
        )?;
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
                writeln!(writer, "   🔧 Before changing:")?;
                writeln!(writer, "      {}", explanation.how_to_fix)?;
                if let Some(example) = explanation.example {
                    writeln!(writer, "      {}", example)?;
                }
                writeln!(writer)?;
            }
        }

        // Suggestion (only if not verbose, since verbose already shows how_to_fix)
        if !verbose {
            writeln!(writer, "   → Review: {}", hotspot.suggestion)?;
        }
        writeln!(writer)?;
    }

    Ok(())
}
