---
name: analyze
description: Run coupling analysis and interpret results. Use when analyzing Rust project coupling patterns and generating improvement proposals.
argument-hint: [path] [--summary|--verbose|--no-git|--history|--baseline REF]
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

# Show full Not Analyzed declaration
cargo run -- coupling --blind-spots ./src

# JSON output
cargo run -- coupling --json ./src

# History timeline
cargo run -- coupling --history ./src
cargo run -- coupling --history=8 --git-months=12 --json ./src

# Baseline diff
cargo run -- coupling --baseline main ./src

# Hotspots
cargo run -- coupling --hotspots ./src

# Impact analysis
cargo run -- coupling --impact <module> ./src

# CI/CD quality gate
cargo run -- coupling --check --min-grade=B ./src
cargo run -- coupling --check --baseline main --fail-on=high ./src
```

## Options

| Option | Description |
|--------|-------------|
| `--summary, -s` | Summary only |
| `--ai` | AI-friendly output |
| `--json` | JSON format |
| `--all` | Show all issues including Low |
| `--blind-spots` | Show full structural blind-spot manifest in text output |
| `--history[=N]` | Analyze coupling health over git history |
| `--baseline REF` | Compare current issues with a baseline git ref |
| `--fail-on SEVERITY` | Severity threshold for `--check` and baseline ratchet |
| `--no-git` | Skip Git history analysis |
| `--max-deps N` | Dependency count threshold |
| `--max-dependents N` | Dependent count threshold |
| `--verbose` | Detailed output |

## Interpretation

See `.claude/docs/khononov-framework.md` for grade meanings.
See `.claude/docs/issue-types.md` for issue severity.

Output template: [output-template.md](output-template.md)
