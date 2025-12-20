# Release Workflow Skill

Steps for releasing a new version.

## Pre-Release Checks

```bash
# Format code
cargo fmt --all

# Lint (treat warnings as errors)
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test
```

## Version Bump

1. Update version in `Cargo.toml`
2. Update `Cargo.lock` with `cargo build`
3. Commit with message: `chore: bump version to X.Y.Z`

## Release Commit

```bash
git add -A
git commit -m "chore: bump version to X.Y.Z"
git push
```

## Automation

Pre-commit hook configured in `.claude/settings.json` runs:
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
