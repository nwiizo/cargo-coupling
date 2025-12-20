# CLAUDE.md

Rust CLI tool for coupling analysis based on Vlad Khononov's "Balancing Coupling in Software Design".

## Features Overview

### CLI Analysis

```bash
# Basic analysis (default: strict mode, hides Low severity)
cargo run -- coupling ./src

# Summary only
cargo run -- coupling --summary ./src

# Japanese output with explanations (日本語出力)
cargo run -- coupling --summary --japanese ./src
cargo run -- coupling --summary --jp ./src

# Show all issues including Low severity
cargo run -- coupling --summary --all ./src

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

# Hotspots with beginner-friendly explanations
cargo run -- coupling --hotspots --verbose ./src

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

### Web Visualization (Experimental)

> ⚠️ Web UI is experimental. Interface and features may change.

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

## 3 Coupling Dimensions (Khononov)

| Dimension | Description | Values |
|-----------|-------------|--------|
| **Strength** | How tightly coupled | Intrusive > Functional > Model > Contract |
| **Distance** | Module proximity | SameModule > DifferentModule > DifferentCrate |
| **Volatility** | Change frequency | High > Medium > Low (from git history) |

### Balance Formula

```
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

- Strong + Close = ✅ High Cohesion (ideal)
- Weak + Far = ✅ Loose Coupling (ideal)
- Strong + Far + Stable = 🤔 Acceptable
- Strong + Far + Volatile = ❌ Needs Refactoring

## Grade System

| Grade | Description | Action |
|-------|-------------|--------|
| **S** | Exceptional - possibly over-engineered | Consider simplifying |
| **A** | Good - no change needed | Target! |
| **B** | Acceptable | Standard baseline |
| **C** | Needs Attention | Minor fixes |
| **D** | Needs Improvement | Take action |
| **F** | Critical Issues | Urgent |

**Philosophy**: Aim for A, not S. S may indicate over-engineering.

## Issue Types Detected

| Issue | Severity | Description |
|-------|----------|-------------|
| CircularDependency | Critical | Modules depend on each other |
| GlobalComplexity | High | Too many strong external dependencies |
| CascadingChangeRisk | High | Changes likely to cascade |
| GodModule | Medium | Too many functions/types/impls |
| HighEfferentCoupling | Medium | Too many outgoing dependencies |
| HighAfferentCoupling | Medium | Too many incoming dependencies |
| InappropriateIntimacy | Medium | Internal details exposed |
| PublicFieldExposure | Low | Public fields (use getters) |
| PrimitiveObsession | Low | Too many primitive params (use newtype) |

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
| `src/analyzer.rs` | AST analysis with syn (newtype, serde detection) |
| `src/balance.rs` | Balance score and issue detection |
| `src/metrics.rs` | Data structures, 3D analysis, BalanceClassification |
| `src/volatility.rs` | Git history volatility analysis |
| `src/report.rs` | Report generation (English/Japanese) |
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

## Learnings & Design Decisions

### CLI UX

- **Tables fail in CLI**: Box-drawing characters break in many terminals. Use bullet points instead.
- **Strict mode as default**: Showing all issues creates noise. Default to hiding Low severity (60 issues → 3 actionable).
- **Opt-in verbosity**: Use `--all` to show everything, `--verbose` for educational explanations.
- **Multi-language support**: `--japanese`/`--jp` flag for localized explanations, not just translations.

### Khononov Framework

- **3 dimensions are sufficient**: Strength × Distance × Volatility covers all coupling concerns.
- **Balance formula works**: `(STRENGTH XOR DISTANCE) OR NOT VOLATILITY` accurately identifies problems.
- **5 classifications make results actionable**:
  - HighCohesion (strong+close) = ideal
  - LooseCoupling (weak+far) = ideal
  - Acceptable (strong+far+stable) = monitor
  - Pain (strong+far+volatile) = fix now
  - LocalComplexity (weak+close) = consider separating

### Rust-Specific Patterns

- **Newtype detection**: Single-field tuple structs (`struct UserId(u64)`) indicate good design.
- **Serde derive detection**: `#[derive(Serialize, Deserialize)]` identifies DTOs for separation analysis.
- **Public field exposure**: Direct field access across module boundaries is a code smell in Rust.
- **Visibility matters**: `pub(crate)` vs `pub` vs private changes coupling implications.

### Issue Severity

- **Critical**: CircularDependency - must fix, blocks refactoring
- **High**: GlobalComplexity, CascadingChangeRisk - architectural problems
- **Medium**: GodModule, HighEfferent/AfferentCoupling - maintenance burden
- **Low**: PublicFieldExposure, PrimitiveObsession - improvement opportunities

### Testing Insights

Real-world validation on OSS projects (bat, fd, eza, ripgrep) showed:
- Grade A projects exist (bat: 0.82, fd: 0.83) - tool isn't too strict
- Different architectures show different patterns - tool is sensitive
- Score variance (0.67-0.98) indicates meaningful differentiation

### What Didn't Work

- ~~Connascence types~~: Too granular, removed. Strength levels are sufficient.
- ~~APOSD metrics~~: Overlap with existing analysis, removed for simplicity.
- ~~Temporal coupling~~: Git-based detection was noisy, kept only volatility.

### Web UI Architecture

**Node ID Normalization (Critical Bug Fix)**:
- `graph.rs`: Node IDs from `metrics.modules` used short names (`analyzer`)
- Edge source/target used full paths (`cargo-coupling::analyzer`)
- **Fix**: Normalize all IDs to short names for internal modules, keep full paths for external crates
- Helper function: `get_short_name()` extracts last segment from `::` paths

**JavaScript Module Structure** (ES6 modules):
```
web-assets/js/
├── state.js      # Shared state & configuration
├── i18n.js       # Internationalization (EN/JA)
├── utils.js      # Utility functions (debounce, STABLE_CRATES, etc.)
├── graph.js      # Cytoscape setup, styles, layouts, analyzeCoupling()
├── ui.js         # UI components, filters, search, keyboard shortcuts
├── item-graph.js # Item-level dependency graph
└── main.js       # Entry point, initialization
```

**API Data Consistency**:
- `NodeMetrics` now includes `fn_count`, `type_count`, `impl_count` from backend
- Frontend should use API values, not recalculate from items array
- External crates get `0` for all counts (no source access)

**Center Mode vs Zoom Mode**:
- Center mode (`centerMode=true`): Re-layout graph with selected node at center (concentric)
- Zoom mode (`centerMode=false`): Just animate to center the node without re-layout
- Toggle with 'c' key or checkbox

## References

- JTBD: `.claude/docs/jobs-to-be-done.md`
- Commands: `.claude/commands/`
- Agents: `.claude/agents/`
- Architecture: `.claude/docs/architecture.md`
- Book: "Balancing Coupling in Software Design" by Vlad Khononov
