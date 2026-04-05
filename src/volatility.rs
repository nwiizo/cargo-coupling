//! Git history analysis for volatility measurement
//!
//! Analyzes git log to determine how frequently files change.
//! Optimized for large repositories using streaming and git path filtering.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use thiserror::Error;

use crate::metrics::Volatility;

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

    /// Analyze git history for a repository (optimized version)
    ///
    /// Optimizations applied:
    /// 1. Use `-- "*.rs"` to filter .rs files at git level
    /// 2. Use streaming with BufReader instead of loading all into memory
    /// 3. Use `--diff-filter=AMRC` to skip deleted files
    pub fn analyze(&mut self, repo_path: &Path) -> Result<(), VolatilityError> {
        // Check if it's a git repo
        let git_check = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(repo_path)
            .stderr(Stdio::null())
            .output()?;

        if !git_check.status.success() {
            return Err(VolatilityError::NotGitRepo);
        }

        // Optimized: use --diff-filter and path spec to reduce output
        // --diff-filter=AMRC: Added, Modified, Renamed, Copied (skip Deleted)
        let mut child = Command::new("git")
            .args([
                "log",
                "--pretty=format:",
                "--name-only",
                "--diff-filter=AMRC",
                &format!("--since={} months ago", self.period_months),
                "--",
                "*.rs",
            ])
            .current_dir(repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        // Stream processing with BufReader
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::with_capacity(64 * 1024, stdout); // 64KB buffer

            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => continue,
                };

                let line = line.trim();
                if !line.is_empty() && line.ends_with(".rs") {
                    *self.file_changes.entry(line.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Wait for git to finish
        let _ = child.wait();

        Ok(())
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
        // Get commit-grouped file changes
        let mut child = Command::new("git")
            .args([
                "log",
                "--pretty=format:__COMMIT__",
                "--name-only",
                "--diff-filter=AMRC",
                &format!("--since={} months ago", self.period_months),
                "--",
                "*.rs",
            ])
            .current_dir(repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let mut commits: Vec<Vec<String>> = Vec::new();
        let mut current_files: Vec<String> = Vec::new();

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::with_capacity(64 * 1024, stdout);
            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => continue,
                };
                let trimmed = line.trim();
                if trimmed == "__COMMIT__" {
                    if current_files.len() >= 2 {
                        commits.push(std::mem::take(&mut current_files));
                    } else {
                        current_files.clear();
                    }
                } else if !trimmed.is_empty() && trimmed.ends_with(".rs") {
                    current_files.push(trimmed.to_string());
                }
            }
            // Don't forget the last commit
            if current_files.len() >= 2 {
                commits.push(current_files);
            }
        }

        let _ = child.wait();

        // Count co-change frequency for each file pair
        // Skip commits with too many files (e.g., formatter runs, merge commits)
        // as they produce O(n²) noise rather than meaningful coupling signal
        const MAX_FILES_PER_COMMIT: usize = 50;
        let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();
        for files in &commits {
            if files.len() > MAX_FILES_PER_COMMIT {
                continue;
            }
            for i in 0..files.len() {
                for j in (i + 1)..files.len() {
                    let (a, b) = if files[i] < files[j] {
                        (files[i].clone(), files[j].clone())
                    } else {
                        (files[j].clone(), files[i].clone())
                    };
                    *pair_counts.entry((a, b)).or_default() += 1;
                }
            }
        }

        // Filter to significant co-changes (3+ times together)
        let mut result: Vec<TemporalCoupling> = pair_counts
            .into_iter()
            .filter(|(_, count)| *count >= 3)
            .map(|((file_a, file_b), count)| {
                let total_a = self.file_changes.get(&file_a).copied().unwrap_or(1);
                let total_b = self.file_changes.get(&file_b).copied().unwrap_or(1);
                let coupling_ratio = count as f64 / total_a.min(total_b).max(1) as f64;
                TemporalCoupling {
                    file_a,
                    file_b,
                    co_change_count: count,
                    coupling_ratio: coupling_ratio.min(1.0),
                }
            })
            .collect();

        result.sort_by(|a, b| {
            b.co_change_count.cmp(&a.co_change_count).then(
                b.coupling_ratio
                    .partial_cmp(&a.coupling_ratio)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });
        Ok(result)
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
    pub total_files: usize,
    pub total_changes: usize,
    pub max_changes: usize,
    pub min_changes: usize,
    pub avg_changes: f64,
    pub low_volatility_count: usize,
    pub medium_volatility_count: usize,
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
