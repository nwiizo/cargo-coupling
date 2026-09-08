// ===== Balance Scoring =====

use crate::metrics::coupling::CouplingMetrics;

/// Version of the numeric methodology, recorded with exported analyses.
pub const SCORING_VERSION: &str = "khononov-compensation-v2";

/// Compensation over normalized dimensions. Used for both observations and
/// explicit what-if scenarios so their calculations cannot drift apart.
pub(crate) fn normalized_balance(strength: f64, distance: f64, volatility: f64) -> f64 {
    (strength - distance).abs().max(1.0 - volatility)
}

#[cfg(test)]
mod compensation_tests {
    use super::*;
    use crate::{Distance, IntegrationStrength, Volatility};

    #[test]
    fn stability_compensates_for_every_strength_and_distance() {
        for strength in [
            IntegrationStrength::Intrusive,
            IntegrationStrength::Functional,
            IntegrationStrength::Model,
            IntegrationStrength::Contract,
        ] {
            for distance in [
                Distance::SameFunction,
                Distance::SameModule,
                Distance::DifferentModule,
                Distance::DifferentCrate,
            ] {
                let coupling = CouplingMetrics::new(
                    "consumer".into(),
                    "provider".into(),
                    strength,
                    distance,
                    Volatility::Low,
                );
                let score = BalanceScore::calculate(&coupling);
                assert_eq!(score.score, 1.0, "{strength:?}, {distance:?}");
                assert!(score.is_balanced());
            }
        }
    }

    #[test]
    fn proximity_preserves_cohesion_when_implementation_is_volatile() {
        let coupling = CouplingMetrics::new(
            "a".into(),
            "a".into(),
            IntegrationStrength::Intrusive,
            Distance::SameFunction,
            Volatility::High,
        );
        assert_eq!(BalanceScore::calculate(&coupling).score, 1.0);
    }
}

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
    /// - Instability removes compensation; it does not cancel good alignment.
    pub fn calculate(coupling: &CouplingMetrics) -> Self {
        let strength = coupling.strength_value();
        let distance = coupling.distance_value();
        let volatility = coupling.volatility_value();

        // Chapter 10: XOR is distance between the two dimension values;
        // OR is max. Stability can compensate for complexity, and volatility
        // cannot erase modularity already achieved by proximity/encapsulation.
        let alignment = (strength - distance).abs();
        let volatility_impact = 1.0 - volatility;
        let score = normalized_balance(strength, distance, volatility);

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
