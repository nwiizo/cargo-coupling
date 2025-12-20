# Issue Types

## Severity Levels

| Issue | Severity | Description |
|-------|----------|-------------|
| CircularDependency | Critical | Modules depend on each other |
| GlobalComplexity | High | Too many strong external dependencies |
| CascadingChangeRisk | High | Changes likely to cascade |
| GodModule | Medium | Too many functions/types/impls |
| HighEfferentCoupling | Medium | Too many outgoing dependencies |
| HighAfferentCoupling | Medium | Too many incoming dependencies |
| InappropriateIntimacy | Medium | Internal details exposed |
| PublicFieldExposure | Low | Public fields (use getters) |
| PrimitiveObsession | Low | Too many primitive params (use newtype) |

## Priority Guidelines

- **Critical**: Must fix immediately - blocks refactoring
- **High**: Architectural problems - address in next sprint
- **Medium**: Maintenance burden - plan for improvement
- **Low**: Improvement opportunities - nice to have
