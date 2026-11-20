# cargo-coupling: C4 documentation and comparison with Litho

Applying `cargo c4doc` and `c4-rust-docs` produced four views with shared source evidence. Existing Litho documentation provides broader explanations of features and design reasoning; these C4 views provide a shorter guide to runtime boundaries and shared assessment entry points. Neither approach can establish all business semantics or operational conditions from code alone.

The guide follows a reader's questions: what the tool helps investigate, where analysis runs,
what CLI and Web share, and how a comparison against HEAD proceeds. Each page opens with a short
answer, then a diagram and reading notes that point out a useful detail. The notes retain scope,
conditions, and source entry points. View order controls the index and next-page links in both
editions; edit the model's `title`, `description`, and optional `notes` to maintain this reading path.

| Question | Document |
| --- | --- |
| Who uses the tool, and which external service does it contact? | [System Context](c4doc/context.md) |
| How do CLI, server, and browser code run? | [Container](c4doc/containers.md) |
| Where do CLI and Web share assessment logic? | [Component](c4doc/components.md) |
| How does a Web revision comparison proceed? | [Dynamic](c4doc/compare_request.md) |
| Where are all views and citations? | [Index](c4doc/README.md), [evidence](c4doc/evidence.md) |
| Where is the Japanese edition? | [Japanese C4 views](c4doc-ja/README.md), [Japanese model](c4doc-model.ja.json) |
| What should be edited and regenerated? | [Model](c4doc-model.json), [analysis record](c4doc-observation.json) |
| Where are detailed feature explanations? | [Existing Litho documentation](deepwiki/README.md), in Japanese |

## Conditions and observations

The comparison below records the September run on v0.4.0. The generated English
and Japanese guides track the current implementation. In v0.4.1 the Web UI
loads four versioned CDN scripts. A request with no comparison, focus or depth
uses the startup assessment. A concurrent request for a new assessment receives
HTTP 503 while the worker is busy.

Checked on 2026-09-14 against cargo-coupling 0.4.0, revision `5b019b2642f7dd7275d742d308b898b9047d2e6c`. No target Rust code was changed. Existing `.gitignore` changes, Litho configuration, and generated documents were preserved.

| Item | Existing Litho run | This cargo-c4doc and skill application |
| --- | --- | --- |
| Date | 2026-09-13 | 2026-09-14 |
| Tool | Litho 1.6.0 with a locally added Codex provider | cargo-c4doc 0.1.0 and c4-rust-docs |
| Source revision | The revision above | Same revision |
| Inputs | 84 files in 13 directories, plus existing design documentation | Two bin/lib targets; 54 modules and 54 unique Rust files; 681 definitions and 313 imports. HTML, JavaScript, and deployment files were inspected separately |
| Responsibilities | Litho research and writing stages invoke a model | Rust CLI collects syntax; the active coding agent investigates and authors the model; CLI validates and renders it |
| Output | Nine content files, two summaries, 20 diagrams in the content files | One model, four diagrams, ten generated Markdown/Mermaid files; separate observation record and this report |
| Review state | CDN, HTTP API details, and links were corrected after generation | Model claims were checked against inspected source |
| Duration | Run record reports 3794.04 seconds for all stages | Total agent research and writing time was not measured; no speed ratio is claimed |

Litho conditions and timing come from its [existing run record](litho-run.md). Litho was not rerun. The comparison uses the current, reviewed output, not the untouched automatic output. Existing Japanese reference documents were not translated or overwritten.

`cargo c4doc analyze` exited 0 with `complete: true` and no diagnostics. The seven unlinked Rust files were six integration-test files and one benchmark, not evidence of dead code. Before generating these C4 documents, `scan` listed 109 files without truncation or size-related skips. Ignore and extension exclusions still applied. Existing untracked documents were included, so this count is not comparable to Litho's 84-file input count.

The target CLI was also executed:

```sh
rtk proxy cargo run --locked -- coupling --design --exclude-tests --json --output ../cargo-c4doc/target/coupling-design-observation.json .
```

It exited 0 and analyzed 54 files and 54 modules with six months of Git history. Its `source_fingerprint`, `fnv1a64:b59a1c2e580b78b7`, matched the target observation recorded during the Litho work. The [observation record](c4doc-observation.json) retains settings, provenance, and coverage notes. `dirty: true` includes documentation and other working-tree additions.

The result still reports missing design context for future plans, lifecycle units, business rules, and runtime relationships. Without CODEOWNERS, ownership is known only where declared. Zero parse failures do not resolve those gaps.

## Boundaries established from implementation

The model has two main Containers. The Web branch in `main.rs` passes analyzed metrics to `start_server` and starts Tokio/Axum in the same native application. CLI and HTTP server are modes of that application, not independently deployed services. The lib target supplies linked implementation, not another Container in this usage scenario. See the [startup branch](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/src/main.rs#L335-L452).

The browser application executes JavaScript and calls HTTP APIs with `fetch`, so it is a separate application Container. RustEmbed asset packaging does not remove that runtime distinction. See [asset serving](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/src/web/routes.rs#L27-L100) and [browser comparison requests](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/web-assets/js/design-view.js#L118-L144). The views assume the default API destination; an overridden destination requires reviewing the deployment assumptions.

The Component view focuses on entry points for changing design assessment. Its six responsibilities are an agent modeling decision, not a uniquely correct partition automatically derived from 54 modules.

| Component | Main code and entry point | Responsibility in this view |
| --- | --- | --- |
| CLI execution and output | `main::run_coupling`, `cli_output`, `report` | Select analysis and output according to arguments and mode |
| Structural analysis | `analyzer::analyze_workspace_with_config`, `workspace`, `discovery`, `classification`, `metrics` | Convert selected sources and dependencies into shared metrics |
| History and baseline analysis | `history::analyze_ref`, `diff`, `volatility` | Evaluate historical sources under the same settings |
| Coupling evaluation | `balance::project::analyze_project_balance_with_thresholds` | Compute scores and identify issues |
| Design assessment | `design::assessment::assess` and `design/` | Combine structure, history, and declared context |
| HTTP adapter | `web::start_server`, `web::routes`, `web::design` | Hold state and provide assets and APIs |

The view omits supporting configuration code and many other call paths. `ProjectMetrics` is a shared data type and `AppState` is held HTTP state; neither is drawn as a separate responsibility-based Component.

| Input, output, or resource | Treatment |
| --- | --- |
| Rust sources, Cargo.toml, TOML settings, CODEOWNERS | User-managed inputs; configuration search can include ancestor directories |
| Git history | Accessed through the Git subprocess; this does not imply a GitHub service connection |
| Temporary worktrees | Created and removed for historical analysis, including writes to Git administrative data |
| Metrics and assessment | Held in the CLI/server process's memory |
| Reports | Written to standard output or a chosen file; no dedicated report service |

Evidence: [configuration search](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/src/config.rs#L509-L554), [historical analysis](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/src/history.rs#L325-L465), [server state](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/src/web/server.rs#L24-L110), and [output destinations](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/src/main.rs#L455-L490). Files can legitimately be modeled as C4 data stores; this table was sufficient for the selected questions.

## Differences from Litho and The C4 Model

The supplied `the-c4-model` checkout was read in its current working-tree state. Its HEAD was `5540b6d0cee0658757bcd7e587981dc98010a1cb`, but the Japanese chapters included uncommitted additions. The chapter hashes below identify the actual material used. This is a comparison against an Early Release draft and local translations, not a C4 certification.

| Dimension | Observation in existing Litho documents | C4 interpretation and response |
| --- | --- | --- |
| Context abstraction | The overview diagram places a local Rust project and Web browser beside users and the product. The prose distinguishes resources and product-owned UI | Chapter 3 focuses on people and software systems. Input resources moved to the table; the product's browser application appears as an internal Container |
| Container split | Architecture prose and diagrams already distinguish the Rust application from the browser application | This agrees with Chapter 2 and was retained. It is not a distinction first discovered by cargo-c4doc |
| Container boundary | RustApp, WebApp, Inputs, Worktree, and Reports appear without an enclosing product boundary | Chapter 4 recommends explicit system boundaries and application/data-store descriptions. The new view encloses the two product Containers and separates external tools/services |
| Component granularity | Architecture section 4.1 mixes functional groups, ProjectMetrics, AppState, and Embedded Web Assets. Many arrows have no labels. The prose explicitly calls it a main data-flow view | Useful as data flow, but ambiguous as a C4 Component view: responsibilities, types, state, and assets are mixed. The new view follows Chapter 5 with responsibilities, entry points, and purpose-labeled calls |
| Notation | Uses ordinary Mermaid flowcharts | Chapter 10 does not require a particular drawing syntax. cargo-c4doc also uses ordinary Mermaid; abstraction and explanation determine usefulness |
| Consistency and evidence | Descriptions and diagrams are maintained in individual documents, with varying source detail | Shared IDs, parents, and citations generate multiple views. Related views can be updated together |
| Breadth | Explains five functional areas, design reasoning, quality gates, and more | The four new views do not replace that coverage. Use Litho and existing design documents for broader functional understanding |

Compared passages: [Litho project overview](deepwiki/1-プロジェクト概要.md) and [architecture overview](deepwiki/2-アーキテクチャ概要.md). C4 sources are chapters 2–5 and 10 in the comparison checkout, under `the-c4-model/content/ja/`. The chapter digests are recorded below.

No Code view was added: Components and source links already expose the relevant change entry points. No Deployment view was added because the actual deployment environment was not verified. A Landscape view adds little to this single-tool question. Filling every C4 level is not the objective. See chapters 6, 8, and 9 in the same checkout.

## What additional source reading established

The v0.4.0 UI imported five scripts from unpkg.com, and the `3d-force-graph` URL had no version pin. Asset embedding alone therefore does not establish offline operation. This corroborates an observation already added to the reviewed Litho output. See [HTML imports](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/web-assets/index.html#L10-L14).

In the reviewed v0.4.0 implementation, `GET /api/design` returned the startup assessment when `changed_since` was empty. Otherwise it reanalyzed current sources, compared the fingerprint against startup state, and invoked shared `assess` logic. The Dynamic view covers that second path rather than claiming Web requests always recompute assessment. See [HTTP handling](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/src/web/design.rs#L19-L62).

The server binds to `127.0.0.1`, while the Compose Web service publishes `3000:3000`. Reachability through a normal container network needs investigation. No Docker connection failure was reproduced in this task; this remains a static configuration observation requiring deployment requirements and a runtime test. See [binding](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/src/web/server.rs#L112-L130) and [Compose](https://github.com/nwiizo/cargo-coupling/blob/5b019b2642f7dd7275d742d308b898b9047d2e6c/docker-compose.yml#L16-L24).

## Remaining cargo-c4doc limits and skill changes

| Limit | Applied guidance or future CLI improvement |
| --- | --- |
| Rust-only analysis misses browser connections | Follow static routes and embedding into HTML/JavaScript and external requests |
| One bin or process does not mechanically determine a Container | Inspect application modes, browser execution, and library hosting |
| Types, state, and files can be mistaken for Components | Map responsibilities to entry points; use supporting data/I/O tables |
| Matching source hashes do not prove semantic support | Separate `check --strict` from claim review; specify Dynamic inputs and branches |
| Unmodeled features and intentional omissions are not detected | Compare code changes and input scope with the model, not only `render --check` |
| Source-line links depend on the viewer | `#L...` uses GitHub conventions. Check local file existence separately from citation ranges; the evidence table also gives line numbers |
| Standalone images initially lacked sufficient title/legend context | Fixed after the deeper book review: `.mmd` now includes a visible title, scope, and legend. Dynamic legends identify dashed arrows as responses and distinguish static relationship evidence from execution order. Keep Markdown for scenario conditions and citations |
| Scenario ordering is not mechanically verified | Steps share static relationship evidence. Source review and explicit conditions are still needed; branches, parallelism, and trace correspondence remain future work |

The shared `c4-rust-docs` skill now includes these modeling and comparison checks. It is reused from cargo-coupling without a project-specific copy. Documentation defaults to English, with explicit guidance for requested Japanese output and stable IDs/evidence across translations. Renderer-owned headings and legends currently remain English. These changes affect documentation and skill instructions, not cargo-coupling's Rust behavior.

The subsequent C4 alignment review (`docs/c4-alignment.md` in the cargo-c4doc checkout) examines the original English chapters, including logical isolation, ownership, database schemas, message queues, model/code drift, and the distinction between C4 guidance and CLI restrictions. Its findings informed the shared skill and the cargo-c4doc renderer update used by both language editions here.

## Regeneration and verification

Run from the repository root with `cargo-c4doc` on PATH:

Use the shared `c4-rust-docs` skill to investigate and edit the models. The separate Rust CLI
validates and renders them; rendering an English model does not translate its contents.

| Edition | Model | Generated directory |
| --- | --- | --- |
| English, the default | `docs/c4doc-model.json` | `docs/c4doc` |
| Requested Japanese | `docs/c4doc-model.ja.json` | `docs/c4doc-ja` |

English generation:

```sh
rtk proxy cargo c4doc analyze . --output /tmp/coupling-analysis.json
rtk proxy cargo c4doc check docs/c4doc-model.json --root . --json --strict
rtk proxy cargo c4doc render docs/c4doc-model.json --root . --output docs/c4doc --strict
rtk proxy cargo c4doc render docs/c4doc-model.json --root . --output docs/c4doc --strict --check
```

Japanese generation:

```sh
rtk proxy cargo c4doc check docs/c4doc-model.ja.json --root . --json --strict
rtk proxy cargo c4doc render docs/c4doc-model.ja.json --root . --output docs/c4doc-ja --strict
rtk proxy cargo c4doc render docs/c4doc-model.ja.json --root . --output docs/c4doc-ja --strict --check
```

For agent requests, specify the output language explicitly when Japanese is needed, for example:

```text
Use $c4-rust-docs to update cargo-coupling's Japanese C4 documentation in docs/c4doc-ja.
Preserve the English edition and keep IDs, citations, and architectural meaning aligned.
```

Translate explanatory model fields, not IDs, schema fields, commands, paths, or evidence classifications.
When source changes affect a claim, review and update both models. Regenerate and check each edition.
Japanese text is supported; built-in headings and legends remain English.

Choose an unused analysis filename for each run: `analyze --output` refuses overwrites. After source changes, reread the cited passages before updating the model. Edit the model or this report, not managed output files.

To catch changes outside the model's cited files, retain the full `analyze` JSON from
before a code change and compare it with a fresh result:

```sh
rtk proxy cargo c4doc drift target/c4-before.json target/c4-after.json \
  --model docs/c4doc-model.json --check
```

These inputs must be full `analyze` outputs, not the summarized `c4doc-observation.json`.
Collect them with the same checkout path and package selection. For browser assets and
deployment/configuration files, retain matching `scan` results and add `--before-scan`
and `--after-scan`. A review candidate is not automatically another C4 Container; even
cited source changes need inspection. See `docs/drift.md` in the cargo-c4doc checkout.
When testing this feature, the earlier stored cargo-coupling analysis and a fresh analysis
reported no Rust changes and returned 0 with `--check`; that comparison omitted auxiliary scans.

Model validation reported no errors or warnings. This checks hierarchy and citation ranges/hashes, not every explanation's correctness. Target runtime verification covers the `--design` CLI execution only; browser interactions, the HTTP comparison path, and Docker connectivity were not exercised.

Both English and Japanese output passed `render --strict --check` after the reading-path update. Their models have identical IDs, structure, technologies, and source evidence after excluding translated display fields, including notes. All eight Mermaid sources rendered successfully with Mermaid CLI and local Chrome; the updated Context, Component, and Dynamic images were inspected. Offline link checking across README, CLAUDE, this report, and both generated editions found no missing local files. A separate heading-fragment check passed after excluding GitHub-style source-line fragments, whose ranges are checked by the model validator. External URL availability was not tested by the offline checks.

## C4 reference snapshot

SHA-256 of the locally inspected Japanese chapters, relative to `the-c4-model`:

```text
77ed6baf22e9b7fc2c9a27a29c93f5ff575027752a0553ae99cff58963e6d07f  content/ja/ch02.md
108497ee040c86bce5b676bc000c66f10a603fea18914c3410074ff20b9b92a9  content/ja/ch03.md
1eea1b37d727c3a69a0de5d8f89d1c05006c7ab114a41d0fa740f8b3d08b06d1  content/ja/ch04.md
0ce35d07037a4504f18932033b6481849f6efd808880caa736ba7dca3d16a67b  content/ja/ch05.md
24b9bab13b243a7c513246157beae9f4b5edd257b7a6eec2c951079d60900e05  content/ja/ch06.md
8315014008fed041ef47e3f4f23e481f8738c0da42f8cd863261842b870d9232  content/ja/ch08.md
a18a6c43f31aca67ccc37f7bd8da2a4245ffa18f8cc7c8db767b27edf7547123  content/ja/ch09.md
e177723ba7e96a78a59027b82f5ef88075fc2233b4304acfb73df2c4cf0cf11d  content/ja/ch10.md
```
