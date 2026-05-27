//! External dependency coupling analysis.
//!
//! This module aggregates `DifferentCrate` coupling records into a snapshot of
//! how broadly third-party crates are used across internal modules.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::balance::action::RefactoringAction;
use crate::balance::issue::CouplingIssue;
use crate::balance::issue_type::IssueType;
use crate::balance::severity::Severity;
use crate::metrics::coupling::CouplingMetrics;
use crate::metrics::dimensions::{Distance, IntegrationStrength};
use crate::metrics::project::ProjectMetrics;

/// Number of internal modules above which direct third-party usage is considered scattered.
pub const SCATTERED_EXTERNAL_BREADTH_THRESHOLD: usize = 3;

/// External dependency coupling report.
#[derive(Debug, Clone)]
pub struct ExternalDependencyReport {
    /// External crates sorted by breadth, then references.
    pub dependencies: Vec<ExternalDependencyUsage>,
    /// Scattered external coupling findings.
    pub scattered_couplings: Vec<CouplingIssue>,
}

/// Aggregated usage of one external crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExternalDependencyUsage {
    /// Crate name.
    pub crate_name: String,
    /// Resolved versions from Cargo.lock, when available.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<String>,
    /// Number of distinct internal modules with direct coupling to this crate.
    pub breadth: usize,
    /// Total direct references observed in coupling metrics.
    pub total_references: usize,
    /// Most common integration strength, breaking ties toward stronger coupling.
    pub dominant_strength: String,
    /// Internal modules with direct coupling to this crate.
    pub source_modules: Vec<String>,
}

#[derive(Debug, Default)]
struct ExternalAccumulator {
    source_modules: BTreeSet<String>,
    total_references: usize,
    strength_counts: [usize; 4],
}

impl ExternalAccumulator {
    fn add(&mut self, coupling: &CouplingMetrics) {
        self.source_modules.insert(coupling.source.clone());
        self.total_references += 1;
        self.strength_counts[strength_index(coupling.strength)] += 1;
    }

    fn dominant_strength(&self) -> IntegrationStrength {
        let (idx, _) = self
            .strength_counts
            .iter()
            .enumerate()
            .max_by(|(left_idx, left_count), (right_idx, right_count)| {
                left_count
                    .cmp(right_count)
                    .then_with(|| strength_rank(*left_idx).cmp(&strength_rank(*right_idx)))
            })
            .unwrap_or((0, &0));
        strength_from_index(idx)
    }
}

/// Analyze external dependency coupling using optional lockfile versions.
pub fn analyze_external_dependencies(
    metrics: &ProjectMetrics,
    versions: &HashMap<String, Vec<String>>,
) -> ExternalDependencyReport {
    let known_external_crates = known_external_crates(metrics, versions);
    let mut by_crate: BTreeMap<String, ExternalAccumulator> = BTreeMap::new();

    for coupling in metrics
        .couplings
        .iter()
        .filter(|coupling| coupling.distance == Distance::DifferentCrate)
    {
        let crate_name = external_crate_name(coupling);
        if !known_external_crates.is_empty()
            && !known_external_crates.contains(&normalize_crate_name(&crate_name))
        {
            continue;
        }
        by_crate.entry(crate_name).or_default().add(coupling);
    }

    let mut dependencies: Vec<_> = by_crate
        .into_iter()
        .map(|(crate_name, accumulator)| {
            let dominant_strength = accumulator.dominant_strength();
            let source_modules: Vec<_> = accumulator.source_modules.into_iter().collect();
            let mut crate_versions = versions.get(&crate_name).cloned().unwrap_or_default();
            crate_versions.sort();
            crate_versions.dedup();

            ExternalDependencyUsage {
                crate_name,
                versions: crate_versions,
                breadth: source_modules.len(),
                total_references: accumulator.total_references,
                dominant_strength: strength_label(dominant_strength).to_string(),
                source_modules,
            }
        })
        .collect();

    dependencies.sort_by(|left, right| {
        right
            .breadth
            .cmp(&left.breadth)
            .then_with(|| right.total_references.cmp(&left.total_references))
            .then_with(|| left.crate_name.cmp(&right.crate_name))
    });

    let scattered_couplings = detect_scattered_external_coupling(&dependencies);

    ExternalDependencyReport {
        dependencies,
        scattered_couplings,
    }
}

fn known_external_crates(
    metrics: &ProjectMetrics,
    versions: &HashMap<String, Vec<String>>,
) -> BTreeSet<String> {
    let mut crates: BTreeSet<String> = metrics
        .crate_dependencies
        .values()
        .flatten()
        .map(|name| normalize_crate_name(name))
        .collect();

    if crates.is_empty() {
        crates.extend(versions.keys().map(|name| normalize_crate_name(name)));
    }

    crates
}

/// Detect external crates used directly from many internal modules.
pub fn detect_scattered_external_coupling(
    dependencies: &[ExternalDependencyUsage],
) -> Vec<CouplingIssue> {
    dependencies
        .iter()
        .filter(|dependency| dependency.breadth > SCATTERED_EXTERNAL_BREADTH_THRESHOLD)
        .map(|dependency| {
            let severity = scattered_severity(dependency.breadth);
            CouplingIssue {
                issue_type: IssueType::ScatteredExternalCoupling,
                severity,
                source: format!("{} internal modules", dependency.breadth),
                target: dependency.crate_name.clone(),
                description: format!(
                    "{} is directly used from {} internal modules ({} references). Third-party upgrade risk is spread across the codebase.",
                    dependency.crate_name, dependency.breadth, dependency.total_references
                ),
                refactoring: RefactoringAction::General {
                    action: format!(
                        "Introduce a `{}` facade/wrapper module and route direct crate usage through it",
                        facade_module_name(&dependency.crate_name)
                    ),
                },
                balance_score: 1.0 - (dependency.breadth as f64 / 12.0).min(1.0),
            }
        })
        .collect()
}

/// Read resolved package versions from the nearest Cargo.lock. Missing or
/// unparsable lockfiles are treated as absent.
pub fn load_lock_versions_near(start: &Path) -> HashMap<String, Vec<String>> {
    let Some(lock_path) = find_cargo_lock(start) else {
        return HashMap::new();
    };
    let Ok(content) = fs::read_to_string(lock_path) else {
        return HashMap::new();
    };
    parse_lock_versions(&content).unwrap_or_default()
}

fn parse_lock_versions(content: &str) -> Option<HashMap<String, Vec<String>>> {
    let lock: CargoLock = toml::from_str(content).ok()?;
    let mut versions: HashMap<String, Vec<String>> = HashMap::new();

    for package in lock.package {
        versions
            .entry(package.name)
            .or_default()
            .push(package.version);
    }

    for package_versions in versions.values_mut() {
        package_versions.sort();
        package_versions.dedup();
    }

    Some(versions)
}

#[derive(Debug, Deserialize)]
struct CargoLock {
    package: Vec<CargoLockPackage>,
}

#[derive(Debug, Deserialize)]
struct CargoLockPackage {
    name: String,
    version: String,
}

fn find_cargo_lock(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent().map(Path::to_path_buf)
    } else {
        Some(start.to_path_buf())
    };

    while let Some(dir) = current {
        let lock_path = dir.join("Cargo.lock");
        if lock_path.exists() {
            return Some(lock_path);
        }
        current = dir.parent().map(Path::to_path_buf);
    }

    None
}

fn external_crate_name(coupling: &CouplingMetrics) -> String {
    coupling.target_crate.clone().unwrap_or_else(|| {
        coupling
            .target
            .split("::")
            .next()
            .unwrap_or(&coupling.target)
            .to_string()
    })
}

fn normalize_crate_name(crate_name: &str) -> String {
    crate_name.replace('-', "_")
}

fn scattered_severity(breadth: usize) -> Severity {
    if breadth >= 10 {
        Severity::Critical
    } else if breadth >= 6 {
        Severity::High
    } else {
        Severity::Medium
    }
}

fn facade_module_name(crate_name: &str) -> String {
    format!("{}_facade", crate_name.replace('-', "_"))
}

fn strength_index(strength: IntegrationStrength) -> usize {
    match strength {
        IntegrationStrength::Contract => 0,
        IntegrationStrength::Model => 1,
        IntegrationStrength::Functional => 2,
        IntegrationStrength::Intrusive => 3,
    }
}

fn strength_rank(index: usize) -> usize {
    index
}

fn strength_from_index(index: usize) -> IntegrationStrength {
    match index {
        0 => IntegrationStrength::Contract,
        1 => IntegrationStrength::Model,
        2 => IntegrationStrength::Functional,
        _ => IntegrationStrength::Intrusive,
    }
}

fn strength_label(strength: IntegrationStrength) -> &'static str {
    match strength {
        IntegrationStrength::Intrusive => "Intrusive",
        IntegrationStrength::Functional => "Functional",
        IntegrationStrength::Model => "Model",
        IntegrationStrength::Contract => "Contract",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::coupling::CouplingMetrics;
    use crate::volatility::Volatility;

    fn external_coupling(
        source: &str,
        target: &str,
        strength: IntegrationStrength,
    ) -> CouplingMetrics {
        let mut coupling = CouplingMetrics::new(
            source.to_string(),
            format!("{target}::Item"),
            strength,
            Distance::DifferentCrate,
            Volatility::Low,
        );
        coupling.target_crate = Some(target.to_string());
        coupling
    }

    #[test]
    fn aggregation_counts_breadth_per_external_crate() {
        let mut metrics = ProjectMetrics::new();
        metrics.add_coupling(external_coupling(
            "app::a",
            "serde",
            IntegrationStrength::Model,
        ));
        metrics.add_coupling(external_coupling(
            "app::a",
            "serde",
            IntegrationStrength::Functional,
        ));
        metrics.add_coupling(external_coupling(
            "app::b",
            "serde",
            IntegrationStrength::Functional,
        ));
        metrics.add_coupling(external_coupling(
            "app::c",
            "clap",
            IntegrationStrength::Model,
        ));
        metrics.add_coupling(CouplingMetrics::new(
            "app::a".to_string(),
            "app::internal".to_string(),
            IntegrationStrength::Model,
            Distance::DifferentModule,
            Volatility::Low,
        ));

        let mut versions = HashMap::new();
        versions.insert("serde".to_string(), vec!["1.0.0".to_string()]);
        let report = analyze_external_dependencies(&metrics, &versions);

        let serde = report
            .dependencies
            .iter()
            .find(|dependency| dependency.crate_name == "serde")
            .unwrap();
        assert_eq!(serde.breadth, 2);
        assert_eq!(serde.total_references, 3);
        assert_eq!(serde.dominant_strength, "Functional");
        assert_eq!(serde.versions, vec!["1.0.0"]);
        assert_eq!(serde.source_modules, vec!["app::a", "app::b"]);
    }

    #[test]
    fn aggregation_filters_unresolved_type_names_when_dependency_inventory_exists() {
        let mut metrics = ProjectMetrics::new();
        metrics
            .crate_dependencies
            .insert("app".to_string(), vec!["serde".to_string()]);
        metrics.add_coupling(external_coupling(
            "app::a",
            "serde",
            IntegrationStrength::Model,
        ));
        metrics.add_coupling(external_coupling(
            "app::b",
            "Vec",
            IntegrationStrength::Functional,
        ));

        let report = analyze_external_dependencies(&metrics, &HashMap::new());

        assert_eq!(report.dependencies.len(), 1);
        assert_eq!(report.dependencies[0].crate_name, "serde");
    }

    #[test]
    fn scattered_coupling_is_flagged_above_threshold() {
        let dependency = ExternalDependencyUsage {
            crate_name: "reqwest".to_string(),
            versions: vec![],
            breadth: 4,
            total_references: 9,
            dominant_strength: "Functional".to_string(),
            source_modules: vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
        };

        let issues = detect_scattered_external_coupling(&[dependency]);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, IssueType::ScatteredExternalCoupling);
        assert_eq!(issues[0].severity, Severity::Medium);
        assert_eq!(issues[0].target, "reqwest");
        assert!(format!("{}", issues[0].refactoring).contains("facade"));
    }

    #[test]
    fn cargo_lock_versions_are_parsed() {
        let content = r#"
[[package]]
name = "serde"
version = "1.0.228"

[[package]]
name = "serde"
version = "1.0.229"

[[package]]
name = "clap"
version = "4.6.0"
"#;

        let versions = parse_lock_versions(content).unwrap();

        assert_eq!(
            versions.get("serde").unwrap(),
            &vec!["1.0.228".to_string(), "1.0.229".to_string()]
        );
        assert_eq!(versions.get("clap").unwrap(), &vec!["4.6.0".to_string()]);
    }
}
