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
- Added web 3D coupling view, Dimension-Space support, trust panels, and clearer module names.

### Fixed

- Fixed trust-critical and data-loss CLI bugs around analysis output.
- Hardened baseline ratchet issue-key behavior and tests.
- Addressed mutation-testing findings and improved test coverage.

### Changed

- Tidied default CLI output for release readability.
- Refreshed proposal, config, release, and repository guidance docs.
- Added the repository `.coupling.toml` example for subdomain classification.
- Updated dependencies to the latest compatible versions.
