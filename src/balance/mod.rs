//! Balance scoring and issue classification for coupling analysis.
//!
//! This module turns raw coupling metrics into health grades, detected issues,
//! and refactoring recommendations so reports can explain why a relationship is
//! well-balanced or costly to change.

pub mod action;
pub mod coupling;
pub mod external_crates;
pub mod grade;
pub mod issue;
pub mod issue_type;
pub mod issues;
pub mod labels;
pub mod patterns;
pub mod project;
pub mod rationale;
pub mod score;
pub mod severity;
pub mod signals;
pub mod subdomain;

pub use action::RefactoringAction;
pub use coupling::{identify_issues, identify_issues_with_thresholds};
pub use external_crates::{
    CrateStability, SCATTERED_EXTERNAL_BREADTH_THRESHOLD, classify_crate_stability,
    detect_scattered_external_coupling, is_external_crate, should_reduce_severity,
    should_skip_crate,
};
pub use grade::{HealthGrade, ProjectBalanceReport};
pub use issue::CouplingIssue;
pub use issue_type::IssueType;
pub use labels::{distance_label, strength_label, volatility_label};
pub use project::{
    analyze_project_balance, analyze_project_balance_with_thresholds, calculate_project_score,
};
pub use rationale::{GradeDimension, GradeRationale, IssueTypeContribution};
pub use score::{BalanceInterpretation, BalanceScore, IssueThresholds};
pub use severity::Severity;

#[cfg(test)]
use crate::metrics::coupling::CouplingMetrics;
#[cfg(test)]
use crate::metrics::dimensions::{Distance, IntegrationStrength, Subdomain};
#[cfg(test)]
use crate::metrics::project::ProjectMetrics;
#[cfg(test)]
use crate::volatility::Volatility;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::balance::coupling::{is_crate_root_facade, is_entrypoint_module};
    use crate::balance::grade::{build_grade_rationale, calculate_health_grade};
    use crate::metrics::dimensions::Visibility;
    use crate::metrics::module::ModuleMetrics;
    use crate::volatility::TemporalCoupling;

    #[test]
    fn entrypoint_module_detection() {
        assert!(is_entrypoint_module("cargo-coupling::main"));
        assert!(is_entrypoint_module("main"));
        assert!(!is_entrypoint_module("cargo-coupling::balance"));
        assert!(!is_entrypoint_module("cargo-coupling::main_loop"));
    }

    #[test]
    fn crate_root_facade_detection() {
        // crate name (hyphenated) -> root module (underscored) == the re-export facade.
        assert!(is_crate_root_facade("cargo-coupling::cargo_coupling"));
        assert!(is_crate_root_facade("myapp::myapp"));
        // real submodules are not the facade.
        assert!(!is_crate_root_facade("cargo-coupling::balance"));
        assert!(!is_crate_root_facade("cargo-coupling::web::graph"));
        assert!(!is_crate_root_facade("balance"));
    }

    fn make_coupling(
        strength: IntegrationStrength,
        distance: Distance,
        volatility: Volatility,
    ) -> CouplingMetrics {
        CouplingMetrics::new(
            "source::module".to_string(),
            "target::module".to_string(),
            strength,
            distance,
            volatility,
        )
    }

    #[test]
    fn test_balance_ideal_close() {
        // Strong coupling + close distance = good (cohesion)
        let coupling = make_coupling(
            IntegrationStrength::Intrusive,
            Distance::SameModule,
            Volatility::Low,
        );
        let score = BalanceScore::calculate(&coupling);
        assert!(score.is_balanced(), "Score: {}", score.score);
    }

    #[test]
    fn test_balance_ideal_far() {
        // Weak coupling + far distance = good (loose coupling)
        let coupling = make_coupling(
            IntegrationStrength::Contract,
            Distance::DifferentCrate,
            Volatility::Low,
        );
        let score = BalanceScore::calculate(&coupling);
        assert!(score.is_balanced(), "Score: {}", score.score);
    }

    #[test]
    fn test_balance_bad_global_complexity() {
        // Strong coupling + far distance = bad (global complexity)
        let coupling = make_coupling(
            IntegrationStrength::Intrusive,
            Distance::DifferentCrate,
            Volatility::Low,
        );
        let score = BalanceScore::calculate(&coupling);
        assert!(
            score.needs_refactoring(),
            "Score: {}, should need refactoring",
            score.score
        );
    }

    #[test]
    fn test_balance_bad_cascading() {
        // Strong coupling + high volatility = bad
        let coupling = make_coupling(
            IntegrationStrength::Intrusive,
            Distance::SameModule,
            Volatility::High,
        );
        let score = BalanceScore::calculate(&coupling);
        assert!(
            !score.is_balanced(),
            "Score: {}, should not be balanced due to volatility",
            score.score
        );
    }

    #[test]
    fn test_identify_global_complexity() {
        // Note: DifferentCrate is now filtered out (external deps)
        // Test with DifferentModule which is still flagged for internal modules
        let coupling = make_coupling(
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::Low,
        );
        let issues = identify_issues(&coupling);
        assert!(
            !issues.is_empty(),
            "Should identify global complexity issue for internal cross-module coupling"
        );
        assert!(
            issues
                .iter()
                .any(|i| i.issue_type == IssueType::GlobalComplexity)
        );
    }

    #[test]
    fn test_external_crates_are_skipped() {
        // External crate dependencies should not generate issues
        let coupling = make_coupling(
            IntegrationStrength::Intrusive,
            Distance::DifferentCrate,
            Volatility::Low,
        );
        let issues = identify_issues(&coupling);
        assert!(
            issues.is_empty(),
            "External crate dependencies should be skipped"
        );
    }

    #[test]
    fn test_identify_cascading_change_requires_far_distance() {
        let close_coupling = make_coupling(
            IntegrationStrength::Intrusive,
            Distance::SameModule,
            Volatility::High,
        );
        let close_issues = identify_issues(&close_coupling);
        assert!(
            !close_issues
                .iter()
                .any(|i| i.issue_type == IssueType::CascadingChangeRisk),
            "Intrusive + High + SameModule is cohesion, not cascading risk"
        );

        let far_coupling = make_coupling(
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        );
        let far_issues = identify_issues(&far_coupling);
        assert!(
            far_issues
                .iter()
                .any(|i| i.issue_type == IssueType::CascadingChangeRisk),
            "Intrusive + High + DifferentModule should detect CascadingChangeRisk"
        );
    }

    #[test]
    fn test_identify_inappropriate_intimacy() {
        // Intrusive + DifferentModule now detects GlobalComplexity (not InappropriateIntimacy)
        // because they overlap and GlobalComplexity takes precedence
        let coupling = make_coupling(
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::Low,
        );
        let issues = identify_issues(&coupling);
        assert!(
            issues
                .iter()
                .any(|i| i.issue_type == IssueType::GlobalComplexity),
            "Intrusive + DifferentModule should detect GlobalComplexity"
        );
    }

    #[test]
    fn test_no_issues_for_balanced() {
        // Model coupling to different module with low volatility
        let coupling = make_coupling(
            IntegrationStrength::Model,
            Distance::DifferentModule,
            Volatility::Low,
        );
        let issues = identify_issues(&coupling);
        // Model coupling should have no issues (only Intrusive triggers issues)
        assert!(
            issues.is_empty(),
            "Model coupling should not generate issues"
        );
    }

    #[test]
    fn test_hidden_coupling_detected_without_code_dependency() {
        let mut metrics = ProjectMetrics::new();
        metrics.add_module(ModuleMetrics::new(
            PathBuf::from("src/pricing.rs"),
            "pricing".to_string(),
        ));
        metrics.add_module(ModuleMetrics::new(
            PathBuf::from("src/invoicing.rs"),
            "invoicing".to_string(),
        ));
        metrics.temporal_couplings.push(TemporalCoupling {
            file_a: "src/pricing.rs".to_string(),
            file_b: "src/invoicing.rs".to_string(),
            co_change_count: 6,
            coupling_ratio: 0.75,
        });

        let report = analyze_project_balance(&metrics);

        let issue = report
            .issues
            .iter()
            .find(|issue| issue.issue_type == IssueType::HiddenCoupling)
            .expect("strong co-change without code dependency should be a hidden coupling issue");
        assert_eq!(issue.source, "invoicing");
        assert_eq!(issue.target, "pricing");
        assert_eq!(issue.severity, Severity::Medium);
        assert!(issue.description.contains("75% ratio"));
        assert!(issue.description.contains("6 co-changes"));
    }

    #[test]
    fn accidental_volatility_diagnostics_do_not_lower_the_grade() {
        // Same medium count: as structural defects it degrades the grade, as
        // accidental-volatility diagnostics it must not (churn routes to the
        // diagnostic, not to scoring — grading-integrity rule).
        let internal = 100;
        let mediums = 15; // 15% density: above the 10% A threshold

        let mut structural: HashMap<Severity, usize> = HashMap::new();
        structural.insert(Severity::Medium, mediums);
        let degraded = calculate_health_grade(&structural, internal);
        assert_ne!(degraded, HealthGrade::A);

        // Diagnostics are excluded before the grade call (see balance/project.rs);
        // an empty gradable map with enough couplings certifies A or S.
        let gradable: HashMap<Severity, usize> = HashMap::new();
        let ungraded = calculate_health_grade(&gradable, internal);
        assert!(matches!(ungraded, HealthGrade::A | HealthGrade::S));

        assert!(IssueType::AccidentalVolatility.is_diagnostic());
        assert!(!IssueType::GodModule.is_diagnostic());
    }

    #[test]
    fn test_hidden_coupling_skips_crate_root_facade() {
        let mut metrics = ProjectMetrics::new();
        metrics.add_module(ModuleMetrics::new(
            PathBuf::from("src/lib.rs"),
            "lib".to_string(),
        ));
        metrics.add_module(ModuleMetrics::new(
            PathBuf::from("src/cli_output.rs"),
            "cli_output".to_string(),
        ));
        metrics.temporal_couplings.push(TemporalCoupling {
            file_a: "src/lib.rs".to_string(),
            file_b: "src/cli_output.rs".to_string(),
            co_change_count: 8,
            coupling_ratio: 0.8,
        });

        let report = analyze_project_balance(&metrics);

        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.issue_type == IssueType::HiddenCoupling),
            "co-change with the crate-root facade (lib.rs) is expected by design"
        );
    }

    #[test]
    fn test_hidden_coupling_skips_adjacent_modules_and_requires_evidence_for_high() {
        // Parent/child co-change is cohesion, not hidden coupling.
        let mut metrics = ProjectMetrics::new();
        metrics.add_module(ModuleMetrics::new(
            PathBuf::from("src/balance/mod.rs"),
            "balance".to_string(),
        ));
        metrics.add_module(ModuleMetrics::new(
            PathBuf::from("src/balance/project.rs"),
            "balance::project".to_string(),
        ));
        metrics.temporal_couplings.push(TemporalCoupling {
            file_a: "src/balance/mod.rs".to_string(),
            file_b: "src/balance/project.rs".to_string(),
            co_change_count: 12,
            coupling_ratio: 0.9,
        });
        let report = analyze_project_balance(&metrics);
        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.issue_type == IssueType::HiddenCoupling),
            "adjacent (parent/child) co-change must not be hidden coupling"
        );

        // High needs a repeated pattern: 5 co-changes at 100% is a burst -> Medium.
        let mut sprint = ProjectMetrics::new();
        sprint.add_module(ModuleMetrics::new(
            PathBuf::from("src/alpha.rs"),
            "alpha".to_string(),
        ));
        sprint.add_module(ModuleMetrics::new(
            PathBuf::from("src/beta.rs"),
            "beta".to_string(),
        ));
        sprint.temporal_couplings.push(TemporalCoupling {
            file_a: "src/alpha.rs".to_string(),
            file_b: "src/beta.rs".to_string(),
            co_change_count: 5,
            coupling_ratio: 1.0,
        });
        let report = analyze_project_balance(&sprint);
        let issue = report
            .issues
            .iter()
            .find(|issue| issue.issue_type == IssueType::HiddenCoupling)
            .expect("burst co-change is still reported");
        assert_eq!(issue.severity, Severity::Medium);

        // A persistent pattern (>= 8 co-changes at >= 80%) is High.
        sprint.temporal_couplings[0].co_change_count = 9;
        let report = analyze_project_balance(&sprint);
        let issue = report
            .issues
            .iter()
            .find(|issue| issue.issue_type == IssueType::HiddenCoupling)
            .expect("persistent co-change is reported");
        assert_eq!(issue.severity, Severity::High);
    }

    #[test]
    fn test_hidden_coupling_skipped_when_code_dependency_exists() {
        let mut metrics = ProjectMetrics::new();
        metrics.add_module(ModuleMetrics::new(
            PathBuf::from("src/pricing.rs"),
            "pricing".to_string(),
        ));
        metrics.add_module(ModuleMetrics::new(
            PathBuf::from("src/invoicing.rs"),
            "invoicing".to_string(),
        ));
        metrics.add_coupling(CouplingMetrics::new(
            "pricing".to_string(),
            "invoicing".to_string(),
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low,
        ));
        metrics.temporal_couplings.push(TemporalCoupling {
            file_a: "src/pricing.rs".to_string(),
            file_b: "src/invoicing.rs".to_string(),
            co_change_count: 6,
            coupling_ratio: 0.75,
        });

        let report = analyze_project_balance(&metrics);

        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.issue_type == IssueType::HiddenCoupling),
            "explicit code dependency should suppress hidden coupling issue"
        );
    }

    #[test]
    fn test_accidental_volatility_detected_for_stable_subdomain_churn() {
        let mut metrics = ProjectMetrics::new();
        let mut report_module =
            ModuleMetrics::new(PathBuf::from("src/report.rs"), "report".to_string());
        report_module.subdomain = Some(Subdomain::Supporting);
        metrics.add_module(report_module);
        metrics.file_changes.insert("src/report.rs".to_string(), 12);
        metrics
            .file_changes
            .insert("src/analyzer.rs".to_string(), 2);
        metrics.file_changes.insert("src/lib.rs".to_string(), 3);
        metrics.file_changes.insert("src/config.rs".to_string(), 4);

        let report = analyze_project_balance(&metrics);

        let issue = report
            .issues
            .iter()
            .find(|issue| issue.issue_type == IssueType::AccidentalVolatility)
            .expect("supporting subdomain in top churn quartile should be flagged");
        assert_eq!(issue.source, "report");
        assert_eq!(issue.target, "report");
        assert!(issue.description.contains("Supporting"));
        assert_eq!(issue.severity, Severity::Medium);
    }

    #[test]
    fn test_accidental_volatility_skips_tiny_change_samples() {
        let mut metrics = ProjectMetrics::new();
        let mut report_module =
            ModuleMetrics::new(PathBuf::from("src/report.rs"), "report".to_string());
        report_module.subdomain = Some(Subdomain::Supporting);
        metrics.add_module(report_module);
        metrics.file_changes.insert("src/report.rs".to_string(), 12);
        metrics.file_changes.insert("src/lib.rs".to_string(), 1);
        metrics.file_changes.insert("src/config.rs".to_string(), 2);

        let report = analyze_project_balance(&metrics);

        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.issue_type == IssueType::AccidentalVolatility),
            "fewer than four tracked files should not trigger top-quartile accidental volatility"
        );
    }

    #[test]
    fn test_supporting_subdomain_volatility_is_authoritative_for_balance_and_risk() {
        let mut metrics = ProjectMetrics::new();
        let mut stable_module =
            ModuleMetrics::new(PathBuf::from("src/stable.rs"), "stable".to_string());
        stable_module.subdomain = Some(Subdomain::Supporting);
        metrics.add_module(stable_module);
        metrics.file_changes.insert("src/stable.rs".to_string(), 12);
        metrics.file_changes.insert("src/lib.rs".to_string(), 1);
        metrics.file_changes.insert("src/config.rs".to_string(), 2);
        metrics.file_changes.insert("src/report.rs".to_string(), 3);
        metrics.add_coupling(CouplingMetrics::new(
            "caller".to_string(),
            "stable".to_string(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        ));

        let report = analyze_project_balance(&metrics);

        assert_eq!(report.average_score, 0.5);
        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.issue_type == IssueType::CascadingChangeRisk),
            "supporting subdomain's low essential volatility should suppress cascading risk"
        );
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.issue_type == IssueType::AccidentalVolatility),
            "raw churn should remain visible as accidental volatility"
        );
    }

    #[test]
    fn test_core_subdomain_volatility_is_authoritative_for_balance_and_risk() {
        let mut metrics = ProjectMetrics::new();
        let mut core_module = ModuleMetrics::new(PathBuf::from("src/core.rs"), "core".to_string());
        core_module.subdomain = Some(Subdomain::Core);
        metrics.add_module(core_module);
        metrics.add_coupling(CouplingMetrics::new(
            "caller".to_string(),
            "core".to_string(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::Low,
        ));

        let report = analyze_project_balance(&metrics);

        assert_eq!(report.average_score, 0.0);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.issue_type == IssueType::CascadingChangeRisk),
            "core subdomain's high essential volatility should drive cascading risk"
        );
    }

    #[test]
    fn test_unclassified_git_churn_still_drives_balance_and_cascading_risk() {
        let mut metrics = ProjectMetrics::new();
        metrics.add_module(ModuleMetrics::new(
            PathBuf::from("src/stable.rs"),
            "stable".to_string(),
        ));
        metrics.add_coupling(CouplingMetrics::new(
            "caller".to_string(),
            "stable".to_string(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        ));

        let report = analyze_project_balance(&metrics);

        assert_eq!(report.average_score, 0.0);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.issue_type == IssueType::CascadingChangeRisk),
            "unclassified modules should keep existing git-churn volatility behavior"
        );
    }

    #[test]
    fn test_ambiguous_item_alias_does_not_override_git_churn() {
        let mut metrics = ProjectMetrics::new();
        let mut supporting_module =
            ModuleMetrics::new(PathBuf::from("src/supporting.rs"), "supporting".to_string());
        supporting_module.subdomain = Some(Subdomain::Supporting);
        supporting_module.add_type_definition("SharedName".to_string(), Visibility::Public, false);
        metrics.add_module(supporting_module);

        let mut core_module = ModuleMetrics::new(PathBuf::from("src/core.rs"), "core".to_string());
        core_module.subdomain = Some(Subdomain::Core);
        core_module.add_type_definition("SharedName".to_string(), Visibility::Public, false);
        metrics.add_module(core_module);

        metrics.add_coupling(CouplingMetrics::new(
            "caller".to_string(),
            "SharedName".to_string(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        ));

        let report = analyze_project_balance(&metrics);

        assert_eq!(report.average_score, 0.0);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.issue_type == IssueType::CascadingChangeRisk),
            "ambiguous item aliases should preserve existing git-churn volatility"
        );
    }

    #[test]
    fn test_grade_rationale_names_top_issue_and_volatility_driver() {
        let mut metrics = ProjectMetrics::new();
        let mut core_module = ModuleMetrics::new(PathBuf::from("src/core.rs"), "core".to_string());
        core_module.subdomain = Some(Subdomain::Core);
        metrics.add_module(core_module);
        metrics.file_changes.insert("src/core.rs".to_string(), 1);
        metrics.file_changes.insert("src/lib.rs".to_string(), 1);
        metrics.file_changes.insert("src/config.rs".to_string(), 2);
        metrics.file_changes.insert("src/report.rs".to_string(), 3);
        metrics.add_coupling(CouplingMetrics::new(
            "caller".to_string(),
            "core".to_string(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::Low,
        ));

        let report = analyze_project_balance(&metrics);

        assert!(
            report
                .grade_rationale
                .top_issue_types
                .iter()
                .any(|item| item.issue_type == IssueType::CascadingChangeRisk && item.count == 1)
        );
        assert_eq!(
            report.grade_rationale.dominant_dimension,
            Some(GradeDimension::Volatility)
        );
        assert!(
            report
                .grade_rationale
                .summary
                .contains("Cascading Change Risk")
        );
        assert!(report.grade_rationale.volatility_note.is_some());
    }

    #[test]
    fn test_duplicate_issue_keys_are_deduped_in_report() {
        let mut metrics = ProjectMetrics::new();
        metrics.add_coupling(CouplingMetrics::new(
            "balance::grade".to_string(),
            "balance::rationale".to_string(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        ));
        metrics.add_coupling(CouplingMetrics::new(
            "balance::grade".to_string(),
            "balance::rationale".to_string(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        ));

        let report = analyze_project_balance(&metrics);
        let duplicate_key_count = report
            .issues
            .iter()
            .filter(|issue| {
                issue.issue_type == IssueType::CascadingChangeRisk
                    && issue.source == "balance::grade"
                    && issue.target == "balance::rationale"
            })
            .count();

        assert_eq!(duplicate_key_count, 1);
    }

    #[test]
    fn test_grade_rationale_mentions_data_limit_for_zero_and_low_couplings() {
        let english_zero = build_grade_rationale(&[], 0, false);
        assert!(english_zero.summary.contains("0 internal couplings"));
        assert!(english_zero.summary.contains("grade capped at B"));

        let english_low = build_grade_rationale(&[], 9, false);
        assert!(
            english_low
                .summary
                .contains("fewer than 10, too little data")
        );
        assert!(english_low.summary.contains("grade capped at B"));

        let japanese_zero = build_grade_rationale(&[], 0, true);
        assert!(japanese_zero.summary.contains("内部結合が 0 件"));
        assert!(japanese_zero.summary.contains("グレードは B が上限"));

        let japanese_low = build_grade_rationale(&[], 9, true);
        assert!(japanese_low.summary.contains("10 件未満"));
        assert!(japanese_low.summary.contains("グレードは B が上限"));
    }

    #[test]
    fn test_health_grade_calculation() {
        let mut issues = HashMap::new();

        // No issues with >= 20 couplings = S (over-optimized warning)
        assert_eq!(calculate_health_grade(&issues, 100), HealthGrade::S);

        // No issues with 10-19 couplings = A (well-balanced)
        assert_eq!(calculate_health_grade(&issues, 15), HealthGrade::A);

        // No internal couplings = B (can't assess without data)
        assert_eq!(calculate_health_grade(&issues, 0), HealthGrade::B);

        // Any High issue = C (structural issues)
        issues.insert(Severity::High, 1);
        assert_eq!(calculate_health_grade(&issues, 100), HealthGrade::C);

        // High density > 5% = D
        issues.clear();
        issues.insert(Severity::High, 6); // 6% of 100
        assert_eq!(calculate_health_grade(&issues, 100), HealthGrade::D);

        // 1 Critical issue = D
        issues.clear();
        issues.insert(Severity::Critical, 1);
        assert_eq!(calculate_health_grade(&issues, 100), HealthGrade::D);

        // 4+ Critical issues = F
        issues.clear();
        issues.insert(Severity::Critical, 4);
        assert_eq!(calculate_health_grade(&issues, 100), HealthGrade::F);

        // Medium issues > 25% = C
        issues.clear();
        issues.insert(Severity::Medium, 30); // 30% of 100
        assert_eq!(calculate_health_grade(&issues, 100), HealthGrade::C);

        // Medium issues > 5% but <= 25% = B
        issues.clear();
        issues.insert(Severity::Medium, 20); // 20% of 100
        assert_eq!(calculate_health_grade(&issues, 100), HealthGrade::B);
    }
}
