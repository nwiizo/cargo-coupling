use super::target::ModuleIndex;
use crate::paths::config_matching_path;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::path::Path;

use crate::volatility::{TemporalCoupling, Volatility};

use super::coupling::CouplingMetrics;
use super::dimensions::{Distance, IntegrationStrength, MetricsConfig, Visibility};
use super::module::{
    BalanceClassification, DimensionStats, FunctionDefinition, ModuleMetrics, TypeDefinition,
};

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
    /// Source files that failed to parse or analyze and were skipped.
    pub parse_failures: usize,
    /// Workspace members with no discoverable source files.
    pub skipped_crates: Vec<String>,
    /// Module references skipped because they cross analyzed package/workspace boundaries.
    pub boundary_skipped_files: usize,
    /// Config patterns that matched no paths in the analysis candidate set.
    pub dead_config_patterns: Vec<String>,
    /// Workspace name (if available from cargo metadata)
    pub workspace_name: Option<String>,
    /// Workspace member crate names
    pub workspace_members: Vec<String>,
    /// Crate-level dependencies (crate name -> list of dependencies)
    pub crate_dependencies: HashMap<String, Vec<String>>,
    /// Global type registry: type name -> (module name, visibility)
    pub type_registry: HashMap<String, (String, Visibility)>,
    /// Temporal coupling data (files that co-change frequently)
    pub temporal_couplings: Vec<TemporalCoupling>,
}

impl ProjectMetrics {
    /// Create an empty project metrics accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evidence available to compatibility entrypoints whose caller supplies no run status.
    pub(crate) fn has_git_observations(&self) -> bool {
        !self.file_changes.is_empty() || !self.temporal_couplings.is_empty()
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
        match self.type_registry.get(&type_name) {
            Some((existing_module, existing_visibility))
                if should_keep_existing_type_registration(
                    existing_module,
                    *existing_visibility,
                    &module_name,
                    visibility,
                ) => {}
            _ => {
                self.type_registry
                    .insert(type_name, (module_name, visibility));
            }
        }
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

    /// Get internal coupling count (excludes external crate dependencies)
    pub fn internal_coupling_count(&self) -> usize {
        self.couplings
            .iter()
            .filter(|c| c.distance != Distance::DifferentCrate)
            .count()
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

        let modules = ModuleIndex::new(&self.modules, &self.workspace_members);
        for coupling in &mut self.couplings {
            if let Some(module) = modules.provider(coupling) {
                coupling.volatility = Volatility::from_count(change_count_for_module_path(
                    &module.path,
                    &self.file_changes,
                ));
            } else if self.modules.is_empty() && coupling.distance != Distance::DifferentCrate {
                // Library clients can supply edges and Git counts without a module inventory.
                // Use only an exact file stem and decline ambiguous filenames.
                let mut matches = self.file_changes.iter().filter(|(path, _)| {
                    let stem = Path::new(path).file_stem().and_then(|name| name.to_str());
                    coupling.target.split("::").any(|part| Some(part) == stem)
                });
                if let Some((_, count)) = matches.next()
                    && matches.next().is_none()
                {
                    coupling.volatility = Volatility::from_count(*count);
                }
            }
        }
    }

    /// Apply config-derived subdomain classifications and volatility overrides.
    ///
    /// Config patterns are path-based while couplings are module-name-based, so
    /// this resolves coupling targets through known module file paths before
    /// querying the config.
    pub fn apply_config_volatility_overrides<C: MetricsConfig>(&mut self, config: &mut C) -> usize {
        if !config.has_volatility_overrides() && !config.has_subdomain_config() {
            return 0;
        }

        if config.has_subdomain_config() {
            for module in self.modules.values_mut() {
                module.subdomain =
                    config.get_subdomain(&config_matching_path(&module.path, config.config_root()));
            }
        }
        let modules = ModuleIndex::new(&self.modules, &self.workspace_members);
        let paths: HashMap<_, _> = self
            .modules
            .values()
            .map(|module| {
                (
                    &module.path,
                    config_matching_path(&module.path, config.config_root()),
                )
            })
            .collect();
        let mut override_count = 0;
        for coupling in &mut self.couplings {
            let lookup = modules
                .provider(coupling)
                .map(|module| paths[&module.path].as_str())
                .unwrap_or(&coupling.target);
            if let Some(override_vol) = config.get_volatility_override(lookup) {
                coupling.volatility = override_vol;
                override_count += 1;
            }
        }

        override_count
    }

    /// Build a dependency graph from couplings
    fn build_dependency_graph(&self) -> BTreeMap<String, BTreeSet<String>> {
        let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

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
    /// Returns a deterministic set of cycle witnesses. Every internal edge that
    /// belongs to a multi-module cycle is covered by at least one returned
    /// chain, without exhaustively enumerating all simple cycles (which can be
    /// exponential).
    pub fn detect_circular_dependencies(&self) -> Vec<Vec<String>> {
        let graph = self.build_dependency_graph();
        let mut cycles = BTreeSet::new();

        // Only non-trivial strongly connected components can contain the
        // multi-module cycles reported here. This keeps acyclic projects at
        // linear graph-traversal cost instead of running a BFS from every node.
        for component in Self::strongly_connected_components(&graph)
            .into_iter()
            .filter(|component| component.len() > 1)
        {
            let mut incoming: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            for source in &component {
                let Some(targets) = graph.get(source) else {
                    continue;
                };
                for target in targets {
                    if component.contains(target) {
                        incoming
                            .entry(target.clone())
                            .or_default()
                            .insert(source.clone());
                    }
                }
            }

            // One BFS per distinct target supplies return paths for all incoming
            // edges inside this cyclic component.
            for (target, sources) in incoming {
                let parents = Self::shortest_path_tree(&graph, &target, &component);
                for source in sources {
                    // Same-module references are not architectural cycles.
                    if source == target {
                        continue;
                    }

                    // source -> target is cyclic because both nodes belong to the
                    // same non-trivial strongly connected component.
                    if let Some(mut return_path) = Self::path_from_tree(&parents, &target, &source)
                    {
                        return_path.pop(); // source closes the cycle; do not repeat it.
                        let mut cycle = Vec::with_capacity(return_path.len() + 1);
                        cycle.push(source);
                        cycle.extend(return_path);
                        cycles.insert(Self::normalize_cycle(&cycle));
                    }
                }
            }
        }

        cycles.into_iter().collect()
    }

    /// Partition the graph deterministically using iterative Kosaraju traversal.
    fn strongly_connected_components(
        graph: &BTreeMap<String, BTreeSet<String>>,
    ) -> Vec<BTreeSet<String>> {
        let mut nodes = BTreeSet::new();
        for (source, targets) in graph {
            nodes.insert(source.clone());
            nodes.extend(targets.iter().cloned());
        }

        let mut visited = BTreeSet::new();
        let mut finish_order = Vec::with_capacity(nodes.len());
        for start in &nodes {
            if visited.contains(start) {
                continue;
            }

            let mut stack = vec![(start.clone(), false)];
            while let Some((node, expanded)) = stack.pop() {
                if expanded {
                    finish_order.push(node);
                    continue;
                }
                if !visited.insert(node.clone()) {
                    continue;
                }

                stack.push((node.clone(), true));
                if let Some(neighbors) = graph.get(&node) {
                    for neighbor in neighbors.iter().rev() {
                        if !visited.contains(neighbor) {
                            stack.push((neighbor.clone(), false));
                        }
                    }
                }
            }
        }

        let mut reverse: BTreeMap<String, BTreeSet<String>> = nodes
            .iter()
            .cloned()
            .map(|node| (node, BTreeSet::new()))
            .collect();
        for (source, targets) in graph {
            for target in targets {
                reverse
                    .entry(target.clone())
                    .or_default()
                    .insert(source.clone());
            }
        }

        let mut assigned = BTreeSet::new();
        let mut components = Vec::new();
        for start in finish_order.into_iter().rev() {
            if !assigned.insert(start.clone()) {
                continue;
            }

            let mut component = BTreeSet::new();
            let mut stack = vec![start];
            while let Some(node) = stack.pop() {
                component.insert(node.clone());
                if let Some(predecessors) = reverse.get(&node) {
                    for predecessor in predecessors.iter().rev() {
                        if assigned.insert(predecessor.clone()) {
                            stack.push(predecessor.clone());
                        }
                    }
                }
            }
            components.push(component);
        }

        components
    }

    /// Build a deterministic shortest-path tree from one module.
    fn shortest_path_tree(
        graph: &BTreeMap<String, BTreeSet<String>>,
        start: &str,
        component: &BTreeSet<String>,
    ) -> BTreeMap<String, String> {
        let mut queue = VecDeque::from([start.to_string()]);
        let mut visited = BTreeSet::from([start.to_string()]);
        let mut parents: BTreeMap<String, String> = BTreeMap::new();

        while let Some(node) = queue.pop_front() {
            let Some(neighbors) = graph.get(&node) else {
                continue;
            };
            for neighbor in neighbors {
                if !component.contains(neighbor) {
                    continue;
                }
                if !visited.insert(neighbor.clone()) {
                    continue;
                }
                parents.insert(neighbor.clone(), node.clone());
                queue.push_back(neighbor.clone());
            }
        }

        parents
    }

    /// Reconstruct a path from a deterministic shortest-path tree.
    fn path_from_tree(
        parents: &BTreeMap<String, String>,
        start: &str,
        goal: &str,
    ) -> Option<Vec<String>> {
        let mut path = vec![goal.to_string()];
        let mut cursor = goal.to_string();
        while cursor != start {
            cursor = parents.get(&cursor)?.clone();
            path.push(cursor.clone());
        }
        path.reverse();
        Some(path)
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

    /// Calculate 3-dimensional coupling statistics
    ///
    /// Computes distribution of couplings across Strength, Distance,
    /// Volatility, and Balance Classification dimensions.
    pub fn calculate_dimension_stats(&self) -> DimensionStats {
        let mut stats = DimensionStats::default();

        for coupling in &self.couplings {
            // Count strength distribution
            match coupling.strength {
                IntegrationStrength::Intrusive => stats.strength_counts.intrusive += 1,
                IntegrationStrength::Functional => stats.strength_counts.functional += 1,
                IntegrationStrength::Model => stats.strength_counts.model += 1,
                IntegrationStrength::Contract => stats.strength_counts.contract += 1,
            }

            // Count distance distribution
            match coupling.distance {
                Distance::SameFunction | Distance::SameModule => {
                    stats.distance_counts.same_module += 1
                }
                Distance::DifferentModule => stats.distance_counts.different_module += 1,
                Distance::DifferentCrate => stats.distance_counts.different_crate += 1,
            }

            // Count volatility distribution
            match coupling.volatility {
                Volatility::Low => stats.volatility_counts.low += 1,
                Volatility::Medium => stats.volatility_counts.medium += 1,
                Volatility::High => stats.volatility_counts.high += 1,
            }

            // Classify and count balance
            let classification = BalanceClassification::classify(
                coupling.strength,
                coupling.distance,
                coupling.volatility,
            );
            match classification {
                BalanceClassification::HighCohesion => stats.balance_counts.high_cohesion += 1,
                BalanceClassification::LooseCoupling => stats.balance_counts.loose_coupling += 1,
                BalanceClassification::Acceptable => stats.balance_counts.acceptable += 1,
                BalanceClassification::Pain => stats.balance_counts.pain += 1,
                BalanceClassification::LocalComplexity => {
                    stats.balance_counts.local_complexity += 1
                }
            }
        }

        stats
    }

    /// Get total newtype count across all modules
    pub fn total_newtype_count(&self) -> usize {
        self.modules.values().map(|m| m.newtype_count()).sum()
    }

    /// Get total type count across all modules (excluding traits)
    pub fn total_type_count(&self) -> usize {
        self.modules
            .values()
            .flat_map(|m| m.type_definitions.values())
            .filter(|t| !t.is_trait)
            .count()
    }

    /// Calculate project-wide newtype usage ratio
    pub fn newtype_ratio(&self) -> f64 {
        let total = self.total_type_count();
        if total == 0 {
            return 0.0;
        }
        self.total_newtype_count() as f64 / total as f64
    }

    /// Get types with serde derives (potential DTO exposure)
    pub fn serde_types(&self) -> Vec<(&str, &TypeDefinition)> {
        self.named_types()
            .filter(|(_, ty)| ty.has_serde_derive)
            .collect()
    }

    fn named_types(&self) -> impl Iterator<Item = (&str, &TypeDefinition)> {
        self.modules.iter().flat_map(|(module, data)| {
            data.type_definitions
                .values()
                .map(move |ty| (module.as_str(), ty))
        })
    }

    /// Identify potential God Modules
    pub fn god_modules(
        &self,
        max_functions: usize,
        max_types: usize,
        max_impls: usize,
    ) -> Vec<&str> {
        self.modules
            .iter()
            .filter(|(_, m)| m.is_god_module(max_functions, max_types, max_impls))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Get all functions with potential Primitive Obsession
    pub fn functions_with_primitive_obsession(&self) -> Vec<(&str, &FunctionDefinition)> {
        self.modules
            .iter()
            .flat_map(|(module_name, m)| {
                m.functions_with_primitive_obsession()
                    .into_iter()
                    .map(move |f| (module_name.as_str(), f))
            })
            .collect()
    }

    /// Get types with exposed public fields
    pub fn types_with_public_fields(&self) -> Vec<(&str, &TypeDefinition)> {
        self.named_types()
            .filter(|(_, ty)| ty.public_field_count > 0 && !ty.is_trait)
            .collect()
    }
}

fn should_keep_existing_type_registration(
    existing_module: &str,
    existing_visibility: Visibility,
    candidate_module: &str,
    candidate_visibility: Visibility,
) -> bool {
    match (
        existing_visibility == Visibility::Public,
        candidate_visibility == Visibility::Public,
    ) {
        (true, false) => true,
        (false, true) => false,
        _ => existing_module <= candidate_module,
    }
}

fn change_count_for_module_path(
    module_path: &Path,
    file_changes: &HashMap<String, usize>,
) -> usize {
    let module_path = module_path.to_string_lossy().replace('\\', "/");
    file_changes
        .iter()
        .filter(|(file_path, _)| module_file_paths_match(&module_path, file_path))
        .map(|(_, changes)| *changes)
        .max()
        .unwrap_or(0)
}

fn module_file_paths_match(module_path: &str, git_path: &str) -> bool {
    let git_path = git_path.replace('\\', "/");
    module_path == git_path
        || module_path.ends_with(&format!("/{git_path}"))
        || git_path.ends_with(&format!("/{module_path}"))
}

/// Summary of circular dependencies.
///
/// The paths are deterministic witnesses that cover every multi-module cyclic
/// internal edge. They are not an exhaustive enumeration of all simple cycles,
/// whose count can be exponential.
#[derive(Debug, Clone)]
pub struct CircularDependencySummary {
    /// Number of representative cycle witness paths.
    ///
    /// The field name is retained for API compatibility.
    pub total_cycles: usize,
    /// Number of modules involved in cycles
    pub affected_modules: usize,
    /// Representative cycle witness paths (lists of module names)
    pub cycles: Vec<Vec<String>>,
}
