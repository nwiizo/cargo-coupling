# Rust Development Rules

## Before Commit

Always run:
```bash
rtk cargo fmt --all -- --check
rtk cargo clippy --locked --all-targets --all-features -- -D warnings
rtk cargo test --locked --all-features
```

## Key Source Files

| File | Purpose |
|------|---------|
| `src/analyzer.rs` | AST analysis with syn |
| `src/discovery.rs` | Source file discovery (cargo targets + module tree) |
| `src/classification.rs` | Target resolution, structural distance, integration strength |
| `src/balance/` | Balance score and issue detection |
| `src/metrics/` | Data structures, BalanceClassification |
| `src/volatility.rs` | Git history volatility analysis |
| `src/report.rs` | Report generation (EN/JA) |
| `src/cli_output.rs` | CLI output (hotspots, impact, check) |
| `src/web/` | Web visualization server |

## Structural Changes

Use [similarity](../../.agents/skills/similarity/SKILL.md) to inspect duplicate-code
candidates and [refactor](../../.agents/skills/refactor/SKILL.md) for implementation
and verification. Similar syntax, a high similarity score, or fewer reported
findings does not establish that two responsibilities belong together.

Treat newtypes, serde derives, and public fields as signals to inspect. A newtype
helps when it expresses a distinction or protects an invariant; serialization
alone does not prove a DTO boundary, and public fields can be intentional data.
Check callers before narrowing visibility or introducing an abstraction.

For output refactors, preserve localization, ordering, whitespace, and I/O error
propagation. Characterize uncovered output cases before changing the renderer;
reuse existing tests where they already establish the behavior.
