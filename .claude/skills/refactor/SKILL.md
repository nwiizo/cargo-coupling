---
name: refactor
description: Generate concrete refactoring proposals with Before/After code based on coupling analysis results.
argument-hint: [path] [issue-type: global-complexity|cascading-change|inappropriate-intimacy|high-efferent|high-afferent|all]
---

# Refactor - Refactoring Proposals

## Execution Steps

1. Run `cargo run -- coupling $ARGUMENTS` to analyze
2. Identify specified issue type (default: all)
3. Propose refactoring steps with Before/After code examples
4. Present priority table and phased migration plan

## Issue Types

| Type | Description |
|------|-------------|
| `global-complexity` | Strong coupling to distant modules |
| `cascading-change` | Coupling to volatile modules |
| `inappropriate-intimacy` | Internal access across boundaries |
| `high-efferent` | Too many outgoing dependencies |
| `high-afferent` | Too many incoming dependencies |
| `all` | All issue types (default) |

## Verification

```bash
# Before refactoring
cargo run -- coupling --summary ./src > before.txt

# After refactoring
cargo run -- coupling --summary ./src > after.txt

# Compare
diff before.txt after.txt
```

## Guidelines

- Split large changes into small commits
- Verify tests pass at each step
- Document change intent for reviewers

Code pattern examples: [patterns.md](patterns.md)
