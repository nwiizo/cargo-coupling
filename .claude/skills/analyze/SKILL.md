# Coupling Analysis Skill

Run coupling analysis on Rust projects.

## Basic Commands

```bash
# Basic analysis (strict mode, hides Low severity)
cargo run -- coupling ./src

# Summary only
cargo run -- coupling --summary ./src

# Japanese output
cargo run -- coupling --summary --japanese ./src

# Show all issues
cargo run -- coupling --summary --all ./src

# AI-friendly output
cargo run -- coupling --ai ./src
```

## Job-Focused Commands

```bash
# Hotspots: Top refactoring targets
cargo run -- coupling --hotspots ./src
cargo run -- coupling --hotspots=10 ./src

# Impact analysis
cargo run -- coupling --impact <module> ./src

# CI/CD quality gate
cargo run -- coupling --check ./src
cargo run -- coupling --check --min-grade=B ./src

# JSON output
cargo run -- coupling --json ./src
```

## Web Visualization

```bash
# Start web UI
cargo run -- coupling --web ./src

# Custom port
cargo run -- coupling --web --port 8080 ./src
```

## Interpretation

See `.claude/docs/khononov-framework.md` for grade meanings.
See `.claude/docs/issue-types.md` for issue severity.
