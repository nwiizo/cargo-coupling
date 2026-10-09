# Development Guide

Use [AGENTS.md](../../AGENTS.md) for the source map and required checks. Read the
current implementation before editing; the following are entrypoints, not an
exhaustive file list.

## Adding or Changing a Finding

1. Check [IssueType](../../src/balance/issue_type.rs) and
   [explain-issue](../../.agents/skills/explain-issue/SKILL.md) for the relevant detector.
2. Update detection and its regression cases, preserving expected entrypoint,
   stable-hub, and re-export behavior.
3. Keep English/Japanese descriptions, suggested actions, CLI/Web output, and
   any exhaustive matches aligned. Search references to the affected variant.
4. Verify the [signal integrity rules](../rules/grading-integrity.md), including
   behavior with and without subdomain configuration.

## CLI and Analysis Changes

- CLI options: start with `Args` and `run_coupling` in
  [main.rs](../../src/main.rs), then update README examples and affected CLI tests.
- AST usage and strength: inspect `UsageContext`, its `to_strength` mapping, and
  visitors in [analyzer.rs](../../src/analyzer.rs), together with target resolution
  in [classification.rs](../../src/classification.rs).
- Source discovery: use [discovery.rs](../../src/discovery.rs) and
  [workspace.rs](../../src/workspace.rs); preserve package boundaries and scope tests.
- Structural refactoring: use [refactor](../../.agents/skills/refactor/SKILL.md)
  and inspect similarity candidates before sharing code.

## Performance

When changing analyzer performance, run `rtk cargo bench --bench analysis_benchmark`.
Compare the same inputs, Git window, configuration, thread count, and build profile.
Record measurements from the actual run; historical timings without a reproducible
environment are not a baseline.
