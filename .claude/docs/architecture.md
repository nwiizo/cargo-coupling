# Architecture Reference

## Project Structure

```
src/
├── main.rs            # CLI entry point (clap-based)
├── lib.rs             # Public API exports (crate facade)
├── analyzer.rs        # AST analysis with syn, parallel processing with Rayon
├── discovery.rs       # Source discovery: cargo targets + module tree (#[path])
├── classification.rs  # Target resolution (incl. re-exports), structural distance, strength
├── balance/           # Balance score, issue detection, grading, rationale
├── metrics/           # Data structures (CouplingMetrics, ProjectMetrics, dimensions)
├── config.rs          # Configuration loading (.coupling.toml), drift detection
├── manifest.rs        # Blind-spot manifest (declared negative space)
├── volatility.rs      # Git history analysis (volatility + temporal coupling)
├── history.rs         # Time-series analysis over git revisions (--history)
├── diff.rs            # Baseline diff / ratchet gating (--baseline)
├── external.rs        # External dependency usage aggregation (--deps)
├── report.rs          # Markdown report generation (EN/JA)
├── cli_output.rs      # CLI output (hotspots, impact, check)
├── web/               # Web visualization server
└── workspace.rs       # Cargo workspace support via cargo_metadata
```

## Analysis Pipeline

1. **Workspace Resolution** (`workspace.rs`): `cargo_metadata`, target-driven source roots
2. **Source Discovery** (`discovery.rs`): directory walk ∪ module-tree resolution
3. **Parallel AST Analysis** (`analyzer.rs`): `syn` + Rayon
4. **Classification** (`classification.rs`): target module, structural distance, strength
5. **Volatility** (`volatility.rs` + `.coupling.toml` subdomains): essential over accidental
6. **Balance & Issues** (`balance/`): `BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY`
7. **Reporting** (`report.rs`, `cli_output.rs`, `web/`, `manifest.rs`)

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

## Detected Issue Types (severity model)

| Issue Type | Severity | Condition |
|------------|----------|-----------|
| CircularDependency | Critical | A → B → C → A |
| CascadingChangeRisk | High | Intrusive + DifferentModule + High volatility (Strong+Far+High quadrant) |
| GlobalComplexity | Low–Medium | Intrusive + DifferentModule, low/medium volatility (Low = Acceptable) |
| InappropriateIntimacy | Medium | Intrusive + DifferentModule + poor balance score |
| HiddenCoupling | Medium (High needs ≥8 co-changes at ≥80%) | Strong temporal co-change, no code edge; adjacent pairs/entrypoint/facade exempt |
| HighEfferentCoupling / HighAfferentCoupling | Medium (severity scales with hub volatility) | Count > threshold |
| GodModule | Medium (High at >2× threshold) | Functions/types/impls > threshold |
| AccidentalVolatility | Diagnostic (reported, not graded) | Supporting/generic subdomain with top-quartile churn |

External-crate couplings (`DifferentCrate`) are excluded from issue detection.

## Dependencies

- `syn` (v2.0): Rust AST parsing
- `rayon`: Parallel processing
- `walkdir`: Filesystem traversal
- `thiserror`: Error types
- `clap`: CLI parsing
- `cargo_metadata`: Workspace analysis
- `serde`/`serde_json`: Serialization
