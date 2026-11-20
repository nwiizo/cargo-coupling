# Jobs to be done

The central job is to make a supported change decision: identify potential
consumers, preserve relevant requirements, and choose what to verify. Scores
and graphs support this job; improving a grade is not a product outcome.

| Situation | Desired progress | Existing entry point |
| --- | --- | --- |
| About to edit an unfamiliar module | Find affected code, declared shared rules, test candidates and unknowns | `--agent --focus MODULE`, `--impact MODULE`, Web Plan a change |
| Reviewing a PR or agent patch | Isolate new/worsened findings without requiring unrelated cleanup | `--changed-since REF`, `--baseline REF`, `--check --baseline REF` |
| Choosing design work | Compare future needs, retained boundaries, constraints and effort | `--design`, context components, alternatives and scenarios |
| Learning a codebase | Locate a responsibility and inspect neighboring boundaries | Structure, search, item trace and 2D/3D inspector |

## Product choices

- Share priority calculation across CLI, Web and design review. Planning context
  may explicitly change the ranking; UI-only score formulas may not.
- Keep code paths distinct from declared shared-rule paths and Git co-change.
  Neither syntax similarity nor co-change proves a shared business requirement.
- Keep reasons, evidence references and rejected alternatives in versioned
  context so implementation replacement does not erase prior learning.
- Expose missing information and output truncation. A test candidate is not a
  test result; a successful analysis is not permission to refactor.
- Keep global grades secondary. No density-based over-engineering warning,
  automatic cycle-breaking prescription, or failure-probability claim.

## Current implementation

History, baseline/ratchet, ownership, lifecycle, runtime declarations, scenarios,
retained decisions, inherited exposure and abstraction evidence already exist.
Do not propose them as new features. Agent JSON, module focus, declared-rule
impact and constraint-aware alternatives are documented in
[`docs/agent-review.md`](../../docs/agent-review.md).

## Verification and future work

The [concept and job checks](../../docs/modularity-conformance.md) map each job to
observable acceptance cases, counterexamples and remaining limits. Keep this map
current when changing analysis, planning or visualization behavior.

Evaluate missed affected modules, unnecessary review requests, and time to a
supported decision on real changes. Customer demand and these outcomes still
need observation; the jobs above are product hypotheses, not survey findings.
Additional graph effects, notifications, editor plugins and autonomous
regeneration should wait for a concrete unmet job. Static analysis does not
establish safe replaceability; behavioral evaluations and controlled runtime
experiments belong to their own tools and workflows.
