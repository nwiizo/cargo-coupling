//! Core metric types shared by analysis, balance scoring, and reporting.
//!
//! It stores the structural facts collected from Rust source and exposes the
//! aggregate calculations used to interpret coupling through Khononov's model.

pub mod coupling;
pub mod dimensions;
pub mod module;
pub mod project;

pub use crate::volatility::{TemporalCoupling, Volatility};
pub use coupling::{CouplingLocation, CouplingMetrics};
pub use dimensions::{Distance, IntegrationStrength, MetricsConfig, Subdomain, Visibility};
pub use module::{
    BalanceClassification, BalanceCounts, DimensionStats, DistanceCounts, FunctionDefinition,
    ModuleMetrics, StrengthCounts, TypeDefinition, VolatilityCounts,
};
pub use project::{CircularDependencySummary, ProjectMetrics};
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_integration_strength_values() {
        assert_eq!(IntegrationStrength::Intrusive.value(), 1.0);
        assert_eq!(IntegrationStrength::Contract.value(), 0.25);
    }

    #[test]
    fn test_distance_values() {
        assert_eq!(Distance::SameFunction.value(), 0.0);
        assert_eq!(Distance::DifferentCrate.value(), 1.0);
    }

    #[test]
    fn test_volatility_from_count() {
        assert_eq!(Volatility::from_count(0), Volatility::Low);
        assert_eq!(Volatility::from_count(5), Volatility::Medium);
        assert_eq!(Volatility::from_count(15), Volatility::High);
    }

    #[test]
    fn test_module_metrics_average_strength() {
        let mut metrics = ModuleMetrics::new(PathBuf::from("test.rs"), "test".to_string());
        metrics.trait_impl_count = 3;
        metrics.inherent_impl_count = 1;

        let avg = metrics.average_strength();
        assert!(avg > 0.0 && avg < 1.0);
    }

    #[test]
    fn test_project_metrics() {
        let mut project = ProjectMetrics::new();

        let module = ModuleMetrics::new(PathBuf::from("lib.rs"), "lib".to_string());
        project.add_module(module);

        assert_eq!(project.module_count(), 1);
        assert_eq!(project.coupling_count(), 0);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut project = ProjectMetrics::new();

        // Create a cycle: A -> B -> C -> A
        project.add_coupling(CouplingMetrics::new(
            "module_a".to_string(),
            "module_b".to_string(),
            IntegrationStrength::Model,
            Distance::DifferentModule,
            Volatility::Low,
        ));
        project.add_coupling(CouplingMetrics::new(
            "module_b".to_string(),
            "module_c".to_string(),
            IntegrationStrength::Model,
            Distance::DifferentModule,
            Volatility::Low,
        ));
        project.add_coupling(CouplingMetrics::new(
            "module_c".to_string(),
            "module_a".to_string(),
            IntegrationStrength::Model,
            Distance::DifferentModule,
            Volatility::Low,
        ));

        let cycles = project.detect_circular_dependencies();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
    }

    #[test]
    fn test_no_circular_dependencies() {
        let mut project = ProjectMetrics::new();

        // Linear dependency: A -> B -> C (no cycle)
        project.add_coupling(CouplingMetrics::new(
            "module_a".to_string(),
            "module_b".to_string(),
            IntegrationStrength::Model,
            Distance::DifferentModule,
            Volatility::Low,
        ));
        project.add_coupling(CouplingMetrics::new(
            "module_b".to_string(),
            "module_c".to_string(),
            IntegrationStrength::Model,
            Distance::DifferentModule,
            Volatility::Low,
        ));

        let cycles = project.detect_circular_dependencies();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_external_crates_excluded_from_cycles() {
        let mut project = ProjectMetrics::new();

        // External crate dependency should be ignored
        project.add_coupling(CouplingMetrics::new(
            "module_a".to_string(),
            "serde::Serialize".to_string(),
            IntegrationStrength::Contract,
            Distance::DifferentCrate, // External
            Volatility::Low,
        ));
        project.add_coupling(CouplingMetrics::new(
            "serde::Serialize".to_string(),
            "module_a".to_string(),
            IntegrationStrength::Contract,
            Distance::DifferentCrate, // External
            Volatility::Low,
        ));

        let cycles = project.detect_circular_dependencies();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_circular_dependency_summary() {
        let mut project = ProjectMetrics::new();

        // Create a simple cycle: A <-> B
        project.add_coupling(CouplingMetrics::new(
            "module_a".to_string(),
            "module_b".to_string(),
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low,
        ));
        project.add_coupling(CouplingMetrics::new(
            "module_b".to_string(),
            "module_a".to_string(),
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low,
        ));

        let summary = project.circular_dependency_summary();
        assert!(summary.total_cycles > 0);
        assert!(summary.affected_modules >= 2);
    }

    #[test]
    fn test_visibility_intrusive_detection() {
        // Public items are never intrusive
        assert!(!Visibility::Public.is_intrusive_from(true, false));
        assert!(!Visibility::Public.is_intrusive_from(false, false));

        // PubCrate is intrusive only from different crate
        assert!(!Visibility::PubCrate.is_intrusive_from(true, false));
        assert!(Visibility::PubCrate.is_intrusive_from(false, false));

        // Private is always intrusive from outside
        assert!(Visibility::Private.is_intrusive_from(true, false));
        assert!(Visibility::Private.is_intrusive_from(false, false));

        // Same module access is never intrusive
        assert!(!Visibility::Private.is_intrusive_from(true, true));
        assert!(!Visibility::Private.is_intrusive_from(false, true));
    }

    #[test]
    fn test_visibility_penalty() {
        assert_eq!(Visibility::Public.intrusive_penalty(), 0.0);
        assert_eq!(Visibility::PubCrate.intrusive_penalty(), 0.25);
        assert_eq!(Visibility::Private.intrusive_penalty(), 1.0);
    }

    #[test]
    fn test_effective_strength() {
        // Public target - no upgrade
        let coupling = CouplingMetrics::with_visibility(
            "source".to_string(),
            "target".to_string(),
            IntegrationStrength::Model,
            Distance::DifferentModule,
            Volatility::Low,
            Visibility::Public,
        );
        assert_eq!(coupling.effective_strength(), IntegrationStrength::Model);

        // Private target from different module - upgraded
        let coupling = CouplingMetrics::with_visibility(
            "source".to_string(),
            "target".to_string(),
            IntegrationStrength::Model,
            Distance::DifferentModule,
            Volatility::Low,
            Visibility::Private,
        );
        assert_eq!(
            coupling.effective_strength(),
            IntegrationStrength::Functional
        );
    }

    #[test]
    fn test_type_registry() {
        let mut project = ProjectMetrics::new();

        project.register_type(
            "MyStruct".to_string(),
            "my_module".to_string(),
            Visibility::Public,
        );
        project.register_type(
            "InternalType".to_string(),
            "my_module".to_string(),
            Visibility::PubCrate,
        );

        assert_eq!(
            project.get_type_visibility("MyStruct"),
            Some(Visibility::Public)
        );
        assert_eq!(
            project.get_type_visibility("InternalType"),
            Some(Visibility::PubCrate)
        );
        assert_eq!(project.get_type_visibility("Unknown"), None);

        assert_eq!(project.get_type_module("MyStruct"), Some("my_module"));
    }

    #[test]
    fn test_module_type_definitions() {
        let mut module = ModuleMetrics::new(PathBuf::from("test.rs"), "test".to_string());

        module.add_type_definition("PublicStruct".to_string(), Visibility::Public, false);
        module.add_type_definition("PrivateStruct".to_string(), Visibility::Private, false);
        module.add_type_definition("PublicTrait".to_string(), Visibility::Public, true);

        assert_eq!(module.public_type_count(), 2);
        assert_eq!(module.private_type_count(), 1);
        assert_eq!(
            module.get_type_visibility("PublicStruct"),
            Some(Visibility::Public)
        );
    }

    #[test]
    fn test_update_volatility_from_git() {
        let mut project = ProjectMetrics::new();

        // Add couplings with targets matching file names
        project.add_coupling(CouplingMetrics::new(
            "crate::main".to_string(),
            "crate::balance".to_string(),
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low, // Initial volatility
        ));
        project.add_coupling(CouplingMetrics::new(
            "crate::main".to_string(),
            "crate::analyzer".to_string(),
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low,
        ));
        project.add_coupling(CouplingMetrics::new(
            "crate::main".to_string(),
            "crate::report".to_string(),
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low,
        ));

        // Simulate git file changes
        project
            .file_changes
            .insert("src/balance.rs".to_string(), 15); // High
        project
            .file_changes
            .insert("src/analyzer.rs".to_string(), 7); // Medium
        project.file_changes.insert("src/report.rs".to_string(), 2); // Low

        // Update volatility from git data
        project.update_volatility_from_git();

        // Verify volatility was updated correctly
        let balance_coupling = project
            .couplings
            .iter()
            .find(|c| c.target == "crate::balance")
            .unwrap();
        assert_eq!(balance_coupling.volatility, Volatility::High);

        let analyzer_coupling = project
            .couplings
            .iter()
            .find(|c| c.target == "crate::analyzer")
            .unwrap();
        assert_eq!(analyzer_coupling.volatility, Volatility::Medium);

        let report_coupling = project
            .couplings
            .iter()
            .find(|c| c.target == "crate::report")
            .unwrap();
        assert_eq!(report_coupling.volatility, Volatility::Low);
    }

    #[test]
    fn test_volatility_with_type_targets() {
        // Test with more realistic targets that include type names (e.g., crate::balance::BalanceScore)
        let mut project = ProjectMetrics::new();

        // Add couplings with Type-level targets (common in real analysis)
        project.add_coupling(CouplingMetrics::new(
            "crate::main".to_string(),
            "crate::balance::BalanceScore".to_string(), // Type in balance module
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low,
        ));
        project.add_coupling(CouplingMetrics::new(
            "crate::main".to_string(),
            "cargo-coupling::analyzer::analyze_file".to_string(), // Function in analyzer module
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low,
        ));

        // Simulate git file changes
        project
            .file_changes
            .insert("src/balance.rs".to_string(), 15); // High
        project
            .file_changes
            .insert("src/analyzer.rs".to_string(), 7); // Medium

        // Update volatility from git data
        project.update_volatility_from_git();

        // Verify volatility was updated correctly by matching module path component
        let balance_coupling = project
            .couplings
            .iter()
            .find(|c| c.target.contains("balance"))
            .unwrap();
        assert_eq!(
            balance_coupling.volatility,
            Volatility::High,
            "Expected High volatility for balance module (15 changes)"
        );

        let analyzer_coupling = project
            .couplings
            .iter()
            .find(|c| c.target.contains("analyzer"))
            .unwrap();
        assert_eq!(
            analyzer_coupling.volatility,
            Volatility::Medium,
            "Expected Medium volatility for analyzer module (7 changes)"
        );
    }

    #[test]
    fn test_volatility_extracted_module_targets() {
        // Test with extracted module names (like what the analyzer produces)
        // The analyzer's extract_target_module() returns just "balance" from "crate::balance::Type"
        let mut project = ProjectMetrics::new();

        // Extracted module targets (single component names)
        project.add_coupling(CouplingMetrics::new(
            "cargo-coupling::main".to_string(),
            "balance".to_string(), // Extracted module name
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low,
        ));
        project.add_coupling(CouplingMetrics::new(
            "cargo-coupling::main".to_string(),
            "analyzer".to_string(), // Extracted module name
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low,
        ));
        project.add_coupling(CouplingMetrics::new(
            "cargo-coupling::main".to_string(),
            "cli_output".to_string(), // Extracted module name with underscore
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::Low,
        ));

        // Simulate git file changes
        project
            .file_changes
            .insert("src/balance.rs".to_string(), 15); // High
        project
            .file_changes
            .insert("src/analyzer.rs".to_string(), 7); // Medium
        project
            .file_changes
            .insert("src/cli_output.rs".to_string(), 3); // Medium

        // Update volatility from git data
        project.update_volatility_from_git();

        // Verify volatility was updated
        let balance = project
            .couplings
            .iter()
            .find(|c| c.target == "balance")
            .unwrap();
        assert_eq!(
            balance.volatility,
            Volatility::High,
            "balance should be High (15 changes)"
        );

        let analyzer = project
            .couplings
            .iter()
            .find(|c| c.target == "analyzer")
            .unwrap();
        assert_eq!(
            analyzer.volatility,
            Volatility::Medium,
            "analyzer should be Medium (7 changes)"
        );

        let cli_output = project
            .couplings
            .iter()
            .find(|c| c.target == "cli_output")
            .unwrap();
        assert_eq!(
            cli_output.volatility,
            Volatility::Medium,
            "cli_output should be Medium (3 changes)"
        );
    }

    #[test]
    fn test_submodule_volatility_prefers_exact_module_file() {
        let mut project = ProjectMetrics::new();
        project.add_module(ModuleMetrics::new(
            PathBuf::from("src/balance/issues.rs"),
            "balance::issues".to_string(),
        ));
        project.add_coupling(CouplingMetrics::new(
            "report".to_string(),
            "balance::issues::CouplingIssue".to_string(),
            IntegrationStrength::Model,
            Distance::DifferentModule,
            Volatility::Low,
        ));
        project
            .file_changes
            .insert("src/balance.rs".to_string(), 15);

        project.update_volatility_from_git();

        let coupling = project.couplings.first().unwrap();
        assert_eq!(
            coupling.volatility,
            Volatility::Low,
            "submodules should not inherit churn from the old parent module file"
        );
    }
}
