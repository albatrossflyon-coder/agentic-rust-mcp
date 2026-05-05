# Agentic Rust MCP

Production-grade **Rust MCP (Model Context Protocol) server** for agentic AI pipelines.

Integrates Render, Vercel, Buffer, and Firestore into a unified interface for Claude Code and other AI agents.

## Features

### Stage 2: Resources & Prompts ✅
- **3 Read-Only Resources** (system status, content schedules, activity logs)
- **3 Reusable Prompts** (deployment analyzer, content scheduler, activity log analyzer)
- **stdio transport** for Claude Code integration

### Stage 3: Tools (Coming Next)
- `agency_pulse` — Check Render & Vercel deployment status
- `content_check` — Query Buffer content schedules
- `data_vault` — Access Firestore leads & logs

### Stage 4: Advanced Features (Coming Soon)
- Streaming responses for long-running tasks
- OAuth 2.1 credential management
- Professional tracing & logging

## Quick Start

```bash
cargo build --release
./target/release/agentic-rust-mcp
```

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

## License

MIT

## Author

Chris Brown  
[Albatross AI](https://albatrossai.online)  
[Portfolio](https://github.com/albatrossflyon-coder)
