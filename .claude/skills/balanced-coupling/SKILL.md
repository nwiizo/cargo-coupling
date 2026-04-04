---
name: balanced-coupling
description: Khononov's Balanced Coupling model reference. Auto-loaded when analyzing coupling patterns, reviewing architecture, or interpreting analysis results.
user-invocable: false
---

# Balanced Coupling Model Reference

Based on Vlad Khononov's "Balancing Coupling in Software Design".

## The Balance Rule

```
MODULARITY = STRENGTH XOR DISTANCE
COMPLEXITY = STRENGTH AND DISTANCE
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

- **Modularity** emerges when strength and distance counterbalance
- **Complexity** emerges when both are equal (both high or both low)
- **Pragmatic**: unbalanced coupling is tolerable if volatility is low

## Three Dimensions

### 1. Integration Strength (Knowledge Shared)

From most to least intrusive:

| Level | Knowledge Type | Implicit/Explicit |
|-------|---------------|-------------------|
| **Intrusive** | Implementation details, private interfaces | Implicit, fragile |
| **Functional** | Business rules, functional specifications | Often implicit |
| **Model** | Domain/business model, data structures | Explicit but broad |
| **Contract** | Integration contracts, facades | Most explicit, stable |

Key insight: Intrusive and Functional coupling are often **implicit** — they exist
without anyone realizing. Contract coupling is **explicit** by design.

### 2. Distance (Cost of Change)

Multiple dimensions contribute to distance:
- **Code structure**: methods → objects → modules → crates → services
- **Organizational**: same team vs different teams (Conway's Law)
- **Runtime**: synchronous (tight) vs asynchronous (loose)
- **Lifecycle**: shared deployments vs independent deployments

Distance is **fractal**: the same rules apply at every abstraction level.

### 3. Volatility (Probability of Change)

Determined by **DDD subdomain classification**, not just git history:

| Subdomain | Volatility | Reason |
|-----------|------------|--------|
| **Core** | High | Competitive advantage, constantly optimized |
| **Supporting** | Low | Boring CRUD/ETL, rarely changes |
| **Generic** | Low | Solved problems, stable implementations |

Important: distinguish **essential** vs **accidental** volatility.
Accidental volatility comes from poor design, not business needs.

## Issue Classification

| Pattern | Strength | Distance | Volatility | Severity |
|---------|----------|----------|------------|----------|
| High Cohesion | Strong | Close | Any | Ideal |
| Loose Coupling | Weak | Far | Any | Ideal |
| Acceptable | Strong | Far | Low | Minor |
| Global Complexity | Strong | Far | High | Critical |
| Local Complexity | Weak | Close | Any | Review |

## Connascence Refinement

Within each strength level, connascence types provide finer granularity:

**Static** (compile-time): Name → Type → Meaning → Position → Algorithm
**Dynamic** (runtime): Execution → Timing → Values → Identity

Stronger connascence = harder to change = higher coupling cost.

## Pragmatic Balancing

- Not all unbalanced coupling needs fixing — prioritize by volatility
- Low volatility neutralizes unbalanced coupling
- Focus refactoring on core subdomains (highest business value)
- Distance increases lifecycle coupling (deployment constraints)

For detailed reference: [model-reference.md](model-reference.md)
