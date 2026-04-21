# cargo-coupling

Rust CLI for coupling analysis based on Khononov's "Balancing Coupling in Software Design".

## Commands

```bash
# Development
cargo build --release && cargo test --all-features && cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings

# Run
cargo run -- coupling ./src          # Analyze
cargo run -- coupling --web ./src    # Web UI (:3000)

# Docker
docker build -t cargo-coupling .
docker run --rm -v $(pwd):/workspace cargo-coupling coupling /workspace/src
docker compose up web

# Release
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
# bump Cargo.toml to X.Y.Z, refresh Cargo.lock, then:
git add Cargo.toml Cargo.lock
git commit -m "chore: release vX.Y.Z"
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin main
git push origin vX.Y.Z
# → GitHub Actions auto-publishes to crates.io

# Docker Release (manual)
gh auth refresh -h github.com -s write:packages
gh auth token | docker login ghcr.io -u nwiizo --password-stdin
docker build -t ghcr.io/nwiizo/cargo-coupling:vX.Y.Z -t ghcr.io/nwiizo/cargo-coupling:latest .
docker push ghcr.io/nwiizo/cargo-coupling:vX.Y.Z
docker push ghcr.io/nwiizo/cargo-coupling:latest
```

## Key Files

| Path | Purpose |
|------|---------|
| `src/analyzer.rs` | AST analysis (syn) |
| `src/balance.rs` | Balance score calculation |
| `src/config.rs` | `.coupling.toml` loading and pattern compilation |
| `src/workspace.rs` | cargo metadata / workspace resolution |
| `src/web/` | Web visualization server |
| `Dockerfile` | distroless (58MB) |
| `Dockerfile.full` | debian-slim + Git |

## Docs & Rules

- `.claude/docs/` - Khononov framework, issue types, learnings
- `.claude/rules/` - Rust, Web UI rules
- `.claude/skills/` - analyze, release skills
- `.claude/commands/` - Slash commands

## Notes

**Edition 2024**: `if let` chains require nightly (< Rust 1.85)

**Docker**:
- cargo-chef: dependency cache for 5-10x faster builds
- distroless: non-root, minimal CVE surface
- ARG: must redeclare after FROM
- No Git → use `Dockerfile.full`
- Run command: `cargo-coupling coupling ...` (not `cargo coupling`)

**Config semantics**:
- `.coupling.toml` is searched from the analysis path upward
- `[analysis].exclude` patterns are rooted at the directory containing the config file
- When analyzing `./src`, prefer patterns like `src/generated/**`
