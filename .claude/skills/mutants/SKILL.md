# Mutation Testing - ミューテーションテスト (project)

Run mutation testing using cargo-mutants to analyze test quality.

## Basic Commands

```bash
# Run all mutations
cargo mutants --timeout 60

# Run on specific file
cargo mutants --file src/analyzer.rs --timeout 60

# Run on specific function
cargo mutants --file src/analyzer.rs -F "function_name" --timeout 60
```

## Analysis Commands

```bash
# Quick check (specific functions)
cargo mutants -F "file_path_to_module_path" --timeout 60

# Full analysis (takes longer)
cargo mutants --timeout 120

# Exclude expensive tests
cargo mutants --timeout 60 -E "integration"
```

## Options Reference

| Option | Description |
|--------|-------------|
| `--file <FILE>` | Limit to specific file |
| `-F <PATTERN>` | Filter by function name |
| `-E <PATTERN>` | Exclude by function name |
| `--timeout <SEC>` | Timeout per mutant |
| `--jobs <N>` | Parallel jobs |

## Interpreting Results

| Result | Meaning |
|--------|---------|
| **caught** | Tests detected the mutation (good!) |
| **missed** | Tests didn't detect the mutation (needs more tests) |
| **unviable** | Mutation caused compile error (ignore) |
| **timeout** | Test took too long (may need optimization) |

## Workflow

```bash
# 1. Run mutation testing
cargo mutants --timeout 60

# 2. Check missed mutants
cat mutants.out/missed.txt

# 3. Add tests for missed mutants
# Edit tests...

# 4. Verify improvement
cargo mutants -F "specific_function" --timeout 60
```

## CI Integration

```bash
# Fail if too many missed mutants
cargo mutants --timeout 60 2>&1 | grep -E "^[0-9]+ mutants tested"
```

## Notes

- Higher caught ratio = better test quality
- Focus on business-critical functions first
- `mutants.out/` is gitignored
