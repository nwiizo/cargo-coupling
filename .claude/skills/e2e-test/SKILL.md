# E2E Test - End-to-End テスト (project)

Run comprehensive E2E tests for cargo-coupling functionality.

## Test Execution

```bash
# Create test project
rm -rf /tmp/e2e-test-cargo-coupling
mkdir -p /tmp/e2e-test-cargo-coupling/src/level/enemy

# Create Cargo.toml
cat > /tmp/e2e-test-cargo-coupling/Cargo.toml << 'EOF'
[package]
name = "e2e-test-project"
version = "0.1.0"
edition = "2021"
EOF

# Create test files (lib.rs, level/mod.rs, level/projectile.rs, etc.)
# Then run tests
```

## Test Cases

### 1. Nested Module Paths (Issue #14)
```bash
cargo run -- coupling /tmp/e2e-test-cargo-coupling/src 2>&1 | grep -E "level::enemy::spawner"
# Expected: Module name shows full path, not just "spawner"
```

### 2. --exclude-tests (Issue #13)
```bash
cargo run -- coupling --exclude-tests /tmp/e2e-test-cargo-coupling/src
# Expected: Test functions excluded from counts
```

### 3. JSON Output
```bash
cargo run -- coupling --json /tmp/e2e-test-cargo-coupling/src | jq .
# Expected: Valid JSON output
```

### 4. Summary Mode
```bash
cargo run -- coupling --summary /tmp/e2e-test-cargo-coupling/src
# Expected: Compact summary output
```

## Verification Checklist

| Test | Expected | Status |
|------|----------|--------|
| Nested module paths | `level::enemy::spawner` | |
| lib.rs module name | `lib` or empty | |
| mod.rs module name | Parent directory name | |
| --exclude-tests | Test functions excluded | |
| JSON output | Valid JSON | |
| --summary | Summary only | |

## Quick Test

```bash
# Run all unit tests first
cargo test

# Then run E2E
cargo run -- coupling ./src
```
