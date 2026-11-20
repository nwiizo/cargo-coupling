# Evidence

[All views](README.md)

Source digests cover the complete file. After a source changes, inspect the affected claim before replacing the citation with `cargo c4doc cite`.

| Claim | Status and references |
| --- | --- |
| Element: Rust developer | Observed: [README.md:7–21](../../README.md#L7-L21) |
| Element: cargo-coupling | Observed: [README.md:1–9](../../README.md#L1-L9) |
| Element: unpkg.com | Observed: [web-assets/index.html:10–13](../../web-assets/index.html#L10-L13) |
| Element: Cargo | Observed: [src/workspace.rs:65–75](../../src/workspace.rs#L65-L75) |
| Element: Git | Observed: [src/volatility.rs:174–217](../../src/volatility.rs#L174-L217); [src/history.rs:418–461](../../src/history.rs#L418-L461) |
| Element: CLI and embedded server | Observed: [src/main.rs:335–442](../../src/main.rs#L335-L442) |
| Element: Browser application | Observed: [web-assets/js/design-view.js:124–146](../../web-assets/js/design-view.js#L124-L146) |
| Element: CLI execution and output | Observed: [src/main.rs:335–517](../../src/main.rs#L335-L517) |
| Element: Structural analysis | Observed: [src/analyzer.rs:1284–1305](../../src/analyzer.rs#L1284-L1305) |
| Element: History and baseline analysis | Observed: [src/history.rs:325–374](../../src/history.rs#L325-L374) |
| Element: Coupling evaluation | Observed: [src/balance/project.rs:19–57](../../src/balance/project.rs#L19-L57) |
| Element: Design assessment | Observed: [src/design/assessment.rs:34–139](../../src/design/assessment.rs#L34-L139) |
| Element: HTTP adapter | Observed: [src/web/routes.rs:27–100](../../src/web/routes.rs#L27-L100); [src/web/design.rs:19–74](../../src/web/design.rs#L19-L74) |
| Relationship: Obtains coupling analysis for change decisions | Observed: [README.md:7–21](../../README.md#L7-L21) |
| Relationship: Retrieves libraries for the Web UI | Observed: [web-assets/index.html:10–13](../../web-assets/index.html#L10-L13) |
| Relationship: Selects the analysis target and options | Observed: [README.md:40–55](../../README.md#L40-L55) |
| Relationship: Explores revision comparisons | Observed: [web-assets/js/design-view.js:130–146](../../web-assets/js/design-view.js#L130-L146) |
| Relationship: Requests UI assets and analysis data | Observed: [src/web/routes.rs:83–100](../../src/web/routes.rs#L83-L100) |
| Relationship: Loads visualization libraries | Observed: [web-assets/index.html:10–13](../../web-assets/index.html#L10-L13) |
| Relationship: Queries workspace and dependencies | Observed: [src/workspace.rs:65–75](../../src/workspace.rs#L65-L75) |
| Relationship: Reads history and checks out revisions | Observed: [src/volatility.rs:174–217](../../src/volatility.rs#L174-L217); [src/history.rs:418–461](../../src/history.rs#L418-L461) |
| Relationship: Requests shared metrics | Observed: [src/main.rs:335–339](../../src/main.rs#L335-L339) |
| Relationship: Starts Web mode | Observed: [src/main.rs:414–442](../../src/main.rs#L414-L442) |
| Relationship: Requests design assessment | Observed: [src/main.rs:485–505](../../src/main.rs#L485-L505) |
| Relationship: Requests assessment or comparison | Observed: [web-assets/js/design-view.js:124–142](../../web-assets/js/design-view.js#L124-L142); [src/web/design.rs:19–74](../../src/web/design.rs#L19-L74) |
| Relationship: Checks source freshness before comparison | Observed: [src/web/design.rs:49–59](../../src/web/design.rs#L49-L59) |
| Relationship: Calls shared assessment | Observed: [src/web/server.rs:87–109](../../src/web/server.rs#L87-L109); [src/web/design.rs:60–74](../../src/web/design.rs#L60-L74) |
| Relationship: Evaluates current coupling | Observed: [src/design/assessment.rs:114–117](../../src/design/assessment.rs#L114-L117) |
| Relationship: Evaluates the reference revision | Observed: [src/design/assessment.rs:117–133](../../src/design/assessment.rs#L117-L133) |
| Relationship: Requests metrics for past sources | Observed: [src/history.rs:325–343](../../src/history.rs#L325-L343) |
| Relationship: Evaluates past coupling with current settings | Observed: [src/history.rs:364–369](../../src/history.rs#L364-L369) |
