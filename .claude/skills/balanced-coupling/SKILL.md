---
name: balanced-coupling
description: Interpret cargo-coupling findings using Khononov's strength, distance, and volatility model.
user-invocable: false
---

# Balanced Coupling Model

Use the model to judge the cost of coordinated change. It is a conceptual lens;
exact detection conditions and reported severities come from the current source.

```text
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

## Qualitative Severity Table

| Pattern | Strength | Distance | Volatility | Interpretation |
|---------|----------|----------|------------|----------------|
| High Cohesion | Strong | Close | Any | Usually appropriate |
| Loose Coupling | Weak | Far | Any | Usually appropriate |
| Acceptable | Strong | Far | Low | Minor concern |
| Global Complexity | Strong | Far | High | Prioritize investigation |
| Local Complexity | Weak | Close | Any | Check whether indirection helps |

## Recognition Rules

- Where configured, subdomain volatility represents essential, business-driven
  change and governs scoring. Raw Git churn feeds accidental-volatility diagnostics;
  it must not manufacture cascading-change risk for a low-essential-volatility target.
- Binary entrypoints normally have high fan-out and co-change with wired modules.
- Crate-root re-export facades provide a stable interface; their consumers do not
  automatically depend on volatile implementation details.
- High afferent coupling to a stable central abstraction can be good design.
- Hidden Coupling is temporal co-change without an AST edge. Investigate shared
  rules or assumptions before diagnosing a defect.

Preserve these distinctions when reviewing or changing detectors. Keep history,
thresholds, and subdomain settings comparable; never manipulate them to improve
a grade. Include the analysis manifest when interpreting a clean report.

Read [model-reference.md](model-reference.md) only for dimension definitions,
subdomain reasoning, or connascence examples. For a particular detector, use
[explain-issue](../explain-issue/SKILL.md) to locate the implementation.
