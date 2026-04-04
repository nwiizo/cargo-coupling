# Refactoring Patterns

## Global Complexity Fix

```rust
// Before: Direct dependency on distant module
use crate::deep::nested::module::InternalType;

impl Handler {
    fn process(&self) {
        let internal = InternalType::new();
    }
}

// After: Trait abstraction
use crate::traits::Processable;

impl Handler {
    fn process(&self, processor: &impl Processable) {
        processor.process();
    }
}
```

## High Efferent Coupling Fix

```rust
// Before: Too many dependencies
use crate::a::A;
use crate::b::B;
use crate::c::C;
// ... 15+ imports

// After: Facade pattern
use crate::facade::ServiceFacade;

impl Handler {
    fn new(facade: ServiceFacade) -> Self { ... }
}
```

## Inappropriate Intimacy Fix

```rust
// Before: Direct access to other module's internals
mod other {
    pub struct Config {
        pub internal_state: Vec<String>,
    }
}

fn process(config: &other::Config) {
    config.internal_state.push("data".into());
}

// After: Encapsulated access
mod other {
    pub struct Config {
        internal_state: Vec<String>,
    }
    impl Config {
        pub fn add_data(&mut self, data: &str) {
            self.internal_state.push(data.into());
        }
    }
}
```

## Cascading Change Risk Fix

```rust
// Before: Direct dependency on volatile module
use crate::volatile_module::FrequentlyChangingType;

// After: Stable interface layer
use crate::stable_api::StableInterface;
// volatile_module implements StableInterface
```

## Report Template

```markdown
# Refactoring Proposal Report

## Target Issue
**Type**: [type] | **Count**: XX

## Refactoring Plan

### 1. [Module] improvement
- **Current state**: / **Problem**: / **Impact**:

#### Step 1: [Action]
Before: [code]
After: [code]
Reason: [why]

#### Expected Effect
- Balance score: X.XX -> X.XX
- Dependency count: XX -> XX

## Priority

| Rank | Target | Effort | Impact | ROI |
|------|--------|--------|--------|-----|

## Phased Migration
### Phase 1 (No breaking changes)
### Phase 2 (After adding tests)
### Phase 3 (Large-scale refactoring)
```
