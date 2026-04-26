//! Workspace analysis using cargo metadata
//!
//! This module uses `cargo metadata` to understand the project structure,
//! including workspace members, dependencies, and module organization.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use cargo_metadata::{Metadata, MetadataCommand, PackageId};
use thiserror::Error;

/// Errors that can occur during workspace analysis
#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("Failed to run cargo metadata: {0}")]
    MetadataError(#[from] cargo_metadata::Error),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Invalid manifest path: {0}")]
    InvalidManifest(String),
}

/// Information about a crate in the workspace
#[derive(Debug, Clone)]
pub struct CrateInfo {
    /// Crate name
    pub name: String,
    /// Package ID for dependency resolution
    pub id: PackageId,
    /// Path to the crate's source directory
    pub src_path: PathBuf,
    /// Path to Cargo.toml
    pub manifest_path: PathBuf,
    /// Direct dependencies (crate names)
    pub dependencies: Vec<String>,
    /// Dev dependencies
    pub dev_dependencies: Vec<String>,
    /// Is this a workspace member?
    pub is_workspace_member: bool,
}

/// Information about the entire workspace
#[derive(Debug)]
pub struct WorkspaceInfo {
    /// Root directory of the workspace
    pub root: PathBuf,
    /// All crates in the workspace
    pub crates: HashMap<String, CrateInfo>,
    /// Workspace members (crate names)
    pub members: Vec<String>,
    /// Dependency graph: crate name -> dependencies
    pub dependency_graph: HashMap<String, HashSet<String>>,
    /// Reverse dependency graph: crate name -> dependents
    pub reverse_deps: HashMap<String, HashSet<String>>,
}

impl WorkspaceInfo {
    /// Analyze a workspace from a path
    pub fn from_path(path: &Path) -> Result<Self, WorkspaceError> {
        // Find Cargo.toml
        let manifest_path = find_cargo_toml(path)?;

        // Run cargo metadata
        let metadata = MetadataCommand::new()
            .manifest_path(&manifest_path)
            .exec()?;

        Self::from_metadata(metadata)
    }

    /// Create workspace info from cargo metadata
    pub fn from_metadata(metadata: Metadata) -> Result<Self, WorkspaceError> {
        let root = metadata.workspace_root.as_std_path().to_path_buf();

        let mut crates = HashMap::new();
        let mut members = Vec::new();
        let mut dependency_graph: HashMap<String, HashSet<String>> = HashMap::new();
        let mut reverse_deps: HashMap<String, HashSet<String>> = HashMap::new();

        // Collect workspace members
        let workspace_member_ids: HashSet<_> = metadata.workspace_members.iter().collect();

        // Process all packages
        for package in &metadata.packages {
            let is_workspace_member = workspace_member_ids.contains(&package.id);
            let package_name = package.name.to_string();

            if is_workspace_member {
                members.push(package_name.clone());
            }

            // Get source directory
            let src_path = package
                .manifest_path
                .parent()
                .map(|p| p.as_std_path().join("src"))
                .unwrap_or_default();

            // Collect dependencies
            let mut deps = Vec::new();
            let mut dev_deps = Vec::new();

            for dep in &package.dependencies {
                if dep.kind == cargo_metadata::DependencyKind::Development {
                    dev_deps.push(dep.name.clone());
                } else {
                    deps.push(dep.name.clone());
                }

                // Build dependency graph
                dependency_graph
                    .entry(package_name.clone())
                    .or_default()
                    .insert(dep.name.clone());

                // Build reverse dependency graph
                reverse_deps
                    .entry(dep.name.clone())
                    .or_default()
                    .insert(package_name.clone());
            }

            let crate_info = CrateInfo {
                name: package_name.clone(),
                id: package.id.clone(),
                src_path,
                manifest_path: package.manifest_path.as_std_path().to_path_buf(),
                dependencies: deps,
                dev_dependencies: dev_deps,
                is_workspace_member,
            };

            crates.insert(package_name, crate_info);
        }

        Ok(Self {
            root,
            crates,
            members,
            dependency_graph,
            reverse_deps,
        })
    }

    /// Get a crate by name
    pub fn get_crate(&self, name: &str) -> Option<&CrateInfo> {
        self.crates.get(name)
    }

    /// Check if a crate is a workspace member
    pub fn is_workspace_member(&self, name: &str) -> bool {
        self.members.contains(&name.to_string())
    }

    /// Get direct dependencies of a crate
    pub fn get_dependencies(&self, name: &str) -> Option<&HashSet<String>> {
        self.dependency_graph.get(name)
    }

    /// Get crates that depend on this crate
    pub fn get_dependents(&self, name: &str) -> Option<&HashSet<String>> {
        self.reverse_deps.get(name)
    }

    /// Calculate the distance between two crates
    /// Returns None if there's no path, 0 if same crate, 1 for direct dep, etc.
    pub fn crate_distance(&self, from: &str, to: &str) -> Option<usize> {
        if from == to {
            return Some(0);
        }

        // Direct dependency check
        if self
            .dependency_graph
            .get(from)
            .is_some_and(|deps| deps.contains(to))
        {
            return Some(1);
        }

        // BFS for longer paths
        let mut visited = HashSet::new();
        let mut queue = vec![(from.to_string(), 0usize)];

        while let Some((current, dist)) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(deps) = self.dependency_graph.get(&current) {
                for dep in deps {
                    if dep == to {
                        return Some(dist + 1);
                    }
                    if !visited.contains(dep) {
                        queue.push((dep.clone(), dist + 1));
                    }
                }
            }
        }

        None
    }

    /// Get all source files for workspace members
    pub fn get_all_source_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();

        for member in &self.members {
            if let Some(crate_info) = self.crates.get(member)
                && crate_info.src_path.exists()
            {
                for entry in walkdir::WalkDir::new(&crate_info.src_path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "rs") {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        files
    }
}

/// Find Cargo.toml by walking up from the given path
fn find_cargo_toml(start: &Path) -> Result<PathBuf, WorkspaceError> {
    let mut current = if start.is_file() {
        start.parent().map(|p| p.to_path_buf())
    } else {
        Some(start.to_path_buf())
    };

    while let Some(dir) = current {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(cargo_toml);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }

    Err(WorkspaceError::InvalidManifest(start.display().to_string()))
}

/// Resolve a module path to a crate name
/// e.g., "crate::models::user" in package "my-app" -> "my-app"
/// e.g., "serde::Serialize" -> "serde"
pub fn resolve_crate_from_path(
    use_path: &str,
    current_crate: &str,
    workspace: &WorkspaceInfo,
) -> Option<String> {
    let parts: Vec<&str> = use_path.split("::").collect();

    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "crate" | "self" | "super" => {
            // Internal reference to current crate
            Some(current_crate.to_string())
        }
        first_segment => {
            // Check if it's a known crate
            // Convert hyphens to underscores for crate names
            let normalized = first_segment.replace('-', "_");

            // Check workspace members first
            for member in &workspace.members {
                let member_normalized = member.replace('-', "_");
                if member_normalized == normalized {
                    return Some(member.clone());
                }
            }

            // Check all crates
            for name in workspace.crates.keys() {
                let name_normalized = name.replace('-', "_");
                if name_normalized == normalized {
                    return Some(name.clone());
                }
            }

            // Assume it's an external crate
            Some(first_segment.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_cargo_toml() {
        // Test in the current project
        let result = find_cargo_toml(Path::new("."));
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("Cargo.toml"));
    }

    #[test]
    fn test_resolve_crate_from_path() {
        let workspace = WorkspaceInfo {
            root: PathBuf::new(),
            crates: HashMap::new(),
            members: vec!["my-app".to_string(), "my-lib".to_string()],
            dependency_graph: HashMap::new(),
            reverse_deps: HashMap::new(),
        };

        // Internal reference
        assert_eq!(
            resolve_crate_from_path("crate::models::User", "my-app", &workspace),
            Some("my-app".to_string())
        );

        // Workspace member reference
        assert_eq!(
            resolve_crate_from_path("my_lib::utils", "my-app", &workspace),
            Some("my-lib".to_string())
        );

        // External crate
        assert_eq!(
            resolve_crate_from_path("serde::Serialize", "my-app", &workspace),
            Some("serde".to_string())
        );
    }
}
