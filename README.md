# cargo-coupling

[![Crates.io](https://img.shields.io/crates/v/cargo-coupling.svg)](https://crates.io/crates/cargo-coupling)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

**Measure the "right distance" in your Rust code.**

`cargo-coupling` analyzes coupling in Rust projects based on Vlad Khononov's "Balancing Coupling in Software Design" framework. It calculates a **Balance Score** from three core dimensions: **Integration Strength**, **Distance**, and **Volatility**.

![CLI Output](docs/images/cli-output.png)

> ⚠️ **Experimental Project**
>
> This tool is currently experimental. The scoring algorithms, thresholds, and detected patterns are subject to change based on real-world feedback.
>
> **We want your input!** If you try this tool on your project, please share your experience:
> - Are the grades and scores meaningful for your codebase?
> - Are there false positives or patterns that shouldn't be flagged?
> - What additional metrics would be useful?
>
> Please open an issue at [GitHub Issues](https://github.com/nwiizo/cargo-coupling/issues) to discuss. Your feedback helps improve the tool for everyone.

## Quick Start

### 1. Install

```bash
cargo install cargo-coupling
```

Or use Docker:

```bash
docker pull ghcr.io/nwiizo/cargo-coupling
```

### 2. Analyze

```bash
# Analyze current project (default: shows only important issues)
cargo coupling ./src

# Show summary only
cargo coupling --summary ./src

# Japanese output with explanations (日本語出力)
cargo coupling --summary --japanese ./src
cargo coupling --summary --jp ./src

# Show all issues including Low severity
cargo coupling --summary --all ./src
```

### 3. Refactor with AI

```bash
# Generate AI-friendly output
cargo coupling --ai ./src
```

Copy the output and use this prompt with Claude, Copilot, or any AI coding assistant:

```
The following is the output of `cargo coupling --ai`, which analyzes coupling issues in a Rust project.
For each issue, suggest specific code changes to reduce coupling.
Focus on introducing traits, moving code closer, or breaking circular dependencies.
```

Example output:

```
Coupling Issues in my-project:
────────────────────────────────────────────────────────────

Grade: B (Good) | Score: 0.88 | Issues: 0 High, 5 Medium

Issues:

1. 🟡 api::handler → db::internal::Query
   Type: Global Complexity
   Problem: Intrusive coupling to db::internal::Query across module boundary
   Fix: Introduce trait `QueryTrait` with methods: // Extract required methods

2. 🟡 25 dependents → core::types
   Type: High Afferent Coupling
   Problem: Module core::types is depended on by 25 other components
   Fix: Introduce trait `TypesInterface` with methods: // Define stable public API
```

The AI will analyze patterns and suggest specific refactoring strategies.

### 4. Interactive Web Visualization (Experimental)

> ⚠️ **Experimental Feature**: The Web UI is currently in an experimental state. The interface, features, and behavior may change significantly in future versions.

![Web UI](docs/images/web-ui.png)

```bash
# Start interactive web UI
cargo coupling --web ./src

# Custom port
cargo coupling --web --port 8080 ./src
```

The web UI provides:
- Interactive graph visualization with Cytoscape.js
- **Hotspots panel**: Top refactoring targets ranked by severity
- **Blast Radius**: Impact analysis with risk score
- **Clusters**: Architecture grouping detection
- Filtering by strength, distance, volatility, balance score
- Source code viewing with syntax highlighting

### 5. Job-Focused CLI Commands

For quick, focused analysis without opening the web UI:

```bash
# Find top refactoring targets
cargo coupling --hotspots ./src
cargo coupling --hotspots=10 ./src

# With beginner-friendly explanations
cargo coupling --hotspots --verbose ./src

# Analyze change impact for a specific module
cargo coupling --impact main ./src
cargo coupling --impact analyzer ./src

# Trace dependencies for a specific function or type
cargo coupling --trace analyze_file ./src
cargo coupling --trace BalanceScore ./src

# CI/CD quality gate (exits with code 1 on failure)
cargo coupling --check ./src
cargo coupling --check --min-grade=B ./src
cargo coupling --check --max-critical=0 --max-circular=0 ./src

# Machine-readable JSON output
cargo coupling --json ./src
cargo coupling --json ./src | jq '.hotspots[0]'
```

Example `--hotspots --verbose` output:

```
#1 my-project::main (Score: 55)
   🟡 Medium: High Efferent Coupling

   💡 What it means:
      This module depends on too many other modules

   ⚠️  Why it's a problem:
      • Changes elsewhere may break this module
      • Testing requires many mocks/stubs
      • Hard to understand in isolation

   🔧 How to fix:
      Split into smaller modules with clear responsibilities
      e.g., Split main.rs into cli.rs, config.rs, runner.rs
```

### More Options

```bash
# Generate detailed report to file
cargo coupling -o report.md ./src

# Show timing information
cargo coupling --summary --timing ./src

# Use 4 threads for parallel processing
cargo coupling -j 4 ./src

# Skip Git history analysis for faster results
cargo coupling --no-git ./src
```

## Features

- **3-Dimensional Balance Score**: Calculates coupling balance based on **Integration Strength**, **Distance**, and **Volatility** (0.0 - 1.0)
- **Khononov Balance Formula**: `BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY`
- **Interactive Web UI**: `--web` flag starts a browser-based visualization with graph, hotspots, and blast radius analysis
- **Job-Focused CLI**: Quick commands for common tasks (`--hotspots`, `--impact`, `--check`, `--json`)
- **Japanese Support**: `--japanese` / `--jp` flag for Japanese output with explanations and design decision matrix
- **Noise Reduction**: Default strict mode hides Low severity issues (`--all` to show all)
- **Beginner-Friendly**: `--verbose` flag explains issues in plain language with fix examples
- **CI/CD Quality Gate**: `--check` command with configurable thresholds and exit codes
- **AI-Friendly Output**: `--ai` flag generates output optimized for coding agents (Claude, Copilot, etc.)
- **Rust Pattern Detection**: Detects newtype usage, serde derives, public fields, primitive obsession
- **Issue Detection**: Automatically identifies problematic coupling patterns (God Module, etc.)
- **Circular Dependency Detection**: Detects and reports dependency cycles
- **Visibility Tracking**: Analyzes Rust visibility modifiers (pub, pub(crate), etc.)
- **Git Integration**: Analyzes change frequency from Git history for volatility scoring
- **Configuration File**: Supports `.coupling.toml` for volatility overrides
- **Parallel Processing**: Uses Rayon for fast analysis of large codebases
- **Configurable Thresholds**: Customize dependency limits via CLI or config
- **Markdown Reports**: Generates detailed analysis reports
- **Cargo Integration**: Works as a cargo subcommand

## Khononov's Coupling Balance

**Coupling Balance** is a framework proposed by Vlad Khononov that evaluates coupling between modules across three dimensions to guide design decisions.

### Core Principle

Coupling is not inherently bad. What matters is **the balance between coupling strength, distance, and volatility**.

## The Three Dimensions

### 1. Strength (Integration Strength)

Represents how tightly components depend on each other.

| Level | Description | Rust Example | Score |
|-------|-------------|--------------|-------|
| **Intrusive** | Direct dependency on internal implementation | Direct access to `struct.field` | 1.00 (strong) |
| **Functional** | Dependency on behavior | Method calls on concrete types | 0.75 |
| **Model** | Dependency on data structures | Sharing type definitions | 0.50 |
| **Contract** | Dependency on interfaces only | Access via `trait` | 0.25 (weak) |

→ Lower in the table = **weaker** coupling (preferred)

### 2. Distance

The physical or logical distance between dependent components.

| Level | Description | Score |
|-------|-------------|-------|
| **Same Module** | Within the same module | 0.25 (close) |
| **Different Module** | Different module in the same crate | 0.50 |
| **External Crate** | Dependency on external crate | 1.00 (far) |

→ Lower in the table = **farther** distance

### 3. Volatility

How frequently a component changes (automatically calculated from Git history).

| Level | Description | Changes (6 months) | Score |
|-------|-------------|-------------------|-------|
| **Low** | Stable, rarely changes | 0-2 times | 0.00 |
| **Medium** | Occasionally changes | 3-10 times | 0.50 |
| **High** | Frequently changes | 11+ times | 1.00 |

> **Note**: Volatility requires Git history. Use `cargo coupling ./src` (not `--no-git`) to enable volatility analysis.

## The Balance Law

Good design follows this principle:

```
Strong coupling is only acceptable when distance is close OR volatility is low
```

Expressed as a logical formula:

```
BALANCED = (STRENGTH ≤ threshold) OR (DISTANCE = near) OR (VOLATILITY = low)
```

Or Khononov's formula:

```
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

- **STRENGTH XOR DISTANCE**: Strong coupling × close distance OR weak coupling × far distance = Good
- **OR NOT VOLATILITY**: Even if the above isn't satisfied, low volatility makes it acceptable

## Design Decision Matrix

| Strength | Distance | Volatility | Decision | Reason |
|----------|----------|------------|----------|--------|
| Strong | Close | Low-Medium | ✅ OK | High cohesion, changes are localized |
| Weak | Far | Any | ✅ OK | Loose coupling with healthy dependencies |
| Strong | Far | Any | ⚠️ Needs improvement | Change impact spreads widely (global complexity) |
| Strong | Any | High | ⚠️ Needs improvement | Changes cascade through the system |
| Weak | Close | Low | 🤔 Consider | Opportunity for integration (possibly over-modularized) |

## Improvement Patterns

### Pattern 1: Reducing Coupling Strength via Abstraction

**Problem**: Strong coupling + far distance

```
┌─────────────┐         ┌─────────────┐
│  Module A   │ ──────▶ │  Module B   │
│             │  strong  │  (impl)     │
└─────────────┘         └─────────────┘
       far distance (different module)
```

**Solution**: Introduce a Contract (trait)

```
┌─────────────┐         ┌─────────────┐
│  Module A   │ ──────▶ │   trait T   │
│             │   weak   │ (contract)  │
└─────────────┘         └─────────────┘
                              ▲
                              │ implements
                        ┌─────────────┐
                        │  Module B   │
                        │   (impl)    │
                        └─────────────┘
```

### Pattern 2: Isolating Volatility

**Problem**: Strong coupling + high volatility

**Solution**: Insert a stable interface layer

## Concrete Example (Rust)

### Before: Problematic Code

```rust
// module_a.rs
fn process_user(user: &User) {
    // Direct access to struct internal fields (Intrusive)
    let name = &user.name;           // ← strong coupling
    let age = user.age;              // ← strong coupling
    let email = &user.email_address; // ← breaks if field name changes
    // ...
}
```

```rust
// module_b.rs (frequently modified)
pub struct User {
    pub name: String,
    pub age: u32,
    pub email_address: String,  // ← renamed from email
}
```

**Issues**:
- Coupling strength: Intrusive (direct field access)
- Distance: Different Module
- Volatility: High (User struct changes frequently)

### After: Improved Code

```rust
// contracts.rs (stable layer)
pub trait UserInfo {
    fn display_name(&self) -> &str;
    fn age(&self) -> u32;
    fn contact_email(&self) -> &str;
}
```

```rust
// module_b.rs (implementation details hidden)
pub struct User {
    name: String,        // changed to private
    age: u32,
    email_address: String,
}

impl UserInfo for User {
    fn display_name(&self) -> &str { &self.name }
    fn age(&self) -> u32 { self.age }
    fn contact_email(&self) -> &str { &self.email_address }
}
```

```rust
// module_a.rs (access via trait)
fn process_user(user: &impl UserInfo) {
    let name = user.display_name();    // ← Contract coupling
    let age = user.age();              // ← Contract coupling
    let email = user.contact_email();  // ← unaffected by internal changes
    // ...
}
```

**Improvements**:
- Coupling strength: Reduced to Contract (via trait)
- Changes are contained within the `User` struct
- `module_a` no longer needs to know `User`'s internal structure

## Coupling Balance Summary

| Perspective | Guideline |
|-------------|-----------|
| Strong coupling... | Keep components close, or reduce volatility |
| Far dependencies... | Use weak coupling (Contract) |
| Highly volatile components... | Isolate with stable abstraction layers |

Coupling balance is not about "eliminating coupling" but about "placing the right strength of coupling in the right place."

## Numeric Implementation

In the actual implementation:

```rust
let alignment = 1.0 - (strength - (1.0 - distance)).abs();
let volatility_impact = 1.0 - (volatility * strength);
let score = alignment * volatility_impact;
```

## CLI Options

```
cargo coupling [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to analyze [default: ./src]

Options:
  -o, --output <FILE>           Output report to file
  -s, --summary                 Show summary only
      --ai                      AI-friendly output for coding agents
      --all                     Show all issues (default: hide Low severity)
      --japanese, --jp          Japanese output with explanations (日本語)
      --git-months <MONTHS>     Git history period [default: 6]
      --no-git                  Skip Git analysis
  -c, --config <CONFIG>         Config file path (default: .coupling.toml)
  -v, --verbose                 Verbose output with explanations
      --timing                  Show timing information
  -j, --jobs <N>                Number of threads (default: auto)
      --max-deps <N>            Max outgoing dependencies [default: 20]
      --max-dependents <N>      Max incoming dependencies [default: 30]

Web Visualization:
      --web                     Start interactive web UI
      --port <PORT>             Web server port [default: 3000]
      --no-open                 Don't auto-open browser
      --api-endpoint <URL>      API endpoint URL (for separate deployments)

Job-Focused Commands:
      --hotspots[=<N>]          Show top N refactoring targets [default: 5]
      --impact <MODULE>         Analyze change impact for a module
      --trace <ITEM>            Trace dependencies for a function/type
      --check                   CI/CD quality gate (exit code 1 on failure)
      --min-grade <GRADE>       Minimum grade for --check (A/B/C/D/F)
      --max-critical <N>        Max critical issues for --check
      --max-circular <N>        Max circular dependencies for --check
      --fail-on <SEVERITY>      Fail --check on severity (critical/high/medium/low)
      --json                    Output in JSON format

  -h, --help                    Print help
  -V, --version                 Print version
```

## Thresholds

### Issue Detection Thresholds

The tool uses the following default thresholds for detecting coupling issues:

| Threshold | Default | CLI Flag | Description |
|-----------|---------|----------|-------------|
| Strong Coupling | 0.75 | - | Minimum strength value considered "strong" (Intrusive level) |
| Far Distance | 0.50 | - | Minimum distance value considered "far" (DifferentModule+) |
| High Volatility | 0.75 | - | Minimum volatility value considered "high" |
| Max Dependencies | 20 | `--max-deps` | Outgoing dependencies before flagging High Efferent Coupling |
| Max Dependents | 30 | `--max-dependents` | Incoming dependencies before flagging High Afferent Coupling |

### Health Grade Calculation

Health grades are calculated based on internal couplings only (external crate dependencies are excluded):

| Grade | Description | Criteria |
|-------|-------------|----------|
| **S (Over-optimized!)** | Stop refactoring! | Medium density <= 5% with >= 20 couplings |
| **A (Well-balanced)** | Coupling is appropriate | Medium density 5-10%, no high issues |
| **B (Healthy)** | Minor issues, manageable | Medium density > 10%, no critical issues |
| **C (Room for improvement)** | Some structural issues | Any high issues OR medium density > 25% |
| **D (Attention needed)** | Significant issues | Any critical issues OR high density > 5% |
| **F (Immediate action required)** | Critical issues | More than 3 critical issues |

**Note**: S is a WARNING, not a reward. It means you might be over-engineering. Aim for A.

### Severity Classification

Issues are classified by severity based on:

| Severity | Criteria |
|----------|----------|
| **Critical** | Multiple critical issues detected (circular dependencies, etc.) |
| **High** | Count > threshold × 2 (e.g., > 40 dependencies when threshold is 20) |
| **Medium** | Count > threshold but <= threshold × 2 |
| **Low** | Minor issues, generally informational |

## Output Example

### Summary Mode (English)

```
$ cargo coupling --summary ./src

Balanced Coupling Analysis: my-project
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Grade: B (Good) | Score: 0.67/1.00 | Modules: 14

3-Dimensional Analysis:
  Strength:   Contract 1% / Model 24% / Functional 66% / Intrusive 8%
  Distance:   Same 6% / Different 2% / External 91%
  Volatility: Low 2% / Medium 98% / High 0%

Balance State:
  ✅ High Cohesion (strong+close): 24 (6%)
  ✅ Loose Coupling (weak+far): 5 (1%)
  🤔 Acceptable (strong+far+stable): 352 (92%)

Detected Issues:
  🟡 Medium: 3

Top Priorities:
  - [Medium] metrics → 17 functions, 17 types, 11 impls
  - [Medium] main → 21 dependencies
```

### Summary Mode (Japanese)

```
$ cargo coupling --summary --jp ./src

カップリング分析: my-project
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

評価: B (Good) | スコア: 0.67/1.00 | モジュール数: 14

3次元分析:
  結合強度: Contract 1% / Model 24% / Functional 66% / Intrusive 8%
           (トレイト)   (型)      (関数)        (内部アクセス)
  距離:     同一モジュール 6% / 別モジュール 2% / 外部 91%
  変更頻度: 低 2% / 中 98% / 高 0%

バランス状態:
  ✅ 高凝集 (強い結合 + 近い距離): 24 (6%) ← 理想的
  ✅ 疎結合 (弱い結合 + 遠い距離): 5 (1%) ← 理想的
  🤔 許容可能 (強い結合 + 遠い距離 + 安定): 352 (92%)

優先的に対処すべき問題:
  - 神モジュール (責務が多すぎる) | metrics
    → モジュールを分割: metrics_core, metrics_helpers

設計判断ガイド (Khononov):
  ✅ 強い結合 + 近い距離 → 高凝集 (理想的)
  ✅ 弱い結合 + 遠い距離 → 疎結合 (理想的)
  🤔 強い結合 + 遠い距離 + 安定 → 許容可能
  ❌ 強い結合 + 遠い距離 + 頻繁に変更 → 要リファクタリング
```

### Coupling Distribution

The tool shows how couplings are distributed by Integration Strength:

```
By Integration Strength:
| Strength   | Count | %   | Description                    |
|------------|-------|-----|--------------------------------|
| Contract   | 23    | 4%  | Depends on traits/interfaces   |
| Model      | 199   | 31% | Uses data types/structs        |
| Functional | 382   | 59% | Calls specific functions       |
| Intrusive  | 46    | 7%  | Accesses internal details      |
```

## Detected Issues

### Critical Severity
- **Circular Dependencies**: Modules that depend on each other in a cycle

### High Severity
- **Global Complexity**: Strong coupling spanning long distances
- **Cascading Change Risk**: Strong coupling with frequently changing components

### Medium Severity
- **God Module**: Module with too many functions, types, or implementations
- **High Efferent Coupling**: Module depends on too many other modules
- **High Afferent Coupling**: Too many modules depend on this module
- **Inappropriate Intimacy**: Intrusive coupling across module boundaries

### Low Severity (hidden by default, use `--all` to show)
- **Public Field Exposure**: Public fields that could use getter methods
- **Primitive Obsession**: Functions with many primitive parameters (suggest newtype)

## Performance

`cargo-coupling` is optimized for large codebases with parallel AST analysis and streaming Git processing.

### Benchmark Results (Large OSS Projects)

| Project | Files | With Git | Without Git | Speed |
|---------|-------|----------|-------------|-------|
| tokio | 488 | 655ms | 234ms | 745 files/sec |
| alacritty | 83 | 298ms | 161ms | 514 files/sec |
| ripgrep | 59 | 181ms | - | 326 files/sec |
| bat | 40 | 318ms | - | 126 files/sec |

### Performance Features

1. **Parallel AST Analysis**: Uses Rayon for multi-threaded file processing
2. **Optimized Git Analysis**: Streaming processing with path filtering
3. **Configurable Thread Count**: Use `-j N` to control parallelism

```bash
# Show timing information
cargo coupling --timing ./src

# Use 4 threads
cargo coupling -j 4 ./src

# Skip Git analysis for faster results
cargo coupling --no-git ./src
```

### Git Analysis Optimization

The Git volatility analysis is optimized with:

- **Path filtering**: `-- "*.rs"` filters at Git level (reduces data transfer)
- **Diff filtering**: `--diff-filter=AMRC` skips deleted files
- **Streaming**: `BufReader` processes output without loading all into memory
- **Async spawn**: Starts processing before Git completes

These optimizations provide **5x-47x speedup** compared to naive implementation on large repositories.

## Library Usage

```rust
use cargo_coupling::{
    analyze_workspace,
    generate_report_with_thresholds,
    IssueThresholds,
    VolatilityAnalyzer
};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Analyze project with workspace support
    let mut metrics = analyze_workspace(Path::new("./src"))?;

    // Add volatility from Git history
    let mut volatility = VolatilityAnalyzer::new(6);
    if let Ok(()) = volatility.analyze(Path::new("./src")) {
        metrics.file_changes = volatility.file_changes;
        metrics.update_volatility_from_git();
    }

    // Detect circular dependencies
    let circular = metrics.circular_dependency_summary();
    if circular.total_cycles > 0 {
        println!("Found {} cycles!", circular.total_cycles);
    }

    // Generate report with custom thresholds
    let thresholds = IssueThresholds {
        max_dependencies: 20,
        max_dependents: 25,
        ..Default::default()
    };
    generate_report_with_thresholds(&metrics, &thresholds, &mut std::io::stdout())?;

    Ok(())
}
```

## Docker

Run cargo-coupling without installing Rust:

```bash
# Basic analysis
docker run --rm -v $(pwd):/workspace ghcr.io/nwiizo/cargo-coupling coupling /workspace/src

# Summary mode
docker run --rm -v $(pwd):/workspace ghcr.io/nwiizo/cargo-coupling coupling --summary /workspace/src

# Web UI (access at http://localhost:3000)
docker run --rm -p 3000:3000 -v $(pwd):/workspace ghcr.io/nwiizo/cargo-coupling coupling --web --no-open /workspace/src

# Japanese output
docker run --rm -v $(pwd):/workspace ghcr.io/nwiizo/cargo-coupling coupling --summary --jp /workspace/src
```

### Docker Compose

```bash
# Run analysis
docker compose run --rm analyze

# Start Web UI
docker compose up web
```

### Available Tags

| Tag | Description |
|-----|-------------|
| `latest` | Latest release |
| `main` | Latest main branch build |
| `vX.Y.Z` | Specific version |

## CI/CD Integration

```yaml
# .github/workflows/coupling.yml
name: Coupling Analysis

on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for volatility analysis

      - name: Install cargo-coupling
        run: cargo install cargo-coupling

      - name: Run coupling analysis
        run: cargo coupling --summary --timing ./src

      - name: Quality gate check
        run: cargo coupling --check --min-grade=C --max-circular=0 ./src

      - name: Generate report
        run: cargo coupling -o coupling-report.md ./src

      - name: Upload report
        uses: actions/upload-artifact@v4
        with:
          name: coupling-report
          path: coupling-report.md
```

### Quality Gate Options

The `--check` command provides flexible quality gate configuration:

```bash
# Fail if grade is below C
cargo coupling --check --min-grade=C ./src

# Fail if there are any circular dependencies
cargo coupling --check --max-circular=0 ./src

# Fail if there are any critical issues
cargo coupling --check --max-critical=0 ./src

# Fail on any high severity or above
cargo coupling --check --fail-on=high ./src

# Combine multiple conditions
cargo coupling --check --min-grade=B --max-circular=0 --max-critical=0 ./src
```

Exit codes:
- `0`: All checks passed
- `1`: One or more checks failed

## Best Practices

### ✅ Good: Strong Coupling at Close Distance

```rust
mod user_profile {
    pub struct User { /* ... */ }
    pub struct UserProfile { /* ... */ }

    impl User {
        pub fn get_profile(&self) -> &UserProfile { /* ... */ }
    }
}
```

### ✅ Good: Weak Coupling at Far Distance

```rust
// core/src/lib.rs
pub trait NotificationService {
    fn send(&self, message: &str) -> Result<()>;
}

// adapters/email/src/lib.rs
impl NotificationService for EmailService { /* ... */ }
```

### ❌ Bad: Strong Coupling at Far Distance

```rust
// src/api/handlers.rs
impl Handler {
    fn handle(&self) {
        // Direct dependency on internal implementation ❌
        let result = database::internal::execute_raw_sql(...);
    }
}
```

### ❌ Bad: Circular Dependencies

```rust
// module_a.rs
use crate::module_b::TypeB;  // ❌ Creates cycle

// module_b.rs
use crate::module_a::TypeA;  // ❌ Creates cycle
```

## Limitations

**This tool is a measurement aid, not an absolute authority on code quality.**

Please keep the following limitations in mind:

### What This Tool Cannot Do

- **Understand Business Context**: The tool analyzes structural patterns but cannot understand why certain couplings exist. Some "problematic" patterns may be intentional design decisions.
- **Replace Human Judgment**: Coupling metrics are heuristics. A high coupling score doesn't always mean bad code, and a low score doesn't guarantee good design.
- **Detect All Issues**: Static analysis has inherent limitations. Runtime behavior, dynamic dispatch, and macro-generated code may not be fully analyzed.
- **Provide Perfect Thresholds**: The default thresholds are calibrated for typical Rust projects but may not fit every codebase. Adjust them based on your project's needs.

### Important Considerations

- **External Dependencies Are Excluded**: The health grade only considers internal couplings. Dependencies on external crates (serde, tokio, etc.) are not penalized since you cannot control their design.
- **Git History Affects Volatility**: If Git history is unavailable or limited, volatility analysis will be incomplete.
- **Small Projects May Score Differently**: Projects with very few internal couplings (< 10) may receive a Grade B by default, as there's insufficient data for accurate assessment.

### Recommended Usage

1. **Use as a Starting Point**: The tool highlights areas worth investigating, not definitive problems.
2. **Combine with Code Review**: Human review should validate any suggested refactoring.
3. **Track Trends Over Time**: Use the tool regularly to track coupling trends rather than focusing on absolute scores.
4. **Customize Thresholds**: Adjust `--max-deps` and `--max-dependents` to match your project's architecture.

**The goal is to provide visibility into coupling patterns, empowering developers to make informed decisions.**

## References

- [Vlad Khononov - "Balancing Coupling in Software Design"](https://www.amazon.com/dp/B0FVDYKJYQ)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
