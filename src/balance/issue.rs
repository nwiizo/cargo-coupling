use super::action::RefactoringAction;
use super::issue_type::IssueType;
use super::severity::Severity;

/// A detected coupling issue with refactoring recommendation
#[derive(Debug, Clone)]
pub struct CouplingIssue {
    /// Type of issue
    pub issue_type: IssueType,
    /// Severity of the issue
    pub severity: Severity,
    /// Source component
    pub source: String,
    /// Target component
    pub target: String,
    /// Specific description of this instance
    pub description: String,
    /// Concrete refactoring action to take
    pub refactoring: RefactoringAction,
    /// Balance score that triggered this issue
    pub balance_score: f64,
}
