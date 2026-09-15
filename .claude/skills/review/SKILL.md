---
name: review
description: Review Rust module boundaries and implicit coupling using cargo-coupling findings and source evidence.
argument-hint: "[path]"
allowed-tools: Read, Grep, Glob, Bash
---

# Modularity Review

Review the requested scope (default `./src`) for changes that would force unrelated
modules to change together. Reuse a current report or obtain one:

```bash
rtk proxy cargo run -- coupling --ai ./src
```

Read the target's `.coupling.toml` and the source behind consequential findings.
Use [the balance model](../balanced-coupling/SKILL.md) for interpretation. Missing
subdomain configuration is uncertainty to explain, not a reason to block a review
or invent business classifications.

Focus semantic inspection on knowledge shared across boundaries: duplicated rules,
shared constants or ordering assumptions, and co-changing files without an AST
edge. Separate code evidence from assumptions about teams, deployments, or runtime
behavior that static analysis cannot establish. Check the analysis manifest before
calling a report clean.

Prioritize supported findings by change impact and essential volatility. Preserve
expected entrypoint fan-out, stable hubs, and re-export facades. Keep configuration
and history comparable; grade improvement alone is not evidence of better design.

Report findings with file locations, shared knowledge, likely impact, and a
specific recommendation. Include sound design choices and relevant blind spots;
omit empty sections. Use history or baseline analysis only when trend or PR context
matters. The review is complete when its findings and limits are supported;
implementation is a separate user-requested action.
