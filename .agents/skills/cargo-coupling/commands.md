# Command Examples

Run from the cargo-coupling checkout, replacing `./src` with the requested path.
Choose the required mode; this list is not a sequence to execute.

## Reports

```bash
rtk proxy cargo run -- coupling ./src
rtk proxy cargo run -- coupling --summary --japanese ./src
rtk proxy cargo run -- coupling --ai ./src
rtk proxy cargo run -- coupling --json ./src
rtk proxy cargo run -- coupling --all --blind-spots ./src
rtk proxy cargo run -- coupling -o report.md ./src
```

Default text output hides Low severity issues. JSON and AI output include the
full analysis manifest; `--blind-spots` expands it in text output. Choose one
output mode rather than combining modes with different report shapes.

## History and Quality Gates

```bash
rtk proxy cargo run -- coupling --history ./src
rtk proxy cargo run -- coupling --history=8 --git-months=12 --json ./src
rtk proxy cargo run -- coupling --baseline main ./src
rtk proxy cargo run -- coupling --check --min-grade=B ./src
rtk proxy cargo run -- coupling --check --baseline main --fail-on=high ./src
```

History reanalyzes Git revisions in temporary worktrees; report skipped revisions.
Baseline mode compares issues with a Git ref. With `--check --baseline`, the gate
fails only for new issues at the requested severity or higher (default: High).
Use the user's baseline and threshold; keep analysis settings comparable.

## Targeted Inspection

```bash
rtk proxy cargo run -- coupling --hotspots=10 --verbose ./src
rtk proxy cargo run -- coupling --impact MODULE ./src
rtk proxy cargo run -- coupling --trace ITEM ./src
```

Replace `MODULE` or `ITEM` with an identifier from the actual report or source.
For server launch, see [web](../web/SKILL.md).

## Configuration

This example is illustrative; preserve the target project's existing choices:

```toml
[thresholds]
max_dependencies = 15
max_dependents = 20

[analysis]
exclude_tests = true
prelude_modules = ["src/lib.rs", "src/prelude.rs"]
exclude = ["src/generated/**"]

[subdomains]
core = ["src/balance/**", "src/metrics/**"]
supporting = ["src/analyzer.rs", "src/report.rs"]
generic = ["src/config.rs", "src/web/**"]
```

`--exclude-tests` enables test exclusion. `--no-git` disables history analysis;
use it only when that reduced coverage is requested or Git data is unavailable,
and disclose the limitation. Do not use exclusions, thresholds, or subdomains to
make a before/after grade appear better.
