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
- Live demo: https://agentic-rust-mcp-demo.onrender.com (Render, free tier, `DEMO_MODE=true`, no real credentials on that service)
- Phase: stdio MCP server working as before; public web demo (stages 1+2) built, security-scanned, code-reviewed, locally verified, and deployed live.
- Not yet wired into Omni Dashboard as a panel.

## 2026-08-16 (TIC 1) — Registration + README hero, stage 5

**Registered as a live MCP server:** built a release binary (`cargo build --release`), added `agentic-rust-mcp` to `~/.claude.json`'s global `mcpServers` block (`command` points at the release exe directly, `env` carries `GMAIL_USER`/`GMAIL_APP_PASSWORD`/`RENDER_API_KEY` — the 3 real credentials so far). Smoke-tested with the exact env the config passes: `initialize` handshake succeeded, and `agency_pulse` made a **real** call to the live Render API and got back `"status":"deploying"` — genuine data, not a fixture. Needs a Claude Code restart to actually load (config changes aren't picked up mid-session, same as every other custom MCP server here).

**README hero:** replaced the generic decorative capsule-render banner with a real screenshot of the live deployed demo page (`docs/images/demo-hero.png`), plus a prominent Live Demo link/button right under it.

**Open:** Vercel/Buffer keys still needed for `agency_pulse`'s Vercel half and all of `content_check` to go live (Chris's to get). Firebase-vs-Supabase for `data_vault`'s real backend is an open question — not yet decided.

## 2026-08-16 (TIC 1) — Visual redesign, stage 4

**What shipped:** Rebuilt `static/index.html` from a plain functional page into a real designed surface, referenced against morphllm.com per Chris's brief (black ground, lime-green accent, dot-matrix "Silkscreen" display font, pill buttons, orbiting-node hero visual). Added a signature interaction: the hero orbit runs slow and green at idle, and accelerates + flares Rust-orange (`#CE422B`→`#F74C00`, the actual Rust-lang logo colors) the instant a tool is called, relaxing back to green when the response lands — dramatizing "Rust is fast" through motion tied to a real network call, not decoration. Added an explainer section (3 numbered cards: register it / agent calls a tool / real APIs real answer) after Chris flagged the page jumped straight to buttons with zero context for a first-time visitor.

**Process note:** brief was fully pinned by the product owner (exact palette, motif, and signature interaction specified directly), so the full impeccable concept-seed/decision-page ceremony was skipped as unnecessary — built directly per "a brief-pinned direction beats the roll." Self-QA only (screenshots at desktop + mobile widths via chrome-cdp-ex, the color-shift interaction verified via forced CSS injection since the real network round-trip is too fast to catch on a manual screenshot), not the full subagent finish-reviewer/documenter pipeline — a deliberate scope call given this is a single static page, not a multi-surface product.

**Detector:** `impeccable/scripts/detect.mjs` ran in degraded mode (missing optional npm parser deps) and flagged one "glowing shadow" warning — kept intentionally, since the glow is the exact effect from the referenced morphllm.com design, not accidental AI-slop.

**Fixed during mobile QA:** the nav wrapped "agentic-rust-mcp" awkwardly next to the GitHub link at narrow widths — added a `@media (max-width: 480px)` rule to wrap the nav onto two rows instead.

**Tests:** 5/5 passing (no Rust logic changed, HTML is compiled in via `include_str!`). Not yet deployed to Render as of this entry — pushing next.

## 2026-08-16 (TIC 1) — Docs accuracy pass, stage 3

**What shipped:** Rewrote `README.md` — it previously claimed OAuth 2.1, 3 MCP resources, 3 reusable prompts, and a streaming task type, none of which exist anywhere in the source (confirmed via grep: zero matches for any of it). It also had a fabricated-looking performance table with suspiciously precise numbers never actually measured. Removed all of it; README now describes the real 4 tools, the real two-entry-point architecture (stdio + web demo), and links the live Render demo. Also fixed `docs/rust-mcp-integration.md` (part of the broader RAG-memory-architecture doc set), which had the same "OAuth/Logging complete" claim and described a never-built Gmail-API-draft-generation plan as "in progress" — corrected to describe what's actually shipped (direct SMTP send via `lettre`), confirmed as the intended design going forward, not an unfinished stand-in. Added the missing `GMAIL_USER`/`GMAIL_APP_PASSWORD`/`DEMO_MODE` entries to `.env.example`, which didn't have them despite both being required for real functionality.

**Why this mattered beyond tidiness:** the README's false claims were independently confirmed to actively mislead an external reviewer (ChatGPT, asked to review this repo for portfolio purposes, reported "OAuth 2.1", "Resources + prompts + tools", and "Streaming/long-running tasks" as things it observed in the repo — all of it lifted from the README's own overclaiming rather than the actual source, which it hadn't read yet). A dishonest README is worse than a modest one for exactly this reason.

**Not code:** no rebuild/retest/security-scan needed for this pass — docs-only change.

## 2026-08-16 (TIC 1) — Public web demo, stage 2

**What shipped:** Removed the other 2 unused dependencies (`schemars`, `uuid` — confirmed zero references anywhere in `src/`). Added `configuration_error` as a distinct `agency_pulse` status (previously a missing `RENDER_API_KEY`/`VERCEL_TOKEN` was indistinguishable from a genuine "deploying" state). Unified `send_gmail`'s request parsing into one shared typed `SendGmailRequest` struct (`src/tools.rs`), used by both `main.rs` (stdio JSON-RPC) and `web_server.rs` (HTTP) instead of each having its own untyped/duplicated parsing.

**Security scan:** clean, 0 findings, after this batch and again after the fix below.

**Code review (correctness):** found a real regression from the `send_gmail` typing change — consolidating three independent field parses into one atomic `serde_json::from_value(...).ok()` meant a type error on `subject` or `body` alone (e.g. `subject` sent as a number) discarded a validly-supplied `to` too, producing a misleading "Missing required parameter: to" error instead of reporting what was actually wrong. Fixed: malformed arguments now return their own `"Invalid arguments: <reason>"` error, distinct from a genuinely missing `to`. Verified both cases directly against the built binary — confirmed the exact regression scenario the review named, and confirmed it's fixed.

**Tests:** 5/5 passing. Live-verified: `DEMO_MODE=false` still hard-exits with the fatal message and no lingering process; `DEMO_MODE=true` serves normally; malformed `subject` and genuinely-missing `to` now report distinct, accurate errors on the stdio path.

**Deployed:** new Render web service `agentic-rust-mcp-demo` (free tier, Oregon, Rust native runtime, `DEMO_MODE=true`, no other secrets set). Real gotcha hit during deploy: changing the service's `health-check-path` via the Render CLI while its just-triggered initial deploy was still in flight hung that deploy for ~14 minutes before it timed out; a second deploy that started after the config had settled went live normally in ~2.5 minutes. Logged in `start-to-finish`'s `LEARNINGS.md` for next time. Verified live via curl: `/health`, `/`, and `/api/agency_pulse` all responding correctly from the actual deployed URL.

**Not done yet:** README accuracy pass (still describes itself as "production-grade" and lists a non-existent "streaming" stage in older copies); `content_check`/`data_vault` don't yet have the same configuration-error-vs-empty distinction as `agency_pulse` — deferred because it requires an additive wire-format change (wrapping `data_vault`'s bare array in an object) that's a judgment call, not flagged as decided.

## 2026-08-16 (TIC 1) — Public web demo, stage 1

**What shipped:** Extracted the 4 tool functions into shared `src/tools.rs` (previously duplicated only in `main.rs`), added `DEMO_MODE` fixture-data support to all 4, and added `src/bin/web_server.rs` (Axum) + `static/index.html` as a new public HTTP demo surface. Removed the unused `rmcp` dependency; bumped `openssl` for a CVE.

**Security scan (vuln-hunter scan_diff):** initial run found 2 real dependency CVEs — `rmcp` 0.1.5 (CVE-2026-42559, high) and `openssl` 0.10.79 (CVE-2026-45784, low). Fixed by removing `rmcp` (confirmed unused in source) and bumping `openssl` to 0.10.81. Re-scan: clean, 0 findings.

**Code review (correctness):** found the public web server only logged a warning if `DEMO_MODE` wasn't set, instead of refusing to serve — meaning a misconfigured deploy would silently expose real Firestore lead PII, real Render/Vercel/Buffer status, and real Gmail sending to anonymous internet traffic. Fixed: `web_server.rs` now hard-exits (code 1) at startup if `DEMO_MODE` isn't `true`. Also flagged this BUILDLOG entry as missing — this entry is that fix.

**Simplification pass (ponytail-review):** one finding — 4 repeated `match result { Ok/Err }` blocks in `web_server.rs`'s route handlers, collapsed into one generic `respond<T>()` helper (net -8 lines).

**Tests:** 5/5 passing (`cargo test`) — including a new test proving `DEMO_MODE` short-circuits every tool before any real API call. Live-smoke-tested via curl: all 4 `/api/*` routes, `/health`, the static page, the missing-`to` validation error, and the DEMO_MODE-required startup gate (confirmed exits 1 without it, serves normally with it).

**Not done yet:** deploy to Render, BUILDLOG entry for the deploy itself once live, stage 2 items (typed request/response contracts, dead-dependency cleanup for `schemars`/`uuid`, README accuracy pass).

## Pending
- [x] Deploy `web_server` to Render with `DEMO_MODE=true` and no other secrets set on that service — live at https://agentic-rust-mcp-demo.onrender.com
- [ ] README accuracy pass
- [ ] `content_check`/`data_vault` configuration-error distinction (needs a wire-format decision — see stage 2 entry)
- [ ] Wire into Omni Dashboard as a status panel
- [ ] Confirm all 4 stages functional end-to-end against real accounts (stdio path)
