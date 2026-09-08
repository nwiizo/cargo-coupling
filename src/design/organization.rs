//! Declared organization, lifecycle, and runtime relationships.

use super::{
    context::{Component, DesignContext, RelationshipKind},
    graph::DependencyGraph,
    model::{Coordination, LifecycleGroup, RuntimeRelation},
};
use crate::ProjectMetrics;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub(super) fn component_for<'a>(context: &'a DesignContext, module: &str) -> Option<&'a Component> {
    context.components.iter().find(|component| {
        component
            .modules
            .iter()
            .any(|selector| matches(selector, module))
    })
}

pub(super) fn matches(selector: &str, module: &str) -> bool {
    glob::Pattern::new(selector).is_ok_and(|pattern| pattern.matches(module))
}

pub(super) fn selected(graph: &DependencyGraph, selectors: &[String]) -> Vec<String> {
    selectors
        .iter()
        .flat_map(|selector| graph.select(selector))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn ownership(
    metrics: &ProjectMetrics,
    graph: &DependencyGraph,
    context: &DesignContext,
    root: &Path,
    notes: &mut Vec<String>,
) -> BTreeMap<String, Vec<String>> {
    let path = [".github/CODEOWNERS", "CODEOWNERS", "docs/CODEOWNERS"]
        .into_iter()
        .map(|file| root.join(file))
        .find(|p| p.is_file());
    let rules = match path {
        Some(path) => match fs::read_to_string(&path) {
            Ok(contents) => parse_codeowners(&contents, notes),
            Err(error) => {
                notes.push(format!("Cannot read {}: {error}", path.display()));
                vec![]
            }
        },
        None => {
            notes.push(
                "No CODEOWNERS file; ownership is known only where declared in the design context."
                    .into(),
            );
            vec![]
        }
    };
    graph
        .nodes
        .iter()
        .map(|name| {
            let owners = component_for(context, name)
                .filter(|c| !c.owners.is_empty())
                .map(|c| c.owners.clone())
                .unwrap_or_else(|| {
                    let Some(module) = metrics.modules.get(name) else {
                        return vec![];
                    };
                    let relative = module
                        .path
                        .strip_prefix(root)
                        .unwrap_or(&module.path)
                        .to_string_lossy();
                    owners_for_path(&rules, &relative)
                });
            (name.clone(), owners)
        })
        .collect()
}

struct OwnerRule {
    patterns: Vec<glob::Pattern>,
    owners: Vec<String>,
}

fn parse_codeowners(contents: &str, notes: &mut Vec<String>) -> Vec<OwnerRule> {
    let mut rules = vec![];
    for (index, line) in contents.lines().enumerate() {
        let mut words = vec![];
        let mut word = String::new();
        let mut escape = false;
        for character in line.chars() {
            if escape {
                word.push(character);
                escape = false;
            } else if character == '\\' {
                escape = true;
            } else if character == '#' && word.is_empty() {
                break;
            } else if character.is_whitespace() {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            } else {
                word.push(character);
            }
        }
        if !word.is_empty() {
            words.push(word);
        }
        let Some(pattern) = words.first() else {
            continue;
        };
        if escape || pattern.starts_with('!') || pattern.contains(['[', ']']) {
            notes.push(format!(
                "CODEOWNERS line {} uses an unsupported pattern and was not applied.",
                index + 1
            ));
            continue;
        }
        let rooted = pattern.starts_with('/');
        let pattern = pattern.trim_start_matches('/');
        let descendants = pattern.ends_with('/');
        let pattern = pattern.trim_end_matches('/');
        let prefix = if !rooted && !pattern.contains('/') {
            "**/"
        } else {
            ""
        };
        let globs = if descendants {
            vec![format!("{prefix}{pattern}/**")]
        } else {
            vec![
                format!("{prefix}{pattern}"),
                format!("{prefix}{pattern}/**"),
            ]
        };
        let compiled: Result<Vec<_>, _> = globs.iter().map(|p| glob::Pattern::new(p)).collect();
        match compiled {
            Ok(patterns) => rules.push(OwnerRule {
                patterns,
                owners: words[1..].to_vec(),
            }),
            Err(error) => notes.push(format!("CODEOWNERS line {}: {error}", index + 1)),
        }
    }
    rules
}

fn owners_for_path(rules: &[OwnerRule], path: &str) -> Vec<String> {
    let options = glob::MatchOptions {
        require_literal_separator: true,
        ..Default::default()
    };
    rules
        .iter()
        .rev()
        .find(|rule| rule.patterns.iter().any(|p| p.matches_with(path, options)))
        .map(|rule| rule.owners.clone())
        .unwrap_or_default()
}

pub(super) fn coordination(
    graph: &DependencyGraph,
    owners: &BTreeMap<String, Vec<String>>,
) -> Vec<Coordination> {
    graph.outgoing.iter().flat_map(|(source, targets)| targets.iter().map(move |target| (source,target))).map(|(source,target)| {
        let source_owners = owners.get(source).cloned().unwrap_or_default();
        let target_owners = owners.get(target).cloned().unwrap_or_default();
        let cross_team = !source_owners.is_empty() && !target_owners.is_empty() && !source_owners.iter().any(|owner| target_owners.contains(owner));
        Coordination { source: source.clone(), target: target.clone(), source_owners, target_owners, cross_team, basis: "Declared owners or last matching CODEOWNERS entry. Disjoint owners suggest coordination; they do not measure organizational distance.".into() }
    }).collect()
}

pub(super) fn lifecycle(graph: &DependencyGraph, context: &DesignContext) -> Vec<LifecycleGroup> {
    let mut groups: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for component in &context.components {
        for (kind, unit) in [
            ("build", &component.build_unit),
            ("test", &component.test_unit),
            ("release", &component.release_unit),
        ] {
            if let Some(unit) = unit {
                groups
                    .entry((kind.into(), unit.clone()))
                    .or_default()
                    .extend(selected(graph, &component.modules));
            }
        }
    }
    groups
        .into_iter()
        .map(|((kind, unit), names)| {
            let modules: Vec<_> = names.into_iter().collect();
            let mut unrelated = 0;
            for (index, left) in modules.iter().enumerate() {
                for right in modules.iter().skip(index + 1) {
                    if !graph
                        .outgoing
                        .get(left)
                        .is_some_and(|targets| targets.contains(right))
                        && !graph
                            .outgoing
                            .get(right)
                            .is_some_and(|targets| targets.contains(left))
                    {
                        unrelated += 1;
                    }
                }
            }
            LifecycleGroup {
                kind,
                unit,
                modules,
                pairs_without_code_edges: unrelated,
                origin: "declared".into(),
            }
        })
        .collect()
}

pub(super) fn runtime(graph: &DependencyGraph, context: &DesignContext) -> Vec<RuntimeRelation> {
    context.relationships.iter().flat_map(|relationship| {
        let review = match relationship.kind {
            RelationshipKind::Sequence => "Review whether the required call order is enforced by the API or type-state design.",
            RelationshipKind::Timing => "Review time assumptions, timeouts, late delivery and the test clock; Git co-change is not runtime timing evidence.",
            RelationshipKind::Transaction => "Review atomicity and invariants before moving operations across a boundary; specify rollback/compensation behavior.",
            RelationshipKind::SharedState => "Review identity, consistency, ownership and concurrency controls for the shared data.",
            RelationshipKind::Synchronous => "Review availability propagation and failure handling separately from shared model knowledge.",
            RelationshipKind::Asynchronous => "Review schema evolution, event ordering and duplicate delivery; asynchronous transport does not imply weak knowledge sharing.",
        };
        let targets = graph.select(&relationship.target);
        graph.select(&relationship.source).into_iter().flat_map(move |source| targets.clone().into_iter().map(move |target| RuntimeRelation { id: relationship.id.clone(), source: source.clone(), target, kind: relationship.kind, origin: relationship.origin, evidence: relationship.evidence.clone(), review: review.into() }))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codeowners_uses_last_match_and_supports_directories_and_owner_clearing() {
        let mut notes = vec![];
        let rules = parse_codeowners(
            "* @all\n/src/ @rust\n/src/private/ @core\n/src/private/generated/\n",
            &mut notes,
        );
        assert_eq!(owners_for_path(&rules, "README.md"), ["@all"]);
        assert_eq!(owners_for_path(&rules, "src/lib.rs"), ["@rust"]);
        assert_eq!(owners_for_path(&rules, "src/private/a.rs"), ["@core"]);
        assert!(owners_for_path(&rules, "src/private/generated/a.rs").is_empty());
        assert!(notes.is_empty());
    }
}
