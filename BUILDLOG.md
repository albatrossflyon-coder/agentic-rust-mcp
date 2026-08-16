# Build Log — agentic-rust-mcp

MCP server built in Rust, plus a public HTTP demo wrapper.

## What This Is
A Model Context Protocol (MCP) server written in Rust. Exposes tools that AI agents can call to run automated workflows, plus a separate public-facing web demo so a recruiter/employer can test-drive it from a browser without an MCP client.

## Architecture
- `src/tools.rs` — the 4 tool implementations, shared by both entry points below. Supports `DEMO_MODE` (env var): when set, every tool returns realistic fixture data and never calls a real external account.
  - `agency_pulse` — Render + Vercel deployment status
  - `content_check` — Buffer scheduled posts
  - `data_vault` — Firestore leads
  - `send_gmail_tool` — sends via Gmail SMTP (real mode) or simulates (demo mode, never sends)
- `src/main.rs` — the original stdio MCP server (JSON-RPC 2.0 over stdin/stdout), for real MCP clients (Claude Desktop/Code) with real credentials. Unchanged in behavior.
- `src/bin/web_server.rs` — new Axum HTTP server exposing the same 4 tools as `POST /api/*`, plus `GET /` (serves `static/index.html`, a 4-button demo page) and `GET /health`. **Hard-refuses to start unless `DEMO_MODE=true`** — this binary is public-facing with no auth, so it must never be able to reach real accounts.

## Status
- GitHub: github.com/albatrossflyon-coder/agentic-rust-mcp
- Local: C:\Repos\agentic-rust-mcp
- Phase: stdio MCP server working as before; public web demo (stage 1) built, security-scanned, code-reviewed, and locally verified — not yet deployed to Render.
- Not yet wired into Omni Dashboard as a panel.

## 2026-08-16 (TIC 1) — Public web demo, stage 1

**What shipped:** Extracted the 4 tool functions into shared `src/tools.rs` (previously duplicated only in `main.rs`), added `DEMO_MODE` fixture-data support to all 4, and added `src/bin/web_server.rs` (Axum) + `static/index.html` as a new public HTTP demo surface. Removed the unused `rmcp` dependency; bumped `openssl` for a CVE.

**Security scan (vuln-hunter scan_diff):** initial run found 2 real dependency CVEs — `rmcp` 0.1.5 (CVE-2026-42559, high) and `openssl` 0.10.79 (CVE-2026-45784, low). Fixed by removing `rmcp` (confirmed unused in source) and bumping `openssl` to 0.10.81. Re-scan: clean, 0 findings.

**Code review (correctness):** found the public web server only logged a warning if `DEMO_MODE` wasn't set, instead of refusing to serve — meaning a misconfigured deploy would silently expose real Firestore lead PII, real Render/Vercel/Buffer status, and real Gmail sending to anonymous internet traffic. Fixed: `web_server.rs` now hard-exits (code 1) at startup if `DEMO_MODE` isn't `true`. Also flagged this BUILDLOG entry as missing — this entry is that fix.

**Simplification pass (ponytail-review):** one finding — 4 repeated `match result { Ok/Err }` blocks in `web_server.rs`'s route handlers, collapsed into one generic `respond<T>()` helper (net -8 lines).

**Tests:** 5/5 passing (`cargo test`) — including a new test proving `DEMO_MODE` short-circuits every tool before any real API call. Live-smoke-tested via curl: all 4 `/api/*` routes, `/health`, the static page, the missing-`to` validation error, and the DEMO_MODE-required startup gate (confirmed exits 1 without it, serves normally with it).

**Not done yet:** deploy to Render, BUILDLOG entry for the deploy itself once live, stage 2 items (typed request/response contracts, dead-dependency cleanup for `schemars`/`uuid`, README accuracy pass).

## Pending
- [ ] Deploy `web_server` to Render with `DEMO_MODE=true` and no other secrets set on that service
- [ ] Wire into Omni Dashboard as a status panel
- [ ] Confirm all 4 stages functional end-to-end against real accounts (stdio path)
