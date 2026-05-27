use std::path::PathBuf;

use crate::volatility::Volatility;

use super::dimensions::{Distance, IntegrationStrength, Visibility};

#[derive(Debug, Clone, Default)]
pub struct CouplingLocation {
    /// File path where the coupling originates
    pub file_path: Option<PathBuf>,
    /// Line number in the source file
    pub line: usize,
}

/// Metrics for a single coupling relationship
#[derive(Debug, Clone)]
pub struct CouplingMetrics {
    /// Source component
    pub source: String,
    /// Target component
    pub target: String,
    /// Integration strength
    pub strength: IntegrationStrength,
    /// Distance between components
    pub distance: Distance,
    /// Volatility of the target
    pub volatility: Volatility,
    /// Source crate name (when workspace analysis is available)
    pub source_crate: Option<String>,
    /// Target crate name (when workspace analysis is available)
    pub target_crate: Option<String>,
    /// Visibility of the target item (for intrusive detection)
    pub target_visibility: Visibility,
    /// Location where the coupling occurs
    pub location: CouplingLocation,
}

impl CouplingMetrics {
    /// Create new coupling metrics
    pub fn new(
        source: String,
        target: String,
        strength: IntegrationStrength,
        distance: Distance,
        volatility: Volatility,
    ) -> Self {
        Self {
            source,
            target,
            strength,
            distance,
            volatility,
            source_crate: None,
            target_crate: None,
            target_visibility: Visibility::default(),
            location: CouplingLocation::default(),
        }
    }

    /// Create new coupling metrics with visibility
    pub fn with_visibility(
        source: String,
        target: String,
        strength: IntegrationStrength,
        distance: Distance,
        volatility: Volatility,
        visibility: Visibility,
    ) -> Self {
        Self {
            source,
            target,
            strength,
            distance,
            volatility,
            source_crate: None,
            target_crate: None,
            target_visibility: visibility,
            location: CouplingLocation::default(),
        }
    }

    /// Create new coupling metrics with location
    #[allow(clippy::too_many_arguments)]
    pub fn with_location(
        source: String,
        target: String,
        strength: IntegrationStrength,
        distance: Distance,
        volatility: Volatility,
        visibility: Visibility,
        file_path: PathBuf,
        line: usize,
    ) -> Self {
        Self {
            source,
            target,
            strength,
            distance,
            volatility,
            source_crate: None,
            target_crate: None,
            target_visibility: visibility,
            location: CouplingLocation {
                file_path: Some(file_path),
                line,
            },
        }
    }

    /// Check if this coupling represents intrusive access based on visibility
    ///
    /// Returns true if the target's visibility suggests this is access to
    /// internal implementation details rather than a public API.
    pub fn is_visibility_intrusive(&self) -> bool {
        let same_crate = self.source_crate == self.target_crate;
        let same_module =
            self.distance == Distance::SameModule || self.distance == Distance::SameFunction;
        self.target_visibility
            .is_intrusive_from(same_crate, same_module)
    }

    /// Get effective strength considering visibility
    ///
    /// If the target is not publicly visible and being accessed from outside,
    /// the coupling is considered more intrusive.
    pub fn effective_strength(&self) -> IntegrationStrength {
        if self.is_visibility_intrusive() && self.strength != IntegrationStrength::Intrusive {
            // Upgrade to more intrusive if accessing non-public items
            match self.strength {
                IntegrationStrength::Contract => IntegrationStrength::Model,
                IntegrationStrength::Model => IntegrationStrength::Functional,
                IntegrationStrength::Functional => IntegrationStrength::Intrusive,
                IntegrationStrength::Intrusive => IntegrationStrength::Intrusive,
            }
        } else {
            self.strength
        }
    }

    /// Get effective strength value considering visibility
    pub fn effective_strength_value(&self) -> f64 {
        self.effective_strength().value()
    }

    /// Get numeric strength value
    pub fn strength_value(&self) -> f64 {
        self.strength.value()
    }

    /// Get numeric distance value
    pub fn distance_value(&self) -> f64 {
        self.distance.value()
    }

    /// Get numeric volatility value
    pub fn volatility_value(&self) -> f64 {
        self.volatility.value()
    }
}

// ===== Module-Level Facts =====
