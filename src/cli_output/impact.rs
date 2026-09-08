//! Module impact assessment and its text presentation.

use crate::{Distance, ProjectMetrics, Volatility};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
};

/// Impact analysis result for a module
#[derive(Debug, Clone, Serialize)]
pub struct ImpactAnalysis {
    /// The module being analyzed
    pub module: String,
    /// Risk score (0-100)
    pub risk_score: u32,
    /// Risk level label
    pub risk_level: String,
    /// Direct dependencies (what this module depends on)
    pub dependencies: Vec<DependencyInfo>,
    /// Direct dependents (what depends on this module)
    pub dependents: Vec<DependencyInfo>,
    /// Cascading impact information
    pub cascading_impact: CascadingImpact,
    /// Whether module is in a circular dependency
    pub in_cycle: bool,
    /// Volatility information
    pub volatility: String,
}

/// Information about a dependency relationship (grouped by module)
#[derive(Debug, Clone, Serialize)]
pub struct DependencyInfo {
    /// Target/source module name
    pub module: String,
    /// Distance to the module
    pub distance: String,
    /// Coupling counts by strength type
    pub strengths: Vec<StrengthCount>,
    /// Total coupling count
    pub total_count: usize,
}

/// Count of couplings by strength type
#[derive(Debug, Clone, Serialize)]
pub struct StrengthCount {
    pub strength: String,
    pub count: usize,
}

/// Cascading impact analysis
#[derive(Debug, Clone, Serialize)]
pub struct CascadingImpact {
    /// Total modules affected (directly + indirectly)
    pub total_affected: usize,
    /// Percentage of codebase affected
    pub percentage: f64,
    /// Second-order dependencies (modules affected through dependents)
    pub second_order: Vec<String>,
    pub paths: Vec<crate::design::graph::ImpactPath>,
    pub max_depth: Option<usize>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ModuleLookup {
    Found(String),
    Ambiguous(Vec<String>),
    NotFound,
}

/// Analyze impact of changing a specific module
pub fn analyze_impact(metrics: &ProjectMetrics, module_name: &str) -> Option<ImpactAnalysis> {
    analyze_impact_with_depth(metrics, module_name, None)
}

pub fn analyze_impact_with_depth(
    metrics: &ProjectMetrics,
    module_name: &str,
    max_depth: Option<usize>,
) -> Option<ImpactAnalysis> {
    let ModuleLookup::Found(module) = find_module(metrics, module_name) else {
        return None;
    };
    let graph = crate::design::graph::DependencyGraph::from_metrics(metrics);
    let canonical = |name: &str| graph.resolve(name).unwrap_or_else(|| name.to_string());
    let module = canonical(&module);

    let circular_deps = metrics.detect_circular_dependencies();
    let cycle_modules: HashSet<String> = circular_deps
        .iter()
        .flatten()
        .map(|name| canonical(name))
        .collect();
    let in_cycle = cycle_modules.contains(&module);

    // Collect and group dependencies by target module
    let mut dep_map: HashMap<String, (String, HashMap<String, usize>)> = HashMap::new();
    let mut dependent_map: HashMap<String, (String, HashMap<String, usize>)> = HashMap::new();
    let mut volatility_max = Volatility::Low;

    for coupling in &metrics.couplings {
        if coupling.distance == Distance::DifferentCrate {
            continue; // Skip external crates
        }

        let source = canonical(&coupling.source);
        let target = canonical(&coupling.target);
        if source == module {
            let entry = dep_map
                .entry(target.clone())
                .or_insert_with(|| (format!("{:?}", coupling.distance), HashMap::new()));
            *entry
                .1
                .entry(format!("{:?}", coupling.strength))
                .or_insert(0) += 1;
        }

        if target == module {
            let entry = dependent_map
                .entry(source)
                .or_insert_with(|| (format!("{:?}", coupling.distance), HashMap::new()));
            *entry
                .1
                .entry(format!("{:?}", coupling.strength))
                .or_insert(0) += 1;

            // Track max volatility of incoming couplings
            if coupling.volatility > volatility_max {
                volatility_max = coupling.volatility;
            }
        }
    }

    let dependencies = grouped_dependencies(dep_map);
    let dependents = grouped_dependencies(dependent_map);

    let reach = graph.impact(&module, max_depth);
    let mut second_order: Vec<String> = reach
        .paths
        .iter()
        .filter(|p| p.path.len() == 3)
        .map(|p| p.module.clone())
        .collect();
    let total_affected = reach.paths.len();
    let total_internal_modules = graph.nodes.len();
    let percentage = if total_internal_modules > 0 {
        (total_affected as f64 / total_internal_modules as f64) * 100.0
    } else {
        0.0
    };

    // Calculate risk score
    let mut risk_score: u32 = 0;
    risk_score += (dependents.len() as u32) * 10; // Each dependent adds risk
    risk_score += (total_affected.saturating_sub(dependents.len()) as u32) * 5;
    if in_cycle {
        risk_score += 30;
    }
    match volatility_max {
        Volatility::High => risk_score += 20,
        Volatility::Medium => risk_score += 10,
        Volatility::Low => {}
    }
    risk_score = risk_score.min(100);

    let risk_level = if risk_score >= 70 {
        "HIGH"
    } else if risk_score >= 40 {
        "MEDIUM"
    } else {
        "LOW"
    }
    .to_string();

    let volatility = format!("{:?}", volatility_max);

    second_order.sort();

    Some(ImpactAnalysis {
        module: module.clone(),
        risk_score,
        risk_level,
        dependencies,
        dependents,
        cascading_impact: CascadingImpact {
            total_affected,
            percentage,
            second_order,
            paths: reach.paths,
            max_depth: reach.max_depth,
            truncated: reach.truncated,
        },
        in_cycle,
        volatility,
    })
}

fn grouped_dependencies(
    groups: HashMap<String, (String, HashMap<String, usize>)>,
) -> Vec<DependencyInfo> {
    let mut dependencies: Vec<_> = groups
        .into_iter()
        .map(|(module, (distance, counts))| {
            let total_count = counts.values().sum();
            let mut strengths: Vec<_> = counts
                .into_iter()
                .map(|(strength, count)| StrengthCount { strength, count })
                .collect();
            strengths.sort_by(|a, b| {
                b.count
                    .cmp(&a.count)
                    .then_with(|| a.strength.cmp(&b.strength))
            });
            DependencyInfo {
                module,
                distance,
                strengths,
                total_count,
            }
        })
        .collect();
    dependencies.sort_by(|a, b| a.module.cmp(&b.module));
    dependencies
}

pub(super) fn find_module(metrics: &ProjectMetrics, name: &str) -> ModuleLookup {
    // Exact names always win, whether they came from the module map or only
    // appear as a coupling endpoint.
    if metrics.modules.contains_key(name)
        || metrics
            .couplings
            .iter()
            .any(|coupling| coupling.source == name || coupling.target == name)
    {
        return ModuleLookup::Found(name.to_string());
    }

    // A short name may match a fully-qualified module at a segment boundary.
    let suffix = format!("::{name}");
    // Only the module registry is eligible for short-name expansion. Coupling
    // endpoints can contain item/type-qualified display names that happen to end
    // in the same segment but are not valid module choices.
    let mut matches: Vec<_> = metrics
        .modules
        .keys()
        .filter(|module_name| module_name.ends_with(&suffix))
        .cloned()
        .collect();
    matches.sort();
    matches.dedup();

    match matches.len() {
        0 => ModuleLookup::NotFound,
        1 => ModuleLookup::Found(matches.pop().expect("single module match")),
        _ => ModuleLookup::Ambiguous(matches),
    }
}

/// Format strength counts for display
fn format_strengths(strengths: &[StrengthCount]) -> String {
    if strengths.is_empty() {
        return "unknown".to_string();
    }
    if strengths.len() == 1 && strengths[0].count == 1 {
        return strengths[0].strength.clone();
    }
    strengths
        .iter()
        .map(|s| {
            if s.count == 1 {
                s.strength.clone()
            } else {
                format!("{}x {}", s.count, s.strength)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Generate impact analysis output
pub fn generate_impact_output<W: Write>(
    metrics: &ProjectMetrics,
    module_name: &str,
    writer: &mut W,
) -> io::Result<bool> {
    generate_impact_output_with_depth(metrics, module_name, None, writer)
}

pub fn generate_impact_output_with_depth<W: Write>(
    metrics: &ProjectMetrics,
    module_name: &str,
    max_depth: Option<usize>,
    writer: &mut W,
) -> io::Result<bool> {
    let resolved_module = match find_module(metrics, module_name) {
        ModuleLookup::Found(module) => module,
        ModuleLookup::Ambiguous(candidates) => {
            writeln!(writer, "❌ Module '{}' is ambiguous.", module_name)?;
            writeln!(writer)?;
            writeln!(writer, "Use one of these fully qualified names:")?;
            for candidate in candidates {
                writeln!(writer, "  - {}", candidate)?;
            }
            return Ok(false);
        }
        ModuleLookup::NotFound => {
            writeln!(writer, "❌ Module '{}' not found.", module_name)?;
            writeln!(writer)?;
            writeln!(writer, "Available modules:")?;
            let mut module_names: Vec<_> = metrics.modules.keys().collect();
            module_names.sort();
            for (i, name) in module_names.iter().take(10).enumerate() {
                writeln!(writer, "  {}. {}", i + 1, name)?;
            }
            if metrics.modules.len() > 10 {
                writeln!(writer, "  ... and {} more", metrics.modules.len() - 10)?;
            }
            return Ok(false);
        }
    };
    let analysis = analyze_impact_with_depth(metrics, &resolved_module, max_depth)
        .expect("an exact resolved module must remain resolvable");

    writeln!(writer, "Impact Analysis: {}", analysis.module)?;
    writeln!(
        writer,
        "═══════════════════════════════════════════════════════════"
    )?;

    // Risk score with visual indicator
    let risk_icon = match analysis.risk_level.as_str() {
        "HIGH" => "🔴",
        "MEDIUM" => "🟡",
        _ => "🟢",
    };
    writeln!(
        writer,
        "Risk Score: {} {} ({}/100)",
        risk_icon, analysis.risk_level, analysis.risk_score
    )?;

    if analysis.in_cycle {
        writeln!(writer, "⚠️  Part of a circular dependency cycle")?;
    }

    writeln!(writer)?;

    // Dependencies - count total couplings
    let total_dep_couplings: usize = analysis.dependencies.iter().map(|d| d.total_count).sum();
    writeln!(
        writer,
        "Direct Dependencies ({} modules, {} couplings):",
        analysis.dependencies.len(),
        total_dep_couplings
    )?;
    if analysis.dependencies.is_empty() {
        writeln!(writer, "  (none)")?;
    } else {
        for dep in &analysis.dependencies {
            let strengths_str = format_strengths(&dep.strengths);
            writeln!(
                writer,
                "  → {} ({}, {})",
                dep.module, strengths_str, dep.distance
            )?;
        }
    }

    writeln!(writer)?;

    // Dependents - count total couplings
    let total_dependent_couplings: usize = analysis.dependents.iter().map(|d| d.total_count).sum();
    writeln!(
        writer,
        "Direct Dependents ({} modules, {} couplings):",
        analysis.dependents.len(),
        total_dependent_couplings
    )?;
    if analysis.dependents.is_empty() {
        writeln!(writer, "  (none)")?;
    } else {
        for dep in &analysis.dependents {
            let strengths_str = format_strengths(&dep.strengths);
            writeln!(writer, "  ← {} ({})", dep.module, strengths_str)?;
        }
    }

    writeln!(writer)?;

    // Cascading impact
    writeln!(writer, "Cascading Impact:")?;
    writeln!(
        writer,
        "  Total affected: {} modules ({:.1}% of codebase)",
        analysis.cascading_impact.total_affected, analysis.cascading_impact.percentage
    )?;

    if !analysis.cascading_impact.second_order.is_empty() {
        writeln!(writer, "  2nd-order affected:")?;
        for module in analysis.cascading_impact.second_order.iter().take(5) {
            writeln!(writer, "    - {}", module)?;
        }
        if analysis.cascading_impact.second_order.len() > 5 {
            writeln!(
                writer,
                "    ... and {} more",
                analysis.cascading_impact.second_order.len() - 5
            )?;
        }
    }

    writeln!(
        writer,
        "  Traversal: {}{}",
        max_depth
            .map(|depth| format!("{depth} hops"))
            .unwrap_or_else(|| "all reachable modules".into()),
        if analysis.cascading_impact.truncated {
            " (more paths beyond limit)"
        } else {
            ""
        }
    )?;
    for impact in &analysis.cascading_impact.paths {
        writeln!(writer, "    {}", impact.path.join(" -> "))?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CouplingMetrics, IntegrationStrength, ModuleMetrics};

    #[test]
    fn aliases_have_the_same_direct_and_transitive_consumers() {
        let mut metrics = ProjectMetrics::new();
        for name in ["consumer", "provider"] {
            metrics.add_module(ModuleMetrics::new(format!("{name}.rs").into(), name.into()));
        }
        metrics.add_coupling(CouplingMetrics::new(
            "app::consumer".into(),
            "app::provider".into(),
            IntegrationStrength::Intrusive,
            Distance::DifferentModule,
            Volatility::High,
        ));
        let impact = analyze_impact(&metrics, "provider").unwrap();
        assert_eq!(impact.dependents.len(), 1);
        assert_eq!(impact.dependents[0].module, "consumer");
        assert_eq!(impact.cascading_impact.total_affected, 1);
        let qualified = analyze_impact(&metrics, "app::provider").unwrap();
        assert_eq!(
            serde_json::to_value(impact).unwrap(),
            serde_json::to_value(qualified).unwrap()
        );
    }
}
