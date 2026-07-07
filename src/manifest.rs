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
    /// Japanese description of the same blind spot.
    pub description_ja: &'static str,
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
    /// Workspace members with no discoverable source files.
    pub skipped_crates: Vec<String>,
    /// Module references skipped after resolving outside the analyzed boundary.
    pub boundary_skipped_files: usize,
    /// Config patterns that matched no paths in this analysis run.
    pub dead_config_patterns: Vec<String>,
}

/// The declared negative space of an analysis run.
#[derive(Debug, Clone, Default)]
pub struct AnalysisManifest {
    /// Structural blind spots inherent to static AST analysis (always present).
    pub blind_spots: Vec<BlindSpot>,
    /// Run-specific notes describing how this run was degraded or narrowed.
    pub notes: Vec<String>,
    /// Japanese run-specific notes, aligned by index with `notes`.
    pub notes_ja: Vec<String>,
}

impl AnalysisManifest {
    /// Return run-specific notes in the requested language.
    pub fn localized_notes(&self, japanese: bool) -> &[String] {
        if japanese {
            &self.notes_ja
        } else {
            &self.notes
        }
    }
}

/// Structural limitations that hold for every run of a static, single-snapshot
/// AST analyzer, grounded in Khononov's coupling dimensions.
const STRUCTURAL_BLIND_SPOTS: &[BlindSpot] = &[
    BlindSpot {
        area: "dynamic-connascence",
        description: "Dynamic connascence (Execution, Timing, Values, Identity) is a runtime \
                      property; static AST analysis cannot detect it. Order/timing/shared-state \
                      coupling will not appear here.",
        description_ja: "動的コナーセンス (Execution, Timing, Values, Identity) は実行時の性質です。\
                         静的AST解析では検出できないため、順序・タイミング・共有状態による結合はここには現れません。",
    },
    BlindSpot {
        area: "implicit-functional-coupling",
        description: "Duplicated business logic and connascence of meaning/algorithm with no \
                      explicit call or import are only partially covered: strong temporal \
                      co-change can reveal hidden coupling between files, but duplicated logic \
                      that does not co-change remains invisible.",
        description_ja: "明示的な呼び出しやimportを伴わないビジネスロジックの重複、意味やアルゴリズムのコナーセンスは一部しか扱えません。\
                         強い共変更はファイル間の隠れた結合を示せますが、共変更しない重複ロジックは見えません。",
    },
    BlindSpot {
        area: "distance-axes",
        description: "Distance is measured by code structure only. Organizational distance \
                      (Conway's Law / team boundaries) and runtime distance (sync vs async) are \
                      not modeled, so the balance verdict reflects only structural distance.",
        description_ja: "距離はコード構造だけで測定します。組織上の距離 (コンウェイの法則やチーム境界) と実行時の距離 (同期/非同期) はモデル化しないため、\
                         バランス判定は構造上の距離だけを反映します。",
    },
    BlindSpot {
        area: "macro-and-cfg",
        description: "Coupling introduced by macro expansion, or behind inactive `cfg(...)`, is \
                      invisible to syn-based parsing. Generated code is not analyzed unless it \
                      exists as source.",
        description_ja: "マクロ展開で生じる結合や、無効な `cfg(...)` の背後にある結合は、synベースの解析では見えません。\
                         生成コードはソースとして存在しない限り解析されません。",
    },
];

/// Build the blind-spot manifest for a run with the given context.
pub fn build_manifest(ctx: &ManifestContext) -> AnalysisManifest {
    let mut notes = Vec::new();
    let mut notes_ja = Vec::new();

    if !ctx.git_used {
        notes.push(
            "Git history was not analyzed (--no-git or no repository): volatility and temporal \
             (co-change) coupling were skipped, so scores reflect structure only."
                .to_string(),
        );
        notes_ja.push(
            "Git履歴は解析されませんでした (--no-git またはリポジトリ外): 変更頻度と時間的な共変更結合はスキップされ、スコアは構造のみを反映します。"
                .to_string(),
        );
    }
    if ctx.tests_excluded {
        notes.push(
            "Test code was excluded: couplings that originate in tests are not counted."
                .to_string(),
        );
        notes_ja.push(
            "テストコードは除外されました: テスト由来の結合はカウントされません。".to_string(),
        );
    }
    if ctx.parse_failures > 0 {
        notes.push(format!(
            "{} source file(s) failed to parse and were skipped; coupling inside them is unknown.",
            ctx.parse_failures
        ));
        notes_ja.push(format!(
            "{} 件のソースファイルを解析できずスキップしました。その内部の結合は不明です。",
            ctx.parse_failures
        ));
    }
    if !ctx.skipped_crates.is_empty() {
        let crate_names = ctx.skipped_crates.join(", ");
        notes.push(format!(
            "Workspace member(s) {crate_names} had no discoverable source files (or all files were excluded by configuration) and were not analyzed."
        ));
        notes_ja.push(format!(
            "ワークスペースメンバー {crate_names} は発見可能なソースファイルがない（または設定により全ファイルが除外された）ため、解析されていません。"
        ));
    }
    if ctx.boundary_skipped_files > 0 {
        notes.push(format!(
            "{} module reference(s) resolved outside the analyzed package/workspace boundary and were not analyzed.",
            ctx.boundary_skipped_files
        ));
        notes_ja.push(format!(
            "{} 件のモジュール参照が解析対象パッケージ/ワークスペース境界の外を指しており、解析されていません。",
            ctx.boundary_skipped_files
        ));
    }
    if !ctx.dead_config_patterns.is_empty() {
        let pattern_list = ctx.dead_config_patterns.join(", ");
        notes.push(format!(
            ".coupling.toml drift: {} pattern(s) matched no analyzed files ({}); the classifications they were meant to apply are not in effect.",
            ctx.dead_config_patterns.len(),
            pattern_list
        ));
        notes_ja.push(format!(
            ".coupling.toml のドリフト: {} 件のパターンがどの解析対象ファイルにもマッチしません（{}）。意図した分類は適用されていません。",
            ctx.dead_config_patterns.len(),
            pattern_list
        ));
    }

    AnalysisManifest {
        blind_spots: STRUCTURAL_BLIND_SPOTS.to_vec(),
        notes,
        notes_ja,
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
            skipped_crates: Vec::new(),
            boundary_skipped_files: 0,
            dead_config_patterns: Vec::new(),
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
            skipped_crates: Vec::new(),
            boundary_skipped_files: 0,
            dead_config_patterns: Vec::new(),
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
            skipped_crates: Vec::new(),
            boundary_skipped_files: 0,
            dead_config_patterns: Vec::new(),
        });
        assert!(manifest.notes.iter().any(|n| n.contains("Test code")));
    }

    #[test]
    fn parse_failures_are_reported_with_count() {
        let manifest = build_manifest(&ManifestContext {
            git_used: true,
            tests_excluded: false,
            parse_failures: 3,
            skipped_crates: Vec::new(),
            boundary_skipped_files: 0,
            dead_config_patterns: Vec::new(),
        });
        assert!(manifest.notes.iter().any(|n| n.contains("3 source file")));
    }

    #[test]
    fn all_degradations_accumulate() {
        let manifest = build_manifest(&ManifestContext {
            git_used: false,
            tests_excluded: true,
            parse_failures: 2,
            skipped_crates: Vec::new(),
            boundary_skipped_files: 0,
            dead_config_patterns: Vec::new(),
        });
        assert_eq!(manifest.notes.len(), 3);
    }

    #[test]
    fn skipped_crates_are_reported_by_name() {
        let manifest = build_manifest(&ManifestContext {
            git_used: true,
            tests_excluded: false,
            parse_failures: 0,
            skipped_crates: vec!["empty-member".to_string()],
            boundary_skipped_files: 0,
            dead_config_patterns: Vec::new(),
        });
        assert!(manifest.notes.iter().any(|n| {
            n.contains(
                "Workspace member(s) empty-member had no discoverable source files (or all files were excluded by configuration) and were not analyzed.",
            )
        }));
        assert!(manifest.notes_ja.iter().any(|n| {
            n.contains("ワークスペースメンバー empty-member は発見可能なソースファイルがない（または設定により全ファイルが除外された）ため、解析されていません。")
        }));
    }

    #[test]
    fn boundary_skipped_files_are_reported_with_count() {
        let manifest = build_manifest(&ManifestContext {
            git_used: true,
            tests_excluded: false,
            parse_failures: 0,
            skipped_crates: Vec::new(),
            boundary_skipped_files: 2,
            dead_config_patterns: Vec::new(),
        });

        assert!(manifest.notes.iter().any(|n| {
            n.contains(
                "2 module reference(s) resolved outside the analyzed package/workspace boundary and were not analyzed.",
            )
        }));
        assert!(manifest.notes_ja.iter().any(|n| {
            n.contains("2 件のモジュール参照が解析対象パッケージ/ワークスペース境界の外を指しており、解析されていません。")
        }));
    }

    #[test]
    fn dead_config_patterns_are_reported_in_both_languages() {
        let manifest = build_manifest(&ManifestContext {
            git_used: true,
            tests_excluded: false,
            parse_failures: 0,
            skipped_crates: Vec::new(),
            boundary_skipped_files: 0,
            dead_config_patterns: vec![
                "subdomains.core: src/old.rs".to_string(),
                "volatility.high: src/dead.rs".to_string(),
            ],
        });

        assert!(manifest.notes.iter().any(|n| {
            n.contains(".coupling.toml drift: 2 pattern(s) matched no analyzed files (subdomains.core: src/old.rs, volatility.high: src/dead.rs); the classifications they were meant to apply are not in effect.")
        }));
        assert!(manifest.notes_ja.iter().any(|n| {
            n.contains(".coupling.toml のドリフト: 2 件のパターンがどの解析対象ファイルにもマッチしません（subdomains.core: src/old.rs, volatility.high: src/dead.rs）。意図した分類は適用されていません。")
        }));
    }

    #[test]
    fn empty_dead_config_patterns_add_no_note() {
        let manifest = build_manifest(&ManifestContext {
            git_used: true,
            tests_excluded: false,
            parse_failures: 0,
            skipped_crates: Vec::new(),
            boundary_skipped_files: 0,
            dead_config_patterns: Vec::new(),
        });

        assert!(
            !manifest
                .notes
                .iter()
                .any(|n| n.contains(".coupling.toml drift"))
        );
    }
}
