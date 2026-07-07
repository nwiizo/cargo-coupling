use std::fmt;
use std::path::Path;

use crate::volatility::Volatility;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Visibility {
    /// Fully public (`pub`)
    Public,
    /// Crate-internal (`pub(crate)`)
    PubCrate,
    /// Super-module visible (`pub(super)`)
    PubSuper,
    /// Module-path restricted (`pub(in path)`)
    PubIn,
    /// Private (no visibility modifier)
    #[default]
    Private,
}

impl Visibility {
    /// Check if this visibility allows access from a different module
    pub fn allows_external_access(&self) -> bool {
        matches!(self, Visibility::Public | Visibility::PubCrate)
    }

    /// Check if access from another module would be "intrusive"
    ///
    /// Intrusive access means accessing something that isn't part of the public API.
    /// This indicates tight coupling to implementation details.
    pub fn is_intrusive_from(&self, same_crate: bool, same_module: bool) -> bool {
        if same_module {
            // Same module access is never intrusive
            return false;
        }

        match self {
            Visibility::Public => false,         // Public API, not intrusive
            Visibility::PubCrate => !same_crate, // Intrusive if from different crate
            Visibility::PubSuper | Visibility::PubIn => true, // Limited visibility, intrusive from outside
            Visibility::Private => true, // Private, always intrusive from outside
        }
    }

    /// Get a penalty multiplier for coupling strength based on visibility
    ///
    /// Higher penalty = more "intrusive" the access is.
    pub fn intrusive_penalty(&self) -> f64 {
        match self {
            Visibility::Public => 0.0,    // No penalty for public API
            Visibility::PubCrate => 0.25, // Small penalty for crate-internal
            Visibility::PubSuper => 0.5,  // Medium penalty
            Visibility::PubIn => 0.5,     // Medium penalty
            Visibility::Private => 1.0,   // Full penalty for private access
        }
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Public => write!(f, "pub"),
            Visibility::PubCrate => write!(f, "pub(crate)"),
            Visibility::PubSuper => write!(f, "pub(super)"),
            Visibility::PubIn => write!(f, "pub(in ...)"),
            Visibility::Private => write!(f, "private"),
        }
    }
}

/// Integration strength levels (how much knowledge is shared)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntegrationStrength {
    /// Strongest coupling - direct access to internals
    Intrusive,
    /// Strong coupling - depends on function signatures
    Functional,
    /// Medium coupling - depends on data models
    Model,
    /// Weakest coupling - depends only on contracts/traits
    Contract,
}

impl IntegrationStrength {
    /// Returns the numeric value (0.0 - 1.0, higher = stronger)
    pub fn value(&self) -> f64 {
        match self {
            IntegrationStrength::Intrusive => 1.0,
            IntegrationStrength::Functional => 0.75,
            IntegrationStrength::Model => 0.5,
            IntegrationStrength::Contract => 0.25,
        }
    }
}

/// Distance levels (how far apart components are)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distance {
    /// Same function/block
    SameFunction,
    /// Same module or structurally adjacent (ancestor/descendant, or siblings under one parent package)
    SameModule,
    /// Different module in same crate
    DifferentModule,
    /// Different crate
    DifferentCrate,
}

impl Distance {
    /// Returns the numeric value (0.0 - 1.0, higher = farther)
    pub fn value(&self) -> f64 {
        match self {
            Distance::SameFunction => 0.0,
            Distance::SameModule => 0.25,
            Distance::DifferentModule => 0.5,
            Distance::DifferentCrate => 1.0,
        }
    }
}

/// DDD subdomain type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subdomain {
    /// Core subdomain - competitive advantage, high volatility
    Core,
    /// Supporting subdomain - stable business logic, low volatility
    Supporting,
    /// Generic subdomain - solved problems, stable implementations
    Generic,
}

impl Subdomain {
    /// Map subdomain to expected volatility level
    pub fn expected_volatility(&self) -> Volatility {
        match self {
            Subdomain::Core => Volatility::High,
            Subdomain::Supporting => Volatility::Low,
            Subdomain::Generic => Volatility::Low,
        }
    }
}

impl fmt::Display for Subdomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Subdomain::Core => write!(f, "Core"),
            Subdomain::Supporting => write!(f, "Supporting"),
            Subdomain::Generic => write!(f, "Generic"),
        }
    }
}

/// Minimal configuration surface needed by project metrics.
pub trait MetricsConfig {
    /// Directory containing the loaded config file, if any.
    fn config_root(&self) -> Option<&Path>;
    /// Whether path-based volatility overrides exist.
    fn has_volatility_overrides(&self) -> bool;
    /// Whether subdomain classification exists.
    fn has_subdomain_config(&self) -> bool;
    /// Resolve a path to its configured subdomain, if any.
    fn get_subdomain(&self, path: &str) -> Option<Subdomain>;
    /// Resolve a path to its explicit volatility override, if any.
    fn get_volatility_override(&mut self, path: &str) -> Option<Volatility>;
}

// ===== Coupling Records =====
