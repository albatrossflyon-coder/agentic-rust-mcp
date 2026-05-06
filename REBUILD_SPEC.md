# agentic-rust-mcp — Rebuild Spec for Copilot

## What's Wrong With the Current Code

The current `src/main.rs` is NOT a working MCP server. Two critical problems:

1. **Wrong protocol.** It uses a custom JSON format (`"type": "resource"`, `"type": "tool"`) instead of the actual MCP spec (JSON-RPC 2.0). Claude Code and any real MCP client will fail to connect.

2. **Fake data.** All three tools (`agency_pulse`, `content_check`, `data_vault`) return hardcoded static structs. No real API calls happen. The `mcp = "0.1.1"` SDK in Cargo.toml is imported but never used.

**Goal: Rebuild this as a real, working MCP server that Claude Code can connect to and that hits live APIs.**

---

## MCP Protocol Requirements

A working MCP server over stdio must implement JSON-RPC 2.0 with these messages:

### 1. Initialize handshake
Client sends:
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"claude-code","version":"1.0"}}}
```
Server must respond:
```json
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"agentic-rust-mcp","version":"0.4.0"}}}
```

### 2. Tools list
Client sends:
```json
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
```
Server must respond with all available tools and their input schemas.

### 3. Tool call
Client sends:
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agency_pulse","arguments":{}}}
```
Server responds with tool result.

---

## Recommended Approach: Use `rmcp` crate

Replace `mcp = "0.1.1"` in Cargo.toml with:
```toml
rmcp = { version = "0.1", features = ["server", "transport-io"] }
```

`rmcp` is the maintained community Rust MCP SDK. It handles the JSON-RPC 2.0 protocol, stdio transport, and tool registration — you don't implement the message loop manually.

Basic server structure with rmcp:
```rust
use rmcp::{ServerHandler, model::*, tool, transport::stdio};

#[derive(Clone)]
struct AgenticServer;

#[tool(tool_box)]
impl AgenticServer {
    #[tool(description = "Check live deployment status on Render and Vercel")]
    async fn agency_pulse(&self) -> Result<CallToolResult, McpError> {
        // Real HTTP call here
    }
}

#[tokio::main]
async fn main() {
    let server = AgenticServer;
    stdio().serve(server).await.unwrap();
}
```

---

## Three Tools — Make Them Real

### Tool 1: `agency_pulse`
**Purpose:** Check live deployment status on Render and Vercel.

**Real API calls needed:**
- Render: `GET https://api.render.com/v1/services` with header `Authorization: Bearer {RENDER_API_KEY}`
- Vercel: `GET https://api.vercel.com/v9/deployments` with header `Authorization: Bearer {VERCEL_TOKEN}`

**Output:** List of services with name, status (live/deploying/failed), last deploy time.

**Env vars needed:** `RENDER_API_KEY`, `VERCEL_TOKEN`

### Tool 2: `content_check`
**Purpose:** Check Buffer scheduled posts across channels.

**Real API call needed:**
- Buffer: `GET https://api.bufferapp.com/1/profiles.json?access_token={BUFFER_TOKEN}`
- Then for each profile: `GET https://api.bufferapp.com/1/profiles/{id}/schedules.json`

**Output:** Channels with scheduled post counts, next post times, pending approvals.

**Env vars needed:** `BUFFER_API_KEY`

### Tool 3: `data_vault`
**Purpose:** Query Firestore for leads/data.

**Real API call needed:**
- Firestore REST: `GET https://firestore.googleapis.com/v1/projects/{PROJECT_ID}/databases/(default)/documents/{collection}` with `Authorization: Bearer {FIREBASE_TOKEN}`
- Or use the `firestore` crate for easier access.

**Output:** Document list with relevant fields.

**Env vars needed:** `FIREBASE_PROJECT_ID`, `FIREBASE_API_KEY` (or service account token)

---

## What To Keep From Current Code

- The struct definitions (`DeploymentStatus`, `BufferPost`, `FirestoreLead`) — good, keep these
- The `Cargo.toml` dependency setup — update `mcp` to `rmcp`, keep `reqwest`, `tokio`, `serde`, `chrono`, `dotenv`, `anyhow`
- The `.env` loading with `dotenv::dotenv().ok()`
- The 4 tests structure — update to test real tool outputs

## What To Delete

- The entire custom message loop in `main()` (lines 44-200)
- The `StreamMessage` / `OAuthToken` fake OAuth structs and functions
- The unused `log_request`, `log_error`, `log_tool_execution` helpers
- The unused `AtomicBool` import
- All hardcoded static data in the tool functions

---

## Env Vars Required (already in Chris's .env)

```
RENDER_API_KEY=
VERCEL_TOKEN=
BUFFER_API_KEY=
FIREBASE_PROJECT_ID=
FIREBASE_API_KEY=
```

Add `.env.example` with these keys (no values) for the repo.

---

## How to Test It Works

Once built, Claude Code can connect via:
```json
// In .claude/mcp.json or settings.json:
{
  "mcpServers": {
    "agentic-rust-mcp": {
      "command": "cargo",
      "args": ["run", "--manifest-path", "C:\\Users\\albat\\Desktop\\agentic-rust-mcp\\Cargo.toml"],
      "env": {}
    }
  }
}
```

Then in Claude Code: `/mcp` should show the three tools available.

---

## Success Criteria

- [ ] `cargo build` succeeds
- [ ] Claude Code `/mcp` lists `agency_pulse`, `content_check`, `data_vault`
- [ ] Calling `agency_pulse` returns LIVE Render/Vercel status (not hardcoded)
- [ ] Calling `content_check` returns LIVE Buffer queue
- [ ] `cargo test` passes with real integration tests
- [ ] `REBUILD_SPEC.md` deleted after completion
