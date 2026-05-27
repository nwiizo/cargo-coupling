//! Balance scoring and issue classification for coupling analysis.
//!
//! This module turns raw coupling metrics into health grades, detected issues,
//! and refactoring recommendations so reports can explain why a relationship is
//! well-balanced or costly to change.

use std::collections::{HashMap, HashSet};

use crate::metrics::{
    CouplingMetrics, Distance, IntegrationStrength, ProjectMetrics, Subdomain, Volatility,
};

// ===== Issue Model =====

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Minor issue, consider addressing
    Low,
    /// Should be addressed in regular maintenance
    Medium,
    /// Needs attention soon, potential source of bugs
    High,
    /// Must be fixed, actively causing problems
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "Low"),
            Severity::Medium => write!(f, "Medium"),
            Severity::High => write!(f, "High"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}

/// Types of coupling problems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueType {
    /// Strong coupling spanning a long distance
    GlobalComplexity,
    /// Strong coupling to a frequently changing component
    CascadingChangeRisk,
    /// Intrusive coupling across boundaries (field/internals access)
    InappropriateIntimacy,
    /// A module with too many dependencies
    HighEfferentCoupling,
    /// A module that too many others depend on
    HighAfferentCoupling,
    /// Weak coupling where stronger might be appropriate
    UnnecessaryAbstraction,
    /// Circular dependency detected
    CircularDependency,
    /// Strong temporal co-change without an explicit code dependency
    HiddenCoupling,
    /// Supporting or generic module changing more often than expected
    AccidentalVolatility,
    /// Direct coupling to a third-party crate is spread across many modules
    ScatteredExternalCoupling,

    // === APOSD-inspired issues (A Philosophy of Software Design) ===
    /// Module with interface complexity close to implementation complexity
    ShallowModule,
    /// Method that only delegates to another method without adding value
    PassThroughMethod,
    /// Module requiring too much knowledge to understand/modify
    HighCognitiveLoad,

    // === Khononov/Rust-specific issues ===
    /// Module with too many functions, types, or implementations
    GodModule,
    /// Public fields exposed to external modules (should use getters/methods)
    PublicFieldExposure,
    /// Functions with too many primitive parameters (consider newtype)
    PrimitiveObsession,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::GlobalComplexity => write!(f, "Global Complexity"),
            IssueType::CascadingChangeRisk => write!(f, "Cascading Change Risk"),
            IssueType::InappropriateIntimacy => write!(f, "Inappropriate Intimacy"),
            IssueType::HighEfferentCoupling => write!(f, "High Efferent Coupling"),
            IssueType::HighAfferentCoupling => write!(f, "High Afferent Coupling"),
            IssueType::UnnecessaryAbstraction => write!(f, "Unnecessary Abstraction"),
            IssueType::CircularDependency => write!(f, "Circular Dependency"),
            IssueType::HiddenCoupling => write!(f, "Hidden Coupling"),
            IssueType::AccidentalVolatility => write!(f, "Accidental Volatility"),
            IssueType::ScatteredExternalCoupling => write!(f, "Scattered External Coupling"),
            // APOSD-inspired
            IssueType::ShallowModule => write!(f, "Shallow Module"),
            IssueType::PassThroughMethod => write!(f, "Pass-Through Method"),
            IssueType::HighCognitiveLoad => write!(f, "High Cognitive Load"),
            // Khononov/Rust-specific
            IssueType::GodModule => write!(f, "God Module"),
            IssueType::PublicFieldExposure => write!(f, "Public Field Exposure"),
            IssueType::PrimitiveObsession => write!(f, "Primitive Obsession"),
        }
    }
}

impl IssueType {
    /// Get a detailed description of what this issue type means
    pub fn description(&self) -> &'static str {
        match self {
            IssueType::GlobalComplexity => {
                "Strong coupling to distant components increases cognitive load and makes the system harder to understand and modify."
            }
            IssueType::CascadingChangeRisk => {
                "Strongly coupling to volatile components means changes will cascade through the system, requiring updates in many places."
            }
            IssueType::InappropriateIntimacy => {
                "Direct access to internal details (fields, private methods) across module boundaries violates encapsulation."
            }
            IssueType::HighEfferentCoupling => {
                "A module depending on too many others is fragile and hard to test. Changes anywhere affect this module."
            }
            IssueType::HighAfferentCoupling => {
                "A module that many others depend on is hard to change. Any modification risks breaking dependents."
            }
            IssueType::UnnecessaryAbstraction => {
                "Using abstract interfaces for closely-related stable components may add complexity without benefit."
            }
            IssueType::CircularDependency => {
                "Circular dependencies make it impossible to understand, test, or modify components in isolation."
            }
            IssueType::HiddenCoupling => {
                "Files frequently change together without an explicit code dependency. This suggests implicit shared knowledge or a missing abstraction."
            }
            IssueType::AccidentalVolatility => {
                "A supporting or generic subdomain changes frequently despite being expected to be stable. This suggests churn from design or ownership issues rather than essential business volatility."
            }
            IssueType::ScatteredExternalCoupling => {
                "A third-party crate is used directly from many internal modules, spreading upgrade and API-change risk across code you control."
            }
            // APOSD-inspired descriptions
            IssueType::ShallowModule => {
                "Interface complexity is close to implementation complexity. The module doesn't hide enough complexity behind a simple interface. (APOSD: Deep vs Shallow Modules)"
            }
            IssueType::PassThroughMethod => {
                "Method only delegates to another method without adding significant functionality. Indicates unclear responsibility division. (APOSD: Pass-Through Methods)"
            }
            IssueType::HighCognitiveLoad => {
                "Module requires too much knowledge to understand and modify. Too many public APIs, dependencies, or complex type signatures. (APOSD: Cognitive Load)"
            }
            // Khononov/Rust-specific descriptions
            IssueType::GodModule => {
                "Module has too many responsibilities - too many functions, types, or implementations. Consider splitting into focused, cohesive modules. (SRP violation)"
            }
            IssueType::PublicFieldExposure => {
                "Struct has public fields accessed from other modules. Consider using getter methods to reduce coupling and allow future implementation changes."
            }
            IssueType::PrimitiveObsession => {
                "Function has many primitive parameters of the same type. Consider using newtype pattern (e.g., `struct UserId(u64)`) for type safety and clarity."
            }
        }
    }

    /// Get a Japanese description of what this issue type means.
    pub fn description_japanese(&self) -> &'static str {
        match self {
            IssueType::GlobalComplexity => {
                "遠いコンポーネントへの強い結合は認知負荷を高め、理解や変更を難しくします。"
            }
            IssueType::CascadingChangeRisk => {
                "頻繁に変わるコンポーネントへ強く結合すると、変更がシステム全体に波及しやすくなります。"
            }
            IssueType::InappropriateIntimacy => {
                "モジュール境界を越えた内部詳細への直接アクセスはカプセル化を損ないます。"
            }
            IssueType::HighEfferentCoupling => {
                "多くのモジュールに依存するモジュールは壊れやすく、テストも難しくなります。"
            }
            IssueType::HighAfferentCoupling => {
                "多くのモジュールから依存されるモジュールは変更しづらく、依存元を壊すリスクがあります。"
            }
            IssueType::UnnecessaryAbstraction => {
                "近く安定したコンポーネントに抽象インターフェースを使うと、利益より複雑さが増える場合があります。"
            }
            IssueType::CircularDependency => {
                "循環依存はコンポーネントを単独で理解、テスト、変更することを難しくします。"
            }
            IssueType::HiddenCoupling => {
                "明示的なコード依存がないのにファイルが頻繁に一緒に変わっています。暗黙の知識や不足した抽象化を示している可能性があります。"
            }
            IssueType::AccidentalVolatility => {
                "安定しているはずの支援/汎用サブドメインが頻繁に変更されています。設計や所有権の問題によるチャーンの可能性があります。"
            }
            IssueType::ScatteredExternalCoupling => {
                "サードパーティクレートが多くの内部モジュールから直接使われており、更新やAPI変更のリスクが広がっています。"
            }
            IssueType::ShallowModule => {
                "インターフェースの複雑さが実装の複雑さに近く、単純なインターフェースの背後に十分な複雑さを隠せていません。"
            }
            IssueType::PassThroughMethod => {
                "メソッドが価値を追加せず別メソッドへ委譲しており、責務分担が曖昧な可能性があります。"
            }
            IssueType::HighCognitiveLoad => {
                "公開API、依存、複雑な型シグネチャが多く、理解や変更に必要な知識が多すぎます。"
            }
            IssueType::GodModule => {
                "関数、型、実装が多すぎて責務が集中しています。焦点の絞られたモジュールへの分割を検討してください。"
            }
            IssueType::PublicFieldExposure => {
                "構造体の公開フィールドが他モジュールから使われています。getterなどで結合を弱めることを検討してください。"
            }
            IssueType::PrimitiveObsession => {
                "同じプリミティブ型の引数が多すぎます。newtypeパターンで型安全性と明確さを高めることを検討してください。"
            }
        }
    }
}

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

/// Coupling dimension that most explains the current grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GradeDimension {
    /// Coupling strength is the dominant issue driver.
    Strength,
    /// Coupling distance is the dominant issue driver.
    Distance,
    /// Volatility or churn is the dominant issue driver.
    Volatility,
}

impl std::fmt::Display for GradeDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GradeDimension::Strength => write!(f, "strength"),
            GradeDimension::Distance => write!(f, "distance"),
            GradeDimension::Volatility => write!(f, "volatility"),
        }
    }
}

/// Top issue-type contribution used to explain a project grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueTypeContribution {
    /// Issue type contributing to the grade.
    pub issue_type: IssueType,
    /// Number of surfaced issues of this type.
    pub count: usize,
    /// Highest severity observed for this issue type.
    pub highest_severity: Severity,
}

/// Short explanation of why a project received its health grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradeRationale {
    /// Human-readable one-line explanation.
    pub summary: String,
    /// Highest-impact issue types by severity-weighted count.
    pub top_issue_types: Vec<IssueTypeContribution>,
    /// Dominant coupling dimension behind the surfaced issues.
    pub dominant_dimension: Option<GradeDimension>,
    /// Extra note when git churn or accidental volatility dominates.
    pub volatility_note: Option<String>,
}

impl GradeRationale {
    /// Empty rationale for hand-built test fixtures.
    pub fn empty() -> Self {
        Self {
            summary: "No surfaced coupling issues; grade reflects low issue density.".to_string(),
            top_issue_types: Vec::new(),
            dominant_dimension: None,
            volatility_note: None,
        }
    }
}

/// Specific refactoring actions
#[derive(Debug, Clone)]
pub enum RefactoringAction {
    /// Introduce a trait to abstract the coupling
    IntroduceTrait {
        suggested_name: String,
        methods: Vec<String>,
    },
    /// Move the component closer (same module/crate)
    MoveCloser { target_location: String },
    /// Extract an interface/adapter
    ExtractAdapter {
        adapter_name: String,
        purpose: String,
    },
    /// Split a large module
    SplitModule { suggested_modules: Vec<String> },
    /// Remove unnecessary abstraction
    SimplifyAbstraction { direct_usage: String },
    /// Break circular dependency
    BreakCycle { suggested_direction: String },
    /// Add stable interface
    StabilizeInterface { interface_name: String },
    /// General refactoring suggestion
    General { action: String },
    /// Add getter methods to replace direct field access
    AddGetters { fields: Vec<String> },
    /// Introduce newtype pattern for type safety
    IntroduceNewtype {
        suggested_name: String,
        wrapped_type: String,
    },
}

impl std::fmt::Display for RefactoringAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefactoringAction::IntroduceTrait {
                suggested_name,
                methods,
            } => {
                write!(
                    f,
                    "Introduce trait `{}` with methods: {}",
                    suggested_name,
                    methods.join(", ")
                )
            }
            RefactoringAction::MoveCloser { target_location } => {
                write!(f, "Move component to `{}`", target_location)
            }
            RefactoringAction::ExtractAdapter {
                adapter_name,
                purpose,
            } => {
                write!(f, "Extract adapter `{}` to {}", adapter_name, purpose)
            }
            RefactoringAction::SplitModule { suggested_modules } => {
                write!(f, "Split into modules: {}", suggested_modules.join(", "))
            }
            RefactoringAction::SimplifyAbstraction { direct_usage } => {
                write!(f, "Replace with direct usage: {}", direct_usage)
            }
            RefactoringAction::BreakCycle {
                suggested_direction,
            } => {
                write!(f, "Break cycle by {}", suggested_direction)
            }
            RefactoringAction::StabilizeInterface { interface_name } => {
                write!(f, "Add stable interface `{}`", interface_name)
            }
            RefactoringAction::General { action } => {
                write!(f, "{}", action)
            }
            RefactoringAction::AddGetters { fields } => {
                write!(f, "Add getter methods for: {}", fields.join(", "))
            }
            RefactoringAction::IntroduceNewtype {
                suggested_name,
                wrapped_type,
            } => {
                write!(
                    f,
                    "Introduce newtype: `struct {}({});`",
                    suggested_name, wrapped_type
                )
            }
        }
    }
}

// ===== Balance Scoring =====

/// Balance score for a coupling relationship
#[derive(Debug, Clone)]
pub struct BalanceScore {
    /// The coupling being scored
    pub coupling: CouplingMetrics,
    /// Overall balance score (0.0 - 1.0, higher = better balanced)
    pub score: f64,
    /// Whether strength and distance are aligned
    pub alignment: f64,
    /// Impact of volatility
    pub volatility_impact: f64,
    /// Interpretation of the score
    pub interpretation: BalanceInterpretation,
}

/// How to interpret a balance score
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BalanceInterpretation {
    /// Well-balanced, no action needed
    Balanced,
    /// Acceptable but could be improved
    Acceptable,
    /// Should be reviewed
    NeedsReview,
    /// Should be refactored
    NeedsRefactoring,
    /// Critical issue, must fix
    Critical,
}

impl std::fmt::Display for BalanceInterpretation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalanceInterpretation::Balanced => write!(f, "Balanced"),
            BalanceInterpretation::Acceptable => write!(f, "Acceptable"),
            BalanceInterpretation::NeedsReview => write!(f, "Needs Review"),
            BalanceInterpretation::NeedsRefactoring => write!(f, "Needs Refactoring"),
            BalanceInterpretation::Critical => write!(f, "Critical"),
        }
    }
}

impl BalanceScore {
    /// Calculate balance score for a coupling
    ///
    /// The formula implements: BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
    ///
    /// Ideal patterns:
    /// - Strong (1.0) + Close (0.0) → High alignment (cohesion)
    /// - Weak (0.0) + Far (1.0) → High alignment (loose coupling)
    ///
    /// Problematic patterns:
    /// - Strong (1.0) + Far (1.0) → Low alignment (global complexity)
    /// - Any + High volatility → Reduced by volatility impact
    pub fn calculate(coupling: &CouplingMetrics) -> Self {
        let strength = coupling.strength_value();
        let distance = coupling.distance_value();
        let volatility = coupling.volatility_value();

        // Alignment: how well strength and distance match the ideal patterns
        // Ideal: (strong + close) OR (weak + far)
        // XOR-like: difference between strength and distance
        // If both high or both low = misaligned, if opposite = aligned
        let alignment = 1.0 - (strength - (1.0 - distance)).abs();

        // Volatility impact: high volatility with strong coupling is bad
        // Only applies when there's actual coupling (strength > 0)
        let volatility_penalty = volatility * strength;
        let volatility_impact = 1.0 - volatility_penalty;

        // Combined score: both alignment AND stability matter
        // Using AND (multiplication) instead of OR (max) for stricter scoring
        let score = alignment * volatility_impact;

        // Determine interpretation based on score
        let interpretation = match score {
            s if s >= 0.8 => BalanceInterpretation::Balanced,
            s if s >= 0.6 => BalanceInterpretation::Acceptable,
            s if s >= 0.4 => BalanceInterpretation::NeedsReview,
            s if s >= 0.2 => BalanceInterpretation::NeedsRefactoring,
            _ => BalanceInterpretation::Critical,
        };

        Self {
            coupling: coupling.clone(),
            score,
            alignment,
            volatility_impact,
            interpretation,
        }
    }

    /// Check if this coupling is well-balanced (no action needed)
    pub fn is_balanced(&self) -> bool {
        matches!(
            self.interpretation,
            BalanceInterpretation::Balanced | BalanceInterpretation::Acceptable
        )
    }

    /// Check if this coupling needs refactoring
    pub fn needs_refactoring(&self) -> bool {
        matches!(
            self.interpretation,
            BalanceInterpretation::NeedsRefactoring | BalanceInterpretation::Critical
        )
    }
}

/// Thresholds for identifying issues
#[derive(Debug, Clone)]
pub struct IssueThresholds {
    /// Minimum strength value considered "strong"
    pub strong_coupling: f64,
    /// Minimum distance value considered "far"
    pub far_distance: f64,
    /// Minimum volatility value considered "high"
    pub high_volatility: f64,
    /// Number of dependencies to consider "high efferent coupling"
    pub max_dependencies: usize,
    /// Number of dependents to consider "high afferent coupling"
    pub max_dependents: usize,
    /// Maximum functions before flagging God Module
    pub max_functions: usize,
    /// Maximum types before flagging God Module
    pub max_types: usize,
    /// Maximum implementations before flagging God Module
    pub max_impls: usize,
    /// Minimum primitive parameter count for Primitive Obsession
    pub min_primitive_params: usize,
    /// Strict mode: only show Medium/High/Critical issues
    pub strict_mode: bool,
    /// Show explanations in Japanese
    pub japanese: bool,
    /// Exclude test code from function counts
    pub exclude_tests: bool,
    /// Prelude module patterns (for reporting purposes)
    pub prelude_module_count: usize,
}

impl Default for IssueThresholds {
    fn default() -> Self {
        Self {
            strong_coupling: 0.75,   // Functional strength or higher (was 0.5)
            far_distance: 0.5,       // DifferentModule or higher
            high_volatility: 0.75,   // High volatility only (was 0.5)
            max_dependencies: 20,    // More than 20 outgoing dependencies (was 15)
            max_dependents: 30,      // More than 30 incoming dependencies (was 20)
            max_functions: 30,       // More than 30 functions = God Module
            max_types: 15,           // More than 15 types = God Module
            max_impls: 20,           // More than 20 implementations = God Module
            min_primitive_params: 3, // 3+ primitive params = Primitive Obsession
            strict_mode: true,       // Show only important issues by default
            japanese: false,         // English by default
            exclude_tests: false,    // Include test code by default
            prelude_module_count: 0, // No prelude modules configured
        }
    }
}

// ===== External Crate Heuristics =====

/// Crate stability classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrateStability {
    /// Rust language fundamentals (std, core, alloc) - always ignore
    Fundamental,
    /// Highly stable, ubiquitous crates (serde, thiserror) - low concern
    Stable,
    /// Infrastructure crates (tokio, tracing) - medium concern
    Infrastructure,
    /// Regular external crate - normal analysis
    Normal,
}

/// Check the stability classification of a crate
pub fn classify_crate_stability(crate_name: &str) -> CrateStability {
    // Extract the base crate name (before ::)
    let base_name = crate_name.split("::").next().unwrap_or(crate_name).trim();

    match base_name {
        // Rust fundamentals - always safe to depend on
        "std" | "core" | "alloc" => CrateStability::Fundamental,

        // Highly stable, de-facto standard crates
        "serde" | "serde_json" | "serde_yaml" | "toml" |  // Serialization
        "thiserror" | "anyhow" |                           // Error handling
        "log" |                                            // Logging trait
        "chrono" | "time" |                                // Date/time
        "uuid" |                                           // UUIDs
        "regex" |                                          // Regex
        "lazy_static" | "once_cell" |                      // Statics
        "bytes" | "memchr" |                               // Byte utilities
        "itertools" |                                      // Iterator utilities
        "derive_more" | "strum"                            // Derive macros
        => CrateStability::Stable,

        // Infrastructure crates - stable but architectural decisions
        "tokio" | "async-std" | "smol" |                   // Async runtimes
        "async-trait" |                                    // Async traits
        "futures" | "futures-util" |                       // Futures
        "tracing" | "tracing-subscriber" |                 // Tracing
        "tracing-opentelemetry" | "opentelemetry" |        // Observability
        "opentelemetry-otlp" | "opentelemetry_sdk" |
        "hyper" | "reqwest" | "http" |                     // HTTP
        "tonic" | "prost" |                                // gRPC
        "sqlx" | "diesel" | "sea-orm" |                    // Database
        "clap" | "structopt"                               // CLI
        => CrateStability::Infrastructure,

        // Everything else
        _ => CrateStability::Normal,
    }
}

/// Check if a crate should be excluded from issue detection
pub fn should_skip_crate(crate_name: &str) -> bool {
    matches!(
        classify_crate_stability(crate_name),
        CrateStability::Fundamental
    )
}

/// Check if a crate should have reduced severity
pub fn should_reduce_severity(crate_name: &str) -> bool {
    matches!(
        classify_crate_stability(crate_name),
        CrateStability::Stable | CrateStability::Infrastructure
    )
}

/// Check if this is an external crate (not part of the workspace)
/// External crates are identified by not containing "::" or starting with known external patterns
pub fn is_external_crate(target: &str, source: &str) -> bool {
    // If target doesn't have ::, it might be external
    // But we need to check if it's the same workspace member

    // Extract the crate/module prefix
    let target_prefix = target.split("::").next().unwrap_or(target);
    let source_prefix = source.split("::").next().unwrap_or(source);

    // If they have the same prefix, it's internal
    if target_prefix == source_prefix {
        return false;
    }

    // If target looks like an external crate pattern (no workspace prefix match)
    // Check if it's a known stable/infrastructure crate
    let stability = classify_crate_stability(target);
    matches!(
        stability,
        CrateStability::Fundamental | CrateStability::Stable | CrateStability::Infrastructure
    )
}

// ===== Issue Detection =====

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

    // Pattern 1: Global Complexity (INTRUSIVE coupling + different module)
    // Only flag intrusive coupling across module boundaries - functional coupling is normal
    if coupling.strength == IntegrationStrength::Intrusive
        && coupling.distance == Distance::DifferentModule
    {
        issues.push(CouplingIssue {
            issue_type: IssueType::GlobalComplexity,
            severity: Severity::Medium, // Medium, not High - it's internal
            source: coupling.source.clone(),
            target: coupling.target.clone(),
            description: format!(
                "Intrusive coupling to {} across module boundary",
                coupling.target,
            ),
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

/// Analyze all couplings in a project and identify issues
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

    // Count issues by severity
    let mut issues_by_severity: HashMap<Severity, usize> = HashMap::new();
    for issue in &all_issues {
        *issues_by_severity.entry(issue.severity).or_insert(0) += 1;
    }

    // Count issues by type
    let mut issues_by_type: HashMap<IssueType, usize> = HashMap::new();
    for issue in &all_issues {
        *issues_by_type.entry(issue.issue_type).or_insert(0) += 1;
    }

    // Determine overall health grade based on INTERNAL coupling issues
    let health_grade = calculate_health_grade(&issues_by_severity, internal_couplings.max(1));
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

fn build_grade_rationale(
    issues: &[CouplingIssue],
    internal_couplings: usize,
    japanese: bool,
) -> GradeRationale {
    if issues.is_empty() {
        let summary = if japanese {
            if internal_couplings == 0 {
                "問題密度の採点に使える内部結合はありませんでした。".to_string()
            } else {
                format!(
                    "{} 件の内部結合に検出対象の問題はありません。問題密度が低いため、このグレードになっています。",
                    internal_couplings
                )
            }
        } else if internal_couplings == 0 {
            "No internal couplings were available for issue-density scoring.".to_string()
        } else {
            format!(
                "No surfaced coupling issues across {} internal coupling(s); grade reflects low issue density.",
                internal_couplings
            )
        };
        return GradeRationale {
            summary,
            ..GradeRationale::empty()
        };
    }

    let mut by_type: HashMap<IssueType, (usize, Severity, usize)> = HashMap::new();
    let mut by_dimension: HashMap<GradeDimension, usize> = HashMap::new();
    let mut high_or_critical = 0;

    for issue in issues {
        let weight = severity_weight(issue.severity);
        let entry = by_type
            .entry(issue.issue_type)
            .or_insert((0, issue.severity, 0));
        entry.0 += 1;
        entry.1 = entry.1.max(issue.severity);
        entry.2 += weight;
        *by_dimension
            .entry(dimension_for_issue(issue.issue_type))
            .or_default() += weight;

        if issue.severity >= Severity::High {
            high_or_critical += 1;
        }
    }

    let mut ranked_types: Vec<_> = by_type
        .into_iter()
        .map(|(issue_type, (count, highest_severity, weighted_score))| {
            (
                IssueTypeContribution {
                    issue_type,
                    count,
                    highest_severity,
                },
                weighted_score,
            )
        })
        .collect();
    ranked_types.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.0.count.cmp(&a.0.count))
            .then_with(|| a.0.issue_type.to_string().cmp(&b.0.issue_type.to_string()))
    });

    let top_issue_types: Vec<_> = ranked_types
        .into_iter()
        .take(3)
        .map(|(contribution, _)| contribution)
        .collect();

    let dominant_dimension = by_dimension
        .into_iter()
        .max_by_key(|(_, score)| *score)
        .map(|(dimension, _)| dimension);

    let volatility_issue_count = issues
        .iter()
        .filter(|issue| dimension_for_issue(issue.issue_type) == GradeDimension::Volatility)
        .count();
    let accidental_count = issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::AccidentalVolatility)
        .count();
    let volatility_note = if volatility_issue_count > 0 {
        let accidental_suffix = if accidental_count > 0 {
            if japanese {
                format!(" (偶発的な変更頻度 {} 件を含む)", accidental_count)
            } else {
                format!(
                    ", including {} accidental-volatility finding(s)",
                    accidental_count
                )
            }
        } else {
            String::new()
        };
        if japanese {
            Some(format!(
                "変更頻度/チャーンが {} 件の問題として影響しています{}。",
                volatility_issue_count, accidental_suffix
            ))
        } else {
            Some(format!(
                "Volatility/churn contributes through {} issue(s){}.",
                volatility_issue_count, accidental_suffix
            ))
        }
    } else {
        None
    };

    let top_phrase = top_issue_types
        .iter()
        .map(|item| {
            if japanese {
                format!(
                    "{} ({})",
                    issue_type_japanese_label(item.issue_type),
                    item.count
                )
            } else {
                format!("{} ({})", item.issue_type, item.count)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let severity_phrase = if japanese {
        if high_or_critical > 0 {
            format!("高/緊急の問題 {} 件", high_or_critical)
        } else {
            format!("中/低の問題 {} 件", issues.len())
        }
    } else if high_or_critical > 0 {
        format!("{} high/critical issue(s)", high_or_critical)
    } else {
        format!("{} medium/low issue(s)", issues.len())
    };
    let dimension_phrase = dominant_dimension
        .map(|dimension| {
            if japanese {
                format!(
                    "。最大の要因は{}です",
                    grade_dimension_japanese_label(dimension)
                )
            } else {
                format!("; {} is the largest contributor", dimension)
            }
        })
        .unwrap_or_default();
    let note_phrase = volatility_note
        .as_ref()
        .map(|note| {
            if japanese {
                note.clone()
            } else {
                format!(" {}", note)
            }
        })
        .unwrap_or_default();

    let summary = if japanese {
        format!(
            "{}が主な理由です。特に {} が目立ちます{}。{note_phrase}",
            severity_phrase, top_phrase, dimension_phrase
        )
    } else {
        format!(
            "Driven by {}, led by {}{}.{note_phrase}",
            severity_phrase, top_phrase, dimension_phrase
        )
    };

    GradeRationale {
        summary,
        top_issue_types,
        dominant_dimension,
        volatility_note,
    }
}

fn grade_dimension_japanese_label(dimension: GradeDimension) -> &'static str {
    match dimension {
        GradeDimension::Strength => "結合強度",
        GradeDimension::Distance => "距離",
        GradeDimension::Volatility => "変更頻度",
    }
}

fn issue_type_japanese_label(issue_type: IssueType) -> &'static str {
    match issue_type {
        IssueType::GlobalComplexity => "グローバル複雑性",
        IssueType::CascadingChangeRisk => "変更波及リスク",
        IssueType::InappropriateIntimacy => "不適切な親密さ",
        IssueType::HighEfferentCoupling => "出力依存過多",
        IssueType::HighAfferentCoupling => "入力依存過多",
        IssueType::UnnecessaryAbstraction => "過剰な抽象化",
        IssueType::CircularDependency => "循環依存",
        IssueType::HiddenCoupling => "隠れた結合",
        IssueType::AccidentalVolatility => "偶発的な変更頻度",
        IssueType::ScatteredExternalCoupling => "外部クレート結合の分散",
        IssueType::ShallowModule => "浅いモジュール",
        IssueType::PassThroughMethod => "パススルーメソッド",
        IssueType::HighCognitiveLoad => "高認知負荷",
        IssueType::GodModule => "神モジュール",
        IssueType::PublicFieldExposure => "公開フィールド",
        IssueType::PrimitiveObsession => "プリミティブ過多",
    }
}

fn severity_weight(severity: Severity) -> usize {
    match severity {
        Severity::Critical => 4,
        Severity::High => 3,
        Severity::Medium => 2,
        Severity::Low => 1,
    }
}

fn dimension_for_issue(issue_type: IssueType) -> GradeDimension {
    match issue_type {
        IssueType::CascadingChangeRisk
        | IssueType::HiddenCoupling
        | IssueType::AccidentalVolatility => GradeDimension::Volatility,
        IssueType::InappropriateIntimacy
        | IssueType::PublicFieldExposure
        | IssueType::PrimitiveObsession
        | IssueType::ShallowModule
        | IssueType::PassThroughMethod => GradeDimension::Strength,
        IssueType::GlobalComplexity
        | IssueType::ScatteredExternalCoupling
        | IssueType::HighEfferentCoupling
        | IssueType::HighAfferentCoupling
        | IssueType::UnnecessaryAbstraction
        | IssueType::CircularDependency
        | IssueType::HighCognitiveLoad
        | IssueType::GodModule => GradeDimension::Distance,
    }
}

// ===== Temporal and Subdomain Signals =====

/// Analyze temporal co-change pairs that have no explicit code dependency.
fn analyze_hidden_temporal_coupling(metrics: &ProjectMetrics) -> Vec<CouplingIssue> {
    let file_to_module = build_file_to_module_map(metrics);
    let mut seen = HashSet::new();
    let mut issues = Vec::new();

    for temporal in metrics
        .temporal_couplings
        .iter()
        .filter(|tc| tc.is_strong())
    {
        let Some(source) = module_for_file(&temporal.file_a, &file_to_module) else {
            continue;
        };
        let Some(target) = module_for_file(&temporal.file_b, &file_to_module) else {
            continue;
        };

        if source == target || has_code_coupling(metrics, &source, &target) {
            continue;
        }

        let (stable_source, stable_target) = ordered_pair(&source, &target);
        if !seen.insert((stable_source.clone(), stable_target.clone())) {
            continue;
        }

        let ratio_pct = temporal.coupling_ratio * 100.0;
        issues.push(CouplingIssue {
            issue_type: IssueType::HiddenCoupling,
            severity: if temporal.coupling_ratio >= 0.8 {
                Severity::High
            } else {
                Severity::Medium
            },
            source: stable_source,
            target: stable_target,
            description: format!(
                "Strong temporal co-change without code dependency ({:.0}% ratio, {} co-changes)",
                ratio_pct, temporal.co_change_count
            ),
            refactoring: RefactoringAction::General {
                action: "Extract a shared abstraction or make the dependency explicit".to_string(),
            },
            balance_score: 1.0 - temporal.coupling_ratio,
        });
    }

    issues
}

/// Analyze modules whose git churn contradicts their expected subdomain volatility.
fn analyze_accidental_volatility(metrics: &ProjectMetrics) -> Vec<CouplingIssue> {
    if metrics.file_changes.len() < 4 {
        return Vec::new();
    }

    let threshold = top_quartile_change_threshold(&metrics.file_changes);
    let mut issues = Vec::new();

    for (module_name, module) in &metrics.modules {
        let Some(subdomain @ (Subdomain::Supporting | Subdomain::Generic)) = module.subdomain
        else {
            continue;
        };
        let change_count = change_count_for_path(&module.path, &metrics.file_changes);
        if change_count < threshold {
            continue;
        }

        issues.push(CouplingIssue {
            issue_type: IssueType::AccidentalVolatility,
            severity: Severity::Medium,
            source: module_name.clone(),
            target: module_name.clone(),
            description: format!(
                "Accidental volatility: {} is {} but changed {} times (top-quartile threshold: {})",
                module_name, subdomain, change_count, threshold
            ),
            refactoring: RefactoringAction::General {
                action: "Separate volatile policy from stable supporting/generic implementation"
                    .to_string(),
            },
            balance_score: 0.5,
        });
    }

    issues
}

fn build_file_to_module_map(metrics: &ProjectMetrics) -> Vec<(String, String)> {
    metrics
        .modules
        .iter()
        .map(|(name, module)| (normalize_path_string(&module.path), name.clone()))
        .collect()
}

fn module_for_file(file_path: &str, file_to_module: &[(String, String)]) -> Option<String> {
    let normalized_file = normalize_path_str(file_path);
    file_to_module
        .iter()
        .find(|(module_path, _)| paths_match(module_path, &normalized_file))
        .map(|(_, module_name)| module_name.clone())
}

/// Checks explicit couplings by assuming coupling source/target names end with
/// the short module name stored in `ProjectMetrics::modules`.
fn has_code_coupling(metrics: &ProjectMetrics, module_a: &str, module_b: &str) -> bool {
    metrics.couplings.iter().any(|coupling| {
        module_names_match(&coupling.source, module_a)
            && module_names_match(&coupling.target, module_b)
            || module_names_match(&coupling.source, module_b)
                && module_names_match(&coupling.target, module_a)
    })
}

fn module_names_match(coupling_module: &str, module_name: &str) -> bool {
    coupling_module == module_name
        || coupling_module
            .strip_suffix(module_name)
            .is_some_and(|prefix| prefix.ends_with("::"))
}

fn ordered_pair(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

fn top_quartile_change_threshold(file_changes: &HashMap<String, usize>) -> usize {
    let mut counts: Vec<_> = file_changes.values().copied().collect();
    counts.sort_unstable();
    let idx = counts.len() * 3 / 4;
    counts[idx.min(counts.len() - 1)].max(3)
}

fn change_count_for_path(path: &std::path::Path, file_changes: &HashMap<String, usize>) -> usize {
    let module_path = normalize_path_string(path);
    file_changes
        .iter()
        .filter(|(file_path, _)| paths_match(&module_path, &normalize_path_str(file_path)))
        .map(|(_, changes)| *changes)
        .max()
        .unwrap_or(0)
}

fn paths_match(module_path: &str, git_path: &str) -> bool {
    module_path == git_path
        || module_path.ends_with(&format!("/{git_path}"))
        || git_path.ends_with(&format!("/{module_path}"))
}

fn normalize_path_string(path: &std::path::Path) -> String {
    normalize_path_str(&path.to_string_lossy())
}

fn normalize_path_str(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

// ===== Module Pattern Detectors =====

/// Analyze module-level coupling (hub detection)
fn analyze_module_coupling(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
) -> Vec<CouplingIssue> {
    let mut issues = Vec::new();

    // Count outgoing (efferent) and incoming (afferent) couplings per module
    // Only count INTERNAL dependencies (within workspace), not external crates
    let mut efferent: HashMap<&str, usize> = HashMap::new();
    let mut afferent: HashMap<&str, usize> = HashMap::new();

    for coupling in &metrics.couplings {
        // Skip external crate dependencies entirely
        if coupling.distance == Distance::DifferentCrate {
            continue;
        }

        *efferent.entry(&coupling.source).or_insert(0) += 1;
        *afferent.entry(&coupling.target).or_insert(0) += 1;
    }

    // Check for high efferent coupling (depends on too many things)
    for (module, count) in &efferent {
        if *count > thresholds.max_dependencies {
            issues.push(CouplingIssue {
                issue_type: IssueType::HighEfferentCoupling,
                severity: if *count > thresholds.max_dependencies * 2 {
                    Severity::High
                } else {
                    Severity::Medium
                },
                source: module.to_string(),
                target: format!("{} dependencies", count),
                description: format!(
                    "Module {} depends on {} other components (threshold: {})",
                    module, count, thresholds.max_dependencies
                ),
                refactoring: RefactoringAction::SplitModule {
                    suggested_modules: vec![
                        format!("{}_core", module),
                        format!("{}_integration", module),
                    ],
                },
                balance_score: 1.0
                    - (*count as f64 / (thresholds.max_dependencies * 3) as f64).min(1.0),
            });
        }
    }

    // Check for high afferent coupling (too many things depend on this)
    // Only internal modules are counted (external crates already filtered above)
    for (module, count) in &afferent {
        if *count > thresholds.max_dependents {
            issues.push(CouplingIssue {
                issue_type: IssueType::HighAfferentCoupling,
                severity: if *count > thresholds.max_dependents * 2 {
                    Severity::High
                } else {
                    Severity::Medium
                },
                source: format!("{} dependents", count),
                target: module.to_string(),
                description: format!(
                    "Module {} is depended on by {} other components (threshold: {})",
                    module, count, thresholds.max_dependents
                ),
                refactoring: RefactoringAction::IntroduceTrait {
                    suggested_name: format!("{}Interface", extract_type_name(module)),
                    methods: vec!["// Define stable public API".to_string()],
                },
                balance_score: 1.0
                    - (*count as f64 / (thresholds.max_dependents * 3) as f64).min(1.0),
            });
        }
    }

    issues
}

/// Analyze Rust-specific patterns (God Module, Public Field Exposure, Primitive Obsession)
fn analyze_rust_patterns(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
) -> Vec<CouplingIssue> {
    let mut issues = Vec::new();

    // God Module detection
    for (module_name, module) in &metrics.modules {
        // Calculate function count, excluding test functions if configured
        let func_count = if thresholds.exclude_tests {
            module
                .function_count()
                .saturating_sub(module.test_function_count)
        } else {
            module.function_count()
        };
        let type_count = module.type_definitions.len();
        let impl_count = module.trait_impl_count + module.inherent_impl_count;

        // Check if module exceeds thresholds (with test exclusion applied)
        let is_god_module = func_count > thresholds.max_functions
            || type_count > thresholds.max_types
            || impl_count > thresholds.max_impls;

        if is_god_module {
            issues.push(CouplingIssue {
                issue_type: IssueType::GodModule,
                severity: if func_count > thresholds.max_functions * 2
                    || type_count > thresholds.max_types * 2
                {
                    Severity::High
                } else {
                    Severity::Medium
                },
                source: module_name.clone(),
                target: format!(
                    "{} functions, {} types, {} impls",
                    func_count, type_count, impl_count
                ),
                description: format!(
                    "Module {} has too many responsibilities (functions: {}/{}, types: {}/{}, impls: {}/{})",
                    module_name,
                    func_count, thresholds.max_functions,
                    type_count, thresholds.max_types,
                    impl_count, thresholds.max_impls,
                ),
                refactoring: RefactoringAction::SplitModule {
                    suggested_modules: vec![
                        format!("{}_core", module_name),
                        format!("{}_helpers", module_name),
                    ],
                },
                balance_score: 0.5,
            });
        }

        // Public Field Exposure detection
        for type_def in module.type_definitions.values() {
            if type_def.public_field_count > 0
                && !type_def.is_trait
                && type_def.visibility == crate::metrics::Visibility::Public
            {
                issues.push(CouplingIssue {
                    issue_type: IssueType::PublicFieldExposure,
                    severity: Severity::Low,
                    source: format!("{}::{}", module_name, type_def.name),
                    target: format!("{} public fields", type_def.public_field_count),
                    description: format!(
                        "Type {} has {} public field(s). Consider using getter methods.",
                        type_def.name, type_def.public_field_count
                    ),
                    refactoring: RefactoringAction::AddGetters {
                        fields: vec!["// Add getter methods".to_string()],
                    },
                    balance_score: 0.7,
                });
            }
        }

        // Primitive Obsession detection
        for func_def in module.function_definitions.values() {
            if func_def.primitive_param_count >= thresholds.min_primitive_params
                && func_def.param_count >= thresholds.min_primitive_params
            {
                let ratio = func_def.primitive_param_count as f64 / func_def.param_count as f64;
                if ratio >= 0.6 {
                    issues.push(CouplingIssue {
                        issue_type: IssueType::PrimitiveObsession,
                        severity: Severity::Low,
                        source: format!("{}::{}", module_name, func_def.name),
                        target: format!(
                            "{}/{} primitive params",
                            func_def.primitive_param_count, func_def.param_count
                        ),
                        description: format!(
                            "Function {} has {} primitive parameters. Consider newtype pattern.",
                            func_def.name, func_def.primitive_param_count
                        ),
                        refactoring: RefactoringAction::IntroduceNewtype {
                            suggested_name: format!("{}Params", capitalize_first(&func_def.name)),
                            wrapped_type: "// Group related parameters".to_string(),
                        },
                        balance_score: 0.7,
                    });
                }
            }
        }
    }

    issues
}

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// ===== Health Grading =====

/// Health grade for the overall project
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthGrade {
    /// Over-optimized signal; the project may be chasing too little coupling.
    S,
    /// Coupling is appropriate for the architecture.
    A,
    /// Minor issues exist but remain manageable.
    B,
    /// Structural issues need planned improvement.
    C,
    /// Significant issues are affecting maintainability.
    D,
    /// Critical issues are blocking safe change.
    F,
}

impl HealthGrade {
    /// Single-letter representation (S, A, B, C, D, F).
    ///
    /// Used for compact, machine-readable output such as the history
    /// timeline where the verbose `Display` form is too noisy.
    pub fn letter(&self) -> char {
        match self {
            HealthGrade::S => 'S',
            HealthGrade::A => 'A',
            HealthGrade::B => 'B',
            HealthGrade::C => 'C',
            HealthGrade::D => 'D',
            HealthGrade::F => 'F',
        }
    }
}

impl std::fmt::Display for HealthGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthGrade::S => write!(f, "S (Over-optimized! Real code has some issues. Ship it!)"),
            HealthGrade::A => write!(f, "A (Well-balanced)"),
            HealthGrade::B => write!(f, "B (Healthy)"),
            HealthGrade::C => write!(f, "C (Room for improvement)"),
            HealthGrade::D => write!(f, "D (Attention needed)"),
            HealthGrade::F => write!(f, "F (Immediate action required)"),
        }
    }
}

/// Calculate health grade based on multiple quality factors
///
/// Unlike the previous version that only checked for issues,
/// this version also considers positive quality indicators:
/// - Contract coupling rate (trait usage)
/// - Balance score distribution
/// - Internal coupling complexity
fn calculate_health_grade(
    issues_by_severity: &HashMap<Severity, usize>,
    internal_couplings: usize,
) -> HealthGrade {
    let critical = *issues_by_severity.get(&Severity::Critical).unwrap_or(&0);
    let high = *issues_by_severity.get(&Severity::High).unwrap_or(&0);
    let medium = *issues_by_severity.get(&Severity::Medium).unwrap_or(&0);

    // No internal couplings = B (not A - we can't assess quality without data)
    if internal_couplings == 0 {
        return HealthGrade::B;
    }

    // F: Multiple critical issues
    if critical > 3 {
        return HealthGrade::F;
    }

    // Calculate issue density (issues per internal coupling)
    let high_density = high as f64 / internal_couplings as f64;
    let medium_density = medium as f64 / internal_couplings as f64;
    let total_issue_density = (critical + high + medium) as f64 / internal_couplings as f64;

    // D: Critical issues or very high issue density (> 5% high)
    if critical > 0 || high_density > 0.05 {
        return HealthGrade::D;
    }

    // C: Any high issues OR high medium density (> 25%)
    // Projects with structural issues that need attention
    if high > 0 || medium_density > 0.25 {
        return HealthGrade::C;
    }

    // B: Some medium issues but manageable (> 10% medium density)
    if medium_density > 0.10 || total_issue_density > 0.15 {
        return HealthGrade::B;
    }

    // S: Over-optimized! Too few issues (< 5%) = you're probably over-engineering
    // This is a WARNING, not a reward. Stop refactoring!
    if high == 0 && medium_density <= 0.05 && internal_couplings >= 20 {
        return HealthGrade::S;
    }

    // A: Well-balanced - no high issues AND reasonable medium issues (5-10%)
    // This is the ideal target grade
    if high == 0 && medium_density <= 0.10 && internal_couplings >= 10 {
        return HealthGrade::A;
    }

    // Default to B for projects with few issues
    HealthGrade::B
}

/// Complete project balance analysis report
#[derive(Debug)]
pub struct ProjectBalanceReport {
    /// Total couplings considered in the report.
    pub total_couplings: usize,
    /// Number of internal couplings considered balanced or acceptable.
    pub balanced_count: usize,
    /// Number of internal couplings that need review.
    pub needs_review: usize,
    /// Number of internal couplings that need refactoring.
    pub needs_refactoring: usize,
    /// Average balance score across internal couplings.
    pub average_score: f64,
    /// Overall project health grade derived from issue density.
    pub health_grade: HealthGrade,
    /// Issue counts grouped by severity.
    pub issues_by_severity: HashMap<Severity, usize>,
    /// Issue counts grouped by issue type.
    pub issues_by_type: HashMap<IssueType, usize>,
    /// All detected issues after threshold and strict-mode filtering.
    pub issues: Vec<CouplingIssue>,
    /// Highest-priority issues selected for concise reporting.
    pub top_priorities: Vec<CouplingIssue>,
    /// Concise explanation of why the health grade was assigned.
    pub grade_rationale: GradeRationale,
}

impl ProjectBalanceReport {
    /// Add top N priority issues
    fn with_top_priorities(mut self, n: usize) -> Self {
        self.top_priorities = self.issues.iter().take(n).cloned().collect();
        self
    }

    /// Get issues grouped by type
    pub fn issues_grouped_by_type(&self) -> HashMap<IssueType, Vec<&CouplingIssue>> {
        let mut grouped: HashMap<IssueType, Vec<&CouplingIssue>> = HashMap::new();
        for issue in &self.issues {
            grouped.entry(issue.issue_type).or_default().push(issue);
        }
        grouped
    }
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

fn build_target_subdomain_map(metrics: &ProjectMetrics) -> HashMap<String, Option<Subdomain>> {
    let mut targets = HashMap::new();

    for (module_key, module) in &metrics.modules {
        let Some(subdomain) = module.subdomain else {
            continue;
        };

        insert_subdomain_alias(&mut targets, module_key, subdomain);
        insert_subdomain_alias(&mut targets, &module.name, subdomain);

        if let Some(short_name) = module_key.rsplit("::").next() {
            insert_subdomain_alias(&mut targets, short_name, subdomain);
        }
        if let Some(short_name) = module.name.rsplit("::").next() {
            insert_subdomain_alias(&mut targets, short_name, subdomain);
        }
        if let Some(file_stem) = module.path.file_stem().and_then(|stem| stem.to_str()) {
            insert_subdomain_alias(&mut targets, file_stem, subdomain);
        }

        for type_name in module.type_definitions.keys() {
            insert_subdomain_alias(&mut targets, type_name, subdomain);
        }
        for function_name in module.function_definitions.keys() {
            insert_subdomain_alias(&mut targets, function_name, subdomain);
        }
    }

    targets
}

fn insert_subdomain_alias(
    targets: &mut HashMap<String, Option<Subdomain>>,
    alias: &str,
    subdomain: Subdomain,
) {
    if !alias.is_empty() {
        targets
            .entry(alias.to_string())
            .and_modify(|existing| {
                if *existing != Some(subdomain) {
                    *existing = None;
                }
            })
            .or_insert(Some(subdomain));
    }
}

fn coupling_with_essential_volatility(
    coupling: &CouplingMetrics,
    target_subdomains: &HashMap<String, Option<Subdomain>>,
) -> CouplingMetrics {
    let Some(subdomain) = target_subdomain_for_coupling(&coupling.target, target_subdomains) else {
        return coupling.clone();
    };

    let mut effective = coupling.clone();
    effective.volatility = subdomain.expected_volatility();
    effective
}

fn target_subdomain_for_coupling(
    target: &str,
    target_subdomains: &HashMap<String, Option<Subdomain>>,
) -> Option<Subdomain> {
    target_subdomains
        .get(target)
        .copied()
        .flatten()
        .or_else(|| {
            target
                .rsplit("::")
                .find_map(|part| target_subdomains.get(part).copied().flatten())
        })
}

// ===== Labels and Formatting Helpers =====

fn extract_type_name(path: &str) -> String {
    path.split("::")
        .last()
        .unwrap_or(path)
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect()
}

/// Get human-readable label for integration strength
pub fn strength_label(strength: IntegrationStrength) -> &'static str {
    match strength {
        IntegrationStrength::Intrusive => "Intrusive",
        IntegrationStrength::Functional => "Functional",
        IntegrationStrength::Model => "Model",
        IntegrationStrength::Contract => "Contract",
    }
}

/// Get human-readable label for distance
pub fn distance_label(distance: Distance) -> &'static str {
    match distance {
        Distance::SameFunction => "same function",
        Distance::SameModule => "same module",
        Distance::DifferentModule => "different module",
        Distance::DifferentCrate => "external crate",
    }
}

/// Get human-readable label for volatility
pub fn volatility_label(volatility: Volatility) -> &'static str {
    match volatility {
        Volatility::Low => "rarely",
        Volatility::Medium => "sometimes",
        Volatility::High => "frequently",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::metrics::{ModuleMetrics, Visibility};
    use crate::volatility::TemporalCoupling;

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
    fn test_identify_cascading_change() {
        // Now only INTRUSIVE + High volatility triggers cascading change risk
        let coupling = make_coupling(
            IntegrationStrength::Intrusive,
            Distance::SameModule,
            Volatility::High,
        );
        let issues = identify_issues(&coupling);
        assert!(
            issues
                .iter()
                .any(|i| i.issue_type == IssueType::CascadingChangeRisk),
            "Intrusive coupling + High volatility should detect CascadingChangeRisk"
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
