---
name: cargo-coupling
description: Explain cargo-coupling CLI options and .coupling.toml configuration. Use for command or setup questions.
---

# cargo-coupling CLI Guidance

Answer the requested command or configuration question with the smallest useful
example. Run commands from this repository; an external project is the analysis
path, not the directory from which to run this package's `cargo run`.

```bash
rtk proxy cargo run -- coupling --help
```

- Read [commands.md](commands.md) for examples of analysis modes and configuration.
- Use [analyze](../analyze/SKILL.md) when asked to run and interpret an analysis.
- Use [web](../web/SKILL.md) when asked to launch the visualization.

Check current help and [CLI definitions](../../../src/main.rs) for precise flag
behavior. For configuration precedence or pattern matching, inspect
[config.rs](../../../src/config.rs) and the repository's
[.coupling.toml](../../../.coupling.toml). Exclusion patterns are relative to the
config directory; the config search starts at the analysis path and walks upward.
Subdomains describe business-driven volatility and must not be relabeled to hide
findings.
