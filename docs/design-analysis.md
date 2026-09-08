# Change and design analysis

`cargo coupling --design` helps identify what to inspect before changing code,
compare design options, and revisit decisions when their assumptions change.
The CLI and **Plan a change** in the Web UI use the same Rust assessment.

```sh
cargo coupling --design .
cargo coupling --design --json . > design-report.json
cargo coupling --changed-since main --impact-depth 4 .
cargo coupling --impact analyzer --impact-depth 3 --json .
cargo coupling --web --context ./design-context.toml --no-open .
```

`--changed-since` and `--context` select design output even without `--design`.
Without `--impact-depth`, traversal reaches every reachable module; a supplied
positive depth reports whether additional paths were omitted. Git comparison
includes committed, staged, unstaged and untracked changes within the selected
scope. Package selection also includes its discovered `#[path]` source files.
Deleted items retain their original revision and line numbers.

## Jobs and available evidence

| Job | Output | Interpretation |
| --- | --- | --- |
| Understand the existing boundaries | Structure cards, directed matrix; item/module/package/workspace summaries | Directories and syntax boundaries are observations, not inferred business domains. |
| Find what a change could affect | Git items, multi-hop paths, candidate tests | Reachability is conservative; paths are not proof every consumer must change. |
| Explain instability | Essential volatility, observed changes and inherited exposure | A strong upstream path can expose a stable component; these are separate measures. |
| Check whether abstraction hides knowledge | Public signature exposure, matching record shapes, pure forwarding | Aliases and names are inspected syntactically; a wrapper can be justified. |
| Find reasons to change together | Identical bodies, Git co-change, declared business rules | Similarity alone does not justify shared code; confirm shared requirements. |
| Choose what to work on | Priorities, business weights, explicit estimates | The ordering is a relative heuristic, not predicted savings. |
| Compare design choices | Retain, hide knowledge, move closer, separate responsibilities; supplied scenarios | Hypothetical dimension changes do not edit source code. |
| Plan replacement of a dependency | Public API occurrences, consumers, exposure paths, boundary candidates | Import aliases are inspected; macros and complete type resolution are not covered. |
| Identify coordination costs | CODEOWNERS, explicit owners, shared build/test/release units | Shared lifecycle can matter without a code dependency. |
| Review runtime assumptions | Order, timing, transactions, shared state, sync/async declarations | Only supplied evidence is evaluated; async does not imply weak coupling. |
| Revisit an accepted decision | Reasons, triggers, review/retain/unknown status | Missing evidence produces pending checks instead of confirming retention. |

## Supply facts that source code cannot establish

The CLI searches upward for `.coupling-context.toml`, or reads the exact path
given to `--context`. A missing default file is reported as unknown context;
a missing explicitly requested file is an error. Start with `version = 1` and
add only facts you can support. Selectors are globs over reported module names;
use names from the JSON hierarchy to make the scope precise.

```toml
version = 1

[[components]]
name = "pricing"
modules = ["*policy"]
owners = ["@pricing"]
planned_changes = "Introduce regional pricing"
business_value = 3
effort_days = 2
build_unit = "shop"
test_unit = "checkout-suite"
release_unit = "shop"

[[components]]
name = "quotes"
modules = ["*service"]
owners = ["@checkout"]
frozen_reason = "Migration is awaiting an upstream API change"
release_unit = "shop"

[[rules]]
id = "regional-pricing"
modules = ["*policy", "*service"]
description = "Both must agree on the regional pricing rule"

[[relationships]]
id = "quote-consistency"
source = "*service"
target = "*policy"
kind = "transaction"
origin = "declared"
evidence = "A quote must use one consistent price revision"

[[scenarios]]
name = "Keep knowledge local"
description = "Compare moving the pricing responsibility into one boundary"
[[scenarios.changes]]
source = "*service"
target = "*policy"
distance = 0.0

[[decisions]]
id = "keep-together"
modules = ["*policy", "*service"]
reason = "Both ship as one unit"
review_on = ["planned-change", "cross-team", "changed", "new-issue"]
```

Unknown fields, unsupported versions, invalid globs, duplicate IDs, overlapping
component assignments, non-positive/non-finite business weights or effort,
and dimensions outside `[0, 1]` are errors. Unmatched selectors produce coverage
notes: facts for an unselected workspace member are not invalidated by analyzing
one package. Context does not overwrite observed Git churn or classify essential
volatility; existing `.coupling.toml` subdomains and volatility settings do that.

Runtime `kind` accepts `sequence`, `timing`, `transaction`, `shared-state`,
`synchronous`, and `asynchronous`. Set `origin = "observed"` only when `evidence`
describes an actual observation, for example a trace or an incident. The tool
does not collect runtime traces. CODEOWNERS uses `.github/CODEOWNERS`, then the
root file, then `docs/CODEOWNERS`; last matching ownership wins. Explicit owners
override file rules. Unsupported GitHub pattern forms are reported as coverage
notes, and absent owners remain unknown.

Decision triggers also accept `worsened-issue` and `context-changed`. `changed`
requires `--changed-since`; issue triggers require a baseline or change
comparison. For `context-changed`, copy the prior report's
`design.provenance.context_fingerprint` into the decision's
`context_fingerprint`. Saved fingerprint fields themselves are excluded from
that hash, so saving a decision does not immediately invalidate it. Declarations
remain versioned in Git; the Web UI reads them and does not edit the TOML file.

## Compare scores and priorities

Balance uses `max(abs(strength - distance), 1 - volatility)` for normalized
dimensions. Stability and proximity compensate for coupling. The methodology
identifier is `khononov-compensation-v2`; old multiplicative scores cannot be
compared directly. Health grades retain the issue-based grading policy.

Alternatives use the least balanced observation for each directed module pair.
Scenarios average matched observations, not all project dependencies; later
matching declarations override earlier values for each supplied dimension.
No matching observation produces `null` scores, not a zero score. Each option
includes assumptions and tradeoffs; adding a trait does not itself establish
encapsulation or prove that a business rule is shared.

Priority is `(severity weights + reachable consumers + inherited paths +
4 if planned + 4 if frozen) × business_value`. Critical/high/medium/low findings
weigh 8/4/2/1. JSON includes `value_per_effort` when an estimate is supplied.
The Web can order estimated work by priority per day; unestimated work remains
visible afterwards. Default ordering uses impact and importance without
inventing an effort estimate.

Every assessment records its schema, analyzer/scoring versions, scope, revision,
dirty state, source/config/context fingerprints, effective settings, Git window,
test exclusion, thresholds and traversal depth. Fingerprints are deterministic
content identities, not cryptographic signatures. `--baseline` and
`--changed-since` analyze both revisions using today's analyzer, scoring,
thresholds and effective configuration. This compares source changes under
fixed settings; it does not reproduce a past published score. A saved report
with different scope, configuration or methodology requires a fresh comparison.

Existing JSON fields remain available. Generic JSON schema version is 2;
`design` schema version is 1. `worsened_issues` joins new/resolved findings in
baseline output. `--check --baseline` fails for new or worsened findings at the
requested severity, while unchanged debt is retained.

## Use the Web to complete a change review

1. Start with **Structure**, choose a source directory, and inspect a module.
   The matrix row depends on the column; counts represent distinct module pairs.
   Use **Inspect connections** to focus the 2D graph on a neighborhood.
2. In **Plan a change**, select a change origin and depth, then **Trace impact**.
   Impact arrows run from the changed provider toward affected consumers,
   opposite to dependency arrows. Follow a path into the graph.
3. Enter a Git reference and choose **Compare changes**. Review **Changed source
   items**, candidate tests, **Baseline changes**, and **Read source**. Deleted
   items open their original Git revision. Source evidence distinguishes usage,
   inferred strength, location and missing information.
4. Compare **Design options** and declared **Scenarios**. Inspect ownership,
   lifecycle/runtime relationships, and **Retained decisions** before deciding.
   **Coverage & conditions** records what was available for that assessment;
   download JSON to retain the result.

The planning view uses the startup working-tree snapshot independently of the
history timeline. Restart the server after editing sources; a comparison that
would mix startup metrics with changed or newly added files returns a refresh
message. Source files must be inside the analyzed workspace, including when a
historical directory has since been deleted.

Both 2D and 3D show a bounded function/method/type preview and retain full names
in the inspector. Projected labels hide lower-priority collisions; zoom, focus,
or select a module to inspect its detail. The initial 3D network uses a stable
lattice to separate nodes. Those coordinates are layout only; measured
dimensions are in **Dimension-Space**. Left drag rotates, right drag pans, the
wheel zooms, and **Fit** resets framing. Label buttons support Tab and Enter
without intercepting canvas dragging. Structure cards provide a readable entry
point when a large graph cannot show every name at once.

## Rust boundaries and practical limits

- `design::source` observes syntax and source spans; `design::graph` resolves
  module aliases and traverses deterministic paths.
- `design::changes` reads Git ranges and records provenance; `assessment`
  assembles evidence and evaluates exposure/shared change reasons.
- `organization` resolves ownership and supplied lifecycle/runtime facts;
  `structure` describes hierarchy and external API exposure; `planning`
  evaluates alternatives, scenarios, priorities and retained decisions.
- `model` defines serializable results; `output` renders the human report.
  `web::design` adapts the shared assessment to HTTP. Both projections reuse
  label content and collision placement.
- `cli_output::impact`, `cli_output::hotspots` and `cli_output::json` own their
  respective reporting jobs. Public re-exports preserve existing caller paths.
  JSON evidence reuses one dependency graph per report; JSON and Web use the
  same essential-volatility adjustment as the core balance calculation.

Syntax is not Rust type checking: macro expansion, active `cfg` selection,
dynamic dispatch and full re-export resolution remain incomplete. Function
calls and names cannot establish shared business meaning. Identical-body
findings require at least 24 normalized tokens across modules; mirrored records
require at least two matching fields. These thresholds reduce trivial matches
and do not establish semantic duplication. Test candidates include matching
names and reachable modules, and may contain false positives. Excluding tests
reduces candidate coverage. Human summaries truncate long sections; JSON keeps
all reported observations. UI navigation and field labels support English and
Japanese; detailed diagnostic explanations and supplied evidence retain their
original text.

See [implementation tracking](implementation-todo.md) and the
[v0.4.0 verification record](v0.4.0-verification.md) for checks and screenshots.
