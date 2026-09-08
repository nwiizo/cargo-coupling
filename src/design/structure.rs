//! Hierarchical boundaries and external API exposure from existing observations.

use super::{
    graph::DependencyGraph,
    model::{BoundarySummary, ExternalExposure},
    source::{SourceInventory, type_mentions},
};
use crate::{Distance, ProjectMetrics};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) fn hierarchy(
    graph: &DependencyGraph,
    inventory: &SourceInventory,
    metrics: &ProjectMetrics,
) -> Vec<BoundarySummary> {
    let mut result = vec![];
    let mut item_edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut module_items: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for item in &inventory.items {
        let id = format!("{}::{}", item.module, item.name);
        module_items
            .entry(item.module.clone())
            .or_default()
            .insert(id.clone());
        for call in &item.calls {
            let simple = call.rsplit("::").next().unwrap_or(call);
            let mut candidates = inventory.items.iter().filter(|target| {
                target.name == *simple
                    || format!("{}::{}", target.module, target.name).ends_with(call)
            });
            if let Some(target) = candidates.next()
                && candidates.next().is_none()
            {
                item_edges
                    .entry(id.clone())
                    .or_default()
                    .insert(format!("{}::{}", target.module, target.name));
            }
        }
    }
    for (module, items) in &module_items {
        result.push(summarize("module", module, items, &item_edges));
        for item in items {
            result.push(summarize(
                "item",
                item,
                &BTreeSet::from([item.clone()]),
                &item_edges,
            ));
        }
    }
    let mut packages: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for name in &graph.nodes {
        let package = metrics
            .couplings
            .iter()
            .find_map(|edge| {
                if graph.resolve(&edge.source).as_ref() == Some(name) {
                    edge.source_crate.clone()
                } else if graph.resolve(&edge.target).as_ref() == Some(name) {
                    edge.target_crate.clone()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "selected-sources".into());
        packages.entry(package).or_default().insert(name.clone());
    }
    for (name, nodes) in packages {
        result.push(summarize("package", &name, &nodes, &graph.outgoing));
    }
    result.push(summarize(
        "workspace",
        "analysis-scope",
        &graph.nodes,
        &graph.outgoing,
    ));
    result
}

fn summarize(
    level: &str,
    name: &str,
    members: &BTreeSet<String>,
    edges: &BTreeMap<String, BTreeSet<String>>,
) -> BoundarySummary {
    let (mut internal, mut incoming, mut outgoing) = (0, 0, 0);
    let mut neighbours: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (source, targets) in edges {
        for target in targets {
            match (members.contains(source), members.contains(target)) {
                (true, true) => {
                    internal += 1;
                    neighbours
                        .entry(source.clone())
                        .or_default()
                        .insert(target.clone());
                    neighbours
                        .entry(target.clone())
                        .or_default()
                        .insert(source.clone());
                }
                (true, false) => outgoing += 1,
                (false, true) => incoming += 1,
                _ => {}
            }
        }
    }
    let mut unseen = members.clone();
    let mut groups = vec![];
    while let Some(start) = unseen.pop_first() {
        let mut group = BTreeSet::from([start.clone()]);
        let mut queue = VecDeque::from([start]);
        while let Some(node) = queue.pop_front() {
            for next in neighbours.get(&node).into_iter().flatten() {
                if unseen.remove(next) {
                    group.insert(next.clone());
                    queue.push_back(next.clone());
                }
            }
        }
        groups.push(group.into_iter().collect());
    }
    let observation = if groups.len() > 1 {
        "Multiple groups have no observed internal connection. Compare their change reasons before changing this boundary."
    } else {
        "Observed internal connections form one group. This alone does not prove shared responsibility."
    };
    BoundarySummary {
        level: level.into(),
        name: name.into(),
        members: members.iter().cloned().collect(),
        internal_edges: internal,
        incoming_edges: incoming,
        outgoing_edges: outgoing,
        internal_groups: groups,
        observation: observation.into(),
    }
}

pub(super) fn external_exposure(
    metrics: &ProjectMetrics,
    graph: &DependencyGraph,
    inventory: &SourceInventory,
) -> Vec<ExternalExposure> {
    let mut crates: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for edge in &metrics.couplings {
        if edge.distance != Distance::DifferentCrate {
            continue;
        }
        let name = edge.target_crate.clone().unwrap_or_else(|| {
            edge.target
                .split("::")
                .next()
                .unwrap_or(&edge.target)
                .to_string()
        });
        if ["std", "core", "alloc", "self", "crate", "super"].contains(&name.as_str()) {
            continue;
        }
        crates.entry(name).or_default().insert(
            graph
                .resolve(&edge.source)
                .unwrap_or_else(|| edge.source.clone()),
        );
    }
    crates.into_iter().map(|(crate_name, direct)| {
        let rust_name = crate_name.replace('-', "_");
        let public_items = inventory.items.iter().filter(|item| item.public && item.signature_types.iter().any(|ty| type_mentions(ty, &rust_name))).cloned().collect();
        let mut affected = BTreeMap::new();
        for module in &direct {
            for impact in graph.impact(module, None).paths {
                affected.entry(impact.module.clone()).and_modify(|existing: &mut super::graph::ImpactPath| { if impact.path.len() < existing.path.len() { *existing = impact.clone(); } }).or_insert(impact);
            }
        }
        let replacement_boundaries = direct.iter().filter(|module| graph.incoming.get(*module).is_some_and(|consumers| !consumers.is_empty())).cloned().collect();
        ExternalExposure { crate_name, public_items, direct_modules: direct.into_iter().collect(), affected: affected.into_values().collect(), replacement_boundaries, unknowns: vec!["Public signature exposure is syntax-based; macro expansion, type aliases and downstream repositories may add consumers.".into(), "Replacement boundaries are review candidates; an adapter is useful only if it hides provider-specific knowledge.".into()] }
    }).collect()
}
