# Rust MCP Integration — agentic-rust-mcp

## Repo
https://github.com/albatrossflyon-coder/agentic-rust-mcp

## Purpose
An MCP server that pipes the memory system into AI agent pipelines. The delivery layer — once memory is in rag-system, this makes it accessible to Claude, Jim, and Copilot via MCP protocol.

## What It Exposes
| Tool | Function |
|------|----------|
| `agency_pulse` | Render + Vercel deployment status |
| `content_check` | Buffer content schedules |
| `data_vault` | Firestore leads + logs |
| `send_gmail` | Sends an email directly via Gmail SMTP (`to`/`subject`/`body`) |

## Gmail Integration — shipped, direct-send not drafts
Sends directly via Gmail SMTP using the `lettre` crate (`src/gmail_sender.rs`) —
simpler than the originally-planned Gmail-API draft-generation approach
(`google-gmail1` + a draft-sender module), which was never built. Direct send
is the real intended design now, not a stand-in for the drafts plan.

## Status
- v0.4.0 — 4 tools shipped (agency_pulse, content_check, data_vault, send_gmail) over a hand-rolled JSON-RPC 2.0/stdio MCP server, plus a public HTTP demo (`src/bin/web_server.rs`). No OAuth, no MCP resources/prompts, no streaming — those were never built despite earlier docs claiming otherwise.
- Gmail: shipped (direct SMTP send)
- RAG query integration: not started

## Run
```bash
cargo build --release
cp .env.example .env  # fill in keys
./target/release/agentic-rust-mcp
```
