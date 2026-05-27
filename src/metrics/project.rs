use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

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

        // Debug: print file changes for troubleshooting
        #[cfg(test)]
        {
            eprintln!("DEBUG: file_changes = {:?}", self.file_changes);
        }

        let module_paths: Vec<(String, PathBuf)> = self
            .modules
            .iter()
            .map(|(name, module)| (name.clone(), module.path.clone()))
            .collect();

        for coupling in &mut self.couplings {
            if let Some(module_path) = target_module_path(&coupling.target, &module_paths) {
                coupling.volatility = Volatility::from_count(change_count_for_module_path(
                    module_path,
                    &self.file_changes,
                ));
                continue;
            }

            // Try to find the target file in file_changes
            // The target is like "crate::module" or "crate::module::Type"
            // We need to match this against file paths like "src/module.rs"
            //
            // Special cases in Rust module system:
            // - crate root "crate::crate_name" or "crate_name::crate_name" -> lib.rs
            // - binary entry point -> main.rs
            // - glob imports "crate::*" -> don't match specific files

            // Extract all path components from target
            let target_segments: Vec<&str> = coupling.target.split("::").collect();

            // Find the best matching file
            let mut max_target_changes = 0usize;
            for (file_path, &changes) in &self.file_changes {
                // Get file name without .rs extension (e.g., "balance" from "src/balance.rs")
                let file_name = file_path
                    .rsplit('/')
                    .next()
                    .unwrap_or(file_path)
                    .trim_end_matches(".rs");

                // Check if any target path component matches the file name
                let target_matches_file = target_segments.iter().any(|part| {
                    let part_lower = part.to_lowercase();
                    let file_lower = file_name.to_lowercase();

                    // Direct match: "balance" == "balance"
                    if part_lower == file_lower {
                        return true;
                    }

                    // Handle crate root: if the part matches the crate name and file is lib.rs
                    // e.g., "cargo_coupling" matches "lib" (lib.rs is the crate root)
                    if file_lower == "lib" && !part.is_empty() && *part != "*" {
                        // This could be the crate root reference
                        // We also match if the part is the crate name (same as first path component)
                        if target_segments.len() >= 2 && target_segments[1] == *part {
                            return true;
                        }
                    }

                    // Handle underscore vs hyphen in crate names
                    // e.g., "cargo-coupling" might appear as "cargo_coupling" in code
                    let part_normalized = part_lower.replace('-', "_");
                    let file_normalized = file_lower.replace('-', "_");
                    if part_normalized == file_normalized {
                        return true;
                    }

                    // Path contains match: "web" matches "src/web/graph.rs"
                    if file_path.to_lowercase().contains(&part_lower) {
                        return true;
                    }

                    false
                });

                if target_matches_file {
                    max_target_changes = max_target_changes.max(changes);
                }
            }

            coupling.volatility = Volatility::from_count(max_target_changes);
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

        let mut module_paths = HashMap::new();
        let has_subdomain_config = config.has_subdomain_config();
        for (name, module) in &mut self.modules {
            let relative_path = path_for_config_matching(&module.path, config);
            if has_subdomain_config {
                module.subdomain = config.get_subdomain(&relative_path);
            }

            insert_module_path_aliases(&mut module_paths, name, module, &relative_path);
        }

        let mut override_count = 0;
        for coupling in &mut self.couplings {
            let target_short = coupling
                .target
                .rsplit("::")
                .next()
                .unwrap_or(&coupling.target);
            let lookup = module_paths
                .get(&coupling.target)
                .or_else(|| module_paths.get(target_short))
                .or_else(|| {
                    coupling
                        .target
                        .rsplit("::")
                        .find_map(|segment| module_paths.get(segment))
                })
                .map(String::as_str)
                .unwrap_or(coupling.target.as_str());

            if let Some(override_vol) = config.get_volatility_override(lookup) {
                coupling.volatility = override_vol;
                override_count += 1;
            }
        }

        override_count
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
        self.modules
            .iter()
            .flat_map(|(module_name, m)| {
                m.type_definitions
                    .values()
                    .filter(|t| t.has_serde_derive)
                    .map(move |t| (module_name.as_str(), t))
            })
            .collect()
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
        self.modules
            .iter()
            .flat_map(|(module_name, m)| {
                m.type_definitions
                    .values()
                    .filter(|t| t.public_field_count > 0 && !t.is_trait)
                    .map(move |t| (module_name.as_str(), t))
            })
            .collect()
    }
}

fn target_module_path<'a>(target: &str, module_paths: &'a [(String, PathBuf)]) -> Option<&'a Path> {
    let target = target.trim_start_matches("crate::");
    let target_without_crate = target.split_once("::").and_then(|(_, rest)| {
        module_paths
            .iter()
            .any(|(module, _)| rest == module || rest.starts_with(&format!("{module}::")))
            .then_some(rest)
    });
    let target = target_without_crate.unwrap_or(target);

    module_paths
        .iter()
        .filter(|(module, _)| target == module || target.starts_with(&format!("{module}::")))
        .max_by_key(|(module, _)| module.len())
        .map(|(_, path)| path.as_path())
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

fn insert_module_path_aliases(
    module_paths: &mut HashMap<String, String>,
    key_name: &str,
    module: &ModuleMetrics,
    relative_path: &str,
) {
    module_paths.insert(key_name.to_string(), relative_path.to_string());
    module_paths.insert(module.name.clone(), relative_path.to_string());

    if let Some(short_name) = key_name.rsplit("::").next() {
        module_paths.insert(short_name.to_string(), relative_path.to_string());
    }

    if let Some(short_name) = module.name.rsplit("::").next() {
        module_paths.insert(short_name.to_string(), relative_path.to_string());
    }

    if let Some(file_stem) = module.path.file_stem().and_then(|stem| stem.to_str()) {
        module_paths.insert(file_stem.to_string(), relative_path.to_string());
    }
}

fn path_for_config_matching(file_path: &Path, config: &impl MetricsConfig) -> String {
    let normalized_file = normalize_path_for_matching(file_path);
    let path = config
        .config_root()
        .map(normalize_path_for_matching)
        .and_then(|base| {
            normalized_file
                .strip_prefix(base)
                .ok()
                .map(Path::to_path_buf)
        })
        .unwrap_or(normalized_file);

    path.to_string_lossy().replace('\\', "/")
}

fn normalize_path_for_matching(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };

    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
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
