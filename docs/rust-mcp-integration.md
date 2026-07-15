# Rust MCP Integration — agentic-rust-mcp

## Repo
https://github.com/albatrossflyon-coder/agentic-rust-mcp

## Purpose
Production-grade MCP server that pipes the memory system into AI agent pipelines. The delivery layer — once memory is in rag-system, this makes it accessible to Claude, Jim, and Copilot via MCP protocol.

## What It Exposes
| Tool | Function |
|------|----------|
| `agency_pulse` | Render + Vercel deployment status |
| `content_check` | Buffer content schedules |
| `data_vault` | Firestore leads + logs |

## Gmail Integration (Phase 2 — In Progress)
- `google-gmail1` crate wired into Cargo.toml
- `src/draft_sender.rs` — Gmail draft generation
- Reads scored leads from job-lead-discovery → generates Gmail drafts automatically

## Status
- v0.4.0 — all 4 stages complete (Foundation, Resources/Prompts, Tools, OAuth/Logging)
- Gmail draft sender: in progress
- RAG query integration: Phase 3

## Run
```bash
cargo build --release
cp .env.example .env  # fill in keys
./target/release/agentic-rust-mcp
```
