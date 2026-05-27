//! Analysis blind-spot manifest.
//!
//! Declares what the coupling analysis did **not** observe. As review shifts from
//! "read and understand the code" to "observe signals and trust them", trusting a
//! clean report depends on knowing its negative space: static analysis cannot see
//! dynamic connascence, implicit functional duplication, non-code distance, or
//! coupling hidden behind macros / inactive `cfg`. This module produces an
//! explicit, machine-readable manifest of those limitations for a given run, so a
//! "no issues" result is never mistaken for "no coupling problems exist".

/// A structural limitation of the analysis — something it cannot observe by design.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlindSpot {
    /// Short stable identifier (kebab-case), e.g. `dynamic-connascence`.
    pub area: &'static str,
    /// What is not observed, and why.
    pub description: &'static str,
}

/// Facts about a single analysis run that affect what was observed.
#[derive(Debug, Clone, Default)]
pub struct ManifestContext {
    /// Whether git history (volatility + temporal coupling) was analyzed.
    pub git_used: bool,
    /// Whether test code was excluded from analysis.
    pub tests_excluded: bool,
    /// Number of source files that failed to parse and were skipped.
    pub parse_failures: usize,
}

/// The declared negative space of an analysis run.
#[derive(Debug, Clone, Default)]
pub struct AnalysisManifest {
    /// Structural blind spots inherent to static AST analysis (always present).
    pub blind_spots: Vec<BlindSpot>,
    /// Run-specific notes describing how this run was degraded or narrowed.
    pub notes: Vec<String>,
}

/// Structural limitations that hold for every run of a static, single-snapshot
/// AST analyzer, grounded in Khononov's coupling dimensions.
const STRUCTURAL_BLIND_SPOTS: &[BlindSpot] = &[
    BlindSpot {
        area: "dynamic-connascence",
        description: "Dynamic connascence (Execution, Timing, Values, Identity) is a runtime \
                      property; static AST analysis cannot detect it. Order/timing/shared-state \
                      coupling will not appear here.",
    },
    BlindSpot {
        area: "implicit-functional-coupling",
        description: "Duplicated business logic and connascence of meaning/algorithm with no \
                      explicit call or import are not detected. Two modules can share knowledge \
                      without any code dependency between them.",
    },
    BlindSpot {
        area: "distance-axes",
        description: "Distance is measured by code structure only. Organizational distance \
                      (Conway's Law / team boundaries) and runtime distance (sync vs async) are \
                      not modeled, so the balance verdict reflects only structural distance.",
    },
    BlindSpot {
        area: "macro-and-cfg",
        description: "Coupling introduced by macro expansion, or behind inactive `cfg(...)`, is \
                      invisible to syn-based parsing. Generated code is not analyzed unless it \
                      exists as source.",
    },
];

/// Build the blind-spot manifest for a run with the given context.
pub fn build_manifest(ctx: &ManifestContext) -> AnalysisManifest {
    let mut notes = Vec::new();

    if !ctx.git_used {
        notes.push(
            "Git history was not analyzed (--no-git or no repository): volatility and temporal \
             (co-change) coupling were skipped, so scores reflect structure only."
                .to_string(),
        );
    }
    if ctx.tests_excluded {
        notes.push(
            "Test code was excluded: couplings that originate in tests are not counted."
                .to_string(),
        );
    }
    if ctx.parse_failures > 0 {
        notes.push(format!(
            "{} source file(s) failed to parse and were skipped; coupling inside them is unknown.",
            ctx.parse_failures
        ));
    }

    AnalysisManifest {
        blind_spots: STRUCTURAL_BLIND_SPOTS.to_vec(),
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_blind_spots_are_always_present() {
        let manifest = build_manifest(&ManifestContext {
            git_used: true,
            tests_excluded: false,
            parse_failures: 0,
        });
        assert_eq!(manifest.blind_spots.len(), STRUCTURAL_BLIND_SPOTS.len());
        assert!(
            manifest
                .blind_spots
                .iter()
                .any(|b| b.area == "dynamic-connascence")
        );
        assert!(
            manifest
                .blind_spots
                .iter()
                .any(|b| b.area == "implicit-functional-coupling")
        );
    }

    #[test]
    fn full_context_has_no_degradation_notes() {
        let manifest = build_manifest(&ManifestContext {
            git_used: true,
            tests_excluded: false,
            parse_failures: 0,
        });
        assert!(manifest.notes.is_empty());
    }

    #[test]
    fn no_git_adds_a_note() {
        let manifest = build_manifest(&ManifestContext {
            git_used: false,
            ..Default::default()
        });
        assert!(manifest.notes.iter().any(|n| n.contains("Git history")));
    }

    #[test]
    fn excluded_tests_adds_a_note() {
        let manifest = build_manifest(&ManifestContext {
            git_used: true,
            tests_excluded: true,
            parse_failures: 0,
        });
        assert!(manifest.notes.iter().any(|n| n.contains("Test code")));
    }

    #[test]
    fn parse_failures_are_reported_with_count() {
        let manifest = build_manifest(&ManifestContext {
            git_used: true,
            tests_excluded: false,
            parse_failures: 3,
        });
        assert!(manifest.notes.iter().any(|n| n.contains("3 source file")));
    }

    #[test]
    fn all_degradations_accumulate() {
        let manifest = build_manifest(&ManifestContext {
            git_used: false,
            tests_excluded: true,
            parse_failures: 2,
        });
        assert_eq!(manifest.notes.len(), 3);
    }
}
