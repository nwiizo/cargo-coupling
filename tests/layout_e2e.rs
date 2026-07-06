//! End-to-end layout tests for workspace source discovery.

use std::fs;
use std::path::Path;

use cargo_coupling::{
    CompiledConfig, ManifestContext, ProjectMetrics, analyze_workspace_with_config, build_manifest,
};

fn write(path: &Path, content: &str) {
    fs::write(path, content).expect("write fixture file");
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).expect("create fixture directory");
}

fn analyze(root: &Path) -> ProjectMetrics {
    analyze_workspace_with_config(root, &CompiledConfig::empty()).expect("analyze fixture")
}

fn module_names(metrics: &ProjectMetrics) -> Vec<String> {
    let mut names = metrics.modules.keys().cloned().collect::<Vec<_>>();
    names.sort();
    names
}

#[test]
fn non_src_bin_with_path_modules_is_analyzed() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    create_dir(&root.join("bin"));
    create_dir(&root.join("crates/domain"));
    create_dir(&root.join("crates/application"));
    create_dir(&root.join("crates/infrastructure"));

    write(
        &root.join("Cargo.toml"),
        r#"[package]
name = "layout1"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "app"
path = "bin/main.rs"
"#,
    );
    write(
        &root.join("bin/main.rs"),
        r#"#[path = "../crates/domain/mod.rs"]
mod domain;
#[path = "../crates/infrastructure/mod.rs"]
mod infrastructure;
#[path = "../crates/application/mod.rs"]
mod application;

fn main() {
    application::run();
}
"#,
    );
    write(
        &root.join("crates/domain/mod.rs"),
        "pub mod hello;\npub mod world;\n",
    );
    write(
        &root.join("crates/domain/hello.rs"),
        r#"pub struct Hello;

impl Hello {
    pub fn message() -> &'static str {
        "hello"
    }
}
"#,
    );
    write(&root.join("crates/domain/world.rs"), "pub struct World;\n");
    write(
        &root.join("crates/application/mod.rs"),
        r#"use crate::domain::hello::Hello;

pub fn run() -> &'static str {
    Hello::message()
}
"#,
    );
    write(
        &root.join("crates/infrastructure/mod.rs"),
        "pub struct Repository;\n",
    );

    let metrics = analyze(root);
    let names = module_names(&metrics);

    assert!(
        metrics.modules.contains_key("domain"),
        "expected domain module, saw {names:?}"
    );
    assert!(
        metrics.modules.contains_key("application"),
        "expected application module, saw {names:?}"
    );
    assert!(
        metrics.modules.contains_key("infrastructure"),
        "expected infrastructure module, saw {names:?}"
    );
    assert!(
        metrics.couplings.iter().any(|coupling| {
            coupling.source.ends_with("::application") && coupling.target.contains("domain")
        }),
        "expected application -> domain coupling, saw {:?}",
        metrics
            .couplings
            .iter()
            .map(|coupling| format!("{} -> {}", coupling.source, coupling.target))
            .collect::<Vec<_>>()
    );
    assert!(metrics.skipped_crates.is_empty());
}

#[test]
fn ripgrep_style_bin_under_crates_core_is_analyzed() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    create_dir(&root.join("crates/core"));

    write(
        &root.join("Cargo.toml"),
        r#"[package]
name = "ripgrep-style"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "rg"
path = "crates/core/main.rs"
"#,
    );
    write(
        &root.join("crates/core/main.rs"),
        "mod args;\nfn main() { let _args = args::Args::new(); }\n",
    );
    write(
        &root.join("crates/core/args.rs"),
        "pub struct Args;\nimpl Args { pub fn new() -> Self { Self } }\n",
    );

    let metrics = analyze(root);
    let names = module_names(&metrics);

    assert!(
        metrics.modules.contains_key("args"),
        "expected args module, saw {names:?}"
    );
    assert!(metrics.skipped_crates.is_empty());
}

#[test]
fn mixed_workspace_analyzes_conventional_and_non_src_members() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    create_dir(&root.join("conventional/src"));
    create_dir(&root.join("non_src/bin"));
    create_dir(&root.join("non_src/crates/domain"));

    write(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = ["conventional", "non_src"]
resolver = "3"
"#,
    );
    write(
        &root.join("conventional/Cargo.toml"),
        r#"[package]
name = "conventional"
version = "0.1.0"
edition = "2024"
"#,
    );
    write(
        &root.join("conventional/src/lib.rs"),
        "pub mod utils;\npub fn call() { utils::ping(); }\n",
    );
    write(
        &root.join("conventional/src/utils.rs"),
        "pub fn ping() {}\n",
    );
    write(
        &root.join("non_src/Cargo.toml"),
        r#"[package]
name = "non-src"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "non-src"
path = "bin/main.rs"
"#,
    );
    write(
        &root.join("non_src/bin/main.rs"),
        "#[path = \"../crates/domain/mod.rs\"]\nmod domain;\nfn main() { domain::run(); }\n",
    );
    write(
        &root.join("non_src/crates/domain/mod.rs"),
        "pub fn run() {}\n",
    );

    let metrics = analyze(root);
    let names = module_names(&metrics);

    assert!(
        metrics.modules.contains_key("utils"),
        "expected conventional member module, saw {names:?}"
    );
    assert!(
        metrics.modules.contains_key("domain"),
        "expected non-src member module, saw {names:?}"
    );
    assert!(metrics.skipped_crates.is_empty());
}

#[test]
fn conventional_src_layout_keeps_existing_module_names() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    create_dir(&root.join("src/level"));

    write(
        &root.join("Cargo.toml"),
        r#"[package]
name = "conventional-layout"
version = "0.1.0"
edition = "2024"
"#,
    );
    write(&root.join("src/lib.rs"), "pub mod utils;\npub mod level;\n");
    write(&root.join("src/utils.rs"), "pub fn helper() {}\n");
    write(&root.join("src/level/mod.rs"), "pub fn depth() {}\n");

    let metrics = analyze(root);
    let names = module_names(&metrics);

    assert!(
        metrics.modules.contains_key("lib"),
        "expected crate root module named by file stem (pre-existing behavior), saw {names:?}"
    );
    assert!(
        metrics.modules.contains_key("utils"),
        "expected utils module, saw {names:?}"
    );
    assert!(
        metrics.modules.contains_key("level"),
        "expected level module, saw {names:?}"
    );
}

#[test]
fn members_with_no_discoverable_sources_are_declared_in_manifest() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    create_dir(&root.join("missing"));

    write(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = ["missing"]
resolver = "3"
"#,
    );
    write(
        &root.join("missing/Cargo.toml"),
        r#"[package]
name = "missing"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "missing"
path = "does-not-exist/main.rs"
"#,
    );

    let metrics = analyze(root);
    assert_eq!(metrics.skipped_crates, vec!["missing".to_string()]);

    let manifest = build_manifest(&ManifestContext {
        git_used: true,
        tests_excluded: false,
        parse_failures: metrics.parse_failures,
        skipped_crates: metrics.skipped_crates,
    });

    assert!(manifest.notes.iter().any(|note| {
        note.contains(
            "Workspace member(s) missing had no discoverable source files and were not analyzed.",
        )
    }));
    assert!(manifest.notes_ja.iter().any(|note| {
        note.contains(
            "ワークスペースメンバー missing のソースファイルを発見できず、解析されていません。",
        )
    }));
}
