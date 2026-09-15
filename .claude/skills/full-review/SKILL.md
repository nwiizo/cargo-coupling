---
name: full-review
description: Perform a comprehensive Rust architecture review covering coupling, change risks, and idiomatic code.
argument-hint: "[path]"
---

# Full Architecture Review

Review the requested scope (default `./src`) and produce evidence-backed priorities.
Use [review](../review/SKILL.md) for coupling analysis, then inspect the relevant
architecture boundaries and Rust implementation. Reuse current analysis results.

Consider coupling balance, architectural change risks, and Rust idioms as review
perspectives. Apply each where it can reveal a consequential problem; named personas,
separate agents, numeric expert scores, and fixed schedules are not required.
Delegate only when the user or applicable instructions request delegation, with a
bounded subtask for each agent.

Use history or a baseline when the question concerns trends or a PR. Run tests,
lint, or structural diagnostics when needed to substantiate findings or required
by repository policy. Report which checks actually ran and any missing coverage;
do not run duplicate output modes or repeat unchanged passing checks as phases.

Consolidate findings with source locations, impact, confidence, and concrete next
actions. Separate automated signals from conclusions verified in code. Preserve
good boundaries, stable hubs, and the analysis blind spots. Use the
[report example](output-template.md) when a structured report helps, adapting it to
the request and omitting unsupported scores or empty sections.

Complete the review with supported findings and limits. If fixes are also requested,
apply and verify the agreed scope without treating the report as a mandatory pause.
