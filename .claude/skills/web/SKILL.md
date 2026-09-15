---
name: web
description: Launch cargo-coupling's interactive Web visualization for a specified Rust project.
argument-hint: "[path] [--port PORT] [--no-open]"
disable-model-invocation: true
---

# Launch the Web Visualization

Run from this checkout with the requested target (default `./src`). `cargo run`
builds as needed; a separate build is unnecessary for launch.

```bash
rtk proxy cargo run -- coupling --web ./src
rtk proxy cargo run -- coupling --web --port 8080 --no-open ./src
```

Preserve the requested port and browser behavior. Use `--no-open` in a headless
session. Keep track of the process started for this task and report its actual URL
once it is serving. If startup fails, inspect the failure before changing ports or
stopping processes. Stop only the server this task owns when cleanup is requested
or when a temporary verification run ends.

Read [ui-guide.md](ui-guide.md) only when the user needs help exploring the graph,
filters, timeline, or export. For implementation changes, read
[Web UI rules](../../rules/web-ui.md) and the relevant source; launching the UI does
not require the full development documentation.
