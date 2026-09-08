//! CLI output functions for job-focused commands
//!
//! Provides specialized output formats for different JTBD (Jobs to be Done):
//! - Hotspots: Quick identification of refactoring priorities
//! - Impact: Change impact analysis for a specific module
//! - Check: CI/CD quality gate with exit codes
//! - JSON: Machine-readable output for automation

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};

use serde::Serialize;

use crate::balance::grade::HealthGrade;
use crate::balance::issue::CouplingIssue;
use crate::balance::issue_type::IssueType;
use crate::balance::project::analyze_project_balance_with_thresholds;
use crate::balance::score::IssueThresholds;
use crate::balance::severity::Severity;
use crate::diff::BaselineDiff;
use crate::external::ExternalDependencyReport;
use crate::history::HistoryReport;
#[cfg(test)]
use crate::metrics::dimensions::Distance;
use crate::metrics::project::ProjectMetrics;

mod hotspots;
mod impact;
mod json;
pub use hotspots::{
    Hotspot, HotspotIssue, IssueExplanation, calculate_hotspots, generate_hotspots_output,
    get_issue_explanation,
};
pub use impact::{
    CascadingImpact, DependencyInfo, ImpactAnalysis, StrengthCount, analyze_impact,
    analyze_impact_with_depth, generate_impact_output, generate_impact_output_with_depth,
};
#[cfg(test)]
use impact::{ModuleLookup, find_module};
use json::json_external_dependencies;
pub use json::*;

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
    /// Maximum allowed representative cycle paths
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

    // Check representative cycle paths
    if let Some(max) = config.max_circular
        && circular_count > max
    {
        passed = false;
        failures.push(format!(
            "{} representative cycle paths (max: {})",
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
    writeln!(
        writer,
        "  Representative cycle paths: {}",
        result.circular_count
    )?;

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
    writeln!(writer, "  Worsened: {}", diff.worsened_issues.len())?;
    writeln!(writer, "  Unchanged: {}", diff.unchanged)?;

    write_issue_section(writer, "New Issues", &diff.new_issues)?;
    write_issue_section(writer, "Resolved Issues", &diff.resolved_issues)?;
    write_issue_section(writer, "Worsened Issues", &diff.worsened_issues)?;

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
    writeln!(writer, "Worsened issues: {}", diff.worsened_issues.len())?;
    writeln!(
        writer,
        "Status: {}",
        if passed { "PASSED" } else { "FAILED" }
    )?;

    if !passed {
        writeln!(writer)?;
        writeln!(writer, "Blocking New/Worsened Issues:")?;
        for issue in failures {
            write_issue_line(writer, issue)?;
        }
    }

    Ok(if passed { 0 } else { 1 })
}

fn write_issue_section<W: Write>(
    writer: &mut W,
    title: &str,
    issues: &[CouplingIssue],
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

fn write_issue_line<W: Write>(writer: &mut W, issue: &CouplingIssue) -> io::Result<()> {
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
    japanese: bool,
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

    if japanese {
        writeln!(writer, "外部依存の結合")?;
    } else {
        writeln!(writer, "External Dependency Coupling")?;
    }
    writeln!(
        writer,
        "═══════════════════════════════════════════════════════════"
    )?;
    let total_references = report
        .dependencies
        .iter()
        .map(|dependency| dependency.total_references)
        .sum::<usize>();
    if japanese {
        writeln!(
            writer,
            "外部クレート: {}  直接参照: {}",
            report.dependencies.len(),
            total_references
        )?;
    } else {
        writeln!(
            writer,
            "External crates: {}  Direct references: {}",
            report.dependencies.len(),
            total_references
        )?;
    }

    if report.dependencies.is_empty() {
        writeln!(writer)?;
        if japanese {
            writeln!(writer, "外部クレートへの結合は検出されませんでした。")?;
        } else {
            writeln!(writer, "No external crate couplings detected.")?;
        }
        return Ok(());
    }

    writeln!(writer)?;
    if japanese {
        writeln!(writer, "利用モジュール数が多いクレート:")?;
    } else {
        writeln!(writer, "Top Crates by Breadth:")?;
    }
    for (index, dependency) in report.dependencies.iter().take(10).enumerate() {
        let version = if dependency.versions.is_empty() {
            if japanese {
                "バージョン: 不明".to_string()
            } else {
                "version: unknown".to_string()
            }
        } else if japanese {
            format!("バージョン: {}", dependency.versions.join(", "))
        } else {
            format!("version: {}", dependency.versions.join(", "))
        };
        if japanese {
            writeln!(
                writer,
                "{}. {} ({}; {} モジュール, {} 参照, 主な強度: {})",
                index + 1,
                dependency.crate_name,
                version,
                dependency.breadth,
                dependency.total_references,
                dependency.dominant_strength
            )?;
        } else {
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
        }
        let sample_modules = dependency
            .source_modules
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        if !sample_modules.is_empty() {
            if japanese {
                writeln!(writer, "   モジュール: {}", sample_modules)?;
            } else {
                writeln!(writer, "   modules: {}", sample_modules)?;
            }
        }
    }

    writeln!(writer)?;
    if japanese {
        writeln!(writer, "分散した外部結合の警告:")?;
    } else {
        writeln!(writer, "Scattered Coupling Flags:")?;
    }
    if report.scattered_couplings.is_empty() {
        if japanese {
            writeln!(writer, "  (なし)")?;
        } else {
            writeln!(writer, "  (none)")?;
        }
    } else {
        for issue in &report.scattered_couplings {
            let source = if japanese {
                issue_source_japanese(issue)
            } else {
                issue.source.clone()
            };
            writeln!(
                writer,
                "  - {}: {} -> {}",
                severity_label(issue.severity, japanese),
                source,
                issue.target
            )?;
            if japanese {
                writeln!(writer, "    {}", issue_instance_description_japanese(issue))?;
                writeln!(writer, "    修正: {}", issue_refactoring_japanese(issue))?;
            } else {
                writeln!(writer, "    {}", issue.description)?;
                writeln!(writer, "    Fix: {}", issue.refactoring)?;
            }
        }
    }

    Ok(())
}

fn severity_label(severity: Severity, japanese: bool) -> String {
    if !japanese {
        return severity.to_string();
    }
    match severity {
        Severity::Critical => "緊急",
        Severity::High => "高",
        Severity::Medium => "中",
        Severity::Low => "低",
    }
    .to_string()
}

fn issue_source_japanese(issue: &CouplingIssue) -> String {
    if issue.issue_type == IssueType::ScatteredExternalCoupling
        && issue.source.ends_with(" internal modules")
    {
        let count = issue.source.split_whitespace().next().unwrap_or_default();
        return format!("{} 個の内部モジュール", count);
    }
    issue.source.clone()
}

fn issue_instance_description_japanese(issue: &CouplingIssue) -> String {
    match issue.issue_type {
        IssueType::ScatteredExternalCoupling => {
            let source = issue_source_japanese(issue);
            format!(
                "{} は、{}から直接使われています。サードパーティ更新時のリスクがコードベース全体に広がっています。",
                issue.target, source
            )
        }
        IssueType::HiddenCoupling => {
            "明示的なコード依存はありませんが、ファイルが頻繁に一緒に変更されています。暗黙の知識や不足した抽象化を示している可能性があります。"
                .to_string()
        }
        IssueType::AccidentalVolatility => {
            "安定しているはずのサブドメインが頻繁に変更されています。本質的な業務変化ではなく、設計や所有権の問題によるチャーンの可能性があります。"
                .to_string()
        }
        _ => issue.description.clone(),
    }
}

fn issue_refactoring_japanese(issue: &CouplingIssue) -> String {
    match issue.issue_type {
        IssueType::ScatteredExternalCoupling => {
            let facade = issue.target.replace('-', "_");
            format!(
                "`{}_facade` モジュールを導入し、直接利用をそこに集約する",
                facade
            )
        }
        IssueType::HiddenCoupling => {
            "共有されている知識を明示的な抽象化や境界に切り出す".to_string()
        }
        IssueType::AccidentalVolatility => {
            "変更理由を分離し、安定サブドメインを高頻度変更から守る".to_string()
        }
        _ => {
            let action = issue.refactoring.to_string();
            if action == "Extract a shared abstraction or make the dependency explicit" {
                "共有された抽象化を抽出するか、依存関係を明示する".to_string()
            } else {
                action
            }
        }
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
    let mut found_in_modules: Vec<(&str, &crate::metrics::module::ModuleMetrics)> = Vec::new();
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

    found_in_modules.sort_by(|a, b| a.0.cmp(b.0));
    outgoing.sort_by(|a, b| {
        a.item
            .cmp(&b.item)
            .then_with(|| a.module.cmp(&b.module))
            .then_with(|| a.file_path.cmp(&b.file_path))
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.dep_type.cmp(&b.dep_type))
            .then_with(|| a.strength.cmp(&b.strength))
    });
    incoming.sort_by(|a, b| {
        a.item
            .cmp(&b.item)
            .then_with(|| a.module.cmp(&b.module))
            .then_with(|| a.file_path.cmp(&b.file_path))
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.dep_type.cmp(&b.dep_type))
            .then_with(|| a.strength.cmp(&b.strength))
    });

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
            suggestions.sort();
            suggestions.dedup();
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
        let mut by_target: BTreeMap<String, Vec<&TraceDependency>> = BTreeMap::new();
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
        let mut by_source: BTreeMap<String, Vec<&TraceDependency>> = BTreeMap::new();
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
    let affected_modules: BTreeSet<_> = incoming.iter().map(|d| d.module.clone()).collect();
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
    fn hotspots_resolve_qualified_aliases_to_source_files() {
        let mut metrics = ProjectMetrics::new();
        for name in ["consumer", "provider"] {
            metrics.add_module(crate::ModuleMetrics::new(
                format!("{name}.rs").into(),
                name.into(),
            ));
        }
        metrics.add_coupling(crate::CouplingMetrics::new(
            "app::consumer".into(),
            "app::provider".into(),
            crate::IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            crate::Volatility::High,
        ));
        let hotspots = calculate_hotspots(&metrics, &IssueThresholds::default(), 10);
        let consumer = hotspots
            .iter()
            .find(|hotspot| hotspot.module == "consumer")
            .expect("qualified aliases must map to one module");
        assert_eq!(consumer.file_path.as_deref(), Some("consumer.rs"));
        assert!(
            !hotspots
                .iter()
                .any(|hotspot| hotspot.module == "app::consumer")
        );
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

        generate_external_dependencies_output(&report, true, false, &mut buf).unwrap();

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
    fn test_impact_analysis_orders_hash_map_derived_output() {
        use crate::metrics::coupling::CouplingMetrics;
        use crate::metrics::dimensions::IntegrationStrength;
        use crate::metrics::module::ModuleMetrics;
        use crate::volatility::Volatility;

        let mut metrics = ProjectMetrics::new();
        for name in ["root", "zeta", "alpha", "middle"] {
            metrics.add_module(ModuleMetrics::new(
                PathBuf::from(format!("src/{name}.rs")),
                name.to_string(),
            ));
        }
        for (source, target, strength) in [
            ("root", "zeta", IntegrationStrength::Model),
            ("root", "alpha", IntegrationStrength::Functional),
            ("root", "alpha", IntegrationStrength::Model),
            ("middle", "root", IntegrationStrength::Contract),
            ("zeta", "middle", IntegrationStrength::Model),
            ("alpha", "middle", IntegrationStrength::Model),
        ] {
            metrics.add_coupling(CouplingMetrics::new(
                source.to_string(),
                target.to_string(),
                strength,
                Distance::DifferentModule,
                Volatility::Low,
            ));
        }

        let analysis = analyze_impact(&metrics, "root").unwrap();

        assert_eq!(
            analysis
                .dependencies
                .iter()
                .map(|dependency| dependency.module.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "zeta"]
        );
        assert_eq!(
            analysis.dependencies[0]
                .strengths
                .iter()
                .map(|strength| strength.strength.as_str())
                .collect::<Vec<_>>(),
            ["Functional", "Model"]
        );
        assert_eq!(analysis.cascading_impact.second_order, ["alpha", "zeta"]);
    }

    #[test]
    fn test_impact_module_lookup_rejects_substrings_and_reports_ambiguity() {
        use crate::metrics::module::ModuleMetrics;

        let mut metrics = ProjectMetrics::new();
        for name in ["crate_a::config", "crate_b::config", "report"] {
            metrics.add_module(ModuleMetrics::new(
                PathBuf::from(format!("src/{}.rs", name.replace("::", "/"))),
                name.to_string(),
            ));
        }
        metrics.add_coupling(crate::metrics::coupling::CouplingMetrics::new(
            "TypeName::crate_c::config".to_string(),
            "target".to_string(),
            crate::metrics::dimensions::IntegrationStrength::Model,
            Distance::DifferentModule,
            crate::volatility::Volatility::Low,
        ));

        assert_eq!(find_module(&metrics, "port"), ModuleLookup::NotFound);
        assert_eq!(
            find_module(&metrics, "config"),
            ModuleLookup::Ambiguous(vec![
                "crate_a::config".to_string(),
                "crate_b::config".to_string()
            ])
        );

        let mut output = Vec::new();
        assert!(!generate_impact_output(&metrics, "config", &mut output).unwrap());
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Module 'config' is ambiguous"));
        assert!(text.contains("crate_a::config"));
        assert!(text.contains("crate_b::config"));
    }

    #[test]
    fn test_json_output_includes_analysis_manifest() {
        let metrics = ProjectMetrics::new();
        let thresholds = IssueThresholds::default();
        let manifest = build_manifest(&ManifestContext {
            git_used: false,
            tests_excluded: true,
            parse_failures: 0,
            skipped_crates: Vec::new(),
            boundary_skipped_files: 0,
            dead_config_patterns: Vec::new(),
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
        use crate::metrics::module::ModuleMetrics;

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
        use crate::metrics::coupling::CouplingMetrics;
        use crate::metrics::dimensions::IntegrationStrength;
        use crate::volatility::Volatility;

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

    #[test]
    fn test_json_output_is_deterministic() {
        use crate::metrics::coupling::CouplingMetrics;
        use crate::metrics::dimensions::IntegrationStrength;
        use crate::metrics::module::ModuleMetrics;
        use crate::volatility::{TemporalCoupling, Volatility};

        let mut metrics = ProjectMetrics::new();
        for name in ["zeta", "gamma", "epsilon", "delta", "beta", "alpha"] {
            metrics.add_module(ModuleMetrics::new(
                PathBuf::from(format!("src/{name}.rs")),
                name.to_string(),
            ));
        }
        for (source, target) in [
            ("alpha", "beta"),
            ("alpha", "gamma"),
            ("beta", "alpha"),
            ("beta", "gamma"),
            ("gamma", "alpha"),
        ] {
            metrics.add_coupling(CouplingMetrics::new(
                source.to_string(),
                target.to_string(),
                IntegrationStrength::Model,
                Distance::DifferentModule,
                Volatility::Low,
            ));
        }
        metrics.temporal_couplings = vec![
            TemporalCoupling {
                file_a: "src/delta.rs".to_string(),
                file_b: "src/zeta.rs".to_string(),
                co_change_count: 5,
                coupling_ratio: 0.5,
            },
            TemporalCoupling {
                file_a: "src/delta.rs".to_string(),
                file_b: "src/epsilon.rs".to_string(),
                co_change_count: 5,
                coupling_ratio: 0.5,
            },
        ];

        let thresholds = IssueThresholds::default();
        let manifest = build_manifest(&ManifestContext::default());
        let mut expected = Vec::new();
        generate_json_output(&metrics, &thresholds, &manifest, &mut expected).unwrap();

        for _ in 0..32 {
            let mut actual = Vec::new();
            generate_json_output(&metrics, &thresholds, &manifest, &mut actual).unwrap();
            assert_eq!(actual, expected);
        }

        let parsed: serde_json::Value = serde_json::from_slice(&expected).unwrap();
        let module_names: Vec<_> = parsed["modules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|module| module["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            module_names,
            ["alpha", "beta", "delta", "epsilon", "gamma", "zeta"]
        );
        assert_eq!(
            parsed["circular_dependencies"],
            serde_json::json!([
                ["alpha", "beta"],
                ["alpha", "beta", "gamma"],
                ["alpha", "gamma"]
            ])
        );
        assert_eq!(parsed["temporal_couplings"][0]["file_b"], "src/epsilon.rs");
        assert_eq!(parsed["temporal_couplings"][1]["file_b"], "src/zeta.rs");
    }
}
