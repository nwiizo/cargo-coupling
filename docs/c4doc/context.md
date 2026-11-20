# What can I learn about my code?

[All views](README.md) · [Diagram source](context.mmd) · [Evidence](evidence.md)

cargo-coupling helps you inspect structural coupling and change history when deciding where to refactor. Use the CLI for reports or open the Web UI to explore the results.

- Kind: System context
- Scope: cargo-coupling

```mermaid
---
title: "System context: cargo-coupling — What can I learn about my code?"
---
flowchart TB
    accTitle: System context: cargo-coupling — What can I learn about my code#63;
    n_developer["Rust developer #91;Person#93; — Investigates coupling and plans code changes"]:::observed
    n_coupling["cargo-coupling #91;Software system#93; — Evaluates Rust coupling to support design decisions"]:::observed
    n_cdn["unpkg.com #91;Software system#93; — Serves JavaScript libraries used by the UI"]:::observed
    n_developer -->|"Obtains coupling analysis for change decisions"| n_coupling
    n_coupling -->|"Retrieves libraries for the Web UI"| n_cdn
    classDef observed fill:#e8f1fa,stroke:#28577c,color:#132f46
    classDef inferred fill:#fff4de,stroke:#8a5100,color:#573300,stroke-dasharray:5 5
    c4doc_legend["Legend (not a model element)<br/>Blue solid boxes: Observed<br/>Orange dashed boxes: Inferred<br/>Solid arrows: Observed<br/>Dotted arrows: Inferred<br/>Observed: cited source, not semantic proof<br/>Inferred: assumption requiring review<br/>Boundary labels name their C4 type<br/>Deployment nesting shows hosting"]
    style c4doc_legend fill:#fff,stroke:#666,color:#222
```

## Reading notes

Start with the Rust developer and follow the request to cargo-coupling. The tool combines source analysis with Git history to help you investigate which parts of a project are connected and how they change.

The CDN arrow represents a request for Web visualization libraries from unpkg.com. It does not represent an upload of analysis results. The next view separates the browser from the Rust application and shows Cargo and Git.

This guide covers local CLI and Web use in 0.4.0 at revision 5b019b2642f7dd7275d742d308b898b9047d2e6c. CI and third-party library embedding are outside this view. The explanations come from inspected source and documentation; the Web paths were not exercised as runtime tests. Comparison conditions and detailed mappings are recorded in docs/c4doc-review.md.

## Elements

| Name | Kind | Technology | Responsibility | Evidence |
| --- | --- | --- | --- | --- |
| Rust developer | Person | — | Investigates coupling and plans code changes | Observed: [README.md:7–21](../../README.md#L7-L21) |
| cargo-coupling | Software system | — | Evaluates Rust coupling to support design decisions | Observed: [README.md:1–9](../../README.md#L1-L9) |
| unpkg.com | Software system | — | Serves JavaScript libraries used by the UI | Observed: [web-assets/index.html:10–13](../../web-assets/index.html#L10-L13) |

## Relationships

| From → To | Interaction | Technology | Evidence |
| --- | --- | --- | --- |
| Rust developer → cargo-coupling | Obtains coupling analysis for change decisions | — | Observed: [README.md:7–21](../../README.md#L7-L21) |
| cargo-coupling → unpkg.com | Retrieves libraries for the Web UI | — | Observed: [web-assets/index.html:10–13](../../web-assets/index.html#L10-L13) |

Continue reading: [Where does the analysis run?](containers.md).
