# Analyze Output Template

```markdown
# Coupling Analysis Report

## Summary

- **Total files**: XX
- **Total modules**: XX
- **Total couplings**: XX
- **Balance score**: X.XX/1.00
- **Health grade**: [A/B/C/D/F]

## Detected Issues

### Critical (Immediate action)
[Issue list]

### High (Fix soon)
[Issue list]

### Medium (Plan to fix)
[Issue list]

## Coupling Distribution

### By Integration Strength

| Strength | Count | Ratio |
|----------|-------|-------|
| Contract | XX | XX% |
| Model | XX | XX% |
| Functional | XX | XX% |
| Intrusive | XX | XX% |

### By Distance

| Distance | Count | Ratio |
|----------|-------|-------|
| SameModule | XX | XX% |
| DifferentModule | XX | XX% |
| DifferentCrate | XX | XX% |

## Improvement Proposals

### Highest Priority
1. [Concrete action]

### Recommended
1. [Concrete action]

## Next Steps
1. [Recommended next action]
```
