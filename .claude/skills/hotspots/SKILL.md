---
name: hotspots
description: Rank a Rust project's refactoring candidates using cargo-coupling hotspot results.
argument-hint: "[path] [--hotspots=N] [--verbose]"
---

# Refactoring Hotspots

Run the built-in ranking for the requested path (default `./src`). Use the
requested limit; the CLI default is five:

```bash
rtk proxy cargo run -- coupling --hotspots ./src
rtk proxy cargo run -- coupling --hotspots=10 --verbose ./src
```

Use the returned scores and issue lists. If the scoring needs explanation, inspect
[hotspots.rs](../../../src/cli_output/hotspots.rs); do not maintain a separate
formula or recompute scores from AI prose output.

Check the leading candidates in source and explain their likely change impact,
business volatility, and a concrete next action. A high score or cycle is a signal
to investigate, not automatic justification for an abstraction. Distinguish the
CLI ranking from any adjusted recommendation and give the reason for it.
Report scope and missing evidence; apply refactors when also requested.
