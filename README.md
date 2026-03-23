# CBOR-Web

**Machine-Readable Binary Web Content for Autonomous Agents**

CBOR-Web is a binary format standard enabling websites to expose a machine-native copy of their content as a parallel channel alongside existing HTML. AI agents consume structured content directly — no HTML parsing, no DOM traversal, no token waste.

## Status

| Document | Version | Status |
|----------|---------|--------|
| **CBOR-WEB-CORE.md** | v2.1.3 | Draft — internal review |
| **CBOR-WEB-SECURITY.md** | v1.0 | Draft — internal review |
| **CBOR-WEB-DOLEANCE.md** | v1.0 | Draft — internal review |
| CBOR-WEB-MULTIMEDIA.md | — | Planned |
| **CBOR-WEB-GENERATIVE.md** | v2.1 | Draft — internal review |
| **CBOR-WEB-ECONOMICS.md** | v2.1 | Draft — internal review |
| **CBOR-WEB-REFERENCE.md** | v2.1 | Draft — internal review |

## What Problem Does This Solve?

A typical web page: 120 KB of HTML, 7% useful content, 1,400 DOM elements.
A CBOR-Web page: 2 KB of binary, 95%+ signal, zero tokenization cost.

For a 25-page site, an AI agent downloads one 50 KB bundle instead of crawling 3 MB of HTML.

## Key Features

- **Binary CBOR format** (RFC 8949) — compact, typed, deterministic
- **Single-request indexing** — bundle endpoint delivers entire site in one request
- **SHA-256 incremental updates** — only re-download pages that changed
- **13 content block types** — headings, paragraphs, tables, lists, code, images, CTAs...
- **Schema.org structured data** — native CBOR, not JSON-LD-inside-script-inside-HTML
- **Forward compatible** — unknown keys are ignored, not errors
- **Doléance Protocol** — agents feed back content quality to publishers, enabling co-evolutionary improvement

## First Deployment

[verdetao.com](https://verdetao.com) — the first CBOR-Web enabled website in the world.

## Author

**ExploDev** — Eddie Plot  
License: CC BY 4.0

---

*"The web has two clients: humans and machines. It's time to serve both."*
