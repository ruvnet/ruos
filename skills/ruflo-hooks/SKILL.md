---
name: ruflo-hooks
description: Self-learning pre/post-task hooks and model/task routing from the Ruflo CLI, on a ruOS desktop. Use to add coordination, formatting, and learning around Claude Code operations, or to route work by complexity/cost.
---

# ruflo-hooks

Self-learning hooks from the **Ruflo** CLI (`@claude-flow/cli`). Hooks fire around
task and edit lifecycle events to coordinate agents, checkpoint state, and learn
patterns from outcomes. A routing surface picks a handler tier by task complexity.

## Commands

```bash
# Initialize hooks in a project
npx @claude-flow/cli@latest hooks init

# List available hooks / workers
npx @claude-flow/cli@latest hooks list

# Route a task to a handler tier by complexity
npx @claude-flow/cli@latest hooks route --task "add types to this module"
```

- **Lifecycle hooks:** `pre-task` / `post-task`, `pre-edit` / `post-edit`,
  `session-start` / `session-end`.
- **Learning:** post-task outcomes feed a pattern store so future routing sharpens.
- **Background workers:** audit, optimize, testgaps, refactor, benchmark, and more,
  dispatchable independently of the main task loop.

## When to use

To add frequent checkpoints and outcome-learning to a long-running or multi-agent
workflow, or to route simple transforms away from an expensive model.

## Package

Ruflo — [`@claude-flow/cli`](https://www.npmjs.com/package/@claude-flow/cli).
Related: [`ruflo-swarm`](../ruflo-swarm/SKILL.md),
[`ruflo-memory`](../ruflo-memory/SKILL.md).
