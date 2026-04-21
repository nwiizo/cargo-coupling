# Release Workflow Skill

Steps for releasing a new version.

## Pre-Release Checks

```bash
# Check formatting
cargo fmt --all -- --check

# Lint (treat warnings as errors)
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Verify release build
cargo build --release
```

## Version Bump

1. Update version in `Cargo.toml`
2. Refresh `Cargo.lock` with `cargo build --release`
3. Commit with message: `chore: release vX.Y.Z`

## Release Commit

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: release vX.Y.Z"
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin main
git push origin vX.Y.Z
```

## Automation

Pre-commit hook configured in `.claude/settings.json` runs:
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
