//! # cargo-coupling - Coupling Analysis Tool
//!
//! A tool for measuring and analyzing coupling in Rust projects,
//! based on Vlad Khononov's "Balancing Coupling in Software Design".
//!
//! ## Overview
//!
//! cargo-coupling measures coupling across three dimensions:
//!
//! 1. **Integration Strength** - How much knowledge is shared between components
//! 2. **Distance** - How far apart components are in the module hierarchy
//! 3. **Volatility** - How frequently components change
//!
//! ## Usage
//!
//! ```bash
//! # Analyze current project (as cargo subcommand)
//! cargo coupling ./src
//!
//! # Generate detailed report
//! cargo coupling -o report.md ./src
//!
//! # Show summary only
//! cargo coupling --summary ./src
//! ```
//!
//! ## Balance Equation
//!
//! The balance score is calculated as:
//! ```text
//! BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
//! ```
//!
//! - Strong coupling + close distance = Good (locality)
//! - Weak coupling + far distance = Good (loose coupling)
//! - Strong coupling + far distance = Bad (global complexity)
//! - High volatility + strong coupling = Bad (cascading changes)

pub mod analyzer;
pub mod balance;
pub mod cli_output;
pub mod config;
pub mod diff;
pub mod external;
pub mod history;
pub mod manifest;
pub mod metrics;
pub mod report;
pub mod volatility;
pub mod web;
pub mod workspace;

pub use analyzer::{
    AnalyzedFileResult, AnalyzerError, CouplingAnalyzer, Dependency, DependencyKind, ItemDepType,
    ItemDependency, ItemKind, analyze_project, analyze_project_parallel_with_config,
    analyze_rust_file, analyze_rust_file_full, analyze_workspace, analyze_workspace_with_config,
};
pub use balance::action::RefactoringAction;
pub use balance::grade::{HealthGrade, ProjectBalanceReport};
pub use balance::issue::CouplingIssue;
pub use balance::issue_type::IssueType;
pub use balance::project::{
    analyze_project_balance, analyze_project_balance_with_thresholds, calculate_project_score,
};
pub use balance::rationale::{GradeDimension, GradeRationale, IssueTypeContribution};
pub use balance::score::{BalanceInterpretation, BalanceScore, IssueThresholds};
pub use balance::severity::Severity;
pub use config::{
    AnalysisConfig, CompiledConfig, ConfigError, CouplingConfig, ThresholdsConfig,
    VolatilityConfig, load_compiled_config, load_config,
};
pub use diff::{BaselineDiff, IssueKey, diff_ref_analysis, diff_reports};
pub use external::{
    ExternalDependencyReport, ExternalDependencyUsage, SCATTERED_EXTERNAL_BREADTH_THRESHOLD,
    analyze_external_dependencies, detect_scattered_external_coupling, load_lock_versions_near,
};
pub use history::{
    HistoryError, HistoryPoint, HistoryReport, RefAnalysis, SkippedRevision, analyze_history,
    analyze_ref,
};
pub use manifest::{AnalysisManifest, BlindSpot, ManifestContext, build_manifest};
pub use metrics::coupling::{CouplingLocation, CouplingMetrics};
pub use metrics::dimensions::{
    Distance, IntegrationStrength, MetricsConfig, Subdomain, Visibility,
};
pub use metrics::module::{
    BalanceClassification, BalanceCounts, DimensionStats, DistanceCounts, FunctionDefinition,
    ModuleMetrics, StrengthCounts, TypeDefinition, VolatilityCounts,
};
pub use metrics::project::{CircularDependencySummary, ProjectMetrics};
pub use report::{
    TextReportOptions, generate_ai_output, generate_ai_output_with_thresholds, generate_report,
    generate_report_with_options, generate_report_with_thresholds, generate_summary,
    generate_summary_with_options, generate_summary_with_thresholds,
};
pub use volatility::Volatility;
pub use volatility::{VolatilityAnalyzer, VolatilityError, VolatilityStats};
pub use workspace::{CrateInfo, WorkspaceError, WorkspaceInfo};
