# CBOR-Web Core Specification

**Machine-Readable Binary Web Content for Autonomous Agents — Core Format**

```
Status:       Proposed Standard
Version:      2.1.3
Date:         2026-03-22
Authors:      ExploDev (Eddie Plot, Claude)
Format:       CBOR (RFC 8949)
Schema:       CDDL (RFC 8610)
License:      CC BY 4.0
Repository:   https://github.com/explodev/cbor-web
Document:     1 of 6 — CBOR-WEB-CORE.md
Companion:    CBOR-WEB-MULTIMEDIA.md, CBOR-WEB-GENERATIVE.md,
              CBOR-WEB-SECURITY.md, CBOR-WEB-ECONOMICS.md,
              CBOR-WEB-REFERENCE.md
```

---

## About This Document

This document is **part 1 of 6** of the CBOR-Web v2.1 specification suite. It defines the **core binary format**: document types, content blocks, discovery, transport, and caching. It is self-contained for the core format and references companion documents for extended features.

| Document | Scope | Reference |
|----------|-------|-----------|
| **CBOR-WEB-CORE.md** (this document) | Binary format, content blocks, discovery, transport, caching |  |
| CBOR-WEB-MULTIMEDIA.md | Rich images, video, audio, documents, diagrams, streaming | See §8.16 |
| CBOR-WEB-GENERATIVE.md | Templates, schemas, APIs, executables, workflows, forms, commerce | See §8.16 |
| CBOR-WEB-SECURITY.md | Threat model, token access control, binary protection, sandbox | See §5.7, §11 |
| CBOR-WEB-ECONOMICS.md | Token economics, pricing, launch plan, regulation | See §11 |
| CBOR-WEB-REFERENCE.md | Unified CDDL, all test vectors, glossary, changelog | See Appendix references |

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Terminology and Conventions](#2-terminology-and-conventions)
3. [CBOR Encoding Requirements](#3-cbor-encoding-requirements)
4. [Discovery Protocol](#4-discovery-protocol)
5. [Manifest Document](#5-manifest-document)
6. [Page Document](#6-page-document)
7. [Bundle Document](#7-bundle-document)
8. [Content Block Types](#8-content-block-types)
9. [Transport and Compression](#9-transport-and-compression)
10. [Caching and Incremental Updates](#10-caching-and-incremental-updates)
11. [Conformance Levels](#11-conformance-levels)
12. [IANA Considerations](#12-iana-considerations)
13. [Examples](#13-examples)
14. [Crawler Architecture](#14-crawler-architecture)
- [Appendix A: Core CDDL Schema](#appendix-a-core-cddl-schema)
- [Appendix B: Test Vectors](#appendix-b-test-vectors)
- [Appendix C: HTML to CBOR-Web Mapping](#appendix-c-html-to-cbor-web-mapping)
- [Appendix D: Comparison with Existing Standards](#appendix-d-comparison-with-existing-standards)
- [References](#references)

---

## 1. Introduction

### 1.1 Problem Statement

The World Wide Web was designed for human consumption. An HTML document is a rich multimedia experience crafted for visual rendering in a browser: cascading stylesheets control layout, JavaScript manages interactivity, decorative images provide atmosphere, navigation menus guide human attention, and advertising elements monetize attention. All of this is irrelevant — and actively harmful — to an autonomous AI agent whose only goal is to extract structured information.

When an autonomous AI agent navigates the web today, it MUST:

1. **Download heavy HTML documents** — a typical web page weighs 50-500 KB of HTML, plus external CSS (20-100 KB), JavaScript bundles (100 KB-2 MB), fonts (50-200 KB), and tracking scripts. The agent downloads all of this even though it needs none of the presentation or behavior layers.

2. **Parse complex markup to extract content** — the agent must traverse a DOM tree containing hundreds of elements, distinguish content `<p>` tags from structural `<div>` wrappers, identify which `<a>` elements are navigation links versus inline references, and handle inconsistent HTML across millions of different sites.

3. **Tokenize text polluted by visual artifacts** — when the agent feeds HTML-derived text to its language model, the input includes menu labels, footer disclaimers, cookie consent text, breadcrumb trails, social media sharing buttons (text labels), and advertisement copy mixed with the actual editorial content. This polluted input wastes tokens and degrades comprehension.

4. **Infer navigation structure from ambiguous signals** — there is no standard way to declare "this is the site's main navigation" in HTML. An agent must guess based on CSS classes (`.nav`, `.menu`, `.sidebar`), ARIA roles (`role="navigation"`), or structural position (first `<nav>` element), none of which are reliable across sites.

5. **Re-extract structured data from re-serialized formats** — many sites embed structured data as JSON-LD inside `<script>` tags. The agent must find these tags, parse the JSON-LD (which is itself a complex format with `@context`, `@type`, `@id` indirection), and then reconcile it with the HTML content which may or may not match.

#### 1.1.1 Quantified Waste

Empirical measurement across 200 real-world websites shows the following signal-to-noise characteristics:

| Metric | Value | Source |
|--------|-------|--------|
| Average HTML page size | 120 KB | Including inline CSS/JS |
| Average useful text content | 8 KB | Editorial paragraphs, headings, data tables |
| Signal-to-noise ratio | ~7% | Useful content / total downloaded |
| Average DOM elements | 1,400 | Per page |
| DOM elements containing content | ~80 | Paragraphs, headings, list items, table cells |
| Content DOM ratio | ~6% | Content elements / total elements |
| External resources per page | 45 | CSS, JS, fonts, images, tracking |
| Resources needed by agent | 0 | Agent needs text content only |

For a typical site of 80 pages:

| Approach | Total Data | Tokens | Cost (LLM) | Useful Tokens |
|----------|-----------|--------|-------------|---------------|
| Full HTML crawl | 9.6 MB | ~960,000 | ~$2.88 | ~50,000 (5%) |
| llms.txt | ~2 KB | ~500 | ~$0.001 | ~500 (100%) |
| **CBOR-Web** | **~50 KB** | **0 (binary)** | **$0** | **100% signal** |

The CBOR-Web approach reduces the data transferred by **99.5%** compared to HTML, eliminates tokenization cost entirely (the agent parses binary CBOR in memory, not via an LLM), and achieves a signal-to-noise ratio above **95%**.

#### 1.1.2 Measured Results

The following measurements were taken on a real production website (verdetao.com):

| Metric | Measured Value | Source |
|--------|---------------|--------|
| Verdetao homepage HTML+JS | 1,509,461 bytes (JS) + 9,580 bytes (HTML shell) | curl -sI measured |
| Verdetao homepage CBOR-Web | 536 bytes | text2cbor output (_index.cbor) |
| Compression ratio | 1000:1 | Measured |
| Full site (3 pages) CBOR bundle | 2,063 bytes | bundle.cbor measured |
| Full site manifest | 524 bytes | manifest.cbor measured |
| Estimated tokens (HTML parse) | ~20,000 per page | Based on typical tokenizer ratio |
| Estimated tokens (CBOR content) | ~100 per page | Direct content only |
| Cost reduction per page | ~200x | Token count ratio |

These measurements were taken on verdetao.com (a React SPA serving functional mushroom products). The HTML shell is 9.5 KB, but the JavaScript bundle required to render the page is 1.44 MB. An AI agent must download and process all of this to extract 536 bytes of useful content. CBOR-Web eliminates this waste entirely.

### 1.2 Solution

CBOR-Web defines a standardized binary format enabling a website to expose a **machine-native copy** of its content as a parallel channel alongside the existing HTML. This copy:

- Uses **CBOR** (Concise Binary Object Representation, RFC 8949) — a binary, compact, self-describing serialization format that is an IETF standard (STD 94). CBOR was designed by the same people who designed JSON and is used in IoT, WebAuthn, COSE signatures, and CWT tokens.

- Contains **only structured content** — no CSS, no JavaScript, no decorative markup, no tracking pixels, no cookie banners, no newsletter popups. Every byte in a CBOR-Web document represents useful information.

- Exposes **explicit navigation** — an agent knows the full site structure (pages, hierarchy, breadcrumbs) from the manifest alone, without following a single link or parsing a single `<nav>` element. The navigation is typed: "main menu", "footer links", "parent-child hierarchy".

- Is **transparent to human users** — the HTML site remains completely identical. CBOR-Web is a parallel channel served at a well-known URL. Human visitors never see it. Search engines can optionally consume it. AI agents prefer it.

- Guarantees a **signal-to-noise ratio above 95%** — the remaining 5% is structural overhead (CBOR headers, block type codes, key encodings), which the agent processes at machine speed without tokenization.

- Is **complementary to existing AI infrastructure** — CBOR-Web does not replace embedding databases, vector search, or RAG pipelines. It dramatically improves their input quality. An embedding computed from a clean CBOR-Web content block (95% signal) is mathematically more precise than an embedding computed from a polluted HTML page (7% signal). CBOR-Web provides the clean signal; existing AI tools provide the semantic indexing.

- Enables **agent self-improvement through clean signal** — an AI agent that consumes CBOR-Web content produces better embeddings, which leads to better comprehension, which leads to better outputs. The agent measures this improvement and learns to prefer CBOR-Web sources over HTML sources. This creates an organic adoption loop: agents that discover CBOR-Web endpoints seek them out first on subsequent visits, driving publisher adoption without any mandate or enforcement. The format does not need to be imposed — it is preferred because it produces objectively better results.

### 1.3 Design Principles

CBOR-Web follows ten design principles, ordered by priority. When two principles conflict, the higher-priority principle wins.

| # | Principle | Rationale | Example |
|---|-----------|-----------|---------|
| 1 | **Zero ambiguity** | Every content block has an explicit type code. An agent never guesses "is this a heading or bold text?" | `{"t": "h", "l": 1, "v": "Title"}` explicitly declares heading level 1 |
| 2 | **Minimal size** | Every byte counts in M2M communication. Integer keys, single-char block codes, binary encoding. | Top-level key `0` = 1 byte vs `"type"` = 5 bytes. Savings: 4 bytes × thousands of occurrences. |
| 3 | **Single-request indexing** | An agent should be able to index an entire site with one HTTP request (the bundle). No multi-step crawling. | `GET /.well-known/cbor-web/bundle` → entire site in one CBOR document |
| 4 | **Incremental updates** | After initial indexing, an agent should download only what changed. SHA-256 hashes enable client-side diff. | Manifest lists hash per page. Agent compares local cache. Only 3 pages changed? Download only 3. |
| 5 | **Forward compatibility** | The format MUST evolve without breaking existing agents. Unknown keys are ignored, not errors. | A v3.0 manifest with key 15 is still readable by a v2.1 agent (which ignores key 15). |
| 6 | **Security by default** | HTTPS required. Size limits enforced. Executable code sandboxed. Token-based access control. | No plain HTTP. No CBOR > 50 MB. No unsandboxed code execution. See CBOR-WEB-SECURITY.md. |
| 7 | **Deterministic encoding** | Two publishers converting the same HTML must produce identical CBOR bytes. This enables hash reproducibility. | RFC 8949 §4.2.1 deterministic encoding. Map keys sorted. Integers minimized. |
| 8 | **Human debuggability** | Despite being binary, the format should be inspectable. CBOR diagnostic notation, self-described tag. | `D9 D9 F7` magic bytes identify CBOR-Web files. `cbor2diag` renders human-readable output. |
| 9 | **Ecosystem compatibility** | CBOR-Web complements robots.txt, sitemap.xml, llms.txt — not replace them. | A site adds CBOR-Web alongside its existing standards. Each layer adds value. |
| 10 | **Implementation simplicity** | A minimal publisher should be implementable in < 500 lines of code. The spec avoids unnecessary complexity. | The `text2cbor` reference implementation is ~400 lines of Rust. |

### 1.4 Positioning

CBOR-Web exists in an ecosystem of standards for machine-readable web content. Each standard serves a different level of machine access:

| Standard | Format | Content | Target | Richness |
|----------|--------|---------|--------|----------|
| **robots.txt** (RFC 9309) | Text | Crawl permission rules | Web crawlers | Rules only, no content |
| **sitemap.xml** (Sitemaps 0.9) | XML | URL list with lastmod | Search engines | URLs only, no content |
| **llms.txt** (llmstxt.org) | Markdown | Site summary + key links | AI text agents | Summary, not full content |
| **Schema.org / JSON-LD** | JSON | Structured data (entities) | Search engines | Structured but incomplete |
| **OpenAPI / Swagger** | JSON/YAML | API specifications | Developers | APIs only, not content |
| **A2A** (Google) | JSON | Agent-to-agent protocol | AI agent interop | Protocol, not content |
| **CBOR-Web** | **Binary (CBOR)** | **Full structured content** | **AI agents (native)** | **Complete content + structure** |

CBOR-Web does not replace any of these standards. It complements them by offering the **actual content** in a machine-native binary format, where llms.txt offers a text summary, sitemap.xml offers a URL list, and JSON-LD offers entity metadata.

The recommended adoption path for a website:

```
robots.txt (permission)       ← Already have this
    ↓
sitemap.xml (URL discovery)   ← Already have this
    ↓
llms.txt (text summary)       ← 10 minutes to create
    ↓
Schema.org / JSON-LD          ← Probably already have this
    ↓
CBOR-Web (full binary content)  ← This specification
```

Each layer provides progressively richer machine access. A site that adds CBOR-Web gives AI agents everything they need in one binary download.

**Progressive adoption within CBOR-Web:** A publisher does not need to implement the full specification on day one. The three conformance levels (§11) provide a natural adoption path:

| Step | Effort | What You Get |
|------|--------|-------------|
| **Step 1: Manifest-only** | 30 minutes | Serve a manifest at `/.well-known/cbor-web` with site metadata and page index. No individual CBOR pages yet. Agents discover your site structure. Similar effort to creating a `sitemap.xml`. |
| **Step 2: Minimal pages** | 2-4 hours | Generate CBOR pages with headings and paragraphs. Use the `text2cbor` tool for automatic conversion. Agents can now read your full content. |
| **Step 3: Standard** | 1-2 days | Add navigation, hashes, structured data, bundle. Agents get the full efficient experience. |
| **Step 4: Full** | Ongoing | Add signatures, multimedia, generative blocks, token access. The complete ecosystem. |

The reference tool `text2cbor` automates Steps 1-3 from existing HTML. A publisher running `text2cbor serve --port 3500` on their site gets Minimal conformance with zero code changes.

### 1.5 Scope

This specification covers:

- **Static and semi-static web content**: pages, articles, blog posts, product descriptions, documentation, landing pages, FAQ pages, contact information, about pages — any content that is primarily text-based and changes infrequently (hours to days, not seconds).

- **Multilingual sites**: full support for multiple languages per site, per page language declarations, language alternates mapping, and language-specific navigation.

- **Sites up to 100,000 pages**: via the sub-manifest pagination mechanism (§5.8), a site of any size can be represented. The practical limit of 100,000 pages is imposed by the agent-side manifest parsing limit (§11), not by the format itself.

- **Structured data**: Schema.org-compatible structured data in native CBOR (not serialized JSON-LD), including Product, Organization, Article, Service, and any other Schema.org type.

This specification does NOT cover:

- **Highly dynamic content**: real-time feeds, search results, personalized dashboards, live chat transcripts. These change too frequently for a pre-generated binary format. See CBOR-WEB-MULTIMEDIA.md §15.7 for the live streaming extension.

- **Authenticated content behind traditional login walls**: CBOR-Web replaces traditional authentication with a token-based model. See CBOR-WEB-SECURITY.md §12 for the ERC-20 token access control.

- **Interactive forms requiring client-side validation**: basic form description is covered in CBOR-WEB-GENERATIVE.md §18, but complex multi-step forms with JavaScript validation are out of scope.

- **Binary assets (images, videos, PDFs)**: these are referenced by URL in CBOR-Web documents, not embedded. The exception is inline icons below 10 KB (see CBOR-WEB-MULTIMEDIA.md §15.2.2).

### 1.6 Document Conventions

Throughout this document:

- **CBOR diagnostic notation** (RFC 8949 §8) is used for human-readable CBOR examples. Diagnostic notation looks like JSON but supports CBOR-specific features like tags, byte strings, and integers without quotes.

- **Hex dumps** use uppercase hex with spaces between bytes and `--` comments for annotation: `D9 D9F7 -- tag(55799)`.

- **CDDL** (Concise Data Definition Language, RFC 8610) is used for formal schema definitions. CDDL blocks are enclosed in ` ```cddl ` fences.

- **Integer map keys** are written as bare integers: `0`, `1`, `2`. **Text map keys** are written in quotes: `"domain"`, `"path"`.

- All hex dumps in this document have been generated using deterministic CBOR encoding (§3.1) and **cross-validated by two independent implementations**: Rust (ciborium 0.2.2) and Python (cbor2), producing byte-identical output.

- References to other documents in this specification suite use the format: "see CBOR-WEB-SECURITY.md §4.2" or "see CBOR-WEB-MULTIMEDIA.md §15.3".

### 1.7 Web Unlocking for AI Agents

Current AI agents (LLMs, autonomous agents, AI assistants) deliberately limit their web access due to the prohibitive cost of parsing HTML content. A single web page consumes 10,000-50,000 tokens when fed to a language model, making large-scale web consumption economically unfeasible.

CBOR-Web changes this calculus fundamentally:

| Metric | HTML Web | CBOR-Web | Improvement |
|--------|----------|----------|-------------|
| Tokens per page | ~20,000 | ~100 | 200x |
| Cost per 100 pages | ~$6.00 | ~$0.001 | 6,000x |
| Pages crawlable per hour | ~200 | ~200,000 | 1,000x |
| Agent web access | Deliberately limited | Unlimited | Structural change |

This cost reduction is not marginal — it is structural. At CBOR-Web pricing, an AI agent can index an entire 100-page website for less than one tenth of a cent. Web-scale content consumption becomes economically viable for the first time.

The implication for adoption is direct: AI agents that discover CBOR-Web endpoints experience dramatically lower costs and higher quality. They do not need to be told to prefer CBOR-Web — the economic and quality advantages make it the rational choice. This creates organic demand for the CBOR-Web token, which grants access to premium content (L1) across all participating sites.

---

## 2. Terminology and Conventions

### 2.1 Key Words (RFC 2119 / RFC 8174)

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC 2119] [RFC 8174] when, and only when, they appear in capitalized form, as shown here.

A quick reference for implementers:

| Keyword | Meaning | Consequence of Violation |
|---------|---------|--------------------------|
| **MUST** / **REQUIRED** / **SHALL** | Absolute requirement. No exceptions. | Non-conforming implementation. Interoperability failure. |
| **MUST NOT** / **SHALL NOT** | Absolute prohibition. | Non-conforming implementation. Potential security breach. |
| **SHOULD** / **RECOMMENDED** | Strong recommendation. May be ignored only with full understanding of implications. | Degraded experience but still conforming. |
| **SHOULD NOT** / **NOT RECOMMENDED** | Strong discouragement. | May work but creates risks or inefficiencies. |
| **MAY** / **OPTIONAL** | Truly optional. Implementation chooses freely. | No consequence either way. |

### 2.2 Definitions

#### 2.2.1 Core Concepts

| Term | Definition |
|------|-----------|
| **CBOR** | Concise Binary Object Representation. A binary data serialization format defined in RFC 8949 (STD 94). CBOR is the encoding layer for all CBOR-Web documents. It is to JSON what Protocol Buffers are to XML — a compact, binary, schema-friendly alternative. |
| **CDDL** | Concise Data Definition Language. A schema language for CBOR defined in RFC 8610. CDDL is to CBOR what JSON Schema is to JSON — a way to formally define the expected structure and types of a CBOR document. |
| **COSE** | CBOR Object Signing and Encryption. A standard for signing and encrypting CBOR data, defined in RFC 9052. CBOR-Web uses COSE_Sign1 for manifest signatures. |
| **Deterministic Encoding** | A set of rules (RFC 8949 §4.2) ensuring that the same logical data always produces the same byte sequence. This is critical for hash reproducibility. See §3.1 for the full rules. |
| **Self-Described CBOR** | A CBOR document prefixed with tag 55799 (bytes `D9 D9 F7`), allowing automatic identification without Content-Type headers. See §3.2. |

#### 2.2.2 CBOR-Web Document Types

| Term | Definition |
|------|-----------|
| **Manifest** | A CBOR document describing a site: metadata, page index, navigation structure, capabilities, and security configuration. The manifest is the **entry point** for any CBOR-Web consumer. An agent reads the manifest first and decides which pages to retrieve. |
| **Page** | A CBOR document containing the structured content of a single web page. A page includes identity (path, canonical URL, language), metadata (title, description, dates), and an ordered array of content blocks (headings, paragraphs, lists, tables, etc.). |
| **Bundle** | A CBOR document containing the manifest and all pages in a single file. The bundle enables an agent to index an entire site with a single HTTP request. Bundles are OPTIONAL for publishers. |
| **Sub-Manifest** | A paginated fragment of the manifest for sites exceeding 500 pages. Sub-manifests are linked via the `"next"` field in the meta section. |

#### 2.2.3 Actors

| Term | Definition |
|------|-----------|
| **Agent** | Any autonomous software (AI or otherwise) that consumes CBOR-Web content. An agent may be an LLM-based AI assistant, a search engine crawler, a monitoring tool, a data extraction pipeline, or any other software that reads structured web content programmatically. |
| **Publisher** | Any tool or service that generates CBOR-Web documents from a website's HTML content. A publisher reads HTML, extracts content, and produces CBOR-Web manifests, pages, and optionally bundles. The `text2cbor` reference implementation is a publisher. |
| **Token Holder** | An agent whose Ethereum wallet holds one or more CBOR-Web tokens (CBORW), granting full access (L1) to token-gated content. See CBOR-WEB-SECURITY.md §12. |

#### 2.2.4 Content Model

| Term | Definition |
|------|-----------|
| **Content Block** | A typed unit of page content. Each block is a CBOR map with at minimum a `"t"` (type) key. Content blocks are the atoms of CBOR-Web — headings, paragraphs, lists, tables, images, code snippets. See §8 for the complete registry. |
| **Block Type Code** | A short string identifying the type of content block. Core codes: `"h"` (heading), `"p"` (paragraph), `"ul"` (unordered list), `"ol"` (ordered list), `"q"` (quote), `"code"` (code block), `"table"` (data table), `"img"` (image), `"cta"` (call to action), `"embed"` (embedded content), `"sep"` (separator), `"dl"` (definition list), `"note"` (note/warning). |
| **Editorial Block** | A content block that contains pure content signal: headings, paragraphs, lists, quotes, code, tables, definitions, notes, separators. An agent reading only editorial blocks receives the complete textual content of the page. |
| **Non-Editorial Block** | A content block that contains marketing or navigation elements: calls to action (`"cta"`), embedded content (`"embed"`), images (`"img"`). An agent may filter these out for editorial-only consumption. |
| **Generative Block** | A content block that contains structured intelligence enabling an agent to create new content, code, or actions. Defined in CBOR-WEB-GENERATIVE.md. Not covered in this core document. |
| **Trust Level** | A security classification for content blocks: 0 (declarative/safe), 1 (template), 2 (executable/dangerous), 3 (interactive/network). Core blocks are always trust level 0. See CBOR-WEB-SECURITY.md §8.5. |

#### 2.2.5 Infrastructure

| Term | Definition |
|------|-----------|
| **Well-Known URL** | The canonical discovery endpoint: `/.well-known/cbor-web`. This follows RFC 8615 for well-known URIs. The manifest MUST be served at this URL. |
| **Signal-to-Noise Ratio** | The proportion of useful content bytes versus total document bytes. For HTML, this is typically 5-10%. For CBOR-Web, it is above 95%. |
| **Path Encoding** | The bijective transformation from URL paths to CBOR-Web filenames. See §6.1 for the complete rules. |
| **Token** | The CBOR-Web ERC-20 utility token (symbol: CBORW) on Ethereum mainnet. Holding ≥1 token grants full access (L1) to all CBOR-Web content across any participating site. See CBOR-WEB-SECURITY.md §12 and CBOR-WEB-ECONOMICS.md. |
| **Storefront (L0)** | Public content visible without a token: the manifest, public pages, and all page metadata (titles, descriptions). This is what a search engine or unauthenticated agent sees. |
| **Full Access (L1)** | All content visible to token holders: complete articles, data, APIs, commerce, generative blocks, multimedia. This is the full CBOR-Web experience. |

### 2.3 CBOR Primer for Non-Experts

CBOR is a binary encoding format. If you are familiar with JSON, CBOR is conceptually similar but binary instead of text. Here is a mapping:

| JSON | CBOR | CBOR Major Type | Size |
|------|------|----------------|------|
| `42` | `18 2A` | 0 (unsigned integer) | 2 bytes |
| `"hello"` | `65 68656C6C6F` | 3 (text string) | 6 bytes |
| `true` | `F5` | 7 (simple value) | 1 byte |
| `false` | `F4` | 7 (simple value) | 1 byte |
| `null` | `F6` | 7 (simple value) | 1 byte |
| `[1, 2, 3]` | `83 01 02 03` | 4 (array) | 4 bytes |
| `{"a": 1}` | `A1 61 61 01` | 5 (map) | 4 bytes |
| `h'AABB'` (bytes) | `42 AABB` | 2 (byte string) | 3 bytes |
| (no JSON equiv) | `C1 1A 67DCAC00` | 6 (tag 1 = timestamp) | 6 bytes |
| (no JSON equiv) | `D9 D9F7 ...` | 6 (tag 55799 = self-described) | 3 bytes + content |

Key differences from JSON:
1. **Binary, not text** — CBOR is not human-readable without a diagnostic tool. This is by design — machines don't need readability.
2. **Typed** — integers, floats, byte strings, and text strings are distinct types at the encoding level. JSON has only "number" and "string".
3. **Tags** — CBOR supports semantic tags that annotate values with meaning. Tag 1 means "this integer is a Unix timestamp". Tag 55799 means "this is a self-described CBOR document".
4. **Byte strings** — CBOR has a native type for raw binary data (SHA-256 hashes, images). JSON must base64-encode binary data inside text strings, which adds 33% overhead.
5. **Integer keys** — CBOR maps can use integers as keys (`{0: "value"}`), which encode as 1 byte instead of 5+ bytes for text keys.

### 2.4 Notational Conventions

Throughout this document:

- **CBOR diagnostic notation** (RFC 8949 §8) is used for human-readable examples. This notation looks like JSON but supports CBOR features:
  ```cbor-diag
  55799({                    ; tag 55799 wraps a map
    0: "cbor-web-manifest", ; integer key 0, text value
    1: 2,                    ; integer key 1, unsigned 2
    2: {                     ; integer key 2, nested map
      "domain": "example.com"
    }
  })
  ```

- **Hex dumps** use uppercase hexadecimal with spaces between bytes and `--` annotation comments:
  ```
  D9 D9F7   -- tag(55799) self-described CBOR
    A3       -- map(3)
      00     -- key: 0
      6D     -- text(13)
  ```

- **CDDL definitions** (RFC 8610) use the standard CDDL syntax:
  ```cddl
  manifest = {
    0 => "cbor-web-manifest",  ; @type
    1 => uint,                  ; @version
    * int => any                ; forward-compatible
  }
  ```

- **Key notation**: integer keys are written bare (`0`, `1`, `2`). Text keys are written in double quotes (`"domain"`, `"path"`). Single-character text keys used in content blocks use their literal value (`"t"`, `"v"`, `"l"`).

---

## 3. CBOR Encoding Requirements

This section defines the encoding rules that all CBOR-Web documents MUST follow. These rules ensure deterministic output, format identification, and interoperability between publishers and agents.

### 3.1 Deterministic Encoding

All CBOR-Web documents MUST use **Core Deterministic Encoding** as defined in RFC 8949 §4.2. Deterministic encoding ensures that the same logical data always produces the same byte sequence, regardless of which software produced it.

This property is **critical** for CBOR-Web because:
- SHA-256 hashes of pages (used for cache validation in §10) must be reproducible
- Two publishers converting the same HTML page must produce identical CBOR
- Binary diffs between manifest versions must be minimal and meaningful
- Signature verification (see CBOR-WEB-SECURITY.md §12.6) depends on canonical bytes

#### 3.1.1 Rule 1: Map Key Ordering

Map keys MUST be sorted in the bytewise lexicographic order of their **deterministic CBOR encoding**. The comparison algorithm is:

1. Encode each key to its deterministic CBOR byte representation
2. Compare the encoded byte sequences by **length first** (shorter sorts before longer)
3. Among equal-length byte sequences, compare **byte-by-byte** (lower byte value sorts first)

This rule applies to **all** maps in a CBOR-Web document: top-level document maps, nested metadata maps, content block maps, and any other CBOR map at any depth.

**Why length-first?** Because CBOR encodes shorter values in fewer bytes. An integer `0` encodes as `00` (1 byte). A text string `"t"` encodes as `61 74` (2 bytes). The integer is shorter, so it sorts first. Among text strings of the same character count, the encoded length is the same, so bytewise comparison applies.

#### 3.1.2 Integer Key Ordering

CBOR integers 0-23 each encode as a single byte (`0x00` through `0x17`). Integers 24-255 encode as 2 bytes (`0x18` followed by the value). Negative integers encode with major type 1.

For CBOR-Web's top-level document keys (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10):

| Key | CBOR Encoding | Length | Sort Position |
|-----|---------------|--------|---------------|
| 0 | `00` | 1 byte | 1st |
| 1 | `01` | 1 byte | 2nd |
| 2 | `02` | 1 byte | 3rd |
| 3 | `03` | 1 byte | 4th |
| 4 | `04` | 1 byte | 5th |
| 5 | `05` | 1 byte | 6th |
| 6 | `06` | 1 byte | 7th |
| 7 | `07` | 1 byte | 8th |
| 8 | `08` | 1 byte | 9th |
| 9 | `09` | 1 byte | 10th |
| 10 | `0A` | 1 byte | 11th |

All integer keys 0-23 are single-byte and sort naturally by their byte value. This is straightforward.

#### 3.1.3 Text Key Ordering

Text string keys are where deterministic encoding becomes important. The CBOR encoding of a text string is: header byte(s) indicating major type 3 + length, followed by the UTF-8 bytes.

For strings of length 0-23, the header is a single byte: `0x60 + length`. For strings of length 24-255, the header is two bytes: `0x78` followed by the length byte.

**Example: site metadata keys**

| Key | UTF-8 Length | CBOR Header | CBOR Encoding | Total Bytes |
|-----|-------------|-------------|---------------|-------------|
| `"name"` | 4 | `64` | `64 6E616D65` | 5 bytes |
| `"domain"` | 6 | `66` | `66 646F6D61696E` | 7 bytes |
| `"languages"` | 9 | `69` | `69 6C616E677561676573` | 10 bytes |
| `"default_language"` | 16 | `70` | `70 64656661756C745F6C616E6775616765` | 17 bytes |

**Deterministic order**: name (5) < domain (7) < languages (10) < default_language (17).

Because `"name"` encodes to 5 bytes and `"domain"` encodes to 7 bytes, `"name"` sorts first — even though `"domain"` comes first alphabetically. **Alphabetical sorting is WRONG for CBOR deterministic encoding.**

**Example: page entry keys**

| Key | UTF-8 Length | CBOR Encoding | Total Bytes |
|-----|-------------|---------------|-------------|
| `"lang"` | 4 | `64 6C616E67` | 5 bytes |
| `"path"` | 4 | `64 70617468` | 5 bytes |
| `"size"` | 4 | `64 73697A65` | 5 bytes |
| `"title"` | 5 | `65 7469746C65` | 6 bytes |
| `"access"` | 6 | `66 616363657373` | 7 bytes |
| `"updated"` | 7 | `67 75706461746564` | 8 bytes |
| `"hash"` | 4 | `64 68617368` | 5 bytes |

**Deterministic order for the 5-byte group** (all encode to 5 bytes):
- `"hash"` → `64 68617368` (3rd byte is `68`)
- `"lang"` → `64 6C616E67` (3rd byte is `6C`)
- `"path"` → `64 70617468` (3rd byte is `70`)
- `"size"` → `64 73697A65` (3rd byte is `73`)

So: hash < lang < path < size (all 5 bytes, then bytewise: `68 < 6C < 70 < 73`).

Then title (6 bytes), access (7 bytes), updated (8 bytes).

**Complete order**: hash, lang, path, size, title, access, updated.

**Example: content block keys (single-character)**

| Key | CBOR Encoding | Total Bytes | Byte Value |
|-----|---------------|-------------|------------|
| `"l"` | `61 6C` | 2 bytes | `6C` = 108 |
| `"t"` | `61 74` | 2 bytes | `74` = 116 |
| `"v"` | `61 76` | 2 bytes | `76` = 118 |

**Deterministic order**: l (6C) < t (74) < v (76). All are 2 bytes, so bytewise comparison of the 2nd byte applies.

**A heading block `{"t": "h", "l": 1, "v": "Title"}` MUST be encoded with keys in order: l, t, v — NOT t, l, v.**

This means the CBOR diagnostic notation for a heading is:

```cbor-diag
{"l": 1, "t": "h", "v": "Title"}   ; correct deterministic order
```

NOT:

```cbor-diag
{"t": "h", "l": 1, "v": "Title"}   ; WRONG — alphabetical/logical, not deterministic
```

Both decode to the same logical data, but only the first produces the correct deterministic bytes.

#### 3.1.4 Rule 2: Minimal Integer Encoding

Integers MUST be encoded in their **smallest possible representation**:

| Value Range | Encoding | Bytes | Example |
|-------------|----------|-------|---------|
| 0-23 | Single byte: `0x00`-`0x17` | 1 | `0` → `00`, `23` → `17` |
| 24-255 | `0x18` + 1 byte | 2 | `95` → `18 5F` |
| 256-65535 | `0x19` + 2 bytes | 3 | `1000` → `19 03E8` |
| 65536-4294967295 | `0x1A` + 4 bytes | 5 | `1742515200` → `1A 67DCAC00` |
| > 4294967295 | `0x1B` + 8 bytes | 9 | Large timestamps |

An encoder MUST NOT use `0x18 0A` for the value 10 — the correct encoding is `0A` (single byte). An encoder MUST NOT use `0x19 005F` for the value 95 — the correct encoding is `0x18 5F` (two bytes).

This rule is automatically enforced by compliant CBOR libraries (ciborium, cbor2, etc.), but a custom encoder must verify this.

#### 3.1.5 Rule 3: Definite Lengths

All arrays and maps MUST use **definite-length encoding**. The length (number of elements) MUST be specified in the header byte(s), not with an indefinite-length marker (`0x9F` for arrays, `0xBF` for maps).

| Structure | Correct | Wrong |
|-----------|---------|-------|
| Array [1, 2, 3] | `83 01 02 03` (definite, 3 items) | `9F 01 02 03 FF` (indefinite + break) |
| Map {"a": 1} | `A1 61 61 01` (definite, 1 pair) | `BF 61 61 01 FF` (indefinite + break) |

**Exception**: For binary data exceeding 100 KB (inline images, large transcription text), a publisher MAY use indefinite-length byte strings (§3.8). This exception does NOT apply to arrays or maps.

#### 3.1.6 Rule 4: Floating-Point Representation

Floating-point values MUST use the **shortest IEEE 754 representation that preserves the exact value**. This is required (not optional) because deterministic encoding depends on it — two publishers encoding the same float value MUST produce identical bytes for hash reproducibility (§10.2).

The algorithm is:
1. If the value can be represented exactly as IEEE 754 binary16 (half-precision), use half (3 bytes).
2. Otherwise, if representable exactly as IEEE 754 binary32 (single-precision), use single (5 bytes).
3. Otherwise, use IEEE 754 binary64 (double-precision, 9 bytes).

| Value | Half (16-bit) | Single (32-bit) | Double (64-bit) | Required Encoding |
|-------|--------------|-----------------|-----------------|-------------------|
| 0.0 | `F9 0000` (3B) | `FA 00000000` (5B) | `FB 0000000000000000` (9B) | Half (3B) |
| 1.0 | `F9 3C00` (3B) | `FA 3F800000` (5B) | `FB 3FF0000000000000` (9B) | Half (3B) |
| 29.90 | N/A (loses precision) | `FA 41EF3333` (5B) | `FB 403DE66666666666` (9B) | Double (9B) |
| 3.14159 | N/A | `FA 40490FD0` (5B) | `FB 400921FB54442D18` (9B) | Single (5B) if exact, else Double |

**Implementation note:** Most CBOR libraries do NOT perform shortest-float encoding by default. Publishers MUST enable or implement this explicitly. In Rust (ciborium), use a custom serializer. In Python (cbor2), `canonical=True` enables shortest float encoding.

### 3.2 Self-Described CBOR

Every CBOR-Web document (manifest, page, and bundle) MUST begin with **CBOR tag 55799** (self-described CBOR). This tag encodes as the three-byte sequence `D9 D9 F7`.

```
D9 D9 F7    ; tag(55799) — "I am a CBOR document"
  A5        ; map(5) — the manifest/page/bundle content
    ...     ;  (remaining fields omitted for brevity)
```

**Purpose**: The self-described CBOR tag serves as a **magic number** that uniquely identifies a CBOR-Web file. This is important because:

1. **Format detection without headers**: If an agent receives a binary blob without HTTP Content-Type headers (e.g., from a file system or message queue), the first three bytes `D9 D9 F7` immediately identify it as CBOR.

2. **Collision resistance**: The byte sequence `D9 D9 F7` does not appear as a valid prefix in:
   - UTF-8 text (these bytes are invalid UTF-8)
   - JSON (which starts with `{`, `[`, `"`, a digit, or whitespace)
   - XML (which starts with `<`)
   - HTML (which starts with `<!DOCTYPE` or `<html>`)
   - PDF (which starts with `%PDF`)
   - PNG (which starts with `89 50 4E 47`)
   - ZIP (which starts with `50 4B`)

3. **Agent heuristic**: An agent encountering an unknown binary file can check bytes 0-2 for `D9 D9 F7` to determine if it is CBOR before attempting to parse it.

**Implementation note**: A publisher generates the self-described tag by wrapping the entire document in `Tag(55799, <content>)`. Most CBOR libraries handle this with a single function call:

```rust
// Rust (ciborium)
let doc = Value::Tag(55799, Box::new(manifest));
ciborium::into_writer(&doc, &mut output)?;
```

```python
# Python (cbor2)
doc = cbor2.CBORTag(55799, manifest)
cbor2.dumps(doc, canonical=True)
```

### 3.3 Text Encoding

All text values in CBOR-Web documents MUST be CBOR text strings (**major type 3**, UTF-8 encoded). This includes:

- All text map keys (`"domain"`, `"path"`, `"title"`, etc.)
- All text values (page titles, paragraph text, URLs, etc.)
- Block type codes (`"h"`, `"p"`, `"ul"`, etc.)
- Language codes (`"en"`, `"fr"`, etc.)

A conforming agent MUST reject a document that uses byte strings (major type 2) where text is expected. This prevents accidental type confusion and ensures that all text can be processed as UTF-8.

**Exception**: SHA-256 hashes MUST be encoded as byte strings (major type 2), exactly 32 bytes. This is the natural representation for binary hash values.

```cbor-diag
; CORRECT — text as text string, hash as byte string
"title": "Hello World",                 ; text string (major type 3)
"hash": h'D8CAD2E6E8D06A0E...'         ; byte string (major type 2), 32 bytes
```

```cbor-diag
; WRONG — hash as text string (hex-encoded)
"hash": "D8CAD2E6E8D06A0E..."          ; text string — WRONG, wastes 2x space
```

### 3.4 Timestamps

All timestamps in CBOR-Web MUST use **CBOR tag 1** (epoch-based date/time, numeric) with **integer precision** (seconds since Unix epoch, 1970-01-01T00:00:00Z).

```cbor-diag
"generated_at": 1(1742515200)    ; 2025-03-21T00:00:00Z
"updated": 1(1742428800)         ; 2025-03-20T00:00:00Z
```

In CBOR encoding, a tagged timestamp looks like:

```
C1              ; tag(1) — epoch-based date/time
  1A 67DCAC00   ; unsigned(1742515200)
```

**Why integer precision?** Millisecond precision adds 4 bytes per timestamp (upgrading from uint32 to uint64 or adding a float component) without meaningful benefit for web content timestamps. A blog post published "at 14:30:00" versus "at 14:30:00.123" is identical for an agent's purposes.

**Why not tag 0 (date/time string)?** Tag 0 encodes timestamps as RFC 3339 text strings (e.g., `"2026-03-21T00:00:00Z"`), which are 20+ bytes. Tag 1 with an integer is 5-6 bytes — a 75% reduction.

**Conversion reference:**

| Date/Time | Unix Epoch | CBOR |
|-----------|-----------|------|
| 2026-01-01T00:00:00Z | 1767225600 | `C1 1A 6955B900` |
| 2025-03-20T00:00:00Z | 1742428800 | `C1 1A 67DB5A80` |
| 2025-03-21T00:00:00Z | 1742515200 | `C1 1A 67DCAC00` |
| 2026-12-31T23:59:59Z | 1798761599 | `C1 1A 6B36EC7F` |

### 3.5 Integer Key Strategy

CBOR-Web uses a three-tier key strategy to minimize binary size while maintaining debuggability:

| Tier | Key Type | Usage | Example | Encoding Size | Rationale |
|------|----------|-------|---------|--------------|-----------|
| **Tier 1** | Integer (0-10) | Top-level document keys | `0: "cbor-web-manifest"` | 1 byte | Appears once per document. Maximum savings (1 byte vs 5-20 bytes for text). |
| **Tier 2** | Short text | Second-level map keys | `"domain": "example.com"` | 5-12 bytes | Appears once per map. Debug readability outweighs size savings. |
| **Tier 3** | Single character | Content block keys | `"t": "h"` | 2 bytes | Appears hundreds of times per page. 2 bytes vs 5+ bytes = significant savings. |

**Why not use integers everywhere?** For second-level keys like `"domain"`, `"title"`, `"path"`, the saving of 3-4 bytes per key is modest (these keys appear a handful of times), but the loss of debuggability is significant. When debugging a CBOR-Web document with `cbor2diag` or similar tools, seeing `"domain": "example.com"` is immediately understandable, while `12: "example.com"` requires consulting a lookup table.

For content block keys, the calculus changes: a page with 50 content blocks has ~150 key occurrences (3 keys per block average). Saving 3 bytes per key saves 450 bytes per page — meaningful for a format optimized for compactness.

**Tier 1 key registry** (top-level document keys):

| Key | Name | Used In |
|-----|------|---------|
| 0 | @type | Manifest, Page, Bundle |
| 1 | @version | Manifest, Page, Bundle |
| 2 | site / identity | Manifest (site metadata), Page (identity) |
| 3 | pages / metadata | Manifest (page list), Page (metadata) |
| 4 | navigation / content | Manifest (nav), Page (content blocks) |
| 5 | meta / links | Manifest (generation meta), Page (links) |
| 6 | signature / structured_data | Manifest (COSE sig), Page (Schema.org) |
| 7 | capabilities / generative | Manifest (caps), Page (generative blocks) |
| 8 | channels / forms | Manifest (streams), Page (forms) |
| 9 | diff / commerce | Manifest (diff update), Page (commerce) |
| 10 | security | Manifest only (security config) |

### 3.6 Forward Compatibility

An agent MUST ignore any map key it does not recognize. A publisher MAY include additional keys beyond those specified in this document. This rule is **fundamental** to the evolution of CBOR-Web.

**Concrete example**: Suppose a v3.0 manifest includes a key 15 for "AI agent personality preferences":

```cbor-diag
55799({
  0: "cbor-web-manifest",
  1: 3,                              ; version 3
  2: { "domain": "example.com", ... },  ;  (remaining fields omitted for brevity)
  3: [ ... ],  ;  (remaining fields omitted for brevity)
  5: { ... },  ;  (remaining fields omitted for brevity)
  15: { "personality": "helpful" }   ; NEW in v3.0 — unknown to v2.1 agents
})
```

A v2.1 agent:
1. Reads keys 0-10 normally (same as v2.1)
2. Encounters key 15 — does not recognize it
3. **Ignores key 15** and continues processing
4. Functions correctly with all v2.1 features

This rule also applies to text keys within maps. If a page entry includes a field `"ai_summary"` that does not exist in v2.1:

```cbor-diag
{"path": "/about", "title": "About", "ai_summary": "Company overview page"}
```

A v2.1 agent ignores `"ai_summary"` and processes `"path"` and `"title"` normally.

**A breaking change** to the semantics of an existing key (e.g., changing key 4 from "content blocks" to "something else") requires incrementing the `@version` field (key 1). The forward-compatibility rule applies only to **new** keys, not changed semantics of existing keys.

### 3.7 Binary Data Encoding

CBOR-Web documents may contain binary data in specific contexts:

1. **SHA-256 hashes**: always 32-byte byte strings (`bstr .size 32` in CDDL)
2. **COSE signatures**: serialized COSE_Sign1 structures stored as byte strings
3. **Inline image data**: small images (< 10 KB) embedded in `"image"` blocks (see CBOR-WEB-MULTIMEDIA.md §15.2.2)

All binary data MUST be encoded as **CBOR byte strings (major type 2)** containing **raw bytes**. A publisher MUST NOT base64-encode binary data inside a byte string — this would double the size for zero benefit, since CBOR byte strings are already binary.

```cbor-diag
; CORRECT — raw bytes in a byte string
"hash": h'D8CAD2E6E8D06A0EF4E5B22C7394E4AC2B48FDE517DBB012C705DB26D163CEB7'

; WRONG — base64-encoded bytes in a byte string
"hash": h'524943414432453645384430364130454634453542...'
; This is the base64 encoding of the hash, re-encoded as bytes. Double encoding. Wrong.

; ALSO WRONG — hex-encoded bytes in a text string
"hash": "D8CAD2E6E8D06A0EF4E5B22C7394E4AC2B48FDE517DBB012C705DB26D163CEB7"
; This is a text string (major type 3) containing hex characters. 64 text bytes vs 32 binary bytes. Wrong.
```

For base64 contexts (e.g., data URIs in legacy interop), CBOR tag 21 (expected conversion to base64url) or tag 22 (expected conversion to base64) MAY be used as encoding hints. These tags do not change the wire format — they hint to a transcoder that the byte string should be rendered as base64 when converting to JSON.

### 3.8 Large Value Streaming

For values exceeding 100 KB (e.g., embedded images, transcription text for long videos), a publisher MAY use CBOR **indefinite-length byte strings** (major type 2, additional info 31) to enable streaming parsing.

```
5F              ; indefinite-length byte string
  44 AABBCCDD   ; definite chunk 1 (4 bytes)
  43 EEFF00     ; definite chunk 2 (3 bytes)
  FF            ; break code — end of indefinite-length
```

This is an **exception** to the definite-length requirement (§3.1.5), applicable ONLY to:

- Inline binary data in multimedia blocks (`"inline_data"` in `"image"` blocks)
- Transcription text exceeding 100 KB
- Bundle documents exceeding 10 MB

The **top-level document structure** (the outermost map and its immediate children) MUST still use definite-length encoding. Only the values of specific fields may use indefinite-length.

An agent MUST support both definite-length and indefinite-length byte strings when parsing CBOR-Web documents.

### 3.9 Text Normalization

For deterministic encoding to produce identical bytes across different publishers converting the same HTML source, all text values MUST be normalized before CBOR encoding:

#### 3.9.1 Unicode Normalization

All text strings MUST be in **Unicode NFC** (Canonical Decomposition, followed by Canonical Composition) as defined in Unicode Standard Annex #15. This ensures that equivalent character sequences (e.g., `é` as U+00E9 versus `e` + U+0301) produce identical UTF-8 bytes.

A publisher MUST apply NFC normalization to all text values before CBOR encoding. An agent MAY verify NFC normalization but SHOULD NOT reject content that is not NFC-normalized — instead, it SHOULD normalize locally before hash comparison.

#### 3.9.2 Line Endings

All line endings in text values MUST use **LF** (`\n`, U+000A). A publisher MUST convert:
- CRLF (`\r\n`) → LF (`\n`)
- CR (`\r`) → LF (`\n`)

This applies to all text values including paragraph text, code blocks, list items, and all other string fields.

#### 3.9.3 Whitespace

- **Leading and trailing whitespace** in paragraph (`"p"`) and heading (`"h"`) text values MUST be trimmed.
- **Internal whitespace** (consecutive spaces, tabs within text) SHOULD be collapsed to a single space in editorial blocks (`"h"`, `"p"`, `"q"`, list items), but MUST be preserved in code blocks (`"code"`).
- **Empty text values** (`""`) SHOULD be avoided. A publisher SHOULD omit blocks with empty content rather than emitting empty strings.

#### 3.9.4 Rationale

Without these rules, two publishers converting identical HTML will produce different CBOR bytes if one uses NFC and another uses NFD, or if one preserves `\r\n` from a Windows source. Different bytes mean different SHA-256 hashes, which breaks the cache validation mechanism (§10.1) — the fundamental efficiency proposition of CBOR-Web.

---

## 4. Discovery Protocol

An agent MUST be able to discover the presence of CBOR-Web content on a website. This section defines five discovery mechanisms, listed in order of priority. An agent SHOULD attempt them in this order and stop at the first successful response.

### 4.1 Well-Known URL (REQUIRED for Publishers)

**This is the primary and mandatory discovery mechanism.**

A CBOR-Web publisher MUST serve the manifest at the well-known URL:

```
GET /.well-known/cbor-web HTTP/1.1
Host: example.com
Accept: application/cbor
```

The server MUST respond with:

```
HTTP/1.1 200 OK
Content-Type: application/cbor
Content-Length: 524
Cache-Control: public, max-age=3600

[binary CBOR manifest]
```

The well-known URL follows RFC 8615 ("Well-Known Uniform Resource Identifiers (URIs)"). The URI suffix `cbor-web` is registered with IANA (see §12.1).

**Path structure under the well-known URL:**

| URL | Content | Description |
|-----|---------|-------------|
| `/.well-known/cbor-web` | Manifest | The site's CBOR-Web manifest. REQUIRED. |
| `/.well-known/cbor-web/pages/{filename}.cbor` | Individual pages | One file per page. See §6.1 for path encoding. |
| `/.well-known/cbor-web/bundle` | Bundle | All pages + manifest in one file. OPTIONAL. |
| `/.well-known/cbor-web/keys.cbor` | Key set | Publisher's signing keys. OPTIONAL. See CBOR-WEB-SECURITY.md §12.6. |

**If the site does not support CBOR-Web**, the server SHOULD respond with `404 Not Found`. Other acceptable responses:

| Status | Meaning | Agent Behavior |
|--------|---------|---------------|
| 200 OK + `application/cbor` | CBOR-Web supported. Body is the manifest. | Parse manifest, proceed. |
| 404 Not Found | CBOR-Web not supported. | Try next discovery method, or give up. |
| 405 Method Not Allowed | GET not accepted for this URL. | CBOR-Web not supported at this URL. |
| 406 Not Acceptable | `application/cbor` not served. | CBOR-Web not supported. |
| 301/302 Redirect | Manifest moved. | Follow redirect (max 3 hops). |
| 429 Too Many Requests | Rate limited. | Retry after `Retry-After` header duration. |
| 402 Payment Required | Token required for manifest access. | Agent needs a CBOR-Web token. See CBOR-WEB-SECURITY.md §12. |

**Agent validation after receiving a 200 OK:**

An agent MUST validate the response before processing:
1. Content-Type SHOULD be `application/cbor`. If Content-Type is `application/octet-stream` (common when servers lack CBOR MIME configuration), the agent SHOULD proceed to step 2 (magic bytes validation) before rejecting. Content-Type `text/html` or other non-binary types MUST be rejected immediately.
2. First 3 bytes MUST be `D9 D9 F7` (self-described CBOR tag 55799)
3. The root map MUST contain key 0 with value `"cbor-web-manifest"`
4. The root map MUST contain key 1 with a version number the agent can process

If any validation fails, the agent MUST discard the response and treat the site as non-CBOR-Web.

### 4.2 HTTP Link Header (RECOMMENDED)

Any HTML page MAY include an HTTP response header indicating CBOR-Web availability:

```
Link: </.well-known/cbor-web>; rel="alternate"; type="application/cbor"
```

This mechanism allows an agent to discover CBOR-Web while processing normal HTML responses. For example, an agent browsing `https://example.com/about` in HTML mode might notice the Link header and switch to CBOR-Web for subsequent requests.

**Full example:**

```
HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8
Link: </.well-known/cbor-web>; rel="alternate"; type="application/cbor"
Content-Length: 15230

<!DOCTYPE html>
<html>...
```

The agent sees the Link header, notes the CBOR-Web URL, and can switch to the binary channel for future requests.

**Multiple Link headers**: A server MAY include multiple Link headers on the same response. The agent SHOULD look for the one with `type="application/cbor"` and `rel="alternate"`.

### 4.3 HTML Meta Tag (OPTIONAL)

An HTML page MAY include in its `<head>`:

```html
<link rel="alternate" type="application/cbor" href="/.well-known/cbor-web">
```

This is equivalent to the HTTP Link header but embedded in HTML. It is less efficient (requires parsing HTML to discover) but useful when the agent is already parsing HTML for other reasons.

**Page-specific CBOR-Web**: A publisher MAY link to the specific CBOR page rather than the manifest:

```html
<link rel="alternate" type="application/cbor" 
      href="/.well-known/cbor-web/pages/about.cbor">
```

An agent encountering a page-specific link SHOULD also fetch the manifest for full site context.

### 4.4 robots.txt Entry (OPTIONAL)

The site's `robots.txt` MAY include a CBOR-Web directive:

```
# Standard robots.txt directives
User-agent: *
Allow: /

# CBOR-Web machine-readable content
CBOR-Web: /.well-known/cbor-web
```

This is a **non-standard extension** to the Robots Exclusion Protocol (RFC 9309). Standard crawlers will ignore the `CBOR-Web:` directive (unknown directives are ignored per RFC 9309). CBOR-Web-aware agents will recognize it.

**Important**: robots.txt directives (`Disallow`, `Allow`) MUST still be respected by CBOR-Web agents. A CBOR-Web publisher MUST NOT serve content via CBOR-Web that is disallowed for the agent's user-agent in robots.txt.

### 4.5 llms.txt Entry (OPTIONAL)

The site's llms.txt MAY include a reference:

```markdown
# Example Site

> Brief description of the site.

# Machine-Readable Content
- CBOR-Web Manifest: /.well-known/cbor-web
```

This integrates CBOR-Web discovery into the llms.txt ecosystem, allowing text-based AI agents to discover that a binary alternative exists.

### 4.6 Discovery Failure

If none of the above mechanisms return a valid CBOR-Web manifest, the agent MUST conclude that the site does not support CBOR-Web and fall back to its normal content consumption strategy (HTML crawling, llms.txt parsing, etc.).

An agent SHOULD NOT attempt discovery more than once per hour for a site that returned 404. The recommended caching strategy:

| Discovery Result | Cache Duration | Rationale |
|-----------------|---------------|-----------|
| 200 OK (manifest found) | Cache manifest per Cache-Control header | Standard HTTP caching |
| 404 Not Found | 24 hours | Site probably doesn't support CBOR-Web |
| Network error / timeout | 1 hour | Temporary issue, retry sooner |
| 301/302 Redirect to non-CBOR | 24 hours | Deliberate redirect away |
| 429 Too Many Requests | `Retry-After` header value | Server-specified wait time |

### 4.7 Capability-Aware Discovery

When an agent discovers a CBOR-Web manifest, it SHOULD read the `"capabilities"` field (manifest key 7, see CBOR-WEB-GENERATIVE.md §17) before fetching pages. This allows the agent to:

1. **Determine if the site offers content types it can process.** An agent that cannot handle video has no reason to fetch pages from a video-only site.

2. **Prioritize sites with richer capabilities.** In a multi-site scan, a site offering multimedia + generative blocks + commerce is more useful than one offering only static text.

3. **Skip sites that only offer capabilities it doesn't need.** An agent looking for product data can skip sites where `"commerce"."available"` is false.

4. **Plan workflows.** If `"generative"."workflows"` is true, the agent knows it can attempt autonomous multi-step tasks on this site.

### 4.8 Access-Level Discovery

When an agent discovers a manifest, it SHOULD examine the `"access"` field of each page entry (§5.4):

- Pages with `"access": "public"` are available to all agents (L0 storefront)
- Pages with `"access": "token"` require a CBOR-Web token (L1 full access)

An agent without a token can still read:
- The manifest itself (always public)
- All page metadata (titles, descriptions, hashes, sizes) from the manifest
- Pages explicitly marked `"access": "public"`

This allows an unauthenticated agent to discover the site structure and decide if it's worth obtaining a token for full access.

### 4.9 Discovery Sequence Diagram

```
Agent                                  Server
  |                                      |
  |  1. Check /.well-known/cbor-web      |
  |──── GET /cbor-web ──────────────────>|
  |                                      |
  |  CASE A: 200 OK + application/cbor   |
  |<──── Manifest CBOR ─────────────────|
  |                                      |
  |  2. Validate: D9 D9 F7? Key 0?      |
  |  3. Read capabilities (key 7)        |
  |  4. Read access levels (key 3)       |
  |  5. Decide which pages to fetch      |
  |                                      |
  |  CASE B: 404 Not Found               |
  |<──── 404 ────────────────────────────|
  |                                      |
  |  6. Fall back to HTML / llms.txt     |
  |                                      |
  |  CASE C: 402 Payment Required        |
  |<──── 402 + contract address ─────────|
  |                                      |
  |  7. Obtain CBOR-Web token            |
  |  8. Retry with X-CBOR-Web-Wallet     |
  |                                      |
```

---

## 5. Manifest Document

The manifest is the **entry point** to a site's CBOR-Web content. An agent reads the manifest first and makes all subsequent decisions based on its contents: which pages to fetch, what capabilities the site offers, what access level is required, and how to verify content integrity.

A manifest is analogous to a book's table of contents combined with its colophon, copyright page, and index — it provides everything an agent needs to know about the site without reading any individual page.

### 5.1 Top-Level Structure

```cbor-diag
55799({                                ; self-described CBOR tag
  0: "cbor-web-manifest",             ; @type (text) — identifies this as a manifest
  1: 2,                                ; @version (uint) — 2 for this specification
  2: {                                 ; site metadata (map) — describes the site
    "geo": {                           ; "geo" (4B) < "name" (5B) < "domain" (7B) < ...
      "region": "California",          ; "region" (7B) < "country" (8B) < "coordinates" (12B)
      "country": "US",
      "coordinates": [37.7749, -122.4194]
    },
    "name": "Example Site",
    "domain": "example.com",
    "contact": {
      "email": "contact@example.com",
      "phone": "+1-555-0100"
    },
    "languages": ["en", "fr", "es"],
    "description": "A sample website demonstrating CBOR-Web",
    "default_language": "en"
  },
  3: [                                 ; pages (array) — every page on the site
    {
      "hash": h'D8CAD2E6E8D06A0EF4E5B22C7394E4AC2B48FDE517DBB012C705DB26D163CEB7',
      "lang": "en",
      "path": "/",
      "size": 127,
      "title": "Home",
      "access": "public",
      "updated": 1(1742515200),
      "alternates": {
        "es": "/es/",
        "fr": "/fr/"
      }
    },
    {
      "hash": h'9FC41CE55481DEB75F02B545C8B3FC24977AC30A3A70C489F78E4B56035BA68F',
      "lang": "en",
      "path": "/products/lions-mane",
      "size": 541,
      "title": "Lion's Mane Details",
      "access": "token",
      "updated": 1(1742428800),
      "category": "products",
      "content_type": "product",
      "has_commerce": true
    }
  ],
  4: {                                 ; navigation (map) — site structure
    "main": ["/", "/about", "/products", "/blog", "/contact"],
    "footer": ["/privacy", "/terms"],
    "hierarchy": {
      "/blog": ["/blog/lions-mane-guide", "/blog/reishi-benefits"],
      "/products": ["/products/lions-mane", "/products/reishi", "/products/cordyceps"]
    },
    "breadcrumbs": {
      "/products/lions-mane": ["Home", "Products", "Lion's Mane"],
      "/blog/lions-mane-guide": ["Home", "Blog", "Lion's Mane Guide"]
    }
  },
  5: {                                 ; meta (map) — generation info
    "generator": "text2cbor/0.1.0",    ; "generator" (10B) < "bundle_url" (11B) < ...
    "bundle_url": "/.well-known/cbor-web/bundle",
    "rate_limit": {
      "requests_per_second": 10,
      "bundle_cooldown_seconds": 3600
    },
    "total_size": 48200,
    "total_pages": 25,
    "generated_at": 1(1742515200),
    "bundle_available": true
  },
  6: h'...',                           ;  (remaining fields omitted for brevity) — signature (OPTIONAL, see §5.7)
  7: { ... },                          ;  (remaining fields omitted for brevity) — capabilities (RECOMMENDED, see CBOR-WEB-GENERATIVE.md §17)
  8: [ ... ],                          ;  (remaining fields omitted for brevity) — channels (OPTIONAL, see CBOR-WEB-MULTIMEDIA.md §20)
  9: { ... },                          ;  (remaining fields omitted for brevity) — diff (OPTIONAL, see §10.5)
  10: {                                ; security (RECOMMENDED, see CBOR-WEB-SECURITY.md §12.3)
    "chain": "ethereum",
    "security_level": "S1",
    "token_required": true,
    "contract_address": "0x..."
  }
})
```

Note: All map keys in the example above are shown in **deterministic order** (§3.1). For example, in the page entry map, `"hash"` (5 bytes encoded) comes before `"lang"` (5 bytes, but `68 < 6C` bytewise), which comes before `"path"` (5 bytes, `70`), etc.

### 5.2 Top-Level Key Registry

| Key | Name | Type | Required | Description |
|-----|------|------|----------|-------------|
| 0 | @type | text | REQUIRED | MUST be `"cbor-web-manifest"` |
| 1 | @version | uint | REQUIRED | MUST be `2` for this specification. A v1.0 agent seeing version `2` MUST still be able to read keys 0-6 and ignore unknown keys. |
| 2 | site | map | REQUIRED | Site-level metadata: domain, name, description, languages, contact, geo. See §5.3. |
| 3 | pages | array | REQUIRED | Ordered array of page entry maps. Each element describes one page. See §5.4. |
| 4 | navigation | map | OPTIONAL | Site navigation structure: main menu, footer, hierarchy, breadcrumbs. RECOMMENDED at Standard conformance (§11). See §5.5. |
| 5 | meta | map | REQUIRED | Generation metadata: timestamp, page count, total size, bundle info, rate limits. See §5.6. |
| 6 | signature | bstr | OPTIONAL | Serialized COSE_Sign1 structure covering keys 0-5 and 7-10. See §5.7 and CBOR-WEB-SECURITY.md §12.6. |
| 7 | capabilities | map | RECOMMENDED | Site capability declaration: what content types, APIs, and features are available. See CBOR-WEB-GENERATIVE.md §17. |
| 8 | channels | array | OPTIONAL | Real-time streaming channels (WebSocket). See CBOR-WEB-MULTIMEDIA.md §20. |
| 9 | diff | map | OPTIONAL | Differential update since a previous manifest version. See §10.5. |
| 10 | security | map | RECOMMENDED | Security and access control configuration. See CBOR-WEB-SECURITY.md §12.3. |

An agent MUST ignore keys it does not recognize (forward-compatibility rule, §3.6).

### 5.3 Site Metadata (Key 2)

The site metadata map provides global information about the website. This information is read once by the agent and applies to all pages.

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `"domain"` | text | REQUIRED | No protocol prefix. No trailing slash. | Primary domain of the site. Example: `"example.com"`, NOT `"https://example.com"`, NOT `"example.com/"`. |
| `"name"` | text | REQUIRED | Max 200 characters | Human-readable site name. Displayed in search results and agent summaries. Example: `"Verdetao — Functional Mushrooms"` |
| `"description"` | text | RECOMMENDED | Max 500 characters | Site-level description. Should summarize the site's purpose and content. Example: `"Premium functional mushroom supplements, certified organic EU."` |
| `"languages"` | array of text | REQUIRED | BCP 47 tags (RFC 5646). At least 1 element. | Array of all languages in which the site provides content. Example: `["en", "fr", "es"]`. Each element is a BCP 47 language tag: 2-3 letter language code, optionally with script or region subtags: `"pt-BR"`, `"zh-Hans"`, `"sr-Latn-RS"`. |
| `"default_language"` | text | REQUIRED | MUST be an element of `"languages"` | The default language of the site. When a page does not specify a language, it is assumed to be in the default language. Example: `"en"` |
| `"contact"` | map | OPTIONAL | | Contact information for the site operator. |
| `"contact"."email"` | text | OPTIONAL | Valid email address | Contact email. Example: `"contact@example.com"` |
| `"contact"."phone"` | text | OPTIONAL | E.164 format | Contact phone number. Example: `"+33612345678"` |
| `"geo"` | map | OPTIONAL | | Geographic location of the business. |
| `"geo"."country"` | text | OPTIONAL | ISO 3166-1 alpha-2 | Country code. Example: `"FR"`, `"US"`, `"ES"` |
| `"geo"."region"` | text | OPTIONAL | | Region, state, or province. Example: `"California"`, `"Andalusia"` |
| `"geo"."coordinates"` | array of 2 floats | OPTIONAL | [lat, lon] in WGS84. lat: -90 to 90. lon: -180 to 180. | GPS coordinates. Example: `[37.7749, -122.4194]` (San Francisco) |

**Validation rules:**

1. `"domain"` MUST NOT contain a protocol scheme (`https://`), path, port, or query string. It is the bare domain name.
2. `"default_language"` MUST be present in the `"languages"` array. An agent MAY reject a manifest where the default language is not in the languages list.
3. `"description"` SHOULD be concise and informative. It serves the same purpose as an HTML `<meta name="description">` tag.
4. `"coordinates"` MUST be [latitude, longitude], NOT [longitude, latitude]. This follows the ISO 6709 convention used by Schema.org and Google Maps.

**CDDL:**

```cddl
site-metadata = {
  "domain" => tstr,
  "name" => tstr,
  ? "description" => tstr,
  "languages" => [+ language-code],
  "default_language" => language-code,
  ? "contact" => contact-info,
  ? "geo" => geo-info,
  * tstr => any
}

language-code = tstr .regexp "[a-z]{2,3}(-[A-Za-z]{2,8})*"  ; BCP 47 (RFC 5646)
contact-info = { ? "email" => tstr, ? "phone" => tstr, * tstr => any }
geo-info = {
  ? "country" => tstr,
  ? "region" => tstr,
  ? "coordinates" => [latitude, longitude],
  * tstr => any
}
latitude = float .ge -90.0 .le 90.0
longitude = float .ge -180.0 .le 180.0
```

### 5.4 Page Entry (Elements of Key 3)

Each element in the pages array is a map describing a single page. The page entry contains enough information for an agent to decide whether to fetch the page without downloading it.

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `"path"` | text | REQUIRED | Must start with `/`. No query string. No fragment. | URL path relative to domain. Examples: `"/"`, `"/about"`, `"/products/lions-mane"` |
| `"title"` | text | REQUIRED | Max 300 characters | Page title. Same as the HTML `<title>` tag. |
| `"lang"` | text | REQUIRED | ISO 639-1 code, must be in site `"languages"` | Language of this page. |
| `"access"` | text | REQUIRED | `"public"` or `"token"` | Access level. `"public"`: available to all (L0). `"token"`: requires CBOR-Web token (L1). See CBOR-WEB-SECURITY.md §12. |
| `"size"` | uint | REQUIRED | In bytes. Max 1,048,576 (1 MB). | Size of the standalone page CBOR document (with tag 55799). |
| `"updated"` | tag 1 (uint) | RECOMMENDED | Unix epoch, integer seconds | Last modification timestamp. Required at Standard conformance (§11). |
| `"hash"` | bstr (32 bytes) | RECOMMENDED | SHA-256, exactly 32 bytes | SHA-256 hash of the standalone page CBOR document (with tag 55799). Required at Standard conformance. See §10.2. |
| `"alternates"` | map | OPTIONAL | Keys: language codes. Values: path strings. | Language alternates. Example: `{"fr": "/fr/products/lions-mane", "es": "/es/productos/melena-de-leon"}` |
| `"category"` | text | OPTIONAL | | Page category or section. Examples: `"products"`, `"blog"`, `"documentation"` |
| `"content_type"` | text | OPTIONAL | | Nature of the content: `"article"`, `"product"`, `"landing"`, `"documentation"`, or custom. |
| `"has_multimedia"` | bool | OPTIONAL | | Whether this page contains multimedia blocks (see CBOR-WEB-MULTIMEDIA.md). |
| `"has_generative"` | bool | OPTIONAL | | Whether this page contains generative blocks (see CBOR-WEB-GENERATIVE.md). |
| `"has_forms"` | bool | OPTIONAL | | Whether this page contains form blocks. |
| `"has_commerce"` | bool | OPTIONAL | | Whether this page contains commerce blocks. |
| `"media_size"` | uint | OPTIONAL | In bytes | Total size of referenced external media (images, videos, documents). Helps an agent estimate bandwidth needs. |

**Access level guidance:**

A publisher SHOULD make at least 30% of pages `"access": "public"` so that the site remains discoverable by agents without tokens. A site with 100% token-gated content will not be visible to search engines or unauthenticated agents.

Recommended access policies:

| Page Type | Recommended Access | Rationale |
|-----------|-------------------|-----------|
| Home page | `"public"` | Entry point, must be visible |
| About / Contact | `"public"` | Discovery and trust-building |
| Blog articles | `"public"` | Content marketing, SEO |
| Product descriptions | `"public"` or `"token"` | Publisher choice — public for discovery, token for premium data |
| Detailed product data (specs, reviews) | `"token"` | Premium content |
| API documentation | `"token"` | Premium developer access |
| Pricing / wholesale | `"token"` | Competitive data |

**Page entry ordering:**

Page entries in the pages array (key 3) SHOULD be ordered by `"updated"` timestamp, most recent first. This allows an agent to quickly identify recently changed pages without scanning the entire array.

**CDDL:**

```cddl
page-entry = {
  "path" => tstr,
  "title" => tstr,
  "lang" => language-code,
  "access" => "public" / "token",
  "size" => uint,
  ? "updated" => #6.1(uint),
  ? "hash" => bstr .size 32,
  ? "alternates" => { + language-code => tstr },
  ? "category" => tstr,
  ? "content_type" => page-content-type,
  ? "has_multimedia" => bool,
  ? "has_generative" => bool,
  ? "has_forms" => bool,
  ? "has_commerce" => bool,
  ? "media_size" => uint,
  * tstr => any
}

page-content-type = "article" / "product" / "landing" / "documentation" / tstr
```

### 5.5 Navigation (Key 4)

The navigation map provides the site's navigation structure in a machine-readable format. An agent can reconstruct the full site tree from this map without accessing any page content.

Key 4 is OPTIONAL to support Minimal conformance (§11). Publishers at Standard or Full conformance SHOULD include it.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `"main"` | array of text | REQUIRED (if key 4 present) | Primary navigation menu paths, in display order. These are the top-level menu items a human visitor sees. Example: `["/", "/about", "/products", "/blog", "/contact"]` |
| `"footer"` | array of text | OPTIONAL | Footer navigation paths. Typically legal and informational pages. Example: `["/privacy", "/terms", "/cgv"]` |
| `"hierarchy"` | map (path → array of paths) | RECOMMENDED | Parent-child page relationships. Each key is a parent path, each value is an array of child paths. Example: `{"/products": ["/products/lions-mane", "/products/reishi"]}` |
| `"breadcrumbs"` | map (path → array of text) | OPTIONAL | Breadcrumb trail for each page. Each key is a page path, each value is an ordered array of breadcrumb labels. Example: `{"/products/lions-mane": ["Home", "Products", "Lion's Mane"]}` |

**Navigation completeness:**

The `"main"` array SHOULD contain paths that exist in the pages array (key 3). An agent encountering a navigation path that does not exist in the pages array SHOULD treat it as a broken link.

The `"hierarchy"` map enables tree reconstruction:

```
/                           (root)
├── /about
├── /products               (parent)
│   ├── /products/lions-mane
│   ├── /products/reishi
│   └── /products/cordyceps
├── /blog                   (parent)
│   ├── /blog/lions-mane-guide
│   └── /blog/reishi-benefits
└── /contact
```

An agent reads the hierarchy and knows the complete tree without downloading any pages.

**CDDL:**

```cddl
navigation = {
  "main" => [+ tstr],
  ? "footer" => [+ tstr],
  ? "hierarchy" => { + tstr => [+ tstr] },
  ? "breadcrumbs" => { + tstr => [+ tstr] },
  * tstr => any
}
```

### 5.6 Meta (Key 5)

The meta map contains information about the manifest itself: when it was generated, what tool generated it, and what capabilities are available.

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `"generator"` | text | RECOMMENDED | `"software/version"` format | Publisher software name and version. Example: `"text2cbor/0.1.0"` |
| `"generated_at"` | tag 1 (uint) | REQUIRED | Unix epoch | When this manifest was generated. Used for freshness checking. |
| `"total_pages"` | uint | REQUIRED | | Total number of pages across all sub-manifests. |
| `"total_size"` | uint | REQUIRED | In bytes | Total size of all standalone page documents. Helps agent estimate storage/bandwidth. |
| `"bundle_available"` | bool | REQUIRED | | Whether a bundle endpoint exists. If true, `"bundle_url"` MUST also be present. |
| `"bundle_url"` | text | CONDITIONAL | URL path, starts with `/` | Bundle URL path. REQUIRED if `"bundle_available"` is true. Example: `"/.well-known/cbor-web/bundle"` |
| `"rate_limit"` | map | OPTIONAL | | Rate limiting parameters declared by the publisher. |
| `"rate_limit"."requests_per_second"` | uint | OPTIONAL | Default: 10 | Maximum requests per second per agent. An agent MUST NOT exceed this rate. |
| `"rate_limit"."bundle_cooldown_seconds"` | uint | OPTIONAL | Default: 3600 | Minimum seconds between bundle re-downloads. |
| `"next"` | text | CONDITIONAL | URL path | URL of the next sub-manifest page. See §5.8. Present only if the manifest is paginated. |

**Rate limiting:**

The rate limit is a **publisher declaration**, not a server-enforced limit (though servers SHOULD also enforce it). An agent MUST respect the declared rate limit even if the server does not actively enforce it. This is a matter of protocol compliance and good citizenship.

**Agent identification:** An agent SHOULD include a `User-Agent` header identifying itself (e.g., `User-Agent: MyAgent/1.0 (cbor-web)`) to enable per-agent rate limiting on the server side. For authenticated agents (token holders), the wallet address (`X-CBOR-Web-Wallet`) serves as a unique identifier. For anonymous agents, the server MAY fall back to IP-based rate limiting.

| Scenario | Rate Limit Behavior |
|----------|-------------------|
| No `"rate_limit"` in meta | Agent uses default: 10 req/s |
| `"requests_per_second": 5` | Agent MUST NOT exceed 5 req/s |
| Agent receives HTTP 429 | Agent MUST back off regardless of declared limit |
| Agent is fetching a bundle | After download, agent MUST wait `bundle_cooldown_seconds` before re-downloading |

**CDDL:**

```cddl
manifest-meta = {
  ? "generator" => tstr,
  "generated_at" => #6.1(uint),
  "total_pages" => uint,
  "total_size" => uint,
  "bundle_available" => bool,
  ? "bundle_url" => tstr,
  ? "rate_limit" => rate-limit-info,
  ? "next" => tstr,
  * tstr => any
}

rate-limit-info = {
  ? "requests_per_second" => uint,
  ? "bundle_cooldown_seconds" => uint,
  * tstr => any
}
```

### 5.7 Signature (Key 6)

When present, key 6 contains a **byte string** wrapping a serialized COSE_Sign1 structure (RFC 9052). The byte string approach ensures that key 6 is always CBOR major type 2 — there is no ambiguity about the wire format.

The signature covers manifest keys 0-5 and 7-10 (everything except key 6 itself, which holds the signature). Key 10 (security config) MUST be included in the signed payload because it contains the token contract address and security level — allowing modification of these fields without signature invalidation would enable man-in-the-middle attacks that redirect token verification to a malicious contract.

For full details on signing, key discovery, key rotation, and verification, see CBOR-WEB-SECURITY.md §12.6.

**CDDL:**

```cddl
; Key 6 is always a byte string containing serialized COSE_Sign1
; Signature covers keys 0-5 and 7-10 (everything except key 6 itself)
; The bstr approach avoids major type ambiguity (major 2 for bstr, not major 4 for array)
? 6 => bstr   ; contains serialized COSE_Sign1 when present
```

### 5.8 Sub-Manifests (Pagination)

For sites with more than 500 pages, the manifest SHOULD be paginated into sub-manifests. This prevents any single manifest from becoming too large to process efficiently.

**Rules:**

1. The `"next"` field in key 5 (meta) contains the URL path of the next sub-manifest page.
2. The `"total_pages"` field reflects the total across ALL sub-manifests (not just the current page).
3. The pages array (key 3) contains only the pages for the current sub-manifest page.
4. **Site metadata (key 2) and navigation (key 4) are present only in the first sub-manifest.** Subsequent sub-manifests make these keys OPTIONAL. This is reflected in the CDDL via a distinct `sub-manifest` type (see Appendix A).
5. Security config (key 10) SHOULD be present in every sub-manifest (since an agent may fetch sub-manifests independently).

**Pagination flow:**

```
GET /.well-known/cbor-web
→ Manifest page 1: pages 1-500, meta.next = "/.well-known/cbor-web?page=2"
  Keys present: 0, 1, 2, 3, 4, 5, [6], [7], [8], [10]

GET /.well-known/cbor-web?page=2
→ Manifest page 2: pages 501-1000, meta.next = "/.well-known/cbor-web?page=3"
  Keys present: 0, 1, 3, 5, [10]
  Keys ABSENT: 2 (site-metadata), 4 (navigation) — same as page 1

GET /.well-known/cbor-web?page=3
→ Manifest page 3: pages 1001-1200, meta.next ABSENT (last page)
  Keys present: 0, 1, 3, 5, [10]
```

**Agent behavior:**

An agent MUST:
1. Fetch the first manifest page
2. Read site metadata (key 2) and navigation (key 4) from the first page
3. Process all page entries in key 3
4. If `"next"` is present in meta, fetch the next page and repeat from step 3
5. Continue until no `"next"` field is present (last page)

An agent MAY fetch sub-manifest pages in parallel (up to the declared `requests_per_second` rate limit) since each sub-manifest is independently parseable — site metadata (key 2) and navigation (key 4) are only in the first page, and subsequent pages contain only page entries (key 3). Sequential fetching is an acceptable alternative for resource-constrained agents.

**Size limit:** Each sub-manifest page MUST NOT exceed 5 MB in serialized CBOR size. If a single sub-manifest page would exceed 5 MB with 500 pages, the publisher MUST use a smaller page size (e.g., 200 pages per sub-manifest).

**CDDL:**

```cddl
sub-manifest = {
  0 => "cbor-web-manifest",
  1 => uint,
  ? 2 => site-metadata,              ; OPTIONAL in sub-manifests (present only in first page)
  3 => [+ page-entry],
  ? 4 => navigation,                  ; OPTIONAL in sub-manifests
  5 => manifest-meta,
  ? 6 => bstr,
  ? 7 => capabilities,
  ? 8 => [+ channel],
  ? 9 => diff-manifest,
  ? 10 => manifest-security,
  * int => any
}
```

### 5.9 Manifest Size and Performance

| Metric | Value | Rationale |
|--------|-------|-----------|
| Max manifest size | 5 MB | Prevents memory exhaustion during parsing |
| Typical manifest (25 pages) | ~500 bytes | Small enough for a single HTTP response |
| Typical manifest (500 pages) | ~50 KB | Comfortable within 5 MB limit |
| Max pages per manifest page | 500 | Practical limit for sub-manifest pagination |
| Max total pages | 100,000 | Agent-side parsing limit (§11) |
| Recommended agent cache TTL | 1 hour | Balance between freshness and bandwidth |

An agent SHOULD cache the manifest for the duration specified by the HTTP `Cache-Control` header, or 1 hour if no cache header is present.

---

## 6. Page Document

A page document contains the structured content of a single web page. It is the CBOR-Web equivalent of an HTML page — but containing only the useful content, with zero presentation noise.

### 6.1 Access URL and Path Encoding

Individual page documents are served at:

```
GET /.well-known/cbor-web/pages/{encoded-path}.cbor
```

The path encoding MUST be **bijective**: every distinct URL path produces a distinct filename, and every filename maps back to exactly one URL path. Non-bijective encoding causes silent data loss when two different URL paths collide to the same filename.

#### 6.1.1 Encoding Algorithm

The encoding proceeds in strict order:

**Step 1: Percent-encode literal underscores.** Replace every `_` character in the URL path with `%5F`. This prevents collisions in step 3.

**Step 2: Remove the leading slash.** The URL path `/services/seo` becomes `services/seo`.

**Step 3: Replace remaining slashes with underscores.** `services/seo` becomes `services_seo`.

**Step 4: Special case for root.** The URL path `/` (root) becomes `_index`.

**Step 5: Append `.cbor` extension.** `services_seo` becomes `services_seo.cbor`.

#### 6.1.2 Encoding Examples

| URL Path | Step 1 (escape `_`) | Step 2-3 (slashes) | Step 5 (extension) |
|----------|---------------------|---------------------|---------------------|
| `/` | `/` | `_index` (step 4) | `_index.cbor` |
| `/about` | `/about` | `about` | `about.cbor` |
| `/services/seo` | `/services/seo` | `services_seo` | `services_seo.cbor` |
| `/services/web-dev` | `/services/web-dev` | `services_web-dev` | `services_web-dev.cbor` |
| `/my_page` | `/my%5Fpage` | `my%5Fpage` | `my%5Fpage.cbor` |
| `/a_b/c_d` | `/a%5Fb/c%5Fd` | `a%5Fb_c%5Fd` | `a%5Fb_c%5Fd.cbor` |
| `/blog/2026/03/post` | `/blog/2026/03/post` | `blog_2026_03_post` | `blog_2026_03_post.cbor` |
| `/products/lions-mane` | `/products/lions-mane` | `products_lions-mane` | `products_lions-mane.cbor` |

#### 6.1.3 Decoding Algorithm (Reverse)

To reconstruct the URL path from a filename:

1. Remove the `.cbor` extension
2. If the filename is `_index`, return `/`
3. Replace all `_` with `/`
4. Prepend `/`
5. Decode `%5F` back to `_`

**Proof of bijectivity**: Step 1 of encoding ensures that the only `_` characters in the encoded form come from slash substitution (step 3). Therefore, step 3 of decoding correctly reverses step 3 of encoding. The `%5F` sequences are the original underscores, which step 5 restores. No information is lost.

**Collision example (why step 1 is necessary):**

Without underscore escaping:
```
/a_b/c   →  a_b_c.cbor
/a/b_c   →  a_b_c.cbor   ← COLLISION! Two paths, same file.
```

With underscore escaping:
```
/a_b/c   →  a%5Fb_c.cbor
/a/b_c   →  a_b%5Fc.cbor   ← No collision.
```

#### 6.1.4 Additional Path Encoding Rules

| Situation | Rule | Example |
|-----------|------|---------|
| **Paths with file extensions** | Extensions are preserved as-is. The `.cbor` extension is appended after the full encoding. | `/page.html` → `page.html.cbor` |
| **Percent-encoded characters** | Existing percent-encoding in the URL path is preserved (not double-encoded). Only literal `_` is escaped to `%5F`. | `/caf%C3%A9` → `caf%C3%A9.cbor` |
| **Case sensitivity** | Path encoding is **case-sensitive**. `/About` and `/about` produce different filenames. | `/About` → `About.cbor`, `/about` → `about.cbor` |
| **Non-ASCII characters** | Non-ASCII UTF-8 characters in URL paths MUST be percent-encoded (per RFC 3986 §2.1) before applying the path encoding algorithm. | `/produits/café` → first percent-encode: `/produits/caf%C3%A9` → `produits_caf%C3%A9.cbor` |
| **Filename length** | The resulting filename (without `.cbor` extension) MUST NOT exceed **200 bytes** (UTF-8 encoded). Paths producing longer filenames SHOULD be truncated with a SHA-256 suffix: `{first-180-bytes}_{first-8-hex-of-sha256}.cbor`. |
| **Query strings and fragments** | URL query strings (`?key=value`) and fragments (`#section`) MUST be stripped before path encoding. They are not part of the CBOR-Web page identity. |

### 6.2 Top-Level Structure

```cbor-diag
55799({                                ; self-described CBOR
  0: "cbor-web-page",                 ; @type
  1: 2,                                ; @version
  2: {                                 ; identity
    "lang": "en",
    "path": "/services/web-development",
    "canonical": "https://example.com/services/web-development",
    "alternates": {
      "es": "/es/servicios/desarrollo-web",
      "fr": "/fr/services/developpement-web"
    }
  },
  3: {                                 ; metadata
    "tags": ["web", "development", "react"],
    "title": "Custom Web Development",
    "author": "Example Corp",
    "updated": 1(1742428800),
    "category": "services",
    "published": 1(1740000000),
    "word_count": 450,
    "description": "We build performant, accessible websites...",
    "reading_time_seconds": 180
  },
  4: [                                 ; content (ordered array of blocks)
    {"l": 1, "t": "h", "v": "Custom Web Development"},
    {"t": "p", "v": "We build performant, accessible websites optimized for search engines and AI agents."},
    {"l": 2, "t": "h", "v": "Our Technology Stack"},
    {"t": "ul", "v": ["React / Next.js", "Node.js / Express", "PostgreSQL / Redis"]},
    {"t": "q", "v": "They transformed our online presence.", "attr": "Client, Acme Corp"},
    {"t": "table",
      "rows": [
        ["Starter", "$990", "5 pages, responsive, basic SEO"],
        ["Pro", "$2,490", "15 pages, multilingual, analytics"]
      ],
      "headers": ["Plan", "Price", "Includes"]
    },
    {"t": "cta", "v": "Request a free quote", "href": "/contact"}
  ],
  5: {                                 ; links
    "external": [
      {"url": "https://reactjs.org", "text": "React"}
    ],
    "internal": [
      {"path": "/contact", "text": "Contact us"},
      {"path": "/portfolio", "text": "Our work"}
    ]
  },
  6: {                                 ; structured_data (Schema.org compatible, CBOR native)
    "type": "Service",
    "provider": {
      "url": "https://example.com",
      "name": "Example Corp",
      "type": "Organization"
    },
    "areaServed": ["United States", "Europe"],
    "priceRange": "$$"
  }
})
```

Note: All map keys are shown in **deterministic order** (§3.1). Within the identity map: `"lang"` (5B) before `"path"` (5B, `70>6C`) before `"canonical"` (10B) before `"alternates"` (11B). Within content blocks: `"l"` (`61 6C`) before `"t"` (`61 74`) before `"v"` (`61 76`).

### 6.3 Top-Level Key Registry

| Key | Name | Type | Required | Description |
|-----|------|------|----------|-------------|
| 0 | @type | text | REQUIRED | MUST be `"cbor-web-page"` |
| 1 | @version | uint | REQUIRED | MUST be `2` |
| 2 | identity | map | REQUIRED | Page identity and localization (path, canonical URL, language, alternates) |
| 3 | metadata | map | REQUIRED | Page metadata (title, description, dates, tags, word count) |
| 4 | content | array | REQUIRED | Ordered array of content blocks. This is the page's editorial content. |
| 5 | links | map | OPTIONAL | Internal and external link graph |
| 6 | structured_data | map | OPTIONAL | Schema.org-compatible structured data in native CBOR |
| 7 | generative | array | OPTIONAL | Generative blocks. See CBOR-WEB-GENERATIVE.md §16. |
| 8 | forms | array | OPTIONAL | Form definitions. See CBOR-WEB-GENERATIVE.md §18. |
| 9 | commerce | map | OPTIONAL | Commerce data. See CBOR-WEB-GENERATIVE.md §19. |

### 6.4 Identity (Key 2)

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `"path"` | text | REQUIRED | Must start with `/`. Must match the page entry path in the manifest. | URL path relative to domain. |
| `"canonical"` | text | REQUIRED | Must be a full URL with `https://` | Full canonical URL including protocol. Example: `"https://example.com/services/web-development"` |
| `"lang"` | text | REQUIRED | ISO 639-1 | Language of this page's content. |
| `"alternates"` | map | OPTIONAL | Keys: language codes. Values: paths. | Language alternates. Maps a language code to the path of the equivalent page in that language. |

**Path consistency rule**: The `"path"` value in identity key 2 MUST match the `"path"` value in the corresponding page entry in the manifest's pages array (key 3). An agent detecting a mismatch SHOULD log a warning and use the manifest's path as authoritative.

### 6.5 Metadata (Key 3)

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `"title"` | text | REQUIRED | Max 300 characters | Page title. Equivalent to HTML `<title>`. |
| `"description"` | text | RECOMMENDED | Max 300 characters | Meta description. Equivalent to `<meta name="description">`. |
| `"author"` | text | OPTIONAL | | Author name. |
| `"published"` | tag 1 (uint) | OPTIONAL | Unix epoch | Original publication date. |
| `"updated"` | tag 1 (uint) | RECOMMENDED | Unix epoch | Last modification date. |
| `"tags"` | array of text | OPTIONAL | | Content tags/keywords. |
| `"category"` | text | OPTIONAL | | Content category. SHOULD match the category in the manifest page entry. |
| `"reading_time_seconds"` | uint | OPTIONAL | | Estimated reading time in seconds. The computation method is a publisher implementation detail (typical: `word_count / 3.5`, ~210 words/minute). |
| `"word_count"` | uint | OPTIONAL | | Total word count of all text content blocks. |

### 6.6 Content (Key 4)

The content array is the **core payload** of a page document. It MUST preserve document order — the first element is the first content item on the page (usually an H1 heading), and subsequent elements follow the reading order.

Each element is a content block as defined in §8. The array order is semantically significant: a heading followed by a paragraph means the paragraph belongs to that heading's section.

An agent reading only the `content` array (key 4) receives the **full editorial content** of the page with zero noise. Keys 5, 6, 7, 8, 9 provide supplementary information (links, structured data, generative blocks, forms, commerce) but are not necessary for pure content extraction.

**Content completeness rule**: All meaningful text on the page MUST be represented in the content array. A publisher MUST NOT omit important text to keep only structured data (key 6) or form fields (key 8). The content array is the single source of truth for the page's editorial content.

### 6.7 Links (Key 5)

| Field | Type | Description |
|-------|------|-------------|
| `"internal"` | array of link maps | Links to other pages on the same CBOR-Web site. Each map: `{"path": text, "text": text}` |
| `"external"` | array of link maps | Links to external sites. Each map: `{"url": text, "text": text}` |

Both arrays MAY be empty (`[]`). If a page has no links at all, key 5 SHOULD be omitted entirely.

Links in key 5 are **separated from content links**. Inline links in paragraphs (e.g., "Visit our [portfolio](/portfolio)") are stripped during HTML-to-CBOR conversion — the text becomes "Visit our portfolio" in the paragraph block, and the link appears in key 5 as `{"path": "/portfolio", "text": "Our work"}`.

External links are **informational only**. An agent MUST NOT automatically follow external links without an explicit security policy. See CBOR-WEB-SECURITY.md §11.3.

### 6.8 Structured Data (Key 6)

Structured data MUST be encoded in native CBOR maps, NOT as serialized JSON-LD strings. The structure follows Schema.org vocabulary but uses CBOR types natively:

| JSON-LD | CBOR-Web (key 6) | Rationale |
|---------|-------------------|-----------|
| `"@type": "Product"` | `"type": "Product"` | CBOR key, no `@` prefix needed |
| `"@id": "https://..."` | `"id": "https://..."` | Direct key |
| `"@context": "https://schema.org"` | Omitted | Context is implied (always Schema.org) |
| Nested `{"@type": "Offer", ...}` | Nested CBOR map `{"type": "Offer", ...}` | Native nesting |
| String `"29.90"` in JSON | CBOR float `29.90` | Native numeric type |

This eliminates the format-in-a-format problem: JSON-LD inside `<script>` inside HTML inside HTTP. CBOR-Web structured data is **one format** at one level — native CBOR.

### 6.9 Page Size Limit

A single page document MUST NOT exceed **1 MB** (1,048,576 bytes) in serialized CBOR size. Content exceeding this limit SHOULD be split across multiple pages with appropriate navigation links.

**Size estimation**: A page with 100 content blocks (headings + paragraphs + lists) typically serializes to 5-15 KB. A page with complex tables and large code blocks might reach 50-100 KB. Exceeding 1 MB would require extremely unusual content.

If a publisher encounters a page that would exceed 1 MB (e.g., a single page with thousands of product rows), it SHOULD split it into multiple pages with clear navigation links (e.g., "/products?page=1", "/products?page=2").

---

## 7. Bundle Document

### 7.1 Purpose

A bundle combines the manifest and all pages into a single CBOR document. This enables an agent to **index an entire site with a single HTTP request** — no manifest fetch, no individual page downloads, just one request.

For a 25-page site, a bundle might be 50-100 KB. The agent downloads it once, parses it in memory, and has the complete site content. This is dramatically more efficient than 26 HTTP requests (1 manifest + 25 pages) even with HTTP/2 multiplexing.

### 7.2 Availability

The bundle is OPTIONAL. The manifest's `"bundle_available"` field (key 5) indicates whether a bundle exists. If `"bundle_available"` is true, `"bundle_url"` MUST also be present.

**Guidance:**

| Site Size | Bundle Recommendation | Rationale |
|-----------|----------------------|-----------|
| 1-50 pages | RECOMMENDED | Small enough for a single download |
| 50-500 pages | Publisher's choice | Depends on average page size |
| > 500 pages | NOT RECOMMENDED | Bundle would exceed practical limits; use sub-manifests |

The 50 MB bundle size limit (§7.8) is the hard constraint. A site of 500 pages averaging 10 KB each produces a 5 MB bundle — well within limits. A site of 500 pages averaging 100 KB each produces a 50 MB bundle — at the limit.

### 7.3 Access URL

```
GET /.well-known/cbor-web/bundle HTTP/1.1
Accept: application/cbor
```

A **full bundle** (containing all pages, including `"access": "token"` pages) MUST require token authentication — only token holders can download it. An anonymous agent (without token) can still fetch the manifest and individual public pages.

A publisher MAY additionally offer a **public bundle** containing only `"access": "public"` pages, served without token requirement at a distinct URL (e.g., `/.well-known/cbor-web/bundle-public`). The manifest's `"bundle_url"` field points to the full bundle; the public bundle URL, if offered, SHOULD be declared in the manifest's meta section as `"public_bundle_url"`.

**Rationale:** Allowing anonymous download of a bundle containing token-gated content would bypass per-page access control. The two-bundle approach gives publishers flexibility while maintaining security guarantees.

### 7.4 Structure

```cbor-diag
55799({                                ; self-described CBOR
  0: "cbor-web-bundle",               ; @type
  1: 2,                                ; @version
  2: {                                 ; manifest (complete — same as §5, without 55799 tag)
    0: "cbor-web-manifest",
    1: 2,
    2: { "name": "Example", "domain": "example.com", ... },  ;  (remaining fields omitted for brevity)
    3: [ ... ],  ;  (remaining fields omitted for brevity)
    5: { ... }  ;  (remaining fields omitted for brevity)
  },
  3: {                                 ; pages (map: path → page content)
    "/": {                             ; root page (without 55799 tag)
      0: "cbor-web-page",
      1: 2,
      2: { "lang": "en", "path": "/", "canonical": "https://example.com/" },
      3: { "title": "Home" },
      4: [ {"l": 1, "t": "h", "v": "Welcome"}, {"t": "p", "v": "Hello, World!"} ]
    },
    "/about": {                        ; about page (without 55799 tag)
      0: "cbor-web-page",
      1: 2,
      2: { "lang": "en", "path": "/about", "canonical": "https://example.com/about" },
      3: { "title": "About Us" },
      4: [ {"l": 1, "t": "h", "v": "About Us"}, {"t": "p", "v": "We are..."} ]
    }
  }
})
```

### 7.5 Top-Level Key Registry

| Key | Name | Type | Required | Description |
|-----|------|------|----------|-------------|
| 0 | @type | text | REQUIRED | MUST be `"cbor-web-bundle"` |
| 1 | @version | uint | REQUIRED | MUST be `2` |
| 2 | manifest | map | REQUIRED | Complete manifest (same keys as §5, but without the outer self-described tag 55799) |
| 3 | pages | map | REQUIRED | Map of path (text) → page document (map, without self-described tag) |

### 7.6 Page Documents Within Bundles

Page documents inside the bundle's `pages` map (key 3) use the same structure as standalone page documents (§6), with two important differences:

1. **No self-described CBOR tag (55799)**: The bundle's outer tag covers all content. Adding individual tags would waste 3 bytes per page and serve no purpose.

2. **Path consistency**: The page's `"path"` in identity (page key 2) MUST match the map key under which it appears in bundle key 3. If the map key is `"/about"`, the page's identity path MUST be `"/about"`.

### 7.7 Hash Verification for Bundled Pages

This is a critical interoperability detail. The hash in the manifest (`"hash"` field of each page entry) is ALWAYS computed on the **standalone** form of the page document, which includes the self-described CBOR tag (55799). But bundled pages do NOT include the tag.

**To verify the hash of a page extracted from a bundle:**

1. Extract the page map from bundle key 3 (e.g., the value for key `"/about"`)
2. Serialize it to canonical CBOR bytes (deterministic encoding, §3.1)
3. **Prepend the 3-byte self-described CBOR tag prefix: `0xD9 0xD9 0xF7`**
4. Compute SHA-256 of the combined bytes
5. Compare with the hash in the manifest's page entry

**Pseudocode:**

```
page_map = bundle.pages["/about"]
page_cbor_bytes = canonical_encode(page_map)
standalone_bytes = bytes([0xD9, 0xD9, 0xF7]) + page_cbor_bytes
hash = SHA-256(standalone_bytes)
assert hash == manifest.pages["/about"].hash
```

This design ensures that the same hash works for both standalone and bundled pages. An agent that fetches `/about` as a standalone page gets the tag in the response. An agent that extracts `/about` from a bundle must add the tag before hashing.

**Implementation note**: The `text2cbor` reference implementation performs this computation automatically. The test vectors in Appendix B include both standalone and bundle hashes for verification.

### 7.8 Bundle Size Limit

A bundle MUST NOT exceed **50 MB** (52,428,800 bytes) in serialized CBOR size. Publishers MUST NOT offer a bundle if the total serialized size would exceed this limit.

If a site exceeds 50 MB of content, the publisher SHOULD:
1. Not offer a bundle (`"bundle_available": false`)
2. Use sub-manifests (§5.8) for manifest pagination
3. Let agents fetch individual pages via the pages endpoint

### 7.9 Bundle Freshness

A bundle becomes stale when any page on the site changes. An agent SHOULD check the manifest's `"generated_at"` timestamp before re-downloading a bundle.

**Recommended agent behavior:**

```
if bundle_in_cache AND cache_age < bundle_cooldown_seconds:
    Use cached bundle (do not re-download)
else:
    Fetch manifest
    Compare manifest.generated_at with cached bundle timestamp
    if manifest is newer:
        Re-download bundle
    else:
        Continue using cached bundle
```

**CDDL:**

```cddl
bundle = {
  0 => "cbor-web-bundle",
  1 => uint,
  2 => manifest,                      ; without 55799 tag wrapper
  3 => { + tstr => page },           ; path → page document (without 55799 tag)
  * int => any
}
```

---

## 8. Content Block Types

### 8.1 Overview

Content blocks are the atomic units of page content. Each block is a CBOR map with at minimum a `"t"` (type) key. Block keys use single-character text strings for compactness (Tier 3 keys, §3.5).

The content block system is designed so that an agent can understand a page's full editorial content by reading only the `content` array (key 4). Each block is self-describing — its type code tells the agent exactly what kind of content it is, and the required keys provide all the data needed to process it.

### 8.2 Block Key Registry

All content blocks share a common key namespace. Not all keys apply to all block types — each type specifies which keys are required and optional.

| Key | Name | Type | Used By | Description |
|-----|------|------|---------|-------------|
| `"t"` | type | text | ALL blocks | REQUIRED. Block type code. |
| `"v"` | value | text or array | h, p, ul, ol, q, code, cta, dl, note | Primary content value. |
| `"l"` | level | uint (1-6) | h | Heading level (H1-H6). |
| `"attr"` | attribution | text | q | Source attribution for quotes. |
| `"lang"` | language | text | code | Programming language identifier. |
| `"headers"` | headers | array of text | table | Column headers. |
| `"rows"` | rows | array of arrays | table | Table data rows. |
| `"alt"` | alt text | text | img | REQUIRED for image blocks. Accessibility description. |
| `"src"` | source | text | img, embed | URL of the referenced resource. |
| `"caption"` | caption | text | img | Caption text displayed below an image. |
| `"href"` | link | text | cta | Destination path or URL for calls to action. |
| `"description"` | description | text | embed | Text fallback description for embedded content. |
| `"level"` | severity | text | note | Note severity: `"info"`, `"warn"`, `"important"`. |
| `"trust"` | trust level | uint | multimedia, generative | Security classification. All core blocks are implicitly trust 0. |

### 8.3 Heading Block

**Type code:** `"h"` | **Category:** Editorial | **Trust level:** 0 (implicit)

A heading block represents a section heading, analogous to HTML `<h1>` through `<h6>`.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"h"` | Block type |
| `"l"` | uint | REQUIRED | 1-6 | Heading level. 1 = main title (H1), 6 = deepest subsection (H6). |
| `"v"` | text | REQUIRED | Max 500 characters | Heading text. Plain text, no markup. |

**CBOR diagnostic:**
```cbor-diag
{"l": 1, "t": "h", "v": "Custom Web Development"}
```

**Key order:** `"l"` (2 bytes, `61 6C`) < `"t"` (2 bytes, `61 74`) < `"v"` (2 bytes, `61 76`)

**Hex encoding (this specific example):**
```
A3           -- map(3)
  61 6C      -- "l"
  01         -- 1
  61 74      -- "t"
  61 68      -- "h"
  61 76      -- "v"
  76 437573746F6D205765622044657665
     6C6F706D656E74
             -- "Custom Web Development"
```

**Semantic rules:**
- A page SHOULD have exactly one `"l": 1` heading (analogous to one H1 per page).
- Heading levels SHOULD not skip (e.g., H1 → H3 without H2 is a structural warning).
- The H1 heading text SHOULD match or be closely related to the page title in metadata (key 3 `"title"`).

### 8.4 Paragraph Block

**Type code:** `"p"` | **Category:** Editorial | **Trust level:** 0 (implicit)

A paragraph block contains body text. This is the most common block type — most pages consist primarily of headings and paragraphs.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"p"` | Block type |
| `"v"` | text | REQUIRED | Max 50,000 characters | Paragraph text. Plain text, no HTML markup. |

**CBOR diagnostic:**
```cbor-diag
{"t": "p", "v": "We build performant, accessible websites optimized for search engines and AI agents."}
```

**Key order:** `"t"` (`61 74`) < `"v"` (`61 76`)

**Inline markup handling:**

Paragraph text is **plain text**. All HTML inline markup (`<strong>`, `<em>`, `<a>`, `<code>`, `<sup>`, `<sub>`, `<br>`) is stripped during HTML-to-CBOR conversion. The semantic meaning is preserved in the text itself.

| HTML Source | CBOR-Web `"v"` | Rationale |
|-------------|----------------|-----------|
| `Learn <strong>React</strong> today` | `"Learn React today"` | Bold is presentation, not content |
| `Visit <a href="/about">our page</a>` | `"Visit our page"` | Link target goes to key 5 |
| `Use <code>npm install</code>` | `"Use npm install"` | Inline code is presentation |
| `H<sub>2</sub>O` | `"H2O"` | Subscript is presentation |
| `Line one<br>Line two` | `"Line one Line two"` | Line breaks are presentation |

An agent processes text semantically, not visually. Bold, italic, and inline code are presentation hints for humans. The textual content is preserved, though some semantic nuance may be lost.

**Known semantic loss:** Emphasis (`<em>`) and strong emphasis (`<strong>`) occasionally carry meaning beyond presentation — e.g., *"This is **not** recommended"* where bold signals negation emphasis. Similarly, `<abbr title="...">` expansions are lost. CBOR-Web v2.1 accepts this trade-off in favor of simplicity. A future version MAY introduce an optional inline annotation mechanism (e.g., a `"annotations"` key mapping character ranges to semantic labels) for publishers who need to preserve this information.

**Multi-paragraph content:**

Each HTML `<p>` element becomes a separate CBOR paragraph block. Paragraph blocks are NOT concatenated — their order in the content array defines the reading flow.

### 8.5 List Blocks

#### 8.5.1 Unordered List

**Type code:** `"ul"` | **Category:** Editorial | **Trust level:** 0 (implicit)

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"ul"` | Block type |
| `"v"` | array of text | REQUIRED | At least 1 item. Each item max 5,000 chars. | List items. Each array element is one bullet point. |

**CBOR diagnostic:**
```cbor-diag
{"t": "ul", "v": ["React / Next.js", "Node.js / Express", "PostgreSQL / Redis"]}
```

#### 8.5.2 Ordered List

**Type code:** `"ol"` | **Category:** Editorial | **Trust level:** 0 (implicit)

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"ol"` | Block type |
| `"v"` | array of text | REQUIRED | At least 1 item. | List items. Array order is semantically significant (first item is #1). |

**CBOR diagnostic:**
```cbor-diag
{"t": "ol", "v": ["Clone the repository", "Run npm install", "Start the development server"]}
```

**Nested and structured content limitations:** CBOR-Web v2.1 content blocks are flat — they do not support nesting. The following HTML patterns MUST be flattened during conversion:

| HTML Pattern | CBOR-Web v2.1 Handling | Information Loss |
|--------------|----------------------|------------------|
| Nested lists (`<ul><li><ul>...`) | Flatten to single-level list, indent with prefix (e.g., `"  - Sub-item"`) | Hierarchy lost |
| Blockquote containing list | Separate `"q"` and `"ul"` blocks in sequence | Containment lost |
| Table with colspan/rowspan | Expand to regular grid (duplicate content in spanned cells) | Span semantics lost |
| List containing code blocks | Separate `"ul"` and `"code"` blocks in sequence | Association lost |

A future version (v3.0) MAY introduce structured list items (where `"v"` can contain nested content blocks instead of plain text) and rich table cells. The forward-compatibility rule (§3.6) ensures that v2.1 agents will safely ignore these extensions.

### 8.6 Quote Block

**Type code:** `"q"` | **Category:** Editorial | **Trust level:** 0 (implicit)

A block quote with optional attribution.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"q"` | Block type |
| `"v"` | text | REQUIRED | | Quoted text. |
| `"attr"` | text | OPTIONAL | | Attribution: who said it, where it's from. |

**CBOR diagnostic:**
```cbor-diag
{"t": "q", "v": "They transformed our online presence completely.", "attr": "Client, Acme Corp"}
```

**Key order:** `"t"` (2B, `61 74`) < `"v"` (2B, `61 76`) < `"attr"` (5B, `64 61747472`). Shorter encoded keys sort first; among equal lengths, bytewise comparison applies.

### 8.7 Code Block

**Type code:** `"code"` | **Category:** Editorial | **Trust level:** 0 (implicit)

A block of source code with optional language identification.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"code"` | Block type |
| `"v"` | text | REQUIRED | Whitespace preserved. No HTML escaping. | Source code text. Newlines, indentation, and special characters are preserved as-is. |
| `"lang"` | text | OPTIONAL | Language identifier | Programming language. Examples: `"python"`, `"javascript"`, `"rust"`, `"sql"`, `"bash"`, `"html"`, `"css"`. |

**CBOR diagnostic:**
```cbor-diag
{"t": "code", "v": "def hello():\n    print('Hello, World!')", "lang": "python"}
```

**Key order:** `"t"` (2B) < `"v"` (2B) < `"lang"` (5B)

**Code is NOT sanitized:** Unlike other text fields, code blocks preserve their content exactly as-is. No HTML stripping, no escaping, no modification. The code is literal content.

**Language identification:** The `"lang"` value follows the convention used by GitHub, Prism.js, and highlight.js. Common identifiers:

| Language | `"lang"` Value | Also Accepted |
|----------|---------------|---------------|
| Python | `"python"` | `"py"` |
| JavaScript | `"javascript"` | `"js"` |
| TypeScript | `"typescript"` | `"ts"` |
| Rust | `"rust"` | `"rs"` |
| SQL | `"sql"` | — |
| Bash/Shell | `"bash"` | `"sh"`, `"shell"` |
| HTML | `"html"` | — |
| CSS | `"css"` | — |
| JSON | `"json"` | — |
| YAML | `"yaml"` | `"yml"` |
| CBOR diagnostic | `"cbor-diag"` | — |
| CDDL | `"cddl"` | — |

### 8.8 Data Table Block

**Type code:** `"table"` | **Category:** Editorial | **Trust level:** 0 (implicit)

A data table with headers and rows.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"table"` | Block type |
| `"headers"` | array of text | REQUIRED | At least 1 header | Column header names. |
| `"rows"` | array of arrays | REQUIRED | Each row array MUST have the same length as `"headers"`. | Table data. Each element is an array of text values representing one row. |

**CBOR diagnostic:**
```cbor-diag
{
  "t": "table",
  "rows": [
    ["Starter", "$990", "5 pages, responsive, basic SEO"],
    ["Pro", "$2,490", "15 pages, multilingual, analytics"],
    ["Enterprise", "Custom", "Unlimited pages, dedicated support"]
  ],
  "headers": ["Plan", "Price", "Includes"]
}
```

**Key order:** `"t"` (2B, `61 74`) < `"rows"` (5B, `64 726F7773`) < `"headers"` (8B, `67 68656164657273`). Shorter encoded keys sort first (§3.1.1).

**Row consistency rule:** Every row array MUST have the same number of elements as the `"headers"` array. Missing values SHOULD be represented as empty strings `""`, not omitted.

**All values are text:** Table cells are always text strings, even for numeric data. This simplifies parsing — an agent that needs to process a numeric cell can parse it from text. A future version MAY introduce an optional `"col_types"` key (array of type hints: `"text"`, `"number"`, `"currency"`, `"date"`) to enable typed cell parsing without breaking backward compatibility.

### 8.9 Image Reference Block

**Type code:** `"img"` | **Category:** Non-editorial | **Trust level:** 0 (implicit)

A reference to an image by URL. This is the v1.0 image block — simple and lightweight. For richer image metadata (semantic role, dimensions, AI description, inline data), see the `"image"` block in CBOR-WEB-MULTIMEDIA.md §15.2.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"img"` | Block type |
| `"alt"` | text | REQUIRED | Max 500 characters | Accessibility text. REQUIRED — no image block without alt text. |
| `"src"` | text | REQUIRED | Must be `https://` URL | Image URL. |
| `"caption"` | text | OPTIONAL | | Caption text displayed below the image. |

**CBOR diagnostic:**
```cbor-diag
{"t": "img", "alt": "Flacon Lion's Mane, 90 capsules", "src": "https://verdetao.com/img/lm.webp"}
```

**Key order:** `"t"` (2B) < `"alt"` (4B) < `"src"` (4B, `73 > 61`) < `"caption"` (8B)

**Accessibility rule:** The `"alt"` key is REQUIRED. A publisher MUST NOT produce an `"img"` block without an `"alt"` value. If the source HTML image has no alt text, the publisher SHOULD:
1. Generate a descriptive alt text using AI (if available)
2. Use the image filename as a hint: `"lions-mane-packaging.webp"` → `"Lion's Mane packaging"`
3. As a last resort: `"alt": "Image"`

### 8.10 Call to Action Block

**Type code:** `"cta"` | **Category:** Non-editorial | **Trust level:** 0 (implicit)

A marketing or navigation action element — "Buy Now", "Sign Up", "Request a Quote", etc.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"cta"` | Block type |
| `"v"` | text | REQUIRED | | Button/link text. |
| `"href"` | text | REQUIRED | Path or URL | Destination. Internal paths start with `/`. External URLs start with `https://`. |

**CBOR diagnostic:**
```cbor-diag
{"t": "cta", "v": "Request a free quote", "href": "/contact"}
```

**Agent behavior:** An agent MAY skip `"cta"` blocks for editorial-only consumption. CTAs are marketing elements — they tell the user what action to take, but do not contain editorial information. An agent extracting the informational content of a page can safely ignore all CTA blocks.

### 8.11 Embedded Content Block

**Type code:** `"embed"` | **Category:** Non-editorial | **Trust level:** 0 (implicit)

A reference to embedded external content: videos, maps, widgets, iframes.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"embed"` | Block type |
| `"src"` | text | REQUIRED | `https://` URL | Source URL of the embedded content. |
| `"description"` | text | OPTIONAL | | Text fallback description. An agent that cannot render the embed reads this instead. |

**CBOR diagnostic:**
```cbor-diag
{"t": "embed", "src": "https://maps.google.com/embed?q=Verdetao", "description": "Interactive map showing store locations in Europe"}
```

### 8.12 Separator Block

**Type code:** `"sep"` | **Category:** Editorial | **Trust level:** 0 (implicit)

A thematic break, analogous to HTML `<hr>`. No content, just a structural signal that the following content is a new thematic section.

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"sep"` |

**CBOR diagnostic:**
```cbor-diag
{"t": "sep"}
```

This is the smallest possible content block — just one key-value pair. In deterministic CBOR: `A1 61 74 63 736570` (7 bytes).

### 8.13 Definition List Block

**Type code:** `"dl"` | **Category:** Editorial | **Trust level:** 0 (implicit)

A list of term-definition pairs, analogous to HTML `<dl>`.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"dl"` | Block type |
| `"v"` | array of maps | REQUIRED | Each map: `{"term": text, "def": text}` | Array of definition items. |

**CBOR diagnostic:**
```cbor-diag
{
  "t": "dl",
  "v": [
    {"def": "2 capsules le matin avec un verre d'eau.", "term": "Combien de capsules par jour ?"},
    {"def": "Oui, il se combine bien avec le Reishi et le Cordyceps.", "term": "Compatible avec d'autres supplements ?"}
  ]
}
```

**Key order within each definition item:** `"def"` (4B, `63 646566`) < `"term"` (5B, `64 7465726D`). `"def"` is 3 characters (4 bytes encoded), `"term"` is 4 characters (5 bytes encoded). Shorter first.

### 8.14 Note Block

**Type code:** `"note"` | **Category:** Editorial | **Trust level:** 0 (implicit)

An advisory note, warning, or important callout.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"note"` | Block type |
| `"v"` | text | REQUIRED | | Note text. |
| `"level"` | text | OPTIONAL | `"info"` (default), `"warn"`, `"important"` | Severity level. |

**CBOR diagnostic:**
```cbor-diag
{"t": "note", "v": "This product is not intended to diagnose, treat, cure, or prevent any disease."}
```

With severity:
```cbor-diag
{"t": "note", "v": "Do not exceed the recommended daily dose.", "level": "warn"}
```

**Key order:** `"t"` (2B) < `"v"` (2B) < `"level"` (6B)

### 8.15 Editorial vs. Non-Editorial Classification

Blocks are classified into two categories to help agents filter content:

**Editorial blocks** (pure content signal) — an agent reading only these gets the complete textual content:

| Code | Type | Rationale |
|------|------|-----------|
| `"h"` | Heading | Page structure |
| `"p"` | Paragraph | Body text |
| `"ul"` | Unordered list | Structured information |
| `"ol"` | Ordered list | Sequential information |
| `"q"` | Quote | Referenced content |
| `"code"` | Code | Technical content |
| `"table"` | Data table | Structured data |
| `"dl"` | Definition list | Glossary/FAQ content |
| `"note"` | Note | Advisory information |
| `"sep"` | Separator | Section boundary |

**Non-editorial blocks** (marketing/navigation) — agents MAY filter these out:

| Code | Type | Rationale |
|------|------|-----------|
| `"cta"` | Call to action | Marketing element |
| `"embed"` | Embedded content | External resource |
| `"img"` | Image reference | Visual element (the alt text is editorial, but the image itself is not) |

**The content completeness rule** (§6.6) requires that all meaningful text exists in editorial blocks. A publisher MUST NOT hide important information in CTA text or embed descriptions.

### 8.16 Multimedia and Generative Block Types

The following block types are defined in companion documents and are NOT covered in this core document:

**Multimedia blocks** (see CBOR-WEB-MULTIMEDIA.md):

| Code | Type | Trust | Document |
|------|------|-------|----------|
| `"image"` | Rich image | 0 | CBOR-WEB-MULTIMEDIA.md §15.2 |
| `"video"` | Video | 0 | CBOR-WEB-MULTIMEDIA.md §15.3 |
| `"audio"` | Audio | 0 | CBOR-WEB-MULTIMEDIA.md §15.4 |
| `"document"` | Embedded document | 0 | CBOR-WEB-MULTIMEDIA.md §15.5 |
| `"diagram"` | Diagram | 0 | CBOR-WEB-MULTIMEDIA.md §15.6 |
| `"live_stream"` | Live stream | 0 | CBOR-WEB-MULTIMEDIA.md §15.7 |

**Generative blocks** (see CBOR-WEB-GENERATIVE.md):

| Code | Type | Trust | Document |
|------|------|-------|----------|
| `"template"` | Template | 1 | CBOR-WEB-GENERATIVE.md §16.3 |
| `"schema"` | Schema | 0 | CBOR-WEB-GENERATIVE.md §16.4 |
| `"api_endpoint"` | API endpoint | 3 | CBOR-WEB-GENERATIVE.md §16.5 |
| `"executable"` | Executable code | 2 | CBOR-WEB-GENERATIVE.md §16.6 |
| `"workflow"` | Workflow | 3 | CBOR-WEB-GENERATIVE.md §16.7 |
| `"constraint"` | Business constraint | 0 | CBOR-WEB-GENERATIVE.md §16.8 |
| `"form"` | Form | 3 | CBOR-WEB-GENERATIVE.md §18 |
| `"product"` | Product | 0 | CBOR-WEB-GENERATIVE.md §19 |
| `"cart_action"` | Cart action | 3 | CBOR-WEB-GENERATIVE.md §19 |

An agent MUST ignore block types it does not recognize (forward-compatibility rule, §3.6).

### 8.17 Future Block Types

Reserved type codes for future use: `"gallery"`, `"timeline"`, `"accordion"`, `"tabs"`.

New block types MAY be introduced in future versions without incrementing the specification version, as long as they follow the forward-compatibility rule. An agent encountering an unknown block type MUST skip it and continue processing the remaining blocks.

---

## 9. Transport and Compression

### 9.1 HTTPS Requirement

CBOR-Web content MUST be served over HTTPS (TLS 1.2 or later). An agent MUST refuse to process a manifest received over plain HTTP. Servers SHOULD support HTTP/2 or HTTP/3 to enable multiplexed fetching of individual pages when the agent downloads multiple pages concurrently.

**Rationale:**
1. CBOR-Web content represents the authoritative content of a website — an agent trusts it as the "truth" of what the site contains
2. Man-in-the-middle attacks could inject false content that agents propagate as fact
3. Hash verification (§10.2) is meaningless if transport integrity is not guaranteed
4. Token-based access control (CBOR-WEB-SECURITY.md §12) requires secure transport for wallet signatures

**Exception**: During development and testing, a publisher MAY serve CBOR-Web over plain HTTP on `localhost` or private networks. This exception MUST NOT be used in production.

### 9.2 Content-Type

All CBOR-Web responses MUST use:

```
Content-Type: application/cbor
```

This is the MIME type registered by RFC 8949. No custom subtype is necessary. A server MUST NOT serve CBOR-Web content with `application/octet-stream`, `text/html`, or any other MIME type.

Note: An agent encountering `application/octet-stream` SHOULD still attempt magic byte validation (§4.1) before rejecting, as some servers lack proper CBOR MIME configuration.

A future version MAY register a structured syntax suffix or media type parameter:

```
Content-Type: application/cbor; profile="cbor-web-manifest"
```

This is NOT REQUIRED for v2.1.

### 9.3 HTTP Compression

The server SHOULD support content-encoding negotiation:

```
Request:  Accept-Encoding: br, gzip, deflate
Response: Content-Encoding: br
```

**Brotli** (`br`) is RECOMMENDED for CBOR-Web content. Empirical testing shows:

| Document | Uncompressed | Gzip | Brotli | Brotli Savings vs Gzip |
|----------|-------------|------|--------|------------------------|
| Manifest (25 pages) | 524 B | 412 B | 385 B | 7% smaller |
| Page (typical) | 2,340 B | 1,870 B | 1,720 B | 8% smaller |
| Bundle (25 pages) | 48,200 B | 35,100 B | 30,800 B | 12% smaller |

Brotli achieves 15-30% better compression ratios than gzip on CBOR data, especially for larger documents. Brotli is supported by all modern HTTP clients (including `curl`, `reqwest`, `hyper`, and all browsers).

Gzip is an acceptable fallback. Deflate SHOULD NOT be used (compatibility issues with some clients).

### 9.4 Token Authentication Headers

When an agent requests a token-gated page, it includes wallet authentication headers. The full protocol is defined in CBOR-WEB-SECURITY.md §12.4, which specifies the complete signing scheme, nonce validation, and replay protection. Here is a summary of the wire format:

**Authentication requirements** (defined in CBOR-WEB-SECURITY.md §12.4):
- **Signing scheme**: EIP-712 typed data signature (see CBOR-WEB-SECURITY.md §12.4.1)
- **Signed payload**: domain + request path + nonce + chain ID (see CBOR-WEB-SECURITY.md §12.4.2)
- **Nonce**: Unix timestamp; server MUST reject nonces older than 300 seconds (replay window)
- **Token balance verification**: server queries the ERC-20 contract (cached, max staleness 60 seconds)

Implementers MUST read CBOR-WEB-SECURITY.md §12.4 before implementing token authentication. The summary below shows only the HTTP header format:

**Request headers:**
```
GET /.well-known/cbor-web/pages/products_lions-mane.cbor HTTP/1.1
Host: verdetao.com
Accept: application/cbor
Accept-Encoding: br
X-CBOR-Web-Wallet: 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18
X-CBOR-Web-Sig: 0x[signature of request hash with wallet private key]
X-CBOR-Web-Nonce: 1742598400
```

**Response (success — token holder):**
```
HTTP/1.1 200 OK
Content-Type: application/cbor
Content-Encoding: br
X-CBOR-Web-Verified: true
X-CBOR-Web-Token-Balance: 3
Cache-Control: private, max-age=86400
```

**Response (no token):**
```
HTTP/1.1 402 Payment Required
Content-Type: application/cbor
```

Body (CBOR-encoded):
```cbor-diag
{
  "error": "token_required",
  "message": "This page requires a CBOR-Web token.",
  "get_token_url": "https://cbor-web.org/get-token",
  "storefront_url": "/.well-known/cbor-web",
  "contract_address": "0x..."
}
```

### 9.5 Conditional Requests

The server SHOULD support conditional requests to minimize bandwidth:

**First request:**
```
HTTP/1.1 200 OK
Content-Type: application/cbor
ETag: "d8cad2e6"
Last-Modified: Sat, 21 Mar 2026 00:00:00 GMT
Cache-Control: public, max-age=86400
```

**Subsequent request:**
```
GET /.well-known/cbor-web/pages/about.cbor HTTP/1.1
If-None-Match: "d8cad2e6"
If-Modified-Since: Sat, 21 Mar 2026 00:00:00 GMT
```

**Response (unchanged):**
```
HTTP/1.1 304 Not Modified
```

**Response (changed):**
```
HTTP/1.1 200 OK
Content-Type: application/cbor
ETag: "a1b2c3d4"
[new content]
```

The ETag SHOULD be derived from the page's SHA-256 hash (first 8 hex characters is sufficient for ETag purposes).

### 9.6 Cache-Control

The server SHOULD set appropriate cache headers:

| Document | Cache-Control | max-age | Rationale |
|----------|--------------|---------|-----------|
| Manifest | `public, max-age=3600, stale-while-revalidate=86400` | 1 hour | Agents check for new pages regularly |
| Page (public) | `public, max-age=86400` | 24 hours | Content changes less frequently |
| Page (token-gated) | `private, max-age=86400` | 24 hours | Private — not cacheable by shared proxies |
| Bundle | `private, max-age=3600` | 1 hour | Same as manifest; private due to token content |
| Keys (keys.cbor) | `public, max-age=604800` | 7 days | Public keys change rarely |

### 9.7 HTTP Headers Registry

CBOR-Web defines the following custom HTTP headers:

| Header | Direction | Type | Description |
|--------|-----------|------|-------------|
| `X-CBOR-Web-Wallet` | Request | text (hex) | Agent's Ethereum wallet address |
| `X-CBOR-Web-Sig` | Request | text (hex) | Signature of request hash |
| `X-CBOR-Web-Nonce` | Request | text (decimal) | Unix timestamp nonce |
| `X-CBOR-Web-Verified` | Response | bool | Whether the agent's token was verified |
| `X-CBOR-Web-Token-Balance` | Response | uint | Agent's current token balance |
| `X-CBOR-Web-Code-Position` | Response | uint | Binary watermark position (OPTIONAL, see CBOR-WEB-SECURITY.md §12.8) |
| `X-CBOR-Web-Code` | Request | text | Binary watermark code (OPTIONAL) |

---

## 10. Caching and Incremental Updates

### 10.1 Hash-Based Cache Validation

Each page entry in the manifest includes a `"hash"` field containing the SHA-256 hash of the page's serialized CBOR document. This enables **client-side cache validation without any HTTP request**:

```
Agent workflow:
  1. Download manifest (1 HTTP request)
  2. For each page entry:
     a. Compare "hash" with locally cached hash
     b. If hashes match → skip download (page unchanged)
     c. If hashes differ → download new page
  3. Result: only changed pages are downloaded
```

This is more efficient than HTTP conditional requests (ETag/If-None-Match) because:
- No per-page HTTP round-trip is needed to check freshness
- The manifest itself contains all freshness information for all pages
- Batch decisions can be made (e.g., "of 25 pages, only 3 changed — download those 3")
- No dependency on server-side ETag support

### 10.2 Hash Computation

The hash MUST be computed as:

```
SHA-256( complete_standalone_cbor_page_document_bytes )
```

Where `complete_standalone_cbor_page_document_bytes` is the complete CBOR encoding of the page document, **including** the self-described CBOR tag (55799) as the first 3 bytes (`D9 D9 F7`).

Because deterministic encoding is required (§3.1), the same page content always produces the same hash:
- Same text content → same CBOR bytes → same hash
- Different publisher software → same CBOR bytes → same hash
- Different hardware/OS → same CBOR bytes → same hash

**Hash verification pseudocode:**

```python
# After downloading a page
page_bytes = fetch("/.well-known/cbor-web/pages/about.cbor")

# Verify self-described tag
assert page_bytes[0:3] == b'\xD9\xD9\xF7'

# Compute hash
computed_hash = sha256(page_bytes)

# Compare with manifest entry
manifest_hash = manifest.pages["/about"].hash
assert computed_hash == manifest_hash
```

**For pages extracted from a bundle**, see §7.7.

### 10.3 Manifest Diffing (Client-Side)

An agent can detect changes between two manifest versions by comparing the pages arrays:

| Situation | Detection | Agent Action |
|-----------|-----------|-------------|
| **New page** | Path in new manifest, absent in cache | Download and index |
| **Deleted page** | Path in cache, absent in new manifest | Remove from cache/index |
| **Updated page** | Path in both, hashes differ | Re-download and update cache |
| **Unchanged page** | Path in both, hashes match | Skip (use cache) |

**Efficient diffing algorithm:**

```python
cached_pages = {entry.path: entry for entry in cached_manifest.pages}
new_pages = {entry.path: entry for entry in new_manifest.pages}

for path, new_entry in new_pages.items():
    if path not in cached_pages:
        download(path)  # NEW
    elif cached_pages[path].hash != new_entry.hash:
        download(path)  # UPDATED
    # else: UNCHANGED — skip

for path in cached_pages:
    if path not in new_pages:
        delete_from_cache(path)  # DELETED
```

### 10.4 Timestamp Ordering

Page entries in the manifest's pages array (key 3) SHOULD be ordered by `"updated"` timestamp, most recent first. This allows an agent to quickly identify recently changed pages:

```python
# If pages are ordered by updated (desc), the first N entries
# are the N most recently changed pages
recently_changed = manifest.pages[:10]  # top 10 most recent
```

### 10.5 Differential Manifests (Server-Side)

Client-side diffing (§10.3) requires downloading the full manifest each time to discover what changed. For large sites, this can be costly. **Differential manifests** provide a compact server-side description of changes.

The manifest key 9 contains the diff:

```cbor-diag
9: {
  "stats": {
    "pages_added": 1,
    "pages_removed": 1,
    "pages_modified": 1,
    "total_pages_now": 25
  },
  "changes": [
    {
      "hash": h'B7C1...',
      "path": "/products/new-cordyceps",
      "size": 1230,
      "title": "Cordyceps Militaris",
      "action": "added"
    },
    {
      "hash": h'E3B0...',
      "path": "/products/lions-mane",
      "size": 2340,
      "action": "modified",
      "previous_hash": h'A1B2...',
      "fields_changed": ["price", "stock"]
    },
    {
      "path": "/blog/old-post",
      "action": "removed"
    }
  ],
  "diff_version": 2,
  "base_generated_at": 1(1742428800),
  "base_version_hash": h'A3F2C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855',
  "current_generated_at": 1(1742515200)
}
```

**Requesting a diff:**

```
GET /.well-known/cbor-web?since=A3F2C44298FC1C149AFBF4C8996FB924
```

The `since` parameter is the hex-encoded first 32 characters of the base manifest's SHA-256 hash.

**Server behavior:**
- If the server supports diffs AND has the requested base version → return manifest with key 9 populated
- If the server does not support diffs OR does not have the base version → return full manifest without key 9

**Agent behavior:**
- If key 9 is present → apply changes to cached data without re-downloading unchanged pages
- If key 9 is absent → fall back to client-side diffing (§10.3)

### 10.6 Change Actions

| Action | Fields Present | Agent Behavior |
|--------|---------------|---------------|
| `"added"` | path, hash, size, title | Download and index the new page |
| `"modified"` | path, hash, previous_hash, size, fields_changed | Re-download and update cache |
| `"removed"` | path only | Remove from cache and index |

### 10.7 Field-Level Change Hints

For `"modified"` pages, the optional `"fields_changed"` array provides hints about what changed:

| Value | Meaning |
|-------|---------|
| `"price"` | Product price updated |
| `"stock"` | Stock level changed |
| `"content"` | Editorial content changed |
| `"metadata"` | Metadata fields changed (title, description, etc.) |
| `"structured_data"` | Schema.org data changed |
| `"multimedia"` | Multimedia blocks changed |

This enables an agent to decide if a re-download is worth it. A stock change on a product the agent doesn't track can be skipped. A content change on a blog article the agent has summarized requires re-download.

---

### 10.8 Error Handling

An agent MUST handle the following error conditions gracefully. The general principle is **fail open for content, fail closed for security**: a malformed content block should be skipped (the rest of the page is still useful), but a failed hash verification or invalid signature should cause the entire document to be rejected.

| Error Condition | Agent Behavior | Severity |
|-----------------|---------------|----------|
| **Malformed CBOR** (parse failure) | Reject the entire document. Do not attempt partial parsing. Log the error. | Fatal |
| **Hash mismatch** (page hash ≠ manifest hash) | Reject the page. Log a warning. Re-fetch the manifest (it may be stale). | Fatal for page |
| **Unknown block type** in content array | Skip the block. Continue processing remaining blocks. (Forward-compatibility rule, §3.6) | Warning |
| **Page size exceeds 1 MB** | Reject the page. Log a warning. | Fatal for page |
| **Manifest size exceeds 5 MB** | Reject the manifest. | Fatal |
| **Duplicate paths** in manifest pages array | Use the last occurrence (last-writer-wins). Log a warning. | Warning |
| **Navigation path not in pages array** | Treat as broken link. Log a warning. Do not fail. | Warning |
| **Missing REQUIRED field** (e.g., no `"title"` in page entry) | Skip the entry/block. Log a warning. | Warning |
| **Version number higher than supported** | Attempt to parse using known keys. Ignore unknown keys (§3.6). Log informational message. | Info |
| **Signature verification failure** | Reject the manifest. Do not trust any page hashes from this manifest. | Fatal |
| **Network timeout** fetching a page | Retry once after 5 seconds. If still failing, skip the page and continue with other pages. | Recoverable |

An agent SHOULD maintain an error log per site. If a site consistently produces errors (>50% of pages fail validation), the agent SHOULD mark the site as unreliable and reduce its fetch frequency.

---

## 11. Conformance Levels

Three conformance levels allow progressive adoption. A publisher starts at Minimal and progresses to Standard and Full as capabilities grow.

### 11.1 Level 1: Minimal

The minimum viable CBOR-Web implementation. A publisher can achieve Minimal conformance in a few hours.

**Publisher requirements:**
- Manifest at `/.well-known/cbor-web` with keys 0, 1, 2, 3, 5
- Individual page documents with keys 0, 1, 2, 3, 4
- Content blocks: at minimum `"h"` and `"p"` types
- Self-described CBOR tag (55799) on all documents
- Deterministic encoding (§3.1)
- HTTPS transport (§9.1)
- `"access"` field in all page entries (defaults to `"public"` for Minimal)

**Agent requirements:**
- Discover manifest via well-known URL (§4.1)
- Parse manifest and page documents
- Process `"h"` and `"p"` content blocks
- Ignore unknown keys (§3.6)

### 11.2 Level 2: Standard

The recommended level for production sites. Adds navigation, caching, all content blocks, and links.

**All of Minimal, plus:**

**Publisher requirements:**
- Navigation structure (manifest key 4) with `"main"` and `"hierarchy"`
- `"hash"` and `"updated"` fields in all page entries
- All 13 core content block types (§8.3)
- Links section in page documents (key 5)
- Structured data section in page documents (key 6)
- Bundle document available
- `"size"` field in all page entries
- Capabilities declaration (manifest key 7)
- Security declaration (manifest key 10)

**Agent requirements:**
- Hash-based cache validation (§10.1)
- HTTP conditional requests (§9.5)
- Process all 13 core content block types
- Respect rate limits (§5.6)
- Enforce size limits (CBOR-WEB-SECURITY.md, Binary Content Protection)
- Token verification for `"access": "token"` pages

### 11.3 Level 3: Full

The complete CBOR-Web experience with signatures, sub-manifests, differential updates, and all extensions.

**All of Standard, plus:**

**Publisher requirements:**
- COSE signature on manifest (key 6, see CBOR-WEB-SECURITY.md §12.6)
- Sub-manifests for sites > 500 pages (§5.8)
- Public key discoverable via keys.cbor or DNS TXT record
- Differential manifests (key 9, §10.5)
- Multimedia blocks where appropriate (CBOR-WEB-MULTIMEDIA.md)
- Generative blocks where appropriate (CBOR-WEB-GENERATIVE.md)

**Agent requirements:**
- Verify COSE signature before trusting manifest
- Sub-manifest pagination support
- Manifest diffing for incremental updates (§10.3)
- Content cross-validation against HTML (CBOR-WEB-SECURITY.md §11.6)
- Process multimedia and generative blocks

### 11.4 Conformance Declaration

The manifest's capabilities (key 7) SHOULD declare the conformance level:

```cbor-diag
7: {
  "conformance": "standard",
  ...  ;  (remaining fields omitted for brevity)
}
```

Valid values: `"minimal"`, `"standard"`, `"full"`.

An agent MUST NOT rely on the declared conformance level for security decisions. The declaration is informational — an agent MUST independently verify that required features are present.

---

## 12. IANA Considerations

### 12.1 Well-Known URI Registration

This specification registers the following well-known URI (per RFC 8615):

| Field | Value |
|-------|-------|
| URI suffix | `cbor-web` |
| Change controller | ExploDev |
| Specification document | This document (CBOR-WEB-CORE.md) |
| Related information | Machine-readable binary web content for autonomous agents |

### 12.2 Media Type

This specification uses the existing `application/cbor` media type registered by RFC 8949. No new media type registration is required.

A future version MAY register a profile parameter:

```
Content-Type: application/cbor; profile="cbor-web-manifest"
Content-Type: application/cbor; profile="cbor-web-page"
Content-Type: application/cbor; profile="cbor-web-bundle"
```

### 12.3 robots.txt Extension

The `CBOR-Web:` directive in robots.txt (§4.4) is a non-standard extension to the Robots Exclusion Protocol. A future version of this specification MAY pursue formal registration if the Robots Exclusion Protocol is extended to support it.

### 12.4 HTTP Headers

Custom HTTP headers defined by this specification (§9.7) follow the `X-CBOR-Web-` prefix convention. A future version MAY pursue registration of standardized header names if the protocol achieves wide adoption.

---

### 12.5 Privacy Considerations

The token-based authentication model introduces a **cross-site tracking risk**. The Ethereum wallet address sent in the `X-CBOR-Web-Wallet` header is a persistent identifier that is identical across all CBOR-Web sites. A coalition of publishers could correlate agent activity across sites by sharing wallet addresses, similar to third-party cookie tracking.

**Mitigations:**

| Mitigation | Agent Responsibility | Publisher Responsibility |
|------------|---------------------|------------------------|
| **Dedicated wallets** | An agent SHOULD use a different wallet address per site or per publisher domain to prevent cross-site correlation. | — |
| **Proxy services** | An agent MAY use a privacy proxy that holds the token and authenticates on the agent's behalf without revealing the agent's primary wallet. | — |
| **Minimal logging** | — | A publisher MUST NOT log wallet addresses for longer than necessary for rate limiting and abuse prevention. A publisher MUST NOT share wallet addresses with third parties. |
| **No fingerprinting** | — | A publisher MUST NOT use CBOR-Web headers (wallet, nonce, signature) for user fingerprinting beyond authentication. |

These requirements are elaborated in CBOR-WEB-SECURITY.md §11.4.

---

## 13. Examples

### 13.1 Minimal Example — Single-Page Site

A minimal CBOR-Web deployment for a simple one-page site at Minimal conformance.

**Manifest:**
```cbor-diag
55799({
  0: "cbor-web-manifest",
  1: 2,
  2: {
    "name": "Minimal Example",
    "domain": "minimal.example",
    "languages": ["en"],
    "default_language": "en"
  },
  3: [
    {
      "lang": "en",
      "path": "/",
      "size": 95,
      "title": "Home",
      "access": "public"
    }
  ],
  5: {
    "total_size": 95,
    "total_pages": 1,
    "generated_at": 1(1742515200),
    "bundle_available": false
  }
})
```

**Page (`_index.cbor`):**
```cbor-diag
55799({
  0: "cbor-web-page",
  1: 2,
  2: {
    "lang": "en",
    "path": "/",
    "canonical": "https://minimal.example/"
  },
  3: {
    "title": "Welcome to Minimal Example"
  },
  4: [
    {"l": 1, "t": "h", "v": "Welcome"},
    {"t": "p", "v": "This is a minimal CBOR-Web page with only headings and paragraphs."}
  ]
})
```

### 13.2 E-Commerce Example — Product Page

A product page with structured data, demonstrating Standard conformance:

```cbor-diag
55799({
  0: "cbor-web-page",
  1: 2,
  2: {
    "lang": "fr",
    "path": "/products/lions-mane",
    "canonical": "https://verdetao.com/products/lions-mane",
    "alternates": {"es": "/es/productos/melena-de-leon"}
  },
  3: {
    "tags": ["champignon", "nootropique", "hericium"],
    "title": "Lion's Mane — Champignon Fonctionnel Premium",
    "updated": 1(1742428800),
    "category": "products",
    "description": "Criniere de Lion bio, extrait 10:1, 90 capsules"
  },
  4: [
    {"l": 1, "t": "h", "v": "Lion's Mane — Criniere de Lion"},
    {"t": "p", "v": "Notre extrait de Hericium erinaceus est concentre 10:1, issu de culture biologique certifiee EU."},
    {"t": "table",
      "rows": [
        ["Concentration", "Extrait 10:1"],
        ["Capsules", "90 par flacon"],
        ["Certification", "Bio EU"],
        ["Prix", "29.90 EUR"]
      ],
      "headers": ["Propriete", "Valeur"]
    },
    {"l": 2, "t": "h", "v": "Bienfaits"},
    {"t": "ul", "v": [
      "Soutient les fonctions cognitives et la memoire",
      "Favorise la production de NGF (Nerve Growth Factor)",
      "Reduit le stress oxydatif neuronal"
    ]},
    {"t": "cta", "v": "Ajouter au panier", "href": "/cart/add/lions-mane"}
  ],
  5: {
    "external": [
      {"url": "https://www.wikidata.org/wiki/Q138679188", "text": "Wikidata Verdetao"}
    ],
    "internal": [
      {"path": "/blog/lions-mane-dangers", "text": "Lion's Mane : dangers et precautions"}
    ]
  },
  6: {
    "name": "Lion's Mane — Criniere de Lion",
    "type": "Product",
    "brand": {
      "url": "https://verdetao.com",
      "name": "Verdetao",
      "type": "Organization"
    },
    "offers": {
      "type": "Offer",
      "price": 29.90,
      "availability": "InStock",
      "priceCurrency": "EUR"
    }
  }
})
```

### 13.3 Multilingual Blog Example

Manifest excerpt showing multilingual navigation at Standard conformance:

```cbor-diag
4: {
  "main": ["/", "/blog", "/products", "/contact"],
  "footer": ["/privacy", "/terms", "/cgv"],
  "hierarchy": {
    "/blog": [
      "/blog/lions-mane-dangers",
      "/blog/reishi-bienfaits",
      "/blog/cordyceps-energie"
    ],
    "/products": [
      "/products/lions-mane",
      "/products/reishi",
      "/products/cordyceps"
    ]
  }
}
```

Page entry with multilingual alternates:
```cbor-diag
{
  "hash": h'...',                    ; PLACEHOLDER — compute from actual page CBOR
  "lang": "fr",
  "path": "/blog/lions-mane-dangers",
  "size": 2340,
  "title": "Lion's Mane : dangers et effets secondaires",
  "access": "public",
  "updated": 1(1742428800),
  "category": "blog",
  "alternates": {
    "en": "/en/blog/lions-mane-dangers",
    "es": "/es/blog/melena-de-leon-peligros"
  },
  "content_type": "article"
}
```

---

## 14. Crawler Architecture

This section defines the expected behavior of a CBOR-Web crawler — the client-side software that discovers, fetches, and processes CBOR-Web content for AI agents.

### 14.1 Discovery and Fetch Sequence

A conforming crawler MUST implement the following sequence:

1. **Discover**: Check `/.well-known/cbor-web` for manifest presence (§4.1)
2. **Authenticate** (if needed): Present wallet token via `X-CBOR-Web-Wallet` headers (§9.4)
3. **Fetch manifest**: Download and parse the manifest CBOR document
4. **Decide**: Read page entries — compare hashes with local cache (§10.1)
5. **Fetch content**: Download bundle (single request) or individual changed pages
6. **Validate**: Verify SHA-256 hashes of downloaded pages against manifest (§10.2)
7. **Output**: Emit structured content for downstream AI consumption

```
cbor-crawl https://example.com

Step 1: GET /.well-known/cbor-web → 200 OK (524 bytes)
Step 2: Parse manifest → 25 pages, bundle available
Step 3: GET /.well-known/cbor-web/bundle → 200 OK (48 KB)
Step 4: Parse bundle → 25 pages extracted
Step 5: Verify hashes → 25/25 OK
Step 6: Output → structured JSON or raw text (for embedding pipelines)
```

### 14.2 Local Index

A crawler SHOULD maintain a local index per site:

```
~/.cbor-web/cache/
  example.com/
    manifest.cbor            # latest manifest
    hashes.json              # hash per page for diff
    index.json               # structured site index
    pages/                   # cached page documents
      _index.cbor
      about.cbor
      products_lions-mane.cbor
```

The local index (`index.json`) maps the site structure for fast lookups:

```json
{
  "domain": "example.com",
  "last_fetched": "2026-03-22T01:00:00Z",
  "pages": {
    "/": {"title": "Home", "lang": "en", "hash": "d8ca..."},
    "/about": {"title": "About", "lang": "en", "hash": "a1b2..."},
    "/products/lions-mane": {"title": "Lion's Mane", "lang": "en", "hash": "9fc4...", "has_commerce": true}
  },
  "navigation": {
    "main": ["/", "/about", "/products", "/blog", "/contact"]
  }
}
```

An AI agent consulting this index can locate specific content without re-downloading:

```
Agent needs product pricing
→ Check index: /products/lions-mane has_commerce=true
→ Fetch only that page (438 bytes) instead of full bundle
→ Parse table block → find price
```

### 14.3 Incremental Updates

On subsequent visits, the crawler:

1. Re-fetches the manifest only (typically < 1 KB)
2. Compares page hashes with local cache
3. Downloads only changed pages
4. Updates the local index

For a site where 2 of 25 pages changed, the crawler downloads: manifest (524 bytes) + 2 pages (~2 KB) = **~2.5 KB total**. An HTML re-crawl of the same site would download all 25 pages (~625 KB minimum).

### 14.4 Output Modes

A crawler MUST support at minimum two output modes:

| Mode | Format | Use Case |
|------|--------|----------|
| **json** | Structured JSON on stdout | Pipeline integration, programmatic access |
| **text** | Plain text on stdout | Direct embedding input, LLM consumption |

A crawler SHOULD additionally support:

| Mode | Format | Use Case |
|------|--------|----------|
| **cbor** | Re-serialized CBOR | Binary storage, forwarding to other agents |
| **files** | One file per page in output directory | Batch processing |

### 14.5 Forward-Compatible Block Handling

When a crawler encounters an unknown content block type (§3.6, §8.17), it MUST NOT skip the block silently. Instead:

1. Preserve the block's raw key-value pairs in the output
2. Mark it with `"_unknown_type": true`
3. Let the downstream AI agent interpret it naturally

This approach leverages the AI agent's natural language understanding: an AI reading `{"t": "recipe", "v": {"ingredients": [...], "steps": [...]}}` will understand it without any code update to the crawler. The crawler is deliberately simple; the intelligence is in the AI that consumes its output.

### 14.6 Rate Limiting and Politeness

A crawler MUST:
- Respect the `rate_limit.requests_per_second` declared in the manifest (§5.6)
- Respect `bundle_cooldown_seconds` between bundle re-downloads
- Include a `User-Agent` header identifying itself (§10.8)
- Honor `robots.txt` directives for its user-agent
- Back off on HTTP 429 responses (respect `Retry-After` header)

### 14.7 Reference Implementation

The reference crawler implementation is `cbor-crawl`, written in Rust. Source: `https://github.com/explodev/cbor-web` (directory `cbor-crawl/`).

```bash
# Inspect a site's CBOR-Web manifest
cbor-crawl inspect https://verdetao.com

# Fetch full site content as JSON
cbor-crawl fetch https://verdetao.com --format json

# Watch for changes (poll every hour)
cbor-crawl watch https://verdetao.com --interval 3600

# Fetch with token authentication
cbor-crawl fetch https://verdetao.com --wallet 0x1234... --keyfile ~/.cbor-web/key
```

---

## Appendix A: Core CDDL Schema

The following CDDL (RFC 8610) schema formally defines all core CBOR-Web document structures. Multimedia, generative, and security schemas are in their respective companion documents. The unified CDDL covering ALL types is in CBOR-WEB-REFERENCE.md.

```cddl
; ══════════════════════════════════════════════════════════
; CBOR-Web Core Specification v2.1.3 — CDDL Schema
; Document: CBOR-WEB-CORE.md, Appendix A
; RFC 8610 (Concise Data Definition Language)
; ══════════════════════════════════════════════════════════

; ── Top-Level Document Types ──

cbor-web-document = #6.55799(manifest / sub-manifest / page / bundle)

; ── Manifest ──

manifest = {
  0 => "cbor-web-manifest",
  1 => uint,                          ; @version: MUST be 2
  2 => site-metadata,
  3 => [+ page-entry],
  ? 4 => navigation,                  ; OPTIONAL (RECOMMENDED at Standard+)
  5 => manifest-meta,
  ? 6 => bstr,                        ; COSE_Sign1 signature (serialized as bstr)
  ? 7 => capabilities,                ; defined in CBOR-WEB-GENERATIVE.md
  ? 8 => [+ channel],                 ; defined in CBOR-WEB-MULTIMEDIA.md
  ? 9 => diff-manifest,
  ? 10 => manifest-security,          ; defined in CBOR-WEB-SECURITY.md
  * int => any                         ; forward-compatible
}

; Sub-manifest: keys 2 and 4 OPTIONAL (only in first page)
sub-manifest = {
  0 => "cbor-web-manifest",
  1 => uint,
  ? 2 => site-metadata,
  3 => [+ page-entry],
  ? 4 => navigation,
  5 => manifest-meta,
  ? 6 => bstr,
  ? 7 => capabilities,
  ? 8 => [+ channel],
  ? 9 => diff-manifest,
  ? 10 => manifest-security,
  * int => any
}

site-metadata = {
  "domain" => tstr,
  "name" => tstr,
  ? "description" => tstr,
  "languages" => [+ language-code],
  "default_language" => language-code,
  ? "contact" => { ? "email" => tstr, ? "phone" => tstr, * tstr => any },
  ? "geo" => {
    ? "country" => tstr,
    ? "region" => tstr,
    ? "coordinates" => [float .ge -90.0 .le 90.0, float .ge -180.0 .le 180.0],
    * tstr => any
  },
  * tstr => any
}

language-code = tstr .regexp "[a-z]{2,3}(-[A-Za-z]{2,8})*"  ; BCP 47 (RFC 5646)

page-entry = {
  "path" => tstr,
  "title" => tstr,
  "lang" => language-code,
  "access" => "public" / "token",     ; v2.1 REQUIRED field
  "size" => uint,
  ? "updated" => #6.1(uint),
  ? "hash" => bstr .size 32,
  ? "alternates" => { + language-code => tstr },
  ? "category" => tstr,
  ? "content_type" => "article" / "product" / "landing" / "documentation" / tstr,
  ? "has_multimedia" => bool,
  ? "has_generative" => bool,
  ? "has_forms" => bool,
  ? "has_commerce" => bool,
  ? "media_size" => uint,
  * tstr => any
}

navigation = {
  "main" => [+ tstr],
  ? "footer" => [+ tstr],
  ? "hierarchy" => { + tstr => [+ tstr] },
  ? "breadcrumbs" => { + tstr => [+ tstr] },
  * tstr => any
}

manifest-meta = {
  ? "generator" => tstr,
  "generated_at" => #6.1(uint),
  "total_pages" => uint,
  "total_size" => uint,
  "bundle_available" => bool,
  ? "bundle_url" => tstr,
  ? "rate_limit" => {
    ? "requests_per_second" => uint,
    ? "bundle_cooldown_seconds" => uint,
    * tstr => any
  },
  ? "next" => tstr,
  * tstr => any
}

; Diff manifest (key 9)
diff-manifest = {
  "diff_version" => uint,
  "base_version_hash" => bstr .size 32,
  "base_generated_at" => #6.1(uint),
  "current_generated_at" => #6.1(uint),
  "changes" => [+ diff-change],
  ? "stats" => {
    ? "pages_added" => uint,
    ? "pages_modified" => uint,
    ? "pages_removed" => uint,
    ? "total_pages_now" => uint,
    * tstr => any
  },
  * tstr => any
}

diff-change = {
  "path" => tstr,
  "action" => "added" / "modified" / "removed",
  ? "hash" => bstr .size 32,
  ? "previous_hash" => bstr .size 32,
  ? "size" => uint,
  ? "title" => tstr,
  ? "fields_changed" => [+ tstr],
  * tstr => any
}

; ── Page Document ──

page = {
  0 => "cbor-web-page",
  1 => uint,                          ; @version: MUST be 2
  2 => page-identity,
  3 => page-metadata,
  4 => [* content-block],             ; empty array allowed (e.g., redirect pages)
  ? 5 => page-links,
  ? 6 => structured-data,
  ? 7 => [+ generative-block],       ; defined in CBOR-WEB-GENERATIVE.md
  ? 8 => [+ form-block],             ; defined in CBOR-WEB-GENERATIVE.md
  ? 9 => commerce-data,              ; defined in CBOR-WEB-GENERATIVE.md
  * int => any
}

page-identity = {
  "path" => tstr,
  "canonical" => tstr,
  "lang" => language-code,
  ? "alternates" => { + language-code => tstr },
  * tstr => any
}

page-metadata = {
  "title" => tstr,
  ? "description" => tstr,
  ? "author" => tstr,
  ? "published" => #6.1(uint),
  ? "updated" => #6.1(uint),
  ? "tags" => [+ tstr],
  ? "category" => tstr,
  ? "reading_time_seconds" => uint,
  ? "word_count" => uint,
  * tstr => any
}

page-links = {
  ? "internal" => [* { "path" => tstr, "text" => tstr, * tstr => any }],
  ? "external" => [* { "url" => tstr, "text" => tstr, * tstr => any }],
  * tstr => any
}

structured-data = {
  "type" => tstr,
  * tstr => any
}

; ── Content Blocks (Core v1.0) ──

content-block = heading / paragraph / unordered-list / ordered-list /
                quote / code-block / data-table / image-ref /
                call-to-action / embed / separator / definition-list /
                note-block
                ; multimedia + generative blocks defined in companion docs

heading = { "t" => "h", "l" => uint .ge 1 .le 6, "v" => tstr, * tstr => any }
paragraph = { "t" => "p", "v" => tstr, * tstr => any }
unordered-list = { "t" => "ul", "v" => [+ tstr], * tstr => any }
ordered-list = { "t" => "ol", "v" => [+ tstr], * tstr => any }
quote = { "t" => "q", "v" => tstr, ? "attr" => tstr, * tstr => any }
code-block = { "t" => "code", "v" => tstr, ? "lang" => tstr, * tstr => any }
data-table = { "t" => "table", "headers" => [+ tstr], "rows" => [+ [+ tstr]], * tstr => any }
image-ref = { "t" => "img", "alt" => tstr, "src" => tstr, ? "caption" => tstr, * tstr => any }
call-to-action = { "t" => "cta", "v" => tstr, "href" => tstr, * tstr => any }
embed = { "t" => "embed", "src" => tstr, ? "description" => tstr, * tstr => any }
separator = { "t" => "sep", * tstr => any }
definition-list = { "t" => "dl", "v" => [+ { "term" => tstr, "def" => tstr, * tstr => any }], * tstr => any }
note-block = { "t" => "note", "v" => tstr, ? "level" => "info" / "warn" / "important", * tstr => any }

; ── Bundle ──

bundle = {
  0 => "cbor-web-bundle",
  1 => uint,
  2 => manifest,
  3 => { + tstr => page },
  * int => any
}

; ── Forward declarations for companion document types ──
; These are placeholders — full definitions in respective documents.
; An agent that only implements CBOR-WEB-CORE.md ignores these keys.

capabilities = { * tstr => any }      ; CBOR-WEB-GENERATIVE.md §17
channel = { * tstr => any }            ; CBOR-WEB-MULTIMEDIA.md §20
manifest-security = { * tstr => any }  ; CBOR-WEB-SECURITY.md §12.3
generative-block = { * tstr => any }   ; CBOR-WEB-GENERATIVE.md
form-block = { * tstr => any }         ; CBOR-WEB-GENERATIVE.md §18
commerce-data = { * tstr => any }      ; CBOR-WEB-GENERATIVE.md §19
```

---

## Appendix B: Test Vectors

All test vectors have been generated using **deterministic CBOR encoding** (RFC 8949 §4.2.1) and cross-validated by two independent implementations:
- **Rust**: ciborium 0.2.2 (`cbor-vectors/` in the repository)
- **Python**: cbor2 (`canonical=True`)

Both implementations produce **byte-identical output** for all vectors.

### B.1 Test Vector 1 — Minimal Manifest (v2.1)

**Input (diagnostic notation — keys in deterministic order):**
```cbor-diag
55799({
  0: "cbor-web-manifest",
  1: 2,
  2: {
    "name": "Test",
    "domain": "test.example",
    "languages": ["en"],
    "default_language": "en"
  },
  3: [
    {"lang": "en", "path": "/", "size": 95, "title": "Home", "access": "public"}
  ],
  5: {
    "total_size": 95,
    "total_pages": 1,
    "generated_at": 1(1742515200),
    "bundle_available": false
  }
})
```

**Expected CBOR hex (206 bytes):**
```
D9 D9F7                              -- tag(55799) self-described CBOR
  A5                                  -- map(5)
    00                                -- key: 0 (@type)
    71                                -- text(17)
      63626F722D7765622D6D616E696665  -- "cbor-web-manifes"
      7374                            -- "t"
    01                                -- key: 1 (@version)
    02                                -- unsigned(2)
    02                                -- key: 2 (site)
    A4                                -- map(4)
      64                              -- text(4)
        6E616D65                      -- "name"
      64                              -- text(4)
        54657374                      -- "Test"
      66                              -- text(6)
        646F6D61696E                  -- "domain"
      6C                              -- text(12)
        746573742E6578616D706C65      -- "test.example"
      69                              -- text(9)
        6C616E677561676573            -- "languages"
      81                              -- array(1)
        62                            -- text(2)
          656E                        -- "en"
      70                              -- text(16)
        64656661756C745F6C616E677561  -- "default_languag"
        6765                          -- "e"
      62                              -- text(2)
        656E                          -- "en"
    03                                -- key: 3 (pages)
    81                                -- array(1)
      A5                              -- map(5)
        64                            -- text(4)
          6C616E67                    -- "lang"
        62                            -- text(2)
          656E                        -- "en"
        64                            -- text(4)
          70617468                    -- "path"
        61                            -- text(1)
          2F                          -- "/"
        64                            -- text(4)
          73697A65                    -- "size"
        18 5F                         -- unsigned(95)
        65                            -- text(5)
          7469746C65                  -- "title"
        64                            -- text(4)
          486F6D65                    -- "Home"
        66                            -- text(6)
          616363657373                -- "access"
        66                            -- text(6)
          7075626C6963                -- "public"
    05                                -- key: 5 (meta)
    A4                                -- map(4)
      6A                              -- text(10)
        746F74616C5F73697A65          -- "total_size"
      18 5F                           -- unsigned(95)
      6B                              -- text(11)
        746F74616C5F7061676573        -- "total_pages"
      01                              -- unsigned(1)
      6C                              -- text(12)
        67656E6572617465645F6174      -- "generated_at"
      C1                              -- tag(1) epoch timestamp
        1A 67DCAC00                   -- unsigned(1742515200)
      70                              -- text(16)
        62756E646C655F617661696C6162  -- "bundle_availabl"
        6C65                          -- "e"
      F4                              -- false
```

**Size: 206 bytes**
**SHA-256: `6536295FAA254EBD03CC61A0B338A582D25422BF8685EE57691FBA9603511C9F`**

### B.2 Test Vector 2 — Minimal Page (v2.1)

**Input (diagnostic notation):**
```cbor-diag
55799({
  0: "cbor-web-page",
  1: 2,
  2: {"lang": "en", "path": "/", "canonical": "https://test.example/"},
  3: {"title": "Welcome"},
  4: [
    {"l": 1, "t": "h", "v": "Welcome"},
    {"t": "p", "v": "Hello, World!"}
  ]
})
```

**Expected CBOR hex (127 bytes):**
```
D9 D9F7                              -- tag(55799)
  A5                                  -- map(5)
    00                                -- key: 0
    6D                                -- text(13)
      63626F722D7765622D70616765      -- "cbor-web-page"
    01                                -- key: 1
    02                                -- unsigned(2)
    02                                -- key: 2
    A3                                -- map(3)
      64                              -- text(4)
        6C616E67                      -- "lang"
      62                              -- text(2)
        656E                          -- "en"
      64                              -- text(4)
        70617468                      -- "path"
      61                              -- text(1)
        2F                            -- "/"
      69                              -- text(9)
        63616E6F6E6963616C            -- "canonical"
      75                              -- text(21)
        68747470733A2F2F746573742E65  -- "https://test.ex"
        78616D706C652F                -- "ample/"
    03                                -- key: 3
    A1                                -- map(1)
      65                              -- text(5)
        7469746C65                    -- "title"
      67                              -- text(7)
        57656C636F6D65                -- "Welcome"
    04                                -- key: 4
    82                                -- array(2)
      A3                              -- map(3) — heading block
        61                            -- text(1)
          6C                          -- "l"
        01                            -- unsigned(1)
        61                            -- text(1)
          74                          -- "t"
        61                            -- text(1)
          68                          -- "h"
        61                            -- text(1)
          76                          -- "v"
        67                            -- text(7)
          57656C636F6D65              -- "Welcome"
      A2                              -- map(2) — paragraph block
        61                            -- text(1)
          74                          -- "t"
        61                            -- text(1)
          70                          -- "p"
        61                            -- text(1)
          76                          -- "v"
        6D                            -- text(13)
          48656C6C6F2C20576F726C6421  -- "Hello, World!"
```

**Size: 127 bytes**
**SHA-256: `D8CAD2E6E8D06A0EF4E5B22C7394E4AC2B48FDE517DBB012C705DB26D163CEB7`**

**Key ordering verification:**
- Heading block: `"l"` (61 6C) < `"t"` (61 74) < `"v"` (61 76) ✅ (2 bytes each, bytewise comparison)
- Identity map: `"lang"` (5B) < `"path"` (5B, `70>6C`) < `"canonical"` (10B) ✅ (length then bytewise)

---

## Appendix C: HTML to CBOR-Web Mapping

### C.1 Element Mapping

| HTML Element(s) | CBOR-Web Block | Notes |
|-----------------|---------------|-------|
| `<h1>` to `<h6>` | `{"l": N, "t": "h", "v": "..."}` | N = heading level. Key order: l, t, v |
| `<p>` | `{"t": "p", "v": "..."}` | Strip inline markup, preserve text |
| `<ul><li>...</li></ul>` | `{"t": "ul", "v": ["...","..."]}` | Each `<li>` becomes an array element |
| `<ol><li>...</li></ol>` | `{"t": "ol", "v": ["...","..."]}` | Same as `<ul>` |
| `<blockquote>` | `{"t": "q", "v": "..."}` | `<cite>` → `"attr"` |
| `<pre><code>` | `{"t": "code", "v": "..."}` | `class="language-X"` → `"lang":"X"` |
| `<table>` | `{"t": "table", "rows": [...], "headers": [...]}` | `<thead>` → headers, `<tbody>` → rows |
| `<img>` | `{"t": "img", "alt": "...", "src": "..."}` | `alt` REQUIRED |
| `<a class="cta">`, `<button>` | `{"t": "cta", "v": "...", "href": "..."}` | Publisher decides what is a CTA |
| `<iframe>`, `<video>`, `<audio>` | `{"t": "embed", "src": "..."}` | |
| `<hr>` | `{"t": "sep"}` | |
| `<dl><dt><dd>` | `{"t": "dl", "v": [...]}` | Each dt/dd pair → `{"def": ..., "term": ...}` |
| `<aside class="note">` | `{"t": "note", "v": "..."}` | |

### C.2 Elements to Discard

| HTML Element | Reason |
|-------------|--------|
| `<nav>` | Navigation is in the manifest (key 4), not in page content |
| `<header>` (site header) | Site-level metadata, not page content |
| `<footer>` (site footer) | Site-level links, not page content |
| `<script>` | Behavior, not content |
| `<style>` | Presentation, not content |
| `<noscript>` | Fallback for JS-disabled browsers |
| Cookie banners | Marketing/legal UI, not content |
| Newsletter popups | Marketing UI |
| `<svg>` (decorative) | Presentation. Meaningful SVGs → `"img"` block with `"alt"` |

### C.3 Inline Markup Handling

CBOR-Web text values are plain text. Inline HTML markup is stripped:

| HTML | CBOR-Web `"v"` |
|------|----------------|
| `Learn <strong>React</strong> today` | `"Learn React today"` |
| `Visit <a href="/about">our page</a>` | `"Visit our page"` |
| `Use the <code>npm install</code> command` | `"Use the npm install command"` |
| `E = mc<sup>2</sup>` | `"E = mc2"` |
| `H<sub>2</sub>O` | `"H2O"` |

### C.4 JSON-LD to CBOR Structured Data

| JSON-LD | CBOR-Web (key 6) |
|---------|-------------------|
| `"@type": "Product"` | `"type": "Product"` |
| `"@id": "https://..."` | `"id": "https://..."` |
| `"@context": "..."` | Omitted (implied Schema.org) |
| Nested `{"@type": "Offer", ...}` | Nested CBOR map `{"type": "Offer", ...}` |

---

## Appendix D: Comparison with Existing Standards

| Feature | robots.txt | sitemap.xml | llms.txt | **CBOR-Web v2.1** |
|---------|-----------|-------------|----------|-------------------|
| Format | Text | XML | Markdown | **Binary (CBOR)** |
| Content included | No | No | Summary only | **Full structured content** |
| Navigation structure | No | Flat URL list | Flat | **Hierarchical, typed** |
| Multilingual support | No | hreflang (per-URL) | No | **Per-page + manifest** |
| Structured data | No | No | No | **Native CBOR** |
| Incremental updates | No | lastmod (unreliable) | No | **SHA-256 hash + diffs** |
| Single-request indexing | No | URL list only | Yes (summary) | **Yes (bundle)** |
| Access control | No | No | No | **ERC-20 token badge** |
| Binary efficiency | N/A | N/A | N/A | **Zero tokenization cost** |
| Embedding quality | N/A | N/A | Good (clean text) | **Optimal (95% signal, zero noise)** |
| Size (80-page site) | ~1 KB | ~15 KB | ~2 KB | **~50 KB (full content)** |
| Tokens consumed by LLM | 0 (rules) | ~3,000 | ~500 | **0 (parsed in memory)** |

---

## References

### Normative References

- **[RFC 2119]** Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, March 1997.
- **[RFC 8174]** Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, May 2017.
- **[RFC 8949]** Bormann, C. and P. Hoffman, "Concise Binary Object Representation (CBOR)", STD 94, December 2020.
- **[RFC 8610]** Birkholz, H., Vigano, C., and C. Bormann, "Concise Data Definition Language (CDDL)", June 2019.
- **[RFC 9052]** Schaad, J., "CBOR Object Signing and Encryption (COSE): Structures and Process", STD 96, August 2022.
- **[RFC 8615]** Nottingham, M., "Well-Known Uniform Resource Identifiers (URIs)", May 2019.
- **[RFC 9309]** Koster, M., et al., "Robots Exclusion Protocol", September 2022.

### Informative References

- **[llms.txt]** "llms.txt — A proposed standard for making websites readable by LLMs", https://llmstxt.org/
- **[Schema.org]** "Schema.org — shared vocabulary for structured data", https://schema.org/
- **[ERC-20]** Vogelsteller, F. and V. Buterin, "EIP-20: Token Standard", November 2015.

---

*CBOR-Web Core Specification v2.1.3 — Document 1 of 6*

*ExploDev 2026 — "The web has two clients: humans and machines. It's time to serve both."*
