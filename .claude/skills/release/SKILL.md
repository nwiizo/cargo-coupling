---
name: release
description: Prepare or publish a cargo-coupling version when the user requests a release.
argument-hint: "[version] [prepare or publish]"
---

# Release cargo-coupling

Match the requested version and publication scope. A preparation request ends with
verified local artifacts; a publish request includes the authorized release commit,
tag, push, and publication verification. Carry existing authorization forward.
Resolve a missing version before changing release metadata; meanwhile inspect the
branch, remote, status, and current release workflow.

Update `Cargo.toml` and refresh `Cargo.lock` without upgrading unrelated dependencies.
Run the required checks against the final release state:

```bash
rtk cargo fmt --all -- --check
rtk cargo clippy --locked --all-targets --all-features -- -D warnings
rtk cargo test --locked --all-features
rtk cargo build --locked --release
```

Before publication, verify the staged diff, intended branch and remote, matching
manifest/lockfile versions, and that the tag does not already identify a different
release. Preserve unrelated work. Use an annotated `vX.Y.Z` tag on the verified
release commit; the commit message is `chore: release vX.Y.Z`. Push only the intended
branch and tag to the verified remote; never replace an existing release tag.

Pushing a `v*` tag triggers [.github/workflows/release.yml](../../../.github/workflows/release.yml),
which publishes to crates.io and creates a GitHub release. Check both outcomes
before reporting publication complete. If publication fails or is uncertain,
inspect the workflow and registry state before retrying; do not create another
version or move a tag to bypass the failure.

The optional Claude hook in [.claude/settings.json](../../settings.json) checks
formatting and lint for matching commit commands. It does not establish that all
release checks ran in another host. Reuse successful current checks while the
release state is unchanged.
