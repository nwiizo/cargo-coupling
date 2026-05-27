use std::collections::HashMap;
use std::path::PathBuf;

use crate::analyzer::ItemDependency;
use crate::volatility::Volatility;

use super::dimensions::{Distance, IntegrationStrength, Subdomain, Visibility};

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// Name of the type
    pub name: String,
    /// Visibility of the type
    pub visibility: Visibility,
    /// Whether this is a trait (vs struct/enum)
    pub is_trait: bool,
    /// Whether this is a newtype pattern (tuple struct with single field)
    pub is_newtype: bool,
    /// Inner type for newtypes (e.g., "u64" for `struct UserId(u64)`)
    pub inner_type: Option<String>,
    /// Whether this type has #[derive(Serialize)] or #[derive(Deserialize)]
    pub has_serde_derive: bool,
    /// Number of public fields (for pub field exposure detection)
    pub public_field_count: usize,
    /// Total number of fields
    pub total_field_count: usize,
}

/// Information about a function definition in a module
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    /// Name of the function
    pub name: String,
    /// Visibility of the function
    pub visibility: Visibility,
    /// Number of parameters
    pub param_count: usize,
    /// Number of primitive type parameters (String, u32, bool, etc.)
    pub primitive_param_count: usize,
    /// Parameter types (for primitive obsession detection)
    pub param_types: Vec<String>,
}

/// Khononov's balance classification for couplings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BalanceClassification {
    /// High strength + Low distance = High cohesion (ideal)
    HighCohesion,
    /// Low strength + High distance = Loose coupling (ideal)
    LooseCoupling,
    /// High strength + High distance + Low volatility = Acceptable
    Acceptable,
    /// High strength + High distance + High volatility = Pain (needs refactoring)
    Pain,
    /// Low strength + Low distance = Local complexity (review needed)
    LocalComplexity,
}

impl BalanceClassification {
    /// Classify a coupling based on Khononov's formula
    pub fn classify(
        strength: IntegrationStrength,
        distance: Distance,
        volatility: Volatility,
    ) -> Self {
        let is_strong = strength.value() >= 0.5;
        let is_far = distance.value() >= 0.5;
        let is_volatile = volatility == Volatility::High;

        match (is_strong, is_far, is_volatile) {
            (true, false, _) => BalanceClassification::HighCohesion,
            (false, true, _) => BalanceClassification::LooseCoupling,
            (false, false, _) => BalanceClassification::LocalComplexity,
            (true, true, false) => BalanceClassification::Acceptable,
            (true, true, true) => BalanceClassification::Pain,
        }
    }

    /// Get Japanese description
    pub fn description_ja(&self) -> &'static str {
        match self {
            BalanceClassification::HighCohesion => "高凝集 (強+近)",
            BalanceClassification::LooseCoupling => "疎結合 (弱+遠)",
            BalanceClassification::Acceptable => "許容可能 (強+遠+安定)",
            BalanceClassification::Pain => "要改善 (強+遠+変動)",
            BalanceClassification::LocalComplexity => "局所複雑性 (弱+近)",
        }
    }

    /// Get English description
    pub fn description_en(&self) -> &'static str {
        match self {
            BalanceClassification::HighCohesion => "High Cohesion",
            BalanceClassification::LooseCoupling => "Loose Coupling",
            BalanceClassification::Acceptable => "Acceptable",
            BalanceClassification::Pain => "Needs Refactoring",
            BalanceClassification::LocalComplexity => "Local Complexity",
        }
    }

    /// Is this classification ideal?
    pub fn is_ideal(&self) -> bool {
        matches!(
            self,
            BalanceClassification::HighCohesion | BalanceClassification::LooseCoupling
        )
    }

    /// Does this need attention?
    pub fn needs_attention(&self) -> bool {
        matches!(
            self,
            BalanceClassification::Pain | BalanceClassification::LocalComplexity
        )
    }
}

/// Statistics for 3-dimensional coupling analysis
#[derive(Debug, Clone, Default)]
pub struct DimensionStats {
    /// Strength distribution
    pub strength_counts: StrengthCounts,
    /// Distance distribution
    pub distance_counts: DistanceCounts,
    /// Volatility distribution
    pub volatility_counts: VolatilityCounts,
    /// Balance classification counts
    pub balance_counts: BalanceCounts,
}

impl DimensionStats {
    /// Total number of couplings analyzed
    pub fn total(&self) -> usize {
        self.strength_counts.total()
    }

    /// Get percentage of each strength level
    pub fn strength_percentages(&self) -> (f64, f64, f64, f64) {
        let total = self.total() as f64;
        if total == 0.0 {
            return (0.0, 0.0, 0.0, 0.0);
        }
        (
            self.strength_counts.intrusive as f64 / total * 100.0,
            self.strength_counts.functional as f64 / total * 100.0,
            self.strength_counts.model as f64 / total * 100.0,
            self.strength_counts.contract as f64 / total * 100.0,
        )
    }

    /// Get percentage of each distance level
    pub fn distance_percentages(&self) -> (f64, f64, f64) {
        let total = self.total() as f64;
        if total == 0.0 {
            return (0.0, 0.0, 0.0);
        }
        (
            self.distance_counts.same_module as f64 / total * 100.0,
            self.distance_counts.different_module as f64 / total * 100.0,
            self.distance_counts.different_crate as f64 / total * 100.0,
        )
    }

    /// Get percentage of each volatility level
    pub fn volatility_percentages(&self) -> (f64, f64, f64) {
        let total = self.total() as f64;
        if total == 0.0 {
            return (0.0, 0.0, 0.0);
        }
        (
            self.volatility_counts.low as f64 / total * 100.0,
            self.volatility_counts.medium as f64 / total * 100.0,
            self.volatility_counts.high as f64 / total * 100.0,
        )
    }

    /// Count of ideal couplings (High Cohesion + Loose Coupling)
    pub fn ideal_count(&self) -> usize {
        self.balance_counts.high_cohesion + self.balance_counts.loose_coupling
    }

    /// Count of problematic couplings (Pain + Local Complexity)
    pub fn problematic_count(&self) -> usize {
        self.balance_counts.pain + self.balance_counts.local_complexity
    }

    /// Percentage of ideal couplings
    pub fn ideal_percentage(&self) -> f64 {
        let total = self.total() as f64;
        if total == 0.0 {
            return 0.0;
        }
        self.ideal_count() as f64 / total * 100.0
    }
}

/// Counts for each strength level
#[derive(Debug, Clone, Default)]
pub struct StrengthCounts {
    /// Number of intrusive-strength couplings.
    pub intrusive: usize,
    /// Number of functional-strength couplings.
    pub functional: usize,
    /// Number of model-strength couplings.
    pub model: usize,
    /// Number of contract-strength couplings.
    pub contract: usize,
}

impl StrengthCounts {
    /// Total count across all strength levels
    pub fn total(&self) -> usize {
        self.intrusive + self.functional + self.model + self.contract
    }
}

/// Counts for each distance level
#[derive(Debug, Clone, Default)]
pub struct DistanceCounts {
    /// Couplings within one module or function.
    pub same_module: usize,
    /// Couplings across modules in the same crate/workspace.
    pub different_module: usize,
    /// Couplings to external crates.
    pub different_crate: usize,
}

/// Counts for each volatility level
#[derive(Debug, Clone, Default)]
pub struct VolatilityCounts {
    /// Couplings whose target rarely changes.
    pub low: usize,
    /// Couplings whose target changes occasionally.
    pub medium: usize,
    /// Couplings whose target changes frequently.
    pub high: usize,
}

/// Counts for each balance classification
#[derive(Debug, Clone, Default)]
pub struct BalanceCounts {
    /// Strong and close couplings.
    pub high_cohesion: usize,
    /// Weak and distant couplings.
    pub loose_coupling: usize,
    /// Strong and distant couplings neutralized by low volatility.
    pub acceptable: usize,
    /// Strong, distant, and volatile couplings.
    pub pain: usize,
    /// Weak couplings kept unnecessarily close.
    pub local_complexity: usize,
}

/// Aggregated metrics for a module
#[derive(Debug, Clone, Default)]
pub struct ModuleMetrics {
    /// Module path
    pub path: PathBuf,
    /// Module name
    pub name: String,
    /// Number of trait implementations (contract coupling)
    pub trait_impl_count: usize,
    /// Number of inherent implementations (intrusive coupling)
    pub inherent_impl_count: usize,
    /// Number of function calls
    pub function_call_count: usize,
    /// Number of struct/enum usages
    pub type_usage_count: usize,
    /// External crate dependencies
    pub external_deps: Vec<String>,
    /// Internal module dependencies
    pub internal_deps: Vec<String>,
    /// Type definitions in this module with visibility info
    pub type_definitions: HashMap<String, TypeDefinition>,
    /// Function definitions in this module with visibility info
    pub function_definitions: HashMap<String, FunctionDefinition>,
    /// Item-level dependencies (function → function, function → type, etc.)
    pub item_dependencies: Vec<ItemDependency>,
    /// Whether this module is a test module (mod tests or #[cfg(test)])
    pub is_test_module: bool,
    /// Number of test functions (#[test])
    pub test_function_count: usize,
    /// DDD subdomain classification from config, if configured.
    pub subdomain: Option<Subdomain>,
}

impl ModuleMetrics {
    /// Create empty metrics for a module path/name pair.
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
            path,
            name,
            ..Default::default()
        }
    }

    /// Add a type definition to this module (simple version for backward compatibility)
    pub fn add_type_definition(&mut self, name: String, visibility: Visibility, is_trait: bool) {
        self.type_definitions.insert(
            name.clone(),
            TypeDefinition {
                name,
                visibility,
                is_trait,
                is_newtype: false,
                inner_type: None,
                has_serde_derive: false,
                public_field_count: 0,
                total_field_count: 0,
            },
        );
    }

    /// Add a type definition with full details
    #[allow(clippy::too_many_arguments)]
    pub fn add_type_definition_full(
        &mut self,
        name: String,
        visibility: Visibility,
        is_trait: bool,
        is_newtype: bool,
        inner_type: Option<String>,
        has_serde_derive: bool,
        public_field_count: usize,
        total_field_count: usize,
    ) {
        self.type_definitions.insert(
            name.clone(),
            TypeDefinition {
                name,
                visibility,
                is_trait,
                is_newtype,
                inner_type,
                has_serde_derive,
                public_field_count,
                total_field_count,
            },
        );
    }

    /// Add a function definition to this module (simple version for backward compatibility)
    pub fn add_function_definition(&mut self, name: String, visibility: Visibility) {
        self.function_definitions.insert(
            name.clone(),
            FunctionDefinition {
                name,
                visibility,
                param_count: 0,
                primitive_param_count: 0,
                param_types: Vec::new(),
            },
        );
    }

    /// Add a function definition with full details
    pub fn add_function_definition_full(
        &mut self,
        name: String,
        visibility: Visibility,
        param_count: usize,
        primitive_param_count: usize,
        param_types: Vec<String>,
    ) {
        self.function_definitions.insert(
            name.clone(),
            FunctionDefinition {
                name,
                visibility,
                param_count,
                primitive_param_count,
                param_types,
            },
        );
    }

    /// Get visibility of a type defined in this module
    pub fn get_type_visibility(&self, name: &str) -> Option<Visibility> {
        self.type_definitions.get(name).map(|t| t.visibility)
    }

    /// Count public types
    pub fn public_type_count(&self) -> usize {
        self.type_definitions
            .values()
            .filter(|t| t.visibility == Visibility::Public)
            .count()
    }

    /// Count non-public types
    pub fn private_type_count(&self) -> usize {
        self.type_definitions
            .values()
            .filter(|t| t.visibility != Visibility::Public)
            .count()
    }

    /// Calculate average integration strength
    pub fn average_strength(&self) -> f64 {
        let total = self.trait_impl_count + self.inherent_impl_count;
        if total == 0 {
            return 0.0;
        }

        let contract_weight = self.trait_impl_count as f64 * IntegrationStrength::Contract.value();
        let intrusive_weight =
            self.inherent_impl_count as f64 * IntegrationStrength::Intrusive.value();

        (contract_weight + intrusive_weight) / total as f64
    }

    /// Count newtypes in this module
    pub fn newtype_count(&self) -> usize {
        self.type_definitions
            .values()
            .filter(|t| t.is_newtype)
            .count()
    }

    /// Count types with serde derives
    pub fn serde_type_count(&self) -> usize {
        self.type_definitions
            .values()
            .filter(|t| t.has_serde_derive)
            .count()
    }

    /// Calculate newtype usage ratio (newtypes / total non-trait types)
    pub fn newtype_ratio(&self) -> f64 {
        let non_trait_types = self
            .type_definitions
            .values()
            .filter(|t| !t.is_trait)
            .count();
        if non_trait_types == 0 {
            return 0.0;
        }
        self.newtype_count() as f64 / non_trait_types as f64
    }

    /// Count types with public fields
    pub fn types_with_public_fields(&self) -> usize {
        self.type_definitions
            .values()
            .filter(|t| t.public_field_count > 0)
            .count()
    }

    /// Total function count
    pub fn function_count(&self) -> usize {
        self.function_definitions.len()
    }

    /// Count functions with high primitive parameter ratio
    /// (potential Primitive Obsession)
    pub fn functions_with_primitive_obsession(&self) -> Vec<&FunctionDefinition> {
        self.function_definitions
            .values()
            .filter(|f| {
                f.param_count >= 3 && f.primitive_param_count as f64 / f.param_count as f64 >= 0.6
            })
            .collect()
    }

    /// Check if this module is a potential "God Module"
    /// (too many functions, types, or implementations)
    pub fn is_god_module(&self, max_functions: usize, max_types: usize, max_impls: usize) -> bool {
        self.function_count() > max_functions
            || self.type_definitions.len() > max_types
            || (self.trait_impl_count + self.inherent_impl_count) > max_impls
    }
}

// ===== Project Aggregates =====
