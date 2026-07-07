# Architecture Reference

## Project Structure

```
src/
├── main.rs       # CLI entry point (clap-based)
├── lib.rs        # Public API exports
├── analyzer.rs   # AST analysis with syn, parallel processing with Rayon
├── aposd.rs      # APOSD metrics (shallow modules, pass-through, cognitive load)
├── balance.rs    # Balance score calculation and issue detection
├── config.rs     # Configuration loading (.coupling.toml)
├── connascence.rs # Connascence pattern detection
├── metrics.rs    # Data structures (CouplingMetrics, ProjectMetrics, etc.)
├── report.rs     # Markdown report generation
├── temporal.rs   # Temporal coupling analysis
├── volatility.rs # Git history analysis
└── workspace.rs  # Cargo workspace support via cargo_metadata
```

## Analysis Pipeline

1. **Workspace Resolution** (`workspace.rs`): Uses `cargo_metadata` to understand project structure
2. **Parallel AST Analysis** (`analyzer.rs`): Parses Rust files with `syn`, uses Rayon for parallelism
3. **Metrics Collection** (`metrics.rs`): IntegrationStrength, Distance, Volatility, Visibility
4. **Balance Calculation** (`balance.rs`): `BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY`
5. **APOSD Analysis** (`aposd.rs`): Module depth, pass-through methods, cognitive load
6. **Report Generation** (`report.rs`): Markdown reports with refactoring recommendations

## Integration Strength Detection

| UsageContext | Maps To | Detection Method |
|--------------|---------|------------------|
| FieldAccess | Model for `pub` target types; Intrusive for crate-restricted/unknown target types | `visit_expr_field` + type registry |
| StructConstruction | Model for `pub` target types; Intrusive for crate-restricted/unknown target types | `visit_expr_struct` + type registry |
| MethodCall | Functional | `visit_expr_method_call` |
| FunctionCall | Functional | `visit_expr_call` |
| TypeParameter | Model | `analyze_signature` |
| Import | Model | `visit_item_use` |
| TraitBound | Contract | `visit_item_impl` |

## Balance Score Logic

- Strong + Close = Good (cohesion)
- Weak + Far = Good (loose coupling)
- Strong + Far = Bad (global complexity)
- Strong + Volatile = Bad (cascading changes)

## Detected Issue Types

| Issue Type | Severity | Condition |
|------------|----------|-----------|
| GlobalComplexity | Critical | Intrusive + DifferentCrate |
| CascadingChangeRisk | Critical | Strong + High volatility |
| InappropriateIntimacy | High | Intrusive + DifferentModule |
| HighEfferentCoupling | High | Dependencies > threshold |
| HighAfferentCoupling | High | Dependents > threshold |
| CircularDependency | High | A → B → C → A |

## Dependencies

- `syn` (v2.0): Rust AST parsing
- `rayon`: Parallel processing
- `walkdir`: Filesystem traversal
- `thiserror`: Error types
- `clap`: CLI parsing
- `cargo_metadata`: Workspace analysis
- `serde`/`serde_json`: Serialization
