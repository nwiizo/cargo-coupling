# Repository Guidelines

## Project Structure & Module Organization
`src/main.rs` contains the Clap entrypoint; `src/lib.rs` re-exports the public API. Core analysis and reporting live in `src/analyzer.rs`, `src/metrics.rs`, `src/balance.rs`, `src/volatility.rs`, `src/report.rs`, `src/config.rs`, `src/workspace.rs`, and `src/cli_output.rs`. Web backend code is under `src/web/`; shipped frontend assets live in `web-assets/`. Benchmarks are in `benches/analysis_benchmark.rs`, and screenshots/docs are in `docs/images/`.

## Build, Test, and Development Commands
Use `cargo build --release` for the production build path used in CI. Run `cargo test --all-features` for the full unit-test suite. Format with `cargo fmt --all`; CI uses `cargo fmt --all -- --check`. Lint with `cargo clippy --all-targets --all-features -- -D warnings`. For a local smoke test, run `cargo run -- coupling ./src`. For the browser UI, run `cargo run -- coupling --web --no-open ./src` and open `http://localhost:3000`. Use `cargo bench --bench analysis_benchmark` when changing analyzer performance.

## Coding Style & Naming Conventions
Target Rust 2024 and keep rustfmt defaults with 4-space indentation. Follow standard Rust naming: `snake_case` for modules, functions, and tests; `PascalCase` for types and enums; `SCREAMING_SNAKE_CASE` for constants. Keep CLI flags, README examples, and output formatting aligned when behavior changes. Prefer small focused modules instead of growing `main.rs`.

## Testing Guidelines
Tests are colocated with implementation in `#[cfg(test)] mod tests`; there is no top-level `tests/` directory at present. Add regression tests for parser edge cases, config loading, CLI output, and web graph/report generation when touching those paths. After analyzer or scoring changes, rerun `cargo run -- coupling --summary ./src` to confirm behavior on the repository itself.

## Commit & Pull Request Guidelines
Follow the Conventional Commit style already used in history: `feat(analysis): ...`, `fix(test): ...`, `deps: ...`, `ci: ...`. Keep commit subjects imperative and scoped. PRs should summarize user-visible changes, note new flags or config keys, link related issues when available, and include screenshots for CLI/Web UI output changes. Before pushing, ensure fmt, clippy, tests, and the release build all pass.

## Agent-Specific Notes
Start with `CLAUDE.md`, then read the relevant `.claude/docs/*`, `.claude/rules/*.md`, and `.claude/skills/*/SKILL.md` files for the task. For modularity work, consult `.claude/docs/khononov-framework.md`, `.claude/docs/issue-types.md`, and `.claude/skills/review/SKILL.md`. For Web UI tasks, use `.claude/rules/web-ui.md` and `.claude/skills/web/SKILL.md`. Treat the current source tree as the source of truth and use `.claude` as workflow guidance, because some older `.claude` file maps no longer exactly match `src/`.
