// ===== Balance Scoring =====

use crate::metrics::coupling::CouplingMetrics;

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
