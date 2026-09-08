//! Canonical, deterministic dependency paths shared by CLI and design reports.

use crate::{Distance, ProjectMetrics};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImpactPath {
    pub module: String,
    /// Change origin first, affected consumer last (opposite dependency arrows).
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Reachability {
    pub paths: Vec<ImpactPath>,
    pub max_depth: Option<usize>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    pub nodes: BTreeSet<String>,
    pub outgoing: BTreeMap<String, BTreeSet<String>>,
    pub incoming: BTreeMap<String, BTreeSet<String>>,
    pub unresolved: BTreeSet<String>,
}

impl DependencyGraph {
    pub fn from_metrics(metrics: &ProjectMetrics) -> Self {
        let names = Self {
            nodes: metrics.modules.keys().cloned().collect(),
            ..Self::default()
        };
        let mut graph = names.clone();
        // Keep nodes that only exist as endpoints, as legacy callers can build
        // coupling-only metrics. Fully qualified aliases map to a unique module.
        for edge in &metrics.couplings {
            if edge.distance == Distance::DifferentCrate {
                continue;
            }
            let source = names
                .resolve(&edge.source)
                .unwrap_or_else(|| edge.source.clone());
            let target = names
                .resolve(&edge.target)
                .unwrap_or_else(|| edge.target.clone());
            if !metrics.modules.contains_key(&source) {
                graph.unresolved.insert(source.clone());
            }
            if !metrics.modules.contains_key(&target) {
                graph.unresolved.insert(target.clone());
            }
            graph.nodes.insert(source.clone());
            graph.nodes.insert(target.clone());
            if source != target {
                graph
                    .outgoing
                    .entry(source.clone())
                    .or_default()
                    .insert(target.clone());
                graph.incoming.entry(target).or_default().insert(source);
            }
        }
        graph
    }

    pub fn resolve(&self, name: &str) -> Option<String> {
        if self.nodes.contains(name) {
            return Some(name.to_string());
        }
        if let Some(found) = self
            .nodes
            .iter()
            .filter(|key| name.ends_with(&format!("::{key}")))
            .max_by_key(|key| key.len())
        {
            return Some(found.clone());
        }
        let mut matches = self
            .nodes
            .iter()
            .filter(|key| key.ends_with(&format!("::{name}")));
        let found = matches.next()?;
        matches.next().is_none().then(|| found.clone())
    }

    pub fn select(&self, selector: &str) -> Vec<String> {
        let Ok(pattern) = glob::Pattern::new(selector) else {
            return vec![];
        };
        self.nodes
            .iter()
            .filter(|node| pattern.matches(node))
            .cloned()
            .collect()
    }

    pub fn impact(&self, origin: &str, max_depth: Option<usize>) -> Reachability {
        self.walk(origin, max_depth, &self.incoming)
    }

    pub fn dependencies(&self, origin: &str, max_depth: Option<usize>) -> Reachability {
        self.walk(origin, max_depth, &self.outgoing)
    }

    fn walk(
        &self,
        origin: &str,
        max_depth: Option<usize>,
        edges: &BTreeMap<String, BTreeSet<String>>,
    ) -> Reachability {
        let mut result = Reachability {
            max_depth,
            ..Reachability::default()
        };
        let Some(origin) = self.resolve(origin) else {
            return result;
        };
        let mut visited = BTreeSet::from([origin.clone()]);
        let mut queue = VecDeque::from([vec![origin]]);
        let mut frontier = BTreeSet::new();
        while let Some(path) = queue.pop_front() {
            let current = path.last().expect("nonempty BFS path");
            if max_depth.is_some_and(|depth| path.len() > depth) {
                frontier.insert(current.clone());
                continue;
            }
            for next in edges.get(current).into_iter().flatten() {
                if visited.insert(next.clone()) {
                    let mut next_path = path.clone();
                    next_path.push(next.clone());
                    result.paths.push(ImpactPath {
                        module: next.clone(),
                        path: next_path.clone(),
                    });
                    queue.push_back(next_path);
                }
            }
        }
        result.truncated = frontier.iter().any(|node| {
            edges
                .get(node)
                .into_iter()
                .flatten()
                .any(|next| !visited.contains(next))
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CouplingMetrics, IntegrationStrength, ModuleMetrics, Volatility};

    #[test]
    fn qualified_names_choose_the_longest_boundary_and_short_names_remain_ambiguous() {
        let graph = DependencyGraph {
            nodes: [
                "design".into(),
                "web::design".into(),
                "web::graph".into(),
                "design::graph".into(),
            ]
            .into(),
            ..Default::default()
        };
        assert_eq!(
            graph.resolve("cargo-coupling::web::design"),
            Some("web::design".into())
        );
        assert_eq!(graph.resolve("graph"), None);
        assert_eq!(graph.resolve("design"), Some("design".into()));
    }

    #[test]
    fn paths_cover_three_hops_and_stop_at_explicit_limit_without_cycle_loops() {
        let mut metrics = ProjectMetrics::new();
        for name in ["a", "b", "c", "d"] {
            metrics.add_module(ModuleMetrics::new(format!("{name}.rs").into(), name.into()));
        }
        for (source, target) in [("b", "a"), ("c", "b"), ("d", "c"), ("a", "d")] {
            metrics.add_coupling(CouplingMetrics::new(
                source.into(),
                target.into(),
                IntegrationStrength::Functional,
                Distance::DifferentModule,
                Volatility::High,
            ));
        }
        let graph = DependencyGraph::from_metrics(&metrics);
        assert_eq!(
            graph.impact("a", None).paths.last().unwrap().path,
            ["a", "b", "c", "d"]
        );
        let bounded = graph.impact("a", Some(2));
        assert_eq!(bounded.paths.len(), 2);
        assert!(bounded.truncated);
        assert!(!graph.impact("a", Some(3)).truncated);
    }
}
