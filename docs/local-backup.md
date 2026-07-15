# Local Backup — Obsidian Vault

## Purpose
Local fallback when OB1/Supabase is unreachable. Also the source that feeds NotebookLM via Google Drive sync.

## Location
`C:\Users\albat\Documents\Universal Brain Vault\`

## Key Folders
| Folder | Contents |
|--------|---------|
| `Agent Memory/session_logs/` | Per-session logs (YYYY-MM-DD-session.md) |
| `Research/summaries/` | NotebookLM exports |
| `Research/scripts/` | Generated video scripts |
| `Rules/` | Translation, tone, brand rules |

## Sync Flow
Obsidian Vault → Google Drive → NotebookLM (auto-ingests from Drive)

## When to Use
- OB1/Supabase is down
- Need to reference older session logs not in recent thoughts
- Research sources need to be added to NotebookLM
