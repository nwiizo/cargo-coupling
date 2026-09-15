---
name: refactor
description: Propose or implement Rust refactors for specific coupling findings, with behavior and analysis verification.
argument-hint: "[path] [issue type or requested change]"
---

# Refactor Coupling

Use the user's finding or a current analysis to identify the change in scope.
If analysis is needed, follow [analyze](../analyze/SKILL.md). Treat issue-type labels
as review filters, not CLI arguments.

Confirm the problem in the implementation and callers. Prefer the smallest change
that reduces shared knowledge or change impact while preserving behavior. Existing
entrypoints, re-export facades, and stable hubs may be appropriate. Use
[patterns.md](patterns.md) only for a relevant example, not as a required design.

For proposal requests, explain the problem, recommendation, and validation plan;
include before/after code or migration steps when they help assess the change.
For implementation requests, carry out the authorized change and verification
without stopping after the proposal. Resolve only material unanswered choices.

Apply the repository's Rust structural-analysis guidance, including `similarity-rs`
and `cargo-coupling`, and inspect tool findings before extracting abstractions.
Compare the affected behavior and analysis with the same settings and Git history.
Preserve input validation and error handling. Run relevant tests and required
repository checks; repeat passing checks only after a relevant change or failure.

Complete with the actual change, verification results, and remaining limitations.
Commit or push when requested; a refactor request alone does not authorize either.
