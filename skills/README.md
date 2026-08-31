# ruOS Skills

A ruOS desktop ships the full **ruvnet AI stack** preinstalled. "Skills" here are
the ruvnet-stack capabilities you drive from inside a ruOS desktop (or from any
Claude Code / MCP client attached to one) via `npx`. Each skill is a
[Claude Code Skill](https://docs.claude.com/en/docs/claude-code/skills)-style
`SKILL.md` descriptor: a name, a one-line purpose, and the exact `npx` commands.

## The two real packages

Everything below is one of **two published npm packages** — this is the honest
picture, not four separate products:

| Package | npm | What it is |
|---------|-----|------------|
| **Ruflo** (`claude-flow`) | [`@claude-flow/cli`](https://www.npmjs.com/package/@claude-flow/cli) | Enterprise AI-agent orchestration for Claude Code — swarms, vector memory, self-learning hooks, MCP integration, hive-mind consensus. "Ruflo" is the product name; the CLI ships as `@claude-flow/cli`. |
| **RuVector** (`ruvector`) | [`ruvector`](https://www.npmjs.com/package/ruvector) | Rust-native vector database / adaptive memory substrate — local semantic embeddings, persistent vector retrieval, graph relationships, HNSW indexing. |

The `swarm`, `memory`, and `hooks` skills are three surfaces of the **same
Ruflo CLI**. `ruvector` is the standalone vector substrate Ruflo's memory can sit on.

## Skills

| Skill | Package | Purpose |
|-------|---------|---------|
| [`ruflo-swarm`](./ruflo-swarm/SKILL.md) | Ruflo | Spin up a multi-agent swarm (hierarchical/mesh) to coordinate a complex task. |
| [`ruflo-memory`](./ruflo-memory/SKILL.md) | Ruflo | Store and semantically search agent memory with ONNX vector embeddings. |
| [`ruflo-hooks`](./ruflo-hooks/SKILL.md) | Ruflo | Self-learning pre/post-task hooks + task routing. |
| [`ruvector`](./ruvector/SKILL.md) | RuVector | Persistent, adaptive vector memory — remember/recall across sessions. |

## Install

No global install needed — every command is `npx`, which fetches on first use:

```bash
npx @claude-flow/cli@latest --help     # Ruflo
npx ruvector@latest --help             # RuVector
```

On a ruOS desktop these run against the desktop's own filesystem and (optionally)
the ruflo daemon. To wire Ruflo's MCP tools into a Claude Code session:

```bash
claude mcp add claude-flow -- npx -y @claude-flow/cli@latest
```

## Real vs roadmap

- **Real, today:** the `npx` CLIs and their `swarm` / `memory` / `hooks`
  subcommands; the Ruflo MCP server; RuVector's local embed + search.
- **Roadmap:** a hosted, multi-tenant "skills gallery" surfaced in the ruOS
  desktop UI (one-click skill invocation per tenant). Today you invoke skills
  from a terminal or an attached MCP client. See the root
  [README](../README.md#roadmap).

> Version numbers move — pin with `@latest` or a checked version, and treat any
> subcommand not shown by `--help` on your installed version as unavailable.
