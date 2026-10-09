---
name: similarity
description: Inspect Rust duplicate-code candidates with similarity-rs. Use to assess whether similar code shares a responsibility before extracting an abstraction.
compatibility: Requires similarity-rs and rtk; coupling verification uses Rust/Cargo from this checkout.
argument-hint: "[path] [--threshold N] [--skip-test]"
---

# Rust Code Similarity

Scan the requested scope (default `./src`) with `similarity-rs`. Start with one scan
and inspect the reported code; thresholds rank candidates, not required actions:

```bash
rtk proxy similarity-rs ./src --skip-test --threshold 0.90 --min-lines 10
```

Honor requested options. Use `--print` for source detail or `--experimental-types`
when comparing types is relevant. Consult `similarity-rs --help` for other options;
do not run several threshold sweeps without an unresolved question.

Record the analyzed scope, tool options, and candidate locations. Check callers,
semantics, and intentional differences. Recommend shared code only when the same
rule should change together and sharing it reduces maintenance cost. Similar
AST traversal, translations, or collection pipelines can serve different rules;
do not merge them based on syntax alone. Explain why consequential candidates
are selected or retained. A high similarity percentage does not justify generics,
traits, or extraction.

When assessing a structural change, use cargo-coupling on the relevant scope to
check its effect on boundaries. Launch visualization only when requested or needed
to answer the task. Report locations, the shared responsibility (if any), and the
recommendation, including cases where duplication should remain. Implement fixes
when requested, preserve affected behavior with existing or focused regression
tests, and rerun similarity and coupling with the same scope and settings.
Report which candidates were resolved and which remain; a lower duplicate count
is supporting evidence, not the goal.
