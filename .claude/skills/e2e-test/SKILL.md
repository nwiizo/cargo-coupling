---
name: e2e-test
description: Run comprehensive E2E tests for cargo-coupling. Creates test projects and verifies all features.
argument-hint: [--quick|--verbose|--web]
disable-model-invocation: true
---

# E2E Test

## Execution Steps

1. Create test project at `/tmp/e2e-test-cargo-coupling`
2. Run each test scenario
3. Verify results and report

## Test Scenarios

### 1. Nested Module Paths (Issue #14)

```bash
cargo run -- coupling /tmp/e2e-test-cargo-coupling/src 2>&1 | grep -E "level::enemy::spawner"
# Expected: Full path displayed, not just "spawner"
```

### 2. Test Exclusion (Issue #13)

```bash
# Without --exclude-tests: test functions counted
cargo run -- coupling /tmp/e2e-test-cargo-coupling/src

# With --exclude-tests: test functions excluded
cargo run -- coupling --exclude-tests /tmp/e2e-test-cargo-coupling/src
```

### 3. Output Formats

```bash
cargo run -- coupling --json /tmp/e2e-test-cargo-coupling/src | jq .
cargo run -- coupling --summary /tmp/e2e-test-cargo-coupling/src
cargo run -- coupling --ai /tmp/e2e-test-cargo-coupling/src
```

### 4. Config File (.coupling.toml)

Verify settings are loaded correctly.

## Verification Checklist

| Test | Expected |
|------|----------|
| Nested module paths | `level::enemy::spawner` |
| lib.rs module name | `lib` or empty |
| mod.rs module name | Parent directory name |
| --exclude-tests | Test functions excluded |
| .coupling.toml | Settings applied |
| JSON output | Valid JSON |
| --summary | Summary only |
| --ai | AI format output |

Detailed test setup: [test-scenarios.md](test-scenarios.md)
