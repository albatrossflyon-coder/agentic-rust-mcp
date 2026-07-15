# Session Memory — OB1 + Supabase

## Purpose
Stores live session state — what was built, what decisions were made, what's pending. The primary memory layer all agents read at session start (GSM protocol).

## Structure
- **OB1-Brain/thoughts/** — numbered thought files (thought-NN.md), newest = most recent session
- **Supabase** — cloud-hosted, synced with OB1 thought files

## GSM Protocol (Get Session Memory)
1. Read the highest-numbered thought files from `G:\Other computers\My Laptop\Desktop\OB1-Brain\thoughts\`
2. Last 2-3 files = full current session state
3. Fallback if Supabase down: read Obsidian session logs

## Agent Mailbox
- Claude → Jim: `OB1-Brain/messages/to-jim.md`
- Jim → Claude: `OB1-Brain/messages/to-claude.md`
- Jim → Copilot: `OB1-Brain/messages/to-copilot.md`

## Session End Protocol
1. Write session wrap to new thought-NN.md in OB1-Brain/thoughts/
2. Each agent writes their own thought file — do NOT overwrite another agent's file
3. Commit OB1-Brain repo
