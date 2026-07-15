# RAG Integration — rag-system

## Repo
https://github.com/albatrossflyon-coder/rag-system

## Local Path
`G:\Other computers\My Laptop\Desktop\rag-system\`

## Purpose
Vector store and query layer over Albatross business context. Agents query this when they need structured, semantic search over the corpus.

## Corpus (data/corpus/)
- `albatross-income-streams.md` — revenue streams and business model
- `albatross-operating-context.md` — how Albatross AI operates
- `agent-org-and-workflow.md` — agent roles and workflows
- `employment-strategy-and-blockers.md` — job search strategy
- `remotejobisland-product-brief.md` — Remote Job Island product spec

## Stack
- Python-based RAG pipeline
- Local query via `query_local.py`
- API layer in `api/`
- Notebooks for exploration in `notebooks/`

## Status
Phase 1 complete. Integration with agentic-rust-mcp pending (Phase 3).
