# Coupling Findings

Use [explain-issue](../../.agents/skills/explain-issue/SKILL.md) for the issue names,
descriptions, and detector entrypoints. That guide links to the current Rust
implementation instead of maintaining a second severity table here.

A finding's severity depends on its evidence, thresholds, configuration, and
detector exceptions. Use the reported severity; an issue name alone does not
establish urgency or require a particular refactoring. In particular, a cycle is
a review signal, public fields are not automatically an encapsulation defect,
and primitive parameters do not automatically need newtypes.

For prioritization, relate the finding to a concrete change and the knowledge
that would spread across boundaries. Read the
[balance model](../../.agents/skills/balanced-coupling/SKILL.md) for interpretation
and the analysis manifest for missing evidence.
