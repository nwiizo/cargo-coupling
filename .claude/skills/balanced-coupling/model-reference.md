# Balanced Coupling Model - Detailed Reference

Use the relevant section for conceptual interpretation. Rust patterns below are
clues to shared knowledge, not exact detector rules. Confirm classification and
exceptions in the current source before claiming a defect.

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

Use a trait when it represents a useful stable boundary; lowering strength alone
does not justify another abstraction.

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

**Accidental volatility**: Change activity beyond what the business domain needs.
High churn in supporting/generic subdomains warrants investigation; a temporary
development sprint alone does not prove a design defect.

**Accidental involatility**: Business wants change but cost is prohibitive.
Tight coupling makes the system resistant to necessary evolution.

### Subdomain Classification (DDD)

Use the target project's `.coupling.toml` to understand its classification.
The repository's [.coupling.toml](../../../.coupling.toml) is a local example with
business rationale. Where classified, essential volatility governs scoring;
raw churn informs accidental-volatility diagnostics. Missing business context is
uncertainty, not a reason to invent or change a classification.

## Temporal Coupling

Files that frequently change together in git commits indicate **implicit coupling**
that AST analysis cannot detect. The `VolatilityAnalyzer::analyze_temporal_coupling()`
method detects these co-change patterns.

High temporal coupling between files without explicit code dependencies suggests:
- Shared business logic (functional coupling)
- Shared assumptions (connascence of meaning/algorithm)
- Missing abstraction layer

When this is strong enough, `cargo-coupling` reports **Hidden Coupling**.
When high churn appears in supporting/generic subdomains, it reports **Accidental Volatility**.

## Blind-Spot Manifest

Static analysis does not observe dynamic connascence, organizational/runtime distance, duplicated logic that does not co-change, or macro/inactive-cfg paths. `--blind-spots` expands this "Not Analyzed" declaration in text output; JSON and AI output include it by default.

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

Focus on the integrations relevant to the requested change. Explain what knowledge
is shared, how far a change travels, and how likely that knowledge is to change.
Choose a refactor only when it improves that situation while preserving behavior;
strong nearby coupling and weak distant coupling can both be appropriate.
