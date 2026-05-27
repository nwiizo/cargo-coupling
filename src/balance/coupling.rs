// ===== Issue Detection =====

use crate::metrics::coupling::CouplingMetrics;
use crate::metrics::dimensions::{Distance, IntegrationStrength};
use crate::volatility::Volatility;

use super::action::RefactoringAction;
use super::issue::CouplingIssue;
use super::issue_type::IssueType;
use super::labels::extract_type_name;
use super::score::{BalanceScore, IssueThresholds};
use super::severity::Severity;

/// Whether a module is the binary entrypoint (e.g. `crate::main`).
///
/// An entrypoint must wire the whole application together, so its high fan-out
/// is expected by design, not a maintainability defect.
pub(crate) fn is_entrypoint_module(name: &str) -> bool {
    name.rsplit("::").next() == Some("main")
}

/// Whether a module is the crate-root re-export facade (module name == crate name).
///
/// `crate-name::crate_name` (i.e. `lib.rs`) is a stable Contract surface; coupling
/// to it is not intrusive coupling to a volatile implementation.
pub(crate) fn is_crate_root_facade(name: &str) -> bool {
    match name.split_once("::") {
        Some((krate, rest)) => !rest.contains("::") && rest == krate.replace('-', "_"),
        None => false,
    }
}

/// Identify issues in a coupling relationship
pub fn identify_issues(coupling: &CouplingMetrics) -> Vec<CouplingIssue> {
    identify_issues_with_thresholds(coupling, &IssueThresholds::default())
}

/// Identify issues with custom thresholds
pub fn identify_issues_with_thresholds(
    coupling: &CouplingMetrics,
    _thresholds: &IssueThresholds,
) -> Vec<CouplingIssue> {
    let mut issues = Vec::new();

    // Skip ALL external crate dependencies - we can't control them
    // Focus only on internal workspace coupling issues
    if coupling.distance == Distance::DifferentCrate {
        return issues;
    }

    let balance = BalanceScore::calculate(coupling);

    // The crate-root re-export facade is a stable Contract, not a volatile
    // implementation — coupling to it is not an intrusive/cascading defect.
    let target_is_facade = is_crate_root_facade(&coupling.target);

    // Pattern 1: Strong (Intrusive) + Far (DifferentModule). Per the Balanced
    // Coupling model, the dangerous quadrant (Strong+Far+High volatility) is owned
    // by Cascading Change Risk below; here we classify the lower-volatility cases:
    // Low volatility = "Acceptable" (Minor), Medium = Global Complexity (review).
    if coupling.strength == IntegrationStrength::Intrusive
        && coupling.distance == Distance::DifferentModule
        && coupling.volatility != Volatility::High
        && !target_is_facade
    {
        let (severity, description) = if coupling.volatility == Volatility::Low {
            (
                Severity::Low,
                format!(
                    "Acceptable: intrusive coupling to stable {} across a module boundary (low volatility neutralizes the distance)",
                    coupling.target
                ),
            )
        } else {
            (
                Severity::Medium,
                format!(
                    "Intrusive coupling to {} across a module boundary",
                    coupling.target
                ),
            )
        };
        issues.push(CouplingIssue {
            issue_type: IssueType::GlobalComplexity,
            severity,
            source: coupling.source.clone(),
            target: coupling.target.clone(),
            description,
            refactoring: RefactoringAction::IntroduceTrait {
                suggested_name: format!("{}Trait", extract_type_name(&coupling.target)),
                methods: vec!["// Extract required methods".to_string()],
            },
            balance_score: balance.score,
        });
    }

    // Pattern 2: Cascading Change Risk (intrusive coupling + high volatility)
    // Only flag if volatility is actually high (from Git data)
    if coupling.strength == IntegrationStrength::Intrusive
        && coupling.volatility == Volatility::High
        && !target_is_facade
    {
        issues.push(CouplingIssue {
            issue_type: IssueType::CascadingChangeRisk,
            severity: Severity::High,
            source: coupling.source.clone(),
            target: coupling.target.clone(),
            description: format!(
                "Intrusive coupling to frequently-changed component {}",
                coupling.target,
            ),
            refactoring: RefactoringAction::StabilizeInterface {
                interface_name: format!("{}Interface", extract_type_name(&coupling.target)),
            },
            balance_score: balance.score,
        });
    }

    // Pattern 3: Inappropriate Intimacy (Intrusive coupling across DIFFERENT MODULE only)
    // Same module intrusive coupling is fine (it's cohesion)
    if coupling.strength == IntegrationStrength::Intrusive
        && coupling.distance == Distance::DifferentModule
        && balance.score < 0.5
        && !target_is_facade
    {
        // Only add if not already added as GlobalComplexity
        if !issues
            .iter()
            .any(|i| i.issue_type == IssueType::GlobalComplexity)
        {
            issues.push(CouplingIssue {
                issue_type: IssueType::InappropriateIntimacy,
                severity: Severity::Medium,
                source: coupling.source.clone(),
                target: coupling.target.clone(),
                description: format!(
                    "Direct internal access to {} across module boundary",
                    coupling.target,
                ),
                refactoring: RefactoringAction::IntroduceTrait {
                    suggested_name: format!("{}Api", extract_type_name(&coupling.target)),
                    methods: vec!["// Expose only necessary operations".to_string()],
                },
                balance_score: balance.score,
            });
        }
    }

    // Pattern 4: Unnecessary Abstraction - DISABLED
    // This pattern generates too much noise and is rarely actionable
    // Trait abstractions are generally good, even for nearby stable components

    issues
}
