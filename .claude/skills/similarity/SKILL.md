---
name: similarity
description: Inspect Rust duplicate-code candidates with similarity-rs and assess whether sharing code would improve the design.
argument-hint: "[path] [--threshold N] [--skip-test]"
---

# Rust Code Similarity

Scan the requested scope (default `./src`) with `similarity-rs`. Start with one scan
and inspect the reported code; thresholds rank candidates, not required actions:

```bash
rtk proxy similarity-rs ./src --threshold 0.85 --skip-test
```

Honor requested options. Use `--print` for source detail or `--experimental-types`
when comparing types is relevant. Consult `similarity-rs --help` for other options;
do not run several threshold sweeps without an unresolved question.

Check callers, semantics, and intentional differences. Recommend shared code only
when responsibilities align and the abstraction reduces maintenance cost. A high
similarity percentage alone does not justify generics, traits, or extraction.

When assessing a structural change, use cargo-coupling on the relevant scope to
check its effect on boundaries. Launch visualization only when requested or needed
to answer the task. Report locations, the shared responsibility (if any), and the
recommendation, including cases where duplication should remain. Implement fixes
when requested and verify the changed behavior and coupling.
