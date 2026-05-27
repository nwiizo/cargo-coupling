//! Baseline diffing for coupling reports.
//!
//! A diff compares issue identity across two snapshots using the stable key
//! `(issue_type, source, target)`, so ratchet checks can focus on regressions
//! introduced by the current change rather than the codebase's absolute state.

use std::collections::HashSet;

use crate::balance::{CouplingIssue, HealthGrade, IssueType, ProjectBalanceReport, Severity};

/// Stable identity for a coupling issue across snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IssueKey {
    pub issue_type: IssueType,
    pub source: String,
    pub target: String,
}

impl IssueKey {
    pub fn from_issue(issue: &CouplingIssue) -> Self {
        Self {
            issue_type: issue.issue_type,
            source: issue.source.clone(),
            target: issue.target.clone(),
        }
    }
}

/// Difference between a baseline report and the current report.
#[derive(Debug, Clone)]
pub struct BaselineDiff {
    pub new_issues: Vec<CouplingIssue>,
    pub resolved_issues: Vec<CouplingIssue>,
    pub unchanged: usize,
    pub score_delta: f64,
    pub baseline_grade: HealthGrade,
    pub current_grade: HealthGrade,
}

impl BaselineDiff {
    /// New issues at or above `severity`.
    pub fn ratchet_failures(&self, severity: Severity) -> Vec<&CouplingIssue> {
        self.new_issues
            .iter()
            .filter(|issue| issue.severity >= severity)
            .collect()
    }
}

/// Compute a stable-key issue diff from baseline to current.
pub fn diff_reports(
    baseline: &ProjectBalanceReport,
    current: &ProjectBalanceReport,
) -> BaselineDiff {
    let baseline_keys: HashSet<IssueKey> =
        baseline.issues.iter().map(IssueKey::from_issue).collect();
    let current_keys: HashSet<IssueKey> = current.issues.iter().map(IssueKey::from_issue).collect();

    let new_issues = current
        .issues
        .iter()
        .filter(|issue| !baseline_keys.contains(&IssueKey::from_issue(issue)))
        .cloned()
        .collect();

    let resolved_issues = baseline
        .issues
        .iter()
        .filter(|issue| !current_keys.contains(&IssueKey::from_issue(issue)))
        .cloned()
        .collect();

    BaselineDiff {
        new_issues,
        resolved_issues,
        unchanged: baseline_keys.intersection(&current_keys).count(),
        score_delta: current.average_score - baseline.average_score,
        baseline_grade: baseline.health_grade,
        current_grade: current.health_grade,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance::{RefactoringAction, Severity};

    fn issue(
        issue_type: IssueType,
        severity: Severity,
        source: &str,
        target: &str,
    ) -> CouplingIssue {
        CouplingIssue {
            issue_type,
            severity,
            source: source.to_string(),
            target: target.to_string(),
            description: String::new(),
            refactoring: RefactoringAction::General {
                action: String::new(),
            },
            balance_score: 0.5,
        }
    }

    fn report(issues: Vec<CouplingIssue>, score: f64, grade: HealthGrade) -> ProjectBalanceReport {
        ProjectBalanceReport {
            total_couplings: 0,
            balanced_count: 0,
            needs_review: 0,
            needs_refactoring: 0,
            average_score: score,
            health_grade: grade,
            issues_by_severity: Default::default(),
            issues_by_type: Default::default(),
            issues,
            top_priorities: Vec::new(),
        }
    }

    #[test]
    fn diffs_by_stable_issue_key() {
        let unchanged = issue(IssueType::GodModule, Severity::Medium, "a", "too much");
        let resolved = issue(
            IssueType::HighAfferentCoupling,
            Severity::High,
            "2 deps",
            "b",
        );
        let new = issue(
            IssueType::HighEfferentCoupling,
            Severity::High,
            "c",
            "3 deps",
        );

        let baseline = report(
            vec![unchanged.clone(), resolved.clone()],
            0.7,
            HealthGrade::B,
        );
        let current = report(vec![unchanged, new.clone()], 0.6, HealthGrade::C);

        let diff = diff_reports(&baseline, &current);

        assert_eq!(diff.new_issues.len(), 1);
        assert_eq!(diff.new_issues[0].source, new.source);
        assert_eq!(diff.resolved_issues.len(), 1);
        assert_eq!(diff.resolved_issues[0].source, resolved.source);
        assert_eq!(diff.unchanged, 1);
        assert!((diff.score_delta + 0.1).abs() < f64::EPSILON);
        assert_eq!(diff.baseline_grade, HealthGrade::B);
        assert_eq!(diff.current_grade, HealthGrade::C);
    }

    #[test]
    fn ratchet_filters_by_severity() {
        let diff = BaselineDiff {
            new_issues: vec![
                issue(IssueType::GodModule, Severity::Medium, "a", "too much"),
                issue(
                    IssueType::HighEfferentCoupling,
                    Severity::High,
                    "b",
                    "3 deps",
                ),
            ],
            resolved_issues: Vec::new(),
            unchanged: 0,
            score_delta: 0.0,
            baseline_grade: HealthGrade::B,
            current_grade: HealthGrade::B,
        };

        assert_eq!(diff.ratchet_failures(Severity::High).len(), 1);
        assert_eq!(diff.ratchet_failures(Severity::Medium).len(), 2);
    }
}
