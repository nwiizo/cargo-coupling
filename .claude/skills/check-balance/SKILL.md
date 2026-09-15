---
name: check-balance
description: Check a Rust project's coupling health summary or run a requested CI quality gate.
argument-hint: "[path] [gate options]"
---

# Check Balance

For a quick check, run from this repository with the requested path (default
`./src`) and report the grade, its rationale, and material analysis limits:

```bash
rtk proxy cargo run -- coupling --summary ./src
```

For a gate, use the user's threshold or baseline. Examples:

```bash
rtk proxy cargo run -- coupling --check --min-grade=C ./src
rtk proxy cargo run -- coupling --check --baseline main --fail-on=high ./src
```

Capture the exit status and explain why the gate passed or failed. Baseline mode
fails only on new issues at the selected severity or above; its default is High.
Do not infer a grade from the average balance score: the current implementation
uses issue density and data sufficiency. `S` is an over-optimization warning.

Inspect [grade.rs](../../../src/balance/grade.rs) or
[gate handling](../../../src/cli_output.rs) only when the result needs explanation.
A low grade alone does not require a full review or code changes. If the user asks
for diagnosis, follow [analyze](../analyze/SKILL.md) for the relevant findings.
