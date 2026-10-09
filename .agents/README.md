# Shared Repository Skills

Edit skills in `.agents/skills/`. Codex loads this directory directly, and
`.claude/skills` is a relative symlink to `../.agents/skills` for Claude Code.
Both hosts read the same `SKILL.md` and supporting files. These skills use source
and commands from this checkout; they are not standalone global installations.

## Using the Skills

In Codex, use `/skills` or type `$` to select a skill. In Claude Code, type its
slash command. For example, `$analyze ./src` and `/analyze ./src` select the same
workflow. Restart the host if a filesystem change does not appear.

| Task | Skills |
|------|--------|
| Run or prioritize analysis | `analyze`, `check-balance`, `hotspots` |
| Understand commands and findings | `cargo-coupling`, `explain-issue`, `balanced-coupling` |
| Review or change Rust structure | `coupling-review`, `coupling-full-review`, `refactor`, `similarity` |
| Verify behavior or prepare a release | `e2e-test`, `mutants`, `release` |
| Start the visualization | `web` |

The Rust review names include `coupling-` to avoid collisions with personal prose
review skills. Use `coupling-review` for module boundaries and
`coupling-full-review` for a broader architecture review. The former `review` and
`full-review` repository names are no longer provided. The duplicate legacy
command files were removed; `/e2e-test` is provided by its skill.

## Host Settings

| Concern | Codex | Claude Code |
|---------|-------|-------------|
| Discovery | `.agents/skills/` | `.claude/skills/` symlink |
| Explicit invocation | `$skill-name` | `/skill-name` |
| `web` is explicit-only | `agents/openai.yaml` with `policy.allow_implicit_invocation: false` | `disable-model-invocation: true` in frontmatter |
| `balanced-coupling` is background guidance | Discoverable as a normal skill | `user-invocable: false` hides the slash command |
| Tool permissions and hooks | Codex configuration | Claude Code configuration; repository hooks are in `.claude/settings.json` |

`argument-hint` provides Claude Code menu help. It is not required to interpret
arguments in Codex. Invocation policy and tool permissions are host settings,
not portable instructions: the shared body carries the task's authorization and
verification requirements. `release` distinguishes preparation from publication;
loading the skill alone does not authorize publishing.

The discovery and invocation behavior follows the
[Codex skills documentation](https://learn.chatgpt.com/docs/build-skills) and
[Claude Code skills documentation](https://code.claude.com/docs/en/skills).

## Maintaining a Skill

Keep one task per skill, with a matching directory and frontmatter `name` and a
short `description` that distinguishes it from nearby workflows. Preserve
working command examples and project-specific pitfalls. Keep optional report
templates and detailed references outside `SKILL.md`, with direct relative links
and a note about when to read them. This follows the
[Agent Skills specification](https://agentskills.io/specification) and
[authoring guidance](https://agentskills.io/skill-creation/best-practices).

Retain existing invocation policies when editing a skill. Do not add blanket
`allowed-tools` grants: in Claude Code these pre-approve tools, rather than
restricting the skill to a tool list. Both hosts should use their normal
permission settings.

For a change to discovery, names, or references, validate the YAML, check local
links, and inspect each host's discovered skill list. Example local checks:

```bash
rtk proxy yq --front-matter=extract eval '.' .agents/skills/analyze/SKILL.md
rtk proxy lychee --offline --hidden --no-progress '.agents/**/*.md'
rtk git diff --check
```

When workflow behavior changes, try a representative request and an adjacent
request that should choose another skill, then inspect the commands and result.
Discovery and link checks alone do not verify model behavior. Keep those trials
within the requested scope and isolate generated artifacts and account state.
