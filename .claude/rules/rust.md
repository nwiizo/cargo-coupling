---
paths:
  - "src/**/*.rs"
  - "tests/**/*.rs"
  - "benches/**/*.rs"
  - "Cargo.toml"
  - "Cargo.lock"
---

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

## Types and Ownership

- Represent unchecked evidence separately from a checked empty result. Prefer
  `Option<&T>` over a result plus a Boolean that can contradict it.
- Borrow analysis data when reading it; clone only the fields needed for an
  independently owned merged snapshot. Do not clone to silence borrow errors.
- Keep internal changes behind the existing public API and serialized shape.
  Confirm output compatibility with fixed inputs when refactoring reports.
- Propagate recoverable input errors with `Result`; keep test assertions and
  documented invariant checks distinct from production error handling.

These choices apply the [idiomatic Rust guidance](https://github.com/nwiizo/rust-best-practices/blob/5c94dc7af1f25783ad2a637a06f1fc08b65c4cf4/docs/idiomatic-rust.md)
and [type-design guidance](https://github.com/nwiizo/rust-best-practices/blob/5c94dc7af1f25783ad2a637a06f1fc08b65c4cf4/docs/type-design.md)
reviewed from the requested local checkout. Adopt recommendations where their
preconditions hold; preserve intentional data types and avoid blanket API churn.
