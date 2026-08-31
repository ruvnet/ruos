---
name: ruflo-memory
description: Store and semantically search agent memory with ONNX vector embeddings on a ruOS desktop, using the Ruflo CLI. Use to persist patterns/decisions across sessions and recall them by meaning rather than exact key.
---

# ruflo-memory

Vector-backed agent memory from the **Ruflo** CLI (`@claude-flow/cli`). Entries
are stored with ONNX (`all-MiniLM-L6-v2`, 384-dim) embeddings and retrieved by
semantic similarity, organized into namespaces.

## Commands

```bash
# Store a value with a vector embedding
npx @claude-flow/cli@latest memory store \
  --key "pattern-auth" --value "JWT with refresh tokens" --namespace patterns

# Semantic search by meaning
npx @claude-flow/cli@latest memory search --query "authentication patterns"

# List a namespace
npx @claude-flow/cli@latest memory list --namespace patterns
```

## Notes

- Namespaces keep memory scoped (e.g. `patterns`, `loop`, per-project).
- The MCP surface (`memory_store`, `memory_search`, `memory_search_unified`)
  exposes the same store to an attached Claude Code session.
- The vector substrate can be [`ruvector`](../ruvector/SKILL.md) — see that skill
  for the standalone vector database.

## Package

Ruflo — [`@claude-flow/cli`](https://www.npmjs.com/package/@claude-flow/cli).
Related: [`ruflo-swarm`](../ruflo-swarm/SKILL.md),
[`ruvector`](../ruvector/SKILL.md).
