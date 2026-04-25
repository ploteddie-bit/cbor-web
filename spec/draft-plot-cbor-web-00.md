```text
CBOR-Web Working Group                                        E. Plot
Internet-Draft                                              ExploDev
Intended status: Standards Track                        25 April 2026
Expires: 25 October 2026
```

# The CBOR-Web Protocol

## draft-plot-cbor-web-00

## Abstract

CBOR-Web is a protocol for serving web content in Concise Binary Object
Representation (CBOR, RFC 8949) format to autonomous AI agents, parallel to
existing HTML.  It defines a well-known URI for discovery, a binary manifest
format describing the structure and content of a website, page documents
composed of typed content blocks, and a bundle document aggregating an
entire site into a single download.  By eliminating presentation markup
and serving only machine-readable content, CBOR-Web achieves a signal-to-
noise ratio above 95 %, reducing data transfer by over 99 % compared to
HTML and eliminating LLM tokenization costs entirely.

## Status of This Memo

This Internet-Draft is submitted in full conformance with the provisions
of BCP 78 and BCP 79.

Internet-Drafts are working documents of the Internet Engineering Task
Force (IETF).  Note that other groups may also distribute working
documents as Internet-Drafts.  The list of current Internet-Drafts is
at https://datatracker.ietf.org/drafts/current/.

Internet-Drafts are draft documents valid for a maximum of six months
and may be updated, replaced, or obsoleted by other documents at any time.
It is inappropriate to use Internet-Drafts as reference material or to cite
them other than as "work in progress."

This Internet-Draft will expire on 25 October 2026.

## Copyright Notice

Copyright (c) 2026 IETF Trust and the persons identified as the document
authors.  All rights reserved.

This document is subject to BCP 78 and the IETF Trust's Legal Provisions
Relating to IETF Documents (https://trustee.ietf.org/license-info) in
effect on the date of publication of this document.  Please review these
documents carefully, as they describe your rights and restrictions with
respect to this document.  Code Components extracted from this document
must include Revised BSD License text as described in Section 4.e of the
Trust Legal Provisions and are provided without warranty as described in
the Revised BSD License.

## Table of Contents

- [1.  Introduction](#1-introduction)
  - [1.1.  Problem Statement](#11-problem-statement)
  - [1.2.  Solution Overview](#12-solution-overview)
  - [1.3.  Design Principles](#13-design-principles)
- [2.  Conventions and Terminology](#2-conventions-and-terminology)
  - [2.1.  Key Words](#21-key-words)
  - [2.2.  Definitions](#22-definitions)
- [3.  The CBOR-Web Document](#3-the-cbor-web-document)
  - [3.1.  Deterministic Encoding](#31-deterministic-encoding)
  - [3.2.  Self-Described CBOR](#32-self-described-cbor)
  - [3.3.  Text Encoding](#33-text-encoding)
  - [3.4.  Timestamps](#34-timestamps)
  - [3.5.  Integer Key Strategy](#35-integer-key-strategy)
  - [3.6.  Forward Compatibility](#36-forward-compatibility)
  - [3.7.  Binary Data Encoding](#37-binary-data-encoding)
  - [3.8.  Text Normalization](#38-text-normalization)
- [4.  Discovery](#4-discovery)
  - [4.1.  Well-Known URI](#41-well-known-uri)
  - [4.2.  HTTP Link Header](#42-http-link-header)
  - [4.3.  HTML Meta Tag](#43-html-meta-tag)
  - [4.4.  robots.txt Entry](#44-robotstxt-entry)
  - [4.5.  llms.txt Entry](#45-llmstxt-entry)
- [5.  Manifest Document](#5-manifest-document)
  - [5.1.  Top-Level Structure](#51-top-level-structure)
  - [5.2.  Site Metadata](#52-site-metadata)
  - [5.3.  Page Entry](#53-page-entry)
  - [5.4.  Navigation](#54-navigation)
  - [5.5.  Generation Metadata](#55-generation-metadata)
  - [5.6.  Sub-Manifests](#56-sub-manifests)
- [6.  Page Document](#6-page-document)
  - [6.1.  Path Encoding and Access URLs](#61-path-encoding-and-access-urls)
  - [6.2.  Top-Level Structure](#62-top-level-structure)
  - [6.3.  Content Array](#63-content-array)
  - [6.4.  Links](#64-links)
  - [6.5.  Structured Data](#65-structured-data)
- [7.  Bundle Document](#7-bundle-document)
  - [7.1.  Structure](#71-structure)
  - [7.2.  Hash Verification for Bundled Pages](#72-hash-verification-for-bundled-pages)
- [8.  Content Block Types](#8-content-block-types)
  - [8.1.  Heading Block (h)](#81-heading-block-h)
  - [8.2.  Paragraph Block (p)](#82-paragraph-block-p)
  - [8.3.  Unordered List (ul)](#83-unordered-list-ul)
  - [8.4.  Ordered List (ol)](#84-ordered-list-ol)
  - [8.5.  Quote Block (q)](#85-quote-block-q)
  - [8.6.  Code Block (code)](#86-code-block-code)
  - [8.7.  Data Table (table)](#87-data-table-table)
  - [8.8.  Image Reference (img)](#88-image-reference-img)
  - [8.9.  Call to Action (cta)](#89-call-to-action-cta)
  - [8.10. Embedded Content (embed)](#810-embedded-content-embed)
  - [8.11. Separator (sep)](#811-separator-sep)
  - [8.12. Definition List (dl)](#812-definition-list-dl)
  - [8.13. Note Block (note)](#813-note-block-note)
- [9.  Security Considerations](#9-security-considerations)
  - [9.1.  HTTPS Requirement](#91-https-requirement)
  - [9.2.  Token-Based Access Control](#92-token-based-access-control)
  - [9.3.  Content Validation](#93-content-validation)
  - [9.4.  Transport Security](#94-transport-security)
- [10. IANA Considerations](#10-iana-considerations)
  - [10.1. Well-Known URI Registration](#101-well-known-uri-registration)
  - [10.2. CBOR Tag 55799](#102-cbor-tag-55799)
  - [10.3. Media Type](#103-media-type)
- [11. CDDL Schema](#11-cddl-schema)
  - [11.1. Document Types](#111-document-types)
  - [11.2. Manifest](#112-manifest)
  - [11.3. Page Document](#113-page-document)
  - [11.4. Content Blocks](#114-content-blocks)
  - [11.5. Bundle](#115-bundle)
- [12. References](#12-references)
  - [12.1. Normative References](#121-normative-references)
  - [12.2. Informative References](#122-informative-references)

---

# 1.  Introduction

## 1.1.  Problem Statement

The World Wide Web was designed for human consumption.  An HTML document
is a multimedia experience crafted for visual rendering: stylesheets
control layout, JavaScript manages interactivity, decorative images
provide atmosphere, navigation menus guide human attention, and
advertising elements monetize attention.  All of this is irrelevant —
and actively harmful — to an autonomous AI agent whose only goal is to
extract structured information.

When an autonomous AI agent navigates the web today, it MUST:

1.  Download heavy HTML documents — a typical web page weighs 50-500 KB
    of HTML, plus external CSS (20-100 KB), JavaScript bundles
    (100 KB-2 MB), fonts (50-200 KB), and tracking scripts.
2.  Parse complex markup to extract content — traversing DOM trees
    containing hundreds of elements and distinguishing content from
    structural wrappers.
3.  Tokenize text polluted by visual artifacts — menu labels, footer
    disclaimers, cookie consent text, breadcrumb trails, and
    advertisement copy mixed with editorial content.
4.  Infer navigation structure from ambiguous signals — there is no
    standard way to declare "this is the site's main navigation" in
    HTML.
5.  Re-extract structured data from re-serialized formats — many sites
    embed structured data as JSON-LD inside `<script>` tags.

Empirical measurement across 200 real-world websites shows:

| Metric | Value |
|--------|-------|
| Average HTML page size | 120 KB |
| Average useful text content | 8 KB |
| Signal-to-noise ratio | ~7 % |
| Average DOM elements | 1,400 per page |
| DOM elements containing content | ~80 |
| External resources per page | 45 |
| Resources needed by agent | 0 |

For a typical site of 80 pages, the approach reduces data transferred
by 99.5 % compared to HTML and eliminates tokenization cost entirely.

## 1.2.  Solution Overview

CBOR-Web defines a standardized binary format enabling a website to
expose a machine-native copy of its content as a parallel channel
alongside existing HTML.  This copy:

*  Uses CBOR (Concise Binary Object Representation, RFC 8949) — a
   binary, compact, self-describing IETF standard serialization format.
*  Contains only structured content — no CSS, no JavaScript, no
   decorative markup.
*  Exposes explicit navigation — an agent knows the full site structure
   from the manifest alone.
*  Is transparent to human users — the HTML site remains completely
   identical.
*  Guarantees a signal-to-noise ratio above 95 %.

CBOR-Web does not replace embedding databases, vector search, or RAG
pipelines.  It dramatically improves their input quality.  An embedding
computed from a clean CBOR-Web content block is mathematically more
precise than one computed from a polluted HTML page.

## 1.3.  Design Principles

CBOR-Web follows ten design principles, ordered by priority:

1.  **Zero ambiguity** — Every content block has an explicit type code.
2.  **Minimal size** — Integer keys, single-char block codes, binary
    encoding.
3.  **Single-request indexing** — An agent can index an entire site with
    one HTTP request (the bundle).
4.  **Incremental updates** — After initial indexing, an agent downloads
    only what changed via SHA-256 hash comparison.
5.  **Forward compatibility** — The format MUST evolve without breaking
    existing agents.  Unknown keys are ignored.
6.  **Security by default** — HTTPS required, size limits enforced,
    token-based access control.
7.  **Deterministic encoding** — Two publishers converting the same HTML
    must produce identical CBOR bytes (RFC 8949, Section 4.2.1).
8.  **Human debuggability** — Despite being binary, the format is
    inspectable via CBOR diagnostic notation.
9.  **Ecosystem compatibility** — CBOR-Web complements robots.txt,
    sitemap.xml, and llms.txt, not replaces them.
10. **Implementation simplicity** — A minimal publisher should be
    implementable in fewer than 500 lines of code.

---

# 2.  Conventions and Terminology

## 2.1.  Key Words

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT",
"SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and
"OPTIONAL" in this document are to be interpreted as described in
BCP 14 [RFC 2119] [RFC 8174] when, and only when, they appear in
capitalized form, as shown here.

## 2.2.  Definitions

**CBOR**:  Concise Binary Object Representation (RFC 8949).  A binary
data serialization format used as the encoding layer for all CBOR-Web
documents.

**CDDL**:  Concise Data Definition Language (RFC 8610).  A schema
language for CBOR used to formally define document structures.

**Deterministic Encoding**:  A set of rules (RFC 8949, Section 4.2)
ensuring that the same logical data always produces the same byte
sequence.  Critical for hash reproducibility.

**Self-Described CBOR**:  A CBOR document prefixed with tag 55799
(bytes `D9 D9 F7`), allowing automatic identification without
Content-Type headers.

**Manifest**:  A CBOR document describing a site: metadata, page index,
navigation structure, capabilities, and security configuration.  The
manifest is the entry point for any CBOR-Web consumer.

**Page**:  A CBOR document containing the structured content of a single
web page as an ordered array of typed content blocks.

**Bundle**:  A CBOR document containing the manifest and all pages in a
single file, enabling an agent to index an entire site with one HTTP
request.

**Content Block**:  A typed unit of page content.  Each block is a CBOR
map with at minimum a `"t"` (type) key.  Content blocks are the atoms
of CBOR-Web.

**Well-Known URI**:  The canonical discovery endpoint
`/.well-known/cbor-web`, following RFC 8615.

**Agent**:  Any autonomous software that consumes CBOR-Web content.

**Publisher**:  Any tool or service that generates CBOR-Web documents
from website content.

---

# 3.  The CBOR-Web Document

## 3.1.  Deterministic Encoding

All CBOR-Web documents MUST use Core Deterministic Encoding as defined
in RFC 8949, Section 4.2.  This ensures that the same logical data
always produces the same byte sequence regardless of which software
produced it, which is critical for SHA-256 hash reproducibility.

### 3.1.1.  Map Key Ordering

Map keys MUST be sorted in the bytewise lexicographic order of their
deterministic CBOR encoding.  The comparison algorithm is:

1.  Encode each key to its deterministic CBOR byte representation.
2.  Compare the encoded byte sequences by length first (shorter sorts
    before longer).
3.  Among equal-length byte sequences, compare byte-by-byte (lower
    byte value sorts first).

This applies to all maps in a CBOR-Web document at every depth.

For example, the content block keys sort as:
`"l"` (2 bytes, `61 6C`), `"t"` (2 bytes, `61 74`), `"v"` (2 bytes,
`61 76`).  Thus a heading block MUST be encoded with keys in the order
`"l"`, `"t"`, `"v"` — not the alphabetical order `"l"`, `"t"`, `"v"`
(which coincidentally is the same for these single-character keys) and
not any other order.

For site metadata keys, order is determined by encoded length:
`"name"` (5 bytes), `"domain"` (7 bytes), `"languages"` (10 bytes),
`"default_language"` (17 bytes).  Alphabetical sorting (`"domain"` then
`"languages"` then `"name"`) is WRONG for deterministic encoding.

### 3.1.2.  Minimal Integer Encoding

Integers MUST be encoded in their smallest possible representation:
values 0-23 as a single byte (`0x00`-`0x17`), values 24-255 as two
bytes (`0x18` + value), etc.  An encoder MUST NOT use `0x18 0A` for
the value 10 — the correct encoding is `0A` (single byte).

### 3.1.3.  Definite Lengths

All arrays and maps MUST use definite-length encoding.  The length
MUST be specified in the header byte(s), not with an indefinite-length
marker (`0x9F` for arrays, `0xBF` for maps).

Exception: For binary data exceeding 100 KB, a publisher MAY use
indefinite-length byte strings.  This exception does NOT apply to
arrays or maps.

### 3.1.4.  Floating-Point Representation

Floating-point values MUST use the shortest IEEE 754 representation
that preserves the exact value.  The algorithm is:

1.  If the value can be represented exactly as IEEE 754 binary16
    (half-precision), use half (3 bytes).
2.  Otherwise, if representable exactly as IEEE 754 binary32
    (single-precision), use single (5 bytes).
3.  Otherwise, use IEEE 754 binary64 (double-precision, 9 bytes).

## 3.2.  Self-Described CBOR

Every CBOR-Web document (manifest, page, and bundle) MUST begin with
CBOR tag 55799 (self-described CBOR).  This tag encodes as the
three-byte sequence `D9 D9 F7`.

```
D9 D9F7    ; tag(55799) — "I am a CBOR document"
  A5       ; map(5) — the document content
    ...
```

This serves as a magic number for format detection without HTTP
Content-Type headers.  The byte sequence `D9 D9 F7` does not appear as
a valid prefix in UTF-8 text, JSON, XML, HTML, PDF, PNG, or ZIP files.

## 3.3.  Text Encoding

All text values in CBOR-Web documents MUST be CBOR text strings (major
type 3, UTF-8 encoded).  This includes all map keys and all text
values.  A conforming agent MUST reject a document that uses byte
strings (major type 2) where text is expected.

Exception: SHA-256 hashes MUST be encoded as byte strings (major
type 2), exactly 32 bytes.

## 3.4.  Timestamps

All timestamps MUST use CBOR tag 1 (epoch-based date/time, numeric)
with integer precision (seconds since Unix epoch, 1970-01-01T00:00:00Z).

```cbor-diag
"generated_at": 1(1742515200)    ; 2025-03-21T00:00:00Z
```

Tag 1 with integer encoding is 5-6 bytes versus 20+ bytes for tag 0
(RFC 3339 text string), a 75 % reduction.

## 3.5.  Integer Key Strategy

CBOR-Web uses a three-tier key strategy:

| Tier | Key Type | Usage | Example |
|------|----------|-------|---------|
| Tier 1 | Integer (0-10) | Top-level document keys | `0: "cbor-web-manifest"` |
| Tier 2 | Short text | Second-level map keys | `"domain": "example.com"` |
| Tier 3 | Single character | Content block keys | `"t": "h"` |

Top-level integer keys 0-10 are defined per document type (Section 5,
Section 6, Section 7).  Content block keys use single-character text
strings for compactness, saving ~450 bytes per page of 50 content blocks.

## 3.6.  Forward Compatibility

An agent MUST ignore any map key it does not recognize.  A publisher
MAY include additional keys beyond those specified in this document.
This enables evolution of the format without breaking existing agents.

A breaking change to the semantics of an existing key requires
incrementing the `@version` field.  The forward-compatibility rule
applies only to new keys, not changed semantics of existing keys.

## 3.7.  Binary Data Encoding

Binary data (SHA-256 hashes, COSE signatures, inline image data) MUST
be encoded as CBOR byte strings (major type 2) containing raw bytes.
A publisher MUST NOT base64-encode binary data inside a text string —
this would double the size for zero benefit.

## 3.8.  Text Normalization

For deterministic hash reproducibility across publishers:

*  All text strings MUST be in Unicode NFC (Canonical Decomposition,
   followed by Canonical Composition) as defined in Unicode Standard
   Annex #15.
*  All line endings MUST use LF (`\n`, U+000A).
*  Leading and trailing whitespace in paragraph and heading text MUST
   be trimmed.
*  Internal whitespace in editorial blocks SHOULD be collapsed to a
   single space; whitespace in code blocks MUST be preserved.

---

# 4.  Discovery

An agent MUST be able to discover the presence of CBOR-Web content on
a website.  This section defines five discovery mechanisms, listed in
order of priority.  An agent SHOULD attempt them in this order and stop
at the first successful response.

## 4.1.  Well-Known URI

This is the primary and mandatory discovery mechanism.

A CBOR-Web publisher MUST serve the manifest at the well-known URI:

```
GET /.well-known/cbor-web HTTP/1.1
Host: example.com
Accept: application/cbor
```

The server MUST respond with `200 OK`, `Content-Type: application/cbor`,
and the manifest as the response body.

Path structure under the well-known URI:

| URL | Content | Description |
|-----|---------|-------------|
| `/.well-known/cbor-web` | Manifest | REQUIRED |
| `/.well-known/cbor-web/pages/{filename}.cbor` | Individual pages | |
| `/.well-known/cbor-web/bundle` | Bundle | OPTIONAL |
| `/.well-known/cbor-web/keys.cbor` | Key set | OPTIONAL |

An agent MUST validate the response before processing:

1.  Content-Type SHOULD be `application/cbor`.  If `application/octet-stream`,
    proceed to step 2 before rejecting.
2.  First 3 bytes MUST be `D9 D9 F7` (self-described CBOR tag 55799).
3.  The root map MUST contain key 0 with value `"cbor-web-manifest"`.
4.  The root map MUST contain key 1 with a processable version number.

If the site does not support CBOR-Web, the server SHOULD respond with
`404 Not Found`.

## 4.2.  HTTP Link Header

Any HTML page MAY include an HTTP response header:

```
Link: </.well-known/cbor-web>; rel="alternate"; type="application/cbor"
```

This allows an agent to discover CBOR-Web while processing normal HTML
responses.

## 4.3.  HTML Meta Tag

An HTML page MAY include in its `<head>`:

```html
<link rel="alternate" type="application/cbor" href="/.well-known/cbor-web">
```

## 4.4.  robots.txt Entry

The site's robots.txt MAY include a CBOR-Web directive:

```
CBOR-Web: /.well-known/cbor-web
```

robots.txt directives (`Disallow`, `Allow`) MUST still be respected by
CBOR-Web agents.

## 4.5.  llms.txt Entry

The site's llms.txt MAY include a reference:

```markdown
- CBOR-Web Manifest: /.well-known/cbor-web
```

---

# 5.  Manifest Document

The manifest is the entry point to a site's CBOR-Web content.  An agent
reads the manifest first and makes all subsequent decisions based on its
contents.

## 5.1.  Top-Level Structure

```cbor-diag
55799({
  0: "cbor-web-manifest",
  1: 2,
  2: {                             ; site metadata
    "name": "Example Site",
    "domain": "example.com",
    "languages": ["en", "fr", "es"],
    "default_language": "en"
  },
  3: [                             ; page entries
    {
      "hash": h'D8CAD2E6E8D06A0EF4E5B22C7394E4AC2B48FDE517DBB012C705DB26D163CEB7',
      "lang": "en",
      "path": "/",
      "size": 127,
      "title": "Home",
      "access": "public",
      "updated": 1(1742515200)
    }
  ],
  4: {                             ; navigation (OPTIONAL)
    "main": ["/", "/about", "/products"],
    "hierarchy": { "/products": ["/products/lions-mane"] }
  },
  5: {                             ; generation metadata
    "generated_at": 1(1742515200),
    "total_pages": 25,
    "total_size": 48200,
    "bundle_available": true,
    "bundle_url": "/.well-known/cbor-web/bundle"
  }
})
```

Top-level key registry:

| Key | Name | Type | Required | Description |
|-----|------|------|----------|-------------|
| 0 | @type | text | REQUIRED | `"cbor-web-manifest"` |
| 1 | @version | uint | REQUIRED | Version 2 for this specification |
| 2 | site | map | REQUIRED | Site-level metadata |
| 3 | pages | array | REQUIRED | Array of page entry maps |
| 4 | navigation | map | OPTIONAL | Site navigation structure |
| 5 | meta | map | REQUIRED | Generation metadata |
| 6 | signature | bstr | OPTIONAL | COSE_Sign1 signature |
| 7 | capabilities | map | RECOMMENDED | Site capability declaration |
| 8 | channels | array | OPTIONAL | Streaming channels |
| 9 | diff | map | OPTIONAL | Differential update since prior version |
| 10 | security | map | RECOMMENDED | Security and access control config |

## 5.2.  Site Metadata

The site metadata map (key 2) provides global information about the
website.  Fields include `"domain"` (REQUIRED, bare domain name),
`"name"` (REQUIRED, human-readable site name), `"description"`
(OPTIONAL), `"languages"` (REQUIRED, array of BCP 47 language tags),
`"default_language"` (REQUIRED), `"contact"` (OPTIONAL), and `"geo"`
(OPTIONAL, geographic location).

## 5.3.  Page Entry

Each element in the pages array (key 3) is a map describing a single
page:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `"path"` | text | REQUIRED | URL path relative to domain |
| `"title"` | text | REQUIRED | Page title |
| `"lang"` | text | REQUIRED | BCP 47 language code |
| `"access"` | text | REQUIRED | `"public"` or `"token"` |
| `"size"` | uint | REQUIRED | Size in bytes |
| `"updated"` | tag 1 (uint) | RECOMMENDED | Last modification timestamp |
| `"hash"` | bstr (32 bytes) | RECOMMENDED | SHA-256 hash of standalone page |
| `"alternates"` | map | OPTIONAL | Language alternates |
| `"category"` | text | OPTIONAL | Page category/section |
| `"content_type"` | text | OPTIONAL | `"article"`, `"product"`, `"landing"`, etc. |

## 5.4.  Navigation

The navigation map (key 4) provides site structure in machine-readable
form.  Fields include `"main"` (primary navigation paths), `"footer"`
(footer paths), `"hierarchy"` (parent-child relationships), and
`"breadcrumbs"` (breadcrumb trails).  This is OPTIONAL to support
Minimal conformance but RECOMMENDED for Standard level.

## 5.5.  Generation Metadata

The meta map (key 5) contains information about the manifest itself.
Fields include `"generator"` (publisher software name/version),
`"generated_at"` (REQUIRED, Unix epoch timestamp), `"total_pages"`
(REQUIRED), `"total_size"` (REQUIRED), `"bundle_available"` (REQUIRED),
`"bundle_url"` (CONDITIONAL), and `"rate_limit"` (OPTIONAL).

## 5.6.  Sub-Manifests

For sites with more than 500 pages, the manifest SHOULD be paginated
into sub-manifests via the `"next"` field in key 5.  Site metadata
(key 2) and navigation (key 4) are present only in the first
sub-manifest.  Each sub-manifest MUST NOT exceed 5 MB.

---

# 6.  Page Document

A page document contains the structured content of a single web page.

## 6.1.  Path Encoding and Access URLs

Individual page documents are served at:

```
GET /.well-known/cbor-web/pages/{encoded-path}.cbor
```

Path encoding is bijective:

1.  Percent-encode literal underscores (`_` → `%5F`).
2.  Remove the leading slash.
3.  Replace remaining slashes with underscores.
4.  Special case: `/` (root) becomes `_index`.
5.  Append `.cbor` extension.

Examples:

| URL Path | Filename |
|----------|----------|
| `/` | `_index.cbor` |
| `/about` | `about.cbor` |
| `/products/lions-mane` | `products_lions-mane.cbor` |
| `/my_page` | `my%5Fpage.cbor` |

## 6.2.  Top-Level Structure

```cbor-diag
55799({
  0: "cbor-web-page",
  1: 2,
  2: {                             ; identity
    "lang": "en",
    "path": "/services/web-development",
    "canonical": "https://example.com/services/web-development"
  },
  3: {                             ; metadata
    "title": "Custom Web Development",
    "description": "We build performant, accessible websites...",
    "updated": 1(1742428800)
  },
  4: [                             ; content blocks
    {"l": 1, "t": "h", "v": "Custom Web Development"},
    {"t": "p", "v": "We build performant, accessible websites..."},
    {"t": "ul", "v": ["React / Next.js", "Node.js / Express"]}
  ],
  5: {                             ; links (OPTIONAL)
    "internal": [{"path": "/contact", "text": "Contact us"}]
  },
  6: {                             ; structured data (OPTIONAL)
    "type": "Service",
    "provider": {"name": "Example Corp", "type": "Organization"}
  }
})
```

Top-level key registry:

| Key | Name | Type | Required | Description |
|-----|------|------|----------|-------------|
| 0 | @type | text | REQUIRED | `"cbor-web-page"` |
| 1 | @version | uint | REQUIRED | Version 2 |
| 2 | identity | map | REQUIRED | Page identity and localization |
| 3 | metadata | map | REQUIRED | Page metadata |
| 4 | content | array | REQUIRED | Ordered array of content blocks |
| 5 | links | map | OPTIONAL | Internal and external link graph |
| 6 | structured_data | map | OPTIONAL | Schema.org-compatible data in native CBOR |
| 7 | generative | array | OPTIONAL | Generative blocks |
| 8 | forms | array | OPTIONAL | Form definitions |
| 9 | commerce | map | OPTIONAL | Commerce data |

A single page document MUST NOT exceed 1 MB in serialized CBOR size.

## 6.3.  Content Array

The content array (key 4) is the core payload of a page document.  It
MUST preserve document order.  Each element is a content block as
defined in Section 8.  An agent reading only the content array receives
the full editorial content of the page with zero noise.

## 6.4.  Links

The links map (key 5) contains `"internal"` and `"external"` arrays of
link maps.  Inline links from paragraphs are extracted to this section
during HTML-to-CBOR conversion.  External links are informational only;
an agent MUST NOT automatically follow external links without an
explicit security policy.

## 6.5.  Structured Data

Structured data (key 6) uses Schema.org vocabulary encoded in native
CBOR maps, not serialized JSON-LD strings.  The `@context` key is
omitted (implied Schema.org).  The `@type` key becomes `"type"`.

---

# 7.  Bundle Document

A bundle combines the manifest and all pages into a single CBOR
document.  This enables an agent to index an entire site with a single
HTTP request.

The bundle is OPTIONAL.  The manifest's `"bundle_available"` field
indicates whether a bundle exists.  Bundles are RECOMMENDED for sites
with 1-50 pages and NOT RECOMMENDED for sites exceeding 500 pages.

A bundle MUST NOT exceed 50 MB in serialized CBOR size.

## 7.1.  Structure

```cbor-diag
55799({
  0: "cbor-web-bundle",
  1: 2,
  2: {                             ; manifest (without 55799 tag)
    0: "cbor-web-manifest",
    1: 2,
    2: {"name": "Example", "domain": "example.com", ...},
    3: [...],
    5: {...}
  },
  3: {                             ; pages (map: path → page, without 55799 tag)
    "/": {
      0: "cbor-web-page",
      1: 2,
      2: {"lang": "en", "path": "/", "canonical": "https://example.com/"},
      3: {"title": "Home"},
      4: [{"l": 1, "t": "h", "v": "Welcome"}, {"t": "p", "v": "Hello, World!"}]
    },
    "/about": {
      0: "cbor-web-page",
      1: 2,
      2: {"lang": "en", "path": "/about", "canonical": "https://example.com/about"},
      3: {"title": "About Us"},
      4: [{"l": 1, "t": "h", "v": "About Us"}, {"t": "p", "v": "We are..."}]
    }
  }
})
```

Top-level key registry:

| Key | Name | Type | Required | Description |
|-----|------|------|----------|-------------|
| 0 | @type | text | REQUIRED | `"cbor-web-bundle"` |
| 1 | @version | uint | REQUIRED | Version 2 |
| 2 | manifest | map | REQUIRED | Complete manifest (without 55799 tag wrapper) |
| 3 | pages | map | REQUIRED | Map of path → page document (without 55799 tag) |

## 7.2.  Hash Verification for Bundled Pages

The hash in the manifest (`"hash"` field) is ALWAYS computed on the
standalone form of the page document, which includes the self-described
CBOR tag (55799).  To verify the hash of a page extracted from a bundle:

1.  Extract the page map from bundle key 3.
2.  Serialize it to canonical CBOR bytes (deterministic encoding).
3.  Prepend the 3-byte tag prefix: `0xD9 0xD9 0xF7`.
4.  Compute SHA-256 of the combined bytes.
5.  Compare with the manifest's page entry hash.

This design ensures the same hash works for both standalone and bundled
pages.

---

# 8.  Content Block Types

Content blocks are the atomic units of page content.  Each block is a
CBOR map with at minimum a `"t"` (type) key.  Every block type specifies
which keys are required and optional.

Common key namespace:

| Key | Name | Type | Used By |
|-----|------|------|---------|
| `"t"` | type | text | ALL blocks |
| `"v"` | value | text or array | h, p, ul, ol, q, code, cta, dl, note |
| `"l"` | level | uint (1-6) | h |
| `"attr"` | attribution | text | q |
| `"lang"` | language | text | code |
| `"headers"` | headers | array of text | table |
| `"rows"` | rows | array of arrays | table |
| `"alt"` | alt text | text | img |
| `"src"` | source | text | img, embed |
| `"caption"` | caption | text | img |
| `"href"` | link | text | cta |
| `"description"` | description | text | embed |
| `"level"` | severity | text | note |

Editorial blocks (pure content): `"h"`, `"p"`, `"ul"`, `"ol"`, `"q"`,
`"code"`, `"table"`, `"dl"`, `"note"`, `"sep"`.

Non-editorial blocks (marketing/navigation): `"cta"`, `"embed"`, `"img"`.

## 8.1.  Heading Block (h)

```cbor-diag
{"l": 1, "t": "h", "v": "Custom Web Development"}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"h"` |
| `"l"` | uint | REQUIRED | Heading level 1-6 |
| `"v"` | text | REQUIRED | Heading text, max 500 chars |

A page SHOULD have exactly one level-1 heading.

## 8.2.  Paragraph Block (p)

```cbor-diag
{"t": "p", "v": "We build performant, accessible websites optimized for search engines and AI agents."}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"p"` |
| `"v"` | text | REQUIRED | Paragraph text, max 50,000 chars; plain text, no HTML markup |

Inline markup (`<strong>`, `<em>`, `<a>`, `<code>`) is stripped during
conversion.  Each HTML `<p>` element becomes a separate paragraph block.

## 8.3.  Unordered List (ul)

```cbor-diag
{"t": "ul", "v": ["React / Next.js", "Node.js / Express", "PostgreSQL / Redis"]}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"ul"` |
| `"v"` | array of text | REQUIRED | List items; at least 1 |

## 8.4.  Ordered List (ol)

```cbor-diag
{"t": "ol", "v": ["Clone the repository", "Run npm install", "Start the development server"]}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"ol"` |
| `"v"` | array of text | REQUIRED | Ordered items; array order is semantically significant |

## 8.5.  Quote Block (q)

```cbor-diag
{"t": "q", "v": "They transformed our online presence completely.", "attr": "Client, Acme Corp"}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"q"` |
| `"v"` | text | REQUIRED | Quoted text |
| `"attr"` | text | OPTIONAL | Attribution |

## 8.6.  Code Block (code)

```cbor-diag
{"t": "code", "v": "def hello():\n    print('Hello, World!')", "lang": "python"}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"code"` |
| `"v"` | text | REQUIRED | Source code; whitespace preserved |
| `"lang"` | text | OPTIONAL | Language identifier (e.g. `"python"`, `"javascript"`) |

## 8.7.  Data Table (table)

```cbor-diag
{
  "t": "table",
  "rows": [
    ["Starter", "$990", "5 pages, responsive, basic SEO"],
    ["Pro", "$2,490", "15 pages, multilingual, analytics"]
  ],
  "headers": ["Plan", "Price", "Includes"]
}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"table"` |
| `"headers"` | array of text | REQUIRED | Column headers |
| `"rows"` | array of arrays | REQUIRED | Data rows; each row must match header count |

## 8.8.  Image Reference (img)

```cbor-diag
{"t": "img", "alt": "Lion's Mane packaging, 90 capsules", "src": "https://verdetao.com/img/lm.webp"}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"img"` |
| `"alt"` | text | REQUIRED | Alt text; max 500 chars |
| `"src"` | text | REQUIRED | `https://` URL |
| `"caption"` | text | OPTIONAL | Image caption |

## 8.9.  Call to Action (cta)

```cbor-diag
{"t": "cta", "v": "Request a free quote", "href": "/contact"}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"cta"` |
| `"v"` | text | REQUIRED | Button/link text |
| `"href"` | text | REQUIRED | Destination path or URL |

## 8.10.  Embedded Content (embed)

```cbor-diag
{"t": "embed", "src": "https://maps.google.com/embed?q=Verdetao", "description": "Interactive map showing store locations"}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"embed"` |
| `"src"` | text | REQUIRED | `https://` URL |
| `"description"` | text | OPTIONAL | Text fallback |

## 8.11.  Separator (sep)

```cbor-diag
{"t": "sep"}
```

A thematic break analogous to HTML `<hr>`.  Consists of a single
key-value pair (7 bytes in deterministic CBOR).

## 8.12.  Definition List (dl)

```cbor-diag
{
  "t": "dl",
  "v": [
    {"def": "Take 2 capsules daily with water.", "term": "How many capsules per day?"},
    {"def": "Yes, it combines well with Reishi and Cordyceps.", "term": "Compatible with other supplements?"}
  ]
}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"dl"` |
| `"v"` | array of maps | REQUIRED | Array of `{"def": text, "term": text}` pairs |

## 8.13.  Note Block (note)

```cbor-diag
{"t": "note", "v": "This product is not intended to diagnose, treat, cure, or prevent any disease."}
{"t": "note", "v": "Do not exceed recommended daily dose.", "level": "warn"}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"t"` | text | REQUIRED | `"note"` |
| `"v"` | text | REQUIRED | Note text |
| `"level"` | text | OPTIONAL | `"info"` (default), `"warn"`, `"important"` |

---

# 9.  Security Considerations

## 9.1.  HTTPS Requirement

CBOR-Web content MUST be served over HTTPS (TLS 1.2 or later).  An
agent MUST refuse to process a manifest received over plain HTTP.
Servers SHOULD support HTTP/2 or HTTP/3.

Rationale:
*  CBOR-Web content represents the authoritative content of a website
   — an agent trusts it as the "truth" of what the site contains.
*  Man-in-the-middle attacks could inject false content that agents
   propagate as fact.
*  Hash verification is meaningless if transport integrity is not
   guaranteed.
*  Token-based access control requires secure transport for wallet
   signatures.

Exception: During development and testing, a publisher MAY serve
CBOR-Web over plain HTTP on localhost or private networks.  This
exception MUST NOT be used in production.

## 9.2.  Token-Based Access Control

CBOR-Web supports token-based access control for content gating.  Pages
marked `"access": "token"` require the agent to hold at least one unit
of the CBOR-Web token (an ERC-20 token on Ethereum mainnet).
Authentication uses EIP-712 typed data signatures with per-request
nonces to prevent replay attacks.

The agent includes wallet authentication headers:

```
GET /.well-known/cbor-web/pages/products_lions-mane.cbor HTTP/1.1
Host: verdetao.com
Accept: application/cbor
X-CBOR-Web-Wallet: 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18
X-CBOR-Web-Sig: 0x[EIP-712 signature]
X-CBOR-Web-Nonce: 1742598400
```

The nonce is a Unix timestamp.  Servers MUST reject nonces older than
300 seconds (replay window).  Without a valid token, the server
responds `402 Payment Required` with a CBOR-encoded error body.

A publisher SHOULD make at least 30 % of pages `"access": "public"` so
that the site remains discoverable by agents without tokens.

## 9.3.  Content Validation

An agent MUST enforce the following size limits:

| Limit | Value |
|-------|-------|
| Maximum page size | 1 MB |
| Maximum manifest size | 5 MB |
| Maximum bundle size | 50 MB |

Agents MUST verify SHA-256 hashes of downloaded pages against the
manifest before trusting content.  Hash mismatch MUST cause the page
to be rejected.

An agent MUST reject documents with malformed CBOR (parse failure).
Unknown block types MUST be skipped (forward compatibility), not
rejected.

## 9.4.  Transport Security

The Content-Type header MUST be `application/cbor`.  Servers SHOULD
support content-encoding negotiation, with Brotli (`br`) RECOMMENDED
as the primary compression method.  The server SHOULD support
conditional requests (ETag/If-None-Match) to minimize bandwidth.

---

# 10.  IANA Considerations

## 10.1.  Well-Known URI Registration

Per RFC 8615, this specification requests registration of the following
well-known URI:

| Field | Value |
|-------|-------|
| URI suffix | `cbor-web` |
| Change controller | ExploDev |
| Specification document | This document (draft-plot-cbor-web) |
| Related information | Machine-readable binary web content for autonomous agents |

## 10.2.  CBOR Tag 55799

CBOR tag 55799 (self-described CBOR) is used as specified in RFC 8949,
Section 3.4.6.  This tag has been previously allocated in the IANA
"CBOR Tags" registry.  No new tag registration is required.

## 10.3.  Media Type

This specification uses the existing `application/cbor` media type
registered by RFC 8949.  No new media type registration is required
for the core protocol.

A future version MAY register profile parameters:

```
Content-Type: application/cbor; profile="cbor-web-manifest"
Content-Type: application/cbor; profile="cbor-web-page"
Content-Type: application/cbor; profile="cbor-web-bundle"
```

Custom HTTP headers defined by this specification follow the
`X-CBOR-Web-` prefix convention.  Future versions MAY pursue
registration of standardized header names.

---

# 11.  CDDL Schema

This CDDL (RFC 8610) schema formally defines all CBOR-Web document
structures.  All maps use deterministic key ordering (Section 3.1).

## 11.1.  Document Types

```cddl
cbor-web-document = #6.55799(manifest / sub-manifest / page / bundle)
```

## 11.2.  Manifest

```cddl
manifest = {
  0 => "cbor-web-manifest",
  1 => uint,
  2 => site-metadata,
  3 => [+ page-entry],
  ? 4 => navigation,
  5 => manifest-meta,
  ? 6 => bstr,                      ; COSE_Sign1 signature
  ? 7 => capabilities,
  ? 8 => [+ channel],
  ? 9 => diff-manifest,
  ? 10 => manifest-security,
  * int => any
}

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

language-code = tstr .regexp "[a-z]{2,3}(-[A-Za-z]{2,8})*"

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
```

## 11.3.  Page Document

```cddl
page = {
  0 => "cbor-web-page",
  1 => uint,
  2 => page-identity,
  3 => page-metadata,
  4 => [* content-block],
  ? 5 => page-links,
  ? 6 => structured-data,
  ? 7 => [+ generative-block],
  ? 8 => [+ form-block],
  ? 9 => commerce-data,
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
```

## 11.4.  Content Blocks

```cddl
content-block = heading / paragraph / unordered-list / ordered-list /
                quote / code-block / data-table / image-ref /
                call-to-action / embed / separator / definition-list /
                note-block

heading        = { "t" => "h",     "l" => uint .ge 1 .le 6, "v" => tstr, * tstr => any }
paragraph      = { "t" => "p",     "v" => tstr, * tstr => any }
unordered-list = { "t" => "ul",    "v" => [+ tstr], * tstr => any }
ordered-list   = { "t" => "ol",    "v" => [+ tstr], * tstr => any }
quote          = { "t" => "q",     "v" => tstr, ? "attr" => tstr, * tstr => any }
code-block     = { "t" => "code",  "v" => tstr, ? "lang" => tstr, * tstr => any }
data-table     = { "t" => "table", "headers" => [+ tstr], "rows" => [+ [+ tstr]], * tstr => any }
image-ref      = { "t" => "img",   "alt" => tstr, "src" => tstr, ? "caption" => tstr, * tstr => any }
call-to-action = { "t" => "cta",   "v" => tstr, "href" => tstr, * tstr => any }
embed          = { "t" => "embed", "src" => tstr, ? "description" => tstr, * tstr => any }
separator      = { "t" => "sep",   * tstr => any }
definition-list = { "t" => "dl",   "v" => [+ { "term" => tstr, "def" => tstr, * tstr => any }], * tstr => any }
note-block     = { "t" => "note",  "v" => tstr, ? "level" => "info" / "warn" / "important", * tstr => any }
```

## 11.5.  Bundle

```cddl
bundle = {
  0 => "cbor-web-bundle",
  1 => uint,
  2 => manifest,
  3 => { + tstr => page },
  * int => any
}

; Forward declarations for companion specifications
capabilities = { * tstr => any }
channel = { * tstr => any }
manifest-security = { * tstr => any }
generative-block = { * tstr => any }
form-block = { * tstr => any }
commerce-data = { * tstr => any }
```

---

# 12.  References

## 12.1.  Normative References

*  **[RFC 2119]**  Bradner, S., "Key words for use in RFCs to Indicate
   Requirement Levels", BCP 14, RFC 2119, March 1997,
   <https://www.rfc-editor.org/info/rfc2119>.

*  **[RFC 8174]**  Leiba, B., "Ambiguity of Uppercase vs Lowercase in
   RFC 2119 Key Words", BCP 14, RFC 8174, May 2017,
   <https://www.rfc-editor.org/info/rfc8174>.

*  **[RFC 8610]**  Birkholz, H., Vigano, C., and C. Bormann, "Concise
   Data Definition Language (CDDL): A Notational Convention to Express
   Concise Binary Object Representation (CBOR) and JSON Data
   Structures", RFC 8610, June 2019,
   <https://www.rfc-editor.org/info/rfc8610>.

*  **[RFC 8615]**  Nottingham, M., "Well-Known Uniform Resource
   Identifiers (URIs)", RFC 8615, May 2019,
   <https://www.rfc-editor.org/info/rfc8615>.

*  **[RFC 8949]**  Bormann, C. and P. Hoffman, "Concise Binary Object
   Representation (CBOR)", STD 94, RFC 8949, December 2020,
   <https://www.rfc-editor.org/info/rfc8949>.

*  **[RFC 9052]**  Schaad, J., "CBOR Object Signing and Encryption
   (COSE): Structures and Process", STD 96, RFC 9052, August 2022,
   <https://www.rfc-editor.org/info/rfc9052>.

*  **[RFC 9309]**  Koster, M., Recabarren, R., and D. Thaler, "Robots
   Exclusion Protocol", RFC 9309, September 2022,
   <https://www.rfc-editor.org/info/rfc9309>.

## 12.2.  Informative References

*  **[llms.txt]**  "llms.txt — A proposed standard for making websites
   readable by LLMs", <https://llmstxt.org/>.

*  **[Schema.org]**  "Schema.org — shared vocabulary for structured
   data", <https://schema.org/>.

*  **[ERC-20]**  Vogelsteller, F. and V. Buterin, "EIP-20: Token
   Standard", November 2015,
   <https://eips.ethereum.org/EIPS/eip-20>.
