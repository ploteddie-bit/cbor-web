# CBOR-Web

[![License: CC BY 4.0](https://img.shields.io/badge/License-CC%20BY%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by/4.0/)
[![Status: Draft](https://img.shields.io/badge/Status-Draft%20%E2%80%94%20Internal%20Review-orange.svg)]()
[![Version](https://img.shields.io/badge/Version-v2.1.3-blue.svg)]()

**Binary Web Content for Autonomous AI Agents — CBOR (RFC 8949)**

## What Is CBOR-Web?

CBOR-Web is a binary format standard that lets websites expose a machine-native copy of their content alongside HTML. AI agents consume structured content directly — no HTML parsing, no DOM traversal, no token waste.

A typical web page: **120 KB HTML → 7% useful content**.
A CBOR-Web page: **2 KB binary → 95%+ signal, zero tokenization cost**.

```cbor-diag
55799({
  0: "cbor-web-manifest",
  1: 2,
  2: {
    "name": "Example",
    "domain": "example.com",
    "languages": ["en"],
    "default_language": "en"
  },
  3: [{"path": "/", "title": "Home", "lang": "en", "access": "public", "size": 95}],
  5: {"total_pages": 1, "total_size": 95, "generated_at": 1(1742515200), "bundle_available": false}
})
```

## The Problem

| | HTML (traditional) | CBOR-Web |
|---|---|---|
| Format | Text (DOM, scripts, styles) | Binary (CBOR, RFC 8949) |
| Signal ratio | ~7% useful content | 95%+ signal |
| Agent access | Crawl + parse + strip noise | Single request, native parse |
| 80-page site | ~3 MB (multiple requests) | ~50 KB (one bundle) |
| Compression vs HTML | Baseline | **~10:1 to 50:1** (content only) |

## Key Features

- **Binary CBOR format** (RFC 8949) — compact, typed, deterministic encoding
- **Single-request indexing** — bundle endpoint delivers entire site in one request
- **SHA-256 incremental updates** — only re-download pages that changed
- **13+ content block types** — headings, paragraphs, tables, lists, code, images, CTAs, multimedia, generative...
- **Schema.org structured data** — native CBOR, not JSON-LD-inside-script-inside-HTML
- **Forward compatible** — unknown keys are ignored, not errors
- **Doléance Protocol** — agents feed back content quality to publishers, enabling co-evolutionary improvement
- **Access control** — ERC-20 token badge for premium content (optional)

## Specification Documents

| Document | Version | Description |
|----------|---------|-------------|
| [CBOR-WEB-CORE.md](CBOR-WEB-CORE.md) | v2.1.3 | Core format: manifest, page, bundle, content blocks, CDDL schema |
| [CBOR-WEB-SECURITY.md](CBOR-WEB-SECURITY.md) | v2.1.1 | COSE signatures, security levels, access control, rate limiting |
| [CBOR-WEB-MULTIMEDIA.md](CBOR-WEB-MULTIMEDIA.md) | v2.1 | Image variants, audio, video, media channels |
| [CBOR-WEB-GENERATIVE.md](CBOR-WEB-GENERATIVE.md) | v2.1 | Generative blocks, form handling, commerce data, A/B variants |
| [CBOR-WEB-ECONOMICS.md](CBOR-WEB-ECONOMICS.md) | v2.1 | Token economics, smart contracts, financial model |
| [CBOR-WEB-REFERENCE.md](CBOR-WEB-REFERENCE.md) | v2.1 | Unified CDDL, all test vectors, glossary, changelog |
| [CBOR-WEB-DOLEANCE.md](CBOR-WEB-DOLEANCE.md) | v1.0 | Agent feedback protocol: quality signals, privacy-first |

## Quick Start

1. **Clone** this repository and read [CBOR-WEB-CORE.md](CBOR-WEB-CORE.md)
2. **Validate** schemas with [CDDL files](schemas/) using `cddl-rs` or any CDDL validator
3. **Implement** — use the test vectors in CBOR-WEB-REFERENCE.md to verify conformance

```bash
git clone https://github.com/ploteddie-bit/cbor-web.git
cd cbor-web
# Read the core specification first
# Validate CDDL schemas
cargo install cddl-rs && cddl validate schemas/cbor-web-core.cddl
```

## Alternatives Considered

| Feature | robots.txt | sitemap.xml | llms.txt | Schema.org | **CBOR-Web** |
|---------|-----------|-------------|----------|------------|-------------|
| Format | Text | XML | Markdown | JSON-LD | **Binary (CBOR)** |
| Full content | No | No | Summary | No | **Yes** |
| Structured data | No | No | No | Yes | **Yes (native)** |
| Incremental updates | No | lastmod | No | No | **SHA-256 + diffs** |
| Single-request bundle | No | No | No | No | **Yes** |
| Access control | No | No | No | No | **Yes (optional)** |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on submitting issues, pull requests, and review criteria.

## First Deployment

[verdetao.com](https://verdetao.com) — the first CBOR-Web enabled website in the world.

## License

[Creative Commons Attribution 4.0 International (CC BY 4.0)](LICENSE)

---

*"The web has two clients: humans and machines. It's time to serve both."*
