# Khononov Coupling Framework

Based on "Balancing Coupling in Software Design" by Vlad Khononov.

## 3 Coupling Dimensions

### 1. Integration Strength (Knowledge Shared)

| Level | Value | Knowledge Type | Implicit/Explicit |
|-------|-------|---------------|-------------------|
| **Intrusive** | 1.0 | Implementation details, private interfaces | Implicit, fragile |
| **Functional** | 0.75 | Business rules, functional specs | Often implicit |
| **Model** | 0.50 | Domain model, data structures | Explicit but broad |
| **Contract** | 0.25 | Integration contracts, traits | Most explicit, stable |

Key: Intrusive and Functional coupling are often **implicit** — they exist
without anyone realizing. Contract coupling is **explicit** by design.

### 2. Distance (Cost of Change)

| Level | Value | Scope |
|-------|-------|-------|
| **SameFunction** | 0.0 | Within same function/block |
| **SameModule** | 0.25 | Same file/module |
| **DifferentModule** | 0.5 | Different module in same crate |
| **DifferentCrate** | 1.0 | Different crate/external |

Distance is **fractal**: same rules apply at every abstraction level.
Multiple factors contribute: code structure, team boundaries, runtime coupling, deployment lifecycle.

### 3. Volatility (Probability of Change)

| Level | Value | Git Detection | DDD Subdomain |
|-------|-------|--------------|---------------|
| **Low** | 0.0 | 0-2 changes | Supporting, Generic |
| **Medium** | 0.5 | 3-10 changes | — |
| **High** | 1.0 | 11+ changes | Core |

Volatility should be assessed from **business domain**, not just git history.
Configure in `.coupling.toml` via `[subdomains]` section.

Distinguish **essential** (business-driven) vs **accidental** (poor design) volatility.

## Balance Formula

```
MODULARITY = STRENGTH XOR DISTANCE
COMPLEXITY = STRENGTH AND DISTANCE
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

- Strong + Close = High Cohesion (ideal)
- Weak + Far = Loose Coupling (ideal)
- Strong + Far + Stable = Acceptable (low volatility neutralizes)
- Strong + Far + Volatile = Needs Refactoring (critical)
- Weak + Close = Local Complexity (review)

## 5 Balance Classifications

| Classification | Condition | Action |
|---------------|-----------|--------|
| HighCohesion | strong + close | Ideal - keep as is |
| LooseCoupling | weak + far | Ideal - keep as is |
| Acceptable | strong + far + stable | Monitor for changes |
| Pain | strong + far + volatile | Fix now - refactor |
| LocalComplexity | weak + close | Consider separating |

## Grade System

| Grade | Description |
|-------|-------------|
| **S** | Over-optimized! Stop refactoring! (WARNING) |
| **A** | Well-balanced - coupling is appropriate (TARGET) |
| **B** | Healthy - minor issues, manageable |
| **C** | Room for improvement - some structural issues |
| **D** | Attention needed - significant issues |
| **F** | Immediate action required - critical issues |

**Note**: S grade is a WARNING, not a reward. Aim for A.

## Connascence Refinement

Within each strength level, connascence types provide finer granularity:

- **Static**: Name → Type → Meaning → Position → Algorithm
- **Dynamic**: Execution → Timing → Values → Identity

Stronger connascence = harder to change = higher coupling cost.

## Temporal Coupling

Files that frequently co-change in git commits indicate implicit coupling
beyond what AST analysis detects. Detected via `analyze_temporal_coupling()`.

## Pragmatic Balancing

- Low volatility neutralizes unbalanced coupling
- Focus refactoring on core subdomains (highest business value)
- Not all coupling is bad — cohesion requires strong coupling
- Distance increases lifecycle coupling (deployment trade-off)
