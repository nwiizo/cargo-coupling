//! Serializable design assessment, shared by text and Web consumers.

use super::{
    changes::{ChangeSet, Provenance},
    graph::{ImpactPath, Reachability},
    source::{SourceFinding, SourceItem},
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DesignAssessment {
    pub schema_version: u32,
    pub provenance: Provenance,
    pub coverage: Vec<String>,
    pub changes: ChangeSet,
    pub edges: Vec<EdgeEvidence>,
    pub impact: Vec<ChangeImpact>,
    pub exposures: Vec<InheritedExposure>,
    pub abstractions: Vec<SourceFinding>,
    pub shared_reasons: Vec<SharedReason>,
    pub hierarchy: Vec<BoundarySummary>,
    pub alternatives: Vec<Alternative>,
    pub scenarios: Vec<ScenarioResult>,
    pub priorities: Vec<Priority>,
    pub external_interfaces: Vec<ExternalExposure>,
    pub coordination: Vec<Coordination>,
    pub lifecycle: Vec<LifecycleGroup>,
    pub runtime: Vec<RuntimeRelation>,
    pub decisions: Vec<DecisionReview>,
    pub baseline: Option<BaselineSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EdgeEvidence {
    pub source: String,
    pub target: String,
    pub observed_usage: Option<String>,
    pub inferred_strength: String,
    pub origin: String,
    pub reason: String,
    pub file_path: Option<String>,
    pub line: usize,
    pub strength: f64,
    pub distance: f64,
    pub volatility: f64,
    pub balance: f64,
    pub unknowns: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeImpact {
    pub origin: String,
    pub changed_items: Vec<String>,
    pub reachability: Reachability,
    pub tests: Vec<TestCandidate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestCandidate {
    pub item: SourceItem,
    pub reason: String,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InheritedExposure {
    pub module: String,
    pub essential_volatility: Option<f64>,
    pub observed_changes: Option<usize>,
    pub upstream: String,
    pub upstream_volatility: f64,
    pub path: Vec<String>,
    /// Weakest integration along the reported path; not a failure probability.
    pub path_strength: f64,
    pub basis: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SharedReason {
    pub id: String,
    pub modules: Vec<String>,
    pub origin: String,
    pub reason: String,
    pub co_changes: Option<usize>,
    pub ratio: Option<f64>,
    pub suggested_boundary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoundarySummary {
    pub level: String,
    pub name: String,
    pub members: Vec<String>,
    pub internal_edges: usize,
    pub incoming_edges: usize,
    pub outgoing_edges: usize,
    pub internal_groups: Vec<Vec<String>>,
    pub observation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Alternative {
    pub source: String,
    pub target: String,
    pub action: String,
    pub current_balance: f64,
    pub hypothetical_balance: f64,
    pub assumptions: Vec<String>,
    pub tradeoffs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioResult {
    pub name: String,
    pub description: String,
    pub affected_edges: usize,
    pub current_balance: Option<f64>,
    pub hypothetical_balance: Option<f64>,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Priority {
    pub module: String,
    pub reasons: Vec<String>,
    pub affected_modules: usize,
    pub issue_count: usize,
    pub business_value: f64,
    pub effort_days: Option<f64>,
    pub planned_changes: Option<String>,
    pub frozen_reason: Option<String>,
    /// Relative ordering heuristic, never interpreted as currency or savings.
    pub priority_score: f64,
    pub value_per_effort: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalExposure {
    pub crate_name: String,
    pub public_items: Vec<SourceItem>,
    pub direct_modules: Vec<String>,
    pub affected: Vec<ImpactPath>,
    pub replacement_boundaries: Vec<String>,
    pub unknowns: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Coordination {
    pub source: String,
    pub target: String,
    pub source_owners: Vec<String>,
    pub target_owners: Vec<String>,
    pub cross_team: bool,
    pub basis: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LifecycleGroup {
    pub kind: String,
    pub unit: String,
    pub modules: Vec<String>,
    pub pairs_without_code_edges: usize,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeRelation {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: super::context::RelationshipKind,
    pub origin: super::context::EvidenceOrigin,
    pub evidence: String,
    pub review: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionReview {
    pub id: String,
    pub modules: Vec<String>,
    pub reason: String,
    pub status: String,
    pub triggered: Vec<String>,
    pub pending_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaselineSummary {
    pub reference: String,
    pub reference_commit: Option<String>,
    pub settings_fingerprint: Option<String>,
    pub new_issues: usize,
    pub worsened_issues: usize,
    pub resolved_issues: usize,
    pub score_delta: f64,
    pub notes: Vec<String>,
    pub findings: Vec<ComparedFinding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparedFinding {
    pub change: String,
    pub source: String,
    pub target: String,
    pub issue_type: String,
    pub severity: String,
    pub description: String,
    pub balance_score: f64,
}
