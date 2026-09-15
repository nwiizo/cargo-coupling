---
name: analyze
description: Run cargo-coupling and interpret a Rust project's coupling report, history, or baseline diff.
argument-hint: "[path] [analysis options]"
---

# Coupling Analysis

Run the requested analysis from this repository, using `./src` when no target is
specified. Choose the output mode that answers the request; one report is usually
enough. For interpretation, AI output includes the analysis manifest:

```bash
rtk proxy cargo run -- coupling --ai ./src
```

Use [command examples](../cargo-coupling/commands.md) for history, baseline,
machine-readable output, or other requested modes. Read `coupling --help` for
options not covered there; pass CLI arguments, not slash-command labels.

Explain the reported grade and its rationale, the consequential findings, and the
analysis limits. Check relevant source and configuration before turning a signal
into a refactoring recommendation. Preserve Git history, thresholds, and subdomain
classification when comparing results; never change them to improve the grade.

Use [the balance model](../balanced-coupling/SKILL.md) when a finding needs semantic
interpretation. The [report example](output-template.md) is optional for a detailed
report. A summary request needs only the result and material limits. Analysis is
complete when the requested results and supporting evidence are explained;
implement changes when the user also requests them.
