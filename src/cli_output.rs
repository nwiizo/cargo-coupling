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
    hotspots.sort_by(|a, b| b.score.cmp(&a.score));
    hotspots.truncate(limit);

    hotspots
}

/// Generate hotspots output to writer
pub fn generate_hotspots_output<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    limit: usize,
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

        // Issues
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
        }

        // Suggestion
        writeln!(writer, "   → Fix: {}", hotspot.suggestion)?;
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

/// Information about a dependency relationship
#[derive(Debug, Clone, Serialize)]
pub struct DependencyInfo {
    pub module: String,
    pub strength: String,
    pub distance: String,
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

    // Collect direct dependencies and dependents
    let mut dependencies: Vec<DependencyInfo> = Vec::new();
    let mut dependents: Vec<DependencyInfo> = Vec::new();
    let mut volatility_max = crate::metrics::Volatility::Low;

    for coupling in &metrics.couplings {
        if coupling.distance == Distance::DifferentCrate {
            continue; // Skip external crates
        }

        if coupling.source == module {
            dependencies.push(DependencyInfo {
                module: coupling.target.clone(),
                strength: format!("{:?}", coupling.strength),
                distance: format!("{:?}", coupling.distance),
            });
        }

        if coupling.target == module {
            dependents.push(DependencyInfo {
                module: coupling.source.clone(),
                strength: format!("{:?}", coupling.strength),
                distance: format!("{:?}", coupling.distance),
            });
            // Track max volatility of incoming couplings
            if coupling.volatility > volatility_max {
                volatility_max = coupling.volatility;
            }
        }
    }

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

    // Dependencies
    writeln!(
        writer,
        "Direct Dependencies ({}):",
        analysis.dependencies.len()
    )?;
    if analysis.dependencies.is_empty() {
        writeln!(writer, "  (none)")?;
    } else {
        for dep in &analysis.dependencies {
            writeln!(
                writer,
                "  → {} ({}, {})",
                dep.module, dep.strength, dep.distance
            )?;
        }
    }

    writeln!(writer)?;

    // Dependents
    writeln!(writer, "Direct Dependents ({}):", analysis.dependents.len())?;
    if analysis.dependents.is_empty() {
        writeln!(writer, "  (none)")?;
    } else {
        for dep in &analysis.dependents {
            writeln!(writer, "  ← {} ({})", dep.module, dep.strength)?;
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
        let grade_order = |g: &HealthGrade| match g {
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
        grade: format!("{:?}", report.health_grade),
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

// ============================================================================
// JSON Output
// ============================================================================

/// Complete analysis in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonOutput {
    pub summary: JsonSummary,
    pub hotspots: Vec<Hotspot>,
    pub issues: Vec<JsonIssue>,
    pub circular_dependencies: Vec<Vec<String>>,
    pub modules: Vec<JsonModule>,
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
    pub couplings_out: usize,
    pub couplings_in: usize,
    pub balance_score: f64,
    pub in_cycle: bool,
}

/// Generate complete JSON output
pub fn generate_json_output<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    writer: &mut W,
) -> io::Result<()> {
    let report = analyze_project_balance_with_thresholds(metrics, thresholds);
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

    let output = JsonOutput {
        summary: JsonSummary {
            health_grade: format!("{:?}", report.health_grade),
            health_score: report.average_score,
            total_modules: metrics.modules.len(),
            total_couplings: metrics.couplings.len(),
            internal_couplings: internal_count,
            external_couplings: external_count,
            critical_issues: critical,
            high_issues: high,
            medium_issues: medium,
        },
        hotspots,
        issues: report
            .issues
            .iter()
            .map(|i| JsonIssue {
                issue_type: format!("{}", i.issue_type),
                severity: format!("{}", i.severity),
                source: i.source.clone(),
                target: i.target.clone(),
                description: i.description.clone(),
                suggestion: format!("{}", i.refactoring),
                balance_score: i.balance_score,
            })
            .collect(),
        circular_dependencies: circular_deps,
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

// ============================================================================
// Parse helpers for CLI
// ============================================================================

/// Parse grade string to HealthGrade
pub fn parse_grade(s: &str) -> Option<HealthGrade> {
    match s.to_uppercase().as_str() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_grade() {
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
    fn test_check_passes_on_empty() {
        let metrics = ProjectMetrics::new();
        let thresholds = IssueThresholds::default();
        let config = CheckConfig::default();
        let result = run_check(&metrics, &thresholds, &config);
        assert!(result.passed);
    }
}
