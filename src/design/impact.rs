//! Declared shared rules extend review scope, never the observed code graph.

use super::{
    changes::ChangeSet,
    context::DesignContext,
    graph::DependencyGraph,
    model::{ChangeImpact, ContextImpact, ContextPath, ImpactStep, TestCandidate},
    organization::selected,
    source::{SourceInventory, TestKind},
};
use crate::ProjectMetrics;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn change_impact(
    changes: &ChangeSet,
    graph: &DependencyGraph,
    inventory: &SourceInventory,
    metrics: &ProjectMetrics,
    depth: Option<usize>,
    context: &DesignContext,
    explicit_focus: Option<&str>,
) -> (Vec<ChangeImpact>, BTreeSet<String>) {
    let mut origins: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file in &changes.files {
        for (name, module) in &metrics.modules {
            if module.path.ends_with(&file.path)
                || file
                    .previous_path
                    .as_ref()
                    .is_some_and(|path| module.path.ends_with(path))
            {
                origins
                    .entry(name.clone())
                    .or_default()
                    .extend(file.items.iter().map(|item| item.name.clone()));
            }
        }
        for item in &file.items {
            if let Some(module) = graph.resolve(&item.module) {
                origins.entry(module).or_default().insert(item.name.clone());
            }
        }
    }
    let changed_modules = origins.keys().cloned().collect();
    if let Some(origin) = explicit_focus {
        origins.entry(origin.into()).or_default();
    }
    let impact = origins.into_iter().map(|(origin, items)| {
        let reachability = graph.impact(&origin, depth);
        let contextual_reachability = contextual_impact(graph, context, &origin, depth);
        let mut tests = vec![];
        for test in inventory.items.iter().filter(|item| item.test_kind.is_test_function()) {
            let direct = test.calls.iter().any(|call| items.iter().any(|item| call.rsplit("::").next() == item.rsplit("::").next()));
            let test_module = graph.resolve(&test.module).or_else(|| {
                graph.nodes.iter().filter(|module| test.module.starts_with(&format!("{module}::"))).max_by_key(|module| module.len()).cloned()
            });
            let path = test_module.as_ref().and_then(|module| reachability.paths.iter().find(|p| &p.module == module));
            let declared_path = test_module.as_ref().and_then(|module| contextual_reachability.paths.iter().find(|p| &p.module == module));
            let local = test_module.as_ref() == Some(&origin);
            if direct || local || path.is_some() || declared_path.is_some() {
                let relevance = if direct { 0 } else if local { 1 } else if path.is_some() { 2 } else { 3 };
                let mut reason = if direct {
                    "Test calls a changed item by name; confirm name resolution."
                } else if local {
                    "Test is in the changed module; this is a conservative candidate."
                } else if path.is_some() {
                    "Test belongs to a transitively affected module."
                } else {
                    "Test belongs to a module reached through a declared shared rule; validate the rule and test behavior."
                }.to_string();
                if test.test_kind == TestKind::Ignored {
                    reason.push_str(" Marked #[ignore]; requires explicit execution with --ignored or --include-ignored.");
                }
                tests.push((relevance, TestCandidate {
                    item: test.clone(),
                    reason,
                    path: path.map(|p| p.path.clone()).or_else(|| declared_path.map(|p| p.path.clone())).unwrap_or_else(|| vec![origin.clone()]),
                }));
            }
        }
        tests.sort_by(|(left_rank, left), (right_rank, right)| {
            (left_rank, left.path.len(), &left.item.file_path, left.item.line, &left.item.name)
                .cmp(&(right_rank, right.path.len(), &right.item.file_path, right.item.line, &right.item.name))
        });
        let tests = tests.into_iter().map(|(_, test)| test).collect();
        ChangeImpact { origin, changed_items: items.into_iter().collect(), reachability, contextual_reachability, tests }
    }).collect();
    (impact, changed_modules)
}

pub fn contextual_impact(
    graph: &DependencyGraph,
    context: &DesignContext,
    origin: &str,
    depth: Option<usize>,
) -> ContextImpact {
    let mut combined = graph.clone();
    let mut rules: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for rule in &context.rules {
        let modules = selected(graph, &rule.modules);
        for source in &modules {
            for target in modules.iter().filter(|target| *target != source) {
                combined
                    .incoming
                    .entry(source.clone())
                    .or_default()
                    .insert(target.clone());
                rules
                    .entry((source.clone(), target.clone()))
                    .or_default()
                    .push(rule.id.clone());
            }
        }
    }
    let reachable = combined.impact(origin, depth);
    let paths = reachable
        .paths
        .into_iter()
        .filter_map(|path| {
            let steps: Vec<_> = path
                .path
                .windows(2)
                .map(|pair| {
                    let mut rule_ids = rules
                        .get(&(pair[0].clone(), pair[1].clone()))
                        .cloned()
                        .unwrap_or_default();
                    rule_ids.sort();
                    ImpactStep {
                        source: pair[0].clone(),
                        target: pair[1].clone(),
                        rule_ids,
                        code_dependency: graph
                            .incoming
                            .get(&pair[0])
                            .is_some_and(|targets| targets.contains(&pair[1])),
                    }
                })
                .collect();
            steps
                .iter()
                .any(|step| !step.rule_ids.is_empty())
                .then_some(ContextPath {
                    module: path.module,
                    path: path.path,
                    steps,
                })
        })
        .collect();
    ContextImpact {
        paths,
        max_depth: depth,
        truncated: reachable.truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_rule_reaches_code_consumers_and_respects_depth_and_cycles() {
        let graph = DependencyGraph {
            nodes: ["a".into(), "b".into(), "c".into()].into(),
            incoming: [("b".into(), ["c".into()].into())].into(),
            ..Default::default()
        };
        let context = DesignContext::parse(
            "version=1\n[[rules]]\nid='price'\nmodules=['a','b']\ndescription='same price'\n",
        )
        .unwrap();
        let bounded = contextual_impact(&graph, &context, "a", Some(1));
        assert_eq!(bounded.paths.len(), 1);
        assert!(bounded.truncated);
        let full = contextual_impact(&graph, &context, "a", None);
        assert_eq!(full.paths[1].path, ["a", "b", "c"]);
        assert_eq!(full.paths[1].steps[0].rule_ids, ["price"]);
        assert!(full.paths[1].steps[1].code_dependency);
        assert!(!full.truncated);
        assert!(graph.impact("a", None).paths.is_empty());
    }
}
