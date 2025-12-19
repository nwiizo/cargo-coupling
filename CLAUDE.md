# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`cargo-coupling` is a Rust CLI tool that analyzes coupling in Rust projects based on Vlad Khononov's "Balancing Coupling in Software Design" framework. It measures coupling across three dimensions: **Integration Strength**, **Distance**, and **Volatility**.

## Build and Development Commands

```bash
# Build
cargo build
cargo build --release

# Run tests
cargo test

# Run a single test
cargo test test_name

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Run benchmarks
cargo bench

# Run the tool (as cargo subcommand)
cargo run -- coupling ./src
cargo run -- coupling --summary ./src
cargo run -- coupling -o report.md ./src

# Run with timing info
cargo run -- coupling --summary --timing ./src

# Parallel processing control
cargo run -- coupling -j 4 ./src  # Use 4 threads
cargo run -- coupling -j 1 ./src  # Single thread (for comparison)

# Customize thresholds
cargo run -- coupling --max-deps 20 --max-dependents 25 ./src
```

## Architecture

```
src/
├── main.rs       # CLI entry point (clap-based)
├── lib.rs        # Public API exports
├── analyzer.rs   # AST analysis with syn, parallel processing with Rayon
├── balance.rs    # Balance score calculation and issue detection
├── metrics.rs    # Data structures (CouplingMetrics, ProjectMetrics, Visibility, etc.)
├── report.rs     # Markdown report generation
├── volatility.rs # Git history analysis
└── workspace.rs  # Cargo workspace support via cargo_metadata

benches/
└── analysis_benchmark.rs  # Criterion benchmarks
```

## Key Components

### Analysis Pipeline

1. **Workspace Resolution** (`workspace.rs`): Uses `cargo_metadata` crate to understand project structure, workspace members, and dependency graphs.

2. **Parallel AST Analysis** (`analyzer.rs`):
   - Parses Rust source files using `syn` crate
   - Uses Rayon for parallel file processing
   - `CouplingAnalyzer` implements `syn::visit::Visit`
   - `UsageContext` enum tracks how dependencies are used

3. **Metrics Collection** (`metrics.rs`):
   - `IntegrationStrength`: Contract < Model < Functional < Intrusive
   - `Distance`: SameFunction < SameModule < DifferentModule < DifferentCrate
   - `Volatility`: Low (0-2 changes) < Medium (3-10) < High (11+)
   - `Visibility`: Public, PubCrate, PubSuper, PubIn, Private
   - `CircularDependencySummary`: Cycle detection results

4. **Balance Calculation** (`balance.rs`): Implements the balance equation:
   ```
   BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
   ```

5. **Git Analysis** (`volatility.rs`): Analyzes `git log` for file change frequency.

6. **Report Generation** (`report.rs`): Markdown reports with refactoring recommendations.

### Integration Strength Detection (UsageContext)

| UsageContext | Maps To | Detection Method |
|--------------|---------|------------------|
| FieldAccess | Intrusive | `visit_expr_field` |
| StructConstruction | Intrusive | `visit_expr_struct` |
| InherentImplBlock | Intrusive | `visit_item_impl` |
| MethodCall | Functional | `visit_expr_method_call` |
| FunctionCall | Functional | `visit_expr_call` |
| FunctionParameter | Functional | `analyze_signature` |
| ReturnType | Functional | `analyze_signature` |
| TypeParameter | Model | `analyze_signature` |
| Import | Model | `visit_item_use` |
| TraitBound | Contract | `visit_item_impl` |

### Balance Score Logic

**Key insight: balanced coupling, not zero coupling.**

- Strong + Close = Good (cohesion)
- Weak + Far = Good (loose coupling)
- Strong + Far = Bad (global complexity)
- Strong + Volatile = Bad (cascading changes)

External crate dependencies (Distance::DifferentCrate) are excluded from issue detection.

### Detected Issue Types

| Issue Type | Severity | Condition |
|------------|----------|-----------|
| GlobalComplexity | Critical | Intrusive + DifferentCrate |
| CascadingChangeRisk | Critical | Strong + High volatility |
| InappropriateIntimacy | High | Intrusive + DifferentModule |
| HighEfferentCoupling | High | Dependencies > threshold |
| HighAfferentCoupling | High | Dependents > threshold |
| CircularDependency | High | A → B → C → A |

### Parallel Processing

File analysis uses Rayon with automatic thread pool sizing:

```rust
let analyzed_results: Vec<_> = file_paths
    .par_iter()
    .filter_map(|file_path| analyze_rust_file_full(file_path).ok())
    .collect();
```

Thread count can be controlled via `-j` option or defaults to CPU cores.

## Dependencies

- `syn` (v2.0): Rust AST parsing with `full` and `visit` features
- `rayon`: Parallel processing
- `walkdir`: Filesystem traversal
- `thiserror`: Error type derivation
- `clap`: CLI argument parsing with `derive` feature
- `cargo_metadata`: Workspace and dependency analysis
- `serde`/`serde_json`: Serialization
- `criterion`: Benchmarking (dev)

## Testing

Tests are colocated in each module using `#[cfg(test)]` blocks:

- `analyzer.rs`: AST parsing tests (`test_analyze_*`, `test_field_access_detection`)
- `balance.rs`: Score calculation (`test_balance_*`, `test_identify_*`)
- `metrics.rs`: Data structures and circular dependency detection
- `report.rs`: Output generation

## Performance

### Large OSS Project Benchmarks

| Project | Files | With Git | Without Git | Speed |
|---------|-------|----------|-------------|-------|
| tokio | 488 | 655ms | 234ms | 745 files/sec |
| alacritty | 83 | 298ms | 161ms | 514 files/sec |
| ripgrep | 59 | 181ms | - | 326 files/sec |
| bat | 40 | 318ms | - | 126 files/sec |

### Git Analysis Optimization

The `volatility.rs` module is optimized for large repositories:

```rust
// Key optimizations in analyze():
Command::new("git")
    .args([
        "log",
        "--pretty=format:",
        "--name-only",
        "--diff-filter=AMRC",  // Skip deleted files
        &format!("--since={} months ago", self.period_months),
        "--",
        "*.rs",  // Filter at Git level
    ])
    .stdout(Stdio::piped())
    .spawn()?;  // Async spawn

// Streaming with BufReader
let reader = BufReader::with_capacity(64 * 1024, stdout);
for line in reader.lines() { ... }
```

**Optimization techniques:**
1. `-- "*.rs"`: Git-level path filtering (reduces output by 90%+)
2. `--diff-filter=AMRC`: Skip deleted files
3. `BufReader::with_capacity`: 64KB buffer for efficient streaming
4. `spawn()` instead of `output()`: Start processing immediately

**Results:** 5x-47x speedup on large repositories.

## Common Development Tasks

### Adding a New Issue Type

1. Add variant to `IssueType` enum in `balance.rs`
2. Implement detection in `identify_coupling_issues()`
3. Add description in `IssueType::description()`
4. Add suggested action in `suggest_refactoring()`

### Adding a CLI Option

1. Add field to `Args` struct in `main.rs`
2. Use the value in `run()` function
3. Update README.md CLI section

### Modifying Strength Detection

1. Update `UsageContext` enum in `analyzer.rs`
2. Add/modify visitor in `CouplingAnalyzer` impl
3. Update `UsageContext::to_strength()` mapping

## False Positive Filtering

The analyzer filters out common false positives:

- `Self::Self` patterns
- Short lowercase names (likely local variables)
- Duplicate patterns like `foo::foo`
- Common local variable names (request, response, etc.)
- Primitive and std types (Option, Result, Vec, etc.)

## References

- [Vlad Khononov - "Balancing Coupling in Software Design"](https://www.amazon.com/dp/B0FVDYKJYQ)
- [syn crate documentation](https://docs.rs/syn)
- [rayon crate documentation](https://docs.rs/rayon)
