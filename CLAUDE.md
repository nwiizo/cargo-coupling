# cargo-coupling

Rust CLI for coupling analysis based on Khononov's "Balancing Coupling in Software Design".

## Commands

```bash
# Development
cargo build --release && cargo test --all-features && cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings

# Run
cargo run -- coupling ./src          # Analyze
cargo run -- coupling --web ./src    # Web UI (:3000)
cargo run -- coupling --history ./src        # Coupling health over git history (time-series)
cargo run -- coupling --history=8 --json ./src  # Time-series as JSON (8 samples)
cargo run -- coupling --baseline main ./src  # Diff current issues against a git ref
cargo run -- coupling --check --baseline main ./src  # Ratchet gate: fail only on new High/Critical issues

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
| `src/history.rs` | Time-series: re-analyzes past git revisions via worktrees (`--history`) |
| `src/config.rs` | `.coupling.toml` loading and pattern compilation |
| `src/workspace.rs` | cargo metadata / workspace resolution |
| `src/web/` | Web visualization server |
| `Dockerfile` | distroless (58MB) |
| `Dockerfile.full` | debian-slim + Git |

## Docs & Rules

- `.claude/docs/` - Khononov framework, issue types, learnings
- `.claude/rules/` - Rust, Web UI rules
- `.claude/skills/` - All slash commands and references (analyze, balanced-coupling, check-balance, e2e-test, explain-issue, full-review, hotspots, mutants, refactor, release, review, similarity, web)

## Notes

**Edition 2024**:
- `if let` chains require nightly (< Rust 1.85)
- nightly toolchainが壊れた場合: `rustup update nightly` で更新してからリトライ

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

**History (`--history`)**:
- Samples up to N commits (default 12) across the `--git-months` window and re-analyzes each in a disposable `git worktree` (auto-removed)
- Per-revision analysis uses the **same methodology as the snapshot** (AST + git-churn volatility + config/subdomain overrides), so a revision's grade matches `coupling <dir>` at that commit
- Output is chronological (oldest first); supports `--json`. Revisions that fail to parse are reported under `skipped`
- Needs a git repo with history; avoid extreme `--git-months` (git's approxidate misparses values like 1200 months)

**Baseline / ratchet (`--baseline <ref>`)**:
- Compares current issues against a baseline ref using `(issue_type, source, target)` as the stable key; `--json` adds a top-level `diff` object
- With `--check --baseline <ref>`, fails only on new issues at `--fail-on` severity or higher; default ratchet threshold is High/Critical
