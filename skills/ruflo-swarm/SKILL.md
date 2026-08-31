---
name: ruflo-swarm
description: Spin up a coordinated multi-agent swarm (hierarchical or mesh topology) to work a complex, multi-file task on a ruOS desktop. Use when a task needs 3+ agents working in parallel with anti-drift coordination; skip for single-file edits.
---

# ruflo-swarm

Multi-agent swarm orchestration from the **Ruflo** CLI (`@claude-flow/cli`).
Initialize a swarm with a topology and a max-agent count, then let coordinated
agents work the task. Hierarchical topology is the anti-drift default for coding.

## Commands

```bash
# Initialize a swarm (hierarchical is the anti-drift default for coding)
npx @claude-flow/cli@latest swarm init --topology hierarchical --max-agents 8 --strategy specialized

# Check status / health
npx @claude-flow/cli@latest swarm status
```

- **Topologies:** `hierarchical` (anti-drift, leader-coordinated), `mesh`
  (peer-to-peer), plus ring/star/adaptive.
- **Keep `max-agents` at 6–8** for tight coordination on coding tasks.
- Pair the CLI (coordination state) with your MCP client's own agent/Task tools,
  which do the actual file operations.

## When to use

Complex features, cross-module refactors, or audits that fan out across many
files. For a one-line lookup or a single-file edit, don't spin up a swarm.

## Package

Ruflo — [`@claude-flow/cli`](https://www.npmjs.com/package/@claude-flow/cli).
Related: [`ruflo-memory`](../ruflo-memory/SKILL.md),
[`ruflo-hooks`](../ruflo-hooks/SKILL.md).
