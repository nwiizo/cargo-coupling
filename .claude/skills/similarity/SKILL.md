# Similarity - コード類似度分析 (project)

Run `similarity-rs` to detect semantic code similarities and create refactoring plans.

## Basic Commands

```bash
# Basic scan
similarity-rs ./src

# With threshold
similarity-rs ./src --threshold 0.85

# Skip test functions
similarity-rs ./src --skip-test

# Show code in output
similarity-rs ./src --print
```

## Advanced Commands

```bash
# Strict detection (95%+)
similarity-rs ./src --threshold 0.95 --skip-test

# Include type similarity
similarity-rs ./src --experimental-types

# Exclude directories
similarity-rs ./src --exclude target --exclude tests

# CI integration (fail on duplicates)
similarity-rs ./src --threshold 0.95 --skip-test --fail-on-duplicates
```

## Options Reference

| Option | Description | Default |
|--------|-------------|---------|
| `-t, --threshold` | Similarity threshold (0.0-1.0) | 0.85 |
| `-m, --min-lines` | Minimum lines | 3 |
| `--min-tokens` | Minimum tokens | 30 |
| `-p, --print` | Print code in output | - |
| `--skip-test` | Skip test functions | - |
| `--exclude` | Exclude directories | - |
| `--experimental-types` | Check type similarity | - |
| `--fail-on-duplicates` | Exit code 1 if duplicates found | - |

## Workflow with cargo-coupling

```bash
# 1. Detect similar code
similarity-rs ./src --threshold 0.85 --skip-test

# 2. Check coupling impact
cargo run -- coupling ./src

# 3. Visualize in Web UI
cargo run -- coupling --web ./src
```

## Interpretation

- **95%+**: Near-exact duplicates - extract to common function
- **85-95%**: Structural similarity - consider generics/traits
- **< 85%**: May be false positives - review manually
