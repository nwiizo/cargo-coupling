---
name: hotspots
description: Identify high-priority refactoring targets using hotspot scoring. Ranks modules by issue severity and coupling count.
argument-hint: [path]
---

# Hotspots - Refactoring Priority

## Execution Steps

1. Run `cargo run --release -- coupling --ai $ARGUMENTS` (default: `./src`)
2. Calculate hotspot score for each module
3. Display ranked list with recommendations

## Commands

```bash
# Default (top 5)
cargo run -- coupling --hotspots ./src

# Top 10
cargo run -- coupling --hotspots=10 ./src

# With explanations
cargo run -- coupling --hotspots --verbose ./src
```

## Scoring Formula

| Factor | Weight | Description |
|--------|--------|-------------|
| Issue count | x30 | Number of coupling issues |
| Coupling count | x5 | In/out dependency count |
| Health: Critical | +50 | Critical health status |
| Health: Needs Review | +20 | Needs review status |
| Circular dependency | +40 | Part of a cycle |

## Output Format

```markdown
## Refactoring Hotspots

### 1. [Module] (Score: XX)
- **Issues**: [Types and count]
- **Coupling**: In X / Out Y
- **Recommended action**: [Specific improvement]
```

## Analysis Dimensions

1. **Why problematic** — Issue types, coupling patterns
2. **Priority** — Blast radius, difficulty, expected benefit
3. **Refactoring proposals** — Interface separation, dependency inversion, module splitting, facade

## Notes

- Score 0 = no issues
- Circular dependencies = highest priority
- Focus on top 5 for actionable results
