---
name: explain-issue
description: Explain coupling issue types in detail with detection conditions, code examples, and solution approaches.
argument-hint: [issue-type: global-complexity|cascading-change-risk|inappropriate-intimacy|high-efferent|high-afferent|unnecessary-abstraction]
---

# Explain Issue

Explain the specified issue type with overview, detection conditions, code examples, and solution approaches.

## Issue Types

### Global Complexity

- **Condition**: Strong coupling + far distance
- **Formula**: `STRENGTH >= 0.5 AND DISTANCE >= 0.5`
- **Fix**: Introduce traits, move modules, facade pattern

### Cascading Change Risk

- **Condition**: Strong coupling + high volatility
- **Formula**: `STRENGTH >= 0.5 AND VOLATILITY >= 0.5`
- **Fix**: Stable interface layer, dependency inversion

### Inappropriate Intimacy

- **Condition**: Intrusive coupling across boundaries
- **Formula**: `STRENGTH = 1.0 AND DISTANCE > 0.0`
- **Fix**: Encapsulation, `pub(crate)` usage

### High Efferent Coupling

- **Condition**: Module has too many outgoing dependencies
- **Threshold**: Default 15
- **Fix**: Module splitting, facade pattern

### High Afferent Coupling

- **Condition**: Module has too many incoming dependencies
- **Threshold**: Default 20
- **Fix**: Interface introduction, responsibility distribution

### Unnecessary Abstraction

- **Condition**: Weak coupling + near distance + low volatility
- **Formula**: `STRENGTH < 0.3 AND DISTANCE < 0.3 AND VOLATILITY < 0.3`
- **Fix**: Remove abstraction, use direct implementation

## Explanation Structure

For each issue type, include:

1. **Overview**: Concise description
2. **Why problematic**: Concrete negative impacts
3. **Detection conditions**: Formula and thresholds
4. **Code examples**: Before/After Rust code
5. **Solution approaches**: Multiple resolution methods
6. **Design principles**: SOLID, Khononov Balance, Rust idioms

## Reference

- [Balancing Coupling in Software Design](https://www.amazon.com/dp/B0FVDYKJYQ)
- Khononov Balance: `(STRENGTH XOR DISTANCE) OR NOT VOLATILITY`
