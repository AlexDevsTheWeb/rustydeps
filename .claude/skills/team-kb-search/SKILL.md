---
name: team-kb-search
description: "Search the team's knowledge base through its MCP server and answer with cited sources. Use for questions about project behavior, architecture, decisions, conventions, or history, and before answering from the local wiki."
---

# Team knowledge-base search

Query the team's centralized knowledge base through its MCP server. This is a read-only workflow: it retrieves and cites knowledge and never writes to the KB, the wiki, or the code repository.

## Prerequisites

- The team KB MCP server is configured for this runtime. Its server name is `<KB_MCP_SERVER>` and its search tool is `<KB_SEARCH_TOOL>`; confirm both from the runtime's MCP configuration before use.

## Workflow

1. Turn the question into one or more focused queries (domain terms, component names, decision topics). Prefer several narrow queries over one broad query.
2. Call `<KB_SEARCH_TOOL>` for each query. Use only the parameters the tool documents; do not invent filters.
3. Treat an empty, off-topic, or low-relevance result as no evidence. Refine the query once before concluding that nothing relevant exists.
4. If the MCP server does not respond (not configured, connection refused, timeout, or repeated errors), stop searching the KB and say so. Answer from the local wiki instead: read `wiki/index.md`, then the relevant pages in full. The wiki is assumed to exist in this fallback path. Mark the answer as wiki-only.
5. When the answer depends on current behavior, verify it against the live code. If the KB or the wiki disagrees with the code, the code is the current truth; say so explicitly.
6. Cite each claim: the title and source URI returned by the KB tool, or the `[[page-name]]` of a wiki page.

## Safeguards

- Never print, echo, or log credentials, tokens, or MCP configuration secrets.
- Do not call any write, ingest, update, or delete operation the MCP server exposes.
- Do not ingest, update, or delete KB content. Hand off approved knowledge with `handoff`, and record project documentation with `wiki-scribe` only when the user asks.

## Response format

Give the answer, list the sources used (title and URI, or wiki page), note any disagreement between KB, wiki, and code, and state explicitly when no source was relevant or when the answer came from the wiki fallback.
