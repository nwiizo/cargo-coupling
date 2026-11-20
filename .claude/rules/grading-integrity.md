# Grade & Signal Integrity Rules

The product's value is a *trustable* coupling signal. These rules protect it.

## NEVER (gaming)
- Raise a grade with `--no-git`, by relocating files to reset git churn, by loosening thresholds, or
  by reclassifying `.coupling.toml` subdomains to suppress real signals.
- Treat the grade letter as a target to hit by any means. A gameable metric is worthless.

## MUST
- Improve a grade only by (a) genuine, behavior-preserving structural change, or (b) fixing a *real*
  false positive that is correct for ALL projects (not just this repo).
- When adding/adjusting an issue, use the [balance model](../../.agents/skills/balanced-coupling/SKILL.md)
  for conceptual tradeoffs and the current detector for exact conditions and severity. The model's
  qualitative table does not define CLI severity levels; confirm changes with regression cases.
- Use **essential** (subdomain) volatility for scoring when classified; route raw git churn to the
  `AccidentalVolatility` diagnostic, not to severity.
- Exempt expected-by-design patterns from defect flags: binary entrypoints (high fan-out / co-change),
  crate-root re-export facades (stable Contract), and stable central abstractions (high afferent OK).
- Keep CLI and Web grades consistent (same metrics + thresholds path).
- Use the same finding detector and severity in CLI, Web edges and inspectors.
  Rendering must not introduce another scoring model or reconstruct missing dimensions.
- Treat Git co-change as history evidence. It cannot establish Functional strength,
  High essential volatility, or a coordinate in the three-dimensional model.
- Keep unknown, inferred and declared evidence distinct. Focus selection is not
  an observed source change; a test candidate is not a passing behavioral check.

## Verify
- After scoring/issue changes: run `rtk proxy cargo run -- coupling --summary ./src` on this checkout
  and explain any grade change from the evidence; ensure no regression for a project without
  `.coupling.toml`.
- Verify model boundary cases and input-order independence with observable tests;
  track concept coverage and supported jobs in [model conformance](../../docs/modularity-conformance.md).
