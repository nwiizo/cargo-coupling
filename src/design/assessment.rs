//! Assemble observations and declared context into actionable design reports.

use super::{
    changes::{self, Provenance},
    context::{self, DesignContext},
    graph::DependencyGraph,
    impact::change_impact,
    model::*,
    organization::{self, selected},
    planning::{ReviewOrder, alternatives, decisions, priorities, scenarios},
    source::{SourceInventory, fingerprint},
    structure,
};
use crate::balance::score::normalized_balance as balance;
use crate::{CompiledConfig, IssueThresholds, ProjectMetrics};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

#[derive(Debug, Clone, Copy)]
pub struct AssessmentRequest<'a> {
    pub path: &'a Path,
    pub context_path: Option<&'a Path>,
    pub changed_since: Option<&'a str>,
    pub baseline: Option<&'a str>,
    pub focus: Option<&'a str>,
    pub max_depth: Option<usize>,
    pub git_months: usize,
    pub git_used: bool,
}

pub fn assess(
    metrics: &ProjectMetrics,
    config: &CompiledConfig,
    thresholds: &IssueThresholds,
    request: AssessmentRequest<'_>,
) -> io::Result<DesignAssessment> {
    let (context, context_path) =
        context::load_context(request.path, request.context_path).map_err(io::Error::other)?;
    let inventory = SourceInventory::read(metrics, config.exclude_tests);
    let graph = DependencyGraph::from_metrics(metrics);
    let explicit_focus = request
        .focus
        .map(|name| {
            graph.resolve(name).ok_or_else(|| {
                io::Error::other(format!(
                    "Unknown or ambiguous focus module '{name}'; use its complete name"
                ))
            })
        })
        .transpose()?;
    let root = changes::git_root(request.path).unwrap_or_else(|| {
        let path = fs::canonicalize(request.path).unwrap_or_else(|_| request.path.to_path_buf());
        if path.is_file() {
            path.parent().unwrap_or(&path).to_path_buf()
        } else {
            path
        }
    });
    let mut fingerprint_context = context.clone();
    for decision in &mut fingerprint_context.decisions {
        decision.context_fingerprint = None;
    }
    let context_text = serde_json::to_vec(&fingerprint_context).map_err(io::Error::other)?;
    let context_fingerprint = fingerprint(&context_text);
    let mut effective_settings = config.effective_settings();
    effective_settings.thresholds.max_dependencies = thresholds.max_dependencies;
    effective_settings.thresholds.max_dependents = thresholds.max_dependents;
    let config_fingerprint = Some(fingerprint(
        &serde_json::to_vec(&effective_settings).map_err(io::Error::other)?,
    ));
    let provenance = Provenance {
        analyzer_version: env!("CARGO_PKG_VERSION").into(),
        scoring_version: crate::balance::score::SCORING_VERSION.into(),
        scope: fs::canonicalize(request.path)?.display().to_string(),
        revision: changes::resolve_ref(&root, "HEAD").ok(),
        dirty: changes::git(&root, &["status", "--porcelain", "-z"])
            .ok()
            .map(|status| !status.is_empty()),
        source_fingerprint: changes::source_fingerprint(&inventory, &root),
        config_fingerprint,
        effective_settings,
        context_fingerprint: context_fingerprint.clone(),
        git_months: request.git_months,
        git_used: request.git_used,
        tests_excluded: config.exclude_tests,
        max_depth: request.max_depth,
        thresholds: BTreeMap::from([
            ("max_dependencies".into(), thresholds.max_dependencies),
            ("max_dependents".into(), thresholds.max_dependents),
        ]),
    };
    let mut coverage = inventory.notes.clone();
    if context_path.is_none() {
        coverage.push("No design context: future plans, lifecycle units, business rules and runtime relationships are unknown.".into());
    }
    coverage.push("Rust syntax and dependency reachability are observations; inferred integration strength is not proof of shared business meaning. Macros, inactive cfg selection and full Rust type resolution are not covered.".into());
    coverage.push("Runtime coverage is limited to supplied declarations/observations. Git co-change measures editing history, not runtime ordering.".into());
    coverage.push("The scored frame is Rust source boundaries with fixed structural distance categories. Selecting a scope does not renormalize distances; item/package hierarchy summaries do not independently rescore those levels.".into());
    coverage.push(format!(
        "{} source parse failure(s); {} unresolved graph endpoint(s).",
        metrics.parse_failures,
        graph.unresolved.len()
    ));
    if !request.git_used {
        coverage
            .push("Git volatility was not analyzed; observed change counts are unknown.".into());
    }
    if config.exclude_tests {
        coverage.push("Test code was excluded; test candidates are incomplete.".into());
    }
    validate_bindings(&context, &graph, &mut coverage)?;

    let current_report = crate::analyze_project_balance_with_thresholds(metrics, thresholds);
    let reference = request.baseline.or(request.changed_since);
    let baseline_analysis = reference
        .map(|reference| {
            crate::analyze_ref(
                request.path,
                config,
                thresholds,
                reference,
                request.git_months,
                request.git_used,
            )
            .map_err(io::Error::other)
        })
        .transpose()?;
    let diff = baseline_analysis
        .as_ref()
        .map(|base| crate::diff_ref_analysis(base, &current_report));
    let baseline = diff.as_ref().map(|diff| BaselineSummary {
        reference: reference.unwrap_or_default().into(), new_issues: diff.new_issues.len(), worsened_issues: diff.worsened_issues.len(), resolved_issues: diff.resolved_issues.len(), score_delta: diff.score_delta,
        reference_commit: changes::resolve_ref(&root, reference.unwrap_or_default()).ok(),
        settings_fingerprint: provenance.config_fingerprint.clone(),
        notes: vec!["Both source revisions are analyzed with today's analyzer, scoring rules, thresholds and effective configuration. This is a structural comparison under fixed settings, not a reproduction of an old published score. The current context supplies planning information; Git history uses the requested observation window.".into()],
        findings: [("new", &diff.new_issues), ("worsened", &diff.worsened_issues), ("resolved", &diff.resolved_issues)].into_iter().flat_map(|(change, issues)| issues.iter().map(move |issue| ComparedFinding { change: change.into(), source: issue.source.clone(), target: issue.target.clone(), issue_type: issue.issue_type.to_string(), severity: issue.severity.to_string(), description: issue.description.clone(), balance_score: issue.balance_score })).collect(),
    });
    let source_paths: Vec<_> = metrics
        .modules
        .values()
        .chain(
            baseline_analysis
                .as_ref()
                .into_iter()
                .flat_map(|base| base.metrics.modules.values()),
        )
        .map(|module| module.path.clone())
        .collect();
    let changes = request
        .changed_since
        .map(|reference| changes::read_changes(request.path, reference, &inventory, &source_paths))
        .transpose()?
        .unwrap_or_default();
    // Current-only impact can borrow the analysis. A baseline needs a merged
    // graph so deleted modules and their former consumers remain reachable.
    let combined_metrics = baseline_analysis.as_ref().map(|base| {
        let mut combined = ProjectMetrics {
            modules: metrics.modules.clone(),
            couplings: metrics.couplings.clone(),
            ..Default::default()
        };
        for (name, module) in &base.metrics.modules {
            combined
                .modules
                .entry(name.clone())
                .or_insert_with(|| module.clone());
        }
        combined.couplings.extend(base.metrics.couplings.clone());
        combined
    });
    let impact_metrics = combined_metrics.as_ref().unwrap_or(metrics);
    let impact_graph = DependencyGraph::from_metrics(impact_metrics);
    let (mut impact, changed_modules) = change_impact(
        &changes,
        &impact_graph,
        &inventory,
        impact_metrics,
        request.max_depth,
        &context,
        explicit_focus.as_deref(),
    );
    let focused = request.focus.is_some() || request.changed_since.is_some();
    let focus: BTreeSet<_> = impact
        .iter()
        .flat_map(|impact| {
            std::iter::once(impact.origin.clone())
                .chain(
                    impact
                        .reachability
                        .paths
                        .iter()
                        .map(|path| path.module.clone()),
                )
                .chain(
                    impact
                        .contextual_reachability
                        .paths
                        .iter()
                        .map(|path| path.module.clone()),
                )
        })
        .collect();
    let mut edges = edge_evidence(metrics, &graph);
    let exposures = inherited_exposure(metrics, &graph, &edges, request.git_used);
    let shared_reasons = shared_reasons(metrics, &graph, &context, &inventory);
    let hierarchy = structure::hierarchy(&graph, &inventory, metrics);
    let alternative_edges = edges
        .iter()
        .filter(|edge| !focused || focus.contains(&edge.source) || focus.contains(&edge.target));
    let mut alternatives = alternatives(alternative_edges, &context);
    let scenarios = scenarios(&edges, &context, &mut coverage);
    let owners = organization::ownership(metrics, &graph, &context, &root, &mut coverage);
    let coordination = organization::coordination(&graph, &owners);
    let lifecycle = organization::lifecycle(&graph, &context);
    let runtime = organization::runtime(&graph, &context);
    let mut priorities = priorities(&graph, &context, &current_report.issues, &exposures);
    if focused {
        priorities.retain(|priority| focus.contains(&priority.module));
    }
    let question_modules = if focused {
        focus.clone()
    } else {
        priorities
            .iter()
            .take(10)
            .map(|priority| priority.module.clone())
            .collect()
    };
    let mut questions =
        super::questions::review_questions(&question_modules, metrics, &context, &coordination);
    let order = ReviewOrder::new(&impact, &priorities);
    let mut constrained_pairs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for option in alternatives
        .iter()
        .filter(|option| !option.constraints.is_empty())
    {
        constrained_pairs
            .entry(option.source.clone())
            .or_default()
            .insert(option.target.clone());
    }
    alternatives.sort_by(|left, right| {
        let key = |option: &Alternative| {
            let (depth, priority) = order.pair(&option.source, &option.target);
            let constrained = constrained_pairs
                .get(&option.source)
                .is_some_and(|targets| targets.contains(&option.target));
            (depth, !constrained, priority)
        };
        key(left)
            .cmp(&key(right))
            .then_with(|| left.current_balance.total_cmp(&right.current_balance))
            .then_with(|| (&left.source, &left.target).cmp(&(&right.source, &right.target)))
    });
    edges.sort_by_key(|edge| {
        (
            edge.source_kind != EndpointKind::Module || edge.target_kind != EndpointKind::Module,
            order.pair(&edge.source, &edge.target),
            edge.observed_usage.as_deref() == Some("Import"),
        )
    });
    questions.sort_by_key(|question| {
        question
            .modules
            .iter()
            .map(|module| order.module(module))
            .min()
    });
    impact.sort_by_key(|impact| {
        (
            explicit_focus
                .as_ref()
                .is_some_and(|focus| focus != &impact.origin),
            order.module(&impact.origin),
        )
    });
    let decisions = decisions(
        &context,
        &graph,
        request.changed_since.map(|_| &changed_modules),
        &coordination,
        diff.as_ref(),
        &context_fingerprint,
    );
    let external_interfaces = structure::external_exposure(metrics, &graph, &inventory);
    Ok(DesignAssessment {
        schema_version: 1,
        review_scope: if request.focus.is_some() {
            "module"
        } else if request.changed_since.is_some() {
            "change"
        } else {
            "project"
        }
        .into(),
        focus: focus.into_iter().collect(),
        questions,
        provenance,
        coverage,
        changes,
        edges,
        impact,
        exposures,
        abstractions: inventory.findings,
        shared_reasons,
        hierarchy,
        alternatives,
        scenarios,
        priorities,
        external_interfaces,
        coordination,
        lifecycle,
        runtime,
        decisions,
        baseline,
    })
}

fn validate_bindings(
    context: &DesignContext,
    graph: &DependencyGraph,
    coverage: &mut Vec<String>,
) -> io::Result<()> {
    let mut assigned = BTreeSet::new();
    for component in &context.components {
        for name in selected(graph, &component.modules) {
            if !assigned.insert(name.clone()) {
                return Err(io::Error::other(format!(
                    "module {name} belongs to overlapping component declarations"
                )));
            }
        }
    }
    let selectors = context
        .components
        .iter()
        .flat_map(|v| &v.modules)
        .chain(context.rules.iter().flat_map(|v| &v.modules))
        .chain(context.decisions.iter().flat_map(|v| &v.modules))
        .chain(
            context
                .relationships
                .iter()
                .flat_map(|v| [&v.source, &v.target]),
        );
    for selector in selectors.collect::<BTreeSet<_>>() {
        if graph.select(selector).is_empty() {
            coverage.push(format!("Context selector '{selector}' matched no module in the selected scope; its facts were not evaluated."));
        }
    }
    Ok(())
}

fn edge_evidence(metrics: &ProjectMetrics, graph: &DependencyGraph) -> Vec<EdgeEvidence> {
    let modules =
        crate::metrics::target::ModuleIndex::new(&metrics.modules, &metrics.workspace_members);
    let mut edges: Vec<_> = metrics.couplings.iter().map(|edge| {
        let source = graph.resolve(&edge.source).unwrap_or_else(|| edge.source.clone());
        let external = edge.distance == crate::Distance::DifferentCrate;
        let target = if external { edge.target.clone() } else { graph.resolve(&edge.target).unwrap_or_else(|| edge.target.clone()) };
        let volatility = modules.provider(edge).and_then(|module| module.subdomain).map(|subdomain| subdomain.expected_volatility().value()).unwrap_or_else(|| edge.volatility.value());
        let strength = edge.strength_value();
        let distance = edge.distance_value();
        let reason = match edge.observed_usage.as_deref() {
            Some("TraitBound") => "Trait usage exposes an interface. Its signature and semantics still determine whether implementation knowledge leaks.",
            Some("MethodCall" | "FunctionCall") => "A call is observed. A shared business invariant cannot be inferred solely from the call syntax.",
            Some("FieldAccess" | "StructConstruction") => "Data access is observed; visibility distinguishes published model access from internal access.",
            Some("Import") => "An import establishes name visibility; actual sharing of behavior is not established by this observation.",
            _ => "Strength is inferred from Rust usage/visibility. Review the published information and shared reasons for change.",
        };
        let source_kind = if metrics.modules.contains_key(&source) { EndpointKind::Module } else { EndpointKind::Unresolved };
        let target_kind = if external {
            EndpointKind::External
        } else if metrics.modules.contains_key(&target) {
            EndpointKind::Module
        } else {
            EndpointKind::Unresolved
        };
        EdgeEvidence { source, target, source_kind, target_kind, observed_usage: edge.observed_usage.clone(), inferred_strength: format!("{:?}",edge.strength), origin: "inferred-from-syntax".into(), reason: reason.into(), file_path: edge.location.file_path.as_ref().map(|path| path.display().to_string()), line: edge.location.line, strength, distance, volatility, balance: balance(strength,distance,volatility), unknowns: vec!["Business invariants and runtime constraints require additional evidence.".into()] }
    }).collect();
    edges.sort_by(|a, b| {
        (&a.source, &a.target, a.line, &a.observed_usage).cmp(&(
            &b.source,
            &b.target,
            b.line,
            &b.observed_usage,
        ))
    });
    edges
}

fn inherited_exposure(
    metrics: &ProjectMetrics,
    graph: &DependencyGraph,
    edges: &[EdgeEvidence],
    git_used: bool,
) -> Vec<InheritedExposure> {
    let mut results = vec![];
    let mut strong_graph = DependencyGraph {
        nodes: graph.nodes.clone(),
        ..Default::default()
    };
    let mut upstream = BTreeMap::<&str, f64>::new();
    let mut strengths = BTreeMap::<(&str, &str), f64>::new();
    for edge in edges.iter().filter(|edge| {
        edge.source_kind == EndpointKind::Module && edge.target_kind == EndpointKind::Module
    }) {
        let volatility = upstream.entry(&edge.target).or_default();
        *volatility = volatility.max(edge.volatility);
        if edge.strength < 0.75 {
            continue;
        }
        let strength = strengths.entry((&edge.source, &edge.target)).or_default();
        *strength = strength.max(edge.strength);
        strong_graph
            .outgoing
            .entry(edge.source.clone())
            .or_default()
            .insert(edge.target.clone());
    }
    for module in &graph.nodes {
        for dependency in strong_graph.dependencies(module, None).paths {
            let upstream_volatility = upstream[dependency.module.as_str()];
            if upstream_volatility == 0.0 {
                continue;
            }
            let path_strength = dependency
                .path
                .windows(2)
                .map(|pair| strengths[&(pair[0].as_str(), pair[1].as_str())])
                .fold(1.0, f64::min);
            let data = metrics.modules.get(module);
            let observed_changes = git_used.then(|| {
                data.map(|module| {
                    metrics
                        .file_changes
                        .iter()
                        .filter(|(path, _)| module.path.ends_with(path))
                        .map(|(_, count)| *count)
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0)
            });
            results.push(InheritedExposure { module: module.clone(), essential_volatility: data.and_then(|module| module.subdomain).map(|subdomain| subdomain.expected_volatility().value()), observed_changes, upstream: dependency.module, upstream_volatility, path: dependency.path, path_strength, basis: "Strong dependency path into a volatile provider. Exposure is an inference; essential volatility is unchanged.".into() });
        }
    }
    results
}

fn shared_reasons(
    metrics: &ProjectMetrics,
    graph: &DependencyGraph,
    context: &DesignContext,
    inventory: &SourceInventory,
) -> Vec<SharedReason> {
    let mut reasons: Vec<_> = context
        .rules
        .iter()
        .map(|rule| {
            let modules = selected(graph, &rule.modules);
            SharedReason {
                id: rule.id.clone(),
                suggested_boundary: common_parent(&modules),
                modules,
                origin: "declared".into(),
                reason: rule.description.clone(),
                co_changes: None,
                ratio: None,
                verification: rule.verification.clone(),
            }
        })
        .collect();
    for finding in inventory
        .findings
        .iter()
        .filter(|finding| finding.kind == "identical-logic")
    {
        let modules: Vec<_> = finding
            .items
            .iter()
            .filter_map(|item| graph.resolve(&item.module))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if modules.len() < 2 {
            continue;
        }
        reasons.push(SharedReason {
            id: format!("similarity:{}", finding.items[0].body_fingerprint.as_deref().unwrap_or_default()),
            suggested_boundary: common_parent(&modules), modules,
            origin: "syntax-similarity".into(), reason: "Identical bodies suggest a question about shared requirements. Compare these modules with declared rules and co-change evidence before merging; independently changing requirements justify duplication.".into(),
            co_changes: None, ratio: None, verification: vec![],
        });
    }
    for pair in &metrics.temporal_couplings {
        if !pair.is_strong() {
            continue;
        }
        let modules: Vec<_> = metrics
            .modules
            .iter()
            .filter(|(_, module)| {
                module.path.ends_with(&pair.file_a) || module.path.ends_with(&pair.file_b)
            })
            .map(|(name, _)| name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if modules.len() < 2 {
            continue;
        }
        reasons.push(SharedReason { id: format!("co-change:{}:{}",pair.file_a,pair.file_b), suggested_boundary: common_parent(&modules), modules, origin: "git-co-change".into(), reason: "These files repeatedly change together. Confirm the common requirement; formatting, generated code and wiring changes can explain co-change.".into(), co_changes: Some(pair.co_change_count), ratio: Some(pair.coupling_ratio), verification: vec![] });
    }
    reasons
}

fn common_parent(modules: &[String]) -> Option<String> {
    let first = modules.first()?;
    let mut parts: Vec<_> = first.split("::").collect();
    parts.pop();
    while !parts.is_empty() {
        let prefix = format!("{}::", parts.join("::"));
        if modules.iter().all(|name| name.starts_with(&prefix)) {
            return Some(parts.join("::"));
        }
        parts.pop();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CouplingMetrics, Distance, IntegrationStrength, ModuleMetrics, Volatility};

    #[test]
    fn design_options_do_not_treat_external_unresolved_or_self_edges_as_boundaries() {
        let mut metrics = ProjectMetrics::new();
        for name in ["consumer", "provider", "other"] {
            metrics.add_module(ModuleMetrics::new(format!("{name}.rs").into(), name.into()));
        }
        for (target, distance) in [
            ("provider", Distance::DifferentModule),
            ("Err::Err", Distance::DifferentCrate),
            ("external::other", Distance::DifferentCrate),
            ("missing", Distance::DifferentModule),
            ("consumer", Distance::SameModule),
        ] {
            metrics.add_coupling(CouplingMetrics::new(
                "consumer".into(),
                target.into(),
                IntegrationStrength::Functional,
                distance,
                Volatility::High,
            ));
        }
        let graph = DependencyGraph::from_metrics(&metrics);
        let edges = edge_evidence(&metrics, &graph);
        let options = alternatives(&edges, &DesignContext::default());
        assert_eq!(options.len(), 4);
        assert!(options.iter().all(|option| option.target == "provider"));
        assert_eq!(
            edges.len(),
            5,
            "keep observations even when no action is justified"
        );
    }

    #[test]
    fn exposure_follows_longer_strong_path_when_shortest_path_is_weak() {
        let mut metrics = ProjectMetrics::new();
        for name in ["consumer", "adapter", "provider"] {
            metrics.add_module(ModuleMetrics::new(format!("{name}.rs").into(), name.into()));
        }
        for (source, target, strength) in [
            ("consumer", "provider", IntegrationStrength::Contract),
            ("consumer", "adapter", IntegrationStrength::Intrusive),
            ("adapter", "provider", IntegrationStrength::Intrusive),
        ] {
            metrics.add_coupling(CouplingMetrics::new(
                source.into(),
                target.into(),
                strength,
                Distance::DifferentModule,
                Volatility::High,
            ));
        }
        let graph = DependencyGraph::from_metrics(&metrics);
        let found = inherited_exposure(&metrics, &graph, &edge_evidence(&metrics, &graph), false);
        let exposure = found
            .iter()
            .find(|e| e.module == "consumer" && e.upstream == "provider")
            .expect("longer strong path must be retained");
        assert_eq!(exposure.path, ["consumer", "adapter", "provider"]);
        assert_eq!(exposure.essential_volatility, None);
    }

    #[test]
    fn external_homonym_cannot_supply_a_local_exposure_path() {
        let mut metrics = ProjectMetrics::new();
        for name in ["consumer", "provider"] {
            let mut module = ModuleMetrics::new(format!("{name}.rs").into(), name.into());
            module.subdomain = Some(crate::Subdomain::Core);
            metrics.add_module(module);
        }
        for (distance, strength) in [
            (Distance::DifferentModule, IntegrationStrength::Contract),
            (Distance::DifferentCrate, IntegrationStrength::Intrusive),
        ] {
            metrics.add_coupling(CouplingMetrics::new(
                "consumer".into(),
                "provider".into(),
                strength,
                distance,
                Volatility::Low,
            ));
        }
        let graph = DependencyGraph::from_metrics(&metrics);
        let edges = edge_evidence(&metrics, &graph);
        assert!(inherited_exposure(&metrics, &graph, &edges, true).is_empty());
        let external = edges
            .iter()
            .find(|edge| edge.target_kind == EndpointKind::External)
            .unwrap();
        assert_eq!(external.volatility, Volatility::Low.value());
    }

    #[test]
    fn missing_decision_evidence_is_unknown_instead_of_retain() {
        let context: DesignContext = toml::from_str("version=1\n[[decisions]]\nid='keep'\nmodules=['a']\nreason='Independent release'\nreview_on=['context-changed']\n").unwrap();
        let graph = DependencyGraph {
            nodes: BTreeSet::from(["a".into()]),
            ..Default::default()
        };
        assert_eq!(
            decisions(&context, &graph, None, &[], None, "current",)[0].status,
            "unknown"
        );
    }
}
