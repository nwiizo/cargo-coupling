# CLAUDE.md

Rust CLI tool for coupling analysis based on Vlad Khononov's "Balancing Coupling in Software Design".

## Features Overview

### CLI Analysis

```bash
# Basic analysis
cargo run -- coupling ./src

# Summary only
cargo run -- coupling --summary ./src

# AI-friendly output (for Claude, Copilot, etc.)
cargo run -- coupling --ai ./src

# With custom thresholds
cargo run -- coupling --max-deps 20 --max-dependents 25 ./src

# Skip git analysis (faster)
cargo run -- coupling --no-git ./src

# Verbose with timing
cargo run -- coupling --verbose --timing ./src
```

### Job-Focused CLI Commands

```bash
# Hotspots: Find top refactoring targets (default: 5)
cargo run -- coupling --hotspots ./src
cargo run -- coupling --hotspots=10 ./src

# Impact: Analyze change impact for a module
cargo run -- coupling --impact analyzer ./src
cargo run -- coupling --impact main ./src

# Check: CI/CD quality gate (returns exit code 1 on failure)
cargo run -- coupling --check ./src
cargo run -- coupling --check --min-grade=B ./src
cargo run -- coupling --check --max-critical=0 --max-circular=0 ./src
cargo run -- coupling --check --fail-on=high ./src

# JSON: Machine-readable output
cargo run -- coupling --json ./src
cargo run -- coupling --json ./src | jq '.hotspots[0]'
```

### Web Visualization

```bash
# Start interactive web UI
cargo run -- coupling --web ./src

# Custom port
cargo run -- coupling --web --port 8080 ./src

# Don't auto-open browser
cargo run -- coupling --web --no-open ./src
```

**Web UI Features:**
- Interactive graph with Cytoscape.js
- 5 layout options: force-directed, concentric, circle, grid, breadthfirst
- Filtering by strength, distance, volatility, balance score
- Search with keyboard shortcuts (/, f, r, e, Esc, ?)
- Source code viewing with syntax highlighting
- Resizable sidebar

**JTBD Panels:**
- **Hotspots**: Top refactoring targets ranked by severity
- **Key Modules**: Sortable rankings (connections/issues/health)
- **Blast Radius**: Impact analysis with risk score
- **Clusters**: Architecture grouping detection

## 5 Coupling Dimensions

| Dimension | Description | Values |
|-----------|-------------|--------|
| **Strength** | How tightly coupled | Intrusive > Functional > Model > Contract |
| **Distance** | Module proximity | SameFunction > SameModule > DifferentModule > DifferentCrate |
| **Volatility** | Change frequency | High > Medium > Low (from git history) |
| **Balance** | Strength vs Distance trade-off | 0.0-1.0 (higher is better) |
| **Connascence** | Type of coupling | Name, Type, Position, Algorithm, etc. |

## Issue Types Detected

| Issue | Severity | Description |
|-------|----------|-------------|
| CircularDependency | Critical | Modules depend on each other |
| GlobalComplexity | High | Too many strong external dependencies |
| CascadingChangeRisk | High | Changes likely to cascade |
| InappropriateIntimacy | Medium | Internal details exposed |
| HighEfferentCoupling | Medium | Too many outgoing dependencies |
| HighAfferentCoupling | Medium | Too many incoming dependencies |

## Quick Commands

```bash
cargo build                    # Build
cargo test                     # Run tests
cargo clippy -- -D warnings    # Lint
cargo fmt                      # Format
cargo bench                    # Benchmarks
```

## Key Files

| File | Purpose |
|------|---------|
| `src/analyzer.rs` | AST analysis with syn |
| `src/balance.rs` | Balance score and issue detection |
| `src/aposd.rs` | APOSD metrics (depth, pass-through, cognitive load) |
| `src/metrics.rs` | Data structures and types |
| `src/connascence.rs` | Connascence pattern detection |
| `src/volatility.rs` | Git history volatility analysis |
| `src/temporal.rs` | Temporal coupling patterns |
| `src/report.rs` | Report generation |
| `src/cli_output.rs` | Job-focused CLI output (hotspots, impact, check, json) |
| `src/web/` | Web visualization server |

## Configuration (.coupling.toml)

```toml
[thresholds]
max_efferent_coupling = 15
max_afferent_coupling = 20
balance_score_warning = 0.6
balance_score_critical = 0.4

[volatility]
high_threshold = 10
medium_threshold = 5

[ignore]
patterns = ["**/tests/**", "**/benches/**"]
```

## Custom Commands

| Command | Description |
|---------|-------------|
| `/web` | Start web visualization (recommended) |
| `/analyze` | Run coupling analysis with interpretation |
| `/check-balance` | Quick balance score check |
| `/hotspots` | Find high-priority refactoring targets |
| `/full-review` | Comprehensive architecture review |
| `/refactor` | Get refactoring suggestions |
| `/explain-issue` | Explain a specific issue type |

## Before Release

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Jobs to be Done (JTBD)

| Job | Description | CLI | Web UI |
|-----|-------------|-----|--------|
| **Change Impact** | 変更の影響範囲を事前把握 | `--impact <module>` | Blast Radius |
| **Refactoring Priority** | 費用対効果の高いリファクタリング対象特定 | `--hotspots` | Hotspots Panel |
| **Architecture Understanding** | モジュール間依存関係の把握 | `--json` | Graph + Clusters |
| **Code Review** | 新しいカップリング問題の検出 | `--ai` | Issue List |
| **Quality Monitoring** | 健全性の継続的監視 | `--check`, `--summary` | Health Grade |

詳細: `.claude/docs/jobs-to-be-done.md`

## References

- JTBD: `.claude/docs/jobs-to-be-done.md`
- Commands: `.claude/commands/`
- Agents: `.claude/agents/`
- Architecture: `.claude/docs/architecture.md`
- Book: "Balancing Coupling in Software Design" by Vlad Khononov
