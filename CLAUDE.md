# CLAUDE.md

Rust CLI for coupling analysis based on Khononov's "Balancing Coupling in Software Design".

## Quick Commands

```bash
cargo build                    # Build
cargo test                     # Test
cargo fmt --all                # Format
cargo clippy -- -D warnings    # Lint
cargo run -- coupling ./src    # Analyze
cargo run -- coupling --web ./src  # Web UI
```

## Key Files

| Path | Purpose |
|------|---------|
| `src/analyzer.rs` | AST analysis with syn |
| `src/balance.rs` | Balance score calculation |
| `src/web/` | Web visualization server |
| `web-assets/` | Frontend (HTML/CSS/JS) |

## Configuration

```toml
# .coupling.toml
[thresholds]
max_efferent_coupling = 15
max_afferent_coupling = 20
```

## Documentation

| Topic | Location |
|-------|----------|
| Khononov Framework | `.claude/docs/khononov-framework.md` |
| Issue Types | `.claude/docs/issue-types.md` |
| Web UI Architecture | `.claude/docs/web-ui-architecture.md` |
| Design Decisions | `.claude/docs/learnings.md` |
| JTBD | `.claude/docs/jobs-to-be-done.md` |

## Rules & Skills

| Type | Location |
|------|----------|
| Rust Rules | `.claude/rules/rust.md` |
| Web UI Rules | `.claude/rules/web-ui.md` |
| Analysis Skill | `.claude/skills/analyze/SKILL.md` |
| Release Skill | `.claude/skills/release/SKILL.md` |

## Commands

See `.claude/commands/` for available slash commands.
