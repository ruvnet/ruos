---
name: ruvector
description: Persistent, adaptive vector memory for agents on a ruOS desktop — a Rust-native vector database with local semantic embeddings, HNSW retrieval, and graph relationships. Use to remember and recall across sessions with no database server or API key.
---

# ruvector

**RuVector** (`ruvector`) is a Rust-native vector database and adaptive memory
substrate for agents that need to remember across sessions. It combines local
semantic embeddings, persistent vector retrieval, graph relationships, explicit
feedback learning, and memory lifecycle controls. The default retrieval path runs
**locally** — no database server or API key required.

## Commands

```bash
npx ruvector@latest --help

# Remember and recall run locally, no server/key needed — see the ruvector
# README for the current verb set on your installed version:
#   https://www.npmjs.com/package/ruvector
```

> RuVector's CLI/MCP verb surface evolves across versions. Run `--help` on the
> version you install (this repo grounded against `ruvector` 0.3.x) rather than
> assuming a verb exists.

## Notes

- Rust-native, AgenticDB-compatible; ships an MCP surface (`brain_*`, `rvf_*`,
  `hooks_*`) for attached clients.
- Learning happens from recorded outcomes and feedback, not from reads alone.
- Hosted/shared-memory services are optional and form a separate data boundary.
- Ruflo's [`memory`](../ruflo-memory/SKILL.md) skill can use RuVector as its
  vector substrate.

## Package

RuVector — [`ruvector`](https://www.npmjs.com/package/ruvector) ·
docs: <https://cognitum.one/ruvector>.
