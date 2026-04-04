---
name: mutants
description: Run mutation testing with cargo-mutants to evaluate test quality. Identifies untested code paths.
argument-hint: [--file FILE] [-F FUNCTION] [--timeout SEC]
---

# Mutation Testing

## Commands

```bash
# Run all mutations
cargo mutants --timeout 60

# Specific file
cargo mutants --file src/analyzer.rs --timeout 60

# Specific function
cargo mutants --file src/analyzer.rs -F "function_name" --timeout 60

# Full analysis (longer)
cargo mutants --timeout 120

# Exclude expensive tests
cargo mutants --timeout 60 -E "integration"
```

## Options

| Option | Description |
|--------|-------------|
| `--file <FILE>` | Limit to specific file |
| `-F <PATTERN>` | Filter by function name |
| `-E <PATTERN>` | Exclude by function name |
| `--timeout <SEC>` | Timeout per mutant |
| `--jobs <N>` | Parallel jobs |

## Result Interpretation

| Result | Meaning |
|--------|---------|
| **caught** | Tests detected the mutation (good) |
| **missed** | Tests didn't detect it (needs more tests) |
| **unviable** | Mutation caused compile error (ignore) |
| **timeout** | Test took too long (may need optimization) |

## Workflow

```bash
# 1. Run mutation testing
cargo mutants --timeout 60

# 2. Check missed mutants
cat mutants.out/missed.txt

# 3. Add tests for missed mutants

# 4. Verify improvement
cargo mutants -F "specific_function" --timeout 60
```

## Notes

- Higher caught ratio = better test quality
- Focus on business-critical functions first
- `mutants.out/` is gitignored
