# Change-oriented design analysis: implementation TODO

Implement the jobs identified from *Balancing Coupling in Software Design* in the
existing Rust CLI and reporting pipeline. Preserve existing analysis-scope work.
Inputs that static Rust analysis cannot establish must be supplied explicitly;
reports distinguish observations, declarations, inferences, and unknowns.

## Work items

- [x] 01. Align balance calculations with compensation; passing regression examples cover stability and proximity. CLI, JSON and Web output use the same calculation.
- [x] 02. Distinguish observed Rust usage from inferred integration strength; explain classification and uncertainty.
- [x] 03. Detect regressions in existing baseline findings, including severity and affected breadth; escalation and count-change regressions pass.
- [x] 04. Trace change impact beyond two hops, retain paths, and expose the traversal limit.
- [x] 05. Seed impact analysis from Git changes, identify changed items, and list evidenced test candidates.
- [x] 06. Explain inherited change exposure separately from essential and observed volatility.
- [x] 07. Inspect abstraction effectiveness: leaked implementation types, mirrored records, and forwarding layers.
- [x] 08. Find shared change reasons using co-change, structural similarity, and declared business rules.
- [x] 09. Report cohesion and cross-boundary knowledge at item, module, package, and workspace levels.
- [x] 10. Compare design alternatives and declared scenarios, including tradeoffs and retaining the current design.
- [x] 11. Accept future change plans and frozen-code context without rewriting observed history.
- [x] 12. Rank work using upcoming changes, impact, business importance, and explicit effort estimates.
- [x] 13. Extend external dependency analysis with public API exposure paths and replacement boundaries.
- [x] 14. Resolve ownership and cross-team coordination from CODEOWNERS and explicit declarations.
- [x] 15. Model shared build, test, and release units, including components without code edges.
- [x] 16. Model declared/observed order, timing, transaction, and shared-state constraints; declare runtime coverage.
- [x] 17. Attach source locations, classification reasons, supporting history, and unknowns to human/JSON reports.
- [x] 18. Record reproducible analysis conditions and distinguish structural changes from changed methodology/configuration/scope.
- [x] 19. Persist accepted design decisions with reasons and review triggers; re-evaluate stale decisions.
- [x] 20. Integrate all capabilities into CLI/JSON/report output, document input examples and limitations, and verify compatibility.
- [x] 21. Run focused regression tests, fmt, clippy, all-feature tests, release build, structural analysis, and repository smoke tests.
- [x] 22. Improve Web visualization with change origins, selectable impact paths, evidence/source navigation, and explicit traversal/coverage limits.
- [x] 23. Add Web views for inherited exposure, abstraction findings, hierarchy, ownership, and lifecycle/runtime relationships.
- [x] 24. Compare scenarios and baseline regressions in the Web UI; display assumptions and accepted-decision review triggers.
- [x] 25. Verify Web keyboard access, empty/error/loading states, Japanese/English labels, and browser rendering with screenshots.
- [ ] 26. After every implementation and verification task passes, update version and release notes for v0.4.0, commit, tag, publish, and verify release automation and registry publication.
- [x] 27. Resolve GitHub issue #86: retain the existing scope-selection changes and eight `scope_e2e` tests; rerun workspace/package/source/file/Cargo.toml/symlink/`#[path]`/config-scope regressions with the final release and verify the real similarity checkout stays at six selected files. Include in v0.4.0 release notes.
- [x] 28. Rethink the initial Web presentation: start with readable source boundaries and progressive detail, provide a directed dependency matrix, avoid overlapping nodes/labels/controls, and verify real desktop/mobile layouts and keyboard navigation.
- [x] 29. Show function, method and type names in both 2D and 3D; verify mouse rotation, zoom, pan and fit after switching views.
- [x] 30. Use cargo coupling and similarity-rs on this repository to guide refactoring; confirm each candidate in source and compare the final results under the same settings.

## Implementation approach

Use the existing `syn`, Git, Cargo metadata, serde, TOML, and reporting machinery.
Keep deterministic graph traversal and semantic observations reusable across
jobs. Put optional business, ownership, lifecycle, runtime, and scenario inputs
in a validated, versioned context document. Existing JSON fields retain their
meaning; new evidence and design-assessment fields are additive. Changes to
scoring methodology are explicitly versioned in reports.

Each work item needs an observable regression test or an integration example.
The [design guide](design-analysis.md) records inputs, outputs, implementation
boundaries and limits. The [verification record](v0.4.0-verification.md) contains
job acceptance, automated checks, before/after self-analysis and Web screenshots.
All implementation and local verification items are complete; item 26 remains
open until both the registry and GitHub release have been verified.
