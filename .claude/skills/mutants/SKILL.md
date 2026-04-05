---
name: mutants
description: Run mutation testing with cargo-mutants to evaluate test quality. Identifies untested code paths.
argument-hint: [--file FILE] [-F FUNCTION]
user-invocable: true
allowed-tools: Bash, Read, Grep, Glob, Edit
---

# Mutation Testing

Run cargo-mutants, analyze missed mutants, write tests for new code, verify fixes.

## Workflow

1. **Run**: Scope to changed files or specified targets
2. **Analyze**: Focus on new/changed code missed mutants (ignore pre-existing)
3. **Fix**: Add tests for missed mutants in new code
4. **Verify**: Re-run on fixed functions to confirm 0 missed

## Commands

```bash
# Scoped to specific files (recommended for incremental work)
cargo mutants --no-shuffle -j4 -f src/config.rs -f src/volatility.rs

# Scoped to specific functions
cargo mutants --no-shuffle -j4 -F "function_name"

# Full run (slow, 1000+ mutants)
cargo mutants --no-shuffle -j4 -- --lib

# Check results
cat mutants.out/missed.txt | grep "target_file.rs"
cat mutants.out/caught.txt | wc -l
```

## Options

| Option | Description |
|--------|-------------|
| `-f <FILE>` | Limit to specific source file |
| `-F <PATTERN>` | Filter by function name pattern |
| `-E <PATTERN>` | Exclude by function name |
| `-j <N>` | Parallel jobs (default: auto) |
| `--no-shuffle` | Deterministic ordering |
| `-- --lib` | Only run library tests |

## Result Interpretation

| Result | Meaning | Action |
|--------|---------|--------|
| **caught** | Test detected mutation | None (good) |
| **missed** | Test didn't detect | Add/improve test |
| **unviable** | Compile error from mutation | Ignore |
| **timeout** | Test too slow | Optimize or skip |

## Triage Strategy

- Pre-existing missed mutants in `main.rs` (CLI entry point): low priority, hard to unit test
- Output formatting functions (`report.rs`, `cli_output.rs`): low priority unless logic-heavy
- Business logic (`config.rs`, `balance.rs`, `analyzer.rs`): high priority
- New code: always fix missed mutants before release
