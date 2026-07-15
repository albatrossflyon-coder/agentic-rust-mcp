# Memory Architecture

**The unified memory architecture for Albatross AI** — a multi-layer system that gives AI agents persistent, queryable memory across sessions, projects, and research. This server (`agentic-rust-mcp`) is the final layer: it pipes that memory into actual agent pipelines.

## What This Is

Most AI agents forget everything between sessions. This system doesn't. It combines four memory layers into one coherent architecture so that Claude, Jim (Gemini), and Copilot all operate from the same shared knowledge base.

## Architecture

| Layer | Tool | Purpose |
|-------|------|---------|
| **Research** | NotebookLM | PDFs, YouTube videos, industry sources — indexed and queryable |
| **Session / Ops** | OB1 + Supabase | Session state, decisions, what shipped, what's pending |
| **Local Backup** | Obsidian Vault | Session logs, OB1 fallback when Supabase is down |
| **RAG Engine** | `rag-system` (private) | Queryable vector store over Albatross business context |
| **Rust / MCP** | This repo | Production MCP server — pipes memory into AI agent pipelines |

## How the Layers Connect

```
Research Sources (PDFs, YouTube, Docs)
        │
        ▼
  [NotebookLM] ──────────────────────────────┐
                                              │
Session State (decisions, shipped, pending)  │
        │                                    │
        ▼                                    │
  [OB1 + Supabase] ── fallback ──► [Obsidian Vault]
        │
        ▼
  [rag-system] ←── indexes Albatross corpus ──┘
        │
        ▼
  [agentic-rust-mcp]  ← you are here
        │
        ▼
  Claude / Jim / Copilot (agents read from unified memory)
```

## Layer Details

- [Research Memory — NotebookLM](research-memory.md)
- [Session Memory — OB1 + Supabase](session-memory.md)
- [Local Backup — Obsidian](local-backup.md)
- [RAG Integration](rag-integration.md)
- [Rust MCP Integration](rust-mcp-integration.md)

## Status

| Component | Status |
|-----------|--------|
| OB1 + Supabase | Live |
| Obsidian local backup | Live |
| rag-system | Built — Phase 1 complete (private repo) |
| agentic-rust-mcp | Built — all 4 stages complete |
| NotebookLM research layer | Active |
| Full pipeline integration | In progress |
