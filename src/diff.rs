//! Baseline diffing for coupling reports.
//!
//! A diff compares issue identity across two snapshots using the stable key
//! `(issue_type, source, target)`, so ratchet checks can focus on regressions
//! introduced by the current change rather than the codebase's absolute state.

use std::collections::HashSet;

// Consume the crate's published facade rather than deep `balance::*` paths: the
// re-exported surface stays stable when the balance package reorganizes internally.
use crate::history::RefAnalysis;
use crate::{CouplingIssue, HealthGrade, IssueKey, ProjectBalanceReport, Severity};

/// Difference between a baseline report and the current report.
#[derive(Debug, Clone)]
pub struct BaselineDiff {
    /// Issues present only in the current report.
    pub new_issues: Vec<CouplingIssue>,
    /// Issues present only in the baseline report.
    pub resolved_issues: Vec<CouplingIssue>,
    /// Number of stable issue keys present in both reports.
    pub unchanged: usize,
    /// Current average score minus baseline average score.
    pub score_delta: f64,
    /// Baseline health grade.
    pub baseline_grade: HealthGrade,
    /// Current health grade.
    pub current_grade: HealthGrade,
}

impl BaselineDiff {
    /// New issues at or above `severity`.
    pub fn ratchet_failures(&self, severity: Severity) -> Vec<&CouplingIssue> {
        self.new_issues
            .iter()
            .filter(|issue| issue.meets(severity))
            .collect()
    }
}

/// Compute a stable-key issue diff from baseline to current.
pub fn diff_reports(
    baseline: &ProjectBalanceReport,
    current: &ProjectBalanceReport,
) -> BaselineDiff {
    let baseline_keys: HashSet<IssueKey> = baseline.issues.iter().map(IssueKey::from).collect();
    let current_keys: HashSet<IssueKey> = current.issues.iter().map(IssueKey::from).collect();

    let mut seen_new = HashSet::new();
    let new_issues = current
        .issues
        .iter()
        .filter_map(|issue| {
            let key = IssueKey::from(issue);
            (!baseline_keys.contains(&key) && seen_new.insert(key)).then(|| issue.clone())
        })
        .collect();

    let mut seen_resolved = HashSet::new();
    let resolved_issues = baseline
        .issues
        .iter()
        .filter_map(|issue| {
            let key = IssueKey::from(issue);
            (!current_keys.contains(&key) && seen_resolved.insert(key)).then(|| issue.clone())
        })
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

/// Diff a baseline git-ref analysis against the current report.
pub fn diff_ref_analysis(baseline: &RefAnalysis, current: &ProjectBalanceReport) -> BaselineDiff {
    diff_reports(&baseline.report, current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance::rationale::GradeRationale;
    use crate::{IssueType, RefactoringAction};

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
            grade_rationale: GradeRationale::empty(),
        }
    }

    #[test]
    fn issue_key_normalizes_efferent_dependency_count() {
        let three = IssueKey::from(&issue(
            IssueType::HighEfferentCoupling,
            Severity::High,
            "m",
            "3 dependencies",
        ));
        let five = IssueKey::from(&issue(
            IssueType::HighEfferentCoupling,
            Severity::High,
            "m",
            "5 dependencies",
        ));
        assert_eq!(three, five, "count change must not change the key");
        assert_eq!(three.target, "<dependency-count>");
        assert_eq!(three.source, "m", "source passes through unchanged");
    }

    #[test]
    fn issue_key_normalizes_afferent_dependent_count() {
        let three = IssueKey::from(&issue(
            IssueType::HighAfferentCoupling,
            Severity::High,
            "3 dependents",
            "m",
        ));
        let five = IssueKey::from(&issue(
            IssueType::HighAfferentCoupling,
            Severity::High,
            "5 dependents",
            "m",
        ));
        assert_eq!(three, five);
        assert_eq!(three.source, "<dependent-count>");
        assert_eq!(three.target, "m", "target passes through unchanged");
    }

    #[test]
    fn issue_key_preserves_non_count_fields() {
        // A non-count issue type keeps source/target verbatim.
        let k = IssueKey::from(&issue(
            IssueType::GodModule,
            Severity::High,
            "src_mod",
            "dst_mod",
        ));
        assert_eq!(k.source, "src_mod");
        assert_eq!(k.target, "dst_mod");

        // Efferent issue whose target is NOT a count is preserved (guard must stay false).
        let e = IssueKey::from(&issue(
            IssueType::HighEfferentCoupling,
            Severity::High,
            "m",
            "not a count",
        ));
        assert_eq!(e.target, "not a count");

        // Afferent issue whose source is NOT a count is preserved.
        let a = IssueKey::from(&issue(
            IssueType::HighAfferentCoupling,
            Severity::High,
            "not a count",
            "m",
        ));
        assert_eq!(a.source, "not a count");
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

    #[test]
    fn high_coupling_count_targets_do_not_create_new_issue_keys() {
        let baseline = report(
            vec![
                issue(
                    IssueType::HighEfferentCoupling,
                    Severity::High,
                    "web::server",
                    "2 dependencies",
                ),
                issue(
                    IssueType::HighAfferentCoupling,
                    Severity::High,
                    "2 dependents",
                    "cli_output",
                ),
            ],
            0.7,
            HealthGrade::B,
        );
        let current = report(
            vec![
                issue(
                    IssueType::HighEfferentCoupling,
                    Severity::High,
                    "web::server",
                    "3 dependencies",
                ),
                issue(
                    IssueType::HighAfferentCoupling,
                    Severity::High,
                    "3 dependents",
                    "cli_output",
                ),
            ],
            0.6,
            HealthGrade::C,
        );

        let diff = diff_reports(&baseline, &current);

        assert!(diff.new_issues.is_empty());
        assert!(diff.resolved_issues.is_empty());
        assert_eq!(diff.unchanged, 2);
    }

    #[test]
    fn diff_deduplicates_same_key_issues() {
        let duplicate_new = issue(
            IssueType::CascadingChangeRisk,
            Severity::High,
            "web::server",
            "cli_output",
        );
        let duplicate_resolved = issue(
            IssueType::GlobalComplexity,
            Severity::Medium,
            "history",
            "volatility",
        );
        let baseline = report(
            vec![duplicate_resolved.clone(), duplicate_resolved],
            0.7,
            HealthGrade::B,
        );
        let current = report(
            vec![duplicate_new.clone(), duplicate_new],
            0.6,
            HealthGrade::C,
        );

        let diff = diff_reports(&baseline, &current);

        assert_eq!(diff.new_issues.len(), 1);
        assert_eq!(diff.resolved_issues.len(), 1);
        let new_keys: HashSet<IssueKey> = diff.new_issues.iter().map(IssueKey::from).collect();
        let resolved_keys: HashSet<IssueKey> =
            diff.resolved_issues.iter().map(IssueKey::from).collect();
        assert_eq!(new_keys.len(), diff.new_issues.len());
        assert_eq!(resolved_keys.len(), diff.resolved_issues.len());
    }
}
