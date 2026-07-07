//! End-to-end coverage for config subdomain volatility overrides.

use std::path::Path;
use std::process::Command;

use cargo_coupling::{
    AnalysisManifest, CompiledConfig, IssueType, ManifestContext, Volatility,
    analyze_project_balance, analyze_workspace_with_config, build_manifest, load_compiled_config,
};

fn write(path: &Path, content: &str) {
    std::fs::write(path, content).expect("write file");
}

fn fixture_project() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();

    write(
        &root.join(".coupling.toml"),
        r#"
[subdomains]
supporting = ["src/stable.rs"]
"#,
    );
    write(&src.join("lib.rs"), "pub mod caller;\npub mod stable;\n");
    write(
        &src.join("stable.rs"),
        "pub struct Stable {\n    pub value: i32,\n}\n",
    );
    write(
        &src.join("caller.rs"),
        "use crate::stable::Stable;\n\npub fn make() -> Stable {\n    Stable { value: 1 }\n}\n",
    );

    tmp
}

fn mark_stable_as_git_churn(metrics: &mut cargo_coupling::ProjectMetrics) {
    metrics.file_changes.insert("src/stable.rs".to_string(), 12);
    metrics.file_changes.insert("src/caller.rs".to_string(), 1);
    metrics.file_changes.insert("src/lib.rs".to_string(), 1);
    metrics.update_volatility_from_git();
}

fn cargo_coupling() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
}

fn manifest_for_config(config_toml: &str) -> AnalysisManifest {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();

    write(
        &root.join("Cargo.toml"),
        r#"[package]
name = "drift-fixture"
version = "0.1.0"
edition = "2024"
"#,
    );
    write(&root.join(".coupling.toml"), config_toml);
    write(&src.join("lib.rs"), "pub mod live;\npub mod excluded;\n");
    write(&src.join("live.rs"), "pub struct Live;\n");
    write(&src.join("excluded.rs"), "pub struct Excluded;\n");

    let config = load_compiled_config(&src).unwrap();
    let metrics = analyze_workspace_with_config(&src, &config).unwrap();

    build_manifest(&ManifestContext {
        git_used: true,
        tests_excluded: config.exclude_tests,
        parse_failures: metrics.parse_failures,
        skipped_crates: metrics.skipped_crates,
        boundary_skipped_files: metrics.boundary_skipped_files,
        dead_config_patterns: metrics.dead_config_patterns,
    })
}

#[test]
fn subdomain_override_changes_coupling_volatility_and_issue_set() {
    let tmp = fixture_project();
    let root = tmp.path();
    let src = root.join("src");

    let empty_config = CompiledConfig::empty();
    let mut git_only_metrics = analyze_workspace_with_config(&src, &empty_config).unwrap();
    mark_stable_as_git_churn(&mut git_only_metrics);
    assert!(
        git_only_metrics
            .couplings
            .iter()
            .any(|coupling| coupling.target.contains("stable")
                && coupling.volatility == Volatility::High),
        "git churn should make the stable target high volatility before config override"
    );
    let git_only_report = analyze_project_balance(&git_only_metrics);
    assert!(
        git_only_report
            .issues
            .iter()
            .any(|issue| issue.issue_type == IssueType::CascadingChangeRisk),
        "intrusive coupling to git-high target should surface cascading-change risk"
    );

    let mut config = load_compiled_config(&src).unwrap();
    let mut configured_metrics = analyze_workspace_with_config(&src, &config).unwrap();
    mark_stable_as_git_churn(&mut configured_metrics);
    let override_count = configured_metrics.apply_config_volatility_overrides(&mut config);

    assert!(override_count > 0, "expected at least one applied override");
    assert!(
        configured_metrics
            .couplings
            .iter()
            .any(|coupling| coupling.target.contains("stable")
                && coupling.volatility == Volatility::Low),
        "supporting subdomain should override the target coupling to low volatility"
    );
    assert_eq!(
        configured_metrics.modules["stable"]
            .subdomain
            .unwrap()
            .to_string(),
        "Supporting"
    );
    let configured_report = analyze_project_balance(&configured_metrics);
    assert!(
        !configured_report
            .issues
            .iter()
            .any(|issue| issue.issue_type == IssueType::CascadingChangeRisk),
        "low essential volatility should clear cascading-change risk"
    );
}

#[test]
fn verbose_cli_reports_nonzero_applied_volatility_overrides() {
    let tmp = fixture_project();
    let root = tmp.path();
    let src = root.join("src");

    let output = cargo_coupling()
        .args(["coupling", "--verbose", "--no-git"])
        .arg(&src)
        .current_dir(root)
        .output()
        .expect("run cargo-coupling");

    assert!(
        output.status.success(),
        "CLI should succeed\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Applied ") && stderr.contains(" volatility overrides from config"),
        "verbose stderr should report applied overrides, got:\n{stderr}"
    );
    assert!(
        !stderr.contains("Applied 0 volatility overrides"),
        "override count should be nonzero, got:\n{stderr}"
    );
}

#[test]
fn stale_subdomain_pattern_is_declared_in_manifest() {
    let manifest = manifest_for_config(
        r#"
[subdomains]
core = ["src/missing.rs"]
"#,
    );

    assert!(manifest.notes.iter().any(|note| {
        note.contains(".coupling.toml drift:") && note.contains("subdomains.core: src/missing.rs")
    }));
}

#[test]
fn matching_subdomain_pattern_has_no_drift_note() {
    let manifest = manifest_for_config(
        r#"
[subdomains]
core = ["src/live.rs"]
"#,
    );

    assert!(
        !manifest
            .notes
            .iter()
            .any(|note| note.contains(".coupling.toml drift"))
    );
}

#[test]
fn excluded_file_pattern_is_dead_for_subdomain_but_not_for_exclude() {
    let manifest = manifest_for_config(
        r#"
[analysis]
exclude = ["src/excluded.rs"]

[subdomains]
core = ["src/excluded.rs"]
"#,
    );

    assert!(manifest.notes.iter().any(|note| {
        note.contains(".coupling.toml drift:") && note.contains("subdomains.core: src/excluded.rs")
    }));
    assert!(
        !manifest
            .notes
            .iter()
            .any(|note| note.contains("analysis.exclude: src/excluded.rs"))
    );
}

#[test]
fn pattern_matching_only_a_parse_failing_file_is_not_drift() {
    // The parse-failure note owns this degradation; a drift note would misdirect
    // the user toward editing .coupling.toml when the real fix is the syntax error.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();

    write(
        &root.join("Cargo.toml"),
        "[package]\nname = \"broken-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write(
        &root.join(".coupling.toml"),
        "[subdomains]\ncore = [\"src/broken.rs\"]\n",
    );
    write(&src.join("lib.rs"), "pub mod broken;\n");
    write(&src.join("broken.rs"), "pub struct {{{ not rust\n");

    let config = load_compiled_config(&src).unwrap();
    let metrics = analyze_workspace_with_config(&src, &config).unwrap();

    assert!(metrics.parse_failures >= 1, "fixture must fail to parse");
    assert!(
        metrics.dead_config_patterns.is_empty(),
        "parse-failing file still counts as a pattern match candidate, got {:?}",
        metrics.dead_config_patterns
    );
}

#[test]
fn out_of_scope_pattern_is_not_drift_in_partial_scope_run() {
    // A shared root .coupling.toml can describe sibling trees; analyzing one
    // subtree (the non-workspace fallback path scopes strictly to the given dir)
    // must not claim sibling-directed patterns are rotted.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::create_dir_all(root.join("other")).unwrap();

    write(
        &root.join(".coupling.toml"),
        "[subdomains]\ncore = [\"other/**\"]\nsupporting = [\"src/missing.rs\"]\n",
    );
    write(&src.join("lib.rs"), "pub struct Live;\n");
    write(&root.join("other/thing.rs"), "pub struct Thing;\n");

    let config = load_compiled_config(&src).unwrap();
    let metrics = cargo_coupling::analyze_project_parallel_with_config(&src, &config).unwrap();

    assert!(
        !metrics
            .dead_config_patterns
            .iter()
            .any(|p| p.contains("other/**")),
        "sibling-tree pattern must not be judged by a src-scoped run, got {:?}",
        metrics.dead_config_patterns
    );
    assert!(
        metrics
            .dead_config_patterns
            .iter()
            .any(|p| p.contains("subdomains.supporting: src/missing.rs")),
        "in-scope stale pattern must still be reported, got {:?}",
        metrics.dead_config_patterns
    );
}
