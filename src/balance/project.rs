use std::collections::{HashMap, HashSet};

use crate::metrics::dimensions::{Distance, Subdomain};
use crate::metrics::project::ProjectMetrics;

use super::coupling::identify_issues_with_thresholds;
use super::grade::{ProjectBalanceReport, build_grade_rationale, calculate_health_grade};
use super::issue_type::IssueType;
use super::patterns::{analyze_module_coupling, analyze_rust_patterns};
use super::score::{BalanceInterpretation, BalanceScore, IssueThresholds};
use super::severity::Severity;
use super::signals::{analyze_accidental_volatility, analyze_hidden_temporal_coupling};
use super::subdomain::{build_target_subdomain_map, coupling_with_essential_volatility};

pub fn analyze_project_balance(metrics: &ProjectMetrics) -> ProjectBalanceReport {
    analyze_project_balance_with_thresholds(metrics, &IssueThresholds::default())
}

/// Analyze all couplings with custom thresholds
pub fn analyze_project_balance_with_thresholds(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
) -> ProjectBalanceReport {
    let thresholds = thresholds.clone();
    let mut all_issues = Vec::new();
    let mut internal_balance_scores: Vec<BalanceScore> = Vec::new();
    let mut all_balance_scores: Vec<BalanceScore> = Vec::new();
    let target_subdomains = build_target_subdomain_map(metrics);

    // Analyze individual couplings
    // Only INTERNAL couplings affect the health score
    for coupling in &metrics.couplings {
        let effective_coupling = coupling_with_essential_volatility(coupling, &target_subdomains);
        let score = BalanceScore::calculate(&effective_coupling);
        all_balance_scores.push(score.clone());

        // Only count internal couplings for scoring
        if effective_coupling.distance != Distance::DifferentCrate {
            internal_balance_scores.push(score);
            let issues = identify_issues_with_thresholds(&effective_coupling, &thresholds);
            all_issues.extend(issues);
        }
    }

    // Analyze module-level coupling patterns (already filters external)
    let module_issues = analyze_module_coupling(metrics, &thresholds);
    all_issues.extend(module_issues);

    // Analyze Khononov/Rust-specific issues
    let rust_issues = analyze_rust_patterns(metrics, &thresholds);
    all_issues.extend(rust_issues);

    // Analyze git-backed implicit coupling and volatility/subdomain mismatches.
    let temporal_issues = analyze_hidden_temporal_coupling(metrics);
    all_issues.extend(temporal_issues);
    let accidental_volatility_issues = analyze_accidental_volatility(metrics);
    all_issues.extend(accidental_volatility_issues);

    // Tag each Cascading Change Risk with whether the target's volatility is
    // essential (a Core subdomain that genuinely evolves) or accidental (recent
    // git churn that may settle) so the reader can judge whether to act now.
    for issue in &mut all_issues {
        if issue.issue_type == IssueType::CascadingChangeRisk {
            // The subdomain map is keyed by short module name; the issue target is fully qualified.
            let target_key = issue.target.rsplit("::").next().unwrap_or(&issue.target);
            let tag = if matches!(
                target_subdomains.get(target_key),
                Some(Some(Subdomain::Core))
            ) {
                " (essential volatility: target is a Core subdomain that genuinely evolves)"
            } else {
                " (accidental volatility: driven by recent churn — may settle as development stabilizes)"
            };
            issue.description.push_str(tag);
        }
    }

    // Strict mode: filter out Low severity issues to reduce noise
    if thresholds.strict_mode {
        all_issues.retain(|issue| issue.severity >= Severity::Medium);
    }

    // Sort by severity (critical first), then by balance score (worst first)
    all_issues.sort_by(|a, b| {
        b.severity.cmp(&a.severity).then_with(|| {
            a.balance_score
                .partial_cmp(&b.balance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    dedupe_issues_by_stable_key(&mut all_issues);

    // Calculate summary statistics based on INTERNAL couplings only
    let total_couplings = metrics.couplings.len();
    let internal_couplings = internal_balance_scores.len();

    let balanced_count = internal_balance_scores
        .iter()
        .filter(|score| score.is_balanced())
        .count();
    let needs_review = internal_balance_scores
        .iter()
        .filter(|score| score.interpretation == BalanceInterpretation::NeedsReview)
        .count();
    let needs_refactoring = internal_balance_scores
        .iter()
        .filter(|score| score.needs_refactoring())
        .count();

    // Average score based on internal couplings only
    let average_score = if internal_balance_scores.is_empty() {
        1.0 // No internal couplings = perfect score
    } else {
        internal_balance_scores.iter().map(|s| s.score).sum::<f64>()
            / internal_balance_scores.len() as f64
    };

    // Report display counts include diagnostics such as Accidental Volatility.
    let mut issues_by_severity: HashMap<Severity, usize> = HashMap::new();
    for issue in &all_issues {
        *issues_by_severity.entry(issue.severity).or_insert(0) += 1;
    }

    // Count issues by type
    let mut issues_by_type: HashMap<IssueType, usize> = HashMap::new();
    for issue in &all_issues {
        *issues_by_type.entry(issue.issue_type).or_insert(0) += 1;
    }

    // Grading counts exclude diagnostics such as Accidental Volatility.
    // The grade reflects structural defects and essential volatility. Diagnostics
    // (raw git churn contradicting a declared subdomain) are reported but must not
    // grade the project down: accidental volatility routes to the diagnostic, not
    // to scoring (see .claude/rules/grading-integrity.md).
    let mut gradable_by_severity: HashMap<Severity, usize> = HashMap::new();
    for issue in all_issues
        .iter()
        .filter(|issue| !issue.issue_type.is_diagnostic())
    {
        *gradable_by_severity.entry(issue.severity).or_insert(0) += 1;
    }

    // Determine overall health grade based on INTERNAL coupling issues
    let health_grade = calculate_health_grade(&gradable_by_severity, internal_couplings);
    let grade_rationale =
        build_grade_rationale(&all_issues, internal_couplings, thresholds.japanese);

    ProjectBalanceReport {
        total_couplings,
        balanced_count,
        needs_review,
        needs_refactoring,
        average_score,
        health_grade,
        issues_by_severity,
        issues_by_type,
        issues: all_issues,
        top_priorities: Vec::new(), // Will be filled below
        grade_rationale,
    }
    .with_top_priorities(5) // Increased from 3 to 5 for better actionability
}

pub(crate) fn dedupe_issues_by_stable_key(issues: &mut Vec<super::issue::CouplingIssue>) {
    let mut seen = HashSet::new();
    issues.retain(|issue| seen.insert(issue.stable_key()));
}

/// Calculate overall project balance score
///
/// Only considers internal couplings (not external crate dependencies)
/// since external dependencies are outside the developer's control.
pub fn calculate_project_score(metrics: &ProjectMetrics) -> f64 {
    let target_subdomains = build_target_subdomain_map(metrics);

    // Filter to internal couplings only
    let internal_scores: Vec<f64> = metrics
        .couplings
        .iter()
        .filter(|c| c.distance != Distance::DifferentCrate)
        .map(|c| {
            let effective_coupling = coupling_with_essential_volatility(c, &target_subdomains);
            BalanceScore::calculate(&effective_coupling).score
        })
        .collect();

    if internal_scores.is_empty() {
        return 1.0; // No internal couplings = perfect score
    }

    internal_scores.iter().sum::<f64>() / internal_scores.len() as f64
}
