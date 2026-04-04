---
name: cargo-coupling
description: General CLI reference for cargo-coupling. All commands, options, config, and health grades.
user-invocable: false
---

# cargo-coupling CLI Reference

## Commands

```bash
cargo run -- coupling ./src                          # Basic analysis
cargo run -- coupling --summary ./src                # Summary only
cargo run -- coupling --summary --japanese ./src     # Japanese
cargo run -- coupling --ai ./src                     # AI-friendly
cargo run -- coupling --json ./src                   # JSON output
cargo run -- coupling -o report.md ./src             # File output
cargo run -- coupling --all ./src                    # Include Low severity
cargo run -- coupling --exclude-tests ./src          # Exclude test code
cargo run -- coupling --no-git ./src                 # Skip Git analysis
cargo run -- coupling --hotspots ./src               # Refactoring hotspots
cargo run -- coupling --hotspots=10 ./src            # Top N hotspots
cargo run -- coupling --impact <module> ./src        # Impact analysis
cargo run -- coupling --trace <function> ./src       # Dependency trace
cargo run -- coupling --check --min-grade B ./src    # CI quality gate
cargo run -- coupling --web ./src                    # Web UI
cargo run -- coupling --web --port 8080 ./src        # Custom port
cargo run -- coupling --max-deps 20 --max-dependents 25 ./src  # Custom thresholds
```

## Config File (.coupling.toml)

```toml
[thresholds]
max_deps = 15
max_dependents = 20

[analysis]
exclude_tests = true
prelude_modules = ["prelude", "ext"]
exclude = ["generated/*"]
```

## Health Grades

| Grade | Score | Meaning |
|-------|-------|---------|
| A | 0.90-1.00 | Well-balanced |
| B | 0.80-0.89 | Healthy |
| C | 0.60-0.79 | Needs Attention |
| D | 0.40-0.59 | At Risk |
| F | 0.00-0.39 | Critical |
