//! End-to-end tests for the `--history` time-series feature.
//!
//! These build a throwaway git repository in a temp directory, commit a couple of
//! revisions, then run the real `analyze_history` engine against it. This exercises
//! worktree creation, per-revision structural analysis, and cleanup end to end,
//! while remaining hermetic and runnable anywhere (including under cargo-mutants,
//! which copies the source tree outside the original git repo).

use std::path::Path;
use std::process::Command;

use cargo_coupling::{
    CompiledConfig, IssueThresholds, VolatilityAnalyzer, analyze_history,
    analyze_project_balance_with_thresholds, analyze_workspace_with_config,
};

/// Run a git command in `dir`, panicking with context on failure.
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

#[test]
fn history_builds_timeline_from_a_real_git_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();

    git(root, &["init", "-q"]);

    // Revision 1: two modules with a single cross-module coupling.
    write(&src.join("lib.rs"), "pub mod a;\npub mod b;\n");
    write(
        &src.join("a.rs"),
        "pub struct A;\npub fn make() -> A {\n    A\n}\n",
    );
    write(
        &src.join("b.rs"),
        "use crate::a::A;\npub fn take(_x: A) {}\n",
    );
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "init modules"]);

    // Revision 2: add more coupling so the structure changes over time.
    write(
        &src.join("b.rs"),
        "use crate::a::A;\npub fn take(_x: A) {}\npub fn more(_x: A) -> A {\n    A\n}\n",
    );
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "extend coupling"]);

    // Repeated changes make git volatility affect the latest snapshot score.
    for i in 0..11 {
        write(
            &src.join("a.rs"),
            &format!("pub struct A;\npub fn make() -> A {{\n    A\n}}\n// change {i}\n"),
        );
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "-m", "touch volatile module"]);
    }

    let config = CompiledConfig::empty();
    let thresholds = IssueThresholds::default();
    let report = analyze_history(&src, &config, &thresholds, 120, 5)
        .expect("history analysis should succeed on a real git repo");

    assert!(
        !report.points.is_empty(),
        "expected at least one analyzable revision, skipped: {:?}",
        report.skipped
    );
    assert!(report.points.len() <= 5, "must respect max_points");
    assert_eq!(
        report.months, 120,
        "report should carry the requested window"
    );

    // Worktrees registered against this repo must be cleaned up (only the main
    // worktree should remain). This is race-free because each test uses its own repo.
    let worktrees = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(root)
        .output()
        .expect("git worktree list");
    let worktree_count = String::from_utf8_lossy(&worktrees.stdout)
        .lines()
        .filter(|l| l.starts_with("worktree "))
        .count();
    assert_eq!(
        worktree_count, 1,
        "history worktrees must be cleaned up; found {worktree_count}"
    );

    for p in &report.points {
        assert!(!p.date.is_empty(), "each point must carry a date");
        assert!(p.module_count > 0, "analyzed revision should have modules");
        assert!(
            ['S', 'A', 'B', 'C', 'D', 'F'].contains(&p.grade.letter()),
            "grade must be a valid letter, got {}",
            p.grade.letter()
        );
        assert!(
            (0.0..=1.0).contains(&p.average_score),
            "score out of range: {}",
            p.average_score
        );
    }

    // Points are chronological (oldest first).
    if report.points.len() >= 2 {
        let first = &report.points[0];
        let last = report.points.last().unwrap();
        assert!(first.date <= last.date, "points must be chronological");
    }

    let mut snapshot_metrics = analyze_workspace_with_config(&src, &config).unwrap();
    let mut volatility = VolatilityAnalyzer::new(120);
    volatility.analyze(&src).unwrap();
    snapshot_metrics.file_changes = volatility.file_changes;
    snapshot_metrics.update_volatility_from_git();
    let snapshot_report = analyze_project_balance_with_thresholds(&snapshot_metrics, &thresholds);
    let latest = report.points.last().unwrap();

    assert_eq!(
        latest.grade, snapshot_report.health_grade,
        "latest history grade should match snapshot methodology"
    );
    assert!(
        (latest.average_score - snapshot_report.average_score).abs() < 1e-9,
        "latest history score {} should match snapshot score {}",
        latest.average_score,
        snapshot_report.average_score
    );
}

#[test]
fn history_errors_outside_git_repo() {
    // A fresh temp dir is not a git repository.
    let tmp = tempfile::tempdir().unwrap();
    let config = CompiledConfig::empty();
    let thresholds = IssueThresholds::default();

    let result = analyze_history(tmp.path(), &config, &thresholds, 6, 3);
    assert!(
        result.is_err(),
        "analysis outside a git repo should return an error"
    );
}
