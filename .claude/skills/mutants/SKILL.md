---
name: mutants
description: Run cargo-mutants on selected Rust code and investigate gaps in tests revealed by surviving mutations.
argument-hint: "[-f FILE] [-F FUNCTION]"
user-invocable: true
allowed-tools: Bash, Read, Grep, Glob, Edit
---

# Mutation Testing

Run the requested scope, or start with the changed files. A full-project run is
appropriate when requested; mutation testing is not a default release requirement.

```bash
rtk proxy cargo mutants --no-shuffle -f src/config.rs
rtk proxy cargo mutants --no-shuffle -F 'function_name'
```

Use `cargo mutants --help` for installed options and job limits. Inspect the actual
run output and `mutants.out/missed.txt` to distinguish outcomes:

| Result | Interpretation |
|--------|----------------|
| caught | Tests detected the mutation |
| missed | Tests did not detect it; inspect reachability and observable behavior |
| unviable | The mutated program did not compile |
| timeout | Execution exceeded its limit; investigate before assigning a cause |

Prioritize changed business behavior and actionable gaps. Existing misses may be
relevant to the requested scope; do not dismiss CLI or output behavior solely by
filename. Some surviving mutations preserve behavior or cannot be meaningfully
exercised, so zero missed mutants is not a universal completion criterion.

For diagnosis, report actionable misses and unresolved cases with evidence. When
test improvements are requested, add assertions for observable behavior, run the
affected tests, and rerun the relevant mutations. Complete with caught/missed/
unviable/timeout results for the tested scope and explain remaining limitations.
