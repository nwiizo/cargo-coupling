---
name: check-balance
description: Quick health check for coupling balance score. Use for daily score verification and CI quality gates.
argument-hint: [path]
---

# Check Balance - Quick Health Check

## Execution Steps

1. Run `cargo run -- coupling --summary $ARGUMENTS` (default: `./src`)
2. Check balance score and health grade
3. Provide brief advice if issues detected

## Commands

```bash
# Summary check
cargo run -- coupling --summary ./src

# Japanese output
cargo run -- coupling --summary --japanese ./src

# CI/CD quality gate
cargo run -- coupling --check --min-grade=C ./src
```

## Health Grades

| Score | Grade | Status | Action |
|-------|-------|--------|--------|
| 0.90-1.00 | A | Excellent | Maintain |
| 0.80-0.89 | B | Good | Minor improvements |
| 0.60-0.79 | C | Acceptable | Planned improvement |
| 0.40-0.59 | D | Needs improvement | Act soon |
| 0.00-0.39 | F | Critical | Immediate action |

## Issue Severity

| Severity | Action |
|----------|--------|
| Critical | Fix immediately |
| High | Fix within 1 week |
| Medium | Plan to fix |
| Low | Monitor |

## Next Steps

- Low score: Run `/analyze` for details
- Critical issues: Run `/full-review` for comprehensive analysis
