//! Source file discovery for crate analysis.
//!
//! This module finds the source files of each crate via (a) directory walks of
//! cargo-metadata target roots and (b) module-tree resolution from crate roots
//! following `mod` declarations, including `#[path]` attributes, and derives
//! module names. Walk-based names take precedence for files both methods find.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

use syn::{Expr, ExprLit, ItemMod, Lit, Meta};
use walkdir::WalkDir;

/// Convert file path to module path relative to the source root.
///
/// Examples:
/// - `src/level/enemy/spawner.rs` with root `src` → `level::enemy::spawner`
/// - `src/lib.rs` with root `src` → `` (empty, crate root)
/// - `src/main.rs` with root `src` → `` (empty, crate root)
/// - `src/level/mod.rs` with root `src` → `level`
/// - `src/utils.rs` with root `src` → `utils`
///
/// See: https://github.com/nwiizo/cargo-coupling/issues/14
pub(crate) fn file_path_to_module_path(file_path: &Path, src_root: &Path) -> String {
    // Get the relative path from src root
    let relative = file_path.strip_prefix(src_root).unwrap_or(file_path);

    let mut parts: Vec<String> = Vec::new();

    for component in relative.components() {
        if let Some(component_name) = component.as_os_str().to_str() {
            parts.push(component_name.to_string());
        }
    }

    // Handle the last component (filename)
    if let Some(last) = parts.last().cloned() {
        parts.pop();
        match last.as_str() {
            "lib.rs" | "main.rs" => {
                // Crate root - don't add anything
            }
            "mod.rs" => {
                // mod.rs represents its parent directory, already in parts
            }
            _ => {
                // Regular file - remove .rs extension and add to path
                if let Some(stem) = last.strip_suffix(".rs") {
                    parts.push(stem.to_string());
                } else {
                    parts.push(last);
                }
            }
        }
    }

    parts.join("::")
}

/// Normalize a path for exclude matching without resolving symlinks.
///
/// This keeps `./src`, `/tmp/foo`, and other caller-provided forms comparable
/// by making them absolute and removing `.` / `..` components lexically.
pub(crate) fn normalize_exclude_path(path: &Path) -> PathBuf {
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

/// Get an iterator over all Rust source files in `dir`, excluding hidden directories and `target/`.
///
/// Uses relative paths for filtering to avoid false positives when the project
/// is located in a path containing hidden directories (e.g., `/home/user/.local/projects/`).
/// See: https://github.com/nwiizo/cargo-coupling/issues/7
pub(crate) fn rs_files(dir: &Path) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(move |entry| {
            let file_path = entry.path();
            // Use relative path from the search root to check for hidden/target directories.
            // This prevents false positives when parent directories contain `.` or `target`.
            // Example: `/home/user/.config/myproject/src/lib.rs` should not be skipped
            // just because `.config` is in the parent path.
            let file_path = file_path.strip_prefix(dir).unwrap_or(file_path);

            // Skip target directory and hidden directories
            !file_path.components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                s == "target" || s.starts_with('.')
            }) && file_path.extension() == Some(OsStr::new("rs"))
        })
        .map(|e| e.path().to_path_buf())
}

pub(crate) fn rs_files_excluding_nested_packages(
    dir: &Path,
    manifest_path: &Path,
) -> impl Iterator<Item = PathBuf> {
    let manifest_path = normalize_exclude_path(manifest_path);
    WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(move |entry| {
            should_descend_workspace_source(entry.path(), dir, &manifest_path)
        })
        .filter_map(|e| e.ok())
        .filter(move |entry| {
            let file_path = entry.path();
            let relative_path = file_path.strip_prefix(dir).unwrap_or(file_path);

            !relative_path.components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                s == "target" || s.starts_with('.')
            }) && file_path.extension() == Some(OsStr::new("rs"))
        })
        .map(|e| e.path().to_path_buf())
}

fn should_descend_workspace_source(path: &Path, root: &Path, manifest_path: &Path) -> bool {
    let relative_path = path.strip_prefix(root).unwrap_or(path);
    if relative_path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        s == "target" || s.starts_with('.')
    }) {
        return false;
    }

    if path.is_dir() {
        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.exists() && normalize_exclude_path(&cargo_toml) != manifest_path {
            return false;
        }
    }

    true
}

#[derive(Debug, Clone)]
pub(crate) struct DiscoveredWorkspaceFile {
    pub(crate) file_path: PathBuf,
    pub(crate) crate_name: String,
    pub(crate) source_root: PathBuf,
    pub(crate) module_name: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ModuleTreeFile {
    pub(crate) file_path: PathBuf,
    pub(crate) module_name: String,
}

pub(crate) fn canonical_file_key(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| normalize_exclude_path(path))
}

pub(crate) fn discover_module_tree(
    crate_root: &Path,
    crate_root_module_name: String,
) -> Vec<ModuleTreeFile> {
    let mut files = Vec::new();
    let mut visited = HashSet::new();
    discover_module_tree_file(
        crate_root,
        crate_root_module_name,
        crate_root.parent().unwrap_or_else(|| Path::new("")),
        &mut visited,
        &mut files,
    );
    files
}

fn discover_module_tree_file(
    file_path: &Path,
    module_name: String,
    module_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    files: &mut Vec<ModuleTreeFile>,
) {
    if !file_path.exists() {
        return;
    }

    let file_key = canonical_file_key(file_path);
    if !visited.insert(file_key) {
        return;
    }

    files.push(ModuleTreeFile {
        file_path: file_path.to_path_buf(),
        module_name: module_name.clone(),
    });

    let Ok(content) = fs::read_to_string(file_path) else {
        return;
    };
    let Ok(parsed) = syn::parse_file(&content) else {
        return;
    };

    discover_module_items(&parsed.items, module_dir, &module_name, visited, files);
}

fn discover_module_items(
    items: &[syn::Item],
    module_dir: &Path,
    parent_module: &str,
    visited: &mut HashSet<PathBuf>,
    files: &mut Vec<ModuleTreeFile>,
) {
    for item in items {
        let syn::Item::Mod(item_mod) = item else {
            continue;
        };

        let child_name = item_mod.ident.to_string();
        let child_module = join_module_path(parent_module, &child_name);
        if let Some((_, inline_items)) = &item_mod.content {
            let inline_module_dir = module_dir.join(&child_name);
            discover_module_items(
                inline_items,
                &inline_module_dir,
                &child_module,
                visited,
                files,
            );
            continue;
        }

        if let Some(resolved_file) = resolve_external_module_file(module_dir, item_mod) {
            let child_module_dir = module_dir_for_resolved_module(&resolved_file);
            discover_module_tree_file(
                &resolved_file,
                child_module,
                &child_module_dir,
                visited,
                files,
            );
        }
    }
}

fn resolve_external_module_file(module_dir: &Path, item_mod: &ItemMod) -> Option<PathBuf> {
    if let Some(path_attr) = path_attribute_value(&item_mod.attrs) {
        return Some(module_dir.join(path_attr));
    }

    let module_name = item_mod.ident.to_string();
    let flat = module_dir.join(format!("{module_name}.rs"));
    if flat.exists() {
        return Some(flat);
    }

    let nested = module_dir.join(&module_name).join("mod.rs");
    if nested.exists() { Some(nested) } else { None }
}

fn path_attribute_value(attrs: &[syn::Attribute]) -> Option<PathBuf> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("path") {
            return None;
        }
        match &attr.meta {
            Meta::NameValue(name_value) => {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) = &name_value.value
                {
                    Some(PathBuf::from(value.value()))
                } else {
                    None
                }
            }
            _ => None,
        }
    })
}

fn module_dir_for_resolved_module(file_path: &Path) -> PathBuf {
    let parent = file_path.parent().unwrap_or_else(|| Path::new(""));
    if file_path.file_name() == Some(OsStr::new("mod.rs")) {
        parent.to_path_buf()
    } else {
        parent.join(
            file_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default(),
        )
    }
}

pub(crate) fn join_module_path(prefix: &str, rest: &str) -> String {
    if prefix.is_empty() {
        rest.to_string()
    } else if rest.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}::{rest}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that rs_files correctly handles paths with hidden parent directories.
    /// Regression test for https://github.com/nwiizo/cargo-coupling/issues/7
    #[test]
    fn test_rs_files_with_hidden_parent_directory() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory structure that simulates a project
        // inside a hidden parent directory (e.g., /home/user/.local/projects/myproject)
        let temp = TempDir::new().unwrap();
        let hidden_parent = temp.path().join(".hidden-parent");
        let project_dir = hidden_parent.join("myproject").join("src");
        fs::create_dir_all(&project_dir).unwrap();

        // Create some Rust files
        fs::write(project_dir.join("lib.rs"), "pub fn hello() {}").unwrap();
        fs::write(project_dir.join("main.rs"), "fn main() {}").unwrap();

        // rs_files should find both files even though there's a hidden parent
        let files: Vec<_> = rs_files(&project_dir).collect();
        assert_eq!(
            files.len(),
            2,
            "Should find 2 .rs files in hidden parent path"
        );

        // Verify the files are the ones we created
        let file_names: Vec<_> = files
            .iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .collect();
        assert!(file_names.contains(&"lib.rs"));
        assert!(file_names.contains(&"main.rs"));
    }

    /// Test that rs_files correctly excludes hidden directories within the project.
    #[test]
    fn test_rs_files_excludes_hidden_dirs_in_project() {
        use std::fs;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let project_dir = temp.path().join("myproject").join("src");
        let hidden_dir = project_dir.join(".hidden");
        fs::create_dir_all(&hidden_dir).unwrap();

        // Create files in both regular and hidden directories
        fs::write(project_dir.join("lib.rs"), "pub fn hello() {}").unwrap();
        fs::write(hidden_dir.join("secret.rs"), "fn secret() {}").unwrap();

        // rs_files should only find lib.rs, not the file in .hidden
        let files: Vec<_> = rs_files(&project_dir).collect();
        assert_eq!(
            files.len(),
            1,
            "Should find only 1 .rs file (excluding .hidden/)"
        );

        let file_names: Vec<_> = files
            .iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .collect();
        assert!(file_names.contains(&"lib.rs"));
        assert!(!file_names.contains(&"secret.rs"));
    }

    /// Test that rs_files correctly excludes the target directory.
    #[test]
    fn test_rs_files_excludes_target_directory() {
        use std::fs;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let project_dir = temp.path().join("myproject");
        let src_dir = project_dir.join("src");
        let target_dir = project_dir.join("target").join("debug");
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();

        // Create files in both src and target directories
        fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();
        fs::write(target_dir.join("generated.rs"), "// generated").unwrap();

        // rs_files should only find lib.rs, not the file in target/
        let files: Vec<_> = rs_files(&project_dir).collect();
        assert_eq!(
            files.len(),
            1,
            "Should find only 1 .rs file (excluding target/)"
        );

        let file_names: Vec<_> = files
            .iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .collect();
        assert!(file_names.contains(&"lib.rs"));
        assert!(!file_names.contains(&"generated.rs"));
    }

    #[test]
    fn test_file_path_to_module_path_nested() {
        // Test: src/level/enemy/spawner.rs -> level::enemy::spawner
        let src_root = Path::new("/project/src");
        let file_path = Path::new("/project/src/level/enemy/spawner.rs");
        assert_eq!(
            file_path_to_module_path(file_path, src_root),
            "level::enemy::spawner"
        );
    }

    #[test]
    fn test_file_path_to_module_path_lib() {
        // Test: src/lib.rs -> "" (crate root)
        let src_root = Path::new("/project/src");
        let file_path = Path::new("/project/src/lib.rs");
        assert_eq!(file_path_to_module_path(file_path, src_root), "");
    }

    #[test]
    fn test_file_path_to_module_path_main() {
        // Test: src/main.rs -> "" (crate root)
        let src_root = Path::new("/project/src");
        let file_path = Path::new("/project/src/main.rs");
        assert_eq!(file_path_to_module_path(file_path, src_root), "");
    }

    #[test]
    fn test_file_path_to_module_path_mod() {
        // Test: src/level/mod.rs -> level
        let src_root = Path::new("/project/src");
        let file_path = Path::new("/project/src/level/mod.rs");
        assert_eq!(file_path_to_module_path(file_path, src_root), "level");
    }

    #[test]
    fn test_file_path_to_module_path_deeply_nested_mod() {
        // Test: src/a/b/c/mod.rs -> a::b::c
        let src_root = Path::new("/project/src");
        let file_path = Path::new("/project/src/a/b/c/mod.rs");
        assert_eq!(file_path_to_module_path(file_path, src_root), "a::b::c");
    }

    #[test]
    fn test_file_path_to_module_path_simple() {
        // Test: src/utils.rs -> utils
        let src_root = Path::new("/project/src");
        let file_path = Path::new("/project/src/utils.rs");
        assert_eq!(file_path_to_module_path(file_path, src_root), "utils");
    }

    #[test]
    fn test_file_path_to_module_path_two_levels() {
        // Test: src/foo/bar.rs -> foo::bar
        let src_root = Path::new("/project/src");
        let file_path = Path::new("/project/src/foo/bar.rs");
        assert_eq!(file_path_to_module_path(file_path, src_root), "foo::bar");
    }

    #[test]
    fn test_file_path_to_module_path_bin() {
        // Test: src/bin/cli.rs -> bin::cli
        let src_root = Path::new("/project/src");
        let file_path = Path::new("/project/src/bin/cli.rs");
        assert_eq!(file_path_to_module_path(file_path, src_root), "bin::cli");
    }

    #[test]
    fn test_file_path_to_module_path_mismatched_root() {
        // When strip_prefix fails, we fall back to using the full path
        // This handles edge cases where src_root doesn't match
        let src_root = Path::new("/other/src");
        let file_path = Path::new("/project/src/utils.rs");
        // Falls back to full path processing
        let result = file_path_to_module_path(file_path, src_root);
        // Should still produce something reasonable
        assert!(result.contains("utils"));
    }
}
