# Full Review Skill

Comprehensive architecture review with expert perspectives.

## Execution Flow

### Phase 1: Automated Analysis

```bash
# Coupling analysis
cargo run -- coupling ./src

# Lint check
cargo clippy --all-targets --all-features -- -D warnings

# Test
cargo test --all-features
```

### Phase 2: Expert Review

Three perspectives:

1. **Balance Advisor (Vlad Khononov)**
   - 3D analysis: Strength, Distance, Volatility
   - Coupling balance evaluation

2. **Architecture Critic**
   - Architecture risk assessment
   - Technical debt estimation

3. **Rust Idiomatic Expert**
   - Rust idiom evaluation
   - Code quality review

### Phase 3: Integrated Report

- Prioritized improvement suggestions
- Good design decisions to maintain
- Action plan (immediate/weekly/long-term)

## Output Sections

1. Executive Summary
2. Automated Analysis Results
3. Expert Reviews
4. Integrated Improvement Plan
5. Next Steps

## Time Estimate

- Phase 1: 1-3 minutes
- Phase 2: 5-10 minutes
- Phase 3: 1-2 minutes
- **Total**: ~10-15 minutes
