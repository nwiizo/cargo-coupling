//! Missing evidence is an actionable question, never a positive safety claim.

use super::{
    context::DesignContext,
    model::{Coordination, ReviewQuestion},
    organization::{component_for, matches},
};
use crate::{ProjectMetrics, metrics::Subdomain};
use std::collections::BTreeSet;

pub(super) fn review_questions(
    modules: &BTreeSet<String>,
    metrics: &ProjectMetrics,
    context: &DesignContext,
    coordination: &[Coordination],
) -> Vec<ReviewQuestion> {
    let mut questions = vec![];
    for module in modules {
        let component = component_for(context, module);
        if component.is_none_or(|c| c.planned_changes.is_none() && c.frozen_reason.is_none()) {
            let subdomain = metrics.modules.get(module).and_then(|m| m.subdomain);
            questions.push(ReviewQuestion {
                id: format!("future-change:{module}"), modules: vec![module.clone()],
                question: if subdomain == Some(Subdomain::Generic) {
                    "Could the provider or implementation change despite stable functionality, or is change deliberately deferred?"
                } else { "What change is expected here, or why is change deliberately deferred?" }.into(),
                reason: if subdomain == Some(Subdomain::Generic) {
                    "Generic functionality can be stable while its provider or implementation changes."
                } else if subdomain.is_some() {
                    "A subdomain classification estimates volatility; it does not identify the upcoming change."
                } else { "Past editing frequency alone cannot establish future change needs." }.into(),
                context_section: "components[].planned_changes / frozen_reason".into(),
            });
        }
        if coordination.iter().any(|relation| {
            (relation.source == *module && relation.source_owners.is_empty())
                || (relation.target == *module && relation.target_owners.is_empty())
        }) {
            questions.push(ReviewQuestion {
                id: format!("ownership:{module}"),
                modules: vec![module.clone()],
                question: "Who needs to coordinate changes at this boundary?".into(),
                reason: "An observed dependency has no declared owner; source proximity does not establish team proximity.".into(),
                context_section: "components[].owners / CODEOWNERS".into(),
            });
        }
    }
    for rule in &context.rules {
        let related: Vec<_> = modules
            .iter()
            .filter(|module| {
                rule.modules
                    .iter()
                    .any(|selector| matches(selector, module))
            })
            .cloned()
            .collect();
        if !related.is_empty() && rule.verification.is_empty() {
            questions.push(ReviewQuestion {
                id: format!("verify-rule:{}", rule.id), modules: related,
                question: format!("Which behavioral check would catch a violation of rule '{}' after this change?", rule.id),
                reason: "A shared rule is a declared requirement, not proof that an implementation or its tests preserve it.".into(),
                context_section: "rules[].verification".into(),
            });
        }
    }
    questions
}
