//! CLI output functions for job-focused commands
//!
//! Provides specialized output formats for different JTBD (Jobs to be Done):
//! - Hotspots: Quick identification of refactoring priorities
//! - Impact: Change impact analysis for a specific module
//! - Check: CI/CD quality gate with exit codes
//! - JSON: Machine-readable output for automation

use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

use serde::Serialize;

use crate::balance::{
    BalanceScore, HealthGrade, IssueThresholds, Severity, analyze_project_balance_with_thresholds,
};
use crate::diff::BaselineDiff;
use crate::external::{
    ExternalDependencyReport, ExternalDependencyUsage, analyze_external_dependencies,
};
use crate::history::HistoryReport;
use crate::manifest::AnalysisManifest;
use crate::metrics::{Distance, ProjectMetrics};

// ============================================================================
// Hotspots: Refactoring Prioritization
// ============================================================================

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
    let report = analyze_project_balance_with_thresholds(metrics, thresholds);
    let circular_deps = metrics.detect_circular_dependencies();
    let cycle_modules: HashSet<String> = circular_deps.iter().flatten().cloned().collect();

    // Group issues by source module
    let mut module_issues: HashMap<String, Vec<&crate::balance::CouplingIssue>> = HashMap::new();
    for issue in &report.issues {
        module_issues
            .entry(issue.source.clone())
            .or_default()
            .push(issue);
    }

    // Calculate coupling counts per module
    let mut couplings_out: HashMap<String, usize> = HashMap::new();
    let mut couplings_in: HashMap<String, usize> = HashMap::new();
    for coupling in &metrics.couplings {
        if coupling.distance != Distance::DifferentCrate {
            *couplings_out.entry(coupling.source.clone()).or_default() += 1;
            *couplings_in.entry(coupling.target.clone()).or_default() += 1;
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

    // Sort by score descending
    hotspots.sort_by_key(|h| std::cmp::Reverse(h.score));
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

// ============================================================================
// Impact Analysis: Change Impact Assessment
// ============================================================================

/// Impact analysis result for a module
#[derive(Debug, Clone, Serialize)]
pub struct ImpactAnalysis {
    /// The module being analyzed
    pub module: String,
    /// Risk score (0-100)
    pub risk_score: u32,
    /// Risk level label
    pub risk_level: String,
    /// Direct dependencies (what this module depends on)
    pub dependencies: Vec<DependencyInfo>,
    /// Direct dependents (what depends on this module)
    pub dependents: Vec<DependencyInfo>,
    /// Cascading impact information
    pub cascading_impact: CascadingImpact,
    /// Whether module is in a circular dependency
    pub in_cycle: bool,
    /// Volatility information
    pub volatility: String,
}

/// Information about a dependency relationship (grouped by module)
#[derive(Debug, Clone, Serialize)]
pub struct DependencyInfo {
    /// Target/source module name
    pub module: String,
    /// Distance to the module
    pub distance: String,
    /// Coupling counts by strength type
    pub strengths: Vec<StrengthCount>,
    /// Total coupling count
    pub total_count: usize,
}

/// Count of couplings by strength type
#[derive(Debug, Clone, Serialize)]
pub struct StrengthCount {
    pub strength: String,
    pub count: usize,
}

/// Cascading impact analysis
#[derive(Debug, Clone, Serialize)]
pub struct CascadingImpact {
    /// Total modules affected (directly + indirectly)
    pub total_affected: usize,
    /// Percentage of codebase affected
    pub percentage: f64,
    /// Second-order dependencies (modules affected through dependents)
    pub second_order: Vec<String>,
}

/// Analyze impact of changing a specific module
pub fn analyze_impact(metrics: &ProjectMetrics, module_name: &str) -> Option<ImpactAnalysis> {
    // Find exact match or partial match
    let module = find_module(metrics, module_name)?;

    let circular_deps = metrics.detect_circular_dependencies();
    let cycle_modules: HashSet<String> = circular_deps.iter().flatten().cloned().collect();
    let in_cycle = cycle_modules.contains(&module);

    // Collect and group dependencies by target module
    let mut dep_map: HashMap<String, (String, HashMap<String, usize>)> = HashMap::new();
    let mut dependent_map: HashMap<String, (String, HashMap<String, usize>)> = HashMap::new();
    let mut volatility_max = crate::metrics::Volatility::Low;

    for coupling in &metrics.couplings {
        if coupling.distance == Distance::DifferentCrate {
            continue; // Skip external crates
        }

        if coupling.source == module {
            let entry = dep_map
                .entry(coupling.target.clone())
                .or_insert_with(|| (format!("{:?}", coupling.distance), HashMap::new()));
            *entry
                .1
                .entry(format!("{:?}", coupling.strength))
                .or_insert(0) += 1;
        }

        if coupling.target == module {
            let entry = dependent_map
                .entry(coupling.source.clone())
                .or_insert_with(|| (format!("{:?}", coupling.distance), HashMap::new()));
            *entry
                .1
                .entry(format!("{:?}", coupling.strength))
                .or_insert(0) += 1;

            // Track max volatility of incoming couplings
            if coupling.volatility > volatility_max {
                volatility_max = coupling.volatility;
            }
        }
    }

    // Convert to DependencyInfo with grouped strengths
    let dependencies: Vec<DependencyInfo> = dep_map
        .into_iter()
        .map(|(mod_name, (distance, strengths))| {
            let total_count: usize = strengths.values().sum();
            let mut strength_list: Vec<StrengthCount> = strengths
                .into_iter()
                .map(|(s, c)| StrengthCount {
                    strength: s,
                    count: c,
                })
                .collect();
            // Sort by count descending
            strength_list.sort_by_key(|s| std::cmp::Reverse(s.count));
            DependencyInfo {
                module: mod_name,
                distance,
                strengths: strength_list,
                total_count,
            }
        })
        .collect();

    let dependents: Vec<DependencyInfo> = dependent_map
        .into_iter()
        .map(|(mod_name, (distance, strengths))| {
            let total_count: usize = strengths.values().sum();
            let mut strength_list: Vec<StrengthCount> = strengths
                .into_iter()
                .map(|(s, c)| StrengthCount {
                    strength: s,
                    count: c,
                })
                .collect();
            strength_list.sort_by_key(|s| std::cmp::Reverse(s.count));
            DependencyInfo {
                module: mod_name,
                distance,
                strengths: strength_list,
                total_count,
            }
        })
        .collect();

    // Calculate second-order impact (what depends on our dependents)
    let mut second_order: HashSet<String> = HashSet::new();
    let dependent_set: HashSet<String> = dependents.iter().map(|d| d.module.clone()).collect();

    for coupling in &metrics.couplings {
        if coupling.distance == Distance::DifferentCrate {
            continue;
        }
        if dependent_set.contains(&coupling.target) && coupling.source != module {
            second_order.insert(coupling.source.clone());
        }
    }
    // Remove direct dependents from second order
    for dep in &dependent_set {
        second_order.remove(dep);
    }

    let total_affected = dependents.len() + second_order.len();
    let total_internal_modules = metrics.modules.len();
    let percentage = if total_internal_modules > 0 {
        (total_affected as f64 / total_internal_modules as f64) * 100.0
    } else {
        0.0
    };

    // Calculate risk score
    let mut risk_score: u32 = 0;
    risk_score += (dependents.len() as u32) * 10; // Each dependent adds risk
    risk_score += (second_order.len() as u32) * 5; // Second order less risky
    if in_cycle {
        risk_score += 30;
    }
    match volatility_max {
        crate::metrics::Volatility::High => risk_score += 20,
        crate::metrics::Volatility::Medium => risk_score += 10,
        crate::metrics::Volatility::Low => {}
    }
    risk_score = risk_score.min(100);

    let risk_level = if risk_score >= 70 {
        "HIGH"
    } else if risk_score >= 40 {
        "MEDIUM"
    } else {
        "LOW"
    }
    .to_string();

    let volatility = format!("{:?}", volatility_max);

    Some(ImpactAnalysis {
        module: module.clone(),
        risk_score,
        risk_level,
        dependencies,
        dependents,
        cascading_impact: CascadingImpact {
            total_affected,
            percentage,
            second_order: second_order.into_iter().collect(),
        },
        in_cycle,
        volatility,
    })
}

fn find_module(metrics: &ProjectMetrics, name: &str) -> Option<String> {
    // First check couplings since those are the names we use for matching
    // Prefer full coupling source/target names over short module names
    for coupling in &metrics.couplings {
        // Exact match
        if coupling.source == name {
            return Some(coupling.source.clone());
        }
        if coupling.target == name {
            return Some(coupling.target.clone());
        }
    }

    // Suffix match in couplings (e.g., "main" matches "cargo-coupling::main")
    for coupling in &metrics.couplings {
        if coupling.source.ends_with(&format!("::{}", name)) {
            return Some(coupling.source.clone());
        }
        if coupling.target.ends_with(&format!("::{}", name)) {
            return Some(coupling.target.clone());
        }
    }

    // Exact match in modules map
    if metrics.modules.contains_key(name) {
        return Some(name.to_string());
    }

    // Partial match (suffix) in modules
    for module_name in metrics.modules.keys() {
        if module_name.ends_with(name) || module_name.ends_with(&format!("::{}", name)) {
            return Some(module_name.clone());
        }
    }

    None
}

/// Format strength counts for display
fn format_strengths(strengths: &[StrengthCount]) -> String {
    if strengths.is_empty() {
        return "unknown".to_string();
    }
    if strengths.len() == 1 && strengths[0].count == 1 {
        return strengths[0].strength.clone();
    }
    strengths
        .iter()
        .map(|s| {
            if s.count == 1 {
                s.strength.clone()
            } else {
                format!("{}x {}", s.count, s.strength)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Generate impact analysis output
pub fn generate_impact_output<W: Write>(
    metrics: &ProjectMetrics,
    module_name: &str,
    writer: &mut W,
) -> io::Result<bool> {
    let analysis = match analyze_impact(metrics, module_name) {
        Some(a) => a,
        None => {
            writeln!(writer, "❌ Module '{}' not found.", module_name)?;
            writeln!(writer)?;
            writeln!(writer, "Available modules:")?;
            for (i, name) in metrics.modules.keys().take(10).enumerate() {
                writeln!(writer, "  {}. {}", i + 1, name)?;
            }
            if metrics.modules.len() > 10 {
                writeln!(writer, "  ... and {} more", metrics.modules.len() - 10)?;
            }
            return Ok(false);
        }
    };

    writeln!(writer, "Impact Analysis: {}", analysis.module)?;
    writeln!(
        writer,
        "═══════════════════════════════════════════════════════════"
    )?;

    // Risk score with visual indicator
    let risk_icon = match analysis.risk_level.as_str() {
        "HIGH" => "🔴",
        "MEDIUM" => "🟡",
        _ => "🟢",
    };
    writeln!(
        writer,
        "Risk Score: {} {} ({}/100)",
        risk_icon, analysis.risk_level, analysis.risk_score
    )?;

    if analysis.in_cycle {
        writeln!(writer, "⚠️  Part of a circular dependency cycle")?;
    }

    writeln!(writer)?;

    // Dependencies - count total couplings
    let total_dep_couplings: usize = analysis.dependencies.iter().map(|d| d.total_count).sum();
    writeln!(
        writer,
        "Direct Dependencies ({} modules, {} couplings):",
        analysis.dependencies.len(),
        total_dep_couplings
    )?;
    if analysis.dependencies.is_empty() {
        writeln!(writer, "  (none)")?;
    } else {
        for dep in &analysis.dependencies {
            let strengths_str = format_strengths(&dep.strengths);
            writeln!(
                writer,
                "  → {} ({}, {})",
                dep.module, strengths_str, dep.distance
            )?;
        }
    }

    writeln!(writer)?;

    // Dependents - count total couplings
    let total_dependent_couplings: usize = analysis.dependents.iter().map(|d| d.total_count).sum();
    writeln!(
        writer,
        "Direct Dependents ({} modules, {} couplings):",
        analysis.dependents.len(),
        total_dependent_couplings
    )?;
    if analysis.dependents.is_empty() {
        writeln!(writer, "  (none)")?;
    } else {
        for dep in &analysis.dependents {
            let strengths_str = format_strengths(&dep.strengths);
            writeln!(writer, "  ← {} ({})", dep.module, strengths_str)?;
        }
    }

    writeln!(writer)?;

    // Cascading impact
    writeln!(writer, "Cascading Impact:")?;
    writeln!(
        writer,
        "  Total affected: {} modules ({:.1}% of codebase)",
        analysis.cascading_impact.total_affected, analysis.cascading_impact.percentage
    )?;

    if !analysis.cascading_impact.second_order.is_empty() {
        writeln!(writer, "  2nd-order affected:")?;
        for module in analysis.cascading_impact.second_order.iter().take(5) {
            writeln!(writer, "    - {}", module)?;
        }
        if analysis.cascading_impact.second_order.len() > 5 {
            writeln!(
                writer,
                "    ... and {} more",
                analysis.cascading_impact.second_order.len() - 5
            )?;
        }
    }

    Ok(true)
}

// ============================================================================
// Check/Gate: CI/CD Quality Gate
// ============================================================================

/// Quality check configuration
#[derive(Debug, Clone)]
pub struct CheckConfig {
    /// Minimum acceptable grade (A, B, C, D, F)
    pub min_grade: Option<HealthGrade>,
    /// Maximum allowed critical issues
    pub max_critical: Option<usize>,
    /// Maximum allowed circular dependencies
    pub max_circular: Option<usize>,
    /// Fail on any issue of this severity or higher
    pub fail_on: Option<Severity>,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            min_grade: Some(HealthGrade::C),
            max_critical: Some(0),
            max_circular: Some(0),
            fail_on: None,
        }
    }
}

/// Check result with details
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub passed: bool,
    pub grade: String,
    pub score: f64,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub circular_count: usize,
    pub failures: Vec<String>,
}

/// Run quality check and return result
pub fn run_check(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    config: &CheckConfig,
) -> CheckResult {
    let report = analyze_project_balance_with_thresholds(metrics, thresholds);
    let circular_deps = metrics.detect_circular_dependencies();

    let critical_count = *report
        .issues_by_severity
        .get(&Severity::Critical)
        .unwrap_or(&0);
    let high_count = *report.issues_by_severity.get(&Severity::High).unwrap_or(&0);
    let medium_count = *report
        .issues_by_severity
        .get(&Severity::Medium)
        .unwrap_or(&0);
    let circular_count = circular_deps.len();

    let mut failures: Vec<String> = Vec::new();
    let mut passed = true;

    // Check minimum grade
    if let Some(min_grade) = &config.min_grade {
        // Note: S is treated as equal to A for comparison purposes
        // (S is a warning about over-optimization, not a higher grade)
        let grade_order = |g: &HealthGrade| match g {
            HealthGrade::S => 5, // Same as A
            HealthGrade::A => 5,
            HealthGrade::B => 4,
            HealthGrade::C => 3,
            HealthGrade::D => 2,
            HealthGrade::F => 1,
        };
        if grade_order(&report.health_grade) < grade_order(min_grade) {
            passed = false;
            failures.push(format!(
                "Grade {:?} is below minimum {:?}",
                report.health_grade, min_grade
            ));
        }
    }

    // Check critical issues
    if let Some(max) = config.max_critical
        && critical_count > max
    {
        passed = false;
        failures.push(format!("{} critical issues (max: {})", critical_count, max));
    }

    // Check circular dependencies
    if let Some(max) = config.max_circular
        && circular_count > max
    {
        passed = false;
        failures.push(format!(
            "{} circular dependencies (max: {})",
            circular_count, max
        ));
    }

    // Check fail_on severity
    if let Some(fail_severity) = &config.fail_on {
        let count = match fail_severity {
            Severity::Critical => critical_count,
            Severity::High => critical_count + high_count,
            Severity::Medium => critical_count + high_count + medium_count,
            Severity::Low => report.issues.len(),
        };
        if count > 0 {
            passed = false;
            failures.push(format!(
                "{} issues at {:?} severity or higher",
                count, fail_severity
            ));
        }
    }

    CheckResult {
        passed,
        grade: report.health_grade.letter().to_string(),
        score: report.average_score,
        critical_count,
        high_count,
        medium_count,
        circular_count,
        failures,
    }
}

/// Generate check output and return exit code (0 = pass, 1 = fail)
pub fn generate_check_output<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    config: &CheckConfig,
    writer: &mut W,
) -> io::Result<i32> {
    let result = run_check(metrics, thresholds, config);

    writeln!(writer, "Coupling Quality Gate")?;
    writeln!(
        writer,
        "═══════════════════════════════════════════════════════════"
    )?;

    let status = if result.passed {
        "✅ PASSED"
    } else {
        "❌ FAILED"
    };
    writeln!(
        writer,
        "Grade: {} ({:.0}%)  {}",
        result.grade,
        result.score * 100.0,
        status
    )?;

    writeln!(writer)?;
    writeln!(writer, "Metrics:")?;
    writeln!(writer, "  Critical issues: {}", result.critical_count)?;
    writeln!(writer, "  High issues: {}", result.high_count)?;
    writeln!(writer, "  Medium issues: {}", result.medium_count)?;
    writeln!(writer, "  Circular dependencies: {}", result.circular_count)?;

    if !result.passed {
        writeln!(writer)?;
        writeln!(writer, "Blocking Issues:")?;
        for failure in &result.failures {
            writeln!(writer, "  - {}", failure)?;
        }
    }

    Ok(if result.passed { 0 } else { 1 })
}

/// Generate a readable baseline diff report.
pub fn generate_baseline_diff_output<W: Write>(
    diff: &BaselineDiff,
    baseline_ref: &str,
    writer: &mut W,
) -> io::Result<()> {
    writeln!(writer, "Coupling Baseline Diff")?;
    writeln!(
        writer,
        "═══════════════════════════════════════════════════════════"
    )?;
    writeln!(writer, "Baseline: {}", baseline_ref)?;
    writeln!(
        writer,
        "Grade: {} -> {}",
        diff.baseline_grade.letter(),
        diff.current_grade.letter()
    )?;
    writeln!(writer, "Score delta: {:+.3}", diff.score_delta)?;
    writeln!(writer)?;
    writeln!(writer, "Issues:")?;
    writeln!(writer, "  New: {}", diff.new_issues.len())?;
    writeln!(writer, "  Resolved: {}", diff.resolved_issues.len())?;
    writeln!(writer, "  Unchanged: {}", diff.unchanged)?;

    write_issue_section(writer, "New Issues", &diff.new_issues)?;
    write_issue_section(writer, "Resolved Issues", &diff.resolved_issues)?;

    Ok(())
}

/// Generate ratchet gate output and return exit code (0 = pass, 1 = fail).
pub fn generate_ratchet_check_output<W: Write>(
    diff: &BaselineDiff,
    baseline_ref: &str,
    fail_on: Severity,
    writer: &mut W,
) -> io::Result<i32> {
    let failures = diff.ratchet_failures(fail_on);
    let passed = failures.is_empty();

    writeln!(writer, "Coupling Ratchet Gate")?;
    writeln!(
        writer,
        "═══════════════════════════════════════════════════════════"
    )?;
    writeln!(writer, "Baseline: {}", baseline_ref)?;
    writeln!(
        writer,
        "Grade: {} -> {}",
        diff.baseline_grade.letter(),
        diff.current_grade.letter()
    )?;
    writeln!(writer, "Score delta: {:+.3}", diff.score_delta)?;
    writeln!(
        writer,
        "New issues: {} (fail-on: {} or higher)",
        diff.new_issues.len(),
        fail_on
    )?;
    writeln!(
        writer,
        "Status: {}",
        if passed { "PASSED" } else { "FAILED" }
    )?;

    if !passed {
        writeln!(writer)?;
        writeln!(writer, "Blocking New Issues:")?;
        for issue in failures {
            write_issue_line(writer, issue)?;
        }
    }

    Ok(if passed { 0 } else { 1 })
}

fn write_issue_section<W: Write>(
    writer: &mut W,
    title: &str,
    issues: &[crate::balance::CouplingIssue],
) -> io::Result<()> {
    writeln!(writer)?;
    writeln!(writer, "{}:", title)?;
    if issues.is_empty() {
        writeln!(writer, "  (none)")?;
        return Ok(());
    }

    for issue in issues {
        write_issue_line(writer, issue)?;
    }
    Ok(())
}

fn write_issue_line<W: Write>(
    writer: &mut W,
    issue: &crate::balance::CouplingIssue,
) -> io::Result<()> {
    writeln!(
        writer,
        "  - {} {}: {} -> {}",
        issue.severity, issue.issue_type, issue.source, issue.target
    )
}

// ============================================================================
// External Dependencies: Third-party coupling exposure
// ============================================================================

/// Render external dependency coupling as text or JSON.
pub fn generate_external_dependencies_output<W: Write>(
    report: &ExternalDependencyReport,
    json: bool,
    writer: &mut W,
) -> io::Result<()> {
    if json {
        let output = JsonExternalDependenciesOutput {
            external_dependencies: json_external_dependencies(report),
        };
        let text = serde_json::to_string_pretty(&output).map_err(io::Error::other)?;
        writeln!(writer, "{}", text)?;
        return Ok(());
    }

    writeln!(writer, "External Dependency Coupling")?;
    writeln!(
        writer,
        "═══════════════════════════════════════════════════════════"
    )?;
    writeln!(
        writer,
        "External crates: {}  Direct references: {}",
        report.dependencies.len(),
        report
            .dependencies
            .iter()
            .map(|dependency| dependency.total_references)
            .sum::<usize>()
    )?;

    if report.dependencies.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "No external crate couplings detected.")?;
        return Ok(());
    }

    writeln!(writer)?;
    writeln!(writer, "Top Crates by Breadth:")?;
    for (index, dependency) in report.dependencies.iter().take(10).enumerate() {
        let version = if dependency.versions.is_empty() {
            "version: unknown".to_string()
        } else {
            format!("version: {}", dependency.versions.join(", "))
        };
        writeln!(
            writer,
            "{}. {} ({}; {} modules, {} references, dominant: {})",
            index + 1,
            dependency.crate_name,
            version,
            dependency.breadth,
            dependency.total_references,
            dependency.dominant_strength
        )?;
        let sample_modules = dependency
            .source_modules
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        if !sample_modules.is_empty() {
            writeln!(writer, "   modules: {}", sample_modules)?;
        }
    }

    writeln!(writer)?;
    writeln!(writer, "Scattered Coupling Flags:")?;
    if report.scattered_couplings.is_empty() {
        writeln!(writer, "  (none)")?;
    } else {
        for issue in &report.scattered_couplings {
            writeln!(
                writer,
                "  - {}: {} -> {}",
                issue.severity, issue.source, issue.target
            )?;
            writeln!(writer, "    {}", issue.description)?;
            writeln!(writer, "    Fix: {}", issue.refactoring)?;
        }
    }

    Ok(())
}

// ============================================================================
// JSON Output
// ============================================================================

/// Temporal coupling in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonTemporalCoupling {
    pub file_a: String,
    pub file_b: String,
    pub co_change_count: usize,
    pub coupling_ratio: f64,
    pub is_strong: bool,
}

/// Complete analysis in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonOutput {
    pub summary: JsonSummary,
    pub grade_rationale: JsonGradeRationale,
    pub analysis_manifest: JsonAnalysisManifest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<JsonBaselineDiff>,
    pub external_dependencies: JsonExternalDependencies,
    pub hotspots: Vec<Hotspot>,
    pub issues: Vec<JsonIssue>,
    pub circular_dependencies: Vec<Vec<String>>,
    pub temporal_couplings: Vec<JsonTemporalCoupling>,
    pub modules: Vec<JsonModule>,
}

/// Standalone external-dependency JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct JsonExternalDependenciesOutput {
    pub external_dependencies: JsonExternalDependencies,
}

/// External dependency analysis in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonExternalDependencies {
    pub total_crates: usize,
    pub total_references: usize,
    pub dependencies: Vec<ExternalDependencyUsage>,
    pub scattered_couplings: Vec<JsonIssue>,
}

/// Summary in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonSummary {
    pub health_grade: String,
    pub health_score: f64,
    pub total_modules: usize,
    pub total_couplings: usize,
    pub internal_couplings: usize,
    pub external_couplings: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub medium_issues: usize,
}

/// Health-grade rationale in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonGradeRationale {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dominant_dimension: Option<String>,
    pub top_issue_types: Vec<JsonIssueTypeContribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Issue-type contribution in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonIssueTypeContribution {
    pub issue_type: String,
    pub count: usize,
    pub highest_severity: String,
}

/// Declared analysis blind spots in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonAnalysisManifest {
    pub blind_spots: Vec<JsonBlindSpot>,
    pub notes: Vec<String>,
}

/// Structural blind spot in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonBlindSpot {
    pub area: String,
    pub description: String,
}

/// Issue in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonIssue {
    pub issue_type: String,
    pub severity: String,
    pub source: String,
    pub target: String,
    pub description: String,
    pub suggestion: String,
    pub balance_score: f64,
}

/// Module in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonModule {
    pub name: String,
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomain: Option<String>,
    pub couplings_out: usize,
    pub couplings_in: usize,
    pub balance_score: f64,
    pub in_cycle: bool,
}

/// Baseline diff in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonBaselineDiff {
    pub new_issues: Vec<JsonIssue>,
    pub resolved_issues: Vec<JsonIssue>,
    pub unchanged: usize,
    pub score_delta: f64,
    pub grade_change: JsonGradeChange,
}

/// Baseline/current grade transition in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonGradeChange {
    pub baseline: String,
    pub current: String,
}

/// Generate complete JSON output
pub fn generate_json_output<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    writer: &mut W,
) -> io::Result<()> {
    generate_json_output_with_optional_diff(metrics, thresholds, manifest, None, writer)
}

/// Generate complete JSON output with a top-level baseline diff object.
pub fn generate_json_output_with_diff<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    diff: &BaselineDiff,
    writer: &mut W,
) -> io::Result<()> {
    generate_json_output_with_optional_diff(metrics, thresholds, manifest, Some(diff), writer)
}

fn generate_json_output_with_optional_diff<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    diff: Option<&BaselineDiff>,
    writer: &mut W,
) -> io::Result<()> {
    let report = analyze_project_balance_with_thresholds(metrics, thresholds);
    let external_dependencies = analyze_external_dependencies(metrics, &HashMap::new());
    let circular_deps = metrics.detect_circular_dependencies();
    let cycle_modules: HashSet<String> = circular_deps.iter().flatten().cloned().collect();
    let hotspots = calculate_hotspots(metrics, thresholds, 10);

    // Count couplings per module
    let mut couplings_out: HashMap<String, usize> = HashMap::new();
    let mut couplings_in: HashMap<String, usize> = HashMap::new();
    let mut balance_scores: HashMap<String, Vec<f64>> = HashMap::new();
    let mut internal_count = 0;

    for coupling in &metrics.couplings {
        if coupling.distance != Distance::DifferentCrate {
            internal_count += 1;
            *couplings_out.entry(coupling.source.clone()).or_default() += 1;
            *couplings_in.entry(coupling.target.clone()).or_default() += 1;
            let score = BalanceScore::calculate(coupling);
            balance_scores
                .entry(coupling.source.clone())
                .or_default()
                .push(score.score);
        }
    }

    let external_count = metrics.couplings.len() - internal_count;

    let critical = *report
        .issues_by_severity
        .get(&Severity::Critical)
        .unwrap_or(&0);
    let high = *report.issues_by_severity.get(&Severity::High).unwrap_or(&0);
    let medium = *report
        .issues_by_severity
        .get(&Severity::Medium)
        .unwrap_or(&0);

    let temporal_couplings: Vec<JsonTemporalCoupling> = metrics
        .temporal_couplings
        .iter()
        .take(20)
        .map(|tc| JsonTemporalCoupling {
            file_a: tc.file_a.clone(),
            file_b: tc.file_b.clone(),
            co_change_count: tc.co_change_count,
            coupling_ratio: tc.coupling_ratio,
            is_strong: tc.is_strong(),
        })
        .collect();

    let output = JsonOutput {
        summary: JsonSummary {
            health_grade: report.health_grade.letter().to_string(),
            health_score: report.average_score,
            total_modules: metrics.modules.len(),
            total_couplings: metrics.couplings.len(),
            internal_couplings: internal_count,
            external_couplings: external_count,
            critical_issues: critical,
            high_issues: high,
            medium_issues: medium,
        },
        grade_rationale: JsonGradeRationale {
            summary: report.grade_rationale.summary.clone(),
            dominant_dimension: report
                .grade_rationale
                .dominant_dimension
                .map(|dimension| dimension.to_string()),
            top_issue_types: report
                .grade_rationale
                .top_issue_types
                .iter()
                .map(|item| JsonIssueTypeContribution {
                    issue_type: item.issue_type.to_string(),
                    count: item.count,
                    highest_severity: item.highest_severity.to_string(),
                })
                .collect(),
            note: report.grade_rationale.volatility_note.clone(),
        },
        analysis_manifest: JsonAnalysisManifest {
            blind_spots: manifest
                .blind_spots
                .iter()
                .map(|blind_spot| JsonBlindSpot {
                    area: blind_spot.area.to_string(),
                    description: blind_spot.description.to_string(),
                })
                .collect(),
            notes: manifest.notes.clone(),
        },
        diff: diff.map(json_baseline_diff),
        external_dependencies: json_external_dependencies(&external_dependencies),
        hotspots,
        issues: report.issues.iter().map(json_issue).collect(),
        circular_dependencies: circular_deps,
        temporal_couplings,
        modules: metrics
            .modules
            .iter()
            .map(|(name, module)| {
                let avg_score = balance_scores
                    .get(name)
                    .map(|scores| scores.iter().sum::<f64>() / scores.len() as f64)
                    .unwrap_or(1.0);
                JsonModule {
                    name: name.clone(),
                    file_path: Some(module.path.display().to_string()),
                    subdomain: module.subdomain.map(|subdomain| subdomain.to_string()),
                    couplings_out: couplings_out.get(name).copied().unwrap_or(0),
                    couplings_in: couplings_in.get(name).copied().unwrap_or(0),
                    balance_score: avg_score,
                    in_cycle: cycle_modules.contains(name),
                }
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&output).map_err(io::Error::other)?;
    writeln!(writer, "{}", json)?;

    Ok(())
}

fn json_baseline_diff(diff: &BaselineDiff) -> JsonBaselineDiff {
    JsonBaselineDiff {
        new_issues: diff.new_issues.iter().map(json_issue).collect(),
        resolved_issues: diff.resolved_issues.iter().map(json_issue).collect(),
        unchanged: diff.unchanged,
        score_delta: diff.score_delta,
        grade_change: JsonGradeChange {
            baseline: diff.baseline_grade.letter().to_string(),
            current: diff.current_grade.letter().to_string(),
        },
    }
}

fn json_issue(issue: &crate::balance::CouplingIssue) -> JsonIssue {
    JsonIssue {
        issue_type: format!("{}", issue.issue_type),
        severity: format!("{}", issue.severity),
        source: issue.source.clone(),
        target: issue.target.clone(),
        description: issue.description.clone(),
        suggestion: format!("{}", issue.refactoring),
        balance_score: issue.balance_score,
    }
}

fn json_external_dependencies(report: &ExternalDependencyReport) -> JsonExternalDependencies {
    JsonExternalDependencies {
        total_crates: report.dependencies.len(),
        total_references: report
            .dependencies
            .iter()
            .map(|dependency| dependency.total_references)
            .sum(),
        dependencies: report.dependencies.clone(),
        scattered_couplings: report.scattered_couplings.iter().map(json_issue).collect(),
    }
}

// ============================================================================
// Parse helpers for CLI
// ============================================================================

/// Parse grade string to HealthGrade
pub fn parse_grade(s: &str) -> Option<HealthGrade> {
    match s.to_uppercase().as_str() {
        "S" => Some(HealthGrade::S),
        "A" => Some(HealthGrade::A),
        "B" => Some(HealthGrade::B),
        "C" => Some(HealthGrade::C),
        "D" => Some(HealthGrade::D),
        "F" => Some(HealthGrade::F),
        _ => None,
    }
}

/// Parse severity string to Severity
pub fn parse_severity(s: &str) -> Option<Severity> {
    match s.to_lowercase().as_str() {
        "critical" => Some(Severity::Critical),
        "high" => Some(Severity::High),
        "medium" => Some(Severity::Medium),
        "low" => Some(Severity::Low),
        _ => None,
    }
}

// ============================================================================
// Trace: Function/Type-level Dependency Analysis
// ============================================================================

/// Trace result for a specific item (function/type)
#[derive(Debug, Clone)]
pub struct TraceResult {
    /// Item name
    pub item_name: String,
    /// Module where the item is defined
    pub module: String,
    /// File path
    pub file_path: String,
    /// What this item depends on (outgoing)
    pub depends_on: Vec<TraceDependency>,
    /// What depends on this item (incoming)
    pub depended_by: Vec<TraceDependency>,
    /// Design recommendation based on coupling analysis
    pub recommendation: Option<String>,
}

/// A traced dependency
#[derive(Debug, Clone)]
pub struct TraceDependency {
    /// Source or target item name
    pub item: String,
    /// Module name
    pub module: String,
    /// Type of dependency (FunctionCall, FieldAccess, etc.)
    pub dep_type: String,
    /// Integration strength
    pub strength: String,
    /// File path
    pub file_path: Option<String>,
    /// Line number
    pub line: usize,
}

/// Generate trace output for a specific function/type
pub fn generate_trace_output<W: Write>(
    metrics: &ProjectMetrics,
    item_name: &str,
    writer: &mut W,
) -> io::Result<bool> {
    use crate::analyzer::ItemDepType;

    // Find all items matching the name
    let mut found_in_modules: Vec<(&str, &crate::metrics::ModuleMetrics)> = Vec::new();
    let mut outgoing: Vec<TraceDependency> = Vec::new();
    let mut incoming: Vec<TraceDependency> = Vec::new();

    // Search through all modules
    for (module_name, module) in &metrics.modules {
        // Check if this module defines the item (as function or type)
        let defines_function = module.function_definitions.contains_key(item_name);
        let defines_type = module.type_definitions.contains_key(item_name);

        if defines_function || defines_type {
            found_in_modules.push((module_name, module));
        }

        // Check item_dependencies for outgoing dependencies FROM this item
        for dep in &module.item_dependencies {
            if dep.source_item.contains(item_name) || dep.source_item.ends_with(item_name) {
                let strength = match dep.dep_type {
                    ItemDepType::FieldAccess | ItemDepType::StructConstruction => "Intrusive",
                    ItemDepType::FunctionCall | ItemDepType::MethodCall => "Functional",
                    ItemDepType::TypeUsage | ItemDepType::Import => "Model",
                    ItemDepType::TraitImpl | ItemDepType::TraitBound => "Contract",
                };
                outgoing.push(TraceDependency {
                    item: dep.target.clone(),
                    module: dep
                        .target_module
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    dep_type: format!("{:?}", dep.dep_type),
                    strength: strength.to_string(),
                    file_path: Some(module.path.display().to_string()),
                    line: dep.line,
                });
            }

            // Check for incoming dependencies TO this item
            if dep.target.contains(item_name) || dep.target.ends_with(item_name) {
                let strength = match dep.dep_type {
                    ItemDepType::FieldAccess | ItemDepType::StructConstruction => "Intrusive",
                    ItemDepType::FunctionCall | ItemDepType::MethodCall => "Functional",
                    ItemDepType::TypeUsage | ItemDepType::Import => "Model",
                    ItemDepType::TraitImpl | ItemDepType::TraitBound => "Contract",
                };
                incoming.push(TraceDependency {
                    item: dep.source_item.clone(),
                    module: module_name.clone(),
                    dep_type: format!("{:?}", dep.dep_type),
                    strength: strength.to_string(),
                    file_path: Some(module.path.display().to_string()),
                    line: dep.line,
                });
            }
        }
    }

    // If not found, try partial match
    if found_in_modules.is_empty() && outgoing.is_empty() && incoming.is_empty() {
        writeln!(writer, "Item '{}' not found.", item_name)?;
        writeln!(writer)?;
        writeln!(
            writer,
            "Hint: Try searching with a partial name or check module names:"
        )?;

        // Show available items that might match
        let mut suggestions: Vec<String> = Vec::new();
        for (module_name, module) in &metrics.modules {
            for func_name in module.function_definitions.keys() {
                if func_name.to_lowercase().contains(&item_name.to_lowercase()) {
                    suggestions.push(format!("  - {} (function in {})", func_name, module_name));
                }
            }
            for type_name in module.type_definitions.keys() {
                if type_name.to_lowercase().contains(&item_name.to_lowercase()) {
                    suggestions.push(format!("  - {} (type in {})", type_name, module_name));
                }
            }
        }

        if suggestions.is_empty() {
            writeln!(writer, "  No similar items found.")?;
        } else {
            for s in suggestions.iter().take(10) {
                writeln!(writer, "{}", s)?;
            }
            if suggestions.len() > 10 {
                writeln!(writer, "  ... and {} more", suggestions.len() - 10)?;
            }
        }

        return Ok(false);
    }

    // Output header
    writeln!(writer, "Dependency Trace: {}", item_name)?;
    writeln!(writer, "{}", "═".repeat(50))?;
    writeln!(writer)?;

    // Show where the item is defined
    if !found_in_modules.is_empty() {
        writeln!(writer, "📍 Defined in:")?;
        for (module_name, module) in &found_in_modules {
            let item_type = if module.function_definitions.contains_key(item_name) {
                "function"
            } else {
                "type"
            };
            writeln!(
                writer,
                "   {} ({}) - {}",
                module_name,
                item_type,
                module.path.display()
            )?;
        }
        writeln!(writer)?;
    }

    // Show outgoing dependencies (what this item depends on)
    writeln!(writer, "📤 Depends on ({} items):", outgoing.len())?;
    if outgoing.is_empty() {
        writeln!(writer, "   (none)")?;
    } else {
        // Group by target
        let mut by_target: HashMap<String, Vec<&TraceDependency>> = HashMap::new();
        for dep in &outgoing {
            by_target.entry(dep.item.clone()).or_default().push(dep);
        }

        for (target, deps) in by_target.iter().take(15) {
            let first = deps[0];
            let strength_icon = match first.strength.as_str() {
                "Intrusive" => "🔴",
                "Functional" => "🟠",
                "Model" => "🟡",
                "Contract" => "🟢",
                _ => "⚪",
            };
            writeln!(
                writer,
                "   {} {} ({}) - line {}",
                strength_icon, target, first.strength, first.line
            )?;
        }
        if by_target.len() > 15 {
            writeln!(writer, "   ... and {} more", by_target.len() - 15)?;
        }
    }
    writeln!(writer)?;

    // Show incoming dependencies (what depends on this item)
    writeln!(writer, "📥 Depended by ({} items):", incoming.len())?;
    if incoming.is_empty() {
        writeln!(writer, "   (none)")?;
    } else {
        // Group by source
        let mut by_source: HashMap<String, Vec<&TraceDependency>> = HashMap::new();
        for dep in &incoming {
            by_source.entry(dep.item.clone()).or_default().push(dep);
        }

        for (source, deps) in by_source.iter().take(15) {
            let first = deps[0];
            let strength_icon = match first.strength.as_str() {
                "Intrusive" => "🔴",
                "Functional" => "🟠",
                "Model" => "🟡",
                "Contract" => "🟢",
                _ => "⚪",
            };
            writeln!(
                writer,
                "   {} {} ({}) - {}:{}",
                strength_icon,
                source,
                first.strength,
                first.file_path.as_deref().unwrap_or("?"),
                first.line
            )?;
        }
        if by_source.len() > 15 {
            writeln!(writer, "   ... and {} more", by_source.len() - 15)?;
        }
    }
    writeln!(writer)?;

    // Design recommendation
    writeln!(writer, "💡 Design Analysis:")?;

    let intrusive_out = outgoing
        .iter()
        .filter(|d| d.strength == "Intrusive")
        .count();
    let intrusive_in = incoming
        .iter()
        .filter(|d| d.strength == "Intrusive")
        .count();
    let total_deps = outgoing.len() + incoming.len();

    if total_deps == 0 {
        writeln!(writer, "   ✅ This item has no tracked dependencies.")?;
    } else if intrusive_out > 3 {
        writeln!(
            writer,
            "   ⚠️  High intrusive outgoing coupling ({} items)",
            intrusive_out
        )?;
        writeln!(
            writer,
            "   → Consider: Extract interface/trait to reduce direct access"
        )?;
        writeln!(
            writer,
            "   → Khononov: Strong coupling should be CLOSE (same module)"
        )?;
    } else if intrusive_in > 5 {
        writeln!(
            writer,
            "   ⚠️  High intrusive incoming coupling ({} items depend on internals)",
            intrusive_in
        )?;
        writeln!(
            writer,
            "   → Consider: This item is a hotspot - changes will cascade"
        )?;
        writeln!(
            writer,
            "   → Khononov: Add stable interface to protect dependents"
        )?;
    } else if outgoing.len() > 10 {
        writeln!(
            writer,
            "   ⚠️  High efferent coupling ({} dependencies)",
            outgoing.len()
        )?;
        writeln!(
            writer,
            "   → Consider: Split into smaller functions with focused responsibilities"
        )?;
    } else if incoming.len() > 10 {
        writeln!(
            writer,
            "   ⚠️  High afferent coupling ({} dependents)",
            incoming.len()
        )?;
        writeln!(
            writer,
            "   → Consider: This is a core component - keep it stable"
        )?;
    } else {
        writeln!(writer, "   ✅ Coupling appears balanced.")?;
    }

    writeln!(writer)?;

    // Change impact summary
    writeln!(writer, "🔄 Change Impact:")?;
    writeln!(
        writer,
        "   If you modify '{}', you may need to update:",
        item_name
    )?;
    let affected_modules: HashSet<_> = incoming.iter().map(|d| d.module.clone()).collect();
    if affected_modules.is_empty() {
        writeln!(writer, "   (no other modules directly affected)")?;
    } else {
        for module in affected_modules.iter().take(10) {
            writeln!(writer, "   • {}", module)?;
        }
        if affected_modules.len() > 10 {
            writeln!(
                writer,
                "   ... and {} more modules",
                affected_modules.len() - 10
            )?;
        }
    }

    Ok(true)
}

// ============================================================================
// History: Time-Series Coupling Health
// ============================================================================

/// A single timeline point in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonHistoryPoint {
    pub commit: String,
    pub date: String,
    pub grade: char,
    pub average_score: f64,
    pub total_couplings: usize,
    pub module_count: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
}

/// A skipped revision in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonSkippedRevision {
    pub commit: String,
    pub date: String,
    pub reason: String,
}

/// Complete history timeline in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonHistory {
    pub months: usize,
    pub points: Vec<JsonHistoryPoint>,
    pub skipped: Vec<JsonSkippedRevision>,
}

/// Convert a history report into its shared JSON representation.
pub fn history_report_to_json(report: &HistoryReport) -> JsonHistory {
    JsonHistory {
        months: report.months,
        points: report
            .points
            .iter()
            .map(|p| JsonHistoryPoint {
                commit: p.commit.clone(),
                date: p.date.clone(),
                grade: p.grade.letter(),
                average_score: p.average_score,
                total_couplings: p.total_couplings,
                module_count: p.module_count,
                critical_issues: p.critical,
                high_issues: p.high,
            })
            .collect(),
        skipped: report
            .skipped
            .iter()
            .map(|s| JsonSkippedRevision {
                commit: s.commit.clone(),
                date: s.date.clone(),
                reason: s.reason.clone(),
            })
            .collect(),
    }
}

/// Render a history report as text or JSON.
pub fn generate_history_output<W: Write>(
    report: &HistoryReport,
    json: bool,
    requested_samples: usize,
    writer: &mut W,
) -> io::Result<()> {
    if json {
        let output = history_report_to_json(report);
        let text = serde_json::to_string_pretty(&output).map_err(io::Error::other)?;
        writeln!(writer, "{}", text)?;
        return Ok(());
    }

    writeln!(
        writer,
        "Coupling History (last {} months, {} sample(s))\n",
        report.months,
        report.points.len()
    )?;

    if report.points.is_empty() {
        writeln!(writer, "  No analyzable revisions in the requested window.")?;
    } else {
        writeln!(
            writer,
            "  date        commit   grade  avg     couplings  critical"
        )?;
        for p in &report.points {
            writeln!(
                writer,
                "  {:<11} {:<8} {:<6} {:<7.3} {:<10} {}",
                p.date,
                p.commit,
                p.grade.letter(),
                p.average_score,
                p.total_couplings,
                p.critical,
            )?;
        }

        if let Some((first, last)) = report.endpoints() {
            let direction = describe_trend(first.average_score, last.average_score);
            writeln!(
                writer,
                "\nTrend: grade {} -> {}, avg {:.3} -> {:.3} ({})",
                first.grade.letter(),
                last.grade.letter(),
                first.average_score,
                last.average_score,
                direction,
            )?;
        }
    }

    if report.points.len() < requested_samples {
        writeln!(
            writer,
            "\nNote: {} of {} requested samples (history/window-limited).",
            report.points.len(),
            requested_samples
        )?;
    }

    if !report.skipped.is_empty() {
        writeln!(writer, "\nSkipped {} revision(s):", report.skipped.len())?;
        for s in &report.skipped {
            writeln!(writer, "  {} ({}): {}", s.commit, s.date, s.reason)?;
        }
    }

    Ok(())
}

/// Describe the direction of change between two scores.
fn describe_trend(from: f64, to: f64) -> &'static str {
    let delta = to - from;
    if delta > 0.01 {
        "improving"
    } else if delta < -0.01 {
        "regressing"
    } else {
        "stable"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::history::{HistoryPoint, HistoryReport};
    use crate::manifest::{ManifestContext, build_manifest};

    fn sample_point(date: &str, grade: HealthGrade, score: f64) -> HistoryPoint {
        HistoryPoint {
            commit: "abc1234".to_string(),
            date: date.to_string(),
            grade,
            average_score: score,
            total_couplings: 100,
            module_count: 12,
            critical: 0,
            high: 1,
        }
    }

    #[test]
    fn test_describe_trend() {
        assert_eq!(describe_trend(0.70, 0.85), "improving");
        assert_eq!(describe_trend(0.85, 0.70), "regressing");
        assert_eq!(describe_trend(0.80, 0.805), "stable");
    }

    #[test]
    fn test_history_text_output_shows_trend() {
        let report = HistoryReport {
            months: 6,
            points: vec![
                sample_point("2026-01-01", HealthGrade::C, 0.60),
                sample_point("2026-05-01", HealthGrade::A, 0.85),
            ],
            skipped: vec![],
        };
        let mut buf = Vec::new();
        generate_history_output(&report, false, 2, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Coupling History (last 6 months, 2 sample(s))"));
        assert!(text.contains("Trend: grade C -> A"));
        assert!(text.contains("improving"));
    }

    #[test]
    fn test_history_json_output_is_valid() {
        let report = HistoryReport {
            months: 12,
            points: vec![sample_point("2026-05-01", HealthGrade::B, 0.75)],
            skipped: vec![],
        };
        let mut buf = Vec::new();
        generate_history_output(&report, true, 1, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["months"], 12);
        assert_eq!(parsed["points"][0]["grade"], "B");
        assert_eq!(parsed["points"][0]["module_count"], 12);
    }

    #[test]
    fn test_history_empty_output() {
        let report = HistoryReport {
            months: 6,
            points: vec![],
            skipped: vec![],
        };
        let mut buf = Vec::new();
        generate_history_output(&report, false, 0, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("No analyzable revisions"));
    }

    #[test]
    fn test_history_text_output_notes_when_requested_samples_are_limited() {
        let report = HistoryReport {
            months: 6,
            points: vec![sample_point("2026-05-01", HealthGrade::B, 0.75)],
            skipped: vec![],
        };
        let mut buf = Vec::new();
        generate_history_output(&report, false, 3, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Note: 1 of 3 requested samples (history/window-limited)."));
    }

    #[test]
    fn test_parse_grade() {
        assert_eq!(parse_grade("S"), Some(HealthGrade::S));
        assert_eq!(parse_grade("A"), Some(HealthGrade::A));
        assert_eq!(parse_grade("b"), Some(HealthGrade::B));
        assert_eq!(parse_grade("C"), Some(HealthGrade::C));
        assert_eq!(parse_grade("X"), None);
    }

    #[test]
    fn test_parse_severity() {
        assert_eq!(parse_severity("critical"), Some(Severity::Critical));
        assert_eq!(parse_severity("HIGH"), Some(Severity::High));
        assert_eq!(parse_severity("invalid"), None);
    }

    #[test]
    fn test_empty_metrics_hotspots() {
        let metrics = ProjectMetrics::new();
        let thresholds = IssueThresholds::default();
        let hotspots = calculate_hotspots(&metrics, &thresholds, 5);
        assert!(hotspots.is_empty());
    }

    #[test]
    fn test_external_dependencies_json_output_shape() {
        use crate::external::{ExternalDependencyReport, ExternalDependencyUsage};

        let dependencies = vec![ExternalDependencyUsage {
            crate_name: "reqwest".to_string(),
            versions: vec!["0.12.0".to_string()],
            breadth: 4,
            total_references: 8,
            dominant_strength: "Functional".to_string(),
            source_modules: vec![
                "api".to_string(),
                "client".to_string(),
                "sync".to_string(),
                "worker".to_string(),
            ],
        }];
        let scattered_couplings =
            crate::external::detect_scattered_external_coupling(&dependencies);
        let report = ExternalDependencyReport {
            dependencies,
            scattered_couplings,
        };
        let mut buf = Vec::new();

        generate_external_dependencies_output(&report, true, &mut buf).unwrap();

        let text = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        let deps = &parsed["external_dependencies"];
        assert_eq!(deps["total_crates"], 1);
        assert_eq!(deps["total_references"], 8);
        assert_eq!(deps["dependencies"][0]["crate_name"], "reqwest");
        assert_eq!(deps["dependencies"][0]["versions"][0], "0.12.0");
        assert_eq!(
            deps["scattered_couplings"][0]["issue_type"],
            "Scattered External Coupling"
        );
    }

    #[test]
    fn test_check_passes_on_empty() {
        let metrics = ProjectMetrics::new();
        let thresholds = IssueThresholds::default();
        let config = CheckConfig::default();
        let result = run_check(&metrics, &thresholds, &config);
        assert!(result.passed);
    }

    #[test]
    fn test_json_output_includes_analysis_manifest() {
        let metrics = ProjectMetrics::new();
        let thresholds = IssueThresholds::default();
        let manifest = build_manifest(&ManifestContext {
            git_used: false,
            tests_excluded: true,
            parse_failures: 0,
        });
        let mut buf = Vec::new();

        generate_json_output(&metrics, &thresholds, &manifest, &mut buf).unwrap();

        let text = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        let blind_spots = parsed["analysis_manifest"]["blind_spots"]
            .as_array()
            .unwrap();
        let notes = parsed["analysis_manifest"]["notes"].as_array().unwrap();

        assert!(blind_spots.iter().any(|spot| {
            spot["area"]
                .as_str()
                .is_some_and(|area| area == "dynamic-connascence")
        }));
        assert!(blind_spots.iter().any(|spot| {
            spot["area"]
                .as_str()
                .is_some_and(|area| area == "macro-and-cfg")
        }));
        assert!(notes.iter().any(|note| {
            note.as_str()
                .is_some_and(|note| note.contains("Git history was not analyzed"))
        }));
        assert!(notes.iter().any(|note| {
            note.as_str()
                .is_some_and(|note| note.contains("Test code was excluded"))
        }));
    }

    #[test]
    fn test_json_output_includes_module_subdomain_when_present() {
        use crate::config::Subdomain;
        use crate::metrics::ModuleMetrics;

        let mut metrics = ProjectMetrics::new();
        let mut module = ModuleMetrics::new(PathBuf::from("src/report.rs"), "report".to_string());
        module.subdomain = Some(Subdomain::Supporting);
        metrics.add_module(module);

        let thresholds = IssueThresholds::default();
        let manifest = build_manifest(&ManifestContext::default());
        let mut buf = Vec::new();

        generate_json_output(&metrics, &thresholds, &manifest, &mut buf).unwrap();

        let text = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        let modules = parsed["modules"].as_array().unwrap();
        assert!(modules.iter().any(|module| {
            module["name"] == "report" && module["subdomain"].as_str() == Some("Supporting")
        }));
    }

    #[test]
    fn test_json_output_includes_grade_rationale() {
        use crate::metrics::{CouplingMetrics, IntegrationStrength, Volatility};

        let mut metrics = ProjectMetrics::new();
        metrics.add_coupling(CouplingMetrics::new(
            "caller".to_string(),
            "stable".to_string(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        ));

        let thresholds = IssueThresholds::default();
        let manifest = build_manifest(&ManifestContext::default());
        let mut buf = Vec::new();

        generate_json_output(&metrics, &thresholds, &manifest, &mut buf).unwrap();

        let text = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        let rationale = &parsed["grade_rationale"];
        assert!(rationale["summary"].as_str().unwrap().contains("Driven by"));
        assert_eq!(
            rationale["top_issue_types"][0]["issue_type"].as_str(),
            Some("Cascading Change Risk")
        );
        assert!(rationale["note"].as_str().is_some());
    }
}
