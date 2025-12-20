# Rust Development Rules

## Before Commit

Always run:
```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Key Source Files

| File | Purpose |
|------|---------|
| `src/analyzer.rs` | AST analysis with syn |
| `src/balance.rs` | Balance score and issue detection |
| `src/metrics.rs` | Data structures, BalanceClassification |
| `src/volatility.rs` | Git history volatility analysis |
| `src/report.rs` | Report generation (EN/JA) |
| `src/cli_output.rs` | CLI output (hotspots, impact, check) |
| `src/web/` | Web visualization server |

## Rust-Specific Patterns

- Detect newtypes: `struct UserId(u64)` = good design
- Detect serde derives: `#[derive(Serialize, Deserialize)]` = DTO
- Flag public field exposure across modules
- Consider visibility: `pub(crate)` vs `pub` vs private
