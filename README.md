<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&color=0:CE422B,100:F74C00&height=250&section=header&text=Agentic%20Rust%20MCP&fontSize=75&fontColor=1a1a1a&animation=fadeIn&fontAlignY=35&desc=Production-grade%20Rust%20MCP%20server%20for%20agentic%20AI%20pipelines&descAlignY=58&descSize=20&descColor=1a1a1a" alt="Agentic Rust MCP" width="100%"/>
</p>

Integrates Render, Vercel, Buffer, and Firestore into a unified interface for Claude Code and other AI agents.

## ✨ All 4 Stages Complete

### Stage 1: Foundation ✅
- Async stdio transport
- Message loop architecture
- tokio runtime initialization

### Stage 2: Resources & Prompts ✅
- **3 Read-Only Resources** (system status, content schedules, activity logs)
- **3 Reusable Prompts** (deployment analyzer, content scheduler, activity log analyzer)

### Stage 3: Tools ✅
- **`agency_pulse`** — Check Render & Vercel deployment status (4 services)
- **`content_check`** — Query Buffer content schedules
- **`data_vault`** — Access Firestore leads & logs

### Stage 4: Roof & Finishings ✅
- **Streaming** — Real-time updates for long-running tasks
- **OAuth 2.1** — Secure token-based authentication
- **Professional Logging** — JSON structured logs for production
- **Tests** — 4/4 unit tests passing
- **.env Support** — Safe credential management

## Quick Start

```bash
# Build production binary
cargo build --release

# Copy and configure environment
cp .env.example .env
# Edit .env with your OAuth token and API keys

# Run server
./target/release/agentic-rust-mcp
```

## Usage Examples

### Request Resources
```json
{"type": "resource", "name": "system-status"}
```

### Request Prompts
```json
{"type": "prompt", "name": "deployment-analyzer"}
```

### Invoke Tools
```json
{"type": "tool", "name": "agency_pulse", "args": {}}
{"type": "tool", "name": "content_check", "args": {}}
{"type": "tool", "name": "data_vault", "args": {}}
```

### Stream Long-Running Task
```json
{"type": "streaming_task", "task": "process_deployment"}
```

Returns stream markers:
- `stream_start` — Task initiated
- `stream_update` — Progress update (3 steps)
- `stream_end` — Task completed

## Security

### OAuth 2.1 Token Management
- Load from `.env` (never hardcoded)
- Verify token validity before requests
- Support token refresh flow
- Bearer token authentication

### Credential Protection
- dotenv for safe .env loading
- .env in .gitignore
- .env.example for documentation
- Environment variable support

## Performance

| Metric | Value |
|--------|-------|
| Build Time | 29.18s (release, LTO) |
| Test Execution | 12.95s |
| Binary Size | ~8MB (release) |
| Memory | ~5-10MB at rest |
| Tests Passing | 4/4 ✅ |

## Logging

### JSON Structured Format
```json
{
  "timestamp": "2026-05-05T00:00:00Z",
  "level": "info",
  "target": "agentic_rust_mcp",
  "message": "Tool executed successfully",
  "tool": "agency_pulse",
  "duration_ms": 125
}
```

### Enable/Disable
```bash
RUST_LOG=info ./target/release/agentic-rust-mcp
RUST_LOG=debug ./target/release/agentic-rust-mcp
```

## Architecture

```
┌─────────────────────────────────────────┐
│  Claude Code / AI Agent                 │
└────────────┬────────────────────────────┘
             │ stdio (JSON)
             ↓
┌─────────────────────────────────────────┐
│  Stage 1: Foundation                    │
│  - Async message loop                   │
│  - Type-safe request parsing            │
└─────────────┬───────────────────────────┘
              │
    ┌─────────┼─────────┐
    ↓         ↓         ↓
┌──────┐  ┌──────┐  ┌──────┐
│ S2   │  │ S3   │  │ S4   │
│─────┐│  │─────┐│  │─────┐│
│Res  ││  │Tool ││  │OAuth││
│Prom ││  │─────││  │Log  ││
│─────││  │agcy ││  │Strm ││
│─────│┘  │cont ││  │─────│┘
│─────│   │data ││  │─────│
└─────┘   └─────┘│  └─────┘
                 │
    ┌────────────┼────────────┐
    ↓            ↓            ↓
[ Render ]  [ Vercel ]   [ Buffer + Firestore ]
```

This server is the final layer in a larger, multi-project memory architecture spanning research (NotebookLM), session state (OB1 + Supabase), and a RAG engine — see [docs/memory-architecture.md](docs/memory-architecture.md) for the full picture.

## Resume Impact

- ✅ **Agentic AI Standard** — Implements 2026 MCP protocol
- ✅ **Production-Ready** — Type-safe, async, tested, secured
- ✅ **Multi-Stage Build** — Shows modular architecture discipline
- ✅ **Tool Integration** — DevOps + Content + Data orchestration
- ✅ **Security First** — OAuth 2.1, env vars, no secrets in code
- ✅ **Observable** — Structured JSON logging for monitoring

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Runtime | tokio (async/await) |
| Serialization | serde + schemars |
| HTTP | reqwest |
| Logging | tracing + tracing-subscriber |
| Security | dotenv, OAuth 2.1 |
| Testing | built-in cargo test |
| IDs | uuid v4 |
| Timestamps | chrono |

## License

MIT

## Author

Chris Brown  
[Albatross AI](https://albatrossai.online)  
[Portfolio](https://chrisbrown-dev.vercel.app)


