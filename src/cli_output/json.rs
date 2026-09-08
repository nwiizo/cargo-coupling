//! Structured reports and their evidence metadata.

use super::hotspots::{Hotspot, calculate_hotspots_with_cycles};
use crate::balance::subdomain::{build_target_subdomain_map, coupling_with_essential_volatility};
use crate::balance::{project::analyze_project_balance_with_thresholds, score::BalanceScore};
use crate::diff::BaselineDiff;
use crate::external::{
    ExternalDependencyReport, ExternalDependencyUsage, analyze_external_dependencies,
};
use crate::manifest::AnalysisManifest;
use crate::{CouplingIssue, Distance, IssueThresholds, ProjectMetrics, Severity};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
};

/// Temporal coupling in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonTemporalCoupling {
    pub file_a: String,
    pub file_b: String,
    pub co_change_count: usize,
    pub coupling_ratio: f64,
    pub is_strong: bool,
}

/// Complete analysis in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonOutput {
    pub schema_version: u32,
    pub analyzer_version: String,
    pub scoring_version: String,
    pub summary: JsonSummary,
    pub grade_rationale: JsonGradeRationale,
    pub analysis_manifest: JsonAnalysisManifest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<JsonBaselineDiff>,
    pub external_dependencies: JsonExternalDependencies,
    pub hotspots: Vec<Hotspot>,
    pub issues: Vec<JsonIssue>,
    pub circular_dependencies: Vec<Vec<String>>,
    pub temporal_couplings: Vec<JsonTemporalCoupling>,
    pub modules: Vec<JsonModule>,
}

/// Standalone external-dependency JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct JsonExternalDependenciesOutput {
    pub external_dependencies: JsonExternalDependencies,
}

/// External dependency analysis in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonExternalDependencies {
    pub total_crates: usize,
    pub total_references: usize,
    pub dependencies: Vec<ExternalDependencyUsage>,
    pub scattered_couplings: Vec<JsonIssue>,
}

/// Summary in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonSummary {
    pub health_grade: String,
    pub health_score: f64,
    pub total_modules: usize,
    pub total_couplings: usize,
    pub internal_couplings: usize,
    pub external_couplings: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub medium_issues: usize,
}

/// Health-grade rationale in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonGradeRationale {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dominant_dimension: Option<String>,
    pub top_issue_types: Vec<JsonIssueTypeContribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Issue-type contribution in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonIssueTypeContribution {
    pub issue_type: String,
    pub count: usize,
    pub highest_severity: String,
}

/// Declared analysis blind spots in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonAnalysisManifest {
    pub blind_spots: Vec<JsonBlindSpot>,
    pub notes: Vec<String>,
}

/// Structural blind spot in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonBlindSpot {
    pub area: String,
    pub description: String,
}

/// Issue in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonIssue {
    pub id: String,
    pub issue_type: String,
    pub severity: String,
    pub source: String,
    pub target: String,
    pub description: String,
    pub suggestion: String,
    pub balance_score: f64,
    pub evidence: Vec<JsonIssueEvidence>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonIssueEvidence {
    pub file_path: Option<String>,
    pub line: usize,
    pub observed_usage: Option<String>,
    pub inferred_strength: String,
    pub source: String,
    pub target: String,
    pub limitation: String,
}

/// Module in JSON format
#[derive(Debug, Clone, Serialize)]
pub struct JsonModule {
    pub name: String,
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomain: Option<String>,
    pub couplings_out: usize,
    pub couplings_in: usize,
    pub balance_score: f64,
    pub in_cycle: bool,
}

/// Baseline diff in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonBaselineDiff {
    pub new_issues: Vec<JsonIssue>,
    pub resolved_issues: Vec<JsonIssue>,
    pub worsened_issues: Vec<JsonIssue>,
    pub unchanged: usize,
    pub score_delta: f64,
    pub grade_change: JsonGradeChange,
}

/// Baseline/current grade transition in JSON format.
#[derive(Debug, Clone, Serialize)]
pub struct JsonGradeChange {
    pub baseline: String,
    pub current: String,
}

/// Generate complete JSON output
pub fn generate_json_output<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    writer: &mut W,
) -> io::Result<()> {
    generate_json_output_with_optional_diff(metrics, thresholds, manifest, None, writer)
}

/// Generate complete JSON output with a top-level baseline diff object.
pub fn generate_json_output_with_diff<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    diff: &BaselineDiff,
    writer: &mut W,
) -> io::Result<()> {
    generate_json_output_with_optional_diff(metrics, thresholds, manifest, Some(diff), writer)
}

fn generate_json_output_with_optional_diff<W: Write>(
    metrics: &ProjectMetrics,
    thresholds: &IssueThresholds,
    manifest: &AnalysisManifest,
    diff: Option<&BaselineDiff>,
    writer: &mut W,
) -> io::Result<()> {
    let report = analyze_project_balance_with_thresholds(metrics, thresholds);
    let external_dependencies = analyze_external_dependencies(metrics, &HashMap::new());
    let circular_deps = metrics.detect_circular_dependencies();
    let evidence_graph = crate::design::graph::DependencyGraph::from_metrics(metrics);
    let canonical = |name: &str| {
        evidence_graph
            .resolve(name)
            .unwrap_or_else(|| name.to_string())
    };
    let cycle_modules: HashSet<String> = circular_deps
        .iter()
        .flatten()
        .map(|name| canonical(name))
        .collect();
    let target_subdomains = build_target_subdomain_map(metrics);
    let hotspots = calculate_hotspots_with_cycles(metrics, thresholds, 10, &circular_deps);

    // Count couplings per module
    let mut couplings_out: HashMap<String, usize> = HashMap::new();
    let mut couplings_in: HashMap<String, usize> = HashMap::new();
    let mut balance_scores: HashMap<String, Vec<f64>> = HashMap::new();
    let mut internal_count = 0;

    for coupling in &metrics.couplings {
        if coupling.distance != Distance::DifferentCrate {
            internal_count += 1;
            *couplings_out
                .entry(canonical(&coupling.source))
                .or_default() += 1;
            *couplings_in.entry(canonical(&coupling.target)).or_default() += 1;
            let score = BalanceScore::calculate(&coupling_with_essential_volatility(
                coupling,
                &target_subdomains,
            ));
            balance_scores
                .entry(canonical(&coupling.source))
                .or_default()
                .push(score.score);
        }
    }

    let external_count = metrics.couplings.len() - internal_count;

    let critical = *report
        .issues_by_severity
        .get(&Severity::Critical)
        .unwrap_or(&0);
    let high = *report.issues_by_severity.get(&Severity::High).unwrap_or(&0);
    let medium = *report
        .issues_by_severity
        .get(&Severity::Medium)
        .unwrap_or(&0);

    let mut temporal_couplings: Vec<JsonTemporalCoupling> = metrics
        .temporal_couplings
        .iter()
        .map(|tc| JsonTemporalCoupling {
            file_a: tc.file_a.clone(),
            file_b: tc.file_b.clone(),
            co_change_count: tc.co_change_count,
            coupling_ratio: tc.coupling_ratio,
            is_strong: tc.is_strong(),
        })
        .collect();
    temporal_couplings.sort_by(|a, b| {
        b.co_change_count
            .cmp(&a.co_change_count)
            .then_with(|| {
                b.coupling_ratio
                    .partial_cmp(&a.coupling_ratio)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.file_a.cmp(&b.file_a))
            .then_with(|| a.file_b.cmp(&b.file_b))
    });
    temporal_couplings.truncate(20);

    let mut modules: Vec<JsonModule> = metrics
        .modules
        .iter()
        .map(|(name, module)| {
            let avg_score = balance_scores
                .get(name)
                .map(|scores| scores.iter().sum::<f64>() / scores.len() as f64)
                .unwrap_or(1.0);
            JsonModule {
                name: name.clone(),
                file_path: Some(module.path.display().to_string()),
                subdomain: module.subdomain.map(|subdomain| subdomain.to_string()),
                couplings_out: couplings_out.get(name).copied().unwrap_or(0),
                couplings_in: couplings_in.get(name).copied().unwrap_or(0),
                balance_score: avg_score,
                in_cycle: cycle_modules.contains(name),
            }
        })
        .collect();
    modules.sort_by(|a, b| a.name.cmp(&b.name));

    let output = JsonOutput {
        schema_version: 2,
        analyzer_version: env!("CARGO_PKG_VERSION").into(),
        scoring_version: crate::balance::score::SCORING_VERSION.into(),
        summary: JsonSummary {
            health_grade: report.health_grade.letter().to_string(),
            health_score: report.average_score,
            total_modules: metrics.modules.len(),
            total_couplings: metrics.couplings.len(),
            internal_couplings: internal_count,
            external_couplings: external_count,
            critical_issues: critical,
            high_issues: high,
            medium_issues: medium,
        },
        grade_rationale: JsonGradeRationale {
            summary: report.grade_rationale.summary.clone(),
            dominant_dimension: report
                .grade_rationale
                .dominant_dimension
                .map(|dimension| dimension.to_string()),
            top_issue_types: report
                .grade_rationale
                .top_issue_types
                .iter()
                .map(|item| JsonIssueTypeContribution {
                    issue_type: item.issue_type.to_string(),
                    count: item.count,
                    highest_severity: item.highest_severity.to_string(),
                })
                .collect(),
            note: report.grade_rationale.volatility_note.clone(),
        },
        analysis_manifest: JsonAnalysisManifest {
            blind_spots: manifest
                .blind_spots
                .iter()
                .map(|blind_spot| JsonBlindSpot {
                    area: blind_spot.area.to_string(),
                    description: if thresholds.japanese {
                        blind_spot.description_ja.to_string()
                    } else {
                        blind_spot.description.to_string()
                    },
                })
                .collect(),
            notes: manifest.localized_notes(thresholds.japanese).to_vec(),
        },
        diff: diff.map(json_baseline_diff),
        external_dependencies: json_external_dependencies(&external_dependencies),
        hotspots,
        issues: report
            .issues
            .iter()
            .map(|issue| json_issue_with_evidence(issue, metrics, &evidence_graph))
            .collect(),
        circular_dependencies: circular_deps,
        temporal_couplings,
        modules,
    };

    let json = serde_json::to_string_pretty(&output).map_err(io::Error::other)?;
    writeln!(writer, "{}", json)?;

    Ok(())
}

fn json_baseline_diff(diff: &BaselineDiff) -> JsonBaselineDiff {
    JsonBaselineDiff {
        new_issues: diff.new_issues.iter().map(json_issue).collect(),
        resolved_issues: diff.resolved_issues.iter().map(json_issue).collect(),
        worsened_issues: diff.worsened_issues.iter().map(json_issue).collect(),
        unchanged: diff.unchanged,
        score_delta: diff.score_delta,
        grade_change: JsonGradeChange {
            baseline: diff.baseline_grade.letter().to_string(),
            current: diff.current_grade.letter().to_string(),
        },
    }
}

fn json_issue(issue: &CouplingIssue) -> JsonIssue {
    let key = issue.stable_key();
    JsonIssue {
        id: crate::design::source::fingerprint(
            format!("{:?}\0{}\0{}", key.issue_type, key.source, key.target).as_bytes(),
        ),
        issue_type: format!("{}", issue.issue_type),
        severity: format!("{}", issue.severity),
        source: issue.source.clone(),
        target: issue.target.clone(),
        description: issue.description.clone(),
        suggestion: format!("{}", issue.refactoring),
        balance_score: issue.balance_score,
        evidence: Vec::new(),
    }
}

fn json_issue_with_evidence(
    issue: &CouplingIssue,
    metrics: &ProjectMetrics,
    graph: &crate::design::graph::DependencyGraph,
) -> JsonIssue {
    let mut json = json_issue(issue);
    let source = graph.resolve(&issue.source);
    let target = graph.resolve(&issue.target);
    json.evidence = metrics.couplings.iter().filter(|edge| {
        let source_matches = source.as_ref().is_some_and(|source| graph.resolve(&edge.source).as_ref() == Some(source));
        let target_matches = target.as_ref().is_some_and(|target| graph.resolve(&edge.target).as_ref() == Some(target));
        match (source.is_some(),target.is_some()) {
            (true,true) => source_matches && target_matches,
            (true,false) => source_matches,
            (false,true) => target_matches,
            _ => false,
        }
    }).map(|edge| JsonIssueEvidence {
        file_path: edge.location.file_path.as_ref().map(|path| path.display().to_string()), line: edge.location.line,
        observed_usage: edge.observed_usage.clone(), inferred_strength: format!("{:?}",edge.strength), source: edge.source.clone(), target: edge.target.clone(),
        limitation: "Syntax and visibility are observed; shared business knowledge is inferred. Use --design for context, history and unobserved constraints.".into(),
    }).collect();
    json.evidence.sort_by(|a, b| {
        (
            &a.file_path,
            a.line,
            &a.source,
            &a.target,
            &a.observed_usage,
        )
            .cmp(&(
                &b.file_path,
                b.line,
                &b.source,
                &b.target,
                &b.observed_usage,
            ))
    });
    json
}

pub(super) fn json_external_dependencies(
    report: &ExternalDependencyReport,
) -> JsonExternalDependencies {
    JsonExternalDependencies {
        total_crates: report.dependencies.len(),
        total_references: report
            .dependencies
            .iter()
            .map(|dependency| dependency.total_references)
            .sum(),
        dependencies: report.dependencies.clone(),
        scattered_couplings: report.scattered_couplings.iter().map(json_issue).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CouplingMetrics, IntegrationStrength, ModuleMetrics, Subdomain, Volatility};

    #[test]
    fn module_counts_resolve_aliases_and_scores_use_essential_volatility() {
        let mut metrics = ProjectMetrics::new();
        metrics.add_module(ModuleMetrics::new("consumer.rs".into(), "consumer".into()));
        let mut provider = ModuleMetrics::new("provider.rs".into(), "provider".into());
        provider.subdomain = Some(Subdomain::Generic);
        metrics.add_module(provider);
        metrics.add_coupling(CouplingMetrics::new(
            "app::consumer".into(),
            "app::provider".into(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        ));
        let mut buffer = Vec::new();
        generate_json_output(
            &metrics,
            &IssueThresholds::default(),
            &AnalysisManifest::default(),
            &mut buffer,
        )
        .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&buffer).unwrap();
        let consumer = json["modules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["name"] == "consumer")
            .unwrap();
        assert_eq!(consumer["couplings_out"], 1);
        assert_eq!(
            consumer["balance_score"],
            1.0 - Subdomain::Generic.expected_volatility().value()
        );
    }
}
