#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GradeDimension {
    /// Coupling strength is the dominant issue driver.
    Strength,
    /// Coupling distance is the dominant issue driver.
    Distance,
    /// Volatility or churn is the dominant issue driver.
    Volatility,
}

impl std::fmt::Display for GradeDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GradeDimension::Strength => write!(f, "strength"),
            GradeDimension::Distance => write!(f, "distance"),
            GradeDimension::Volatility => write!(f, "volatility"),
        }
    }
}

/// Top issue-type contribution used to explain a project grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueTypeContribution {
    /// Issue type contributing to the grade.
    pub issue_type: IssueType,
    /// Number of surfaced issues of this type.
    pub count: usize,
    /// Highest severity observed for this issue type.
    pub highest_severity: Severity,
}

/// Short explanation of why a project received its health grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradeRationale {
    /// Human-readable one-line explanation.
    pub summary: String,
    /// Highest-impact issue types by severity-weighted count.
    pub top_issue_types: Vec<IssueTypeContribution>,
    /// Dominant coupling dimension behind the surfaced issues.
    pub dominant_dimension: Option<GradeDimension>,
    /// Extra note when git churn or accidental volatility dominates.
    pub volatility_note: Option<String>,
}

impl GradeRationale {
    /// Empty rationale for hand-built test fixtures.
    pub fn empty() -> Self {
        Self {
            summary: "No surfaced coupling issues; grade reflects low issue density.".to_string(),
            top_issue_types: Vec::new(),
            dominant_dimension: None,
            volatility_note: None,
        }
    }
}

/// Specific refactoring actions
use super::issue_type::IssueType;
use super::severity::Severity;
