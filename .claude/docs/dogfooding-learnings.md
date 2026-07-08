# Dogfooding learnings — running cargo-coupling on itself

Captured while trying to raise cargo-coupling's own grade honestly (it sits at D mid-development).
These shaped real changes and remain as guidance.

## Applied
1. **Essential ≫ accidental volatility.** Raw git churn conflates essential volatility (a Core
   subdomain genuinely changes often) with accidental volatility (a one-off commit burst / sprint).
   Letting churn drive the balance score manufactured false `Cascading Change Risk` during heavy
   development. Fix: when a target is subdomain-classified, its **essential** volatility is
   authoritative for scoring; raw churn still feeds the `AccidentalVolatility` diagnostic; unclassified
   projects keep git-churn behavior (no regression). (`src/balance.rs`, `src/main.rs`, `.coupling.toml`)
2. **Make hidden coupling explicit.** A real `config`/`volatility` hidden coupling (shared volatility
   semantics, no code edge) was removed by giving `volatility` ownership of `Volatility` and importing
   it in `config` — turning implicit shared knowledge into an explicit dependency.
3. **Default report deduped.** `Immediate Actions` listed the same `(type, source, target)` up to 5×;
   now distinct.

## Open learnings (guidance / roadmap, not yet changed)
4. **A stable central abstraction is good design, not a defect.** `balance` has ~41 afferent
   dependents and is flagged High. Per the balance rule, strong + far + **low** volatility is
   *Acceptable*; high afferent is only dangerous when the hub is **volatile**. Afferent/cascading
   severity should scale with the target's essential volatility. (For cargo-coupling specifically,
   `balance` is Core=high essential volatility, so it stays flagged — correctly.)
5. **Entrypoint hubs distort a whole-`src` grade.** The binary entrypoint (`main`) is inherently a
   high-efferent hub and couples to the crate root; this keeps High issues present and blocks grade A
   for `./src` regardless of structure. Consider recognizing entrypoints rather than penalizing them.
6. **Git-history is a gaming vector.** Splitting/moving a file resets its churn, which can look like a
   volatility improvement with no real change. Churn attribution should follow renames; reviewers
   should treat sudden grade jumps after file moves with suspicion.
7. **The grade rewards fewer High issues, not fewer cycles.** Removing all circular dependencies did
   not improve the grade; relocating types into a volatile module made it worse. Optimize for High
   issues and ownership boundaries, not cycle count alone.

## Honest conclusion
Grade A was **not** reached, and chasing it further would require gaming (`--no-git`, git-history
resets, threshold/subdomain manipulation) — which would destroy the tool's only real value: a signal
you can trust. The honest grade for a tool mid-sprint with a genuinely central core (`balance`) and an
entrypoint hub (`main`) is **D→C**; it rises naturally as churn settles. Integrity of the metric
outranks the letter.

## v0.3.7 learnings (scoring-model re-verification)

8. **Distance must be structural, never syntactic.** Deriving distance from import syntax
   (`crate::` = Same, `super::` = Different) inverted reality: it punished the tool's own
   recommended god-module splits (siblings import via `super::`) and hid genuinely far
   couplings written as `crate::far::x`. Distance now comes from module-tree adjacency.
9. **Import-style changes can silently delete coupling — an adversarial reviewer caught it.**
   Switching deep imports to facade re-exports (`use crate::Type`) looked like "consume the
   stable contract", but bare re-exported type names failed module resolution, were classified
   as external crates, and vanished from internal accounting — flipping the self-grade C→A for
   the wrong reason. The fix was a resolver improvement (type registry lookup), not an import
   revert. Lesson: any grade jump caused by an import/file-shape change is suspect until the
   coupling is shown to be *still visible* under the new shape.
10. **Rust visibility is the publication boundary for strength.** Cross-module access to a
    private field does not compile, so blanket "field access = Intrusive" conflated use of the
    published record (Model) with intrusion. Intrusive now means unpublished access:
    crate-restricted types and foreign inherent impls.
11. **Act-now severity needs evidence.** A refactoring sprint gave young sibling files perfect
    co-change ratios with n=5 commits; Hidden Coupling High now requires a persistent pattern
    (≥8 co-changes), and adjacent (same-package) pairs are exempt — their co-change is cohesion.
12. **The honest grade can be B and that is the deliverable.** After all verified fixes, the
    remaining mediums are this release's own cross-cutting burst, correctly observed; forcing
    the last few off would have required threshold/classification gaming. Integrity of the
    metric outranks the letter (see learning above — it keeps being true).
