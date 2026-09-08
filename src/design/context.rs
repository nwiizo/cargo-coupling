//! Optional human-supplied design facts. Nothing here rewrites observed metrics.

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DesignContext {
    pub version: u32,
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(default)]
    pub rules: Vec<BusinessRule>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
    #[serde(default)]
    pub scenarios: Vec<Scenario>,
    #[serde(default)]
    pub decisions: Vec<Decision>,
}

impl Default for DesignContext {
    fn default() -> Self {
        Self {
            version: 1,
            components: vec![],
            rules: vec![],
            relationships: vec![],
            scenarios: vec![],
            decisions: vec![],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub name: String,
    pub modules: Vec<String>,
    #[serde(default)]
    pub owners: Vec<String>,
    pub build_unit: Option<String>,
    pub test_unit: Option<String>,
    pub release_unit: Option<String>,
    pub planned_changes: Option<String>,
    pub frozen_reason: Option<String>,
    #[serde(default = "one")]
    pub business_value: f64,
    pub effort_days: Option<f64>,
}

fn one() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BusinessRule {
    pub id: String,
    pub modules: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RelationshipKind {
    Sequence,
    Timing,
    Transaction,
    SharedState,
    Synchronous,
    Asynchronous,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceOrigin {
    #[default]
    Declared,
    Observed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: RelationshipKind,
    pub evidence: String,
    #[serde(default)]
    pub origin: EvidenceOrigin,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub changes: Vec<ScenarioChange>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioChange {
    pub source: String,
    pub target: String,
    pub strength: Option<f64>,
    pub distance: Option<f64>,
    pub volatility: Option<f64>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewTrigger {
    PlannedChange,
    CrossTeam,
    NewIssue,
    WorsenedIssue,
    Changed,
    ContextChanged,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub id: String,
    pub modules: Vec<String>,
    pub reason: String,
    pub review_on: Vec<ReviewTrigger>,
    /// Optional previously recorded context fingerprint.
    pub context_fingerprint: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("Cannot read design context: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid design context: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Invalid design context: {0}")]
    Invalid(String),
}

impl DesignContext {
    pub fn parse(source: &str) -> Result<Self, ContextError> {
        let context: Self = toml::from_str(source)?;
        context.validate()?;
        Ok(context)
    }

    pub fn validate(&self) -> Result<(), ContextError> {
        if self.version != 1 {
            return Err(ContextError::Invalid(format!(
                "unsupported version {}",
                self.version
            )));
        }
        unique(self.components.iter().map(|v| v.name.as_str()), "component")?;
        unique(self.rules.iter().map(|v| v.id.as_str()), "rule")?;
        unique(
            self.relationships.iter().map(|v| v.id.as_str()),
            "relationship",
        )?;
        unique(self.scenarios.iter().map(|v| v.name.as_str()), "scenario")?;
        unique(self.decisions.iter().map(|v| v.id.as_str()), "decision")?;
        for component in &self.components {
            selectors(&component.modules)?;
            if !component.business_value.is_finite()
                || component.business_value <= 0.0
                || component
                    .effort_days
                    .is_some_and(|v| !v.is_finite() || v <= 0.0)
            {
                return Err(ContextError::Invalid(format!(
                    "{}: business_value and effort_days must be finite and positive",
                    component.name
                )));
            }
            for text in component
                .owners
                .iter()
                .map(String::as_str)
                .chain(component.planned_changes.as_deref())
                .chain(component.frozen_reason.as_deref())
                .chain(component.build_unit.as_deref())
                .chain(component.test_unit.as_deref())
                .chain(component.release_unit.as_deref())
            {
                nonempty(text)?;
            }
        }
        for rule in &self.rules {
            selectors(&rule.modules)?;
            nonempty(&rule.description)?;
        }
        for relation in &self.relationships {
            selector(&relation.source)?;
            selector(&relation.target)?;
            nonempty(&relation.evidence)?;
        }
        for scenario in &self.scenarios {
            nonempty(&scenario.description)?;
            if scenario.changes.is_empty() {
                return Err(ContextError::Invalid(format!(
                    "{} has no changes",
                    scenario.name
                )));
            }
            for change in &scenario.changes {
                selector(&change.source)?;
                selector(&change.target)?;
                let values = [change.strength, change.distance, change.volatility];
                if values.iter().all(Option::is_none)
                    || values
                        .into_iter()
                        .flatten()
                        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
                {
                    return Err(ContextError::Invalid(format!(
                        "{}: specify at least one dimension in 0..=1",
                        scenario.name
                    )));
                }
            }
        }
        for decision in &self.decisions {
            selectors(&decision.modules)?;
            nonempty(&decision.reason)?;
            if decision.review_on.is_empty() {
                return Err(ContextError::Invalid(format!(
                    "{} needs review triggers",
                    decision.id
                )));
            }
        }
        Ok(())
    }
}

fn nonempty(value: &str) -> Result<(), ContextError> {
    if value.trim().is_empty() {
        Err(ContextError::Invalid("empty value".into()))
    } else {
        Ok(())
    }
}
fn unique<'a>(values: impl Iterator<Item = &'a str>, kind: &str) -> Result<(), ContextError> {
    let mut seen = BTreeSet::new();
    for value in values {
        nonempty(value)?;
        if !seen.insert(value) {
            return Err(ContextError::Invalid(format!("duplicate {kind}: {value}")));
        }
    }
    Ok(())
}
fn selector(value: &str) -> Result<(), ContextError> {
    nonempty(value)?;
    glob::Pattern::new(value).map_err(|e| ContextError::Invalid(e.to_string()))?;
    Ok(())
}
fn selectors(values: &[String]) -> Result<(), ContextError> {
    if values.is_empty() {
        return Err(ContextError::Invalid("modules cannot be empty".into()));
    }
    for value in values {
        selector(value)?;
    }
    Ok(())
}

/// Search upward from the selected source, respecting an explicitly supplied path.
pub fn load_context(
    path: &Path,
    explicit: Option<&Path>,
) -> Result<(DesignContext, Option<PathBuf>), ContextError> {
    let selected = if let Some(explicit) = explicit {
        Some(explicit.to_path_buf())
    } else {
        let absolute = fs::canonicalize(path)?;
        let start = if absolute.is_file() {
            absolute.parent().unwrap_or(&absolute)
        } else {
            &absolute
        };
        start
            .ancestors()
            .map(|dir| dir.join(".coupling-context.toml"))
            .find(|p| p.is_file())
    };
    match selected {
        Some(path) => Ok((
            DesignContext::parse(&fs::read_to_string(&path)?)?,
            Some(fs::canonicalize(path)?),
        )),
        None => Ok((DesignContext::default(), None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unknown_and_invalid_inputs() {
        for text in [
            "version=2",
            "version=1\ncomponents_typo=[]",
            "version=1\n[[components]]\nname='x'\nmodules=['[']",
            "version=1\n[[components]]\nname='x'\nmodules=['*']\neffort_days=0",
        ] {
            assert!(DesignContext::parse(text).is_err(), "{text}");
        }
    }
}
