---
name: e2e-test
description: Verify cargo-coupling CLI behavior end to end using disposable Rust fixtures.
argument-hint: "[--quick] [--verbose] [--keep] [--web] [scenario]"
---

# End-to-End Verification

Exercise the requested CLI behavior with observable assertions. Use existing
regression tests when they cover the request; use
[test-scenarios.md](test-scenarios.md) for a minimal external project and checks for
module paths, test exclusion, and config loading. Run this package's `cargo run`
from its checkout, passing the fixture path as the analysis target.

Create a unique fixture with `mktemp -d`; never remove or reuse a fixed `/tmp` path.
Local fixtures are disposable and have no production access. Continue their setup,
execution, and checks without repeated approval. Remove only resources created by
this run, retaining them when requested with `--keep`.

Select coverage from the request:

- `--quick`: nested modules, valid JSON, and summary output.
- Default: also verify test exclusion, configuration, and requested output modes.
- `--verbose`: include commands, assertions, and observed values in the report.
- `--web`: add server/API checks using the
  [Web UI rules](../../rules/web-ui.md); stop only the server started for this run.

These are skill options, not flags to forward to cargo-coupling. Run repository
checks when required by the task; a smoke test does not establish full E2E coverage.
Fix and recheck failures caused by an implementation the user asked you to make.
For a test-only request, report failures with reproduction details.

Finish with actual results, skipped scenarios, exit statuses, and retained fixture
paths. Do not report checks as passed merely because a command produced output.
