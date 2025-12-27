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
pub mod metrics;
pub mod report;
pub mod volatility;
pub mod web;
pub mod workspace;

pub use analyzer::{
    AnalyzedFileResult, AnalyzerError, CouplingAnalyzer, Dependency, DependencyKind, ItemDepType,
    ItemDependency, ItemKind, analyze_project, analyze_rust_file, analyze_rust_file_full,
    analyze_workspace,
};
pub use balance::{
    BalanceInterpretation, BalanceScore, CouplingIssue, HealthGrade, IssueThresholds, IssueType,
    ProjectBalanceReport, RefactoringAction, Severity, analyze_project_balance,
    analyze_project_balance_with_thresholds, calculate_project_score,
};
pub use config::{
    AnalysisConfig, CompiledConfig, ConfigError, CouplingConfig, ThresholdsConfig,
    VolatilityConfig, load_compiled_config, load_config,
};
pub use metrics::{
    BalanceClassification, BalanceCounts, CircularDependencySummary, CouplingMetrics,
    DimensionStats, Distance, DistanceCounts, FunctionDefinition, IntegrationStrength,
    ModuleMetrics, ProjectMetrics, StrengthCounts, TypeDefinition, Visibility, Volatility,
    VolatilityCounts,
};
pub use report::{
    generate_ai_output, generate_ai_output_with_thresholds, generate_report,
    generate_report_with_thresholds, generate_summary, generate_summary_with_thresholds,
};
pub use volatility::{VolatilityAnalyzer, VolatilityError, VolatilityStats};
pub use workspace::{CrateInfo, WorkspaceError, WorkspaceInfo};
