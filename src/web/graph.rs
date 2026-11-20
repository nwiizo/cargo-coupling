//! Graph data structures for web visualization
//!
//! Converts ProjectMetrics to a JSON-serializable graph format
//! suitable for Cytoscape.js visualization.

use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::analyzer::ItemDepType;
use crate::balance::issue::CouplingIssue;
use crate::balance::issue_type::IssueType;
use crate::balance::score::{BalanceInterpretation, BalanceScore, IssueThresholds};
use crate::balance::severity::Severity;
use crate::manifest::AnalysisManifest;
use crate::metrics::coupling::CouplingMetrics;
use crate::metrics::dimensions::{Distance, IntegrationStrength};
use crate::metrics::module::BalanceClassification;
use crate::metrics::project::ProjectMetrics;
use crate::volatility::Volatility;

/// Temporal coupling data for visualization
#[derive(Debug, Clone, Serialize)]
pub struct TemporalCouplingData {
    pub file_a: String,
    pub file_b: String,
    pub source_module: Option<String>,
    pub target_module: Option<String>,
    pub co_change_count: usize,
    pub coupling_ratio: f64,
    pub is_strong: bool,
    pub hidden: bool,
}

/// Strong temporal coupling with no explicit code edge.
#[derive(Debug, Clone, Serialize)]
pub struct HiddenCouplingEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub co_change_count: usize,
    pub coupling_ratio: f64,
    /// Co-change alone cannot establish strength, distance, or volatility.
    pub dimensions: Option<Dimensions>,
    pub issue: IssueInfo,
}

/// Full issue list for web panels, including module-level issues.
#[derive(Debug, Clone, Serialize)]
pub struct GraphIssue {
    pub id: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub severity: String,
    pub source: String,
    pub target: String,
    pub description: String,
    pub refactoring: String,
    pub balance_score: f64,
    pub focus: IssueFocus,
}

/// Where a web issue should focus in the graph.
#[derive(Debug, Clone, Serialize)]
pub struct IssueFocus {
    pub kind: String,
    pub node_ids: Vec<String>,
    pub edge_id: Option<String>,
}

/// Declared analysis blind spots shown in the web UI.
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisManifestData {
    pub blind_spots: Vec<BlindSpotData>,
    pub notes: Vec<String>,
    pub notes_ja: Vec<String>,
}

/// One blind spot in the web manifest.
#[derive(Debug, Clone, Serialize)]
pub struct BlindSpotData {
    pub area: String,
    pub description: String,
    pub description_ja: String,
}

/// Complete graph data for visualization
#[derive(Debug, Clone, Serialize)]
pub struct GraphData {
    pub hotspots: Vec<crate::cli_output::Hotspot>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub summary: Summary,
    pub circular_dependencies: Vec<Vec<String>>,
    pub temporal_couplings: Vec<TemporalCouplingData>,
    pub hidden_couplings: Vec<HiddenCouplingEdge>,
    pub issues: Vec<GraphIssue>,
    pub not_analyzed: AnalysisManifestData,
}

/// A node in the coupling graph (represents a module)
#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub subdomain: Option<String>,
    pub expected_volatility: Option<String>,
    pub flags: Vec<String>,
    pub metrics: NodeMetrics,
    pub in_cycle: bool,
    pub file_path: Option<String>,
    /// Items defined in this module (structs, enums, traits, functions)
    pub items: Vec<ModuleItem>,
}

/// An item defined in a module (struct, enum, trait, or function)
#[derive(Debug, Clone, Serialize)]
pub struct ModuleItem {
    pub name: String,
    pub kind: String,
    pub visibility: String,
    /// Dependencies of this item (what it calls/uses)
    pub dependencies: Vec<ItemDepInfo>,
    pub file_path: Option<String>,
    pub line: Option<usize>,
}

/// Information about an item-level dependency
#[derive(Debug, Clone, Serialize)]
pub struct ItemDepInfo {
    /// Target (what is being called/used)
    pub target: String,
    /// Type of dependency (FunctionCall, MethodCall, FieldAccess, etc.)
    pub dep_type: String,
    /// Distance (SameModule, DifferentModule, DifferentCrate)
    pub distance: String,
    /// Integration strength
    pub strength: String,
    /// The actual expression (e.g., "config.thresholds")
    pub expression: Option<String>,
}

/// Metrics for a single node
#[derive(Debug, Clone, Serialize)]
pub struct NodeMetrics {
    pub couplings_out: usize,
    pub couplings_in: usize,
    pub balance_score: f64,
    pub health: String,
    pub trait_impl_count: usize,
    pub inherent_impl_count: usize,
    pub volatility: f64,
    /// Number of functions defined in this module
    pub fn_count: usize,
    /// Number of types (structs, enums) defined in this module
    pub type_count: usize,
    /// Total impl count (trait + inherent)
    pub impl_count: usize,
}

/// Location information for an edge
#[derive(Debug, Clone, Serialize)]
pub struct LocationInfo {
    pub file_path: Option<String>,
    pub line: usize,
}

/// An edge in the coupling graph (represents a coupling relationship)
#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub dimensions: Dimensions,
    pub issue: Option<IssueInfo>,
    pub in_cycle: bool,
    pub location: Option<LocationInfo>,
}

/// The 5 coupling dimensions
#[derive(Debug, Clone, Serialize)]
pub struct Dimensions {
    pub strength: DimensionValue,
    pub distance: DimensionValue,
    pub volatility: DimensionValue,
    pub balance: BalanceValue,
    pub connascence: Option<ConnascenceValue>,
}

/// A single dimension value with numeric and label representation
#[derive(Debug, Clone, Serialize)]
pub struct DimensionValue {
    pub value: f64,
    pub label: String,
}

/// Balance score with interpretation
#[derive(Debug, Clone, Serialize)]
pub struct BalanceValue {
    pub value: f64,
    pub label: String,
    pub interpretation: String,
    /// Khononov's balance classification
    pub classification: String,
    /// Japanese description
    pub classification_ja: String,
}

/// Connascence information
#[derive(Debug, Clone, Serialize)]
pub struct ConnascenceValue {
    #[serde(rename = "type")]
    pub connascence_type: String,
    pub strength: f64,
}

/// Issue information for problematic couplings
#[derive(Debug, Clone, Serialize)]
pub struct IssueInfo {
    #[serde(rename = "type")]
    pub issue_type: String,
    pub severity: String,
    pub description: String,
}

/// Summary statistics for the graph
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub health_grade: String,
    pub health_score: f64,
    pub total_modules: usize,
    pub total_couplings: usize,
    pub internal_couplings: usize,
    pub external_couplings: usize,
    pub issues_by_severity: IssuesByServerity,
}

/// Issue counts by severity
#[derive(Debug, Clone, Serialize)]
pub struct IssuesByServerity {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Helper to extract short module name from full path
fn get_short_name(full_path: &str) -> &str {
    full_path.split("::").last().unwrap_or(full_path)
}

fn ordered_pair(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn volatility_label(volatility: crate::volatility::Volatility) -> String {
    match volatility {
        crate::volatility::Volatility::Low => "Low",
        crate::volatility::Volatility::Medium => "Medium",
        crate::volatility::Volatility::High => "High",
    }
    .to_string()
}

fn build_file_to_module_map(metrics: &ProjectMetrics) -> Vec<(String, String)> {
    metrics
        .modules
        .iter()
        .map(|(name, module)| (normalize_path_string(&module.path), name.clone()))
        .collect()
}

fn module_for_file(file_path: &str, file_to_module: &[(String, String)]) -> Option<String> {
    let normalized_file = normalize_path_str(file_path);
    file_to_module
        .iter()
        .find(|(module_path, _)| paths_match(module_path, &normalized_file))
        .map(|(_, module_name)| module_name.clone())
}

fn normalize_path_string(path: &std::path::Path) -> String {
    normalize_path_str(&path.to_string_lossy())
}

fn normalize_path_str(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn paths_match(module_path: &str, git_path: &str) -> bool {
    module_path == git_path
        || module_path.ends_with(&format!("/{git_path}"))
        || git_path.ends_with(&format!("/{module_path}"))
}

/// Convert ProjectMetrics to GraphData for visualization
pub fn project_to_graph(metrics: &ProjectMetrics, thresholds: &IssueThresholds) -> GraphData {
    let git_used = metrics.has_git_observations();
    project_to_graph_with_history(metrics, thresholds, git_used)
}

pub(super) fn project_to_graph_with_history(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    git_used: bool,
) -> GraphData {
    let module_index =
        crate::metrics::target::ModuleIndex::new(&metrics.modules, &metrics.workspace_members);
    let effective_couplings: Vec<_> = metrics
        .couplings
        .iter()
        .map(|coupling| module_index.effective_coupling(coupling))
        .collect();
    // Use the caller's configured thresholds so the web grade matches the CLI grade
    // (the previous default-thresholds call produced an inconsistent grade).
    let balance_report =
        crate::balance::analyze_project_balance_with_thresholds(metrics, thresholds);
    let circular_deps = metrics.detect_circular_dependencies();
    let accidental_volatility_modules: HashSet<String> = balance_report
        .issues
        .iter()
        .filter(|issue| issue.issue_type == IssueType::AccidentalVolatility)
        .map(|issue| issue.source.clone())
        .collect();

    // Collect nodes in cycles for highlighting
    let cycle_nodes: HashSet<String> = circular_deps.iter().flatten().cloned().collect();

    // Build edge lookup for cycle detection
    let cycle_edges: HashSet<(String, String)> = circular_deps
        .iter()
        .flat_map(|cycle| {
            cycle
                .windows(2)
                .map(|w| (w[0].clone(), w[1].clone()))
                .chain(std::iter::once((
                    cycle.last().cloned().unwrap_or_default(),
                    cycle.first().cloned().unwrap_or_default(),
                )))
        })
        .collect();

    let normalize_to_node_id = |path: &str| {
        module_index
            .resolve(path)
            .map_or_else(|| path.to_owned(), |module| module.name.clone())
    };
    let target_node_id = |edge: &CouplingMetrics| {
        if edge.distance != Distance::DifferentCrate {
            normalize_to_node_id(&edge.target)
        } else if metrics.modules.contains_key(&edge.target) {
            format!("[external] {}", edge.target)
        } else {
            edge.target.clone()
        }
    };

    // Build node metrics from couplings (using normalized IDs)
    let mut node_couplings_out: HashMap<String, usize> = HashMap::new();
    let mut node_couplings_in: HashMap<String, usize> = HashMap::new();
    let mut node_balance_scores: HashMap<String, Vec<f64>> = HashMap::new();
    let mut node_volatility: HashMap<String, f64> = HashMap::new();

    for coupling in &effective_couplings {
        let source_id = normalize_to_node_id(&coupling.source);
        let target_id = target_node_id(coupling);

        *node_couplings_out.entry(source_id.clone()).or_insert(0) += 1;
        *node_couplings_in.entry(target_id.clone()).or_insert(0) += 1;

        let score = BalanceScore::calculate(coupling);
        node_balance_scores
            .entry(source_id)
            .or_default()
            .push(score.score);

        // Track volatility for target
        let vol = coupling.volatility.value();
        node_volatility
            .entry(target_id)
            .and_modify(|v| *v = v.max(vol))
            .or_insert(vol);
    }

    // Build nodes
    let mut nodes: Vec<Node> = Vec::new();
    let mut seen_nodes: HashSet<String> = HashSet::new();

    for (name, module) in &metrics.modules {
        seen_nodes.insert(name.clone());

        let out_count = node_couplings_out.get(name).copied().unwrap_or(0);
        let in_count = node_couplings_in.get(name).copied().unwrap_or(0);
        let avg_balance = node_balance_scores
            .get(name)
            .map(|scores| scores.iter().sum::<f64>() / scores.len() as f64)
            .unwrap_or(1.0);

        let health = if avg_balance >= 0.8 {
            "good"
        } else if avg_balance >= 0.6 {
            "acceptable"
        } else if avg_balance >= 0.4 {
            "needs_review"
        } else {
            "critical"
        };

        // Build a map of item dependencies by source item
        let mut item_deps_map: HashMap<String, Vec<ItemDepInfo>> = HashMap::new();
        for dep in &module.item_dependencies {
            let deps = item_deps_map.entry(dep.source_item.clone()).or_default();

            // Determine distance based on target module
            let distance = if dep.target_module.as_ref() == Some(&module.name) {
                "SameModule"
            } else if dep.target_module.is_some() {
                "DifferentModule"
            } else {
                "DifferentCrate"
            };

            // Determine strength based on dep type
            let strength = match dep.dep_type {
                ItemDepType::FieldAccess | ItemDepType::StructConstruction => "Intrusive",
                ItemDepType::FunctionCall | ItemDepType::MethodCall => "Functional",
                ItemDepType::TypeUsage | ItemDepType::Import => "Model",
                ItemDepType::TraitImpl | ItemDepType::TraitBound => "Contract",
            };

            deps.push(ItemDepInfo {
                target: dep.target.clone(),
                dep_type: format!("{:?}", dep.dep_type),
                distance: distance.to_string(),
                strength: strength.to_string(),
                expression: dep.expression.clone(),
            });
        }

        // Convert type_definitions and function_definitions to ModuleItem list
        let mut items: Vec<ModuleItem> = module
            .type_definitions
            .values()
            .map(|def| ModuleItem {
                name: def.name.clone(),
                kind: if def.is_trait { "trait" } else { "type" }.to_string(),
                visibility: format!("{}", def.visibility),
                dependencies: item_deps_map.get(&def.name).cloned().unwrap_or_default(),
                file_path: Some(module.path.display().to_string()),
                line: module.item_locations.get(&def.name).copied(),
            })
            .collect();

        // Add functions to items
        items.extend(module.function_definitions.values().map(|def| ModuleItem {
            name: def.name.clone(),
            kind: "fn".to_string(),
            visibility: format!("{}", def.visibility),
            dependencies: item_deps_map.get(&def.name).cloned().unwrap_or_default(),
            file_path: Some(module.path.display().to_string()),
            line: module.item_locations.get(&def.name).copied(),
        }));
        for (name, visibility) in &module.method_definitions {
            items.push(ModuleItem {
                name: name.clone(),
                kind: "fn".into(),
                visibility: visibility.to_string(),
                dependencies: item_deps_map.get(name).cloned().unwrap_or_default(),
                file_path: Some(module.path.display().to_string()),
                line: module.item_locations.get(name).copied(),
            });
        }
        items.sort_by(|a, b| a.name.cmp(&b.name));

        // Count functions and types
        let fn_count = module.function_definitions.len();
        let type_count = module.type_definitions.len();
        let impl_count = module.trait_impl_count + module.inherent_impl_count;

        let subdomain = module.subdomain.map(|s| s.to_string());
        let expected_volatility = module
            .subdomain
            .map(|s| volatility_label(s.expected_volatility()));
        let mut flags = Vec::new();
        if accidental_volatility_modules.contains(name) {
            flags.push("AccidentalVolatility".to_string());
        }

        nodes.push(Node {
            id: name.clone(),
            label: module.name.clone(),
            subdomain,
            expected_volatility,
            flags,
            metrics: NodeMetrics {
                couplings_out: out_count,
                couplings_in: in_count,
                balance_score: avg_balance,
                health: health.to_string(),
                trait_impl_count: module.trait_impl_count,
                inherent_impl_count: module.inherent_impl_count,
                volatility: node_volatility.get(name).copied().unwrap_or(0.0),
                fn_count,
                type_count,
                impl_count,
            },
            in_cycle: cycle_nodes.contains(name),
            file_path: Some(module.path.display().to_string()),
            items,
        });
    }

    // Add nodes that appear only in couplings but not in modules (external crates)
    for coupling in &effective_couplings {
        for (full_path, node_id, is_external) in [
            (
                &coupling.source,
                normalize_to_node_id(&coupling.source),
                false,
            ),
            (
                &coupling.target,
                target_node_id(coupling),
                coupling.distance == Distance::DifferentCrate,
            ),
        ] {
            // Skip glob imports (e.g., "crate::*", "foo::*")
            if full_path.ends_with("::*") || full_path == "*" {
                continue;
            }

            // Skip if already seen (either as internal module or previously added external)
            if seen_nodes.contains(&node_id) {
                continue;
            }
            seen_nodes.insert(node_id.clone());

            let out_count = node_couplings_out.get(&node_id).copied().unwrap_or(0);
            let in_count = node_couplings_in.get(&node_id).copied().unwrap_or(0);
            let avg_balance = node_balance_scores
                .get(&node_id)
                .map(|scores| scores.iter().sum::<f64>() / scores.len() as f64)
                .unwrap_or(1.0);

            let health = if avg_balance >= 0.8 {
                "good"
            } else {
                "needs_review"
            };

            nodes.push(Node {
                id: node_id.clone(),
                label: get_short_name(full_path).to_string(),
                subdomain: None,
                expected_volatility: None,
                flags: Vec::new(),
                metrics: NodeMetrics {
                    couplings_out: out_count,
                    couplings_in: in_count,
                    balance_score: avg_balance,
                    health: health.to_string(),
                    trait_impl_count: 0,
                    inherent_impl_count: 0,
                    volatility: node_volatility.get(&node_id).copied().unwrap_or(0.0),
                    fn_count: 0,
                    type_count: 0,
                    impl_count: 0,
                },
                in_cycle: cycle_nodes.contains(&node_id),
                file_path: if is_external {
                    Some(format!("[external] {}", full_path))
                } else {
                    None
                },
                items: Vec::new(),
            });
        }
    }

    nodes.sort_by(|a, b| a.id.cmp(&b.id));

    // Build edges (using normalized node IDs)
    let mut edges: Vec<Edge> = Vec::new();
    let reported: HashMap<_, _> = balance_report
        .issues
        .iter()
        .enumerate()
        .map(|(index, issue)| (issue.stable_key(), (index, issue)))
        .collect();
    let mut issue_edges = HashMap::new();

    for (edge_id, coupling) in effective_couplings.iter().enumerate() {
        // Skip edges involving glob imports
        if coupling.source.ends_with("::*")
            || coupling.source == "*"
            || coupling.target.ends_with("::*")
            || coupling.target == "*"
        {
            continue;
        }

        let source_id = normalize_to_node_id(&coupling.source);
        let target_id = target_node_id(coupling);

        // Skip self-loops (module referencing itself)
        if source_id == target_id {
            continue;
        }

        let score = BalanceScore::calculate(coupling);
        let in_cycle = cycle_edges.contains(&(coupling.source.clone(), coupling.target.clone()));

        let id = format!("e{edge_id}");
        // A reported finding belongs to the observation that triggered it.
        // Parallel references can have different strength, severity and balance.
        let issue = crate::balance::identify_issues_with_thresholds(coupling, thresholds)
            .into_iter()
            .filter_map(|candidate| {
                let &(index, finding) = reported.get(&candidate.stable_key())?;
                if finding.severity != candidate.severity
                    || finding.balance_score != candidate.balance_score
                {
                    return None;
                }
                issue_edges.entry(index).or_insert_with(|| id.clone());
                Some(index)
            })
            .min()
            .map(|index| &balance_report.issues[index])
            .map(|issue| IssueInfo {
                issue_type: format!("{:?}", issue.issue_type),
                severity: issue.severity.to_string(),
                description: issue.description.clone(),
            });

        // Build location info if available
        let location = if coupling.location.line > 0 || coupling.location.file_path.is_some() {
            Some(LocationInfo {
                file_path: coupling
                    .location
                    .file_path
                    .as_ref()
                    .map(|p| p.display().to_string()),
                line: coupling.location.line,
            })
        } else {
            None
        };

        edges.push(Edge {
            id,
            source: source_id,
            target: target_id,
            dimensions: coupling_to_dimensions(coupling, &score),
            issue,
            in_cycle,
            location,
        });
    }

    let explicit_edge_pairs: HashSet<(String, String)> = edges
        .iter()
        .map(|edge| ordered_pair(&edge.source, &edge.target))
        .collect();
    let file_to_module = build_file_to_module_map(metrics);
    let hidden_couplings = build_hidden_couplings(metrics, &file_to_module, &balance_report.issues);
    let graph_issues = build_graph_issues(
        &balance_report.issues,
        &normalize_to_node_id,
        &issue_edges,
        &hidden_couplings,
    );

    // Count issues by severity
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;

    for issue in &balance_report.issues {
        match issue.severity {
            Severity::Critical => critical += 1,
            Severity::High => high += 1,
            Severity::Medium => medium += 1,
            Severity::Low => low += 1,
        }
    }

    // Count internal vs external couplings
    let internal_couplings = metrics.internal_coupling_count();
    let external_couplings = metrics.couplings.len() - internal_couplings;

    GraphData {
        hotspots: crate::cli_output::calculate_hotspots(metrics, thresholds, usize::MAX),
        nodes,
        edges,
        summary: Summary {
            health_grade: format!("{:?}", balance_report.health_grade),
            health_score: balance_report.average_score,
            total_modules: metrics.modules.len(),
            total_couplings: metrics.couplings.len(),
            internal_couplings,
            external_couplings,
            issues_by_severity: IssuesByServerity {
                critical,
                high,
                medium,
                low,
            },
        },
        circular_dependencies: circular_deps,
        temporal_couplings: metrics
            .temporal_couplings
            .iter()
            .take(20)
            .map(|tc| {
                let source_module = module_for_file(&tc.file_a, &file_to_module);
                let target_module = module_for_file(&tc.file_b, &file_to_module);
                let hidden = tc.is_strong()
                    && source_module.as_ref().is_some_and(|source| {
                        target_module.as_ref().is_some_and(|target| {
                            source != target
                                && !explicit_edge_pairs.contains(&ordered_pair(source, target))
                        })
                    });
                TemporalCouplingData {
                    file_a: tc.file_a.clone(),
                    file_b: tc.file_b.clone(),
                    source_module,
                    target_module,
                    co_change_count: tc.co_change_count,
                    coupling_ratio: tc.coupling_ratio,
                    is_strong: tc.is_strong(),
                    hidden,
                }
            })
            .collect(),
        hidden_couplings,
        issues: graph_issues,
        not_analyzed: build_not_analyzed_manifest(AnalysisManifest::from_metrics(
            metrics,
            git_used,
            thresholds.exclude_tests,
        )),
    }
}

fn build_not_analyzed_manifest(manifest: AnalysisManifest) -> AnalysisManifestData {
    AnalysisManifestData {
        blind_spots: manifest
            .blind_spots
            .iter()
            .map(|spot| BlindSpotData {
                area: spot.area.to_string(),
                description: spot.description.to_string(),
                description_ja: spot.description_ja.to_string(),
            })
            .collect(),
        notes: manifest.notes,
        notes_ja: manifest.notes_ja,
    }
}

fn build_hidden_couplings(
    metrics: &ProjectMetrics,
    file_to_module: &[(String, String)],
    issues: &[CouplingIssue],
) -> Vec<HiddenCouplingEdge> {
    let mut seen = HashSet::new();
    let mut hidden = Vec::new();

    for temporal in metrics
        .temporal_couplings
        .iter()
        .filter(|tc| tc.is_strong())
    {
        let Some(source) = module_for_file(&temporal.file_a, file_to_module) else {
            continue;
        };
        let Some(target) = module_for_file(&temporal.file_b, file_to_module) else {
            continue;
        };
        if source == target {
            continue;
        }

        let pair = ordered_pair(&source, &target);
        let Some(issue) = issues.iter().find(|issue| {
            issue.issue_type == IssueType::HiddenCoupling
                && ordered_pair(&issue.source, &issue.target) == pair
        }) else {
            continue;
        };
        if !seen.insert(pair.clone()) {
            continue;
        }

        hidden.push(HiddenCouplingEdge {
            id: format!("hidden-{}-{}", sanitize_id(&pair.0), sanitize_id(&pair.1)),
            source: pair.0,
            target: pair.1,
            co_change_count: temporal.co_change_count,
            coupling_ratio: temporal.coupling_ratio,
            dimensions: None,
            issue: IssueInfo {
                issue_type: "HiddenCoupling".to_string(),
                severity: issue.severity.to_string(),
                description: issue.description.clone(),
            },
        });
    }

    hidden
}

fn build_graph_issues<F>(
    issues: &[CouplingIssue],
    normalize_to_node_id: &F,
    issue_edges: &HashMap<usize, String>,
    hidden_couplings: &[HiddenCouplingEdge],
) -> Vec<GraphIssue>
where
    F: Fn(&str) -> String,
{
    issues
        .iter()
        .enumerate()
        .map(|(index, issue)| {
            let source = normalize_issue_endpoint(&issue.source, normalize_to_node_id);
            let target = normalize_issue_endpoint(&issue.target, normalize_to_node_id);
            let focus = issue_focus(
                issue,
                &source,
                &target,
                issue_edges.get(&index),
                hidden_couplings,
            );

            GraphIssue {
                id: format!("issue-{index}"),
                issue_type: format!("{:?}", issue.issue_type),
                severity: issue.severity.to_string(),
                source,
                target,
                description: issue.description.clone(),
                refactoring: issue.refactoring.to_string(),
                balance_score: issue.balance_score,
                focus,
            }
        })
        .collect()
}

fn issue_focus(
    issue: &CouplingIssue,
    source: &str,
    target: &str,
    edge_id: Option<&String>,
    hidden_couplings: &[HiddenCouplingEdge],
) -> IssueFocus {
    if issue.issue_type == IssueType::HiddenCoupling
        && let Some(hidden_edge) = hidden_couplings
            .iter()
            .find(|edge| ordered_pair(&edge.source, &edge.target) == ordered_pair(source, target))
    {
        return IssueFocus {
            kind: "hidden-edge".to_string(),
            node_ids: vec![hidden_edge.source.clone(), hidden_edge.target.clone()],
            edge_id: Some(hidden_edge.id.clone()),
        };
    }

    if let Some(edge_id) = edge_id {
        return IssueFocus {
            kind: "edge".to_string(),
            node_ids: vec![source.to_string(), target.to_string()],
            edge_id: Some(edge_id.clone()),
        };
    }

    let node_ids = match issue.issue_type {
        IssueType::HighAfferentCoupling => vec![target.to_string()],
        IssueType::HighEfferentCoupling
        | IssueType::GodModule
        | IssueType::AccidentalVolatility
        | IssueType::PublicFieldExposure
        | IssueType::PrimitiveObsession => vec![source.to_string()],
        _ if source != target => vec![source.to_string(), target.to_string()],
        _ => vec![source.to_string()],
    };

    IssueFocus {
        kind: if node_ids.len() > 1 { "nodes" } else { "node" }.to_string(),
        node_ids,
        edge_id: None,
    }
}

fn normalize_issue_endpoint<F>(endpoint: &str, normalize_to_node_id: &F) -> String
where
    F: Fn(&str) -> String,
{
    if endpoint.ends_with(" dependencies")
        || endpoint.ends_with(" dependents")
        || matches!(endpoint, "Core" | "Supporting" | "Generic")
    {
        return endpoint.to_string();
    }
    normalize_to_node_id(endpoint)
}

fn coupling_to_dimensions(coupling: &CouplingMetrics, score: &BalanceScore) -> Dimensions {
    let strength_label = match coupling.strength {
        IntegrationStrength::Intrusive => "Intrusive",
        IntegrationStrength::Functional => "Functional",
        IntegrationStrength::Model => "Model",
        IntegrationStrength::Contract => "Contract",
    };

    let distance_label = match coupling.distance {
        Distance::SameFunction => "SameFunction",
        Distance::SameModule => "SameModule",
        Distance::DifferentModule => "DifferentModule",
        Distance::DifferentCrate => "DifferentCrate",
    };

    let volatility_label = match coupling.volatility {
        Volatility::Low => "Low",
        Volatility::Medium => "Medium",
        Volatility::High => "High",
    };

    let balance_label = match score.interpretation {
        BalanceInterpretation::Balanced => "Balanced",
        BalanceInterpretation::Acceptable => "Acceptable",
        BalanceInterpretation::NeedsReview => "NeedsReview",
        BalanceInterpretation::NeedsRefactoring => "NeedsRefactoring",
        BalanceInterpretation::Critical => "Critical",
    };

    // Calculate Khononov's BalanceClassification
    let classification =
        BalanceClassification::classify(coupling.strength, coupling.distance, coupling.volatility);
    let classification_en = match classification {
        BalanceClassification::Pain => "Global Complexity",
        _ => classification.description_en(),
    };
    let classification_ja = match classification {
        BalanceClassification::Pain => "グローバル複雑性 (強+遠+変動)",
        _ => classification.description_ja(),
    };

    Dimensions {
        strength: DimensionValue {
            value: coupling.strength.value(),
            label: strength_label.to_string(),
        },
        distance: DimensionValue {
            value: coupling.distance.value(),
            label: distance_label.to_string(),
        },
        volatility: DimensionValue {
            value: coupling.volatility.value(),
            label: volatility_label.to_string(),
        },
        balance: BalanceValue {
            value: score.score,
            label: balance_label.to_string(),
            interpretation: format!("{:?}", score.interpretation),
            classification: classification_en.to_string(),
            classification_ja: classification_ja.to_string(),
        },
        connascence: None, // TODO: Add connascence tracking per coupling
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_keeps_external_homonyms_outside_internal_nodes() {
        let mut metrics = ProjectMetrics::new();
        for name in ["app::consumer", "shared"] {
            metrics.add_module(crate::ModuleMetrics::new(
                format!("{name}.rs").into(),
                name.into(),
            ));
        }
        for (target, distance) in [
            ("app::shared", Distance::DifferentModule),
            ("other::shared", Distance::DifferentCrate),
            ("shared", Distance::DifferentCrate),
        ] {
            metrics.add_coupling(CouplingMetrics::new(
                "app::consumer".into(),
                target.into(),
                IntegrationStrength::Model,
                distance,
                Volatility::Low,
            ));
        }
        let graph = project_to_graph(&metrics, &IssueThresholds::default());
        let local = graph.nodes.iter().find(|node| node.id == "shared").unwrap();
        assert_eq!(local.metrics.couplings_in, 1);
        assert_eq!(graph.summary.internal_couplings, 1);
        assert_eq!(graph.summary.external_couplings, 2);
        assert_eq!(
            graph
                .nodes
                .iter()
                .filter(|node| node
                    .file_path
                    .as_deref()
                    .is_some_and(|path| path.starts_with("[external]")))
                .count(),
            2
        );
        assert!(
            graph
                .edges
                .iter()
                .any(|edge| edge.target == "other::shared")
        );
    }

    #[test]
    fn graph_reports_excluded_tests() {
        let thresholds = IssueThresholds {
            exclude_tests: true,
            ..Default::default()
        };
        let graph = project_to_graph(&ProjectMetrics::new(), &thresholds);
        assert!(
            graph
                .not_analyzed
                .notes
                .iter()
                .any(|note| note.contains("Test code was excluded"))
        );
    }

    #[test]
    fn temporal_edges_reuse_report_findings_without_inventing_dimensions() {
        let mut metrics = ProjectMetrics::new();
        for name in ["pricing", "invoicing", "lib"] {
            metrics.add_module(crate::ModuleMetrics::new(
                format!("src/{name}.rs").into(),
                name.into(),
            ));
        }
        for other in ["invoicing", "lib"] {
            metrics
                .temporal_couplings
                .push(crate::volatility::TemporalCoupling {
                    file_a: "src/pricing.rs".into(),
                    file_b: format!("src/{other}.rs"),
                    co_change_count: 3,
                    coupling_ratio: 1.0,
                });
        }
        let graph = project_to_graph(&metrics, &IssueThresholds::default());
        assert_eq!(
            graph.hidden_couplings.len(),
            1,
            "facade co-change is expected"
        );
        let edge = &graph.hidden_couplings[0];
        assert_eq!(
            edge.issue.severity, "Medium",
            "three co-changes do not establish High severity"
        );
        let json = serde_json::to_value(edge).unwrap();
        assert!(
            json["dimensions"].is_null(),
            "co-change does not measure any model dimension"
        );
        let issue = graph
            .issues
            .iter()
            .find(|issue| issue.issue_type == "HiddenCoupling")
            .unwrap();
        assert_eq!(edge.issue.severity, issue.severity);
        assert_eq!(issue.focus.edge_id.as_deref(), Some(edge.id.as_str()));
    }

    #[test]
    fn cohesive_edges_do_not_receive_web_only_risk_findings() {
        let mut metrics = ProjectMetrics::new();
        for name in ["domain::consumer", "domain::provider"] {
            metrics.add_module(crate::ModuleMetrics::new(
                format!("{name}.rs").into(),
                name.into(),
            ));
        }
        metrics.add_coupling(CouplingMetrics::new(
            "domain::consumer".into(),
            "domain::provider".into(),
            IntegrationStrength::Intrusive,
            Distance::SameModule,
            Volatility::High,
        ));
        let thresholds = IssueThresholds::default();
        let report = crate::analyze_project_balance_with_thresholds(&metrics, &thresholds);
        assert!(report.issues.is_empty());
        let graph = project_to_graph(&metrics, &thresholds);
        assert!(
            graph.edges[0].issue.is_none(),
            "Web must share the CLI's balance judgment"
        );
    }

    #[test]
    fn parallel_edges_attach_and_focus_findings_on_the_observed_dependency() {
        let mut metrics = ProjectMetrics::new();
        for strength in [
            IntegrationStrength::Contract,
            IntegrationStrength::Intrusive,
        ] {
            metrics.add_coupling(CouplingMetrics::new(
                "consumer".into(),
                "provider".into(),
                strength,
                Distance::DifferentModule,
                Volatility::High,
            ));
        }
        let graph = project_to_graph(&metrics, &IssueThresholds::default());
        let contract = graph
            .edges
            .iter()
            .find(|e| e.dimensions.strength.label == "Contract")
            .unwrap();
        let intrusive = graph
            .edges
            .iter()
            .find(|e| e.dimensions.strength.label == "Intrusive")
            .unwrap();
        assert!(contract.issue.is_none());
        assert_eq!(intrusive.issue.as_ref().unwrap().severity, "High");
        let issue = graph
            .issues
            .iter()
            .find(|issue| issue.issue_type == "CascadingChangeRisk")
            .unwrap();
        assert_eq!(issue.focus.edge_id.as_ref(), Some(&intrusive.id));
    }

    #[test]
    fn graph_dimensions_use_declared_essential_volatility() {
        let mut metrics = ProjectMetrics::new();
        metrics.add_module(crate::ModuleMetrics::new(
            "consumer.rs".into(),
            "consumer".into(),
        ));
        let mut provider = crate::ModuleMetrics::new("provider.rs".into(), "provider".into());
        provider.subdomain = Some(crate::Subdomain::Generic);
        metrics.add_module(provider);
        metrics.add_coupling(CouplingMetrics::new(
            "consumer".into(),
            "provider".into(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        ));
        let graph = project_to_graph(&metrics, &IssueThresholds::default());
        assert_eq!(
            graph.edges[0].dimensions.volatility.value,
            crate::Subdomain::Generic.expected_volatility().value()
        );
    }

    #[test]
    fn qualified_module_identity_wins_over_an_unrelated_item_with_the_same_name() {
        let mut metrics = ProjectMetrics::new();
        let original = crate::ModuleMetrics::new("a.rs".into(), "app::service".into());
        let mut other = crate::ModuleMetrics::new("b.rs".into(), "other".into());
        other.add_function_definition("service".into(), crate::Visibility::Public);
        metrics.add_module(original);
        metrics.add_module(other);
        metrics.add_coupling(CouplingMetrics::new(
            "app::service".into(),
            "other".into(),
            IntegrationStrength::Functional,
            Distance::DifferentModule,
            Volatility::High,
        ));
        let graph = project_to_graph(&metrics, &IssueThresholds::default());
        assert!(
            graph
                .edges
                .iter()
                .any(|edge| edge.source == "app::service" && edge.target == "other")
        );
    }

    #[test]
    fn method_names_and_locations_belong_to_the_analyzed_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lib.rs");
        std::fs::write(
            &path,
            "pub struct Service;\nimpl Service { pub fn submit(&self) {} }\n",
        )
        .unwrap();
        let metrics = crate::analyzer::analyze_project(dir.path()).unwrap();
        std::fs::write(
            &path,
            "pub struct Service;\nimpl Service { pub fn changed(&self) {} }\n",
        )
        .unwrap();
        let graph = project_to_graph(&metrics, &IssueThresholds::default());
        let method = graph
            .nodes
            .iter()
            .flat_map(|node| &node.items)
            .find(|item| item.name == "Service::submit")
            .unwrap();
        assert_eq!(method.line, Some(2));
        assert_eq!(method.kind, "fn");
        assert!(
            !graph
                .nodes
                .iter()
                .flat_map(|node| &node.items)
                .any(|item| item.name == "Service::changed")
        );
    }

    #[test]
    fn test_empty_project() {
        let metrics = ProjectMetrics::default();
        let thresholds = IssueThresholds::default();
        let graph = project_to_graph(&metrics, &thresholds);

        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert_eq!(graph.summary.total_modules, 0);
    }
}
