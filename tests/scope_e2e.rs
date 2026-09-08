//! Regression coverage for explicitly scoped workspace analysis (Issue #86).

use std::fs;
use std::path::Path;

use cargo_coupling::{
    CompiledConfig, ProjectMetrics, analyze_workspace_with_config, load_compiled_config,
};

fn fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("alpha/src/feature")).unwrap();
    fs::create_dir_all(root.join("beta/src")).unwrap();
    for (path, content) in [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"alpha\", \"beta\"]\nresolver = \"3\"\n",
        ),
        (
            "alpha/Cargo.toml",
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[dependencies]\nbeta = { path = \"../beta\" }\n",
        ),
        (
            "beta/Cargo.toml",
            "[package]\nname = \"beta\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        ),
        (
            "alpha/src/lib.rs",
            "use beta::shared;\npub mod feature;\npub mod other;\npub fn run() { shared::call(); feature::run(); }\n",
        ),
        (
            "alpha/src/feature/mod.rs",
            "pub mod helper;\npub fn run() { helper::call(); }\n",
        ),
        ("alpha/src/feature/helper.rs", "pub fn call() {}\n"),
        ("alpha/src/other.rs", "pub fn other() {}\n"),
        ("beta/src/lib.rs", "pub mod shared;\n"),
        ("beta/src/shared.rs", "pub fn call() {}\n"),
    ] {
        fs::write(root.join(path), content).unwrap();
    }
    tmp
}

fn analyze(path: &Path) -> ProjectMetrics {
    analyze_workspace_with_config(path, &CompiledConfig::empty()).unwrap()
}

fn assert_scope(metrics: &ProjectMetrics, scope: &Path, files: usize) {
    assert_eq!(metrics.total_files, files);
    let scope = fs::canonicalize(scope).unwrap();
    for module in metrics.modules.values() {
        assert!(
            fs::canonicalize(&module.path).unwrap().starts_with(&scope),
            "{} is outside {}",
            module.path.display(),
            scope.display(),
        );
    }
    assert!(metrics.skipped_crates.is_empty());
}

#[test]
fn workspace_root_keeps_all_members() {
    let tmp = fixture();
    let metrics = analyze(tmp.path());
    assert_scope(&metrics, tmp.path(), 6);
    assert!(metrics.modules.contains_key("shared"));
    assert!(metrics.modules.contains_key("other"));
    assert_eq!(metrics.workspace_members.len(), 2);
}

#[test]
fn member_scope_excludes_siblings_but_preserves_dependency_resolution() {
    let tmp = fixture();
    let alpha = tmp.path().join("alpha");
    let metrics = analyze(&alpha);
    assert_scope(&metrics, &alpha, 4);
    assert!(
        metrics
            .couplings
            .iter()
            .all(|c| c.source.starts_with("alpha::"))
    );
    assert!(
        metrics
            .couplings
            .iter()
            .any(|c| c.target_crate.as_deref() == Some("beta"))
    );
    assert_eq!(metrics.crate_dependencies.len(), 1);
    assert!(metrics.crate_dependencies["alpha"].contains(&"beta".to_string()));
}

#[test]
fn source_directory_scope_preserves_qualified_module_names() {
    let tmp = fixture();
    let feature = tmp.path().join("alpha/src/feature");
    let metrics = analyze(&feature);
    assert_scope(&metrics, &feature, 2);
    assert!(metrics.modules.contains_key("feature"));
    assert!(metrics.modules.contains_key("feature::helper"));
}

#[test]
fn source_file_scope_analyzes_only_that_file() {
    let tmp = fixture();
    let helper = tmp.path().join("alpha/src/feature/helper.rs");
    assert_scope(&analyze(&helper), &helper, 1);
}

#[test]
fn member_manifest_selects_the_member() {
    let tmp = fixture();
    let alpha = tmp.path().join("alpha");
    assert_scope(&analyze(&alpha.join("Cargo.toml")), &alpha, 4);
}

#[test]
fn partial_scope_does_not_report_sibling_config_as_dead() {
    let tmp = fixture();
    fs::write(
        tmp.path().join(".coupling.toml"),
        "[subdomains]\ncore = [\"beta/src/retired.rs\"]\n",
    )
    .unwrap();
    let config = load_compiled_config(tmp.path()).unwrap();
    let full = analyze_workspace_with_config(tmp.path(), &config).unwrap();
    assert!(!full.dead_config_patterns.is_empty());
    let partial = analyze_workspace_with_config(&tmp.path().join("alpha"), &config).unwrap();
    assert!(partial.dead_config_patterns.is_empty());
}

#[test]
fn member_scope_keeps_its_path_modules_outside_the_member_directory() {
    let tmp = fixture();
    fs::write(tmp.path().join("shared.rs"), "pub fn call() {}\n").unwrap();
    fs::write(
        tmp.path().join("alpha/src/lib.rs"),
        "#[path = \"../../shared.rs\"]\npub mod shared;\npub fn run() { shared::call(); }\n",
    )
    .unwrap();
    let metrics = analyze(&tmp.path().join("alpha"));
    assert_eq!(metrics.total_files, 5);
    assert!(metrics.modules.values().any(|module| {
        fs::canonicalize(&module.path).unwrap()
            == fs::canonicalize(tmp.path().join("shared.rs")).unwrap()
    }));
    assert!(
        metrics
            .couplings
            .iter()
            .all(|c| c.source.starts_with("alpha::"))
    );
}

#[cfg(unix)]
#[test]
fn symlink_and_parent_components_preserve_the_requested_scope() {
    let tmp = fixture();
    let links = tempfile::tempdir().unwrap();
    let alias = links.path().join("workspace");
    std::os::unix::fs::symlink(tmp.path(), &alias).unwrap();
    let path = alias.join("alpha/src/feature/../other.rs");
    assert_scope(&analyze(&path), &path, 1);
}
