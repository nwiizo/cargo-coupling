---
name: similarity
description: Detect semantic code similarities using similarity-rs. Creates refactoring plans for duplicate code.
argument-hint: [path] [--threshold N] [--skip-test]
---

# Similarity - Code Similarity Analysis

## Execution Steps

1. Run `similarity-rs` to detect code similarities
2. Analyze detected duplicates (95%+: immediate, 85-95%: review)
3. Create refactoring plan

## Commands

```bash
# Basic scan
similarity-rs ./src

# With threshold
similarity-rs ./src --threshold 0.85

# Skip test functions
similarity-rs ./src --skip-test

# Show code in output
similarity-rs ./src --print

# Strict detection (95%+)
similarity-rs ./src --threshold 0.95 --skip-test

# Include type similarity
similarity-rs ./src --experimental-types

# CI integration
similarity-rs ./src --threshold 0.95 --skip-test --fail-on-duplicates
```

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `-t, --threshold` | Similarity threshold (0.0-1.0) | 0.85 |
| `-m, --min-lines` | Minimum lines | 3 |
| `--min-tokens` | Minimum tokens | 30 |
| `-p, --print` | Show code in output | - |
| `--skip-test` | Skip test functions | - |
| `--exclude` | Exclude directories | - |
| `--experimental-types` | Check type similarity | - |
| `--fail-on-duplicates` | Exit 1 if duplicates found | - |

## Interpretation

| Similarity | Category | Action |
|------------|----------|--------|
| 95%+ | Near-exact duplicate | Extract to common function |
| 85-95% | Structural similarity | Consider generics/traits |
| < 85% | Possible false positive | Review manually |

## Workflow with cargo-coupling

```bash
# 1. Detect similar code
similarity-rs ./src --threshold 0.85 --skip-test

# 2. Check coupling impact
cargo run -- coupling ./src

# 3. Visualize in Web UI
cargo run -- coupling --web ./src
```

## Recommended Workflow

1. **Initial scan**: `similarity-rs . --threshold 0.8 --skip-test`
2. **Detailed analysis**: `similarity-rs ./src --threshold 0.95 --print`
3. **Type check**: `similarity-rs ./src --experimental-types --threshold 0.85`
