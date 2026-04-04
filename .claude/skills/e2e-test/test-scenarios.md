# E2E Test Scenarios - Detailed Setup

## Test Project Structure

```
/tmp/e2e-test-cargo-coupling/
├── Cargo.toml
├── .coupling.toml
└── src/
    ├── lib.rs
    ├── level/
    │   ├── mod.rs
    │   ├── projectile.rs
    │   └── enemy/
    │       ├── mod.rs
    │       └── spawner.rs
```

## Setup

```bash
# 1. Create test project
rm -rf /tmp/e2e-test-cargo-coupling
mkdir -p /tmp/e2e-test-cargo-coupling/src/level/enemy

# 2. Cargo.toml
cat > /tmp/e2e-test-cargo-coupling/Cargo.toml << 'EOF'
[package]
name = "e2e-test-project"
version = "0.1.0"
edition = "2021"
EOF

# 3. Create source files
# lib.rs, level/mod.rs, level/projectile.rs,
# level/enemy/mod.rs, level/enemy/spawner.rs

# 4. Run analysis
cargo run -- coupling /tmp/e2e-test-cargo-coupling/src

# 5. Verify results
```

## Config File Test

```toml
# .coupling.toml
[analysis]
exclude_tests = true
prelude_modules = ["prelude", "ext"]
exclude = ["generated/*"]
```

## Test File Example (for test exclusion)

```rust
pub fn production_code() {}

#[test]
fn test_something() {}

#[cfg(test)]
mod tests {
    fn helper() {}
}
```

## Result Report Format

```markdown
# E2E Test Results

## Summary
- **Total tests**: X
- **Passed**: X
- **Failed**: X

## Details

### Passed
1. [Test name]: [Details]

### Failed
1. [Test name]: [Expected] vs [Actual]

## Recommended Actions
[Fix suggestions for failures]
```
