use std::collections::HashMap;

use super::issue::CouplingIssue;
use super::issue_type::IssueType;
use super::rationale::{GradeDimension, GradeRationale, IssueTypeContribution};
use super::severity::Severity;

pub(crate) fn build_grade_rationale(
    issues: &[CouplingIssue],
    internal_couplings: usize,
    japanese: bool,
) -> GradeRationale {
    if issues.is_empty() {
        let summary = if japanese {
            if internal_couplings == 0 {
                "内部結合が 0 件のため、バランスを認定するにはデータが少なすぎます。グレードは B が上限です。".to_string()
            } else if internal_couplings < 10 {
                format!(
                    "内部結合が {} 件で 10 件未満のため、バランスを認定するにはデータが少なすぎます。グレードは B が上限です。",
                    internal_couplings
                )
            } else {
                format!(
                    "{} 件の内部結合に検出対象の問題はありません。問題密度が低いため、このグレードになっています。",
                    internal_couplings
                )
            }
        } else if internal_couplings == 0 {
            "0 internal couplings: too little data to certify balance; grade capped at B."
                .to_string()
        } else if internal_couplings < 10 {
            format!(
                "{} internal coupling(s): fewer than 10 internal couplings, too little data to certify balance; grade capped at B.",
                internal_couplings
            )
        } else {
            format!(
                "No surfaced coupling issues across {} internal coupling(s); grade reflects low issue density.",
                internal_couplings
            )
        };
        return GradeRationale {
            summary,
            ..GradeRationale::empty()
        };
    }

    let mut by_type: HashMap<IssueType, (usize, Severity, usize)> = HashMap::new();
    let mut by_dimension: HashMap<GradeDimension, usize> = HashMap::new();
    let mut high_or_critical = 0;

    for issue in issues {
        let weight = severity_weight(issue.severity);
        let entry = by_type
            .entry(issue.issue_type)
            .or_insert((0, issue.severity, 0));
        entry.0 += 1;
        entry.1 = entry.1.max(issue.severity);
        entry.2 += weight;
        *by_dimension
            .entry(dimension_for_issue(issue.issue_type))
            .or_default() += weight;

        if issue.severity >= Severity::High {
            high_or_critical += 1;
        }
    }

    let mut ranked_types: Vec<_> = by_type
        .into_iter()
        .map(|(issue_type, (count, highest_severity, weighted_score))| {
            (
                IssueTypeContribution {
                    issue_type,
                    count,
                    highest_severity,
                },
                weighted_score,
            )
        })
        .collect();
    ranked_types.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.0.count.cmp(&a.0.count))
            .then_with(|| a.0.issue_type.to_string().cmp(&b.0.issue_type.to_string()))
    });

    let top_issue_types: Vec<_> = ranked_types
        .into_iter()
        .take(3)
        .map(|(contribution, _)| contribution)
        .collect();

    let dominant_dimension = by_dimension
        .into_iter()
        .max_by_key(|(_, score)| *score)
        .map(|(dimension, _)| dimension);

    let volatility_issue_count = issues
        .iter()
        .filter(|issue| dimension_for_issue(issue.issue_type) == GradeDimension::Volatility)
        .count();
    let accidental_count = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::AccidentalVolatility)
        .count();
    let volatility_note = if volatility_issue_count > 0 {
        let accidental_suffix = if accidental_count > 0 {
            if japanese {
                format!(
                    " (偶発的な変更頻度 {} 件を含む。診断のためグレードには算入しない)",
                    accidental_count
                )
            } else {
                format!(
                    ", including {} accidental-volatility diagnostic(s) reported but not graded",
                    accidental_count
                )
            }
        } else {
            String::new()
        };
        if japanese {
            Some(format!(
                "変更頻度/チャーンが {} 件の問題として影響しています{}。",
                volatility_issue_count, accidental_suffix
            ))
        } else {
            Some(format!(
                "Volatility/churn contributes through {} issue(s){}.",
                volatility_issue_count, accidental_suffix
            ))
        }
    } else {
        None
    };

    let top_phrase = top_issue_types
        .iter()
        .map(|item| {
            if japanese {
                format!(
                    "{} ({})",
                    issue_type_japanese_label(item.issue_type),
                    item.count
                )
            } else {
                format!("{} ({})", item.issue_type, item.count)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let severity_phrase = if japanese {
        if high_or_critical > 0 {
            format!("高/緊急の問題 {} 件", high_or_critical)
        } else {
            format!("中/低の問題 {} 件", issues.len())
        }
    } else if high_or_critical > 0 {
        format!("{} high/critical issue(s)", high_or_critical)
    } else {
        format!("{} medium/low issue(s)", issues.len())
    };
    let dimension_phrase = dominant_dimension
        .map(|dimension| {
            if japanese {
                format!(
                    "。最大の要因は{}です",
                    grade_dimension_japanese_label(dimension)
                )
            } else {
                format!("; {} is the largest contributor", dimension)
            }
        })
        .unwrap_or_default();
    let note_phrase = volatility_note
        .as_ref()
        .map(|note| {
            if japanese {
                note.clone()
            } else {
                format!(" {}", note)
            }
        })
        .unwrap_or_default();

    let summary = if japanese {
        format!(
            "{}が主な理由です。特に {} が目立ちます{}。{note_phrase}",
            severity_phrase, top_phrase, dimension_phrase
        )
    } else {
        format!(
            "Driven by {}, led by {}{}.{note_phrase}",
            severity_phrase, top_phrase, dimension_phrase
        )
    };

    GradeRationale {
        summary,
        top_issue_types,
        dominant_dimension,
        volatility_note,
    }
}

fn grade_dimension_japanese_label(dimension: GradeDimension) -> &'static str {
    match dimension {
        GradeDimension::Strength => "結合強度",
        GradeDimension::Distance => "距離",
        GradeDimension::Volatility => "変更頻度",
    }
}

fn issue_type_japanese_label(issue_type: IssueType) -> &'static str {
    match issue_type {
        IssueType::GlobalComplexity => "グローバル複雑性",
        IssueType::CascadingChangeRisk => "変更波及リスク",
        IssueType::InappropriateIntimacy => "不適切な親密さ",
        IssueType::HighEfferentCoupling => "出力依存過多",
        IssueType::HighAfferentCoupling => "入力依存過多",
        IssueType::UnnecessaryAbstraction => "過剰な抽象化",
        IssueType::CircularDependency => "循環依存",
        IssueType::HiddenCoupling => "隠れた結合",
        IssueType::AccidentalVolatility => "偶発的な変更頻度",
        IssueType::ScatteredExternalCoupling => "外部クレート結合の分散",
        IssueType::ShallowModule => "浅いモジュール",
        IssueType::PassThroughMethod => "パススルーメソッド",
        IssueType::HighCognitiveLoad => "高認知負荷",
        IssueType::GodModule => "神モジュール",
        IssueType::PublicFieldExposure => "公開フィールド",
        IssueType::PrimitiveObsession => "プリミティブ過多",
    }
}

fn severity_weight(severity: Severity) -> usize {
    match severity {
        Severity::Critical => 4,
        Severity::High => 3,
        Severity::Medium => 2,
        Severity::Low => 1,
    }
}

fn dimension_for_issue(issue_type: IssueType) -> GradeDimension {
    match issue_type {
        IssueType::CascadingChangeRisk
        | IssueType::HiddenCoupling
        | IssueType::AccidentalVolatility => GradeDimension::Volatility,
        IssueType::InappropriateIntimacy
        | IssueType::PublicFieldExposure
        | IssueType::PrimitiveObsession
        | IssueType::ShallowModule
        | IssueType::PassThroughMethod => GradeDimension::Strength,
        IssueType::GlobalComplexity
        | IssueType::ScatteredExternalCoupling
        | IssueType::HighEfferentCoupling
        | IssueType::HighAfferentCoupling
        | IssueType::UnnecessaryAbstraction
        | IssueType::CircularDependency
        | IssueType::HighCognitiveLoad
        | IssueType::GodModule => GradeDimension::Distance,
    }
}

// ===== Health Grading =====
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthGrade {
    /// Over-optimized signal; the project may be chasing too little coupling.
    S,
    /// Coupling is appropriate for the architecture.
    A,
    /// Minor issues exist but remain manageable.
    B,
    /// Structural issues need planned improvement.
    C,
    /// Significant issues are affecting maintainability.
    D,
    /// Critical issues are blocking safe change.
    F,
}

impl HealthGrade {
    /// Single-letter representation (S, A, B, C, D, F).
    ///
    /// Used for compact, machine-readable output such as the history
    /// timeline where the verbose `Display` form is too noisy.
    pub fn letter(&self) -> char {
        match self {
            HealthGrade::S => 'S',
            HealthGrade::A => 'A',
            HealthGrade::B => 'B',
            HealthGrade::C => 'C',
            HealthGrade::D => 'D',
            HealthGrade::F => 'F',
        }
    }
}

impl std::fmt::Display for HealthGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthGrade::S => write!(f, "S (Over-optimized! Real code has some issues. Ship it!)"),
            HealthGrade::A => write!(f, "A (Well-balanced)"),
            HealthGrade::B => write!(f, "B (Healthy)"),
            HealthGrade::C => write!(f, "C (Room for improvement)"),
            HealthGrade::D => write!(f, "D (Attention needed)"),
            HealthGrade::F => write!(f, "F (Immediate action required)"),
        }
    }
}

/// Calculate health grade based on multiple quality factors
///
/// Unlike the previous version that only checked for issues,
/// this version also considers positive quality indicators:
/// - Contract coupling rate (trait usage)
/// - Balance score distribution
/// - Internal coupling complexity
pub(crate) fn calculate_health_grade(
    issues_by_severity: &HashMap<Severity, usize>,
    internal_couplings: usize,
) -> HealthGrade {
    let critical = *issues_by_severity.get(&Severity::Critical).unwrap_or(&0);
    let high = *issues_by_severity.get(&Severity::High).unwrap_or(&0);
    let medium = *issues_by_severity.get(&Severity::Medium).unwrap_or(&0);

    // No internal couplings = B (not A - we can't assess quality without data)
    if internal_couplings == 0 {
        return HealthGrade::B;
    }

    // F: Multiple critical issues
    if critical > 3 {
        return HealthGrade::F;
    }

    // Calculate issue density (issues per internal coupling)
    let high_density = high as f64 / internal_couplings as f64;
    let medium_density = medium as f64 / internal_couplings as f64;
    let total_issue_density = (critical + high + medium) as f64 / internal_couplings as f64;

    // D: Critical issues or very high issue density (> 5% high)
    if critical > 0 || high_density > 0.05 {
        return HealthGrade::D;
    }

    // C: Any high issues OR high medium density (> 25%)
    // Projects with structural issues that need attention
    if high > 0 || medium_density > 0.25 {
        return HealthGrade::C;
    }

    // B: Some medium issues but manageable (> 10% medium density)
    if medium_density > 0.10 || total_issue_density > 0.15 {
        return HealthGrade::B;
    }

    // S: Over-optimized! Too few issues (< 5%) = you're probably over-engineering
    // This is a WARNING, not a reward. Stop refactoring!
    if high == 0 && medium_density <= 0.05 && internal_couplings >= 20 {
        return HealthGrade::S;
    }

    // A: Well-balanced - no high issues AND reasonable medium issues (5-10%)
    // This is the ideal target grade
    if high == 0 && medium_density <= 0.10 && internal_couplings >= 10 {
        return HealthGrade::A;
    }

    // Default to B for projects with few issues
    HealthGrade::B
}

/// Complete project balance analysis report
#[derive(Debug)]
pub struct ProjectBalanceReport {
    /// Total couplings considered in the report.
    pub total_couplings: usize,
    /// Number of internal couplings considered balanced or acceptable.
    pub balanced_count: usize,
    /// Number of internal couplings that need review.
    pub needs_review: usize,
    /// Number of internal couplings that need refactoring.
    pub needs_refactoring: usize,
    /// Average balance score across internal couplings.
    pub average_score: f64,
    /// Overall project health grade derived from issue density.
    pub health_grade: HealthGrade,
    /// Issue counts grouped by severity.
    pub issues_by_severity: HashMap<Severity, usize>,
    /// Issue counts grouped by issue type.
    pub issues_by_type: HashMap<IssueType, usize>,
    /// All detected issues after threshold and strict-mode filtering.
    pub issues: Vec<CouplingIssue>,
    /// Highest-priority issues selected for concise reporting.
    pub top_priorities: Vec<CouplingIssue>,
    /// Concise explanation of why the health grade was assigned.
    pub grade_rationale: GradeRationale,
}

impl ProjectBalanceReport {
    /// Add top N priority issues
    pub(crate) fn with_top_priorities(mut self, n: usize) -> Self {
        self.top_priorities = self.issues.iter().take(n).cloned().collect();
        self
    }

    /// Get issues grouped by type
    pub fn issues_grouped_by_type(&self) -> HashMap<IssueType, Vec<&CouplingIssue>> {
        let mut grouped: HashMap<IssueType, Vec<&CouplingIssue>> = HashMap::new();
        for issue in &self.issues {
            grouped.entry(issue.issue_type).or_default().push(issue);
        }
        grouped
    }
}
