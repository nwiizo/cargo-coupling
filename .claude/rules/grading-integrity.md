# Grade & Signal Integrity Rules

The product's value is a *trustable* coupling signal. These rules protect it.

## NEVER (gaming)
- Raise a grade with `--no-git`, by relocating files to reset git churn, by loosening thresholds, or
  by reclassifying `.coupling.toml` subdomains to suppress real signals.
- Treat the grade letter as a target to hit by any means. A gameable metric is worthless.

## MUST
- Improve a grade only by (a) genuine, behavior-preserving structural change, or (b) fixing a *real*
  false positive that is correct for ALL projects (not just this repo).
- When adding/adjusting an issue, follow `.claude/skills/balanced-coupling/SKILL.md` severity table:
  Strong+Far+**High** = Global Complexity (act); Strong+Far+**Low** = Acceptable (Minor).
- Use **essential** (subdomain) volatility for scoring when classified; route raw git churn to the
  `AccidentalVolatility` diagnostic, not to severity.
- Exempt expected-by-design patterns from defect flags: binary entrypoints (high fan-out / co-change),
  crate-root re-export facades (stable Contract), and stable central abstractions (high afferent OK).
- Keep CLI and Web grades consistent (same metrics + thresholds path).

## Verify
- After scoring/issue changes: run `cargo coupling ./src` (dogfood) and confirm the grade moved for a
  *real* reason; ensure no regression for a project without `.coupling.toml`.
