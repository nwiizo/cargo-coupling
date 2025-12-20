# Khononov Coupling Framework

Based on "Balancing Coupling in Software Design" by Vlad Khononov.

## 3 Coupling Dimensions

| Dimension | Description | Values |
|-----------|-------------|--------|
| **Strength** | How tightly coupled | Intrusive > Functional > Model > Contract |
| **Distance** | Module proximity | SameModule > DifferentModule > DifferentCrate |
| **Volatility** | Change frequency | High > Medium > Low (from git history) |

## Balance Formula

```
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

- Strong + Close = ✅ High Cohesion (ideal)
- Weak + Far = ✅ Loose Coupling (ideal)
- Strong + Far + Stable = 🤔 Acceptable
- Strong + Far + Volatile = ❌ Needs Refactoring

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
