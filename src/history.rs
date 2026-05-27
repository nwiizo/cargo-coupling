//! Time-series coupling analysis.
//!
//! Re-runs structural balance analysis at past git revisions using disposable
//! `git worktree`s, producing a chronological health timeline. This is the data
//! foundation for *observing* how a codebase's coupling evolves over time, rather
//! than inspecting a single snapshot.
//!
//! Each sampled revision is analyzed structurally only (AST + config overrides);
//! git-churn volatility is intentionally skipped so that points are comparable and
//! fast to compute. The score therefore reflects structural balance at that commit.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::balance::{HealthGrade, Severity, analyze_project_balance_with_thresholds};
use crate::config::CompiledConfig;
use crate::{IssueThresholds, analyze_workspace_with_config};

/// One sampled point in the coupling timeline.
#[derive(Debug, Clone)]
pub struct HistoryPoint {
    /// Abbreviated commit hash.
    pub commit: String,
    /// Committer date in ISO-8601 (e.g. `2026-05-20`).
    pub date: String,
    /// Overall project health grade at this revision.
    pub grade: HealthGrade,
    /// Average balance score (0.0 - 1.0).
    pub average_score: f64,
    /// Number of detected couplings.
    pub total_couplings: usize,
    /// Number of analyzed modules.
    pub module_count: usize,
    /// Critical-severity issue count.
    pub critical: usize,
    /// High-severity issue count.
    pub high: usize,
}

/// A revision that was sampled but could not be analyzed.
#[derive(Debug, Clone)]
pub struct SkippedRevision {
    pub commit: String,
    pub date: String,
    pub reason: String,
}

/// Result of a history analysis run.
#[derive(Debug, Default)]
pub struct HistoryReport {
    /// Points in chronological order (oldest first).
    pub points: Vec<HistoryPoint>,
    /// Revisions that were sampled but skipped, with reasons.
    pub skipped: Vec<SkippedRevision>,
    /// Look-back window in months that was requested.
    pub months: usize,
}

impl HistoryReport {
    /// Oldest and newest analyzed points, if any exist.
    pub fn endpoints(&self) -> Option<(&HistoryPoint, &HistoryPoint)> {
        match (self.points.first(), self.points.last()) {
            (Some(first), Some(last)) => Some((first, last)),
            _ => None,
        }
    }
}

/// Errors that can occur during history analysis.
#[derive(Debug)]
pub enum HistoryError {
    /// The analysis path is not inside a git repository.
    NotGitRepo,
    /// A git command failed.
    Git(String),
    /// An I/O error occurred.
    Io(std::io::Error),
    /// No commits were found in the requested window.
    NoCommits,
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryError::NotGitRepo => write!(f, "path is not inside a git repository"),
            HistoryError::Git(msg) => write!(f, "git command failed: {}", msg),
            HistoryError::Io(e) => write!(f, "I/O error: {}", e),
            HistoryError::NoCommits => write!(f, "no commits found in the requested time window"),
        }
    }
}

impl std::error::Error for HistoryError {}

impl From<std::io::Error> for HistoryError {
    fn from(e: std::io::Error) -> Self {
        HistoryError::Io(e)
    }
}

/// Analyze the coupling health of a project across its recent git history.
///
/// Samples up to `max_points` commits evenly across the last `months` months,
/// re-running structural analysis at each via a disposable worktree.
pub fn analyze_history(
    path: &Path,
    config: &CompiledConfig,
    thresholds: &IssueThresholds,
    months: usize,
    max_points: usize,
) -> Result<HistoryReport, HistoryError> {
    let repo_root = repo_root(path)?;
    let subpath = relative_subpath(&repo_root, path);

    let commits = list_commits(&repo_root, months)?;
    if commits.is_empty() {
        return Err(HistoryError::NoCommits);
    }

    let sampled = sample_evenly(commits.len(), max_points);

    let mut report = HistoryReport {
        months,
        ..Default::default()
    };

    for (idx, &i) in sampled.iter().enumerate() {
        let (commit, date) = &commits[i];
        match analyze_revision(&repo_root, &subpath, commit, config, thresholds, idx) {
            Ok(point) => report.points.push(HistoryPoint {
                date: date.clone(),
                ..point
            }),
            Err(reason) => report.skipped.push(SkippedRevision {
                commit: short_hash(commit),
                date: date.clone(),
                reason,
            }),
        }
    }

    Ok(report)
}

/// Resolve the git repository root containing `path`.
fn repo_root(path: &Path) -> Result<PathBuf, HistoryError> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Err(HistoryError::NotGitRepo);
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        return Err(HistoryError::NotGitRepo);
    }
    let root = PathBuf::from(root);
    // Canonicalize so it matches the canonicalized analysis path in
    // `relative_subpath` (e.g. macOS `/var` -> `/private/var`).
    Ok(root.canonicalize().unwrap_or(root))
}

/// Compute the analysis path relative to the repo root (for use inside a worktree).
fn relative_subpath(repo_root: &Path, path: &Path) -> PathBuf {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    abs.strip_prefix(repo_root)
        .map(|p| p.to_path_buf())
        .unwrap_or_default()
}

/// List `(hash, iso-date)` of `.rs`-touching commits in the window, oldest first.
fn list_commits(repo_root: &Path, months: usize) -> Result<Vec<(String, String)>, HistoryError> {
    let output = Command::new("git")
        .args([
            "log",
            "--reverse",
            "--pretty=format:%H|%cs",
            &format!("--since={} months ago", months),
            "--",
            "*.rs",
        ])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        return Err(HistoryError::Git(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    let commits = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let (hash, date) = line.split_once('|')?;
            if hash.is_empty() {
                return None;
            }
            Some((hash.to_string(), date.to_string()))
        })
        .collect();

    Ok(commits)
}

/// Analyze a single revision inside a disposable worktree.
fn analyze_revision(
    repo_root: &Path,
    subpath: &Path,
    commit: &str,
    config: &CompiledConfig,
    thresholds: &IssueThresholds,
    seq: usize,
) -> Result<HistoryPoint, String> {
    let worktree = Worktree::add(repo_root, commit, seq).map_err(|e| e.to_string())?;

    let analysis_path = worktree.dir.join(subpath);
    if !analysis_path.exists() {
        return Err("analysis path does not exist at this revision".to_string());
    }

    let metrics = analyze_workspace_with_config(&analysis_path, config)
        .map_err(|e| format!("analysis failed: {}", e))?;

    if metrics.modules.is_empty() {
        return Err("no modules found at this revision".to_string());
    }

    let report = analyze_project_balance_with_thresholds(&metrics, thresholds);
    let critical = *report
        .issues_by_severity
        .get(&Severity::Critical)
        .unwrap_or(&0);
    let high = *report.issues_by_severity.get(&Severity::High).unwrap_or(&0);

    Ok(HistoryPoint {
        commit: short_hash(commit),
        date: String::new(), // filled in by caller
        grade: report.health_grade,
        average_score: report.average_score,
        total_couplings: metrics.couplings.len(),
        module_count: metrics.modules.len(),
        critical,
        high,
    })
}

/// A git worktree that is removed when dropped.
struct Worktree {
    repo_root: PathBuf,
    dir: PathBuf,
}

impl Worktree {
    fn add(repo_root: &Path, commit: &str, seq: usize) -> Result<Self, HistoryError> {
        let dir = std::env::temp_dir().join(format!(
            "cargo-coupling-hist-{}-{}-{}",
            std::process::id(),
            seq,
            short_hash(commit)
        ));

        let output = Command::new("git")
            .args(["worktree", "add", "--detach", "--force"])
            .arg(&dir)
            .arg(commit)
            .current_dir(repo_root)
            .output()?;

        if !output.status.success() {
            return Err(HistoryError::Git(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        Ok(Worktree {
            repo_root: repo_root.to_path_buf(),
            dir,
        })
    }
}

impl Drop for Worktree {
    fn drop(&mut self) {
        let _ = Command::new("git")
            .args(["worktree", "remove", "--force"])
            .arg(&self.dir)
            .current_dir(&self.repo_root)
            .output();
    }
}

/// Abbreviate a commit hash to 7 characters.
fn short_hash(hash: &str) -> String {
    hash.chars().take(7).collect()
}

/// Pick up to `max` evenly-spaced indices from `[0, len)`.
///
/// Always includes the first and last index when `len >= 2` so the endpoints
/// of the timeline are preserved. Returns all indices when `len <= max`.
pub fn sample_evenly(len: usize, max: usize) -> Vec<usize> {
    // Covers the empty cases too: `len <= max` is true when `len == 0`, and the
    // loop below produces nothing when `max == 0`, so no separate guard is needed.
    if len <= max {
        return (0..len).collect();
    }
    if max == 1 {
        return vec![len - 1];
    }

    let mut indices = Vec::with_capacity(max);
    for k in 0..max {
        // Spread k across [0, len-1] inclusive.
        let idx = k * (len - 1) / (max - 1);
        if indices.last() != Some(&idx) {
            indices.push(idx);
        }
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_returns_all_when_under_limit() {
        assert_eq!(sample_evenly(3, 10), vec![0, 1, 2]);
        assert_eq!(sample_evenly(5, 5), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn sample_includes_endpoints() {
        let s = sample_evenly(100, 5);
        assert_eq!(s.first(), Some(&0));
        assert_eq!(s.last(), Some(&99));
        assert!(s.len() <= 5);
    }

    #[test]
    fn sample_is_monotonic_and_unique() {
        let s = sample_evenly(50, 8);
        for w in s.windows(2) {
            assert!(w[0] < w[1], "indices must be strictly increasing: {:?}", s);
        }
    }

    #[test]
    fn sample_edge_cases() {
        assert_eq!(sample_evenly(0, 5), Vec::<usize>::new());
        assert_eq!(sample_evenly(5, 0), Vec::<usize>::new());
        assert_eq!(sample_evenly(10, 1), vec![9]);
        assert_eq!(sample_evenly(1, 5), vec![0]);
    }

    #[test]
    fn short_hash_truncates() {
        assert_eq!(short_hash("0123456789abcdef"), "0123456");
        assert_eq!(short_hash("abc"), "abc");
    }

    #[test]
    fn error_display_messages() {
        assert_eq!(
            HistoryError::NotGitRepo.to_string(),
            "path is not inside a git repository"
        );
        assert_eq!(
            HistoryError::NoCommits.to_string(),
            "no commits found in the requested time window"
        );
        assert!(
            HistoryError::Git("boom".into())
                .to_string()
                .contains("boom")
        );
    }

    #[test]
    fn relative_subpath_strips_root() {
        // `/repo` does not exist, so canonicalize falls back to the path as-is
        // (`/repo/src`), and strip_prefix then yields the relative `src`.
        let sub = relative_subpath(Path::new("/repo"), Path::new("/repo/src"));
        assert_eq!(sub, Path::new("src"));
    }
}
