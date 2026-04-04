---
name: review
description: Modularity review using Balanced Coupling model. Combines automated analysis with semantic code review to find implicit coupling and design issues.
argument-hint: [path]
allowed-tools: Read, Grep, Glob, Bash
---

# Modularity Review

Structured review workflow inspired by Khononov's modularity analysis.
Combines cargo-coupling's automated metrics with semantic code understanding.

## Workflow

### Step 1: Automated Analysis

```bash
cargo run -- coupling --ai $ARGUMENTS
```

Parse the output to understand current coupling state.

### Step 2: Subdomain Classification

Read `.coupling.toml` for existing subdomain config.
If absent, examine the codebase and suggest classification:

- **Core**: Modules providing competitive advantage (frequently evolving)
- **Supporting**: Stable business logic (CRUD, ETL, data pipelines)
- **Generic**: Solved problems (auth, logging, config, web framework)

### Step 3: Map Integrations

For each detected coupling, evaluate across 3 dimensions:

1. **Strength**: What knowledge is shared? Is it implicit or explicit?
2. **Distance**: Code structure + team boundaries + runtime coupling
3. **Volatility**: Business-driven (subdomain) + git-history-based

Look beyond explicit code dependencies for **implicit coupling**:
- Duplicated business logic across modules
- Shared magic constants/strings
- Assumptions about data format or ordering (connascence of meaning)
- Co-changing files without explicit dependencies

### Step 4: Apply Balance Rule

For each integration:
```
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

Flag issues by severity:
- **Critical**: High strength + high distance + high volatility
- **Significant**: Unbalanced in moderately volatile area
- **Minor**: Unbalanced in low-volatility area

### Step 5: Generate Review

For each flagged issue, document:

1. **What**: Which modules and what knowledge is shared
2. **Why problematic**: Impact on changeability, cascading risk
3. **Recommendation**: Concrete improvement with Rust code example
4. **Priority**: Based on volatility and business impact

## Output Format

```markdown
# Modularity Review

**Date**: YYYY-MM-DD
**Scope**: [path]
**Health Grade**: [A-F]

## Subdomain Classification

| Module | Subdomain | Rationale |
|--------|-----------|-----------|

## Issues Found

### [Critical/Significant/Minor]: [Title]

**Modules**: source → target
**Strength**: [level] | **Distance**: [level] | **Volatility**: [level]

**Knowledge Leakage**: What internal knowledge is exposed
**Cascading Changes**: What breaks when this changes
**Recommendation**: Concrete fix with code example

## Good Design Decisions
[Patterns worth maintaining]

## Summary
[Key takeaways and prioritized action items]
```
