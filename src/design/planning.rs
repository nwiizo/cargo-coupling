//! Compare proposed design changes and review retained decisions.

use super::{
    context::{DesignContext, ReviewTrigger},
    graph::DependencyGraph,
    model::{
        Alternative, ChangeImpact, Coordination, DecisionReview, EdgeEvidence, InheritedExposure,
        Priority, ScenarioResult,
    },
    organization::{self, component_for, selected},
};
use crate::balance::score::normalized_balance as balance;
use std::collections::BTreeMap;

pub(super) fn alternatives(edges: &[EdgeEvidence]) -> Vec<Alternative> {
    let mut pairs: BTreeMap<_, &EdgeEvidence> = BTreeMap::new();
    for edge in edges.iter().filter(|edge| edge.balance < 0.8) {
        pairs
            .entry((&edge.source, &edge.target))
            .and_modify(|current| {
                if edge.balance < current.balance {
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
            alternatives.push(Alternative {
                source: edge.source.clone(), target: edge.target.clone(), action: action.into(),
                current_balance: edge.balance, hypothetical_balance: balance(strength, distance, edge.volatility),
                assumptions: vec![assumption.into()],
                tradeoffs: vec![tradeoff.into(), "Prediction covers the least balanced observation for this pair only; rerun the full analysis after any real change.".into()],
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

pub(super) fn priorities(
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
        if related.is_empty() && planned.is_none() && frozen.is_none() && inherited == 0 {
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
            + inherited
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
    impacts: &[ChangeImpact],
    coordination: &[Coordination],
    diff: Option<&crate::BaselineDiff>,
    context_fingerprint: &str,
    changes_checked: bool,
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
                    if !changes_checked { pending_checks.push("Changed requires --changed-since".into()); }
                    impacts.iter().any(|impact| modules.contains(&impact.origin))
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
        let first = alternatives(&[edge(0.5), edge(1.0)]);
        let reversed = alternatives(&[edge(1.0), edge(0.5)]);
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
            decisions(
                &context,
                &graph,
                &[],
                &[coordination],
                None,
                "current",
                false
            )[0]
            .status,
            "unknown"
        );
    }
}
