//! Compare proposed design changes and review retained decisions.

use super::{
    context::{DesignContext, ReviewTrigger},
    graph::DependencyGraph,
    model::{
        Alternative, ChangeImpact, Coordination, DecisionReview, EdgeEvidence, EndpointKind,
        InheritedExposure, Priority, ScenarioResult,
    },
    organization::{self, component_for, selected},
};
use crate::balance::score::normalized_balance as balance;
use std::collections::{BTreeMap, BTreeSet};

/// Change proximity first, then the shared priority ordering. Names break ties
/// at the call site so bounded reports remain deterministic.
pub(super) struct ReviewOrder {
    modules: BTreeMap<String, (usize, usize)>,
}

impl ReviewOrder {
    pub(super) fn new(impacts: &[ChangeImpact], priorities: &[Priority]) -> Self {
        let mut modules: BTreeMap<_, _> = priorities
            .iter()
            .enumerate()
            .map(|(rank, priority)| (priority.module.clone(), (usize::MAX, rank)))
            .collect();
        for impact in impacts {
            for (module, depth) in std::iter::once((&impact.origin, 0))
                .chain(
                    impact
                        .reachability
                        .paths
                        .iter()
                        .map(|path| (&path.module, path.path.len() - 1)),
                )
                .chain(
                    impact
                        .contextual_reachability
                        .paths
                        .iter()
                        .map(|path| (&path.module, path.path.len() - 1)),
                )
            {
                let entry = modules
                    .entry(module.clone())
                    .or_insert((usize::MAX, usize::MAX));
                entry.0 = entry.0.min(depth);
            }
        }
        Self { modules }
    }

    pub(super) fn module(&self, module: &str) -> (usize, usize) {
        self.modules
            .get(module)
            .copied()
            .unwrap_or((usize::MAX, usize::MAX))
    }

    pub(super) fn pair(&self, source: &str, target: &str) -> (usize, usize) {
        let (left_depth, left_priority) = self.module(source);
        let (right_depth, right_priority) = self.module(target);
        (
            left_depth.min(right_depth),
            left_priority.min(right_priority),
        )
    }
}

pub(super) fn alternatives<'a>(
    edges: impl IntoIterator<Item = &'a EdgeEvidence>,
    context: &DesignContext,
) -> Vec<Alternative> {
    let mut pairs: BTreeMap<_, &EdgeEvidence> = BTreeMap::new();
    for edge in edges.into_iter().filter(|edge| {
        edge.source_kind == EndpointKind::Module
            && edge.target_kind == EndpointKind::Module
            && edge.source != edge.target
            && edge.balance < 0.8
    }) {
        pairs
            .entry((&edge.source, &edge.target))
            .and_modify(|current| {
                // Equal scores can arise from different dimensions. Prefer more
                // shared knowledge, then greater distance/volatility, consistently.
                let order = edge
                    .balance
                    .total_cmp(&current.balance)
                    .then_with(|| current.strength.total_cmp(&edge.strength))
                    .then_with(|| current.distance.total_cmp(&edge.distance))
                    .then_with(|| current.volatility.total_cmp(&edge.volatility));
                if order.is_lt() {
                    *current = edge;
                }
            })
            .or_insert(edge);
    }
    let mut alternatives = vec![];
    for edge in pairs.into_values() {
        for (action, strength, distance, assumption, tradeoff) in [
            (
                "retain",
                edge.strength,
                edge.distance,
                "Current change rate and boundary assumptions remain valid.",
                "Revisit if business requirements, ownership or release units change.",
            ),
            (
                "hide-knowledge",
                0.25,
                edge.distance,
                "An integration-specific interface can hide implementation knowledge; merely adding a trait is insufficient.",
                "Adds translation and maintenance; can be pointless if it mirrors the internal model.",
            ),
            (
                "move-closer",
                edge.strength,
                0.25,
                "The coupled responsibilities can be maintained in one boundary.",
                "Increases shared lifecycle; moving unrelated functionality increases local complexity.",
            ),
            (
                "separate-responsibilities",
                edge.strength,
                1.0,
                "Responsibilities really can evolve independently with the current interface.",
                "Increases coordination and rollout effort if knowledge still needs to change together.",
            ),
        ] {
            let mut constraints = vec![];
            if action != "retain" {
                for relation in &context.relationships {
                    let connects = |left: &str, right: &str| {
                        organization::matches(&relation.source, left)
                            && organization::matches(&relation.target, right)
                    };
                    if connects(&edge.source, &edge.target) || connects(&edge.target, &edge.source)
                    {
                        constraints.push(format!("Relationship '{}': {:?}. Preserve this requirement before adopting the option: {}", relation.id, relation.kind, relation.evidence));
                    }
                }
                for rule in &context.rules {
                    let includes = |module: &str| {
                        rule.modules
                            .iter()
                            .any(|selector| organization::matches(selector, module))
                    };
                    if includes(&edge.source) && includes(&edge.target) {
                        constraints.push(format!(
                            "Shared rule '{}': {}. Verify both implementations after the change.",
                            rule.id, rule.description
                        ));
                    }
                }
                let source =
                    component_for(context, &edge.source).and_then(|c| c.release_unit.as_deref());
                let target =
                    component_for(context, &edge.target).and_then(|c| c.release_unit.as_deref());
                match (source, target, action) {
                    (Some(left), Some(right), "move-closer") if left != right => constraints.push(format!("Separate release units '{left}' and '{right}': confirm that a shared release lifecycle is acceptable.")),
                    (Some(left), Some(right), "separate-responsibilities") if left == right => constraints.push(format!("Shared release unit '{left}': code separation alone does not establish independent delivery.")),
                    _ => {},
                }
            }
            alternatives.push(Alternative {
                source: edge.source.clone(), target: edge.target.clone(), action: action.into(),
                current_balance: edge.balance, hypothetical_balance: balance(strength, distance, edge.volatility),
                assumptions: vec![assumption.into()],
                tradeoffs: vec![tradeoff.into(), "Prediction covers the least balanced observation for this pair only; rerun the full analysis after any real change.".into()],
                status: if constraints.is_empty() { "hypothetical" } else { "requires-review" }.into(),
                constraints,
            });
        }
    }
    alternatives
}

pub(super) fn scenarios(
    edges: &[EdgeEvidence],
    context: &DesignContext,
    coverage: &mut Vec<String>,
) -> Vec<ScenarioResult> {
    context.scenarios.iter().map(|scenario| {
        let mut current = 0.0;
        let mut hypothetical = 0.0;
        let mut count = 0;
        for edge in edges {
            let changes: Vec<_> = scenario.changes.iter().filter(|change| organization::matches(&change.source, &edge.source) && organization::matches(&change.target, &edge.target)).collect();
            if changes.is_empty() { continue; }
            let (mut strength, mut distance, mut volatility) = (edge.strength, edge.distance, edge.volatility);
            for change in changes {
                strength = change.strength.unwrap_or(strength);
                distance = change.distance.unwrap_or(distance);
                volatility = change.volatility.unwrap_or(volatility);
            }
            current += edge.balance;
            hypothetical += balance(strength, distance, volatility);
            count += 1;
        }
        if count == 0 { coverage.push(format!("Scenario '{}' matched no observed edges in this scope.", scenario.name)); }
        ScenarioResult {
            name: scenario.name.clone(), description: scenario.description.clone(), affected_edges: count,
            current_balance: (count > 0).then(|| current / count as f64),
            hypothetical_balance: (count > 0).then(|| hypothetical / count as f64),
            assumptions: vec!["Dimension changes are supplied assumptions; scores average matched observations only and do not predict whole-system quality. Later matching changes override earlier values for each dimension.".into(), "Source code, business classifications and the observed report are not changed.".into()],
        }
    }).collect()
}

pub(crate) fn priorities(
    graph: &DependencyGraph,
    context: &DesignContext,
    issues: &[crate::CouplingIssue],
    exposures: &[InheritedExposure],
) -> Vec<Priority> {
    let mut priorities = vec![];
    for module in &graph.nodes {
        let component = component_for(context, module);
        let related: Vec<_> = issues
            .iter()
            .filter(|issue| {
                graph.resolve(&issue.source).as_ref() == Some(module)
                    || graph.resolve(&issue.target).as_ref() == Some(module)
            })
            .collect();
        let planned = component.and_then(|c| c.planned_changes.clone());
        let frozen = component.and_then(|c| c.frozen_reason.clone());
        let inherited = exposures.iter().filter(|e| e.module == *module).count();
        if related.is_empty() && planned.is_none() && frozen.is_none() {
            continue;
        }
        let affected_modules = graph.impact(module, None).paths.len();
        let business_value = component.map(|c| c.business_value).unwrap_or(1.0);
        let effort_days = component.and_then(|c| c.effort_days);
        let severity_weight: usize = related
            .iter()
            .map(|issue| match issue.severity {
                crate::Severity::Critical => 8,
                crate::Severity::High => 4,
                crate::Severity::Medium => 2,
                crate::Severity::Low => 1,
            })
            .sum();
        let priority_score = (severity_weight
            + affected_modules
            + usize::from(planned.is_some()) * 4
            + usize::from(frozen.is_some()) * 4) as f64
            * business_value;
        let mut reasons = vec![format!(
            "{} finding(s), {affected_modules} reachable consumers, {inherited} inherited exposure path(s)",
            related.len()
        )];
        if planned.is_some() {
            reasons.push("Upcoming changes make the boundary worth reviewing now.".into());
        }
        if frozen.is_some() {
            reasons.push(
                "Low observed churn may reflect deliberately frozen code, not low need for change."
                    .into(),
            );
        }
        priorities.push(Priority {
            module: module.clone(),
            reasons,
            affected_modules,
            issue_count: related.len(),
            business_value,
            effort_days,
            planned_changes: planned,
            frozen_reason: frozen,
            priority_score,
            value_per_effort: effort_days.map(|effort| priority_score / effort),
        });
    }
    priorities.sort_by(|a, b| {
        b.priority_score
            .total_cmp(&a.priority_score)
            .then_with(|| a.module.cmp(&b.module))
    });
    priorities
}

pub(super) fn decisions(
    context: &DesignContext,
    graph: &DependencyGraph,
    changed_modules: Option<&BTreeSet<String>>,
    coordination: &[Coordination],
    diff: Option<&crate::BaselineDiff>,
    context_fingerprint: &str,
) -> Vec<DecisionReview> {
    context.decisions.iter().map(|decision| {
        let modules = selected(graph, &decision.modules);
        let mut triggered = vec![];
        let mut pending_checks = vec![];
        for trigger in &decision.review_on {
            let active = match trigger {
                ReviewTrigger::PlannedChange => modules.iter().any(|m| component_for(context, m).is_some_and(|c| c.planned_changes.is_some())),
                ReviewTrigger::CrossTeam => {
                    let related: Vec<_> = coordination.iter().filter(|c| modules.contains(&c.source) || modules.contains(&c.target)).collect();
                    if related.iter().any(|c| c.source_owners.is_empty() || c.target_owners.is_empty()) {
                        pending_checks.push("CrossTeam requires owners for both ends of each related dependency".into());
                    }
                    related.iter().any(|c| c.cross_team)
                }
                ReviewTrigger::Changed => {
                    if let Some(changed_modules) = changed_modules {
                        modules.iter().any(|module| changed_modules.contains(module))
                    } else {
                        pending_checks.push("Changed requires --changed-since".into());
                        false
                    }
                }
                ReviewTrigger::ContextChanged => {
                    if decision.context_fingerprint.is_none() { pending_checks.push("ContextChanged requires a saved context_fingerprint".into()); }
                    decision.context_fingerprint.as_deref().is_some_and(|old| old != context_fingerprint)
                }
                ReviewTrigger::NewIssue | ReviewTrigger::WorsenedIssue => {
                    if let Some(diff) = diff {
                        let issues = if *trigger == ReviewTrigger::NewIssue { &diff.new_issues } else { &diff.worsened_issues };
                        issues.iter().any(|issue| modules.iter().any(|m| graph.resolve(&issue.source).as_ref() == Some(m) || graph.resolve(&issue.target).as_ref() == Some(m)))
                    } else {
                        pending_checks.push(format!("{trigger:?} requires --baseline or --changed-since"));
                        false
                    }
                }
            };
            if active { triggered.push(format!("{trigger:?}")); }
        }
        if modules.is_empty() { pending_checks.push("No selected modules; decision could not be evaluated.".into()); }
        DecisionReview {
            id: decision.id.clone(), modules, reason: decision.reason.clone(),
            status: if !triggered.is_empty() { "review" } else if !pending_checks.is_empty() { "unknown" } else { "retain" }.into(),
            triggered, pending_checks,
            evidence: decision.evidence.clone(), rejected_options: decision.rejected_options.clone(),
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(strength: f64) -> EdgeEvidence {
        EdgeEvidence {
            source: "consumer".into(),
            target: "provider".into(),
            source_kind: EndpointKind::Module,
            target_kind: EndpointKind::Module,
            observed_usage: None,
            inferred_strength: "inferred".into(),
            origin: "syntax".into(),
            reason: String::new(),
            file_path: None,
            line: 1,
            strength,
            distance: 1.0,
            volatility: 1.0,
            balance: balance(strength, 1.0, 1.0),
            unknowns: vec![],
        }
    }

    #[test]
    fn alternatives_use_the_worst_observation_independent_of_input_order() {
        let first = alternatives(&[edge(0.5), edge(1.0)], &DesignContext::default());
        let reversed = alternatives(&[edge(1.0), edge(0.5)], &DesignContext::default());
        assert_eq!(first.len(), 4);
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&reversed).unwrap()
        );
        assert_eq!(first[0].current_balance, 0.0);
        assert_eq!(
            first
                .iter()
                .find(|option| option.action == "hide-knowledge")
                .unwrap()
                .hypothetical_balance,
            0.75
        );
    }

    #[test]
    fn equal_balance_options_do_not_depend_on_observation_order() {
        let mut close = edge(1.0);
        close.distance = 0.5;
        close.balance = balance(close.strength, close.distance, close.volatility);
        let distant = edge(0.5);
        assert_eq!(close.balance, distant.balance);

        let first = alternatives(&[close.clone(), distant.clone()], &DesignContext::default());
        let reversed = alternatives(&[distant, close], &DesignContext::default());
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&reversed).unwrap(),
            "equal current scores must not change the proposed outcomes"
        );
    }

    #[test]
    fn module_focus_does_not_establish_a_changed_decision() {
        let context = DesignContext::parse(
            "version=1\n[[decisions]]\nid='keep'\nmodules=['consumer']\nreason='Keep the boundary'\nreview_on=['changed']\n",
        ).unwrap();
        let graph = DependencyGraph {
            nodes: ["consumer".into()].into(),
            ..Default::default()
        };
        let review = decisions(&context, &graph, None, &[], None, "current");
        assert_eq!(review[0].status, "unknown");
        assert!(review[0].triggered.is_empty());
        assert!(!review[0].pending_checks.is_empty());
    }

    #[test]
    fn unknown_ownership_cannot_confirm_a_retained_cross_team_decision() {
        let context: DesignContext = toml::from_str("version=1\n[[decisions]]\nid='keep'\nmodules=['consumer']\nreason='One team'\nreview_on=['cross-team']\n").unwrap();
        let graph = DependencyGraph {
            nodes: ["consumer".into(), "provider".into()].into(),
            ..Default::default()
        };
        let coordination = Coordination {
            source: "consumer".into(),
            target: "provider".into(),
            source_owners: vec!["@one".into()],
            target_owners: vec![],
            cross_team: false,
            basis: String::new(),
        };
        assert_eq!(
            decisions(&context, &graph, None, &[coordination], None, "current",)[0].status,
            "unknown"
        );
    }
}
