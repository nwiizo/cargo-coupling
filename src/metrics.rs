//! Coupling metrics data structures
//!
//! This module defines the core data structures for measuring coupling
//! based on Vlad Khononov's "Balancing Coupling in Software Design".

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::PathBuf;

use crate::connascence::ConnascenceStats;

/// Visibility level of a Rust item
///
/// This is used to determine if access to an item from another module
/// constitutes "Intrusive" coupling (access to private/internal details).
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
    /// Same module/file
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

/// Volatility levels (how often a component changes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Volatility {
    /// Rarely changes (0-2 times)
    Low,
    /// Sometimes changes (3-10 times)
    Medium,
    /// Frequently changes (11+ times)
    High,
}

impl Volatility {
    /// Returns the numeric value (0.0 - 1.0, higher = more volatile)
    pub fn value(&self) -> f64 {
        match self {
            Volatility::Low => 0.0,
            Volatility::Medium => 0.5,
            Volatility::High => 1.0,
        }
    }

    /// Classify from change count
    pub fn from_count(count: usize) -> Self {
        match count {
            0..=2 => Volatility::Low,
            3..=10 => Volatility::Medium,
            _ => Volatility::High,
        }
    }
}

/// Location information for a coupling
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

/// Information about a type definition in a module
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// Name of the type
    pub name: String,
    /// Visibility of the type
    pub visibility: Visibility,
    /// Whether this is a trait (vs struct/enum)
    pub is_trait: bool,
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
}

impl ModuleMetrics {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
            path,
            name,
            ..Default::default()
        }
    }

    /// Add a type definition to this module
    pub fn add_type_definition(&mut self, name: String, visibility: Visibility, is_trait: bool) {
        self.type_definitions.insert(
            name.clone(),
            TypeDefinition {
                name,
                visibility,
                is_trait,
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
}

/// Project-wide analysis results
#[derive(Debug, Default)]
pub struct ProjectMetrics {
    /// All module metrics
    pub modules: HashMap<String, ModuleMetrics>,
    /// All detected couplings
    pub couplings: Vec<CouplingMetrics>,
    /// File change counts (for volatility)
    pub file_changes: HashMap<String, usize>,
    /// Total files analyzed
    pub total_files: usize,
    /// Workspace name (if available from cargo metadata)
    pub workspace_name: Option<String>,
    /// Workspace member crate names
    pub workspace_members: Vec<String>,
    /// Crate-level dependencies (crate name -> list of dependencies)
    pub crate_dependencies: HashMap<String, Vec<String>>,
    /// Global type registry: type name -> (module name, visibility)
    pub type_registry: HashMap<String, (String, Visibility)>,
    /// Connascence analysis statistics
    pub connascence_stats: ConnascenceStats,
}

impl ProjectMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add module metrics
    pub fn add_module(&mut self, metrics: ModuleMetrics) {
        self.modules.insert(metrics.name.clone(), metrics);
    }

    /// Add coupling
    pub fn add_coupling(&mut self, coupling: CouplingMetrics) {
        self.couplings.push(coupling);
    }

    /// Register a type definition in the global registry
    pub fn register_type(
        &mut self,
        type_name: String,
        module_name: String,
        visibility: Visibility,
    ) {
        self.type_registry
            .insert(type_name, (module_name, visibility));
    }

    /// Look up visibility of a type by name
    pub fn get_type_visibility(&self, type_name: &str) -> Option<Visibility> {
        self.type_registry.get(type_name).map(|(_, vis)| *vis)
    }

    /// Look up the module where a type is defined
    pub fn get_type_module(&self, type_name: &str) -> Option<&str> {
        self.type_registry
            .get(type_name)
            .map(|(module, _)| module.as_str())
    }

    /// Update visibility information for existing couplings
    ///
    /// This should be called after all modules have been analyzed
    /// to populate the target_visibility field of couplings.
    pub fn update_coupling_visibility(&mut self) {
        // First collect all the visibility lookups
        let visibility_updates: Vec<(usize, Visibility)> = self
            .couplings
            .iter()
            .enumerate()
            .filter_map(|(idx, coupling)| {
                let target_type = coupling
                    .target
                    .split("::")
                    .last()
                    .unwrap_or(&coupling.target);
                self.type_registry
                    .get(target_type)
                    .map(|(_, vis)| (idx, *vis))
            })
            .collect();

        // Then apply the updates
        for (idx, visibility) in visibility_updates {
            self.couplings[idx].target_visibility = visibility;
        }
    }

    /// Get total module count
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    /// Get total coupling count
    pub fn coupling_count(&self) -> usize {
        self.couplings.len()
    }

    /// Calculate average strength across all couplings
    pub fn average_strength(&self) -> Option<f64> {
        if self.couplings.is_empty() {
            return None;
        }
        let sum: f64 = self.couplings.iter().map(|c| c.strength_value()).sum();
        Some(sum / self.couplings.len() as f64)
    }

    /// Calculate average distance across all couplings
    pub fn average_distance(&self) -> Option<f64> {
        if self.couplings.is_empty() {
            return None;
        }
        let sum: f64 = self.couplings.iter().map(|c| c.distance_value()).sum();
        Some(sum / self.couplings.len() as f64)
    }

    /// Update volatility for all couplings based on file changes
    ///
    /// This should be called after git history analysis to update
    /// the volatility of each coupling based on how often the target
    /// module/file has changed.
    pub fn update_volatility_from_git(&mut self) {
        if self.file_changes.is_empty() {
            return;
        }

        for coupling in &mut self.couplings {
            // Try to find the target file in file_changes
            // The target is like "crate::module" or "crate::submodule::item"
            // We need to match this against file paths like "src/module.rs"

            let target_module = coupling
                .target
                .split("::")
                .last()
                .unwrap_or(&coupling.target);

            // Find the best matching file
            let mut max_changes = 0usize;
            for (file_path, &changes) in &self.file_changes {
                // Check if file matches the target module
                let file_name = file_path
                    .rsplit('/')
                    .next()
                    .unwrap_or(file_path)
                    .trim_end_matches(".rs");

                if file_name == target_module || file_path.contains(target_module) {
                    max_changes = max_changes.max(changes);
                }
            }

            coupling.volatility = Volatility::from_count(max_changes);
        }
    }

    /// Build a dependency graph from couplings
    fn build_dependency_graph(&self) -> HashMap<String, HashSet<String>> {
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

        for coupling in &self.couplings {
            // Only consider internal couplings (not external crates)
            if coupling.distance == Distance::DifferentCrate {
                continue;
            }

            // Extract module names (remove crate prefix for cleaner cycles)
            let source = coupling.source.clone();
            let target = coupling.target.clone();

            graph.entry(source).or_default().insert(target);
        }

        graph
    }

    /// Detect circular dependencies in the project
    ///
    /// Returns a list of cycles, where each cycle is a list of module names
    /// forming the circular dependency chain.
    pub fn detect_circular_dependencies(&self) -> Vec<Vec<String>> {
        let graph = self.build_dependency_graph();
        let mut cycles: Vec<Vec<String>> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();

        for node in graph.keys() {
            if !visited.contains(node) {
                let mut path = Vec::new();
                self.dfs_find_cycles(
                    node,
                    &graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        // Deduplicate cycles (same cycle can be detected from different starting points)
        let mut unique_cycles: Vec<Vec<String>> = Vec::new();
        for cycle in cycles {
            let normalized = Self::normalize_cycle(&cycle);
            if !unique_cycles
                .iter()
                .any(|c| Self::normalize_cycle(c) == normalized)
            {
                unique_cycles.push(cycle);
            }
        }

        unique_cycles
    }

    /// DFS helper for cycle detection
    fn dfs_find_cycles(
        &self,
        node: &str,
        graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_find_cycles(neighbor, graph, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle - extract the cycle from path
                    if let Some(start_idx) = path.iter().position(|n| n == neighbor) {
                        let cycle: Vec<String> = path[start_idx..].to_vec();
                        if cycle.len() >= 2 {
                            cycles.push(cycle);
                        }
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    /// Normalize a cycle for deduplication
    /// Rotates the cycle so the lexicographically smallest element is first
    fn normalize_cycle(cycle: &[String]) -> Vec<String> {
        if cycle.is_empty() {
            return Vec::new();
        }

        // Find the position of the minimum element
        let min_pos = cycle
            .iter()
            .enumerate()
            .min_by_key(|(_, s)| s.as_str())
            .map(|(i, _)| i)
            .unwrap_or(0);

        // Rotate the cycle
        let mut normalized: Vec<String> = cycle[min_pos..].to_vec();
        normalized.extend_from_slice(&cycle[..min_pos]);
        normalized
    }

    /// Get circular dependency summary
    pub fn circular_dependency_summary(&self) -> CircularDependencySummary {
        let cycles = self.detect_circular_dependencies();
        let affected_modules: HashSet<String> = cycles.iter().flatten().cloned().collect();

        CircularDependencySummary {
            total_cycles: cycles.len(),
            affected_modules: affected_modules.len(),
            cycles,
        }
    }
}

/// Summary of circular dependencies
#[derive(Debug, Clone)]
pub struct CircularDependencySummary {
    /// Total number of circular dependency cycles
    pub total_cycles: usize,
    /// Number of modules involved in cycles
    pub affected_modules: usize,
    /// The actual cycles (list of module names)
    pub cycles: Vec<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
