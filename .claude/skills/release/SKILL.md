---
name: release
description: Release workflow for cargo-coupling. Version bump, pre-release checks, tag creation, and crates.io publish.
argument-hint: [version]
disable-model-invocation: true
---

# Release Workflow

## Pre-Release Checks

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Version Bump

1. Update version in `Cargo.toml`
2. Run `cargo build` to update `Cargo.lock`
3. Commit: `chore: release vX.Y.Z`

## Tag and Push

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main
git push origin vX.Y.Z
```

GitHub Actions auto-publishes to crates.io on tag push.

## Docker Release (manual)

```bash
gh auth refresh -h github.com -s write:packages
gh auth token | docker login ghcr.io -u nwiizo --password-stdin
docker build -t ghcr.io/nwiizo/cargo-coupling:vX.Y.Z -t ghcr.io/nwiizo/cargo-coupling:latest .
docker push ghcr.io/nwiizo/cargo-coupling:vX.Y.Z
docker push ghcr.io/nwiizo/cargo-coupling:latest
```

## Notes

- Follow semantic versioning
- Pre-commit hook runs fmt + clippy automatically
