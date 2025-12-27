//! Configuration file support for cargo-coupling
//!
//! This module handles parsing and applying `.coupling.toml` configuration files
//! that allow users to override volatility predictions and customize analysis.
//!
//! ## Configuration File Format
//!
//! ```toml
//! # .coupling.toml
//!
//! [analysis]
//! # Exclude test code (#[test], #[cfg(test)], mod tests) from analysis
//! exclude_tests = true
//!
//! # "Prelude-like" modules that are expected to be used by many other modules.
//! # These modules will not trigger "High Afferent Coupling" warnings.
//! prelude_modules = ["src/lib.rs", "src/prelude.rs", "src/core/*"]
//!
//! # Modules to completely exclude from analysis
//! exclude = ["src/generated/*", "src/test_utils/*"]
//!
//! [volatility]
//! # Modules expected to change frequently (High volatility)
//! high = ["src/business_rules/*", "src/pricing/*"]
//!
//! # Stable modules (Low volatility)
//! low = ["src/core/*", "src/contracts/*"]
//!
//! # Paths to ignore from analysis (deprecated: use [analysis].exclude instead)
//! ignore = ["src/generated/*", "tests/*"]
//!
//! [thresholds]
//! # Maximum dependencies before flagging High Efferent Coupling
//! max_dependencies = 15
//!
//! # Maximum dependents before flagging High Afferent Coupling
//! max_dependents = 20
//! ```

use glob::Pattern;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::metrics::Volatility;

/// Errors that can occur when loading configuration
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Invalid glob pattern: {0}")]
    PatternError(String),
}

/// Analysis configuration section
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AnalysisConfig {
    /// Exclude test code from analysis (#[test], #[cfg(test)], mod tests)
    #[serde(default)]
    pub exclude_tests: bool,

    /// "Prelude-like" modules that are expected to be depended on by many modules.
    /// These modules will not trigger "High Afferent Coupling" warnings.
    #[serde(default)]
    pub prelude_modules: Vec<String>,

    /// Modules to completely exclude from analysis
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Volatility configuration section
#[derive(Debug, Clone, Deserialize, Default)]
pub struct VolatilityConfig {
    /// Paths that should be considered high volatility
    #[serde(default)]
    pub high: Vec<String>,

    /// Paths that should be considered medium volatility
    #[serde(default)]
    pub medium: Vec<String>,

    /// Paths that should be considered low volatility
    #[serde(default)]
    pub low: Vec<String>,

    /// Paths to ignore from analysis
    #[serde(default)]
    pub ignore: Vec<String>,
}

/// Threshold configuration section
#[derive(Debug, Clone, Deserialize)]
pub struct ThresholdsConfig {
    /// Maximum dependencies before flagging High Efferent Coupling
    #[serde(default = "default_max_dependencies")]
    pub max_dependencies: usize,

    /// Maximum dependents before flagging High Afferent Coupling
    #[serde(default = "default_max_dependents")]
    pub max_dependents: usize,
}

fn default_max_dependencies() -> usize {
    15
}

fn default_max_dependents() -> usize {
    20
}

impl Default for ThresholdsConfig {
    fn default() -> Self {
        Self {
            max_dependencies: default_max_dependencies(),
            max_dependents: default_max_dependents(),
        }
    }
}

/// Root configuration structure
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CouplingConfig {
    /// Analysis configuration (test exclusion, prelude modules, etc.)
    #[serde(default)]
    pub analysis: AnalysisConfig,

    /// Volatility override configuration
    #[serde(default)]
    pub volatility: VolatilityConfig,

    /// Threshold configuration
    #[serde(default)]
    pub thresholds: ThresholdsConfig,
}

/// Compiled configuration with glob patterns
#[derive(Debug)]
pub struct CompiledConfig {
    // === Analysis settings ===
    /// Whether to exclude test code from analysis
    pub exclude_tests: bool,
    /// Patterns for prelude-like modules (exempt from afferent coupling warnings)
    prelude_patterns: Vec<Pattern>,
    /// Patterns for modules to completely exclude from analysis
    exclude_patterns: Vec<Pattern>,

    // === Volatility settings ===
    /// Patterns for high volatility paths
    high_patterns: Vec<Pattern>,
    /// Patterns for medium volatility paths
    medium_patterns: Vec<Pattern>,
    /// Patterns for low volatility paths
    low_patterns: Vec<Pattern>,
    /// Patterns for ignored paths (deprecated, use exclude_patterns)
    ignore_patterns: Vec<Pattern>,

    // === Thresholds ===
    /// Threshold configuration
    pub thresholds: ThresholdsConfig,

    // === Cache ===
    /// Cache of path -> volatility mappings
    cache: HashMap<String, Option<Volatility>>,
}

impl CompiledConfig {
    /// Create a compiled config from raw config
    pub fn from_config(config: CouplingConfig) -> Result<Self, ConfigError> {
        let compile_patterns = |patterns: &[String]| -> Result<Vec<Pattern>, ConfigError> {
            patterns
                .iter()
                .map(|p| {
                    Pattern::new(p).map_err(|e| ConfigError::PatternError(format!("{}: {}", p, e)))
                })
                .collect()
        };

        Ok(Self {
            // Analysis settings
            exclude_tests: config.analysis.exclude_tests,
            prelude_patterns: compile_patterns(&config.analysis.prelude_modules)?,
            exclude_patterns: compile_patterns(&config.analysis.exclude)?,
            // Volatility settings
            high_patterns: compile_patterns(&config.volatility.high)?,
            medium_patterns: compile_patterns(&config.volatility.medium)?,
            low_patterns: compile_patterns(&config.volatility.low)?,
            ignore_patterns: compile_patterns(&config.volatility.ignore)?,
            // Thresholds
            thresholds: config.thresholds,
            cache: HashMap::new(),
        })
    }

    /// Create an empty config (no overrides)
    pub fn empty() -> Self {
        Self {
            exclude_tests: false,
            prelude_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            high_patterns: Vec::new(),
            medium_patterns: Vec::new(),
            low_patterns: Vec::new(),
            ignore_patterns: Vec::new(),
            thresholds: ThresholdsConfig::default(),
            cache: HashMap::new(),
        }
    }

    /// Set exclude_tests flag (used by CLI --exclude-tests option)
    pub fn set_exclude_tests(&mut self, exclude: bool) {
        self.exclude_tests = exclude;
    }

    /// Check if a module is marked as "prelude-like" (exempt from afferent coupling warnings)
    pub fn is_prelude_module(&self, path: &str) -> bool {
        self.prelude_patterns.iter().any(|p| p.matches(path))
    }

    /// Check if a path should be completely excluded from analysis
    pub fn should_exclude(&self, path: &str) -> bool {
        self.exclude_patterns.iter().any(|p| p.matches(path))
    }

    /// Check if a path should be ignored (deprecated: use should_exclude)
    pub fn should_ignore(&self, path: &str) -> bool {
        self.ignore_patterns.iter().any(|p| p.matches(path))
            || self.exclude_patterns.iter().any(|p| p.matches(path))
    }

    /// Get the list of prelude module patterns (for reporting)
    pub fn prelude_module_count(&self) -> usize {
        self.prelude_patterns.len()
    }

    /// Get overridden volatility for a path, if any
    pub fn get_volatility_override(&mut self, path: &str) -> Option<Volatility> {
        // Check cache first
        if let Some(cached) = self.cache.get(path) {
            return *cached;
        }

        // Check patterns in order of specificity (high > medium > low)
        let result = if self.high_patterns.iter().any(|p| p.matches(path)) {
            Some(Volatility::High)
        } else if self.medium_patterns.iter().any(|p| p.matches(path)) {
            Some(Volatility::Medium)
        } else if self.low_patterns.iter().any(|p| p.matches(path)) {
            Some(Volatility::Low)
        } else {
            None
        };

        // Cache the result
        self.cache.insert(path.to_string(), result);
        result
    }

    /// Get volatility with override, falling back to git-based value
    pub fn get_volatility(&mut self, path: &str, git_volatility: Volatility) -> Volatility {
        self.get_volatility_override(path).unwrap_or(git_volatility)
    }

    /// Check if config has any volatility overrides
    pub fn has_volatility_overrides(&self) -> bool {
        !self.high_patterns.is_empty()
            || !self.medium_patterns.is_empty()
            || !self.low_patterns.is_empty()
    }
}

/// Load configuration from the project directory
///
/// Searches for `.coupling.toml` in the given directory and parent directories.
pub fn load_config(project_path: &Path) -> Result<CouplingConfig, ConfigError> {
    // Search for config file
    let config_path = find_config_file(project_path);

    match config_path {
        Some(path) => {
            let content = fs::read_to_string(&path)?;
            let config: CouplingConfig = toml::from_str(&content)?;
            Ok(config)
        }
        None => Ok(CouplingConfig::default()),
    }
}

/// Find the config file by searching up the directory tree
fn find_config_file(start_path: &Path) -> Option<std::path::PathBuf> {
    let config_names = [".coupling.toml", "coupling.toml"];

    let mut current = if start_path.is_file() {
        start_path.parent()?.to_path_buf()
    } else {
        start_path.to_path_buf()
    };

    loop {
        for name in &config_names {
            let config_path = current.join(name);
            if config_path.exists() {
                return Some(config_path);
            }
        }

        // Move to parent directory
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    None
}

/// Load and compile configuration
pub fn load_compiled_config(project_path: &Path) -> Result<CompiledConfig, ConfigError> {
    let config = load_config(project_path)?;
    CompiledConfig::from_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CouplingConfig::default();
        assert!(config.volatility.high.is_empty());
        assert!(config.volatility.low.is_empty());
        assert_eq!(config.thresholds.max_dependencies, 15);
        assert_eq!(config.thresholds.max_dependents, 20);
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
            [volatility]
            high = ["src/api/*", "src/handlers/*"]
            low = ["src/core/*"]
            ignore = ["tests/*"]

            [thresholds]
            max_dependencies = 20
            max_dependents = 30
        "#;

        let config: CouplingConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.volatility.high.len(), 2);
        assert_eq!(config.volatility.low.len(), 1);
        assert_eq!(config.volatility.ignore.len(), 1);
        assert_eq!(config.thresholds.max_dependencies, 20);
        assert_eq!(config.thresholds.max_dependents, 30);
    }

    #[test]
    fn test_compiled_config() {
        let toml = r#"
            [volatility]
            high = ["src/business/*"]
            low = ["src/core/*"]
        "#;

        let config: CouplingConfig = toml::from_str(toml).unwrap();
        let mut compiled = CompiledConfig::from_config(config).unwrap();

        assert_eq!(
            compiled.get_volatility_override("src/business/pricing.rs"),
            Some(Volatility::High)
        );
        assert_eq!(
            compiled.get_volatility_override("src/core/types.rs"),
            Some(Volatility::Low)
        );
        assert_eq!(compiled.get_volatility_override("src/other/file.rs"), None);
    }

    #[test]
    fn test_ignore_patterns() {
        let toml = r#"
            [volatility]
            ignore = ["tests/*", "benches/*"]
        "#;

        let config: CouplingConfig = toml::from_str(toml).unwrap();
        let compiled = CompiledConfig::from_config(config).unwrap();

        assert!(compiled.should_ignore("tests/integration.rs"));
        assert!(compiled.should_ignore("benches/perf.rs"));
        assert!(!compiled.should_ignore("src/lib.rs"));
    }

    #[test]
    fn test_get_volatility_with_fallback() {
        let toml = r#"
            [volatility]
            high = ["src/api/*"]
        "#;

        let config: CouplingConfig = toml::from_str(toml).unwrap();
        let mut compiled = CompiledConfig::from_config(config).unwrap();

        // Override wins
        assert_eq!(
            compiled.get_volatility("src/api/handler.rs", Volatility::Low),
            Volatility::High
        );

        // Fallback to git volatility
        assert_eq!(
            compiled.get_volatility("src/other/file.rs", Volatility::Medium),
            Volatility::Medium
        );
    }
}
