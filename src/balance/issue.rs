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

impl CouplingIssue {
    /// Stable identity of this issue across snapshots.
    ///
    /// Lives here (not in the diff layer) because the normalization depends on how
    /// this module formats count-only sources/targets — when that formatting
    /// changes, the key logic must change with it, atomically.
    pub fn stable_key(&self) -> IssueKey {
        IssueKey {
            issue_type: self.issue_type,
            source: match self.issue_type {
                IssueType::HighAfferentCoupling if is_count_target(&self.source, "dependents") => {
                    "<dependent-count>".to_string()
                }
                _ => self.source.clone(),
            },
            target: match self.issue_type {
                IssueType::HighEfferentCoupling
                    if is_count_target(&self.target, "dependencies") =>
                {
                    "<dependency-count>".to_string()
                }
                _ => self.target.clone(),
            },
        }
    }

    /// Whether this issue is at least as severe as `floor`.
    pub fn meets(&self, floor: Severity) -> bool {
        self.severity >= floor
    }
}

/// Stable identity for a coupling issue across snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IssueKey {
    /// Issue category used as the primary diff discriminator.
    pub issue_type: IssueType,
    /// Normalized issue source; count-only sources are canonicalized.
    pub source: String,
    /// Normalized issue target; count-only targets are canonicalized.
    pub target: String,
}

impl From<&CouplingIssue> for IssueKey {
    fn from(issue: &CouplingIssue) -> Self {
        issue.stable_key()
    }
}

fn is_count_target(value: &str, unit: &str) -> bool {
    let Some((count, suffix)) = value.split_once(' ') else {
        return false;
    };
    count.chars().all(|c| c.is_ascii_digit()) && suffix == unit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_count_target_requires_digits_and_exact_unit() {
        assert!(is_count_target("3 dependencies", "dependencies"));
        assert!(is_count_target("12 dependents", "dependents"));
        // digits but wrong unit -> false (guards against `&&` -> `||`).
        assert!(!is_count_target("3 widgets", "dependencies"));
        // non-digit count -> false.
        assert!(!is_count_target("many dependencies", "dependencies"));
        // no space separator -> false.
        assert!(!is_count_target("dependencies", "dependencies"));
    }

    #[test]
    fn meets_compares_severity_floor() {
        let issue = CouplingIssue {
            issue_type: IssueType::GodModule,
            severity: Severity::Medium,
            source: String::new(),
            target: String::new(),
            description: String::new(),
            refactoring: RefactoringAction::General {
                action: String::new(),
            },
            balance_score: 0.5,
        };
        assert!(issue.meets(Severity::Low));
        assert!(issue.meets(Severity::Medium));
        assert!(!issue.meets(Severity::High));
    }
}
