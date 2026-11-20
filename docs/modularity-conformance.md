# Modularity concepts and job verification

Reference reviewed on 2026-11-11: Vlad Khononov's
[modularity repository at bcdca9a](https://github.com/vladikk/modularity/tree/bcdca9a595764b9aa88d5d8d6020f52e7f5f1f52).
Its review, design and balanced-coupling skills describe shared knowledge,
strength, distance, volatility and the tradeoff between coordinated change and
independent lifecycles. The upstream material is CC BY-NC-SA 4.0; this project
uses an independent implementation and explanation of the concepts, without
copying the upstream skill bodies or templates.

## What is deterministic

For fixed source, configuration, Git history/window, design context and tool
version, classification and reporting are code-defined. Conceptual fidelity is
checked with observable invariants and counterexamples, rather than identical
terminology. Neither the numeric heuristics nor the agent instructions prove
behavior preservation, identify every shared requirement, or make LLM judgments
deterministic. Time-relative history windows must also be held comparable.

| Concept | Implementation and regression evidence | Limit |
| --- | --- | --- |
| Balance combines all three dimensions | `src/balance/score.rs`: eight binary extremes, stability compensation, volatile cohesion tests | Continuous values are this tool's approximation, not measured failure probabilities |
| Strength concerns shared knowledge | AST classification plus declared rules in `src/design/`; `agent_review_traces_declared_rules_without_inventing_code_dependencies` | Calls, fields, traits and matching syntax are clues, not proof of a business relationship |
| Distance has an abstraction frame | Fixed Rust source categories; hierarchy, ownership, lifecycle and runtime evidence remain separate | Focus and hierarchy views do not rescore distances; source distance does not infer team or network distance |
| Generic functionality can have a changing implementation | `generic_provider_changes_and_unknown_owners_remain_review_questions` exposes provider-change questions and hypothetical volatility | Essential classification remains authoritative; a scenario never silently overwrites measured evidence |
| Proximity reduces joint-change cost but binds lifecycles | Context lifecycle/runtime declarations and constraint-aware alternatives in `tests/design_e2e.rs` | Async transport alone does not reduce shared knowledge; external operating constraints must be supplied |
| Co-change is evidence, not a semantic conclusion | `temporal_edges_reuse_report_findings_without_inventing_dimensions`; Web model tests | No invented strength/volatility/balance; investigate shared requirements before extracting code |
| Views share one assessment | Backend issue/severity reuse; `cohesive_edges_do_not_receive_web_only_risk_findings`; JS backend-result and reversed parallel-observation tests | Aggregated edges retain one real observation's dimensions and labels; rendering must not combine incompatible measurements |
| Retaining a boundary is a valid choice | `src/design/planning.rs` alternatives and decision tests | Equal-balance observations use a documented dimension tie-break, not input order |
| Explicit focus is not an observed change | `review_is_repeatable_and_focus_does_not_fabricate_a_git_change` | A Changed trigger requires actual changes mapped to its selected modules |
| Missing evidence stays visible | Future-change/ownership questions, analysis manifest, test verification status | No owner, test candidate, or unparsed file may be interpreted as affirmative safety evidence |

The Web API now emits `hidden_couplings[].dimensions: null`: a history-only edge
has no measured coordinate. The shipped UI ignores fabricated dimensions in older
snapshots too, retains co-change counts/ratios in network inspectors, and omits
those edges from Dimension Space. Consumers must accept a nullable `dimensions`
field; normal AST edge dimensions retain their existing shape.

## Feature choices grounded in the book

The chapter references below use the English text in the `balancing-coupling-in`
checkout at revision `7c62a7d67e9f51ecb9dad823cb8d93b40794a48e`.
The working question is which other code a particular change could require a
maintainer to revisit. A signal earns a place in the tool when a counterexample
can distinguish a meaningful change relationship from a coincidental one.

| Chapters | Decision | Acceptance case or missing evidence |
| --- | --- | --- |
| 3–4, interactions and modularity | Count distinct neighbours in module fan-in and fan-out. Keep individual source observations for inspection. | Ten references to one provider have breadth one. Introducing a second provider increases breadth. A module calling itself contributes zero external neighbours. |
| 5–7, knowledge and connascence | Retain separate source evidence, shared-rule declarations and test candidates. Defer automatic extraction suggestions based on similar function bodies. | Two equal formulas owned by different policies can change independently. A declared invariant and a test exercising both consumers give a reviewer a reason to consider sharing the implementation. |
| 8, distance | Retain lifecycle and ownership declarations alongside Rust distance. Defer a combined organizational-distance score. | The same crate layout can span one or two release units. To justify a new signal, show which release or team must coordinate a concrete change. |
| 9, upstream volatility | Resolve the provider before assigning churn, subdomain or inherited exposure. Keep missing history distinct from a completed window with zero changes. | A strong external dependency can share a name with a weak internal dependency. Neither its strength nor the local provider’s volatility may transfer between them. |
| 10–11, balance and rebalancing | Retain the existing balance calculation, scenarios, baseline comparison and recorded decision triggers. A separate 1–10 calculator would require maintainers to enter judgments already expressible in scenarios. | A provider becoming business-critical can change the scenario while source stays fixed. A confirmed change can reopen a recorded decision. Adding a second scale supplies no new evidence for either job. |
| 12, abstraction levels | Keep package and module identities in workspace reports. Defer independent service-level grading until the endpoints and frame of each integration are supplied. | Two crates can each own a module named `shared`. One module's consumers must not be reported under the other. A package grouping alone cannot establish a service contract. |
| 13, practical tradeoffs | Keep retaining a boundary among the alternatives. Defer ranking options by predicted savings or prescribing an adapter from call syntax. | Compare two implementations of a real change, the consumers each touched, and the contract tests that still pass. Until such outcomes exist, savings and replaceability are unverified. |

The counterexamples for fan-in, workspace identity, provider classification and
inherited exposure run in the Rust tests. Runtime sequencing, business-rule
ownership and the cost of coordinating releases still require project evidence.

## Jobs and acceptance evidence

The four jobs in [the product guide](../.claude/docs/jobs-to-be-done.md) remain
hypotheses about customer needs. These checks establish tool capabilities:

| Job | Acceptance case | Still requires observation |
| --- | --- | --- |
| Prepare a change | Focus on a provider; include its consumer and current test candidates, exclude an unrelated module, expose unknown owners and future changes | Whether the real change's affected code and requirements were missed |
| Review a patch | Compare against a Git revision; focus alone leaves a Changed decision retained, an actual provider edit triggers review; repeated agent JSON is byte-identical with 1 and 4 Rayon threads | Review time and unnecessary requests on real PRs |
| Choose design work | Compare retain/change alternatives; explicit provider-replacement scenarios change hypothetical balance while leaving the observed Generic score intact | Whether the chosen boundary preserves real runtime/release requirements |
| Understand structure | Web tests preserve module identity, aggregate directed boundaries and omit unmeasured coordinates; inspectors distinguish source evidence from co-change | Whether users locate responsibilities faster and understand the encodings |

`tests/modularity_e2e.rs`, `tests/design_e2e.rs` and
`tests/web_structure.test.mjs` cover these cases. A listed behavioral test is a
candidate marked `not-run` until separately executed. Passing these fixtures does
not claim safe replacement or completion of a real customer's change.

## Verification workflow

Run focused regressions while changing behavior, then the repository's required
checks. The following checks exercise the model and jobs directly:

```bash
rtk proxy cargo test --locked --all-features --test modularity_e2e --test design_e2e
rtk proxy node --test tests/web_structure.test.mjs
rtk proxy cargo run -- coupling --summary ./src
```

Use the same history, thresholds and subdomain configuration for before/after
analysis. Explain changed findings instead of optimizing the grade. For UI edits,
also exercise the affected inspector and view in a browser. For instruction edits,
use the separate [skill cases](../.agents/skill-evaluation.md); native tests cannot
measure prompt adherence.

The internal refactor also applies the requested `rust-best-practices` checkout
at `5c94dc7`: unchecked change evidence uses `None`, a checked empty change set
uses `Some(empty)`, and current-only impact borrows existing metrics. Only a
baseline comparison constructs an owned merged snapshot. Fixed-fixture JSON and
the existing impact tests check that this ownership change preserves output.
