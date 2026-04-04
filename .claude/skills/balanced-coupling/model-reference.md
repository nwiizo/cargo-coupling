# Balanced Coupling Model - Detailed Reference

## Integration Strength Deep Dive

### Intrusive Coupling (Strength = 1.0)

Access to implementation details that aren't part of the public API.

**Rust detection patterns**:
- Direct field access (`obj.field`)
- Struct literal construction (`Struct { field: value }`)
- Accessing `pub(crate)` or `pub(super)` items from outside scope
- Inherent impl blocks on external types

**Why dangerous**: Changes to internals cascade to all dependents without warning.

### Functional Coupling (Strength = 0.75)

Shared business requirements or functional specifications.

**Rust detection patterns**:
- Method calls (`obj.method()`)
- Function calls (`module::function()`)
- Function parameter types
- Return types

**Key insight**: Can be implicit — duplicated business logic across modules
is functional coupling even without explicit code dependencies.

### Model Coupling (Strength = 0.50)

Shared domain/business model or data structures.

**Rust detection patterns**:
- Type imports (`use crate::types::UserId`)
- Type parameters in generics
- Shared DTOs (types with `#[derive(Serialize, Deserialize)]`)

### Contract Coupling (Strength = 0.25)

Integration through stable, well-defined contracts.

**Rust detection patterns**:
- Trait bounds (`T: MyTrait`)
- Trait implementations (`impl MyTrait for MyStruct`)
- Published interfaces with semantic versioning

**Best practice**: Reduce coupling strength by introducing trait-based contracts.

## Distance Factors

### Code Structure Distance
```
SameFunction (0.0) → SameModule (0.25) → DifferentModule (0.5) → DifferentCrate (1.0)
```

### Organizational Distance
- Same developer: minimal coordination cost
- Same team: low coordination cost
- Different team: high coordination cost (Conway's Law)
- Different organization: maximum coordination cost

### Runtime Coupling
- Synchronous calls: tighter coupling (caller blocks)
- Asynchronous messages: looser coupling (temporal decoupling)
- Event-driven: loosest coupling (no direct knowledge)

### Lifecycle Coupling Trade-off
Increasing distance often increases lifecycle coupling:
- Same binary: always deployed together
- Separate services: independent deployment but coordination needed
- This creates a fundamental tension in architecture design

## Volatility Assessment

### Essential vs Accidental

**Essential volatility**: The business domain genuinely requires frequent changes.
Core subdomains have high essential volatility — that's their nature.

**Accidental volatility**: Frequent changes caused by poor design.
If you see high git churn in supporting/generic subdomains, the design may be wrong.

**Accidental involatility**: Business wants change but cost is prohibitive.
Tight coupling makes the system resistant to necessary evolution.

### Subdomain Classification (DDD)

Configure in `.coupling.toml`:
```toml
[subdomains]
core = ["src/analyzer.rs", "src/balance.rs"]
supporting = ["src/report.rs", "src/cli_output.rs"]
generic = ["src/web/*", "src/config.rs"]
```

This informs volatility assessment from business context, complementing git history.

## Temporal Coupling

Files that frequently change together in git commits indicate **implicit coupling**
that AST analysis cannot detect. The `VolatilityAnalyzer::analyze_temporal_coupling()`
method detects these co-change patterns.

High temporal coupling between files without explicit code dependencies suggests:
- Shared business logic (functional coupling)
- Shared assumptions (connascence of meaning/algorithm)
- Missing abstraction layer

## Connascence Types (Refined Strength)

### Static Connascence (within a strength level)

| Type | Example | Refactoring |
|------|---------|-------------|
| **Name** | Referencing specific function name | Rename affects all callers |
| **Type** | Sharing concrete types | Extract trait |
| **Meaning** | Magic numbers/strings shared | Extract constants |
| **Position** | Parameter ordering matters | Use named parameters/builder |
| **Algorithm** | Duplicated algorithm | Extract to shared function |

### Dynamic Connascence (runtime)

| Type | Example | Impact |
|------|---------|--------|
| **Execution** | Must call A before B | Order dependency |
| **Timing** | Must happen within time window | Race conditions |
| **Values** | Values must be consistent | Invariant violations |
| **Identity** | Must reference same instance | Shared state bugs |

## Applying the Model

1. **Map integrations**: Identify all coupling relationships
2. **Classify dimensions**: Strength, Distance, Volatility for each
3. **Apply balance rule**: Flag unbalanced + volatile couplings
4. **Prioritize by volatility**: Focus on core subdomains first
5. **Reduce strength**: Intrusive → Functional → Model → Contract
6. **Adjust distance**: Move closely coupled code closer together
