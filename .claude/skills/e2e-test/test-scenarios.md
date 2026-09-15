# E2E Scenarios

Use this fixture when an end-to-end CLI check is needed. Run commands from the
cargo-coupling checkout. Create a unique directory and retain its path for this run:

```bash
rtk proxy mktemp -d /tmp/cargo-coupling-e2e.XXXXXX
```

Set `coupling_fixture` to that returned path in the current shell. Create the
following files under it with the normal file-editing tools. This fixture has no
Git history; the analysis manifest should disclose that limitation.

## Minimal Project

`Cargo.toml`:

```toml
[package]
name = "coupling-fixture"
version = "0.1.0"
edition = "2024"
```

`.coupling.toml`:

```toml
[analysis]
exclude_tests = false
```

`src/lib.rs`:

```rust
pub mod level;

#[cfg(test)]
mod tests {
    #[test]
    fn spawns() {
        crate::level::enemy::spawner::spawn();
    }
}
```

The remaining files:

| File | Content |
|------|---------|
| `src/level/mod.rs` | `pub mod enemy; pub mod projectile;` |
| `src/level/projectile.rs` | `pub struct Projectile;` |
| `src/level/enemy/mod.rs` | `pub mod spawner;` |
| `src/level/enemy/spawner.rs` | `use crate::level::projectile::Projectile; pub fn spawn() -> Projectile { Projectile }` |

## Module Paths and Output

```bash
rtk proxy cargo run -- coupling --json "$coupling_fixture/src" -o "$coupling_fixture/included.json"
rtk proxy jq -e '.modules | any(.name == "level::enemy::spawner")' "$coupling_fixture/included.json"
rtk proxy cargo run -- coupling --summary "$coupling_fixture/src"
```

Check the command's exit status before its assertions. JSON parsing alone does
not verify content. For this source-directory input, expect five modules with
full nested paths and `lib` for the crate-root file. Summary output should include
the reported grade, rationale, and analysis limits. Check default text or `--ai`
when those modes are in scope.

## Test Exclusion and Configuration

```bash
rtk proxy cargo run -- coupling --exclude-tests --json "$coupling_fixture/src" -o "$coupling_fixture/excluded.json"
rtk proxy jq -e '.analysis_manifest.notes | any(contains("Test code was excluded"))' "$coupling_fixture/excluded.json"
```

This checks the exclusion declaration. To verify function-count behavior, extend
the fixture so test functions put a module above the current `max_functions` limit
in [score.rs](../../../src/balance/score.rs), then compare its God Module finding
with and without exclusion. The declaration alone does not establish that count.

For config loading, add `exclude = ["src/level/projectile.rs"]` to `[analysis]` and
rerun JSON analysis. Assert that `level::projectile` is absent from `.modules`.
This specifically checks that patterns are relative to the config directory even
when the analysis target is `src`. Keep these fixture changes separate from any
before/after comparison of the real project's grade.

Report expected versus observed results and untested cases. Honor `--keep`;
otherwise remove only the unique fixture created for this run.
