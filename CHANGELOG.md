# Changelog

## v0.4.0

Change-oriented design analysis, a structure-first Web UI, and refactoring guided
by running cargo-coupling on its own source. See the
[design guide](docs/design-analysis.md) and
[verification record](docs/v0.4.0-verification.md).

### Added

- `--design`, `--context`, `--changed-since` and `--impact-depth`: inspect changed
  functions/types, transitive impact paths, test candidates, inherited exposure,
  abstraction findings, shared change reasons, and item-to-workspace boundaries.
- Validated, versioned TOML context for future changes, frozen code, business
  importance, effort estimates, CODEOWNERS overrides, shared build/test/release
  units, supplied runtime relationships, scenarios and retained decisions.
- Explainable priorities and alternative designs, including retaining a design.
  Observations, declarations, inferences and missing information remain distinct;
  scenario scores describe assumptions rather than predicting system quality.
- Source evidence and reproducibility metadata in JSON, including effective
  settings, scoring version, source fingerprints and baseline conditions.
- Web **Plan a change** uses the same Rust assessment as the CLI. It supports
  Git comparisons, impact tracing, source navigation, scenario review and
  decision review triggers, with English/Japanese navigation and JSON export.
- Web **Structure** is the initial view: directory cards, searchable modules and
  a directed dependency matrix provide an entry point before opening a graph.

### Fixed

- Issue #86: selecting a workspace member, manifest, source directory or file
  analyzes only that scope while retaining workspace metadata. Symlinks and
  `#[path]` modules are covered; the real similarity-rs member stays at six files.
- Balance now follows compensation: `max(abs(strength - distance), 1 - volatility)`.
  JSON and Web apply the same essential-volatility classification as the core.
  The scoring version is `khononov-compensation-v2`; old numeric scores should
  not be compared without reanalysis under the same settings.
- Baseline ratchets reject worsened existing findings as well as new findings.
- Canonical module aliases produce consistent direct/transitive impact,
  hotspot counts and JSON counts. The longest known module suffix takes
  precedence over a shorter suffix; ambiguous names are not chosen arbitrarily.
- Both graph projections display function, method and type names. Initial
  layouts separate nodes; projected labels avoid collisions and retain full
  details through selection. 3D initializes after its container is visible,
  restores mouse rotation/pan/zoom, and pauses rendering when hidden.
- Web comparisons reject stale source snapshots, including newly added files.
  Historical source navigation handles deleted directories while rejecting
  traversal and symlink escapes. Keyboard-accessible labels and source dialogs,
  mobile controls and independently scrolling planning navigation remain usable.

### Refactored and verified

- Split CLI impact, hotspots and JSON into focused modules while preserving
  public re-exports; separated planning from assessment and Web HTTP adapters
  from analysis. Reuse score calculation, alias resolution and label placement.
- Self-analysis with Git and `--exclude-tests` reports grade A, no High/Critical
  findings and 44 Medium findings. The focused CLI refactor reduced its outgoing
  dependencies from 36 to 28 and hotspot score from 176 to 132; its God Module
  finding was resolved. Remaining findings and similarity candidates are recorded
  in the verification document rather than hidden by threshold changes.
- Added regression coverage for scope, scoring, impact, Git/source evidence,
  input validation, Web endpoints and frontend layout/data behavior.

## v0.3.8

A reproducibility and dependency-security release. Analysis results are now stable across processes, circular-dependency reporting covers every multi-module cyclic internal edge without claiming to enumerate an exponential number of simple cycles, and CI verifies the exact reviewed dependency graph.

### Fixed

- Circular-dependency detection no longer depends on randomized `HashMap` / `HashSet` iteration. It now emits a deterministic set of representative cycle paths that covers every multi-module cyclic internal edge and module.
- JSON output is byte-stable for identical inputs: cycle paths, issues, hotspots, modules, temporal couplings, and tied grade dimensions all use explicit deterministic ordering.
- Markdown, impact, and trace output now use deterministic tie-breakers instead of inheriting randomized map iteration order.
- `--impact` no longer treats arbitrary string suffixes as module matches or silently chooses one crate when a short module name is ambiguous; it now requires a segment match and lists fully qualified candidates.
- Hotspot generation reuses the cycle analysis already computed for JSON output, and cycle return-path searches are restricted to strongly connected components instead of traversing acyclic regions once per node.
- CLI, Markdown, AI, and library documentation now describe the reported paths as representative cycle witnesses instead of presenting their count as an exhaustive count of all simple cycles.

### Security

- Updated `crossbeam-epoch` to 0.9.20, fixing `RUSTSEC-2026-0204` (invalid pointer dereference in pointer formatting). `cargo audit` is clean.
- Added a RustSec audit workflow for dependency changes and scheduled advisory checks.

### Changed

- Refreshed the remaining compatible dependency updates waiting behind the bot rate limit: Tokio 1.53.1, serde_json 1.0.151, and thiserror 2.0.19.
- CI and release workflows now use `--locked` for tests, Clippy, release builds, and publishing, so automation cannot silently test or publish a dependency graph different from `Cargo.lock`.
- Dependency updates are managed by Renovate only; the duplicate Dependabot configuration was removed.
- `--max-circular` now counts deterministic representative cycle paths. The recommended zero threshold remains equivalent to “no circular dependencies”; non-zero thresholds from earlier releases may need recalibration.
- Development-only agent, CI, Docker, and Renovate files are excluded from the published crate package.

## v0.3.7

A scoring-model correctness release. A full re-verification of the tool (CLI surfaces, web API/frontend, and the scoring pipeline, including adversarial review with empirical fixtures) found and fixed several places where the implementation contradicted the documented Balanced Coupling model.

### Fixed — scoring model

- Distance is now structural: computed from the module tree (ancestor/descendant at any depth and siblings under one non-root parent are Same; other same-crate pairs are Different) instead of from import syntax. Previously `use crate::far::x` counted as Same no matter how far, and `use super::sibling` counted as Different — inversions that punished the tool's own recommended god-module splits.
- Re-exported bare type names (`pub use module::Type;` + `use crate::Type;`) now resolve through the type registry to their defining module. They previously fell through to "external crate" and vanished from internal-coupling accounting entirely — hiding real coupling in any project using the common re-export idiom.
- Integration strength is Rust-faithful: field access / struct construction on a public type is use of the published data model (Model); crate-restricted or unresolved targets, and inherent impls on foreign types, remain Intrusive. Cross-module access to private fields cannot compile, so blanket Intrusive conflated "uses the published record" with "reaches into internals".
- Cascading Change Risk requires the documented Strong+Far+High quadrant; the implementation omitted the distance condition, flagging deliberately co-located volatile code (cohesion) as cascade risk.
- Report issues are deduped by their stable key; duplicates previously appeared for repeated dependency records of one pair (note: N usage sites between the same two modules now count as one issue).
- Accidental Volatility findings are diagnostics: still reported (and labeled "not graded"), but excluded from the health grade, per the declared design that raw git churn routes to diagnostics, not scoring. The grade rationale now narrates only gradable issues.
- Hidden Coupling skips structurally adjacent pairs (a package and its own submodules co-change by design) and requires a persistent pattern (≥8 co-changes at ≥80%) before claiming High severity — a burst of a few co-commits stays a Medium observation.
- Grade rationale for data-starved projects (<10 internal couplings) now says the B cap is a data limitation instead of claiming "low issue density".

### Fixed — web UI

- "Detect Clusters" / "Clear Colors" buttons were dead, and the cluster panel went stale after timeline revision swaps.
- The details-modal Source tab ignored the scrubbed revision (`?ref=`), so the modal and the sidebar viewer could show different code for the same module.
- Selecting a 3D link no longer drops its own selection highlight (cytoscape/API id mismatch).

### Changed

- Coupling classification (target resolution, structural distance, strength) extracted from `analyzer` into a dedicated `classification` module; scattered-external issue production moved into `balance::external_crates`.
- `.coupling.toml` subdomains refined to file granularity: the dimension enums (`src/metrics/dimensions.rs`) are the model's stable vocabulary (supporting), unlike the evolving model data in the rest of `metrics/`.
- Public API: `IssueKey` moved to the crate root re-export (`cargo_coupling::IssueKey` unchanged; the deep path `cargo_coupling::diff::IssueKey` no longer resolves).

### Honest self-assessment

Dogfooding this release: grade B (0 High/Critical). The earlier self-reported A grades were partly artifacts of the bugs above; the remaining Medium findings are dominated by this release's own cross-cutting development burst, which the tool now correctly observes and which decays as history normalizes. ripgrep keeps its unchanged, honest C.

## v0.3.5

### Added

- `.coupling.toml` drift detection: `[subdomains]`/`[volatility]` patterns that match no analyzed files are declared in the blind-spot manifest (EN/JA, CLI/report/web). When code is moved or split, stale patterns previously reverted scoring to raw git churn with no signal; now the rot is visible. `analysis.exclude` and `prelude_modules` are deliberately not reported — a dead exclude analyzes nothing wrongly, and defensive future-proof excludes are idiomatic.
- Precision guards so a drift note is never a false alarm: historical-revision analyses (`--history`, web `?ref=`) don't judge today's config against old trees; partial-scope runs don't judge patterns aimed at sibling trees; patterns whose only match fails to parse stay attributed to the parse-failure note.

### Fixed

- This repository's own `.coupling.toml` had exactly this rot: `[subdomains].core` still pointed at `src/balance.rs`/`src/metrics.rs`, dead since the split into directories, so core classification silently applied to nothing. Fixing it re-applies high essential volatility to the model modules and honestly lowers the self-reported grade — the previous grade was partly inflated by the drift this release now detects.

### Notes

- Issue #61 (crates without `src/`) was fixed in v0.3.4 and is verified and closed.

## v0.3.4

### Added

- Recognize crates without a `src/` folder (#61): source discovery now follows `cargo metadata` targets and the real module tree from each crate root, including `#[path]` module declarations. Layouts like ripgrep's root package (`[[bin]] path = "crates/core/main.rs"`) and `bin/main.rs` + `crates/*` module trees are analyzed instead of being silently skipped.
- Workspace members whose sources cannot be discovered, and module references that resolve outside the analyzed package/workspace boundary, are now declared in the blind-spot manifest instead of disappearing from the report.

### Fixed

- `#[path]` module resolution is confined to the analyzed workspace: absolute paths and `../` escapes are rejected instead of read, and files owned by another package are no longer double-analyzed under two members (which duplicated coupling edges and corrupted module metrics).
- Flat layouts (a target at the manifest root) no longer sweep `tests/`, `examples/`, `benches/`, or `build.rs` into the analyzed module set.
- The web `/api/source` endpoint rejects paths outside the analyzed workspace root.
- Hidden Coupling no longer flags co-change with the crate-root facade (`lib.rs`) — the facade declares/re-exports the crate's modules, so that co-change is expected by design (completes the entrypoint exemption from v0.3.3).

### Changed

- Module-tree discovery only runs for crates that actually use `#[path]`, removing a whole-crate double parse (matters for `--history` and web `?ref=`).
- Removed the unused `tower-http` dependency and refreshed all dependencies to the latest compatible versions.
- CI: bumped `actions/checkout` to v7 and `actions/cache` to v6.

## v0.3.3

### Added

- Added `--history[=N]` for chronological coupling health across sampled git revisions, using disposable worktrees and the `--git-months` window.
- Added JSON history output and a web timeline view with auto-play.
- Added a blind-spot manifest that declares analysis limitations. Text output shows run notes and a pointer by default; `--blind-spots` and `--all` expand it, while `--json` and `--ai` include the full manifest.
- Added `--baseline <ref>` issue diffs and `--check --baseline <ref>` ratchet gating for new issues only.
- Added DDD subdomain classification via `.coupling.toml` `[subdomains]` (`core`, `supporting`, `generic`) to distinguish essential from accidental volatility.
- Added Hidden Coupling detection for strong temporal co-change without explicit code dependency.
- Added Accidental Volatility detection for churny supporting or generic subdomains.
- Added `--deps` external-dependency analysis: per-crate integration breadth, `Cargo.lock` versions, and a Scattered External Coupling issue (a crate used across many modules without a facade).
- Added an explainable grade rationale (text and `--json` `grade_rationale`) describing what drives the grade and which dimension dominates.
- Added a cohesive web UI: 2D/3D/Dimension-Space views, a context-sensitive inspector (module and coupling details with inline source), clickable Critical/High/Medium counts that focus the involved dependencies, an always-reachable legend, and timeline scrubbing that morphs the graph through git revisions.
- Added a lightweight auto-reloading Markdown report view in the web UI.
- Added full English + Japanese coverage across all new CLI and web surfaces.

### Fixed

- Fixed trust-critical and data-loss CLI bugs (bare `--check` no longer silently passes; `--output` files are flushed; invalid `--min-grade`/`--fail-on` error explicitly).
- Fixed the subdomain volatility override, which silently applied to nothing (module-name vs file-path mismatch); subdomain classification now affects scoring as designed.
- Made history per-revision scoring match the snapshot methodology, so a revision's grade is consistent across views.
- Hardened baseline ratchet issue-key behavior (count changes no longer fake new/resolved issues) and tests.
- Guarded Accidental Volatility against false positives on tiny projects; made issue sorts NaN-safe.
- Addressed three-reviewer and mutation-testing findings and improved test coverage.

### Changed

- Tidied default CLI output for release readability.
- Refreshed proposal, config, release, and repository guidance docs.
- Added the repository `.coupling.toml` example for subdomain classification.
- Updated dependencies to the latest compatible versions.
