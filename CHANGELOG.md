# Changelog

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
