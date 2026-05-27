//! Graph data structures for web visualization
//!
//! Converts ProjectMetrics to a JSON-serializable graph format
//! suitable for Cytoscape.js visualization.

use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::analyzer::ItemDepType;
use crate::balance::{
    BalanceScore, CouplingIssue, IssueThresholds, IssueType, analyze_project_balance,
};
use crate::manifest::{ManifestContext, build_manifest};
use crate::metrics::{BalanceClassification, CouplingMetrics, ProjectMetrics};

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
    pub dimensions: Dimensions,
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

fn volatility_label(volatility: crate::metrics::Volatility) -> String {
    match volatility {
        crate::metrics::Volatility::Low => "Low",
        crate::metrics::Volatility::Medium => "Medium",
        crate::metrics::Volatility::High => "High",
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
    let balance_report = analyze_project_balance(metrics);
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

    // Build a mapping from full path to short name for internal modules
    // This allows us to normalize edge source/target to match node IDs
    let module_short_names: HashSet<&str> = metrics.modules.keys().map(|s| s.as_str()).collect();

    // Build a mapping from type/function names to their module names
    // This allows us to resolve paths like "BalanceScore::calculate" to "balance"
    let mut item_to_module: HashMap<&str, &str> = HashMap::new();
    for (module_name, module) in &metrics.modules {
        for type_name in module.type_definitions.keys() {
            item_to_module.insert(type_name.as_str(), module_name.as_str());
        }
        for fn_name in module.function_definitions.keys() {
            item_to_module.insert(fn_name.as_str(), module_name.as_str());
        }
    }

    // Helper closure to normalize a path to existing node ID
    let normalize_to_node_id = |path: &str| -> String {
        // First try direct module name match
        let short = get_short_name(path);
        if module_short_names.contains(short) {
            return short.to_string();
        }

        // Try to resolve via item name (type or function)
        // e.g., "BalanceScore::calculate" -> look up "BalanceScore" -> "balance"
        let parts: Vec<&str> = path.split("::").collect();
        for part in &parts {
            if let Some(module_name) = item_to_module.get(part) {
                return (*module_name).to_string();
            }
        }

        // Also try the first part which might be the module name
        if let Some(first) = parts.first()
            && module_short_names.contains(*first)
        {
            return (*first).to_string();
        }

        // Keep full path for external crates
        path.to_string()
    };

    // Build node metrics from couplings (using normalized IDs)
    let mut node_couplings_out: HashMap<String, usize> = HashMap::new();
    let mut node_couplings_in: HashMap<String, usize> = HashMap::new();
    let mut node_balance_scores: HashMap<String, Vec<f64>> = HashMap::new();
    let mut node_volatility: HashMap<String, f64> = HashMap::new();

    for coupling in &metrics.couplings {
        let source_id = normalize_to_node_id(&coupling.source);
        let target_id = normalize_to_node_id(&coupling.target);

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
            })
            .collect();

        // Add functions to items
        items.extend(module.function_definitions.values().map(|def| ModuleItem {
            name: def.name.clone(),
            kind: "fn".to_string(),
            visibility: format!("{}", def.visibility),
            dependencies: item_deps_map.get(&def.name).cloned().unwrap_or_default(),
        }));

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
    for coupling in &metrics.couplings {
        for full_path in [&coupling.source, &coupling.target] {
            // Skip glob imports (e.g., "crate::*", "foo::*")
            if full_path.ends_with("::*") || full_path == "*" {
                continue;
            }

            // Normalize to node ID (use short name for internal modules)
            let node_id = normalize_to_node_id(full_path);

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

            // Determine if this is an external crate
            let is_external = full_path.contains("::")
                && !full_path.starts_with("crate::")
                && !module_short_names.contains(get_short_name(full_path));

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

    // Build edges (using normalized node IDs)
    let mut edges: Vec<Edge> = Vec::new();

    for (edge_id, coupling) in metrics.couplings.iter().enumerate() {
        // Skip edges involving glob imports
        if coupling.source.ends_with("::*")
            || coupling.source == "*"
            || coupling.target.ends_with("::*")
            || coupling.target == "*"
        {
            continue;
        }

        let source_id = normalize_to_node_id(&coupling.source);
        let target_id = normalize_to_node_id(&coupling.target);

        // Skip self-loops (module referencing itself)
        if source_id == target_id {
            continue;
        }

        let score = BalanceScore::calculate(coupling);
        let in_cycle = cycle_edges.contains(&(coupling.source.clone(), coupling.target.clone()));

        let issue = find_issue_for_coupling(coupling, &score, thresholds);

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
            id: format!("e{}", edge_id),
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
    let hidden_couplings = build_hidden_couplings(metrics, &file_to_module, &explicit_edge_pairs);
    let graph_issues = build_graph_issues(
        &balance_report.issues,
        &normalize_to_node_id,
        &edges,
        &hidden_couplings,
    );

    // Count issues by severity
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;

    for issue in &balance_report.issues {
        match issue.severity {
            crate::balance::Severity::Critical => critical += 1,
            crate::balance::Severity::High => high += 1,
            crate::balance::Severity::Medium => medium += 1,
            crate::balance::Severity::Low => low += 1,
        }
    }

    // Count internal vs external couplings
    let internal_couplings = metrics
        .couplings
        .iter()
        .filter(|c| !c.target.contains("::") || c.target.starts_with("crate::"))
        .count();
    let external_couplings = metrics.couplings.len() - internal_couplings;

    GraphData {
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
        not_analyzed: build_not_analyzed_manifest(metrics),
    }
}

fn build_not_analyzed_manifest(metrics: &ProjectMetrics) -> AnalysisManifestData {
    let manifest = build_manifest(&ManifestContext {
        git_used: !metrics.file_changes.is_empty() || !metrics.temporal_couplings.is_empty(),
        tests_excluded: false,
        parse_failures: metrics.parse_failures,
    });

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
    explicit_edge_pairs: &HashSet<(String, String)>,
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
        if explicit_edge_pairs.contains(&pair) || !seen.insert(pair.clone()) {
            continue;
        }

        let ratio_pct = temporal.coupling_ratio * 100.0;
        hidden.push(HiddenCouplingEdge {
            id: format!("hidden-{}-{}", sanitize_id(&pair.0), sanitize_id(&pair.1)),
            source: pair.0,
            target: pair.1,
            co_change_count: temporal.co_change_count,
            coupling_ratio: temporal.coupling_ratio,
            dimensions: hidden_coupling_dimensions(temporal.coupling_ratio),
            issue: IssueInfo {
                issue_type: "HiddenCoupling".to_string(),
                severity: if temporal.coupling_ratio >= 0.8 {
                    "High"
                } else {
                    "Medium"
                }
                .to_string(),
                description: format!(
                    "Strong temporal co-change without code dependency ({ratio_pct:.0}% ratio, {} co-changes)",
                    temporal.co_change_count
                ),
            },
        });
    }

    hidden
}

fn hidden_coupling_dimensions(coupling_ratio: f64) -> Dimensions {
    Dimensions {
        strength: DimensionValue {
            value: 0.75,
            label: "Functional".to_string(),
        },
        distance: DimensionValue {
            value: 0.5,
            label: "DifferentModule".to_string(),
        },
        volatility: DimensionValue {
            value: 1.0,
            label: "High".to_string(),
        },
        balance: BalanceValue {
            value: 1.0 - coupling_ratio,
            label: "NeedsRefactoring".to_string(),
            interpretation: "Hidden temporal coupling".to_string(),
            classification: "Global Complexity".to_string(),
            classification_ja: "グローバル複雑性 (隠れた結合)".to_string(),
        },
        connascence: None,
    }
}

fn build_graph_issues<F>(
    issues: &[CouplingIssue],
    normalize_to_node_id: &F,
    edges: &[Edge],
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
            let focus = issue_focus(issue, &source, &target, edges, hidden_couplings);

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
    edges: &[Edge],
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

    if let Some(edge) = edges
        .iter()
        .find(|edge| edge.source == source && edge.target == target)
    {
        return IssueFocus {
            kind: "edge".to_string(),
            node_ids: vec![edge.source.clone(), edge.target.clone()],
            edge_id: Some(edge.id.clone()),
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
        crate::metrics::IntegrationStrength::Intrusive => "Intrusive",
        crate::metrics::IntegrationStrength::Functional => "Functional",
        crate::metrics::IntegrationStrength::Model => "Model",
        crate::metrics::IntegrationStrength::Contract => "Contract",
    };

    let distance_label = match coupling.distance {
        crate::metrics::Distance::SameFunction => "SameFunction",
        crate::metrics::Distance::SameModule => "SameModule",
        crate::metrics::Distance::DifferentModule => "DifferentModule",
        crate::metrics::Distance::DifferentCrate => "DifferentCrate",
    };

    let volatility_label = match coupling.volatility {
        crate::metrics::Volatility::Low => "Low",
        crate::metrics::Volatility::Medium => "Medium",
        crate::metrics::Volatility::High => "High",
    };

    let balance_label = match score.interpretation {
        crate::balance::BalanceInterpretation::Balanced => "Balanced",
        crate::balance::BalanceInterpretation::Acceptable => "Acceptable",
        crate::balance::BalanceInterpretation::NeedsReview => "NeedsReview",
        crate::balance::BalanceInterpretation::NeedsRefactoring => "NeedsRefactoring",
        crate::balance::BalanceInterpretation::Critical => "Critical",
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

fn find_issue_for_coupling(
    coupling: &CouplingMetrics,
    score: &BalanceScore,
    _thresholds: &IssueThresholds,
) -> Option<IssueInfo> {
    // Check for obvious issues
    if coupling.strength == crate::metrics::IntegrationStrength::Intrusive
        && coupling.distance == crate::metrics::Distance::DifferentCrate
    {
        return Some(IssueInfo {
            issue_type: "GlobalComplexity".to_string(),
            severity: "High".to_string(),
            description: format!(
                "Intrusive coupling to {} across crate boundary",
                coupling.target
            ),
        });
    }

    if coupling.strength.value() >= 0.75 && coupling.volatility == crate::metrics::Volatility::High
    {
        return Some(IssueInfo {
            issue_type: "CascadingChangeRisk".to_string(),
            severity: "Medium".to_string(),
            description: format!(
                "Strong coupling to highly volatile target {}",
                coupling.target
            ),
        });
    }

    if score.score < 0.4 {
        return Some(IssueInfo {
            issue_type: "LowBalance".to_string(),
            severity: if score.score < 0.2 { "High" } else { "Medium" }.to_string(),
            description: format!(
                "Low balance score ({:.2}) indicates coupling anti-pattern",
                score.score
            ),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

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
