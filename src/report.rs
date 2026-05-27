//! Text and Markdown report rendering for coupling analysis.
//!
//! This module translates balance scores, issue lists, volatility signals, and
//! blind-spot manifests into CLI-facing summaries and full reports.

use std::io::{self, Write};

use crate::balance::{
    BalanceScore, IssueThresholds, ProjectBalanceReport, Severity,
    analyze_project_balance_with_thresholds,
};
use crate::manifest::{AnalysisManifest, ManifestContext, build_manifest};
use crate::metrics::{Distance, IntegrationStrength, ProjectMetrics};

const DEFAULT_STRONG_TEMPORAL_LIMIT: usize = 5;

// ===== Report Options =====

/// Options for the default human-readable text report.
#[derive(Debug, Clone, Copy, Default)]
pub struct TextReportOptions {
    /// Include the full structural blind-spot descriptions instead of a pointer.
    pub show_structural_blind_spots: bool,
    /// Include all temporal-coupling pairs instead of the concise default.
    pub show_all_temporal_couplings: bool,
}

// ===== Summary Report =====

/// Generate a summary report to the given writer
pub fn generate_summary<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    let manifest = default_manifest();
    generate_summary_with_thresholds(metrics, &IssueThresholds::default(), &manifest, writer)
}

/// Generate a summary report with custom thresholds
pub fn generate_summary_with_thresholds<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    writer: &mut W,
) -> io::Result<()> {
    generate_summary_with_options(metrics, thresholds, manifest, false, writer)
}

/// Generate a summary report with custom thresholds and blind-spot detail.
pub fn generate_summary_with_options<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    show_structural_blind_spots: bool,
    writer: &mut W,
) -> io::Result<()> {
    let report = analyze_project_balance_with_thresholds(metrics, thresholds);
    let dimension_stats = metrics.calculate_dimension_stats();
    let jp = thresholds.japanese;

    let project_name = metrics.workspace_name.as_deref().unwrap_or("project");

    if jp {
        writeln!(writer, "カップリング分析: {}", project_name)?;
        writeln!(writer, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
        writeln!(writer)?;
        writeln!(
            writer,
            "評価: {} | スコア: {:.2}/1.00 | モジュール数: {}",
            report.health_grade,
            report.average_score,
            metrics.module_count()
        )?;
        writeln!(writer, "理由: {}", report.grade_rationale.summary)?;
    } else {
        writeln!(writer, "Balanced Coupling Analysis: {}", project_name)?;
        writeln!(writer, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
        writeln!(writer)?;
        writeln!(
            writer,
            "Grade: {} | Score: {:.2}/1.00 | Modules: {}",
            report.health_grade,
            report.average_score,
            metrics.module_count()
        )?;
        writeln!(writer, "Why this grade: {}", report.grade_rationale.summary)?;
    }
    writeln!(writer)?;

    // 3-Dimensional Analysis
    if !metrics.couplings.is_empty() {
        // Strength distribution
        let (intr_pct, func_pct, model_pct, contract_pct) = dimension_stats.strength_percentages();
        // Distance distribution
        let (same_pct, diff_pct, ext_pct) = dimension_stats.distance_percentages();
        // Volatility distribution
        let (low_pct, med_pct, high_pct) = dimension_stats.volatility_percentages();

        if jp {
            writeln!(writer, "3次元分析:")?;
            writeln!(
                writer,
                "  結合強度: Contract {:.0}% / Model {:.0}% / Functional {:.0}% / Intrusive {:.0}%",
                contract_pct, model_pct, func_pct, intr_pct
            )?;
            writeln!(
                writer,
                "           (トレイト)   (型)      (関数)        (内部アクセス)"
            )?;
            writeln!(
                writer,
                "  距離:     同一モジュール {:.0}% / 別モジュール {:.0}% / 外部 {:.0}%",
                same_pct, diff_pct, ext_pct
            )?;
            writeln!(
                writer,
                "  変更頻度: 低 {:.0}% / 中 {:.0}% / 高 {:.0}%",
                low_pct, med_pct, high_pct
            )?;
        } else {
            writeln!(writer, "3-Dimensional Analysis:")?;
            writeln!(
                writer,
                "  Strength:   Contract {:.0}% / Model {:.0}% / Functional {:.0}% / Intrusive {:.0}%",
                contract_pct, model_pct, func_pct, intr_pct
            )?;
            writeln!(
                writer,
                "  Distance:   Same {:.0}% / Different {:.0}% / External {:.0}%",
                same_pct, diff_pct, ext_pct
            )?;
            writeln!(
                writer,
                "  Volatility: Low {:.0}% / Medium {:.0}% / High {:.0}%",
                low_pct, med_pct, high_pct
            )?;
        }
        writeln!(writer)?;

        // Balance Classification
        if jp {
            writeln!(writer, "バランス状態:")?;
        } else {
            writeln!(writer, "Balance State:")?;
        }
        let bc = &dimension_stats.balance_counts;
        let total = dimension_stats.total();
        if bc.high_cohesion > 0 {
            if jp {
                writeln!(
                    writer,
                    "  ✅ 高凝集 (強い結合 + 近い距離): {} ({:.0}%) ← 理想的",
                    bc.high_cohesion,
                    bc.high_cohesion as f64 / total as f64 * 100.0
                )?;
            } else {
                writeln!(
                    writer,
                    "  ✅ High Cohesion (strong+close): {} ({:.0}%)",
                    bc.high_cohesion,
                    bc.high_cohesion as f64 / total as f64 * 100.0
                )?;
            }
        }
        if bc.loose_coupling > 0 {
            if jp {
                writeln!(
                    writer,
                    "  ✅ 疎結合 (弱い結合 + 遠い距離): {} ({:.0}%) ← 理想的",
                    bc.loose_coupling,
                    bc.loose_coupling as f64 / total as f64 * 100.0
                )?;
            } else {
                writeln!(
                    writer,
                    "  ✅ Loose Coupling (weak+far): {} ({:.0}%)",
                    bc.loose_coupling,
                    bc.loose_coupling as f64 / total as f64 * 100.0
                )?;
            }
        }
        if bc.acceptable > 0 {
            if jp {
                writeln!(
                    writer,
                    "  🤔 許容可能 (強い結合 + 遠い距離 + 安定): {} ({:.0}%)",
                    bc.acceptable,
                    bc.acceptable as f64 / total as f64 * 100.0
                )?;
            } else {
                writeln!(
                    writer,
                    "  🤔 Acceptable (strong+far+stable): {} ({:.0}%)",
                    bc.acceptable,
                    bc.acceptable as f64 / total as f64 * 100.0
                )?;
            }
        }
        if bc.pain > 0 {
            if jp {
                writeln!(
                    writer,
                    "  ❌ ペインゾーン (強い結合 + 遠い距離 + 頻繁に変更): {} ({:.0}%)",
                    bc.pain,
                    bc.pain as f64 / total as f64 * 100.0
                )?;
            } else {
                writeln!(
                    writer,
                    "  ❌ Pain Zone (strong+far+volatile): {} ({:.0}%)",
                    bc.pain,
                    bc.pain as f64 / total as f64 * 100.0
                )?;
            }
        }
        if bc.local_complexity > 0 {
            if jp {
                writeln!(
                    writer,
                    "  🔍 局所的複雑性 (弱い結合 + 近い距離): {} ({:.0}%)",
                    bc.local_complexity,
                    bc.local_complexity as f64 / total as f64 * 100.0
                )?;
            } else {
                writeln!(
                    writer,
                    "  🔍 Local Complexity (weak+close): {} ({:.0}%)",
                    bc.local_complexity,
                    bc.local_complexity as f64 / total as f64 * 100.0
                )?;
            }
        }
        writeln!(writer)?;
    }

    // Issue breakdown
    let critical = *report
        .issues_by_severity
        .get(&Severity::Critical)
        .unwrap_or(&0);
    let high = *report.issues_by_severity.get(&Severity::High).unwrap_or(&0);
    let medium = *report
        .issues_by_severity
        .get(&Severity::Medium)
        .unwrap_or(&0);
    let low = *report.issues_by_severity.get(&Severity::Low).unwrap_or(&0);

    if critical > 0 || high > 0 || medium > 0 || low > 0 {
        if jp {
            writeln!(writer, "検出された問題:")?;
            if critical > 0 {
                writeln!(writer, "  🔴 緊急: {} 件 (すぐに修正が必要)", critical)?;
            }
            if high > 0 {
                writeln!(writer, "  🟠 高: {} 件 (早めに対処)", high)?;
            }
            if medium > 0 {
                writeln!(writer, "  🟡 中: {} 件", medium)?;
            }
            if low > 0 {
                writeln!(writer, "  ⚪ 低: {} 件", low)?;
            }
        } else {
            writeln!(writer, "Detected Issues:")?;
            if critical > 0 {
                writeln!(writer, "  🔴 Critical: {} (must fix)", critical)?;
            }
            if high > 0 {
                writeln!(writer, "  🟠 High: {} (should fix)", high)?;
            }
            if medium > 0 {
                writeln!(writer, "  🟡 Medium: {}", medium)?;
            }
            if low > 0 {
                writeln!(writer, "  ⚪ Low: {}", low)?;
            }
        }
        writeln!(writer)?;
    } else if thresholds.strict_mode {
        if jp {
            writeln!(writer, "検出された問題: なし (--all で低優先度も表示)\n")?;
        } else {
            writeln!(
                writer,
                "Detected Issues: None (use --all to see Low severity)\n"
            )?;
        }
    }

    // Top priority if any
    if !report.top_priorities.is_empty() {
        if jp {
            writeln!(writer, "優先的に対処すべき問題:")?;
            for issue in report.top_priorities.iter().take(3) {
                let issue_jp = issue_type_japanese(issue.issue_type);
                writeln!(writer, "  - {} | {}", issue_jp, issue.source)?;
                writeln!(
                    writer,
                    "    → {}",
                    refactoring_action_japanese(&issue.refactoring)
                )?;
            }
        } else {
            writeln!(writer, "Top Priorities:")?;
            for issue in report.top_priorities.iter().take(3) {
                writeln!(
                    writer,
                    "  - [{}] {} → {}",
                    issue.severity, issue.source, issue.target
                )?;
            }
        }
        writeln!(writer)?;
    }

    // Rust Design Quality (newtype usage)
    let newtype_count = metrics.total_newtype_count();
    let type_count = metrics.total_type_count();
    if type_count > 0 {
        let newtype_ratio = metrics.newtype_ratio() * 100.0;
        if jp {
            let quality = if newtype_ratio >= 20.0 {
                "✅ 良好"
            } else if newtype_ratio >= 10.0 {
                "🤔 増やすことを検討"
            } else {
                "⚠️ 少ない"
            };
            writeln!(
                writer,
                "Rustパターン: newtype使用率 {}/{} ({:.0}%) - {}",
                newtype_count, type_count, newtype_ratio, quality
            )?;
        } else {
            let quality = if newtype_ratio >= 20.0 {
                "✅ Good"
            } else if newtype_ratio >= 10.0 {
                "🤔 Consider more"
            } else {
                "⚠️ Low usage"
            };
            writeln!(
                writer,
                "Rust Patterns: Newtype usage: {}/{} ({:.0}%) - {}",
                newtype_count, type_count, newtype_ratio, quality
            )?;
        }
        writeln!(writer)?;
    }

    // Circular dependencies
    let circular = metrics.circular_dependency_summary();
    if circular.total_cycles > 0 {
        if jp {
            writeln!(
                writer,
                "⚠️ 循環依存: {} サイクル ({} モジュール)",
                circular.total_cycles, circular.affected_modules
            )?;
        } else {
            writeln!(
                writer,
                "⚠️ Circular Dependencies: {} cycles ({} modules)",
                circular.total_cycles, circular.affected_modules
            )?;
        }
    }

    // Design decision guide (Japanese only, for educational purposes)
    if jp {
        writeln!(writer)?;
        writeln!(writer, "設計判断ガイド (Khononov):")?;
        writeln!(writer, "  ✅ 強い結合 + 近い距離 → 高凝集 (理想的)")?;
        writeln!(writer, "  ✅ 弱い結合 + 遠い距離 → 疎結合 (理想的)")?;
        writeln!(writer, "  🤔 強い結合 + 遠い距離 + 安定 → 許容可能")?;
        writeln!(
            writer,
            "  ❌ 強い結合 + 遠い距離 + 頻繁に変更 → 要リファクタリング"
        )?;
    }

    write_manifest_summary_section(manifest, jp, show_structural_blind_spots, writer)?;

    Ok(())
}

// ===== Localization Helpers =====

/// Get Japanese translation for issue type
fn issue_type_japanese(issue_type: crate::balance::IssueType) -> &'static str {
    use crate::balance::IssueType;
    match issue_type {
        IssueType::GlobalComplexity => "グローバル複雑性 (遠距離への強い依存)",
        IssueType::CascadingChangeRisk => "変更波及リスク (頻繁に変わるものへの依存)",
        IssueType::InappropriateIntimacy => "不適切な親密さ (内部実装への依存)",
        IssueType::HighEfferentCoupling => "出力依存過多 (多くのモジュールに依存)",
        IssueType::HighAfferentCoupling => "入力依存過多 (多くのモジュールから依存される)",
        IssueType::UnnecessaryAbstraction => "過剰な抽象化",
        IssueType::CircularDependency => "循環依存",
        IssueType::HiddenCoupling => "隠れた結合 (共変更のみで発見)",
        IssueType::AccidentalVolatility => "偶発的な変更頻度",
        IssueType::ScatteredExternalCoupling => "外部クレート結合の分散",
        IssueType::ShallowModule => "浅いモジュール",
        IssueType::PassThroughMethod => "パススルーメソッド",
        IssueType::HighCognitiveLoad => "高認知負荷",
        IssueType::GodModule => "神モジュール (責務が多すぎる)",
        IssueType::PublicFieldExposure => "公開フィールド (getterを検討)",
        IssueType::PrimitiveObsession => "プリミティブ過多 (newtypeを検討)",
    }
}

/// Get Japanese translation for refactoring action
fn refactoring_action_japanese(action: &crate::balance::RefactoringAction) -> String {
    use crate::balance::RefactoringAction;
    match action {
        RefactoringAction::IntroduceTrait { suggested_name, .. } => {
            format!("トレイト `{}` を導入して抽象化する", suggested_name)
        }
        RefactoringAction::MoveCloser { target_location } => {
            format!("`{}` に移動して距離を縮める", target_location)
        }
        RefactoringAction::ExtractAdapter { adapter_name, .. } => {
            format!("アダプタ `{}` を抽出する", adapter_name)
        }
        RefactoringAction::SplitModule { suggested_modules } => {
            format!("モジュールを分割: {}", suggested_modules.join(", "))
        }
        RefactoringAction::SimplifyAbstraction { .. } => "抽象化を簡素化する".to_string(),
        RefactoringAction::BreakCycle {
            suggested_direction,
        } => {
            format!("循環を断つ: {}", suggested_direction)
        }
        RefactoringAction::StabilizeInterface { interface_name } => {
            format!("安定したインターフェース `{}` を追加", interface_name)
        }
        RefactoringAction::General { action } => action.clone(),
        RefactoringAction::AddGetters { .. } => "getterメソッドを追加する".to_string(),
        RefactoringAction::IntroduceNewtype {
            suggested_name,
            wrapped_type,
        } => {
            format!(
                "newtype `struct {}({})` を導入",
                suggested_name, wrapped_type
            )
        }
    }
}

// ===== Full Markdown Report =====

/// Generate a full Markdown report with refactoring suggestions
pub fn generate_report<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    let manifest = default_manifest();
    generate_report_with_thresholds(metrics, &IssueThresholds::default(), &manifest, writer)
}

/// Generate a full Markdown report with custom thresholds
pub fn generate_report_with_thresholds<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    writer: &mut W,
) -> io::Result<()> {
    generate_report_with_options(
        metrics,
        thresholds,
        manifest,
        TextReportOptions::default(),
        writer,
    )
}

/// Generate a full Markdown report with custom thresholds and text options.
pub fn generate_report_with_options<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    options: TextReportOptions,
    writer: &mut W,
) -> io::Result<()> {
    let report = analyze_project_balance_with_thresholds(metrics, thresholds);

    writeln!(writer, "# Coupling Analysis Report\n")?;

    // Executive Summary
    write_executive_summary(metrics, &report, writer)?;

    // Refactoring Priorities (if any issues)
    if !report.issues.is_empty() {
        write_refactoring_priorities(&report, writer)?;
    }

    // Detailed Issues by Type
    write_issues_by_type(&report, writer)?;

    // Coupling details
    write_coupling_section(metrics, writer)?;

    // Module analysis
    write_module_section(metrics, writer)?;

    // Volatility section
    write_volatility_section(metrics, writer)?;

    // Temporal coupling section
    write_temporal_coupling_section(metrics, options.show_all_temporal_couplings, writer)?;

    // Circular dependency section
    write_circular_dependencies_section(metrics, writer)?;

    // Best practices
    write_best_practices(writer)?;

    // Declared analysis blind spots
    write_manifest_markdown_section(manifest, options.show_structural_blind_spots, writer)?;

    Ok(())
}

fn write_executive_summary<W: Write>(
    metrics: &ProjectMetrics,
    report: &ProjectBalanceReport,
    writer: &mut W,
) -> io::Result<()> {
    writeln!(writer, "## Executive Summary\n")?;

    // Health Grade with emoji
    let grade_emoji = match report.health_grade {
        crate::balance::HealthGrade::S => "⚠️",
        crate::balance::HealthGrade::A => "🟢",
        crate::balance::HealthGrade::B => "🟢",
        crate::balance::HealthGrade::C => "🟡",
        crate::balance::HealthGrade::D => "🟠",
        crate::balance::HealthGrade::F => "🔴",
    };

    writeln!(
        writer,
        "**Health Grade**: {} {}\n",
        grade_emoji, report.health_grade
    )?;
    writeln!(
        writer,
        "**Why this grade**: {}\n",
        report.grade_rationale.summary
    )?;

    writeln!(writer, "| Metric | Value |")?;
    writeln!(writer, "|--------|-------|")?;
    writeln!(writer, "| Files Analyzed | {} |", metrics.total_files)?;
    writeln!(writer, "| Total Modules | {} |", metrics.module_count())?;
    writeln!(writer, "| Total Couplings | {} |", report.total_couplings)?;
    writeln!(
        writer,
        "| Balance Score | {:.2}/1.00 |",
        report.average_score
    )?;
    writeln!(
        writer,
        "| Balanced | {} ({:.0}%) |",
        report.balanced_count,
        if report.total_couplings > 0 {
            (report.balanced_count as f64 / report.total_couplings as f64) * 100.0
        } else {
            100.0
        }
    )?;
    // This headline count mirrors `report.issues`/JSON `issues`; balance buckets
    // such as Pain Zone are separate coupling classifications, not surfaced issues.
    writeln!(writer, "| Issues Surfaced | {} |", report.issues.len())?;
    writeln!(writer)?;

    // Issue counts
    let critical = *report
        .issues_by_severity
        .get(&Severity::Critical)
        .unwrap_or(&0);
    let high = *report.issues_by_severity.get(&Severity::High).unwrap_or(&0);
    let medium = *report
        .issues_by_severity
        .get(&Severity::Medium)
        .unwrap_or(&0);
    let low = *report.issues_by_severity.get(&Severity::Low).unwrap_or(&0);

    if critical > 0 || high > 0 {
        writeln!(writer, "**⚠️ Action Required**\n")?;
        if critical > 0 {
            writeln!(
                writer,
                "- 🔴 **{} Critical** issues must be fixed immediately",
                critical
            )?;
        }
        if high > 0 {
            writeln!(
                writer,
                "- 🟠 **{} High** priority issues should be addressed soon",
                high
            )?;
        }
        if medium > 0 {
            writeln!(writer, "- 🟡 {} Medium priority issues to review", medium)?;
        }
        if low > 0 {
            writeln!(writer, "- {} Low priority suggestions", low)?;
        }
        writeln!(writer)?;
    } else if medium > 0 {
        writeln!(
            writer,
            "**ℹ️ Review Suggested**: {} issues to consider.\n",
            medium + low
        )?;
    } else {
        writeln!(
            writer,
            "**✅ Good Health**: No significant coupling issues detected.\n"
        )?;
    }

    Ok(())
}

fn write_refactoring_priorities<W: Write>(
    report: &ProjectBalanceReport,
    writer: &mut W,
) -> io::Result<()> {
    writeln!(writer, "## 🔧 Refactoring Priorities\n")?;

    // Show top 5 priority issues with concrete actions
    writeln!(writer, "### Immediate Actions\n")?;

    let priority_issues: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.severity >= Severity::Medium)
        .take(5)
        .collect();

    if priority_issues.is_empty() {
        writeln!(writer, "No immediate refactoring actions required.\n")?;
        return Ok(());
    }

    for (i, issue) in priority_issues.iter().enumerate() {
        let severity_icon = match issue.severity {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "⚪",
        };

        writeln!(
            writer,
            "**{}. {} `{}` → `{}`**\n",
            i + 1,
            severity_icon,
            issue.source,
            issue.target
        )?;

        writeln!(
            writer,
            "- **Issue**: {} - {}",
            issue.issue_type, issue.description
        )?;
        writeln!(writer, "- **Why**: {}", issue.issue_type.description())?;
        writeln!(writer, "- **Action**: {}", issue.refactoring)?;
        writeln!(writer, "- **Balance Score**: {:.2}\n", issue.balance_score)?;
    }

    Ok(())
}

fn write_issues_by_type<W: Write>(report: &ProjectBalanceReport, writer: &mut W) -> io::Result<()> {
    if report.issues.is_empty() {
        return Ok(());
    }

    writeln!(writer, "## Issues by Category\n")?;

    let grouped = report.issues_grouped_by_type();

    // Order by severity of issues in each group
    let mut issue_types: Vec<_> = grouped.keys().collect();
    issue_types.sort_by(|a, b| {
        let a_max = grouped
            .get(a)
            .and_then(|v| v.iter().map(|i| i.severity).max());
        let b_max = grouped
            .get(b)
            .and_then(|v| v.iter().map(|i| i.severity).max());
        b_max.cmp(&a_max)
    });

    for issue_type in issue_types {
        if let Some(issues) = grouped.get(issue_type) {
            let count = issues.len();

            writeln!(writer, "### {} ({} instances)\n", issue_type, count)?;
            writeln!(writer, "> {}\n", issue_type.description())?;

            // Show up to 5 examples
            writeln!(writer, "| Severity | Source | Target | Action |")?;
            writeln!(writer, "|----------|--------|--------|--------|")?;

            for issue in issues.iter().take(5) {
                let action_short = format!("{}", issue.refactoring);
                let action_truncated = if action_short.len() > 40 {
                    format!("{}...", &action_short[..40])
                } else {
                    action_short
                };
                writeln!(
                    writer,
                    "| {} | `{}` | `{}` | {} |",
                    issue.severity,
                    truncate_path(&issue.source, 25),
                    truncate_path(&issue.target, 25),
                    action_truncated
                )?;
            }

            if count > 5 {
                writeln!(writer, "\n*...and {} more instances*", count - 5)?;
            }
            writeln!(writer)?;
        }
    }

    Ok(())
}

fn write_coupling_section<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    if metrics.couplings.is_empty() {
        return Ok(());
    }

    writeln!(writer, "## Coupling Distribution\n")?;

    // Strength distribution
    writeln!(writer, "### By Integration Strength\n")?;
    writeln!(writer, "| Strength | Count | % | Description |")?;
    writeln!(writer, "|----------|-------|---|-------------|")?;

    let total = metrics.couplings.len() as f64;
    for (strength, label, desc) in [
        (
            IntegrationStrength::Contract,
            "Contract",
            "Depends on traits/interfaces only",
        ),
        (
            IntegrationStrength::Model,
            "Model",
            "Uses data types/structs",
        ),
        (
            IntegrationStrength::Functional,
            "Functional",
            "Calls specific functions",
        ),
        (
            IntegrationStrength::Intrusive,
            "Intrusive",
            "Accesses internal details",
        ),
    ] {
        let count = metrics
            .couplings
            .iter()
            .filter(|c| c.strength == strength)
            .count();
        let pct = (count as f64 / total) * 100.0;
        writeln!(writer, "| {} | {} | {:.0}% | {} |", label, count, pct, desc)?;
    }
    writeln!(writer)?;

    // Distance distribution
    writeln!(writer, "### By Distance\n")?;
    writeln!(writer, "| Distance | Count | % |")?;
    writeln!(writer, "|----------|-------|---|")?;

    for (distance, label) in [
        (Distance::SameModule, "Same Module (close)"),
        (Distance::DifferentModule, "Different Module"),
        (Distance::DifferentCrate, "External Crate (far)"),
    ] {
        let count = metrics
            .couplings
            .iter()
            .filter(|c| c.distance == distance)
            .count();
        let pct = (count as f64 / total) * 100.0;
        writeln!(writer, "| {} | {} | {:.0}% |", label, count, pct)?;
    }
    writeln!(writer)?;

    // Volatility distribution (only for internal couplings where we have git data)
    let internal_couplings: Vec<_> = metrics
        .couplings
        .iter()
        .filter(|c| c.distance != Distance::DifferentCrate)
        .collect();

    if !internal_couplings.is_empty() {
        let internal_total = internal_couplings.len() as f64;
        writeln!(writer, "### By Volatility (Internal Couplings)\n")?;
        writeln!(writer, "| Volatility | Count | % | Impact on Balance |")?;
        writeln!(writer, "|------------|-------|---|-------------------|")?;

        for (volatility, label, impact) in [
            (
                crate::metrics::Volatility::Low,
                "Low (rarely changes)",
                "No penalty",
            ),
            (
                crate::metrics::Volatility::Medium,
                "Medium (sometimes changes)",
                "Moderate penalty",
            ),
            (
                crate::metrics::Volatility::High,
                "High (frequently changes)",
                "Significant penalty",
            ),
        ] {
            let count = internal_couplings
                .iter()
                .filter(|c| c.volatility == volatility)
                .count();
            let pct = (count as f64 / internal_total) * 100.0;
            writeln!(
                writer,
                "| {} | {} | {:.0}% | {} |",
                label, count, pct, impact
            )?;
        }
        writeln!(writer)?;
    }

    // Worst balanced couplings
    writeln!(writer, "### Worst Balanced Couplings\n")?;

    let mut couplings_with_scores: Vec<_> = metrics
        .couplings
        .iter()
        .map(|c| (c, BalanceScore::calculate(c)))
        .collect();

    couplings_with_scores.sort_by(|a, b| {
        a.1.score
            .partial_cmp(&b.1.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    writeln!(
        writer,
        "| Source | Target | Strength | Distance | Volatility | Score | Status |"
    )?;
    writeln!(
        writer,
        "|--------|--------|----------|----------|------------|-------|--------|"
    )?;

    for (coupling, score) in couplings_with_scores.iter().take(15) {
        let strength_str = match coupling.strength {
            IntegrationStrength::Contract => "Contract",
            IntegrationStrength::Model => "Model",
            IntegrationStrength::Functional => "Functional",
            IntegrationStrength::Intrusive => "Intrusive",
        };
        let distance_str = match coupling.distance {
            Distance::SameFunction => "Same Fn",
            Distance::SameModule => "Same Mod",
            Distance::DifferentModule => "Diff Mod",
            Distance::DifferentCrate => "External",
        };
        let volatility_str = match coupling.volatility {
            crate::metrics::Volatility::Low => "Low",
            crate::metrics::Volatility::Medium => "Med",
            crate::metrics::Volatility::High => "High",
        };
        let status = match score.interpretation {
            crate::balance::BalanceInterpretation::Balanced => "✅ Balanced",
            crate::balance::BalanceInterpretation::Acceptable => "✅ OK",
            crate::balance::BalanceInterpretation::NeedsReview => "🟡 Review",
            crate::balance::BalanceInterpretation::NeedsRefactoring => "🟠 Refactor",
            crate::balance::BalanceInterpretation::Critical => "🔴 Critical",
        };

        writeln!(
            writer,
            "| `{}` | `{}` | {} | {} | {} | {:.2} | {} |",
            truncate_path(&coupling.source, 20),
            truncate_path(&coupling.target, 20),
            strength_str,
            distance_str,
            volatility_str,
            score.score,
            status
        )?;
    }

    if couplings_with_scores.len() > 15 {
        writeln!(
            writer,
            "\n*Showing 15 of {} couplings*",
            couplings_with_scores.len()
        )?;
    }
    writeln!(writer)?;

    Ok(())
}

fn write_module_section<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    if metrics.modules.is_empty() {
        return Ok(());
    }

    writeln!(writer, "## Module Statistics\n")?;

    let show_subdomain = metrics
        .modules
        .values()
        .any(|module| module.subdomain.is_some());
    if show_subdomain {
        writeln!(
            writer,
            "| Module | Subdomain | Trait Impl | Inherent Impl | Internal Deps | External Deps |"
        )?;
        writeln!(
            writer,
            "|--------|-----------|------------|---------------|---------------|---------------|"
        )?;
    } else {
        writeln!(
            writer,
            "| Module | Trait Impl | Inherent Impl | Internal Deps | External Deps |"
        )?;
        writeln!(
            writer,
            "|--------|------------|---------------|---------------|---------------|"
        )?;
    }

    let mut modules: Vec<_> = metrics.modules.iter().collect();
    modules.sort_by(|a, b| {
        let a_deps = a.1.internal_deps.len() + a.1.external_deps.len();
        let b_deps = b.1.internal_deps.len() + b.1.external_deps.len();
        b_deps.cmp(&a_deps)
    });

    for (name, module) in modules.iter().take(20) {
        if show_subdomain {
            writeln!(
                writer,
                "| `{}` | {} | {} | {} | {} | {} |",
                truncate_path(name, 30),
                module
                    .subdomain
                    .map(|subdomain| subdomain.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                module.trait_impl_count,
                module.inherent_impl_count,
                module.internal_deps.len(),
                module.external_deps.len()
            )?;
        } else {
            writeln!(
                writer,
                "| `{}` | {} | {} | {} | {} |",
                truncate_path(name, 30),
                module.trait_impl_count,
                module.inherent_impl_count,
                module.internal_deps.len(),
                module.external_deps.len()
            )?;
        }
    }

    if modules.len() > 20 {
        writeln!(writer, "\n*Showing top 20 of {} modules*", modules.len())?;
    }
    writeln!(writer)?;

    Ok(())
}

fn write_volatility_section<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    writeln!(writer, "## Volatility Analysis\n")?;

    if metrics.file_changes.is_empty() {
        writeln!(
            writer,
            "*Git history analysis not available. Run in a git repository for volatility data.*\n"
        )?;
        return Ok(());
    }

    let mut high_vol: Vec<_> = metrics
        .file_changes
        .iter()
        .filter(|&(_, count)| *count > 10)
        .collect();

    high_vol.sort_by(|a, b| b.1.cmp(a.1));

    if high_vol.is_empty() {
        writeln!(
            writer,
            "No high volatility files detected (threshold: >10 changes).\n"
        )?;
    } else {
        writeln!(writer, "### High Volatility Files\n")?;
        writeln!(
            writer,
            "⚠️ Strong coupling to these files increases cascading change risk.\n"
        )?;
        writeln!(writer, "| File | Changes |")?;
        writeln!(writer, "|------|---------|")?;
        for (file, count) in high_vol.iter().take(10) {
            writeln!(writer, "| `{}` | {} |", file, count)?;
        }
        writeln!(writer)?;
    }

    Ok(())
}

fn write_temporal_coupling_section<W: Write>(
    metrics: &ProjectMetrics,
    show_all: bool,
    writer: &mut W,
) -> io::Result<()> {
    if metrics.temporal_couplings.is_empty() {
        return Ok(());
    }

    let mut strong: Vec<_> = metrics
        .temporal_couplings
        .iter()
        .filter(|tc| tc.is_strong())
        .collect();
    strong.sort_by(|a, b| {
        b.coupling_ratio
            .partial_cmp(&a.coupling_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if !show_all && strong.is_empty() {
        return Ok(());
    }

    writeln!(writer, "## Temporal Coupling (Co-Change Analysis)\n")?;
    writeln!(
        writer,
        "Files that frequently change together in git commits, indicating implicit coupling"
    )?;
    writeln!(writer, "beyond what code structure reveals.\n")?;

    if !strong.is_empty() {
        writeln!(
            writer,
            "### Strong Temporal Coupling (>50% co-change ratio)\n"
        )?;
        writeln!(
            writer,
            "⚠️ These pairs may share implicit knowledge (business logic, assumptions, data formats).\n"
        )?;
        writeln!(writer, "| File A | File B | Co-changes | Ratio |")?;
        writeln!(writer, "|--------|--------|------------|-------|")?;
        let strong_limit = if show_all {
            strong.len()
        } else {
            DEFAULT_STRONG_TEMPORAL_LIMIT
        };
        for tc in strong.iter().take(strong_limit) {
            writeln!(
                writer,
                "| `{}` | `{}` | {} | {:.0}% |",
                tc.file_a,
                tc.file_b,
                tc.co_change_count,
                tc.coupling_ratio * 100.0
            )?;
        }
        if !show_all && strong.len() > strong_limit {
            writeln!(
                writer,
                "\n*... and {} more (use --all)*",
                strong.len() - strong_limit
            )?;
        }
        writeln!(writer)?;
    }

    if !show_all {
        return Ok(());
    }

    let moderate: Vec<_> = metrics
        .temporal_couplings
        .iter()
        .filter(|tc| !tc.is_strong())
        .collect();

    if !moderate.is_empty() {
        writeln!(writer, "### Moderate Temporal Coupling\n")?;
        writeln!(writer, "| File A | File B | Co-changes | Ratio |")?;
        writeln!(writer, "|--------|--------|------------|-------|")?;
        for tc in moderate {
            writeln!(
                writer,
                "| `{}` | `{}` | {} | {:.0}% |",
                tc.file_a,
                tc.file_b,
                tc.co_change_count,
                tc.coupling_ratio * 100.0
            )?;
        }
        writeln!(writer)?;
    }

    Ok(())
}

fn write_circular_dependencies_section<W: Write>(
    metrics: &ProjectMetrics,
    writer: &mut W,
) -> io::Result<()> {
    let summary = metrics.circular_dependency_summary();

    if summary.total_cycles == 0 {
        writeln!(writer, "## Circular Dependencies\n")?;
        writeln!(writer, "✅ No circular dependencies detected.\n")?;
        return Ok(());
    }

    writeln!(writer, "## ⚠️ Circular Dependencies\n")?;
    writeln!(
        writer,
        "Found **{} circular dependency cycle(s)** involving **{} modules**.\n",
        summary.total_cycles, summary.affected_modules
    )?;

    writeln!(
        writer,
        "Circular dependencies make code harder to understand, test, and maintain."
    )?;
    writeln!(writer, "Consider breaking cycles by:\n")?;
    writeln!(writer, "1. Extracting shared types into a separate module")?;
    writeln!(writer, "2. Inverting dependencies using traits/interfaces")?;
    writeln!(writer, "3. Moving functionality to reduce coupling\n")?;

    writeln!(writer, "### Detected Cycles\n")?;

    for (i, cycle) in summary.cycles.iter().take(10).enumerate() {
        let cycle_str = cycle.join(" → ");
        writeln!(
            writer,
            "{}. `{}` → `{}`",
            i + 1,
            cycle_str,
            cycle.first().unwrap_or(&"?".to_string())
        )?;
    }

    if summary.cycles.len() > 10 {
        writeln!(
            writer,
            "\n*...and {} more cycles*",
            summary.cycles.len() - 10
        )?;
    }
    writeln!(writer)?;

    Ok(())
}

fn write_best_practices<W: Write>(writer: &mut W) -> io::Result<()> {
    writeln!(writer, "## Balance Guidelines\n")?;

    writeln!(
        writer,
        "The goal is **balanced coupling**, not zero coupling.\n"
    )?;

    writeln!(writer, "### Ideal Patterns ✅\n")?;
    writeln!(writer, "| Pattern | Example | Why It Works |")?;
    writeln!(writer, "|---------|---------|--------------|")?;
    writeln!(
        writer,
        "| Strong + Close | `impl` blocks in same module | Cohesion within boundaries |"
    )?;
    writeln!(
        writer,
        "| Weak + Far | Trait impl for external crate | Loose coupling across boundaries |"
    )?;
    writeln!(writer)?;

    writeln!(writer, "### Problematic Patterns ❌\n")?;
    writeln!(writer, "| Pattern | Problem | Solution |")?;
    writeln!(writer, "|---------|---------|----------|")?;
    writeln!(
        writer,
        "| Strong + Far | Global complexity | Introduce adapter or move closer |"
    )?;
    writeln!(
        writer,
        "| Strong + Volatile | Cascading changes | Add stable interface |"
    )?;
    writeln!(
        writer,
        "| Intrusive + Cross-boundary | Encapsulation violation | Extract trait API |"
    )?;
    writeln!(writer)?;

    Ok(())
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - max_len + 3..])
    }
}

// ===== AI-Oriented Report =====

/// Generate AI-friendly output format for coding agents
///
/// This format is designed to be:
/// 1. Concise and structured for LLM consumption
/// 2. Actionable with specific file/module references
/// 3. Copy-paste ready for AI refactoring prompts
pub fn generate_ai_output<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    let manifest = default_manifest();
    generate_ai_output_with_thresholds(metrics, &IssueThresholds::default(), &manifest, writer)
}

/// Generate AI-friendly output with custom thresholds
pub fn generate_ai_output_with_thresholds<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    writer: &mut W,
) -> io::Result<()> {
    let report = analyze_project_balance_with_thresholds(metrics, thresholds);

    let project_name = metrics.workspace_name.as_deref().unwrap_or("project");
    writeln!(writer, "Coupling Issues in {}:", project_name)?;
    writeln!(
        writer,
        "────────────────────────────────────────────────────────────"
    )?;
    writeln!(writer)?;

    // Summary line
    writeln!(
        writer,
        "Grade: {} | Score: {:.2} | Issues: {} High, {} Medium",
        report.health_grade,
        report.average_score,
        report.issues_by_severity.get(&Severity::High).unwrap_or(&0),
        report
            .issues_by_severity
            .get(&Severity::Medium)
            .unwrap_or(&0)
    )?;
    writeln!(writer, "Why this grade: {}", report.grade_rationale.summary)?;
    writeln!(writer)?;

    // List issues in a structured format
    if report.issues.is_empty() {
        writeln!(writer, "✅ No coupling issues detected.")?;
        writeln!(writer)?;
    } else {
        writeln!(writer, "Issues:")?;
        writeln!(writer)?;

        for (i, issue) in report.issues.iter().take(10).enumerate() {
            let severity_marker = match issue.severity {
                Severity::Critical => "🔴",
                Severity::High => "🟠",
                Severity::Medium => "🟡",
                Severity::Low => "⚪",
            };

            writeln!(
                writer,
                "{}. {} {} → {}",
                i + 1,
                severity_marker,
                issue.source,
                issue.target
            )?;
            writeln!(writer, "   Type: {}", issue.issue_type)?;
            writeln!(writer, "   Problem: {}", issue.description)?;
            writeln!(writer, "   Fix: {}", issue.refactoring)?;
            writeln!(writer)?;
        }

        if report.issues.len() > 10 {
            writeln!(writer, "... and {} more issues", report.issues.len() - 10)?;
            writeln!(writer)?;
        }
    }

    // Circular dependencies (critical for AI to understand)
    let circular = metrics.circular_dependency_summary();
    if circular.total_cycles > 0 {
        writeln!(
            writer,
            "Circular Dependencies ({} cycles):",
            circular.total_cycles
        )?;
        for cycle in circular.cycles.iter().take(5) {
            writeln!(
                writer,
                "  {} → {}",
                cycle.join(" → "),
                cycle.first().unwrap_or(&"?".to_string())
            )?;
        }
        writeln!(writer)?;
    }

    // Temporal coupling (important for AI to understand implicit dependencies)
    let strong_temporal: Vec<_> = metrics
        .temporal_couplings
        .iter()
        .filter(|tc| tc.is_strong())
        .collect();
    if !strong_temporal.is_empty() {
        writeln!(writer, "Temporal Coupling (implicit dependencies):")?;
        for tc in strong_temporal.iter().take(5) {
            writeln!(
                writer,
                "  {} ↔ {} ({} co-changes, {:.0}% ratio)",
                tc.file_a,
                tc.file_b,
                tc.co_change_count,
                tc.coupling_ratio * 100.0
            )?;
        }
        writeln!(writer)?;
    }

    // only show ai refactor advice if there is something to refactor
    if !report.issues.is_empty() || circular.total_cycles > 0 {
        writeln!(
            writer,
            "────────────────────────────────────────────────────────────"
        )?;
        writeln!(writer)?;

        // AI prompt suggestion
        writeln!(
            writer,
            "💡 To refactor with AI, copy this output and use this prompt:"
        )?;
        writeln!(writer)?;
        writeln!(writer, "```")?;
        writeln!(
            writer,
            "Analyze the coupling issues above from `cargo coupling --ai`. "
        )?;
        writeln!(
            writer,
            "For each issue, suggest specific code changes to reduce coupling."
        )?;
        writeln!(
            writer,
            "Focus on introducing traits, moving code closer, or breaking circular dependencies."
        )?;
        writeln!(writer, "```")?;
    }

    write_manifest_summary_section(manifest, false, true, writer)?;

    Ok(())
}

fn default_manifest() -> AnalysisManifest {
    build_manifest(&ManifestContext {
        git_used: true,
        tests_excluded: false,
        parse_failures: 0,
    })
}

fn write_manifest_markdown_section<W: Write>(
    manifest: &AnalysisManifest,
    show_structural_blind_spots: bool,
    writer: &mut W,
) -> io::Result<()> {
    writeln!(writer, "## Not Analyzed (blind spots)\n")?;

    if show_structural_blind_spots {
        for blind_spot in &manifest.blind_spots {
            writeln!(
                writer,
                "- **{}**: {}",
                blind_spot.area, blind_spot.description
            )?;
        }
    }

    if !manifest.notes.is_empty() {
        if show_structural_blind_spots {
            writeln!(writer)?;
        }
        writeln!(writer, "Run-specific notes:")?;
        for note in &manifest.notes {
            writeln!(writer, "- {}", note)?;
        }
    }

    if !show_structural_blind_spots {
        if !manifest.notes.is_empty() {
            writeln!(writer)?;
        }
        writeln!(
            writer,
            "ℹ {} structural blind spots not analyzed — see --blind-spots (or --json).",
            manifest.blind_spots.len()
        )?;
    }

    writeln!(writer)?;
    Ok(())
}

fn write_manifest_summary_section<W: Write>(
    manifest: &AnalysisManifest,
    japanese: bool,
    show_structural_blind_spots: bool,
    writer: &mut W,
) -> io::Result<()> {
    if japanese {
        writeln!(writer, "未分析範囲 (blind spots):")?;
    } else {
        writeln!(writer, "Not Analyzed (blind spots):")?;
    }

    if show_structural_blind_spots {
        for blind_spot in &manifest.blind_spots {
            writeln!(
                writer,
                "  - {}: {}",
                blind_spot.area, blind_spot.description
            )?;
        }
    }

    if !manifest.notes.is_empty() {
        if japanese {
            writeln!(writer, "実行時の注意:")?;
        } else {
            writeln!(writer, "Run-specific notes:")?;
        }
        for note in &manifest.notes {
            writeln!(writer, "  - {}", note)?;
        }
    }

    if !show_structural_blind_spots {
        if japanese {
            writeln!(
                writer,
                "ℹ {} 件の構造的 blind spot は未分析です — 詳細は --blind-spots (または --json)。",
                manifest.blind_spots.len()
            )?;
        } else {
            writeln!(
                writer,
                "ℹ {} structural blind spots not analyzed — see --blind-spots (or --json).",
                manifest.blind_spots.len()
            )?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ManifestContext, build_manifest};
    use std::path::PathBuf;

    #[test]
    fn test_generate_summary() {
        let metrics = ProjectMetrics::new();
        let mut output = Vec::new();

        let result = generate_summary(&metrics, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Balanced Coupling Analysis"));
        assert!(output_str.contains("Grade:"));
        assert!(output_str.contains("Why this grade:"));
    }

    #[test]
    fn test_generate_report() {
        let metrics = ProjectMetrics::new();
        let mut output = Vec::new();

        let result = generate_report(&metrics, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("# Coupling Analysis Report"));
        assert!(output_str.contains("Executive Summary"));
        assert!(output_str.contains("**Why this grade**:"));
        assert!(output_str.contains("## Not Analyzed (blind spots)"));
        assert!(output_str.contains("4 structural blind spots not analyzed"));
        assert!(!output_str.contains("Dynamic connascence (Execution"));
    }

    #[test]
    fn test_generate_summary_includes_manifest_notes_and_pointer() {
        let metrics = ProjectMetrics::new();
        let thresholds = IssueThresholds::default();
        let manifest = build_manifest(&ManifestContext {
            git_used: false,
            tests_excluded: true,
            parse_failures: 0,
        });
        let mut output = Vec::new();

        let result =
            generate_summary_with_thresholds(&metrics, &thresholds, &manifest, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Not Analyzed (blind spots):"));
        assert!(output_str.contains("4 structural blind spots not analyzed"));
        assert!(!output_str.contains("Dynamic connascence (Execution"));
        assert!(output_str.contains("Git history was not analyzed"));
        assert!(output_str.contains("Test code was excluded"));
    }

    #[test]
    fn test_text_report_blind_spots_are_opt_in() {
        let metrics = ProjectMetrics::new();
        let thresholds = IssueThresholds::default();
        let manifest = build_manifest(&ManifestContext {
            git_used: false,
            tests_excluded: true,
            parse_failures: 1,
        });

        let mut default_output = Vec::new();
        generate_report_with_options(
            &metrics,
            &thresholds,
            &manifest,
            TextReportOptions::default(),
            &mut default_output,
        )
        .unwrap();
        let default_text = String::from_utf8(default_output).unwrap();
        assert!(default_text.contains("4 structural blind spots not analyzed"));
        assert!(default_text.contains("Git history was not analyzed"));
        assert!(default_text.contains("Test code was excluded"));
        assert!(default_text.contains("1 source file(s) failed to parse"));
        assert!(!default_text.contains("Dynamic connascence (Execution"));

        for options in [
            TextReportOptions {
                show_structural_blind_spots: true,
                show_all_temporal_couplings: false,
            },
            TextReportOptions {
                show_structural_blind_spots: true,
                show_all_temporal_couplings: true,
            },
        ] {
            let mut output = Vec::new();
            generate_report_with_options(&metrics, &thresholds, &manifest, options, &mut output)
                .unwrap();
            let text = String::from_utf8(output).unwrap();
            assert!(text.contains("dynamic-connascence"));
            assert!(text.contains("Dynamic connascence (Execution"));
        }
    }

    #[test]
    fn test_text_report_temporal_coupling_default_truncates_strong_pairs() {
        use crate::volatility::TemporalCoupling;

        let mut metrics = ProjectMetrics::new();
        metrics.temporal_couplings = (0..7)
            .map(|i| TemporalCoupling {
                file_a: format!("src/a{}.rs", i),
                file_b: format!("src/b{}.rs", i),
                co_change_count: i + 1,
                coupling_ratio: 0.95 - (i as f64 * 0.05),
            })
            .collect();
        metrics.temporal_couplings.push(TemporalCoupling {
            file_a: "src/moderate_a.rs".to_string(),
            file_b: "src/moderate_b.rs".to_string(),
            co_change_count: 2,
            coupling_ratio: 0.4,
        });

        let manifest = default_manifest();
        let thresholds = IssueThresholds::default();
        let mut output = Vec::new();
        generate_report_with_options(
            &metrics,
            &thresholds,
            &manifest,
            TextReportOptions::default(),
            &mut output,
        )
        .unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("src/a0.rs"));
        assert!(text.contains("src/a4.rs"));
        assert!(!text.contains("src/a5.rs"));
        assert!(!text.contains("src/moderate_a.rs"));
        assert!(text.contains("... and 2 more (use --all)"));

        let mut all_output = Vec::new();
        generate_report_with_options(
            &metrics,
            &thresholds,
            &manifest,
            TextReportOptions {
                show_structural_blind_spots: false,
                show_all_temporal_couplings: true,
            },
            &mut all_output,
        )
        .unwrap();
        let all_text = String::from_utf8(all_output).unwrap();
        assert!(all_text.contains("src/a6.rs"));
        assert!(all_text.contains("src/moderate_a.rs"));
    }

    #[test]
    fn test_report_issues_surfaced_count_matches_issue_list() {
        use crate::balance::analyze_project_balance_with_thresholds;
        use crate::metrics::{CouplingMetrics, Distance, IntegrationStrength, Volatility};

        let mut metrics = ProjectMetrics::new();
        metrics.add_coupling(CouplingMetrics::new(
            "source".to_string(),
            "target".to_string(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        ));

        let thresholds = IssueThresholds::default();
        let report = analyze_project_balance_with_thresholds(&metrics, &thresholds);
        let mut output = Vec::new();
        generate_report_with_options(
            &metrics,
            &thresholds,
            &default_manifest(),
            TextReportOptions::default(),
            &mut output,
        )
        .unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains(&format!("| Issues Surfaced | {} |", report.issues.len())));
    }

    #[test]
    fn test_generate_ai_output_includes_manifest() {
        let metrics = ProjectMetrics::new();
        let manifest = build_manifest(&ManifestContext {
            git_used: false,
            tests_excluded: false,
            parse_failures: 0,
        });
        let mut output = Vec::new();

        let result = generate_ai_output_with_thresholds(
            &metrics,
            &IssueThresholds::default(),
            &manifest,
            &mut output,
        );
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Not Analyzed (blind spots):"));
        assert!(output_str.contains("macro-and-cfg"));
        assert!(output_str.contains("Git history was not analyzed"));
    }

    #[test]
    fn test_generate_report_with_modules() {
        use crate::metrics::ModuleMetrics;

        let mut metrics = ProjectMetrics::new();
        let mut module = ModuleMetrics::new(PathBuf::from("lib.rs"), "lib".to_string());
        module.trait_impl_count = 3;
        module.inherent_impl_count = 2;
        metrics.add_module(module);

        let mut output = Vec::new();
        let result = generate_report(&metrics, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Module Statistics"));
    }

    #[test]
    fn test_generate_report_surfaces_subdomain_when_present() {
        use crate::config::Subdomain;
        use crate::metrics::ModuleMetrics;

        let mut metrics = ProjectMetrics::new();
        let mut module = ModuleMetrics::new(PathBuf::from("src/report.rs"), "report".to_string());
        module.subdomain = Some(Subdomain::Supporting);
        metrics.add_module(module);

        let mut output = Vec::new();
        generate_report(&metrics, &mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("| Module | Subdomain |"));
        assert!(output_str.contains("| `report` | Supporting |"));
    }

    #[test]
    fn test_truncate_path() {
        assert_eq!(truncate_path("short", 10), "short");
        assert_eq!(
            truncate_path("this_is_a_very_long_path", 15),
            "...ry_long_path"
        );
    }
}
