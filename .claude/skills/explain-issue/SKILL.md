---
name: explain-issue
description: Explain a cargo-coupling issue type or reported finding, including detection limits and possible remedies.
argument-hint: "[issue type or finding]"
---

# Explain a Coupling Issue

Explain what the requested finding means, what change it could make difficult,
and whether the available evidence supports acting on it. Use a small Rust example
when it clarifies the user's question; do not require a before/after example for
every explanation.

For names and descriptions, inspect
[issue_type.rs](../../../src/balance/issue_type.rs). Read only the relevant detector
when explaining exact conditions or severity:

- [coupling.rs](../../../src/balance/coupling.rs): strength, distance, volatility.
- [patterns.rs](../../../src/balance/patterns.rs): module-level structural patterns.
- [signals.rs](../../../src/balance/signals.rs): temporal and volatility diagnostics.
- [external_crates.rs](../../../src/balance/external_crates.rs): external dependencies.

Use current thresholds and the target project's configuration. The
[balance model](../balanced-coupling/SKILL.md) explains the conceptual tradeoffs;
its qualitative table is not a replacement for detection code.

Include relevant exceptions such as entrypoints, stable hubs, and re-export
facades. Recommend a remedy only when it addresses the demonstrated problem;
traits, facades, and module splits are options, not default fixes. No analysis run
is needed for a conceptual question that the available evidence already answers.
