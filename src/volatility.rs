//! Git history analysis for volatility measurement
//!
//! Analyzes git log to determine how frequently files change.
//! Optimized for large repositories using streaming and git path filtering.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use thiserror::Error;

/// Volatility levels (how often a component changes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Volatility {
    /// Rarely changes (0-2 times)
    Low,
    /// Sometimes changes (3-10 times)
    Medium,
    /// Frequently changes (11+ times)
    High,
}

impl Volatility {
    /// Returns the numeric value (0.0 - 1.0, higher = more volatile)
    pub fn value(&self) -> f64 {
        match self {
            Volatility::Low => 0.0,
            Volatility::Medium => 0.5,
            Volatility::High => 1.0,
        }
    }

    /// Classify from change count
    pub fn from_count(count: usize) -> Self {
        match count {
            0..=2 => Volatility::Low,
            3..=10 => Volatility::Medium,
            _ => Volatility::High,
        }
    }
}

/// Errors that can occur during volatility analysis
#[derive(Error, Debug)]
pub enum VolatilityError {
    #[error("Failed to execute git command: {0}")]
    GitCommand(#[from] std::io::Error),

    #[error("Invalid UTF-8 in git output: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("Not a git repository")]
    NotGitRepo,
}

#[derive(Default)]
struct GitChanges {
    file_changes: HashMap<String, usize>,
    pair_counts: HashMap<(String, String), usize>,
}

impl GitChanges {
    fn read(reader: impl BufRead, count_pairs: bool) -> Result<Self, VolatilityError> {
        const MAX_FILES_PER_COMMIT: usize = 50;
        let mut history = Self::default();
        let mut files = Vec::new();
        let mut first_path = false;
        for record in reader.split(0) {
            let record = record?;
            if record == b"COMMIT" {
                history.count_pairs(&files, MAX_FILES_PER_COMMIT);
                files.clear();
                first_path = true;
            } else if !record.is_empty() {
                // Git separates the pretty header and its first pathname with a newline.
                let mut file = String::from_utf8(record)?;
                if first_path && file.starts_with('\n') {
                    file.remove(0);
                }
                first_path = false;
                if file.ends_with(".rs") {
                    if count_pairs && files.len() <= MAX_FILES_PER_COMMIT {
                        files.push(file.clone());
                    }
                    *history.file_changes.entry(file).or_default() += 1;
                }
            }
        }
        history.count_pairs(&files, MAX_FILES_PER_COMMIT);
        Ok(history)
    }

    fn count_pairs(&mut self, files: &[String], limit: usize) {
        if files.len() > limit {
            return;
        }
        for (i, left) in files.iter().enumerate() {
            for right in &files[i + 1..] {
                let pair = (left.min(right).clone(), left.max(right).clone());
                *self.pair_counts.entry(pair).or_default() += 1;
            }
        }
    }

    fn co_changes(&self, file_changes: &HashMap<String, usize>) -> Vec<TemporalCoupling> {
        // Filter to significant co-changes (3+ times together)
        let mut result: Vec<TemporalCoupling> = self
            .pair_counts
            .iter()
            .filter(|(_, count)| **count >= 3)
            .map(|((file_a, file_b), &count)| {
                let total_a = file_changes.get(file_a).copied().unwrap_or(1);
                let total_b = file_changes.get(file_b).copied().unwrap_or(1);
                let coupling_ratio = count as f64 / total_a.min(total_b).max(1) as f64;
                TemporalCoupling {
                    file_a: file_a.clone(),
                    file_b: file_b.clone(),
                    co_change_count: count,
                    coupling_ratio: coupling_ratio.min(1.0),
                }
            })
            .collect();

        result.sort_by(|a, b| {
            b.co_change_count
                .cmp(&a.co_change_count)
                .then_with(|| {
                    b.coupling_ratio
                        .partial_cmp(&a.coupling_ratio)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| a.file_a.cmp(&b.file_a))
                .then_with(|| a.file_b.cmp(&b.file_b))
        });
        result
    }
}

/// Volatility analyzer using git history
#[derive(Debug, Default)]
pub struct VolatilityAnalyzer {
    /// File path -> change count
    pub file_changes: HashMap<String, usize>,
    /// Analysis period in months
    pub period_months: usize,
}

impl VolatilityAnalyzer {
    /// Create a new volatility analyzer
    pub fn new(period_months: usize) -> Self {
        Self {
            file_changes: HashMap::new(),
            period_months,
        }
    }

    /// Refresh counts from the selected Git history, replacing any previous snapshot.
    pub fn analyze(&mut self, repo_path: &Path) -> Result<(), VolatilityError> {
        self.file_changes = self.read_history(repo_path, false)?.file_changes;
        Ok(())
    }

    /// Read change counts and co-changes from the same streaming Git log.
    pub fn analyze_with_temporal_coupling(
        &mut self,
        path: &Path,
    ) -> Result<Vec<TemporalCoupling>, VolatilityError> {
        let history = self.read_history(path, true)?;
        let temporal = history.co_changes(&history.file_changes);
        self.file_changes = history.file_changes;
        Ok(temporal)
    }

    fn read_history(&self, path: &Path, count_pairs: bool) -> Result<GitChanges, VolatilityError> {
        let directory = crate::paths::containing_directory(path);
        if !Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(directory)
            .output()?
            .status
            .success()
        {
            return Err(VolatilityError::NotGitRepo);
        }
        let mut command = Command::new("git");
        command
            .current_dir(directory)
            .args([
                "log",
                "--pretty=tformat:COMMIT",
                "--name-only",
                "-z",
                "--diff-filter=AMRC",
            ])
            .arg(format!("--since={} months ago", self.period_months))
            .arg("--");
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            let mut spec = std::ffi::OsString::from(":(literal)");
            spec.push(path.file_name().expect("source file has a name"));
            command.arg(spec);
        } else {
            command.arg("*.rs");
        }
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdout = child.stdout.take().expect("piped Git output");
        let parsed = GitChanges::read(BufReader::with_capacity(64 * 1024, stdout), count_pairs);
        let status = child.wait()?;
        let history = parsed?;
        if !status.success() {
            return Err(std::io::Error::other(format!("git log exited with {status}")).into());
        }
        Ok(history)
    }

    /// Get volatility level for a file
    pub fn get_volatility(&self, file_path: &str) -> Volatility {
        let count = self.file_changes.get(file_path).copied().unwrap_or(0);
        Volatility::from_count(count)
    }

    /// Get change count for a file
    pub fn get_change_count(&self, file_path: &str) -> usize {
        self.file_changes.get(file_path).copied().unwrap_or(0)
    }

    /// Get all high volatility files
    pub fn high_volatility_files(&self) -> Vec<(&String, usize)> {
        self.file_changes
            .iter()
            .filter(|&(_, count)| *count > 10)
            .map(|(path, count)| (path, *count))
            .collect()
    }

    /// Analyze temporal coupling (co-change patterns) from git history
    ///
    /// Detects files that frequently change together in the same commit,
    /// indicating implicit coupling that AST analysis cannot detect.
    /// Based on Khononov's modularity model: co-changing files suggest
    /// shared knowledge even without explicit code dependencies.
    pub fn analyze_temporal_coupling(
        &self,
        repo_path: &Path,
    ) -> Result<Vec<TemporalCoupling>, VolatilityError> {
        Ok(self
            .read_history(repo_path, true)?
            .co_changes(&self.file_changes))
    }

    /// Get volatility statistics
    pub fn statistics(&self) -> VolatilityStats {
        if self.file_changes.is_empty() {
            return VolatilityStats::default();
        }

        let counts: Vec<usize> = self.file_changes.values().copied().collect();
        let total: usize = counts.iter().sum();
        let max = counts.iter().max().copied().unwrap_or(0);
        let min = counts.iter().min().copied().unwrap_or(0);
        let avg = total as f64 / counts.len() as f64;

        let low_count = counts.iter().filter(|&&c| c <= 2).count();
        let medium_count = counts.iter().filter(|&&c| c > 2 && c <= 10).count();
        let high_count = counts.iter().filter(|&&c| c > 10).count();

        VolatilityStats {
            total_files: counts.len(),
            total_changes: total,
            max_changes: max,
            min_changes: min,
            avg_changes: avg,
            low_volatility_count: low_count,
            medium_volatility_count: medium_count,
            high_volatility_count: high_count,
        }
    }
}

/// Temporal coupling between two files (co-change pattern)
///
/// Represents files that frequently change together in git commits,
/// indicating implicit coupling beyond what code structure reveals.
#[derive(Debug, Clone)]
pub struct TemporalCoupling {
    /// First file in the pair
    pub file_a: String,
    /// Second file in the pair
    pub file_b: String,
    /// Number of commits where both files changed together
    pub co_change_count: usize,
    /// Ratio of co-changes to total changes of the less-changed file (0.0-1.0)
    pub coupling_ratio: f64,
}

impl TemporalCoupling {
    /// Whether this represents strong temporal coupling (>50% co-change ratio)
    pub fn is_strong(&self) -> bool {
        self.coupling_ratio >= 0.5
    }
}

/// Statistics about volatility across the project
#[derive(Debug, Default)]
pub struct VolatilityStats {
    /// Number of Rust files with git change data.
    pub total_files: usize,
    /// Total observed file changes across the analysis window.
    pub total_changes: usize,
    /// Highest change count for one file.
    pub max_changes: usize,
    /// Lowest change count for one file.
    pub min_changes: usize,
    /// Average change count per file.
    pub avg_changes: f64,
    /// Number of files classified as low volatility.
    pub low_volatility_count: usize,
    /// Number of files classified as medium volatility.
    pub medium_volatility_count: usize,
    /// Number of files classified as high volatility.
    pub high_volatility_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volatility_classification() {
        let mut analyzer = VolatilityAnalyzer::new(6);
        analyzer.file_changes.insert("stable.rs".to_string(), 1);
        analyzer.file_changes.insert("moderate.rs".to_string(), 5);
        analyzer.file_changes.insert("volatile.rs".to_string(), 15);

        assert_eq!(analyzer.get_volatility("stable.rs"), Volatility::Low);
        assert_eq!(analyzer.get_volatility("moderate.rs"), Volatility::Medium);
        assert_eq!(analyzer.get_volatility("volatile.rs"), Volatility::High);
        assert_eq!(analyzer.get_volatility("unknown.rs"), Volatility::Low);
    }

    #[test]
    fn test_high_volatility_files() {
        let mut analyzer = VolatilityAnalyzer::new(6);
        analyzer.file_changes.insert("stable.rs".to_string(), 2);
        analyzer.file_changes.insert("volatile.rs".to_string(), 15);
        analyzer
            .file_changes
            .insert("very_volatile.rs".to_string(), 25);

        let high_vol = analyzer.high_volatility_files();
        assert_eq!(high_vol.len(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut analyzer = VolatilityAnalyzer::new(6);
        analyzer.file_changes.insert("a.rs".to_string(), 1);
        analyzer.file_changes.insert("b.rs".to_string(), 5);
        analyzer.file_changes.insert("c.rs".to_string(), 15);

        let stats = analyzer.statistics();
        assert_eq!(stats.total_files, 3);
        assert_eq!(stats.total_changes, 21);
        assert_eq!(stats.max_changes, 15);
        assert_eq!(stats.min_changes, 1);
        assert_eq!(stats.low_volatility_count, 1);
        assert_eq!(stats.medium_volatility_count, 1);
        assert_eq!(stats.high_volatility_count, 1);
    }

    #[test]
    fn test_temporal_coupling_is_strong() {
        let strong = TemporalCoupling {
            file_a: "a.rs".to_string(),
            file_b: "b.rs".to_string(),
            co_change_count: 10,
            coupling_ratio: 0.8,
        };
        assert!(strong.is_strong());

        let exactly_threshold = TemporalCoupling {
            file_a: "a.rs".to_string(),
            file_b: "b.rs".to_string(),
            co_change_count: 5,
            coupling_ratio: 0.5,
        };
        assert!(exactly_threshold.is_strong());

        let weak = TemporalCoupling {
            file_a: "a.rs".to_string(),
            file_b: "b.rs".to_string(),
            co_change_count: 3,
            coupling_ratio: 0.3,
        };
        assert!(!weak.is_strong());
    }
}
