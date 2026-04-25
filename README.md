# CBOR-Web

[![License: CC BY 4.0](https://img.shields.io/badge/License-CC%20BY%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by/4.0/)
[![Status](https://img.shields.io/badge/status-production%20%7C%2038%20sites-brightgreen.svg)]()
[![npm](https://img.shields.io/npm/v/%40deltopide_edy%2Fcborweb?label=npm)](https://www.npmjs.com/package/@deltopide_edy/cborweb)

[![Rust](https://img.shields.io/badge/Rust-1.93-000000?logo=rust&logoColor=white)]()
[![TypeScript](https://img.shields.io/badge/TypeScript-5.x-3178C6?logo=typescript&logoColor=white)](https://www.npmjs.com/package/@deltopide_edy/cborweb)
[![React](https://img.shields.io/badge/React-18-61DAFB?logo=react&logoColor=black)](clients/react/)
[![Python](https://img.shields.io/badge/Python-3.9%2B-3776AB?logo=python&logoColor=white)](clients/python/)
[![Go](https://img.shields.io/badge/Go-1.21%2B-00ADD8?logo=go&logoColor=white)](clients/go/cborweb.go)
[![C++](https://img.shields.io/badge/C%2B%2B-17-00599C?logo=c%2B%2B&logoColor=white)](clients/cpp/cborweb.hpp)
[![PHP](https://img.shields.io/badge/PHP-8.0%2B-777BB4?logo=php&logoColor=white)](clients/php/CborWeb.php)
[![Ruby](https://img.shields.io/badge/Ruby-3.0%2B-CC342D?logo=ruby&logoColor=white)](clients/ruby/cborweb.rb)
[![Java](https://img.shields.io/badge/Java-11%2B-ED8B00?logo=openjdk&logoColor=white)](clients/java/CborWebClient.java)
[![Cloudflare](https://img.shields.io/badge/edge-Cloudflare-F38020?logo=cloudflare&logoColor=white)](https://cbor-web.explodev.workers.dev)
[![IETF](https://img.shields.io/badge/spec-IETF%20RFC%208949-000?logo=ietf&logoColor=white)](spec/draft-plot-cbor-web-00.md)

> **The open protocol that makes websites readable by AI — 10x faster, at 1/100th the cost.**

---

## The opportunity

Every day, millions of AI agents — ChatGPT, Claude, Gemini, Perplexity, enterprise RAG pipelines — crawl the web to extract information. They all face the same problem: **HTML was designed for humans in 1993.**

| HTML today | CBOR-Web |
|---|---|
| Downloads 1.6 MB of menus, scripts, ads, trackers | Reads 50 KB of pure structured content |
| 93% of what an agent downloads is noise | 95% signal ratio |
| Costs $3 to crawl a 100-page site | Costs $0.01 |
| No standard way for sites to talk to machines | Open protocol, IETF standard (RFC 8949) |

**The web has two clients now. Only one of them is served.**

---

## What CBOR-Web does

CBOR-Web lets any website publish a **machine-readable copy** of its content — in parallel with its existing HTML. Humans see the regular site. AI agents get a binary, structured, 10x smaller version.

It's like having a **REST API for your content**, automatically generated from your HTML. No CMS change. No redesign. One command.

```bash
text2cbor --input ./my-site --output ./cbor --domain mysite.com
```

---

## Why this matters now

| Signal | Source |
|---|---|
| **AI agent market** | $150B+ by 2027. Every agent needs web content. |
| **LLM token costs** | Dropping but still $3-15 per 1M tokens. Crawling 80 pages = $2.88. In CBOR-Web = $0.02. |
| **Google A2A** | Google is building agent-to-agent protocols. CBOR-Web is the content layer they'll need. |
| **EU AI Act** | Mandates transparency for AI training data. CBOR-Web provides clean, auditable content signatures. |

---

## Trusted by production infrastructure

| Metric | Value |
|---|---|
| **Sites live** | 38 domains, 59 pages across 3 languages |
| **Edge CDN** | Cloudflare global network (300+ cities) |
| **SDK languages** | 8 (TypeScript, React, Python, Go, PHP, Ruby, Java, C++) |
| **IETF alignment** | Built on RFC 8949 (CBOR), RFC 8610 (CDDL), RFC 8615 (Well-Known URIs) |
| **npm package** | [`@deltopide_edy/cborweb`](https://www.npmjs.com/package/@deltopide_edy/cborweb) — zero dependencies |

---

## For publishers

Your site already has an HTML version. Adding CBOR-Web takes **30 minutes**:

1. Install the converter: `cargo install text2cbor`
2. Point it at your HTML: `text2cbor generate -i ./html -o ./cbor -d mysite.com`
3. Serve the CBOR directory alongside your HTML

**Benefits:**
- AI agents **prefer** your site over competitors (lower cost = more crawls)
- You get **analytics** on which agents visit, what they read
- You control **access levels** — public content vs premium content
- Zero impact on your existing site. Humans see zero difference.

---

## For AI developers

```bash
npm install @deltopide_edy/cborweb
```

```ts
import { CBORWebClient } from "@deltopide_edy/cborweb";

const site = new CBORWebClient("https://cbor.deltopide.com");

// Read the entire site in one request
const manifest = await site.manifest();
// → { name: "Deltopide", pages: 49, languages: ["fr", "en", "es"] }

// Or fetch a single page — already structured
const about = await site.page("/a-propos");
// → { title: "À propos", blocks: [{type: "h", text: "..."}, {type: "p", text: "..."}] }
```

Same API available in **Python, Go, PHP, Ruby, Java, C++** — zero dependencies.

---

## Comparison

| | robots.txt | sitemap.xml | llms.txt | Schema.org | **CBOR-Web** |
|---|---|---|---|---|---|
| Content delivered | No | No | Summary | Metadata | **Full structured content** |
| Binary efficiency | — | — | — | — | **Zero tokenization cost** |
| Access control | No | No | No | No | **Token-gated premium** |
| Agent analytics | No | No | No | No | **Doléance feedback protocol** |
| Edge delivery | — | — | — | — | **Cloudflare CDN** |
| Multi-language | — | — | — | — | **Per-page, native** |

CBOR-Web doesn't replace these standards — it's the **final layer** that gives AI agents what they actually need: the content.

---

## Architecture overview

```
Your HTML site ──→ text2cbor ──→ .cbor files
                                      │
                                      ▼
AI Agent ──→ Cloudflare Edge ──→ cbor-server ──→ Structured binary content
              (300+ cities)        (Rust/axum)      (CBOR, RFC 8949)
```

The full specification is documented in [CBOR-WEB-CORE.md](CBOR-WEB-CORE.md) and companion documents.

---

## Get started

### For publishers
```bash
git clone https://github.com/ploteddie-bit/cbor-web.git
cd cbor-web/tools/text2cbor
cargo build --release
./target/release/text2cbor generate -i ./my-html-site -o ./cbor-output -d mysite.com
```

### For developers
```bash
npm install @deltopide_edy/cborweb
```
Or pick your language: [Python](clients/python/) · [Go](clients/go/) · [PHP](clients/php/) · [Ruby](clients/ruby/) · [Java](clients/java/) · [C++](clients/cpp/) · [React](clients/react/)

### See it live
- **Dashboard**: [cbor-web.explodev.workers.dev](https://cbor-web.explodev.workers.dev) — 38 sites, live manifest
- **Spec**: [draft-plot-cbor-web-00.md](spec/draft-plot-cbor-web-00.md) — IETF Internet-Draft

---

## Roadmap

- [x] Core protocol & reference implementation (Rust)
- [x] 8-language SDK ecosystem
- [x] Cloudflare global edge deployment
- [x] 38-site production proving ground
- [x] IETF Internet-Draft submission
- [x] npm package publication
- [ ] WordPress plugin (43% of the web)
- [ ] Shopify app (e-commerce integration)
- [ ] IETF Working Group adoption

---

## License

Specification: [CC BY 4.0](LICENSE) — open standard, attribution required.  
Reference implementation: MIT — use freely, contribute back.

---

*"The web has two clients. It's time to serve both."*

**CBOR-Web** — ExploDev / Deltopide SL — 2026
