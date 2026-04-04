---
name: analyze
description: Run coupling analysis and interpret results. Use when analyzing Rust project coupling patterns and generating improvement proposals.
argument-hint: [path] [--summary|--verbose|--no-git]
---

# Analyze - Coupling Analysis

## Execution Steps

1. Run `cargo run -- coupling $ARGUMENTS` (default: `./src`)
2. Interpret results as Balance Advisor
3. Present concrete improvement proposals

## Commands

```bash
# Basic analysis (strict mode, hides Low severity)
cargo run -- coupling ./src

# Summary only
cargo run -- coupling --summary ./src

# Japanese output
cargo run -- coupling --summary --japanese ./src

# AI-friendly output
cargo run -- coupling --ai ./src

# Show all issues including Low
cargo run -- coupling --all ./src

# JSON output
cargo run -- coupling --json ./src

# Hotspots
cargo run -- coupling --hotspots ./src

# Impact analysis
cargo run -- coupling --impact <module> ./src

# CI/CD quality gate
cargo run -- coupling --check --min-grade=B ./src
```

## Options

| Option | Description |
|--------|-------------|
| `--summary, -s` | Summary only |
| `--ai` | AI-friendly output |
| `--json` | JSON format |
| `--all` | Show all issues including Low |
| `--no-git` | Skip Git history analysis |
| `--max-deps N` | Dependency count threshold |
| `--max-dependents N` | Dependent count threshold |
| `--verbose` | Detailed output |

## Interpretation

See `.claude/docs/khononov-framework.md` for grade meanings.
See `.claude/docs/issue-types.md` for issue severity.

Output template: [output-template.md](output-template.md)
