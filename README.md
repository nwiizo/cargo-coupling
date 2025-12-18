# cargo-coupling

[![Crates.io](https://img.shields.io/crates/v/cargo-coupling.svg)](https://crates.io/crates/cargo-coupling)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

**Measure the "right distance" in your Rust code.**

`cargo-coupling` analyzes coupling in Rust projects based on Vlad Khononov's "Balancing Coupling in Software Design" framework. It measures coupling across multiple dimensions: **Integration Strength**, **Distance**, **Volatility**, **Connascence**, and **Temporal Coupling**.

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

- **5-Dimensional Analysis**: Measures Integration Strength, Distance, Volatility, Connascence, and Temporal Coupling
- **Balance Score**: Calculates overall coupling balance (0.0 - 1.0)
- **AI-Friendly Output**: `--ai` flag generates output optimized for coding agents (Claude, Copilot, etc.)
- **Connascence Detection**: Identifies coupling types (Name, Type, Position, Algorithm)
- **Temporal Coupling Detection**: Detects execution order dependencies and Rust-specific patterns
- **Issue Detection**: Automatically identifies problematic coupling patterns
- **Circular Dependency Detection**: Detects and reports dependency cycles
- **Visibility Tracking**: Analyzes Rust visibility modifiers (pub, pub(crate), etc.)
- **Git Integration**: Analyzes change frequency from Git history
- **Configuration File**: Supports `.coupling.toml` for volatility overrides
- **Parallel Processing**: Uses Rayon for fast analysis of large codebases
- **Configurable Thresholds**: Customize dependency limits via CLI or config
- **Markdown Reports**: Generates detailed analysis reports
- **Cargo Integration**: Works as a cargo subcommand

## The Five Dimensions

### 1. Integration Strength

How much knowledge is shared between components. Detected through AST analysis:

| Level | Description | Detection Method |
|-------|-------------|------------------|
| Contract | Trait bounds and implementations | `impl Trait for Type`, trait bounds |
| Model | Type usage and imports | Type parameters, use statements |
| Functional | Function/method calls | Method calls, function calls |
| Intrusive | Direct field/internal access | Field access, struct construction |

### 2. Distance

How far apart components are in the module hierarchy.

| Level | Description | Score |
|-------|-------------|-------|
| Same Function | Within the same function | 0.00 |
| Same Module | Within the same file/module | 0.25 |
| Different Module | Across modules in same crate | 0.50 |
| Different Crate | External crate dependency | 1.00 |

### 3. Volatility

How frequently a component changes (from Git history).

| Level | Changes (6 months) | Score |
|-------|-------------------|-------|
| Low | 0-2 changes | 0.00 |
| Medium | 3-10 changes | 0.50 |
| High | 11+ changes | 1.00 |

### 4. Connascence Types

Based on Meilir Page-Jones' taxonomy, connascence measures how changes in one component require changes in another.

| Type | Strength | Description | Refactoring Suggestion |
|------|----------|-------------|------------------------|
| Name | 0.2 (weak) | Components agree on names | Use IDE rename refactoring |
| Type | 0.4 | Components agree on types | Use traits/generics |
| Meaning | 0.6 | Agreement on semantic values | Replace magic values with constants |
| Position | 0.7 | Agreement on ordering | Use builder pattern or named parameters |
| Algorithm | 0.9 (strong) | Agreement on algorithms | Extract to shared module |

### 5. Temporal Coupling

Components that must be used in a specific order. Detected through heuristic pattern analysis.

#### Paired Operations

| Operation | Description | Severity |
|-----------|-------------|----------|
| open/close | File, connection, resource handles | High |
| lock/unlock | Mutex, RwLock synchronization | Critical |
| begin/commit | Transaction boundaries | High |
| init/cleanup | Lifecycle management | Medium |
| subscribe/unsubscribe | Event handlers | Medium |

#### Rust-Specific Patterns

| Pattern | Detection | Status |
|---------|-----------|--------|
| **Drop impl** | Types with automatic cleanup | Positive (RAII) |
| **Guard patterns** | MutexGuard, RwLockGuard, RefMut | Positive (auto-release) |
| **Async spawn/join** | Orphaned tasks detection | Warning |
| **Unsafe allocations** | Manual memory management | Critical |

#### Lifecycle Phases

The analyzer tracks lifecycle methods to detect initialization order dependencies:

1. **Create**: `new`, `create`, `build`
2. **Configure**: `configure`, `with_config`
3. **Initialize**: `init`, `setup`, `prepare`
4. **Start**: `start`, `run`, `connect`
5. **Active**: `process`, `handle`
6. **Stop**: `stop`, `close`, `disconnect`
7. **Cleanup**: `cleanup`, `destroy`, `shutdown`

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
  -v, --verbose                 Verbose output
      --timing                  Show timing information
  -j, --jobs <N>                Number of threads (default: auto)
      --max-deps <N>            Max outgoing dependencies [default: 20]
      --max-dependents <N>      Max incoming dependencies [default: 30]
  -h, --help                    Print help
  -V, --version                 Print version
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

      - name: Check for critical issues
        run: |
          cargo coupling --summary ./src 2>&1 | grep -q "Critical:" && exit 1 || exit 0

      - name: Generate report
        run: cargo coupling -o coupling-report.md ./src

      - name: Upload report
        uses: actions/upload-artifact@v4
        with:
          name: coupling-report
          path: coupling-report.md
```

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

## References

- [Vlad Khononov - "Balancing Coupling in Software Design"](https://www.amazon.com/dp/B0FVDYKJYQ)
- [Meilir Page-Jones - Connascence](https://en.wikipedia.org/wiki/Connascence)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
