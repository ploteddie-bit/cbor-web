# CBOR-Web Multimedia Specification

**Rich Media Blocks for Machine-Readable Web Content — Images, Video, Audio, Documents, Diagrams, Streaming**

```
Status:       Proposed Standard
Version:      2.1
Date:         2026-03-21
Authors:      ExploDev (Eddie Plot, Claude)
Format:       CBOR (RFC 8949)
Schema:       CDDL (RFC 8610)
License:      CC BY 4.0
Repository:   https://github.com/explodev/cbor-web
Document:     2 of 6 — CBOR-WEB-MULTIMEDIA.md
Companion:    CBOR-WEB-CORE.md, CBOR-WEB-GENERATIVE.md,
              CBOR-WEB-SECURITY.md, CBOR-WEB-ECONOMICS.md,
              CBOR-WEB-REFERENCE.md
```

---

## About This Document

This document is **part 2 of 6** of the CBOR-Web v2.1 specification suite. It defines **multimedia content blocks** that extend the core content blocks defined in CBOR-WEB-CORE.md §8. It also defines real-time streaming channels.

All multimedia blocks are placed in the page's content array (key 4), alongside core blocks. They share the same content block key namespace (`"t"`, `"v"`, `"src"`, etc.) and follow the same deterministic encoding rules (CBOR-WEB-CORE.md §3.1).

| Document | Scope | Reference |
|----------|-------|-----------|
| CBOR-WEB-CORE.md | Binary format, core content blocks, discovery, transport, caching | Prerequisite for this document |
| **CBOR-WEB-MULTIMEDIA.md** (this document) | Rich images, video, audio, documents, diagrams, live streaming | |
| CBOR-WEB-GENERATIVE.md | Templates, schemas, APIs, executables, workflows, forms, commerce | |
| CBOR-WEB-SECURITY.md | Threat model, token access control, binary protection, sandbox | |
| CBOR-WEB-ECONOMICS.md | Token economics, pricing, launch plan, regulation | |
| CBOR-WEB-REFERENCE.md | Unified CDDL, all test vectors, glossary, changelog | |

**Prerequisites**: This document assumes familiarity with CBOR-WEB-CORE.md, particularly:
- §3.1 (Deterministic Encoding — map key ordering)
- §3.7 (Binary Data Encoding — raw bytes in bstr, not base64)
- §8.1-8.2 (Content Block overview and key registry)
- §8.5 (Trust Level Classification — all multimedia blocks are trust level 0)

---

## Table of Contents

1. [Introduction and Rationale](#1-introduction-and-rationale)
2. [Rich Image Block](#2-rich-image-block)
3. [Video Block](#3-video-block)
4. [Audio Block](#4-audio-block)
5. [Document Block](#5-document-block)
6. [Diagram Block](#6-diagram-block)
7. [Live Stream Block](#7-live-stream-block)
8. [Real-Time Streaming Channels](#8-real-time-streaming-channels)
9. [Multimedia in the Content Array](#9-multimedia-in-the-content-array)
10. [Agent Processing Strategies](#10-agent-processing-strategies)
- [Appendix A: Multimedia CDDL Schema](#appendix-a-multimedia-cddl-schema)
- [Appendix B: Multimedia Test Vectors](#appendix-b-multimedia-test-vectors)
- [Appendix C: Multimedia Examples](#appendix-c-multimedia-examples)
- [References](#references)

---

## 1. Introduction and Rationale

### 1.1 The Problem with Media on the Web

The core `"img"` block defined in CBOR-WEB-CORE.md §8.9 is a simple reference: a URL and an alt text. This is sufficient for basic content extraction, but an AI agent operating on a modern website needs much more:

1. **Semantic role**: Is this image the company logo, a product photo, a decorative background, or an editorial illustration? An agent that knows the image is a `"product_photo"` can prioritize it for commerce queries, while a `"decorative"` image can be skipped entirely.

2. **Technical metadata**: What are the dimensions? What format? How large is the file? An agent deciding whether to download the image needs this information before making the request. A 84 KB WebP product photo is worth downloading; a 5 MB decorative PNG is not.

3. **Content intelligence**: What does the image actually show? The `"alt"` text provides a brief accessibility description, but an AI-generated detailed description (`"ai_description"`) can give the agent a full understanding of the image content without downloading the binary file.

4. **Video and audio intelligence**: An agent does not watch video or listen to audio. It reads transcriptions, chapter markers, and speaker attributions. Without these, the entire multimedia content of a page is opaque to the agent.

5. **Document intelligence**: A referenced PDF or DOCX contains text that the agent cannot access without downloading and parsing the binary file. A `"text_extract"` and `"table_of_contents"` make the document's content accessible without the download.

6. **Diagram intelligence**: An SVG diagram is meaningless to an agent as rendered pixels. But a structured description with entities and relationships makes the diagram's information fully accessible.

### 1.2 Design Approach

CBOR-Web multimedia blocks follow a consistent design philosophy:

**Principle: An agent should be able to understand a media element's content, role, and importance WITHOUT downloading the binary asset.**

Every multimedia block provides:
- **What it is**: type code + semantic role
- **What it shows**: text descriptions (alt, ai_description, text_extract, transcription)
- **Technical details**: format, size, dimensions, duration, codec
- **Where to get it**: URL reference (`"src"`)
- **Optionally, the data itself**: inline binary for small assets (< 10 KB)

This means an agent can process a page with 20 images, 3 videos, and 2 PDFs in milliseconds — reading the metadata — without downloading a single binary file. If it later decides it needs the actual image or video, the URL is right there.

### 1.3 Trust Level

All multimedia blocks have **trust level 0 (declarative)**. They contain only data — no executable code, no network interactions, no template logic. An agent can process all multimedia blocks freely without security concerns.

The `"trust": 0` key is REQUIRED in all multimedia blocks for consistency with the trust level classification system (see CBOR-WEB-SECURITY.md §8.5), even though it is always the same value. This makes the trust level explicit and machine-parseable.

### 1.4 Block Placement

Multimedia blocks are placed in the page's content array (key 4), alongside core blocks. They appear in document order — where the image, video, or audio appears in the original HTML page.

```cbor-diag
55799({
  0: "cbor-web-page",
  1: 2,
  2: { ... },
  3: { ... },
  4: [
    {"l": 1, "t": "h", "v": "Product Guide"},
    {"t": "p", "v": "Watch our video guide to learn more."},
    {"t": "video", "trust": 0, "src": "...", "title": "Guide", ...},
    {"t": "p", "v": "Key features of the product:"},
    {"t": "image", "trust": 0, "src": "...", "alt": "...", ...},
    {"t": "ul", "v": ["Feature 1", "Feature 2"]}
  ]
})
```

The video appears between two paragraphs, exactly where it appears in the HTML. The image follows the features introduction. Document order is preserved.

---

## 2. Rich Image Block

### 2.1 Overview

**Type code:** `"image"` | **Trust level:** 0 (declarative)

The `"image"` block replaces the core `"img"` block (CBOR-WEB-CORE.md §8.9) for publishers seeking richer metadata. The v1.0 `"img"` block remains valid for backward compatibility — a publisher MAY use either `"img"` or `"image"`, but SHOULD prefer `"image"` for new content.

The difference:

| Feature | `"img"` (core) | `"image"` (multimedia) |
|---------|----------------|------------------------|
| Alt text | ✅ REQUIRED | ✅ REQUIRED |
| Source URL | ✅ REQUIRED | ✅ REQUIRED |
| Caption | ✅ OPTIONAL | ✅ OPTIONAL |
| Semantic role | ❌ | ✅ REQUIRED |
| Dimensions | ❌ | ✅ RECOMMENDED |
| Format | ❌ | ✅ RECOMMENDED |
| File size | ❌ | ✅ RECOMMENDED |
| AI description | ❌ | ✅ OPTIONAL |
| Dominant color | ❌ | ✅ OPTIONAL |
| EXIF data | ❌ | ✅ OPTIONAL |
| Inline data | ❌ | ✅ OPTIONAL (< 10 KB) |
| Trust level | Implicit 0 | Explicit `"trust": 0` |

### 2.2 Structure

```cbor-diag
{
  "t": "image",
  "trust": 0,
  "alt": "Flacon de Lion's Mane Verdetao, 90 capsules, etiquette verte",
  "src": "https://verdetao.com/images/lions-mane-packaging.webp",
  "caption": "Lion's Mane — Criniere de Lion, 90 capsules bio",
  "format": "webp",
  "file_size": 84200,
  "ai_description": "A green glass bottle with a white cap, labeled Lion's Mane with botanical illustrations of Hericium erinaceus mushroom. 90 capsules. Organic EU certification logo visible.",
  "dimensions": {"h": 800, "w": 1200},
  "exif": {
    "camera": "Canon EOS R5",
    "date": "2026-02-15",
    "focal_length": "85mm"
  },
  "inline_data": null,
  "dominant_color": "#2D5A27",
  "semantic_role": "product_photo"
}
```

Note: Map keys are shown in **deterministic order** (CBOR-WEB-CORE.md §3.1):
- 2-byte keys first: `"t"` (`61 74`)
- Then by ascending encoded length: `"alt"` (4B), `"src"` (4B, `73>61`), `"exif"` (5B), `"format"` (7B), `"caption"` (8B), `"file_size"` (10B), `"dimensions"` (11B), `"inline_data"` (12B), `"dominant_color"` (15B), `"ai_description"` (15B, `61>64`... needs checking), `"semantic_role"` (14B)

For readability, examples in this section show keys in logical grouping order. The binary encoding uses deterministic order.

### 2.3 Field Reference

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"image"` | Block type code |
| `"trust"` | uint | REQUIRED | `0` | Trust level. Always 0 for images. |
| `"src"` | text | REQUIRED | `https://` URL. See CBOR-WEB-SECURITY.md §11.3 for URL validation. | Image URL. The agent MAY fetch this URL to download the image binary. |
| `"alt"` | text | REQUIRED | Max 500 characters. MUST be meaningful (not empty, not "Image"). | Accessibility text. Human-written description of the image content. This is the primary text representation of the image for agents that do not download the binary. |
| `"semantic_role"` | text | REQUIRED | One of the defined roles (§2.4) or a custom role. | The purpose of this image within the page. |
| `"dimensions"` | map | RECOMMENDED | `{"w": uint, "h": uint}` in pixels. | Image dimensions. `"w"` is width, `"h"` is height. |
| `"format"` | text | RECOMMENDED | MIME subtype only (no `image/` prefix). | Image format. Examples: `"webp"`, `"png"`, `"jpeg"`, `"svg+xml"`, `"avif"`, `"gif"`. |
| `"file_size"` | uint | RECOMMENDED | In bytes. | File size of the image. Helps agents decide whether to download. |
| `"dominant_color"` | text | OPTIONAL | CSS hex color: `"#RRGGBB"` | The dominant color of the image. Useful for placeholder rendering and visual analysis. |
| `"ai_description"` | text | OPTIONAL | Max 2000 characters. | AI-generated detailed description of the image content. Richer than `"alt"` — may describe objects, text visible in the image, spatial relationships, colors, etc. |
| `"caption"` | text | OPTIONAL | Max 500 characters. | Caption text as displayed below the image on the HTML page. |
| `"exif"` | map | OPTIONAL | Keys: standard EXIF field names (text). Values: text or uint. | Simplified EXIF metadata. Common keys: `"camera"`, `"date"`, `"focal_length"`, `"iso"`, `"aperture"`, `"exposure"`, `"gps_lat"`, `"gps_lon"`. |
| `"inline_data"` | bstr or null | OPTIONAL | If bstr: max 50 KB. Raw image bytes, NOT base64. | Embedded image data for small icons/logos. See §2.5. `null` means "no inline data". |

### 2.4 Semantic Roles

The `"semantic_role"` field tells the agent **why** this image is on the page. This is a machine-actionable classification that does not exist in HTML.

| Role | Description | Agent Priority | Example |
|------|-------------|---------------|---------|
| `"logo"` | Brand logo or icon | **HIGH** — identifies the entity/brand | Company logo in header |
| `"product_photo"` | Product packaging, product shot, product in use | **HIGH** — essential for commerce queries | E-commerce product image |
| `"hero"` | Main banner/hero image at the top of the page | **MEDIUM** — page context and atmosphere | Landing page hero |
| `"illustration"` | Editorial illustration supporting article content | **MEDIUM** — aids comprehension | Blog post illustration |
| `"screenshot"` | UI screenshot, software interface, app mockup | **MEDIUM** — technical context | Documentation screenshot |
| `"avatar"` | Person photo, user avatar, team member headshot | **LOW** — person identification | Team page headshots |
| `"diagram"` | Chart, graph, technical diagram | **HIGH** — use `"diagram"` block type (§6) for structured data | Flowchart, architecture diagram |
| `"decorative"` | Purely decorative: gradient, pattern, background texture, visual separator | **SKIP** — no informational value | Background patterns, dividers |
| `"infographic"` | Data visualization, infographic with embedded data | **HIGH** — contains structured information | Revenue chart, market analysis |
| `"photo_editorial"` | Article/blog photograph (not product-related) | **MEDIUM** — editorial context | Nature photo in blog post |

**Custom roles**: A publisher MAY use custom role values beyond this registry. An agent MUST treat unknown roles as `"illustration"` (medium priority) by default.

**Agent filtering**: An agent MAY use the semantic role to filter images:
- For editorial-only consumption: skip `"decorative"`, `"hero"`, `"avatar"`
- For commerce queries: prioritize `"product_photo"`, `"logo"`
- For technical documentation: prioritize `"screenshot"`, `"diagram"`

### 2.5 Inline Image Data

A publisher SHOULD embed images inline (via `"inline_data"`) when:
- The image is an **icon, favicon, or small logo** (< 10 KB)
- The image is **critical for understanding the content** (e.g., a tiny technical diagram)
- The agent would otherwise need an additional HTTP request for a very small file

A publisher MUST NOT embed images inline when:
- File size exceeds **50 KB** (hard limit)
- The image is `"decorative"` (decorative images should never be inlined)
- The image is a **photograph** (use URL reference instead — photos are typically > 50 KB)

**Binary encoding**: The `"inline_data"` value is a CBOR byte string (major type 2) containing the **raw image bytes**. It is NOT base64-encoded text. CBOR byte strings are already binary — adding base64 would increase the size by 33% for zero benefit.

```cbor-diag
; CORRECT — raw bytes in bstr
"inline_data": h'89504E470D0A1A0A...'   ; PNG file bytes

; WRONG — base64 text in bstr
"inline_data": h'6956424F5277304B...'   ; base64 of the PNG, wasteful

; ALSO WRONG — base64 text in tstr
"inline_data": "iVBORw0KGgo..."          ; text string, wrong CBOR type
```

When `"inline_data"` is present, `"src"` SHOULD still contain the URL as a fallback. An agent that prefers URL-based loading can ignore inline data.

When `"inline_data"` is `null`, it explicitly means "no inline data — use the URL". This is distinct from the key being absent (which is ambiguous — the publisher might not support inline data at all).

### 2.6 EXIF Data

The `"exif"` map provides simplified EXIF metadata. It is intentionally NOT a full EXIF dump — it contains only human-readable fields that an agent might use for content understanding.

| EXIF Key | Type | Example | Description |
|----------|------|---------|-------------|
| `"camera"` | text | `"Canon EOS R5"` | Camera make and model |
| `"date"` | text | `"2026-02-15"` | Photo date (ISO 8601 date) |
| `"focal_length"` | text | `"85mm"` | Focal length |
| `"iso"` | uint | `400` | ISO sensitivity |
| `"aperture"` | text | `"f/2.8"` | Aperture |
| `"exposure"` | text | `"1/250"` | Exposure time |
| `"gps_lat"` | float | `36.7749` | GPS latitude |
| `"gps_lon"` | float | `-4.4194` | GPS longitude |
| `"software"` | text | `"Adobe Lightroom"` | Processing software |

A publisher SHOULD include `"date"` and `"camera"` when available. GPS coordinates SHOULD be included only when the image location is editorially relevant (e.g., travel photography) and the photographer has consented to sharing location data.

### 2.7 Dimensions Map

```cbor-diag
"dimensions": {"h": 800, "w": 1200}
```

Key order: `"h"` (2B, `61 68`) < `"w"` (2B, `61 77`). The `"h"` key sorts before `"w"` because `0x68 < 0x77`.

| Key | Type | Description |
|-----|------|-------------|
| `"w"` | uint | Width in pixels |
| `"h"` | uint | Height in pixels |

An agent can use dimensions to:
- Estimate image aspect ratio (landscape vs portrait vs square)
- Determine if the image is high-resolution enough for its purpose
- Calculate layout space requirements

### 2.8 Example: Product Photo

```cbor-diag
{
  "t": "image",
  "trust": 0,
  "alt": "Flacon de Lion's Mane Verdetao, 90 capsules",
  "src": "https://verdetao.com/img/lions-mane-front.webp",
  "format": "webp",
  "file_size": 84200,
  "dimensions": {"h": 800, "w": 1200},
  "dominant_color": "#2D5A27",
  "semantic_role": "product_photo",
  "ai_description": "A dark green glass bottle with a white screw cap. The label reads 'Lion's Mane' in white serif font on a forest green background. Below the name, botanical illustrations show the Hericium erinaceus mushroom in white line art. The label indicates '90 capsules' and shows the EU organic certification logo in the bottom-left corner. The Verdetao brand name appears at the top of the label."
}
```

### 2.9 Example: Decorative Image (Skip)

```cbor-diag
{
  "t": "image",
  "trust": 0,
  "alt": "Decorative green gradient background",
  "src": "https://verdetao.com/img/bg-gradient.webp",
  "format": "webp",
  "file_size": 2100,
  "semantic_role": "decorative"
}
```

An agent SHOULD skip this image entirely — it has no informational value. The `"semantic_role": "decorative"` is the signal.

### 2.10 Example: Inline Logo

```cbor-diag
{
  "t": "image",
  "trust": 0,
  "alt": "Verdetao logo",
  "src": "https://verdetao.com/img/logo.png",
  "format": "png",
  "file_size": 3400,
  "dimensions": {"h": 64, "w": 200},
  "semantic_role": "logo",
  "inline_data": h'89504E470D0A1A0A0000000D49484452...'
}
```

The logo is 3.4 KB — well within the 10 KB inline threshold. An agent can render or analyze the logo without an additional HTTP request.

---

## 3. Video Block

### 3.1 Overview

**Type code:** `"video"` | **Trust level:** 0 (declarative)

An AI agent does not watch video. It reads the **transcription** and **chapter structure**. The video block provides these text-based representations alongside technical metadata, enabling an agent to fully understand video content without downloading or playing the binary file.

### 3.2 Structure

```cbor-diag
{
  "t": "video",
  "trust": 0,
  "src": "https://verdetao.com/videos/how-to-take-lions-mane.mp4",
  "title": "Comment prendre le Lion's Mane — Guide complet",
  "duration_seconds": 187,
  "resolution": {"h": 1080, "w": 1920},
  "codec": "h264",
  "file_size": 24500000,
  "thumbnail_url": "https://verdetao.com/videos/thumbs/lions-mane-guide.webp",
  "transcription": {
    "format": "timestamped",
    "lang": "fr",
    "segments": [
      {"end": 12,  "text": "Bonjour, dans cette video nous allons voir comment prendre correctement le Lion's Mane.", "start": 0},
      {"end": 35,  "text": "La posologie recommandee est de deux capsules par jour, le matin, avec un grand verre d'eau.", "start": 12},
      {"end": 58,  "text": "Vous pouvez le prendre a jeun ou pendant le petit-dejeuner, les deux fonctionnent bien.", "start": 35},
      {"end": 95,  "text": "Parlons maintenant des combinaisons possibles avec d'autres champignons fonctionnels.", "start": 58},
      {"end": 140, "text": "Le Lion's Mane se combine tres bien avec le Reishi pour un effet relaxant, ou le Cordyceps pour l'energie.", "start": 95},
      {"end": 187, "text": "Merci d'avoir regarde. N'hesitez pas a consulter notre blog pour plus d'informations.", "start": 140}
    ]
  },
  "chapters": [
    {"timestamp": 0,   "title": "Introduction"},
    {"timestamp": 12,  "title": "Posologie"},
    {"timestamp": 58,  "title": "Combinaisons"},
    {"timestamp": 140, "title": "Conclusion"}
  ]
}
```

### 3.3 Field Reference

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"video"` | Block type |
| `"trust"` | uint | REQUIRED | `0` | Trust level |
| `"src"` | text | REQUIRED | `https://` URL | Video file URL |
| `"title"` | text | REQUIRED | Max 300 characters | Video title. Descriptive — not a filename. |
| `"duration_seconds"` | uint | REQUIRED | Positive integer | Duration in seconds. Allows agent to estimate processing time. |
| `"resolution"` | map | RECOMMENDED | `{"w": uint, "h": uint}` | Video resolution in pixels. |
| `"codec"` | text | OPTIONAL | | Video codec: `"h264"`, `"h265"`, `"vp9"`, `"av1"` |
| `"file_size"` | uint | OPTIONAL | In bytes | Video file size. |
| `"thumbnail_url"` | text | RECOMMENDED | `https://` URL | Preview image URL. Agent can use this as a visual reference without downloading the video. |
| `"transcription"` | map | RECOMMENDED | See §3.4 | Text transcription of the video content. **This is the most important field for agent consumption.** |
| `"chapters"` | array | OPTIONAL | `[{"timestamp": uint, "title": text}, ...]` | Chapter markers with timestamps in seconds and titles. |

### 3.4 Transcription Format

The transcription is the **primary way an agent consumes video content**. Two formats are supported, defined as distinct CDDL types to enforce structural correctness.

#### 3.4.1 Plain Transcription

For simple videos where timestamps are not needed. The entire transcript is a single text string.

```cbor-diag
"transcription": {
  "format": "plain",
  "lang": "fr",
  "text": "Bonjour, dans cette video nous allons voir comment prendre correctement le Lion's Mane. La posologie recommandee est de deux capsules par jour, le matin, avec un grand verre d'eau. Vous pouvez le prendre a jeun ou pendant le petit-dejeuner. Parlons maintenant des combinaisons possibles avec d'autres champignons fonctionnels. Le Lion's Mane se combine tres bien avec le Reishi pour un effet relaxant, ou le Cordyceps pour l'energie. Merci d'avoir regarde."
}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"format"` | text | REQUIRED | `"plain"` |
| `"lang"` | text | REQUIRED | ISO 639-1 language code |
| `"text"` | text | REQUIRED | Complete transcription text |

**CDDL:**
```cddl
plain-transcription = {
  "format" => "plain",
  "lang" => language-code,
  "text" => tstr,
  * tstr => any
}
```

#### 3.4.2 Timestamped Transcription

For videos with chapters or seeking capability. Each segment has start/end timestamps (in seconds) and text.

```cbor-diag
"transcription": {
  "format": "timestamped",
  "lang": "fr",
  "segments": [
    {"end": 12,  "start": 0,  "text": "Bonjour, dans cette video..."},
    {"end": 35,  "start": 12, "text": "La posologie recommandee..."},
    {"end": 58,  "start": 35, "text": "Vous pouvez le prendre a jeun..."}
  ]
}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"format"` | text | REQUIRED | `"timestamped"` |
| `"lang"` | text | REQUIRED | ISO 639-1 language code |
| `"segments"` | array | REQUIRED | Array of timestamped segments |

Each segment:

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"start"` | uint | REQUIRED | Start time in seconds |
| `"end"` | uint | REQUIRED | End time in seconds. MUST be > `"start"`. |
| `"text"` | text | REQUIRED | Transcribed text for this time range |

Key order within segments: `"end"` (4B) < `"text"` (5B) < `"start"` (6B). Wait — let me compute:
- `"end"` = 3 chars → `63 656E64` = 4 bytes
- `"text"` = 4 chars → `64 74657874` = 5 bytes
- `"start"` = 5 chars → `65 7374617274` = 6 bytes

So: `"end"` (4B) < `"text"` (5B) < `"start"` (6B). ✅

**CDDL:**
```cddl
timestamped-transcription = {
  "format" => "timestamped",
  "lang" => language-code,
  "segments" => [+ segment],
  * tstr => any
}

segment = {
  "start" => uint,
  "end" => uint,
  "text" => tstr
}
```

#### 3.4.3 Choosing a Format

| Scenario | Recommended Format | Rationale |
|----------|-------------------|-----------|
| Short video (< 2 min) without chapters | `"plain"` | Timestamps add overhead for short content |
| Video with chapters | `"timestamped"` | Timestamps align with chapter markers |
| Tutorial/educational video | `"timestamped"` | Enables seeking to specific topics |
| Podcast/interview | `"timestamped"` | Enables speaker-based navigation |
| Background/ambient video | `"plain"` or omit | Low informational content |

A publisher SHOULD provide timestamped segments when chapters are also present, as the combination enables precise content navigation.

#### 3.4.4 Transcription as Combined Type

The CDDL uses a choice type to enforce that plain transcriptions have `"text"` and timestamped ones have `"segments"`:

```cddl
transcription = plain-transcription / timestamped-transcription
```

A document with `"format": "timestamped"` but no `"segments"` key is **invalid**. A document with `"format": "plain"` but no `"text"` key is **invalid**. This is enforced by the CDDL schema — each variant is a distinct type with its own required fields.

### 3.5 Chapters

Chapters provide a high-level table of contents for the video. Each chapter has a timestamp (in seconds) and a title.

```cbor-diag
"chapters": [
  {"timestamp": 0,   "title": "Introduction"},
  {"timestamp": 12,  "title": "Posologie"},
  {"timestamp": 58,  "title": "Combinaisons"},
  {"timestamp": 140, "title": "Conclusion"}
]
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"timestamp"` | uint | REQUIRED | Chapter start time in seconds |
| `"title"` | text | REQUIRED | Chapter title |

**Key order**: `"title"` (6B) < `"timestamp"` (10B).

**Rules:**
- The first chapter SHOULD have `"timestamp": 0`
- Chapters MUST be ordered by timestamp (ascending)
- Chapter timestamps SHOULD correspond to segment boundaries in the transcription

An agent reading chapters gets a structured overview of the video content without reading the full transcription:

```
Video: "Comment prendre le Lion's Mane — Guide complet" (3:07)
  [0:00] Introduction
  [0:12] Posologie
  [0:58] Combinaisons
  [2:20] Conclusion
```

### 3.6 Example: Tutorial Video (Complete)

```cbor-diag
{
  "t": "video",
  "trust": 0,
  "src": "https://verdetao.com/videos/lions-mane-guide.mp4",
  "title": "Guide complet : Comment prendre le Lion's Mane",
  "codec": "h264",
  "file_size": 24500000,
  "duration_seconds": 187,
  "resolution": {"h": 1080, "w": 1920},
  "thumbnail_url": "https://verdetao.com/videos/thumbs/lm-guide.webp",
  "transcription": {
    "format": "timestamped",
    "lang": "fr",
    "segments": [
      {"end": 12,  "start": 0,  "text": "Bonjour, dans cette video nous allons voir comment prendre correctement le Lion's Mane."},
      {"end": 35,  "start": 12, "text": "La posologie recommandee est de deux capsules par jour."},
      {"end": 58,  "start": 35, "text": "Vous pouvez le prendre a jeun ou pendant le petit-dejeuner."}
    ]
  },
  "chapters": [
    {"timestamp": 0,  "title": "Introduction"},
    {"timestamp": 12, "title": "Posologie"},
    {"timestamp": 35, "title": "Moment de prise"}
  ]
}
```

### 3.7 Example: YouTube Embed (Minimal)

For a YouTube video where the publisher does not have the raw file:

```cbor-diag
{
  "t": "video",
  "trust": 0,
  "src": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
  "title": "Product Demo — Functional Mushroom Supplements",
  "duration_seconds": 240,
  "thumbnail_url": "https://img.youtube.com/vi/dQw4w9WgXcQ/maxresdefault.jpg"
}
```

Without a transcription, the video is opaque to the agent. The agent knows the title, duration, and has a thumbnail — but cannot extract the spoken content. Publishers SHOULD provide transcriptions whenever possible.

---

## 4. Audio Block

### 4.1 Overview

**Type code:** `"audio"` | **Trust level:** 0 (declarative)

The audio block extends the video block's approach to audio-only content: podcasts, radio recordings, audiobooks, music (metadata only). The key addition is **diarization** — attributing speech segments to identified speakers.

### 4.2 Structure

```cbor-diag
{
  "t": "audio",
  "trust": 0,
  "src": "https://example.com/podcast/ep42.mp3",
  "title": "Episode 42 — Les champignons fonctionnels",
  "format": "mp3",
  "file_size": 37400000,
  "duration_seconds": 2340,
  "transcription": {
    "format": "timestamped",
    "lang": "fr",
    "segments": [
      {"end": 15,  "start": 0,  "text": "Bienvenue dans notre podcast sur la sante naturelle."},
      {"end": 45,  "start": 15, "text": "Aujourd'hui nous recevons le Dr. Laurent, specialiste des champignons fonctionnels."},
      {"end": 120, "start": 45, "text": "Merci Marie. Les champignons fonctionnels sont utilises depuis des millenaires en medecine traditionnelle chinoise."}
    ]
  },
  "speakers": [
    {"id": "host", "name": "Marie", "role": "host"},
    {"id": "guest", "name": "Dr. Laurent", "role": "guest"}
  ],
  "diarization": [
    {"end": 45,  "speaker": "host",  "start": 0,  "text": "Bienvenue dans notre podcast... Aujourd'hui nous recevons le Dr. Laurent..."},
    {"end": 120, "speaker": "guest", "start": 45, "text": "Merci Marie. Les champignons fonctionnels sont utilises depuis des millenaires..."}
  ]
}
```

### 4.3 Field Reference

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"audio"` | Block type |
| `"trust"` | uint | REQUIRED | `0` | Trust level |
| `"src"` | text | REQUIRED | `https://` URL | Audio file URL |
| `"title"` | text | REQUIRED | Max 300 characters | Audio title |
| `"duration_seconds"` | uint | REQUIRED | Positive integer | Duration in seconds |
| `"format"` | text | RECOMMENDED | | Audio format: `"mp3"`, `"ogg"`, `"aac"`, `"flac"`, `"wav"` |
| `"file_size"` | uint | OPTIONAL | In bytes | File size |
| `"transcription"` | map | RECOMMENDED | Same as video (§3.4) | Full transcription |
| `"speakers"` | array | OPTIONAL | | Speaker identification |
| `"diarization"` | array | OPTIONAL | Requires `"speakers"` | Speaker-attributed segments |

### 4.4 Speakers

The `"speakers"` array identifies the people speaking in the audio:

```cbor-diag
"speakers": [
  {"id": "host", "name": "Marie Dupont", "role": "host"},
  {"id": "guest1", "name": "Dr. Laurent", "role": "guest"},
  {"id": "guest2", "name": "Prof. Chen", "role": "guest"}
]
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"id"` | text | REQUIRED | Unique identifier for this speaker (used in diarization) |
| `"name"` | text | REQUIRED | Speaker name |
| `"role"` | text | OPTIONAL | Role: `"host"`, `"guest"`, `"narrator"`, `"interviewer"`, `"interviewee"` |

### 4.5 Diarization

Diarization attributes each speech segment to a specific speaker. This is the audio equivalent of a screenplay — who says what, when.

```cbor-diag
"diarization": [
  {"end": 15,  "speaker": "host",  "start": 0,  "text": "Bienvenue dans notre podcast."},
  {"end": 45,  "speaker": "host",  "start": 15, "text": "Aujourd'hui nous recevons le Dr. Laurent."},
  {"end": 120, "speaker": "guest", "start": 45, "text": "Merci Marie. Les champignons fonctionnels..."}
]
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"start"` | uint | REQUIRED | Segment start time in seconds |
| `"end"` | uint | REQUIRED | Segment end time in seconds |
| `"speaker"` | text | REQUIRED | Speaker ID (MUST match an `"id"` in the `"speakers"` array) |
| `"text"` | text | REQUIRED | What the speaker said in this segment |

**Relationship between transcription and diarization:**

A publisher MAY provide both `"transcription"` and `"diarization"`. They cover the same content but with different granularity:

- `"transcription"` — pure text with timing, no speaker attribution
- `"diarization"` — text with timing AND speaker attribution

An agent that needs only the text reads `"transcription"`. An agent that needs to know who said what reads `"diarization"`.

If both are present, `"diarization"` is the richer data source. An agent SHOULD NOT need to read both.

### 4.6 Example: Podcast Episode

```cbor-diag
{
  "t": "audio",
  "trust": 0,
  "src": "https://verdetao.com/podcast/ep42-champignons.mp3",
  "title": "Episode 42 — Les champignons fonctionnels",
  "format": "mp3",
  "file_size": 37400000,
  "duration_seconds": 2340,
  "speakers": [
    {"id": "host", "name": "Marie", "role": "host"},
    {"id": "guest", "name": "Dr. Laurent", "role": "guest"}
  ],
  "diarization": [
    {"end": 15,  "speaker": "host",  "start": 0,  "text": "Bienvenue dans notre podcast sur la sante naturelle. Je suis Marie."},
    {"end": 45,  "speaker": "host",  "start": 15, "text": "Aujourd'hui nous recevons le Dr. Laurent, specialiste des champignons medicinaux."},
    {"end": 120, "speaker": "guest", "start": 45, "text": "Merci Marie. Les champignons fonctionnels sont utilises depuis des millenaires en medecine traditionnelle chinoise. Le Lion's Mane, par exemple, est documente dans la pharmacopee chinoise depuis le 3eme siecle."}
  ]
}
```

---

## 5. Document Block

### 5.1 Overview

**Type code:** `"document"` | **Trust level:** 0 (declarative)

For embedded documents (PDF, DOCX, spreadsheets, presentations) referenced within a page. The key insight: a PDF is opaque binary data to an agent unless the publisher extracts its text content and structure.

### 5.2 Structure

```cbor-diag
{
  "t": "document",
  "trust": 0,
  "src": "https://verdetao.com/docs/certificate-bio-eu.pdf",
  "title": "Certificat Bio EU — Verdetao",
  "lang": "fr",
  "mime_type": "application/pdf",
  "file_size": 245000,
  "page_count": 3,
  "text_extract": "CERTIFICAT DE CONFORMITE BIO\n\nOrganisme certificateur: Ecocert SA\nNumero: FR-BIO-01\nDate de delivrance: 2025-06-15\nDate d'expiration: 2026-12-31\n\nOperateur certifie:\n  Nom: Verdetao SL\n  Adresse: Calle Principal 42, Malaga, Espagne\n\nProduits certifies:\n  - Lion's Mane (Hericium erinaceus) — capsules\n  - Reishi (Ganoderma lucidum) — capsules\n  - Cordyceps (Cordyceps militaris) — capsules\n\nLe present certificat atteste que les produits ci-dessus sont conformes au reglement (UE) 2018/848 relatif a la production biologique.",
  "table_of_contents": [
    {"page": 1, "title": "Informations generales"},
    {"page": 2, "title": "Liste des produits certifies"},
    {"page": 3, "title": "Conditions de validite"}
  ]
}
```

### 5.3 Field Reference

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"document"` | Block type |
| `"trust"` | uint | REQUIRED | `0` | Trust level |
| `"src"` | text | REQUIRED | `https://` URL | Document download URL |
| `"title"` | text | REQUIRED | Max 300 characters | Document title |
| `"mime_type"` | text | REQUIRED | Standard MIME type | Document format. Examples: `"application/pdf"`, `"application/vnd.openxmlformats-officedocument.wordprocessingml.document"` (DOCX), `"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"` (XLSX), `"application/vnd.openxmlformats-officedocument.presentationml.presentation"` (PPTX) |
| `"file_size"` | uint | RECOMMENDED | In bytes | File size |
| `"page_count"` | uint | OPTIONAL | | Number of pages |
| `"text_extract"` | text | RECOMMENDED | Max 100 KB | Plain text extraction of the document content. This is how the agent reads the document without downloading the binary. |
| `"table_of_contents"` | array | OPTIONAL | `[{"page": uint, "title": text}]` | Document sections with page numbers |
| `"lang"` | text | OPTIONAL | ISO 639-1 | Document language |

### 5.4 Text Extract Guidance

The `"text_extract"` field is the **primary way an agent reads a referenced document**. Without it, the document is a black box — the agent knows it exists but cannot read its content.

**Publisher guidance for generating text extracts:**

| Document Type | Extraction Approach |
|---------------|-------------------|
| PDF (text-based) | Direct text extraction via `pdftotext` or equivalent |
| PDF (scanned) | OCR via Tesseract or cloud OCR service |
| DOCX | Extract via `python-docx`, `pandoc`, or LibreOffice |
| XLSX | Extract cell values, preserve table structure as text |
| PPTX | Extract slide text in order, one slide per paragraph |

The text extract SHOULD:
- Preserve paragraph structure (separate paragraphs with newlines)
- Preserve headings (prefix with appropriate markers)
- Preserve lists and tables in a readable text format
- NOT include headers, footers, page numbers, or watermarks
- NOT exceed 100 KB (for very long documents, extract the first 50 pages and note truncation)

---

## 6. Diagram Block

### 6.1 Overview

**Type code:** `"diagram"` | **Trust level:** 0 (declarative)

An agent doesn't render SVG or interpret raster images of diagrams. It needs to understand what the diagram **shows** — the entities, relationships, flow, and structure — in a machine-readable format.

The diagram block provides a **textual description** of the diagram's content alongside structured data (entities and relationships) that an agent can process programmatically.

### 6.2 Structure

```cbor-diag
{
  "t": "diagram",
  "trust": 0,
  "src": "https://verdetao.com/diagrams/supply-chain.svg",
  "format": "svg",
  "description": "Supply chain diagram showing the journey from farm to customer. Raw materials are grown on organic farms in Peru, then shipped to a processing plant in Valencia, Spain where they are extracted and concentrated. The extract is sent to a packaging facility in Marseille, France where it is encapsulated and labeled. Finally, packaged products are distributed to customers across the EU via a distribution center in Madrid.",
  "diagram_type": "flowchart",
  "entities": ["Organic Farm (Peru)", "Processing Plant (Valencia)", "Packaging Facility (Marseille)", "Distribution Center (Madrid)", "EU Customers"],
  "relationships": [
    {"from": "Organic Farm (Peru)", "label": "raw mushroom material", "to": "Processing Plant (Valencia)"},
    {"from": "Processing Plant (Valencia)", "label": "concentrated extract", "to": "Packaging Facility (Marseille)"},
    {"from": "Packaging Facility (Marseille)", "label": "finished product", "to": "Distribution Center (Madrid)"},
    {"from": "Distribution Center (Madrid)", "label": "retail distribution", "to": "EU Customers"}
  ]
}
```

### 6.3 Field Reference

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"diagram"` | Block type |
| `"trust"` | uint | REQUIRED | `0` | Trust level |
| `"src"` | text | OPTIONAL | `https://` URL | URL of the visual diagram file (SVG, PNG, Mermaid source) |
| `"format"` | text | OPTIONAL | | Source format: `"svg"`, `"png"`, `"mermaid"`, `"plantuml"`, `"drawio"` |
| `"description"` | text | REQUIRED | Max 5000 characters | **Full textual description** of what the diagram shows. This is the most important field — it must be detailed enough that an agent can understand the diagram without seeing it. |
| `"diagram_type"` | text | OPTIONAL | See §6.4 | Type classification |
| `"entities"` | array of text | OPTIONAL | | Named entities (nodes) in the diagram |
| `"relationships"` | array | OPTIONAL | `[{"from": text, "to": text, "label": text}]` | Directed relationships (edges) between entities |

### 6.4 Diagram Types

| Type | Description | Typical Entities | Typical Relationships |
|------|-------------|-----------------|----------------------|
| `"flowchart"` | Process flow, decision tree | Steps, decisions | Sequence, branches |
| `"sequence"` | Interaction sequence (UML sequence diagram) | Actors, systems | Messages, responses |
| `"entity_relationship"` | Database schema, data model | Tables, entities | Foreign keys, associations |
| `"architecture"` | System architecture, infrastructure | Services, components | Connections, data flows |
| `"timeline"` | Chronological events | Dates, events | Sequence |
| `"mindmap"` | Hierarchical idea map | Concepts, topics | Parent-child |
| `"organizational"` | Org chart, team structure | People, roles, departments | Reports-to |

### 6.5 Description Quality

The `"description"` field is the **single most important field** in a diagram block. An agent will never see the visual diagram — it reads this description. The description MUST:

1. **State the overall purpose** of the diagram in the first sentence
2. **Name all entities** and explain their roles
3. **Describe all relationships** and their direction/meaning
4. **Explain the flow** or logic (for flowcharts and sequences)
5. **Use concrete details** — numbers, names, locations — not abstract descriptions

**Good description example:**
> "Supply chain diagram showing the journey from farm to customer. Raw materials are grown on organic farms in Peru, then shipped to a processing plant in Valencia, Spain where they are extracted and concentrated. The extract is sent to a packaging facility in Marseille, France where it is encapsulated and labeled. Finally, packaged products are distributed to customers across the EU via a distribution center in Madrid."

**Bad description example:**
> "Supply chain diagram."

The good description contains all the information an agent needs. The bad description is useless — it says nothing about the content.

### 6.6 Entities and Relationships

The `"entities"` and `"relationships"` arrays provide **structured data** about the diagram, enabling programmatic processing. An agent can:

- Build a graph data structure from entities + relationships
- Query the graph (e.g., "what comes after the Processing Plant?")
- Compare diagrams across pages
- Generate code from the structure (e.g., database schema from an ER diagram)

Relationship key order: `"from"` (5B) < `"label"` (6B) < `"to"` (3B) — wait, `"to"` = 2 chars = `62 746F` = 3 bytes. So: `"to"` (3B) < `"from"` (5B) < `"label"` (6B).

```cbor-diag
{"to": "Processing Plant", "from": "Organic Farm", "label": "raw material"}
```

---

## 7. Live Stream Block

### 7.1 Overview

**Type code:** `"live_stream"` | **Trust level:** 0 (declarative)

For web radio, live video, or any continuous streaming source. Unlike video and audio blocks which reference recorded files, the live stream block describes an **ongoing stream**.

### 7.2 Structure

```cbor-diag
{
  "t": "live_stream",
  "trust": 0,
  "title": "Radio Fungi — 24/7 Ambient & Nature",
  "stream_url": "https://radio.example.com/stream",
  "stream_format": "icecast",
  "current_show": {
    "title": "Morning Meditation",
    "host": "DJ Mycelium",
    "started_at": 1(1742515200),
    "description": "Ambient sounds from mushroom forests worldwide"
  },
  "schedule": [
    {"host": "DJ Mycelium", "time": "06:00", "title": "Morning Meditation"},
    {"host": "Dr. Spore",   "time": "10:00", "title": "Fungi Facts"},
    {"host": "DJ Mycelium", "time": "14:00", "title": "Afternoon Mix"},
    {"host": "DJ Hyphae",   "time": "18:00", "title": "Evening Grooves"},
    {"host": "DJ Mycelium", "time": "22:00", "title": "Night Forest"}
  ]
}
```

### 7.3 Field Reference

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"live_stream"` | Block type |
| `"trust"` | uint | REQUIRED | `0` | Trust level |
| `"stream_url"` | text | REQUIRED | URL (may be `https://` or protocol-specific) | Stream URL |
| `"stream_format"` | text | REQUIRED | | Stream protocol/format: `"hls"`, `"dash"`, `"icecast"`, `"rtmp"`, `"webrtc"` |
| `"title"` | text | REQUIRED | | Stream name / station name |
| `"current_show"` | map | OPTIONAL | | What is currently playing |
| `"schedule"` | array | OPTIONAL | | Programming schedule |

**current_show fields:**

| Key | Type | Description |
|-----|------|-------------|
| `"title"` | text | Show name |
| `"host"` | text | Host/presenter name |
| `"started_at"` | tag 1 (uint) | When the current show started |
| `"description"` | text | Show description |

**schedule entry fields:**

| Key | Type | Description |
|-----|------|-------------|
| `"time"` | text | Show start time (HH:MM format, 24-hour, local time) |
| `"title"` | text | Show name |
| `"host"` | text | Host/presenter name |

### 7.4 Stream Formats

| Format | Protocol | Typical Use | Agent Notes |
|--------|----------|-------------|-------------|
| `"hls"` | HTTP Live Streaming (Apple) | Video/audio | `.m3u8` playlist URL |
| `"dash"` | MPEG-DASH | Video/audio | `.mpd` manifest URL |
| `"icecast"` | Icecast/Shoutcast | Web radio | Direct audio stream |
| `"rtmp"` | Real-Time Messaging Protocol | Live video | Flash-era, still used |
| `"webrtc"` | WebRTC | Low-latency video | Peer-to-peer |

An agent typically does NOT consume live streams directly — it reads the metadata (title, schedule, current show) to understand what content is available.

---

## 8. Real-Time Streaming Channels

### 8.1 Overview

Some content changes continuously — stock levels, live radio metadata, notifications. CBOR-Web defines a mechanism for an agent to subscribe to real-time content updates over WebSocket, receiving CBOR-encoded messages.

Channels are declared in the **manifest** (key 8), not in individual pages. An agent reads the channel list once from the manifest and decides which channels to subscribe to.

### 8.2 Channel Declaration

```cbor-diag
; In manifest key 8:
8: [
  {
    "channel_id": "stock_updates",
    "url": "wss://api.verdetao.com/ws/stock",
    "purpose": "Real-time product stock level updates",
    "protocol": "cbor-web-stream",
    "auth": {"type": "bearer", "description": "CBOR-Web token required"},
    "message_schema": {
      "product_id": "string",
      "stock": "integer",
      "timestamp": "integer"
    },
    "frequency": "on_change",
    "example_message": {
      "product_id": "lions-mane-90",
      "stock": 149,
      "timestamp": 1742515300
    }
  },
  {
    "channel_id": "radio_metadata",
    "url": "wss://radio.verdetao.com/ws/metadata",
    "purpose": "Radio stream now-playing metadata",
    "protocol": "cbor-web-stream",
    "frequency": "event_driven",
    "message_schema": {
      "artist": "string",
      "event": "string",
      "timestamp": "integer",
      "track_title": "string"
    },
    "example_message": {
      "artist": "Ambient Forest",
      "event": "track_change",
      "timestamp": 1742515400,
      "track_title": "Morning Dew on Hyphae"
    }
  }
]
```

### 8.3 Channel Fields

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"channel_id"` | text | REQUIRED | Unique within manifest | Channel identifier |
| `"url"` | text | REQUIRED | `wss://` URL | WebSocket endpoint |
| `"purpose"` | text | REQUIRED | | Human-readable description of what this channel provides |
| `"protocol"` | text | REQUIRED | `"cbor-web-stream"` | MUST be `"cbor-web-stream"` |
| `"auth"` | map | OPTIONAL | `{"type": text, "description": text}` | Authentication requirements |
| `"message_schema"` | map | REQUIRED | Keys: field names (text). Values: type names (text). | Schema of each message — tells the agent what fields to expect |
| `"frequency"` | text | REQUIRED | `"on_change"` / `"event_driven"` / `"periodic"` | How often messages are sent |
| `"periodic_interval_ms"` | uint | CONDITIONAL | REQUIRED if frequency is `"periodic"` | Milliseconds between messages |
| `"example_message"` | map | RECOMMENDED | | Example message for testing and documentation |

### 8.4 Frequency Types

| Frequency | Description | Example |
|-----------|-------------|---------|
| `"on_change"` | Message sent when underlying data changes | Stock level update when a sale occurs |
| `"event_driven"` | Message sent when a specific event occurs | New track playing on radio |
| `"periodic"` | Message sent at regular intervals | Temperature reading every 60 seconds |

### 8.5 Message Format

Each WebSocket message is a CBOR-encoded map. No self-described tag is needed for individual messages (the connection is already established as CBOR-Web-Stream). Messages SHOULD use deterministic encoding for consistency.

### 8.6 Agent Behavior

An agent connecting to a CBOR-Web-Stream channel:

1. MUST use secure WebSocket (`wss://`)
2. MUST handle authentication if required (see `"auth"` field)
3. MUST validate each message against the declared `"message_schema"`
4. SHOULD implement reconnection with exponential backoff (initial 1s, max 60s, jitter)
5. MUST NOT maintain more than **5 simultaneous channel connections** to a single domain
6. SHOULD disconnect channels it is not actively using (battery/bandwidth conservation)

---

## 9. Multimedia in the Content Array

### 9.1 Placement Rules

All multimedia blocks are placed in the page's content array (key 4) at the position where they appear in the original HTML. They coexist with core blocks:

```cbor-diag
4: [
  {"l": 1, "t": "h", "v": "Product Page"},                    ; core heading
  {"t": "image", "trust": 0, "alt": "...", "src": "...", ...}, ; multimedia image
  {"t": "p", "v": "Description text..."},                      ; core paragraph
  {"t": "video", "trust": 0, "src": "...", "title": "...", ...}, ; multimedia video
  {"t": "p", "v": "After watching the video..."},              ; core paragraph
  {"t": "diagram", "trust": 0, "description": "...", ...}      ; multimedia diagram
]
```

### 9.2 v1.0 Compatibility

A v1.0 agent encountering multimedia blocks in the content array will not recognize the type codes (`"image"`, `"video"`, `"audio"`, etc.) and will **skip them** per the forward-compatibility rule (CBOR-WEB-CORE.md §3.6). The core blocks remain fully accessible.

This means a page with multimedia blocks works for both v1.0 and v2.1 agents — the v1.0 agent gets the text content, the v2.1 agent gets text + multimedia.

### 9.3 Manifest Flags

A publisher SHOULD set `"has_multimedia": true` in the page entry for any page containing multimedia blocks. This allows agents to filter pages by capability without downloading them.

---

## 10. Agent Processing Strategies

### 10.1 Text-Only Agent

An agent that does not process multimedia can still consume a page with multimedia blocks:

1. Read key 4 (content array)
2. For each block, check `"t"`:
   - Core types (`"h"`, `"p"`, `"ul"`, etc.): process normally
   - Unknown types (`"image"`, `"video"`, etc.): skip
3. Result: full editorial text content, multimedia silently ignored

### 10.2 Multimedia-Aware Agent

An agent that processes multimedia:

1. Read key 4 (content array)
2. For each block:
   - Core types: process as text
   - `"image"`: read `"alt"` + `"ai_description"` for text understanding. Download image only if visual analysis is needed.
   - `"video"`: read `"transcription"` + `"chapters"` for text understanding. Never download the video.
   - `"audio"`: read `"transcription"` + `"diarization"` for text understanding. Never download the audio.
   - `"document"`: read `"text_extract"` for text understanding. Download PDF only if exact formatting matters.
   - `"diagram"`: read `"description"` + `"entities"` + `"relationships"` for structural understanding.
   - `"live_stream"`: read metadata (schedule, current show) for awareness.

### 10.3 Priority Processing

When an agent has limited time or tokens, it should process multimedia blocks in priority order:

| Priority | Block Type | Agent Action |
|----------|-----------|-------------|
| 1 (highest) | `"diagram"` with `"entities"` | Parse entities + relationships as structured data |
| 2 | `"video"` / `"audio"` with transcription | Read transcription text |
| 3 | `"document"` with `"text_extract"` | Read extracted text |
| 4 | `"image"` with `"ai_description"` | Read AI description |
| 5 | `"image"` with `"product_photo"` role | Note product visual exists |
| 6 (lowest) | `"live_stream"` | Note availability only |
| Skip | Any block with `"semantic_role": "decorative"` | No informational value |

---

## Appendix A: Multimedia CDDL Schema

```cddl
; ══════════════════════════════════════════════════════════
; CBOR-Web Multimedia Specification v2.1 — CDDL Schema
; Document: CBOR-WEB-MULTIMEDIA.md, Appendix A
; ══════════════════════════════════════════════════════════

; ── Multimedia Content Blocks ──
; These extend the content-block choice in CBOR-WEB-CORE.md Appendix A

multimedia-block = rich-image / video-block / audio-block /
                   document-block / diagram-block / live-stream-block

; ── Semantic Roles ──

semantic-role = "logo" / "product_photo" / "hero" / "illustration" /
                "screenshot" / "avatar" / "diagram" / "decorative" /
                "infographic" / "photo_editorial" / tstr

; ── Rich Image ──

rich-image = {
  "t" => "image",
  "trust" => 0,
  "src" => tstr,
  "alt" => tstr,
  "semantic_role" => semantic-role,
  ? "dimensions" => { "w" => uint, "h" => uint },
  ? "format" => tstr,
  ? "file_size" => uint,
  ? "dominant_color" => tstr,
  ? "ai_description" => tstr,
  ? "caption" => tstr,
  ? "exif" => { * tstr => any },
  ? "inline_data" => bstr / null,     ; raw image bytes, NOT base64
  * tstr => any
}

; ── Transcription (split into conditional types) ──

transcription = plain-transcription / timestamped-transcription

plain-transcription = {
  "format" => "plain",
  "lang" => language-code,
  "text" => tstr,                     ; REQUIRED for plain
  * tstr => any
}

timestamped-transcription = {
  "format" => "timestamped",
  "lang" => language-code,
  "segments" => [+ segment],         ; REQUIRED for timestamped
  * tstr => any
}

segment = {
  "start" => uint,
  "end" => uint,
  "text" => tstr
}

; language-code is defined in CBOR-WEB-CORE.md
; language-code = tstr .regexp "[a-z]{2}(-[A-Z]{2})?"

; ── Video ──

video-block = {
  "t" => "video",
  "trust" => 0,
  "src" => tstr,
  "title" => tstr,
  "duration_seconds" => uint,
  ? "resolution" => { "w" => uint, "h" => uint },
  ? "codec" => tstr,
  ? "file_size" => uint,
  ? "thumbnail_url" => tstr,
  ? "transcription" => transcription,
  ? "chapters" => [+ { "timestamp" => uint, "title" => tstr }],
  * tstr => any
}

; ── Audio ──

audio-block = {
  "t" => "audio",
  "trust" => 0,
  "src" => tstr,
  "title" => tstr,
  "duration_seconds" => uint,
  ? "format" => tstr,
  ? "file_size" => uint,
  ? "transcription" => transcription,
  ? "speakers" => [+ { "id" => tstr, "name" => tstr, ? "role" => tstr }],
  ? "diarization" => [+ { "start" => uint, "end" => uint, "speaker" => tstr, "text" => tstr }],
  * tstr => any
}

; ── Document ──

document-block = {
  "t" => "document",
  "trust" => 0,
  "src" => tstr,
  "title" => tstr,
  "mime_type" => tstr,
  ? "file_size" => uint,
  ? "page_count" => uint,
  ? "text_extract" => tstr,
  ? "table_of_contents" => [+ { "page" => uint, "title" => tstr }],
  ? "lang" => language-code,
  * tstr => any
}

; ── Diagram ──

diagram-block = {
  "t" => "diagram",
  "trust" => 0,
  ? "src" => tstr,
  ? "format" => tstr,
  "description" => tstr,
  ? "diagram_type" => diagram-type,
  ? "entities" => [+ tstr],
  ? "relationships" => [+ { "from" => tstr, "to" => tstr, ? "label" => tstr }],
  * tstr => any
}

diagram-type = "flowchart" / "sequence" / "entity_relationship" /
               "architecture" / "timeline" / "mindmap" / "organizational" / tstr

; ── Live Stream ──

live-stream-block = {
  "t" => "live_stream",
  "trust" => 0,
  "title" => tstr,
  "stream_url" => tstr,
  "stream_format" => "hls" / "dash" / "icecast" / "rtmp" / "webrtc" / tstr,
  ? "current_show" => {
    "title" => tstr,
    ? "host" => tstr,
    ? "started_at" => #6.1(uint),
    ? "description" => tstr,
    * tstr => any
  },
  ? "schedule" => [+ { "time" => tstr, "title" => tstr, ? "host" => tstr }],
  * tstr => any
}

; ── Streaming Channels (manifest key 8) ──

channel = {
  "channel_id" => tstr,
  "url" => tstr,
  "purpose" => tstr,
  "protocol" => "cbor-web-stream",
  ? "auth" => { "type" => tstr, ? "description" => tstr, * tstr => any },
  "message_schema" => { + tstr => tstr },
  "frequency" => "on_change" / "event_driven" / "periodic",
  ? "periodic_interval_ms" => uint,
  ? "example_message" => { + tstr => any },
  * tstr => any
}
```

---

## Appendix B: Multimedia Test Vectors

Binary test vectors for multimedia blocks are available in the repository at `test-vectors/`. These were generated using the same Rust (ciborium 0.2.2) and Python (cbor2) cross-validation process as the core test vectors (CBOR-WEB-CORE.md Appendix B).

### B.1 Test Vector — Rich Image Block

```cbor-diag
{
  "t": "image",
  "trust": 0,
  "alt": "Flacon Lion's Mane",
  "src": "https://verdetao.com/img/lm.webp",
  "format": "webp",
  "file_size": 84200,
  "dimensions": {"h": 800, "w": 1200},
  "semantic_role": "product_photo"
}
```

### B.2 Test Vector — Video with Timestamped Transcription

```cbor-diag
{
  "t": "video",
  "trust": 0,
  "src": "https://verdetao.com/videos/guide.mp4",
  "title": "Guide Lion's Mane",
  "duration_seconds": 187,
  "transcription": {
    "format": "timestamped",
    "lang": "fr",
    "segments": [
      {"end": 12, "start": 0, "text": "Bonjour."},
      {"end": 35, "start": 12, "text": "La posologie recommandee est de deux capsules."}
    ]
  },
  "chapters": [
    {"timestamp": 0, "title": "Introduction"},
    {"timestamp": 12, "title": "Posologie"}
  ]
}
```

---

## Appendix C: Multimedia Examples

### C.1 Complete Product Page with Multimedia

A page combining core blocks with multimedia — the kind of page a real e-commerce site produces:

```cbor-diag
55799({
  0: "cbor-web-page",
  1: 2,
  2: {
    "lang": "fr",
    "path": "/products/lions-mane",
    "canonical": "https://verdetao.com/products/lions-mane"
  },
  3: {
    "title": "Lion's Mane — Champignon Fonctionnel Premium",
    "updated": 1(1742515200),
    "category": "products"
  },
  4: [
    {"l": 1, "t": "h", "v": "Lion's Mane — Criniere de Lion"},
    {"t": "image", "trust": 0,
      "alt": "Flacon de Lion's Mane Verdetao",
      "src": "https://verdetao.com/img/lm-front.webp",
      "format": "webp",
      "file_size": 84200,
      "dimensions": {"h": 800, "w": 1200},
      "semantic_role": "product_photo",
      "ai_description": "Green glass bottle, white cap, labeled Lion's Mane, 90 capsules, EU organic logo."
    },
    {"t": "p", "v": "Notre extrait de Hericium erinaceus est concentre 10:1."},
    {"t": "video", "trust": 0,
      "src": "https://verdetao.com/videos/lm-guide.mp4",
      "title": "Comment prendre le Lion's Mane",
      "duration_seconds": 187,
      "thumbnail_url": "https://verdetao.com/videos/thumbs/lm-guide.webp",
      "transcription": {
        "format": "plain",
        "lang": "fr",
        "text": "Bonjour. La posologie recommandee est de deux capsules par jour."
      }
    },
    {"t": "table",
      "rows": [["Concentration", "10:1"], ["Capsules", "90"], ["Prix", "29.90 EUR"]],
      "headers": ["Propriete", "Valeur"]
    },
    {"t": "document", "trust": 0,
      "src": "https://verdetao.com/docs/cert-bio.pdf",
      "title": "Certificat Bio EU",
      "mime_type": "application/pdf",
      "file_size": 245000,
      "page_count": 3,
      "text_extract": "CERTIFICAT DE CONFORMITE BIO\nOperateur: Verdetao SL\nProduits: Lion's Mane, Reishi, Cordyceps"
    },
    {"t": "cta", "href": "/cart/add/lions-mane", "v": "Ajouter au panier"}
  ]
})
```

This page has: 1 heading, 1 product photo, 1 paragraph, 1 video with transcription, 1 data table, 1 document reference, and 1 CTA. An agent reading this page gets complete product information from both text and multimedia metadata — without downloading a single binary file.

---

## References

### Normative References

- **[RFC 8949]** Bormann, C. and P. Hoffman, "Concise Binary Object Representation (CBOR)", STD 94, December 2020.
- **[RFC 8610]** Birkholz, H., et al., "Concise Data Definition Language (CDDL)", June 2019.

### Informative References

- **[CBOR-WEB-CORE.md]** CBOR-Web Core Specification v2.1 — prerequisite for this document.
- **[CBOR-WEB-SECURITY.md]** CBOR-Web Security Specification v2.1 — URL validation rules for `"src"` fields.
- **[CBOR-WEB-REFERENCE.md]** CBOR-Web Reference v2.1 — unified CDDL, all test vectors.

---

*CBOR-Web Multimedia Specification v2.1 — Document 2 of 6*

*ExploDev 2026*
