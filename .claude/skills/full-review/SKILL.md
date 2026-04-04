---
name: full-review
description: Comprehensive architecture review with 3 expert perspectives. Use before releases or for architecture evaluation.
argument-hint: [path]
---

# Full Review - Architecture Review

## Execution Flow

### Phase 1: Automated Analysis

```bash
cargo run -- coupling $ARGUMENTS  # default: ./src
cargo clippy -- -D warnings
cargo test
```

### Phase 2: Expert Review (parallel)

Uses personas from `.claude/agents/`:

1. **Balance Advisor (Vlad Khononov)** — 3D coupling balance analysis
2. **Architecture Critic** — Architecture risk and technical debt assessment
3. **Rust Idiomatic Expert** — Rust idioms and code quality review

### Phase 3: Integrated Report

- Prioritized improvements (Critical/High/Medium)
- Good design decisions to maintain
- Action plan (immediate/weekly/long-term)

## Output Sections

1. Executive Summary (overall score table)
2. Phase 1 automated analysis results
3. Phase 2 expert reviews
4. Integrated improvement proposals
5. Next steps
6. Review team messages

Detailed output template: [output-template.md](output-template.md)
