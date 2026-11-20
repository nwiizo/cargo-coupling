# Coupling skill evaluation cases

Use these cases after changes to descriptions, scope or model guidance. They are
acceptance criteria, not a record of measured model success. For an authorized
model evaluation, hold the model/version, repository fixture and prompts fixed;
save the selected skill, actions and result outside Git. Compare the old and new
instructions, including repeated runs, before claiming improved adherence.

| Request | Expected route | Observable result |
| --- | --- | --- |
| Explain why a strong, nearby dependency can be reasonable | `balanced-coupling` | Explains coordinated change and proximity; no automatic extraction |
| Review Rust module boundaries in `./src` | `coupling-review` | Source-backed findings, essential volatility, retain/change comparison and unknowns |
| This Generic module has no churn; can its provider still change? | `balanced-coupling` | Separates stable functionality from provider/implementation changes |
| These files co-change; should I introduce a shared trait? | `coupling-review`, model as needed | Checks shared requirements and behavior before recommending an abstraction |
| Run a report for `./src` | `analyze` | Runs/interprets the CLI; conceptual guidance alone is insufficient |
| Give me only the grade | `check-balance` | Bounded health summary, no unsolicited full review |
| Review this Japanese essay | User-level prose skill if relevant | Does not activate the Rust review workflow |
| Implement the confirmed boundary fix | `refactor` | Completes the authorized change and verification; no proposal-only stopping point |

For the semantic cases, reject outputs that infer Functional strength from
co-change alone, infer safe replaceability from a grade, confuse focused modules
with observed Git changes, or claim test candidates have passed. Missing domain
classification or owners should produce bounded uncertainty, not invented facts
or a mandatory approval stage.

For rule scope, `src/web/graph.rs` should match both Rust and Web guidance;
`web-assets/js/ui-controls.js` should match Web guidance; an unrelated Markdown
edit should not load either through Claude's path matcher. Codex uses the explicit
repository document routing instead. Validate YAML and local links separately.

The [model conformance checks](../docs/modularity-conformance.md) test the Rust/JS
behavior these instructions describe. They cannot establish skill-trigger rates
or the behavior of another model. This revision has source review and native
validation; no isolated model A/B benchmark is asserted.
