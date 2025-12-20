# cargo-coupling

[![Crates.io](https://img.shields.io/crates/v/cargo-coupling.svg)](https://crates.io/crates/cargo-coupling)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

**Measure the "right distance" in your Rust code.**

`cargo-coupling` analyzes coupling in Rust projects based on Vlad Khononov's "Balancing Coupling in Software Design" framework. It calculates a **Balance Score** from three core dimensions: **Integration Strength**, **Distance**, and **Volatility**. Additional insights are provided through Connascence and Temporal Coupling analysis.

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

### 2. Analyze

```bash
# Analyze current project
cargo coupling ./src

# Show summary only
cargo coupling --summary ./src
```

### 3. Refactor with AI

```bash
# Generate AI-friendly output
cargo coupling --ai ./src
```

Copy the output and use this prompt with Claude, Copilot, or any AI coding assistant:

```
Analyze the coupling issues above from `cargo coupling --ai`.
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

### 4. Interactive Web Visualization

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
- **Interactive Web UI**: `--web` flag starts a browser-based visualization with graph, hotspots, and blast radius analysis
- **Job-Focused CLI**: Quick commands for common tasks (`--hotspots`, `--impact`, `--check`, `--json`)
- **Beginner-Friendly**: `--verbose` flag explains issues in plain language with fix examples
- **CI/CD Quality Gate**: `--check` command with configurable thresholds and exit codes
- **AI-Friendly Output**: `--ai` flag generates output optimized for coding agents (Claude, Copilot, etc.)
- **Issue Detection**: Automatically identifies problematic coupling patterns
- **Circular Dependency Detection**: Detects and reports dependency cycles
- **Visibility Tracking**: Analyzes Rust visibility modifiers (pub, pub(crate), etc.)
- **Git Integration**: Analyzes change frequency from Git history for volatility scoring
- **Additional Insights** (informational): Connascence types, Temporal Coupling patterns, APOSD metrics
- **Configuration File**: Supports `.coupling.toml` for volatility overrides
- **Parallel Processing**: Uses Rayon for fast analysis of large codebases
- **Configurable Thresholds**: Customize dependency limits via CLI or config
- **Markdown Reports**: Generates detailed analysis reports
- **Cargo Integration**: Works as a cargo subcommand

## The Three Dimensions of Coupling

Based on Vlad Khononov's framework, coupling is measured across three dimensions. The key insight is that **the goal is not to eliminate coupling, but to balance it appropriately**.

### 1. Integration Strength

The **amount of knowledge shared** between components. Four levels from weakest to strongest:

| Level | Description | Detection Method | Score |
|-------|-------------|------------------|-------|
| **Contract** | Interface/trait abstraction | `impl Trait for Type`, trait bounds | 0.25 (weak) |
| **Model** | Shared business domain models | Type parameters, use statements | 0.50 |
| **Functional** | Shared responsibilities | Method calls, function calls | 0.75 |
| **Intrusive** | Internal implementation details | Field access, struct construction | 1.00 (strong) |

Lower scores (Contract, Model) are generally preferable as they provide looser coupling.

### 2. Distance

**How far apart** the dependent components are:

| Level | Description | Score |
|-------|-------------|-------|
| Same Function | Within the same function | 0.00 (close) |
| Same Module | Within the same file/module | 0.25 |
| Different Module | Across modules in same crate | 0.50 |
| Different Crate | External crate dependency | 1.00 (far) |

Greater distance means higher cost of change - changes must propagate across more boundaries.

### 3. Volatility

**How frequently** a component changes (from Git history):

| Level | Description | Score |
|-------|-------------|-------|
| **Low** (Generic Subdomain) | Stable, rarely changes (0-2 changes/6mo) | 0.00 |
| **Medium** (Supporting Subdomain) | Moderate changes (3-10 changes/6mo) | 0.50 |
| **High** (Core Subdomain) | Frequently changing (11+ changes/6mo) | 1.00 |

High volatility components require more careful coupling management.

> **Note**: Volatility requires Git history. Use `cargo coupling ./src` (not `--no-git`) to enable volatility analysis.

## Balance Equation

The balance score is conceptually expressed as:

```
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

### Understanding the Formula

**MODULARITY = STRENGTH XOR DISTANCE**

- **Strong coupling + Close distance = Good** (locality preserved)
- **Weak coupling + Far distance = Good** (loose coupling)
- Strong coupling + Far distance = Bad (global complexity)
- Weak coupling + Close distance = Suboptimal (unnecessary indirection)

**BALANCE = MODULARITY OR NOT VOLATILITY**

- Either modular (good strength/distance combo), OR
- Low volatility (stable, so coupling is less risky)
- Meeting either condition achieves balance

### Numeric Implementation

In the actual implementation:

- `XOR` → Calculated as alignment between extremes (strong×close, weak×far)
- `OR` → Calculated as max value
- `NOT` → Calculated as complement (1.0 - x)

```rust
let alignment = 1.0 - (strength - (1.0 - distance)).abs();
let volatility_impact = 1.0 - (volatility * strength);
let score = alignment * volatility_impact;
```

## Connascence (Additional Insight)

> **Note**: Connascence is detected and reported for additional insight, but does **not** affect the Balance Score or Health Grade.

Based on Meilir Page-Jones' taxonomy, connascence measures how changes in one component require changes in another.

### Static Connascence (Compile-time detectable)

| Type | Strength | Description | Refactoring Suggestion |
|------|----------|-------------|------------------------|
| **Name (CoN)** | 0.2 (weakest) | Dependency on names | Use IDE rename refactoring |
| **Type (CoT)** | 0.4 | Dependency on types | Use traits/generics |
| **Meaning (CoM)** | 0.6 | Dependency on value semantics | Replace magic values with constants |
| **Position (CoP)** | 0.7 | Dependency on parameter order | Use builder pattern or named parameters |
| **Algorithm (CoA)** | 0.9 (strongest) | Dependency on algorithms | Extract to shared module |

### Dynamic Connascence (Runtime detectable)

| Type | Description |
|------|-------------|
| **Execution (CoE)** | Dependency on execution order |
| **Timing (CoT)** | Dependency on timing |
| **Value (CoV)** | Synchronous value changes required |
| **Identity (CoI)** | Dependency on same instance |

> **Important**: Even the weakest dynamic connascence is stronger than the strongest static connascence. Dynamic coupling is harder to detect and manage.

## Temporal Coupling (Additional Insight)

> **Note**: Temporal Coupling is detected and reported for additional insight, but does **not** affect the Balance Score or Health Grade.

Components that must be used in a specific order.

### Paired Operations

| Operation | Description | Severity |
|-----------|-------------|----------|
| open/close | File, connection, resource handles | High |
| lock/unlock | Mutex, RwLock synchronization | Critical |
| begin/commit | Transaction boundaries | High |
| init/cleanup | Lifecycle management | Medium |
| subscribe/unsubscribe | Event handlers | Medium |

### Rust-Specific Patterns

| Pattern | Detection | Status |
|---------|-----------|--------|
| **Drop impl** | Types with automatic cleanup | Positive (RAII) |
| **Guard patterns** | MutexGuard, RwLockGuard, RefMut | Positive (auto-release) |
| **Async spawn/join** | Orphaned tasks detection | Warning |
| **Unsafe allocations** | Manual memory management | Critical |

### Lifecycle Phases

The analyzer tracks lifecycle methods to detect initialization order dependencies:

1. **Create**: `new`, `create`, `build`
2. **Configure**: `configure`, `with_config`
3. **Initialize**: `init`, `setup`, `prepare`
4. **Start**: `start`, `run`, `connect`
5. **Active**: `process`, `handle`
6. **Stop**: `stop`, `close`, `disconnect`
7. **Cleanup**: `cleanup`, `destroy`, `shutdown`

## APOSD Metrics

> **Note**: APOSD metrics are **informational only** and do not affect the Health Grade calculation. The grade is determined solely by traditional coupling metrics (Integration Strength, Distance, Volatility).

Based on John Ousterhout's ["A Philosophy of Software Design"](https://web.stanford.edu/~ouster/cgi-bin/book.php) (2nd Edition), cargo-coupling detects the following design anti-patterns:

### Module Depth

Measures whether a module provides a simple interface that hides complex implementation.

| Classification | Depth Ratio | Description |
|----------------|-------------|-------------|
| Very Deep | >= 10.0 | Excellent abstraction (like Unix I/O) |
| Deep | >= 5.0 | Good hiding of complexity |
| Moderate | >= 2.0 | Acceptable design |
| Shallow | >= 1.0 | Interface nearly as complex as implementation ⚠️ |
| Very Shallow | < 1.0 | Interface MORE complex than implementation ⚠️ |

**Depth Ratio** = Implementation Complexity / Interface Complexity

### Pass-Through Methods

Detects methods that simply delegate to another method without adding significant functionality:

```rust
// ❌ Pass-through method (Red Flag)
impl Service {
    pub fn process(&self, data: Data) -> Result<Output> {
        self.inner.process(data)  // Just delegation
    }
}

// ✅ Deep method (Good)
impl Service {
    pub fn process(&self, data: Data) -> Result<Output> {
        let validated = self.validate(data)?;
        let transformed = self.transform(validated);
        self.inner.process(transformed)
    }
}
```

### Cognitive Load

Estimates how much a developer needs to know to work with a module:

| Level | Score | Description |
|-------|-------|-------------|
| Low | < 5.0 | Easy to understand |
| Moderate | 5.0 - 15.0 | Manageable complexity |
| High | 15.0 - 30.0 | Requires significant effort ⚠️ |
| Very High | > 30.0 | Overwhelming complexity ⚠️ |

Factors considered:
- Number of public APIs
- Number of dependencies
- Average parameter count
- Generic type parameters
- Trait bounds
- Control flow complexity

### APOSD and Rust Compatibility

APOSD concepts generally align well with Rust. This tool is **Rust-optimized** and automatically excludes idiomatic Rust patterns from detection.

**Good Compatibility:**
- Rust's visibility system (`pub`, `pub(crate)`, private) naturally supports information hiding
- Traits enable deep abstractions with simple interfaces
- RAII (Drop trait) reduces temporal coupling automatically

**Excluded from Pass-Through Detection (Rust Idioms):**

The following patterns are automatically excluded because they are intentional Rust idioms:

| Category | Patterns |
|----------|----------|
| **Conversion Methods** | `as_*`, `into_*`, `from_*`, `to_*` |
| **Accessor Methods** | `get_*`, `set_*`, `*_ref`, `*_mut` |
| **Trait Implementations** | `deref`, `deref_mut`, `as_ref`, `as_mut`, `borrow`, `clone`, `default`, `eq`, `cmp`, `hash`, `fmt`, `drop`, `index` |
| **Builder Pattern** | `with_*`, `and_*` |
| **Iterator Methods** | `iter`, `iter_mut`, `into_iter` |
| **Simple Accessors** | `len`, `is_empty`, `capacity`, `inner`, `get`, `new` |
| **Error Propagation** | Methods using `?` operator |

**Example - Not Flagged:**
```rust
// These are Rust idioms, NOT design issues:

impl MyType {
    pub fn as_str(&self) -> &str { &self.inner }     // Conversion
    pub fn into_inner(self) -> Inner { self.inner }  // Ownership transfer
    pub fn len(&self) -> usize { self.data.len() }   // Simple accessor
}

impl Deref for MyType {
    fn deref(&self) -> &Self::Target { &self.inner } // Trait impl
}

fn process(&self) -> Result<T> {
    self.inner.process()?  // Error propagation with `?`
}
```

**Flagged as Potential Issues:**
```rust
// These MAY indicate design issues:

impl Service {
    // Just delegates without adding value - consider if needed
    pub fn execute(&self, cmd: Command) -> Output {
        self.executor.execute(cmd)
    }
}
```

## Balance Equation

```
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

**Well-balanced patterns:**
- Strong coupling + Close distance = Good (locality)
- Weak coupling + Far distance = Good (loose coupling)

**Problematic patterns:**
- Strong coupling + Far distance = Bad (global complexity)
- Strong coupling + High volatility = Bad (cascading changes)

## CLI Options

```
cargo coupling [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to analyze [default: ./src]

Options:
  -o, --output <FILE>           Output report to file
  -s, --summary                 Show summary only
      --ai                      AI-friendly output for coding agents
      --git-months <MONTHS>     Git history period [default: 6]
      --no-git                  Skip Git analysis
  -v, --verbose                 Verbose output with explanations
      --timing                  Show timing information
  -j, --jobs <N>                Number of threads (default: auto)
      --max-deps <N>            Max outgoing dependencies [default: 20]
      --max-dependents <N>      Max incoming dependencies [default: 30]

Web Visualization:
      --web                     Start interactive web UI
      --port <PORT>             Web server port [default: 3000]
      --no-open                 Don't auto-open browser

Job-Focused Commands:
      --hotspots[=<N>]          Show top N refactoring targets [default: 5]
      --impact <MODULE>         Analyze change impact for a module
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

| Grade | Criteria |
|-------|----------|
| **A (Excellent)** | No high issues, medium density <= 5%, and >= 10 internal couplings |
| **B (Good)** | Medium density > 5% or total issue density > 10%, but no critical issues |
| **C (Acceptable)** | Any high issues OR medium density > 25% |
| **D (Needs Improvement)** | Any critical issues OR high density > 5% |
| **F (Critical Issues)** | More than 3 critical issues |

### Severity Classification

Issues are classified by severity based on:

| Severity | Criteria |
|----------|----------|
| **Critical** | Multiple critical issues detected (circular dependencies, etc.) |
| **High** | Count > threshold × 2 (e.g., > 40 dependencies when threshold is 20) |
| **Medium** | Count > threshold but <= threshold × 2 |
| **Low** | Minor issues, generally informational |

### APOSD Configuration

Configure APOSD metrics detection in `.coupling.toml`:

```toml
[aposd]
# Minimum depth ratio to consider a module "deep" (default: 2.0)
min_depth_ratio = 2.0

# Maximum cognitive load score before flagging (default: 15.0)
max_cognitive_load = 15.0

# Enable/disable automatic exclusion of Rust idioms (default: true)
exclude_rust_idioms = true

# Additional method prefixes to exclude from pass-through detection
exclude_prefixes = ["my_custom_", "legacy_"]

# Additional specific method names to exclude
exclude_methods = ["special_delegate", "wrapper_call"]
```

**Configuration Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `min_depth_ratio` | 2.0 | Modules with depth ratio below this are flagged as "shallow" |
| `max_cognitive_load` | 15.0 | Modules with load score above this are flagged as "high load" |
| `exclude_rust_idioms` | true | Auto-exclude Rust patterns (`as_*`, `into_*`, `deref`, etc.) |
| `exclude_prefixes` | [] | Custom prefixes to exclude from pass-through detection |
| `exclude_methods` | [] | Custom method names to exclude from pass-through detection |

**Example - Disabling Rust Idiom Exclusion:**
```toml
[aposd]
# Detect ALL pass-through methods, including Rust idioms
exclude_rust_idioms = false
```

## Output Example

### Summary Mode

```
$ cargo coupling --summary --timing ./src

Analyzing project at './src'...
Analysis complete: 65 files, 38 modules (took 200.00ms)

Coupling Analysis Summary:
  Health Grade: C (Fair)
  Files: 65
  Modules: 38
  Couplings: 650
  Balance Score: 0.55

  Issues:
    High: 12 (should fix)
    Medium: 34

  Breakdown:
    Internal: 104
    External: 546
    Balanced: 207
    Needs Review: 144
    Needs Refactoring: 299

Total time: 205.32ms (316.7 files/sec)
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

### 1. Global Complexity (Critical)
Strong coupling spanning long distances.
```
Issue: Strong coupling over long distance increases global complexity
Action: Move components closer or reduce coupling strength
```

### 2. Cascading Change Risk (Critical)
Strong coupling with frequently changing components.
```
Issue: Strongly coupled to a frequently changing component
Action: Isolate the volatile component behind a stable interface
```

### 3. High Efferent Coupling (High)
Module depends on too many other modules.
```
Issue: Module has 25 outgoing dependencies (threshold: 15)
Action: Split module or introduce facade
```

### 4. High Afferent Coupling (High)
Too many modules depend on this module.
```
Issue: Module has 30 incoming dependencies (threshold: 20)
Action: Extract stable interface or split responsibilities
```

### 5. Inappropriate Intimacy (High)
Intrusive coupling across module boundaries.
```
Issue: Direct access to internal details of another module
Action: Use public API or extract interface
```

### 6. Circular Dependencies
Modules that depend on each other.
```
⚠️ Circular Dependencies: 2 cycles (5 modules)
1. module_a → module_b → module_c → module_a
```

### 7. Temporal Coupling Issues
Execution order dependencies detected.
```
Issue: More open() calls (5) than close() calls (3)
Action: Ensure every open() has a matching close(). Consider RAII pattern.

Issue: Async spawn without join detected
Action: Ensure spawned tasks are awaited or JoinHandles collected.
```

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

### ✅ Good: RAII for Temporal Coupling

```rust
// Use Drop trait for automatic cleanup
struct Connection { /* ... */ }

impl Drop for Connection {
    fn drop(&mut self) {
        self.close();  // Automatic cleanup
    }
}

// Use guards for lock management
fn process_data(mutex: &Mutex<Data>) {
    let guard = mutex.lock().unwrap();  // Auto-unlocks on drop
    // ... use guard ...
}  // Automatically unlocked here
```

### ❌ Bad: Manual Temporal Coupling

```rust
// Requires remembering to call close()
let conn = Connection::open()?;
process(&conn);
conn.close();  // Easy to forget!

// Manual lock management
mutex.lock();
// ... if panic here, lock is never released!
mutex.unlock();
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
- **Heuristic-Based Detection**: Temporal coupling and connascence detection use pattern matching heuristics, which may produce false positives or miss some patterns.

### Recommended Usage

1. **Use as a Starting Point**: The tool highlights areas worth investigating, not definitive problems.
2. **Combine with Code Review**: Human review should validate any suggested refactoring.
3. **Track Trends Over Time**: Use the tool regularly to track coupling trends rather than focusing on absolute scores.
4. **Customize Thresholds**: Adjust `--max-deps` and `--max-dependents` to match your project's architecture.

**The goal is to provide visibility into coupling patterns, empowering developers to make informed decisions.**

## References

- [Vlad Khononov - "Balancing Coupling in Software Design"](https://www.amazon.com/dp/B0FVDYKJYQ)
- [John Ousterhout - "A Philosophy of Software Design" (2nd Edition)](https://web.stanford.edu/~ouster/cgi-bin/book.php)
- [Meilir Page-Jones - Connascence](https://en.wikipedia.org/wiki/Connascence)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
