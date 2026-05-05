# Agentic Rust MCP

Production-grade **Rust MCP (Model Context Protocol) server** for agentic AI pipelines.

Integrates Render, Vercel, Buffer, and Firestore into a unified interface for Claude Code and other AI agents.

## Features

### Stage 2: Resources & Prompts ✅
- **3 Read-Only Resources** (system status, content schedules, activity logs)
- **3 Reusable Prompts** (deployment analyzer, content scheduler, activity log analyzer)
- **stdio transport** for Claude Code integration

### Stage 3: Tools ✅
- **`agency_pulse`** — Check Render & Vercel deployment status (4 services monitored)
- **`content_check`** — Query Buffer content schedules across YouTube, Instagram, LinkedIn
- **`data_vault`** — Access Firestore leads & logs (track active, pending, completed projects)

### Stage 4: Advanced Features (Coming Next)
- Streaming responses for long-running tasks
- OAuth 2.1 credential management
- Professional tracing & logging

## Quick Start

```bash
cargo build --release
./target/release/agentic-rust-mcp
```

## Example: Invoking Tools

```json
{"type": "tool", "name": "agency_pulse", "args": {}}
{"type": "tool", "name": "content_check", "args": {}}
{"type": "tool", "name": "data_vault", "args": {}}
```

Returns structured JSON with:
- Deployment statuses (live/deploying/failed)
- Scheduled content with approval status
- Lead pipeline data and metrics

## Architecture

Built with **production-grade Rust** for:
- **Type Safety** — serde & schemars ensure correct data structures
- **Performance** — tokio async/await + LTO optimizations
- **Logging** — tracing-subscriber with structured logs
- **Security** — dotenv for safe credential handling

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Runtime | tokio (async/await) |
| Serialization | serde + schemars |
| HTTP | reqwest |
| Logging | tracing + tracing-subscriber |
| Config | dotenv |

## Resume Points

- ✅ **Agentic AI Foundation** — Implements 2026 MCP standard (110M SDK downloads)
- ✅ **Modular Architecture** — Stage-based build enables independent feature completion
- ✅ **Production-Ready** — Type-safe, async, optimized for enterprise scale
- ✅ **Multi-Tool Integration** — Demonstrates DevOps + content + data pipeline orchestration
- ✅ **Tool Invocation** — Claude Code can now trigger deployments checks, content queries, lead data

## Capabilities

| Component | Count | Status |
|-----------|-------|--------|
| Resources | 3 | ✅ Implemented |
| Prompts | 3 | ✅ Implemented |
| Tools | 3 | ✅ Implemented |
| Streaming | — | 🔄 Stage 4 |
| Security | OAuth 2.1 | 🔄 Stage 4 |

## License

MIT

## Author

Chris Brown  
[Albatross AI](https://albatrossai.online)  
[Portfolio](https://github.com/albatrossflyon-coder)

