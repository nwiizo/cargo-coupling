//! End-to-end tests for baseline diffs and ratchet checks.
//!
//! These build a throwaway git repository with two commits, then compare the
//! current checkout against a previous ref through both the library diff engine
//! and the real CLI binary.

use std::path::Path;
use std::process::Command;

use cargo_coupling::{
    CompiledConfig, IssueThresholds, IssueType, Severity, analyze_project_balance_with_thresholds,
    analyze_ref, analyze_workspace_with_config, diff_reports,
};

fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .output()
        .expect("git should be runnable");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write(path: &Path, content: &str) {
    std::fs::write(path, content).expect("write file");
}

fn fixture_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();

    git(root, &["init", "-q"]);

    write(&src.join("a.rs"), "pub struct A;\n");
    write(&src.join("b.rs"), "pub struct B;\n");
    write(&src.join("c.rs"), "pub struct C;\n");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "baseline modules"]);

    write(
        &src.join("hub.rs"),
        "use crate::a::A;\nuse crate::b::B;\nuse crate::c::C;\n\npub struct Hub {\n    pub a: A,\n    pub b: B,\n    pub c: C,\n}\n",
    );
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "introduce hub coupling"]);

    tmp
}

fn strict_thresholds() -> IssueThresholds {
    IssueThresholds {
        max_dependencies: 1,
        ..IssueThresholds::default()
    }
}

fn cargo_coupling() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
}

#[test]
fn baseline_diff_reports_new_issue_and_ratchet_results() {
    let tmp = fixture_repo();
    let root = tmp.path();
    let src = root.join("src");
    let config = CompiledConfig::empty();
    let thresholds = strict_thresholds();

    let baseline = analyze_ref(&src, &config, &thresholds, "HEAD~1", 6, true)
        .expect("baseline ref should analyze");
    let current_metrics = analyze_workspace_with_config(&src, &config).unwrap();
    let current_report = analyze_project_balance_with_thresholds(&current_metrics, &thresholds);
    let diff = diff_reports(&baseline.report, &current_report);

    assert_eq!(
        diff.new_issues.len(),
        1,
        "expected exactly one new issue, got {:?}",
        diff.new_issues
            .iter()
            .map(|issue| format!("{} {} -> {}", issue.issue_type, issue.source, issue.target))
            .collect::<Vec<_>>()
    );
    let issue = &diff.new_issues[0];
    assert_eq!(issue.issue_type, IssueType::HighEfferentCoupling);
    assert_eq!(issue.severity, Severity::High);
    assert_eq!(diff.ratchet_failures(Severity::High).len(), 1);

    let fail = cargo_coupling()
        .args([
            "coupling",
            "--check",
            "--baseline",
            "HEAD~1",
            "--max-deps",
            "1",
        ])
        .arg(&src)
        .current_dir(root)
        .output()
        .expect("run cargo-coupling");
    assert!(
        !fail.status.success(),
        "ratchet should fail when HEAD introduced a high new issue\nstdout:\n{}",
        String::from_utf8_lossy(&fail.stdout)
    );
    let fail_stdout = String::from_utf8_lossy(&fail.stdout);
    assert!(fail_stdout.contains("Coupling Ratchet Gate"));
    assert!(fail_stdout.contains("Blocking New Issues"));
    assert!(fail_stdout.contains("High Efferent Coupling"));

    let pass = cargo_coupling()
        .args([
            "coupling",
            "--check",
            "--baseline",
            "HEAD",
            "--max-deps",
            "1",
        ])
        .arg(&src)
        .current_dir(root)
        .output()
        .expect("run cargo-coupling");
    assert!(
        pass.status.success(),
        "ratchet should pass when baseline equals HEAD\nstdout:\n{}",
        String::from_utf8_lossy(&pass.stdout)
    );

    let same_baseline = analyze_ref(&src, &config, &thresholds, "HEAD", 6, true)
        .expect("HEAD baseline should analyze");
    let no_change = diff_reports(&same_baseline.report, &current_report);
    assert!(no_change.new_issues.is_empty());
    assert!(no_change.ratchet_failures(Severity::High).is_empty());
}

#[test]
fn baseline_ref_with_slash_analyzes_in_path_safe_worktree() {
    let tmp = fixture_repo();
    let root = tmp.path();
    let src = root.join("src");
    git(root, &["branch", "feat/foo", "HEAD~1"]);

    let config = CompiledConfig::empty();
    let thresholds = strict_thresholds();

    let baseline = analyze_ref(&src, &config, &thresholds, "feat/foo", 6, true)
        .expect("slash-containing baseline ref should analyze");

    assert_eq!(baseline.git_ref, "feat/foo");
    assert!(baseline.module_count > 0);
}

#[test]
fn baseline_json_contains_top_level_diff_object() {
    let tmp = fixture_repo();
    let root = tmp.path();
    let src = root.join("src");
    let output = root.join("diff.json");

    let status = cargo_coupling()
        .args([
            "coupling",
            "--baseline",
            "HEAD~1",
            "--max-deps",
            "1",
            "--json",
            "--output",
        ])
        .arg(&output)
        .arg(&src)
        .current_dir(root)
        .status()
        .expect("run cargo-coupling");
    assert!(status.success());

    let text = std::fs::read_to_string(output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    let diff = &parsed["diff"];

    assert!(diff.is_object(), "expected top-level diff object: {parsed}");
    assert_eq!(diff["new_issues"].as_array().unwrap().len(), 1);
    assert!(diff["resolved_issues"].as_array().unwrap().is_empty());
    assert!(diff["unchanged"].as_u64().is_some());
    assert!(diff["score_delta"].as_f64().is_some());
    assert!(diff["grade_change"]["baseline"].as_str().is_some());
    assert!(diff["grade_change"]["current"].as_str().is_some());
}
