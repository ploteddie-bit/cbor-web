# CBOR-Web Multimedia Specification v3.0

**Companion to:** CBOR-Web Core Specification v3.0 (CBOR-WEB-SPEC-v3.0.md)
**Status:** Draft
**Date:** 2026-03-24
**Authors:** Eddie Plot & Claude — Deltopide

---

## 1. Overview

This document defines multimedia content blocks for CBOR-Web: rich images, video, audio, documents, diagrams, and live streams. All multimedia blocks are **trust level 0** (declarative, no side effects).

Multimedia blocks extend the core content block system (CBOR-WEB-SPEC-v3.0.md §8) with richer metadata while maintaining the same principles: explicit types, minimal size, zero ambiguity.

**Design principle:** Multimedia content is **referenced by URL**, not embedded. The exception is inline images below 10 KB (§2.2). CBOR-Web is a content format, not a media container.

---

## 2. Image Block

### 2.1 Rich Image (`"image"`)

The rich image block extends the core `"img"` block (CBOR-WEB-SPEC-v3.0.md §8.9) with semantic role, dimensions, AI description, and optional inline data.

**Type code:** `"image"` | **Category:** Non-editorial | **Trust level:** 0

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"image"` | Block type |
| `"alt"` | text | REQUIRED | Max 500 chars | Accessibility text |
| `"src"` | text | REQUIRED | `https://` URL | Image URL (full size) |
| `"role"` | text | OPTIONAL | See §2.3 | Semantic role of the image |
| `"width"` | uint | OPTIONAL | Pixels | Natural width |
| `"height"` | uint | OPTIONAL | Pixels | Natural height |
| `"format"` | text | OPTIONAL | MIME subtype | Image format: `"webp"`, `"png"`, `"jpg"`, `"svg"`, `"avif"` |
| `"size"` | uint | OPTIONAL | Bytes | File size of the referenced image |
| `"caption"` | text | OPTIONAL | | Caption text |
| `"ai_description"` | text | OPTIONAL | Max 2000 chars | Detailed AI-generated description for agents that cannot process images |
| `"srcset"` | array of maps | OPTIONAL | | Responsive image sources (like HTML srcset) |
| `"inline_data"` | bstr | OPTIONAL | Max 10 KB | Inline image data (see §2.2) |
| `"inline_format"` | text | CONDITIONAL | Required if `"inline_data"` present | MIME subtype of inline data |

**CBOR diagnostic:**

```cbor-diag
{
  "t": "image",
  "alt": "Flacon Lion's Mane, 90 capsules vegetales",
  "src": "https://verdetao.com/img/lions-mane-bottle.webp",
  "role": "product",
  "size": 45200,
  "width": 800,
  "format": "webp",
  "height": 1200,
  "caption": "Lion's Mane — Extrait 10:1, certifie bio EU",
  "ai_description": "Photo produit sur fond blanc. Flacon ambré 60ml avec étiquette verte et dorée. Texte: Verdetao Lion's Mane 500mg, 90 capsules. Logo certification bio EU visible en bas à droite."
}
```

### 2.2 Inline Images

For small images (icons, logos, thumbnails) below **10 KB**, a publisher MAY embed the image data directly in the CBOR document using the `"inline_data"` key.

```cbor-diag
{
  "t": "image",
  "alt": "Logo Verdetao",
  "src": "https://verdetao.com/logo.svg",
  "role": "logo",
  "inline_data": h'3C73766720786D6C6E733D...',
  "inline_format": "svg"
}
```

**Rules:**
- `"inline_data"` MUST be a CBOR byte string (major type 2) containing raw image bytes
- `"inline_data"` MUST NOT exceed 10,240 bytes (10 KB)
- `"inline_format"` MUST be present when `"inline_data"` is present
- `"src"` MUST still be present as a fallback URL
- An agent MAY use `"inline_data"` directly or fetch from `"src"` — both are valid
- For data exceeding 100 KB, indefinite-length byte strings MAY be used (CBOR-WEB-SPEC-v3.0.md §3.8)

### 2.3 Image Semantic Roles

| Role | Meaning | Agent Behavior |
|------|---------|---------------|
| `"hero"` | Main page visual (banner, header image) | Primary visual representation of the page |
| `"product"` | Product photo | Use for product cards, comparison, commerce |
| `"logo"` | Brand or site logo | Use for attribution, branding |
| `"icon"` | UI icon or small illustration | Low priority, skip for text-only agents |
| `"photo"` | Editorial photograph | Contextual illustration |
| `"screenshot"` | Application or website screenshot | Technical documentation |
| `"chart"` | Data visualization (chart, graph) | Consider `"ai_description"` for data extraction |
| `"diagram"` | Technical diagram | Consider the `"diagram"` block type (§6) instead |
| `"avatar"` | Person or entity avatar | Use for attribution |
| `"decorative"` | Purely visual, no content value | Agent SHOULD skip entirely |

An agent MAY filter images by role. For example, a text-only agent can skip all images with `"role": "decorative"` or `"role": "icon"`.

### 2.4 Responsive Sources (srcset)

```cbor-diag
"srcset": [
  {"src": "https://verdetao.com/img/lm-400.webp", "width": 400},
  {"src": "https://verdetao.com/img/lm-800.webp", "width": 800},
  {"src": "https://verdetao.com/img/lm-1200.webp", "width": 1200}
]
```

An agent that needs to download the image (e.g., for analysis or embedding) SHOULD choose the smallest resolution sufficient for its purpose. A text-only agent SHOULD NOT download any image.

---

## 3. Video Block

**Type code:** `"video"` | **Category:** Non-editorial | **Trust level:** 0

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"video"` | Block type |
| `"src"` | text | REQUIRED | `https://` URL | Video URL or embed URL |
| `"title"` | text | REQUIRED | Max 300 chars | Video title |
| `"duration"` | uint | OPTIONAL | Seconds | Video duration |
| `"format"` | text | OPTIONAL | | Video format: `"mp4"`, `"webm"`, `"youtube"`, `"vimeo"` |
| `"thumbnail"` | text | OPTIONAL | `https://` URL | Thumbnail image URL |
| `"description"` | text | OPTIONAL | Max 2000 chars | Video description |
| `"transcript"` | text | OPTIONAL | | Full transcript text (plain text, with timestamps optional) |
| `"chapters"` | array of maps | OPTIONAL | | Video chapters with timestamps |
| `"lang"` | text | OPTIONAL | BCP 47 | Language of the video |
| `"captions"` | array of maps | OPTIONAL | | Available caption tracks |
| `"size"` | uint | OPTIONAL | Bytes | File size |

**CBOR diagnostic:**

```cbor-diag
{
  "t": "video",
  "src": "https://youtube.com/watch?v=abc123",
  "lang": "fr",
  "title": "Comment utiliser le Lion's Mane",
  "format": "youtube",
  "duration": 342,
  "chapters": [
    {"time": 0, "title": "Introduction"},
    {"time": 45, "title": "Dosage recommandé"},
    {"time": 180, "title": "Effets et bienfaits"},
    {"time": 290, "title": "Précautions"}
  ],
  "thumbnail": "https://verdetao.com/img/video-lm-thumb.webp",
  "transcript": "Bonjour, aujourd'hui nous allons parler du Lion's Mane...",
  "description": "Guide complet d'utilisation du champignon Lion's Mane (Hericium erinaceus)"
}
```

**Agent behavior:**
- A text-only agent reads `"title"`, `"description"`, and `"transcript"` — the full informational content without downloading the video
- A multimedia agent MAY use `"chapters"` to jump to specific sections
- `"transcript"` is the **key field** for AI agents — it makes video content text-accessible

### 3.1 Chapters

```cbor-diag
"chapters": [
  {"time": 0, "title": "Introduction"},
  {"time": 120, "title": "Step 1: Preparation"},
  {"time": 240, "title": "Step 2: Assembly"}
]
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"time"` | uint | REQUIRED | Start time in seconds from video beginning |
| `"title"` | text | REQUIRED | Chapter title |

### 3.2 Captions

```cbor-diag
"captions": [
  {"lang": "fr", "src": "https://verdetao.com/captions/lm-fr.vtt"},
  {"lang": "en", "src": "https://verdetao.com/captions/lm-en.vtt"}
]
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `"lang"` | text | REQUIRED | BCP 47 language code |
| `"src"` | text | REQUIRED | URL to caption file (WebVTT or SRT) |

---

## 4. Audio Block

**Type code:** `"audio"` | **Category:** Non-editorial | **Trust level:** 0

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"audio"` | Block type |
| `"src"` | text | REQUIRED | `https://` URL | Audio file URL |
| `"title"` | text | REQUIRED | Max 300 chars | Audio title |
| `"duration"` | uint | OPTIONAL | Seconds | Duration |
| `"format"` | text | OPTIONAL | | Format: `"mp3"`, `"ogg"`, `"aac"`, `"wav"`, `"flac"` |
| `"transcript"` | text | OPTIONAL | | Full transcript (key field for AI agents) |
| `"chapters"` | array of maps | OPTIONAL | | Same format as video chapters (§3.1) |
| `"lang"` | text | OPTIONAL | BCP 47 | Language |
| `"size"` | uint | OPTIONAL | Bytes | File size |
| `"episode"` | map | OPTIONAL | | Podcast episode metadata |

**CBOR diagnostic (podcast episode):**

```cbor-diag
{
  "t": "audio",
  "src": "https://verdetao.com/podcast/ep12.mp3",
  "lang": "fr",
  "size": 24500000,
  "title": "Ep.12 — Les nootropiques naturels",
  "format": "mp3",
  "episode": {
    "number": 12,
    "season": 1,
    "series": "Verdetao Podcast"
  },
  "duration": 1830,
  "transcript": "Bienvenue dans l'épisode 12 de Verdetao Podcast..."
}
```

---

## 5. Document Block

**Type code:** `"document"` | **Category:** Non-editorial | **Trust level:** 0

References an embedded document (PDF, spreadsheet, presentation).

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"document"` | Block type |
| `"src"` | text | REQUIRED | `https://` URL | Document URL |
| `"title"` | text | REQUIRED | Max 300 chars | Document title |
| `"format"` | text | REQUIRED | | Format: `"pdf"`, `"xlsx"`, `"docx"`, `"pptx"`, `"csv"` |
| `"size"` | uint | OPTIONAL | Bytes | File size |
| `"pages"` | uint | OPTIONAL | | Number of pages (for PDF, PPTX) |
| `"summary"` | text | OPTIONAL | Max 2000 chars | Document summary for agents that cannot process the format |
| `"extracted_text"` | text | OPTIONAL | | Full extracted text content |
| `"lang"` | text | OPTIONAL | BCP 47 | Language |

**CBOR diagnostic:**

```cbor-diag
{
  "t": "document",
  "src": "https://verdetao.com/docs/certificat-bio-eu.pdf",
  "size": 234500,
  "lang": "fr",
  "pages": 3,
  "title": "Certificat Bio EU — Verdetao",
  "format": "pdf",
  "summary": "Certificat de conformité biologique européen délivré par Ecocert pour les produits Verdetao. Valide jusqu'au 31/12/2027."
}
```

**Agent behavior:** A text-only agent reads `"title"`, `"summary"`, and `"extracted_text"`. It does NOT need to download the PDF.

---

## 6. Diagram Block

**Type code:** `"diagram"` | **Category:** Non-editorial | **Trust level:** 0

A technical diagram with machine-readable source code (Mermaid, PlantUML, DOT) and optional rendered image.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"diagram"` | Block type |
| `"title"` | text | REQUIRED | Max 300 chars | Diagram title |
| `"syntax"` | text | REQUIRED | | Diagram language: `"mermaid"`, `"plantuml"`, `"dot"`, `"d2"` |
| `"source"` | text | REQUIRED | | Diagram source code |
| `"rendered"` | text | OPTIONAL | `https://` URL | URL to rendered image (PNG/SVG) |
| `"alt"` | text | OPTIONAL | | Text description of the diagram |

**CBOR diagnostic:**

```cbor-diag
{
  "t": "diagram",
  "alt": "Architecture du pipeline CBOR-Web: Publisher génère CBOR, Agent consomme via index.cbor",
  "title": "CBOR-Web Pipeline",
  "source": "graph LR\n  A[HTML Site] --> B[text2cbor]\n  B --> C[index.cbor]\n  B --> D[CBOR Pages]\n  C --> E[AI Agent]\n  D --> E",
  "syntax": "mermaid",
  "rendered": "https://example.com/diagrams/pipeline.svg"
}
```

**Agent behavior:** An AI agent CAN interpret the Mermaid/PlantUML source directly (these are text-based). The `"rendered"` URL is for display purposes. The `"alt"` provides a plain-text fallback.

---

## 7. Live Stream Block

**Type code:** `"live_stream"` | **Category:** Non-editorial | **Trust level:** 0

References a real-time or near-real-time data stream. This block enables CBOR-Web to point to dynamic content that changes faster than the static document model can capture.

| Key | Type | Required | Constraints | Description |
|-----|------|----------|-------------|-------------|
| `"t"` | text | REQUIRED | `"live_stream"` | Block type |
| `"src"` | text | REQUIRED | `wss://` or `https://` URL | Stream endpoint |
| `"title"` | text | REQUIRED | Max 300 chars | Stream title |
| `"protocol"` | text | REQUIRED | | Protocol: `"websocket"`, `"sse"`, `"hls"`, `"dash"` |
| `"format"` | text | OPTIONAL | | Content format: `"cbor"`, `"json"`, `"text"`, `"video"` |
| `"description"` | text | OPTIONAL | | Stream description |
| `"auth_required"` | text | OPTIONAL | `"T0"`, `"T1"`, `"T2"` | Access tier required to connect |
| `"snapshot"` | map | OPTIONAL | | Last known state (for agents that cannot connect to streams) |

**CBOR diagnostic:**

```cbor-diag
{
  "t": "live_stream",
  "src": "wss://verdetao.com/stream/stock",
  "title": "Stock en temps réel",
  "format": "json",
  "protocol": "websocket",
  "description": "Niveaux de stock actualisés toutes les 30 secondes",
  "auth_required": "T1",
  "snapshot": {
    "lions_mane": {"stock": 142, "updated": 1(1742598400)},
    "reishi": {"stock": 89, "updated": 1(1742598400)}
  }
}
```

**Agent behavior:** An agent that cannot connect to WebSocket reads the `"snapshot"` for the last known state. An agent capable of real-time processing MAY connect to the stream for live updates.

---

## 8. Channels (index.cbor Key 8)

The index.cbor key 8 declares site-level streaming channels. This is different from per-page `"live_stream"` blocks — channels are site-wide real-time feeds.

```cbor-diag
8: [
  {
    "id": "stock-updates",
    "url": "wss://verdetao.com/channels/stock",
    "format": "cbor",
    "protocol": "websocket",
    "description": "Real-time stock level updates for all products",
    "auth_required": "T1",
    "message_schema": {
      "type": "object",
      "properties": {
        "product_path": {"type": "string"},
        "stock": {"type": "integer"},
        "timestamp": {"type": "integer"}
      }
    }
  },
  {
    "id": "content-updates",
    "url": "https://verdetao.com/channels/content",
    "format": "cbor",
    "protocol": "sse",
    "description": "Server-Sent Events for page content changes",
    "auth_required": "T2",
    "message_schema": {
      "type": "object",
      "properties": {
        "action": {"type": "string", "enum": ["added", "modified", "removed"]},
        "path": {"type": "string"},
        "hash": {"type": "string"}
      }
    }
  }
]
```

**CDDL:**

```cddl
channel = {
  "id" => tstr,
  "url" => tstr,
  "protocol" => "websocket" / "sse" / "hls" / "dash" / tstr,
  ? "format" => "cbor" / "json" / "text" / "video" / tstr,
  ? "description" => tstr,
  ? "auth_required" => "T0" / "T1" / "T2",
  ? "message_schema" => { * tstr => any },
  * tstr => any
}
```

**Agent behavior:**
- An agent SHOULD subscribe to `"content-updates"` (SSE) if available — it eliminates the need to poll the index.cbor for changes
- Stock channels enable real-time commerce decisions
- Channel authentication follows the same tier model as pages

---

## 9. CDDL Schema

```cddl
; ══════════════════════════════════════════════════════════
; CBOR-Web Multimedia Specification v3.0 — CDDL Schema
; ══════════════════════════════════════════════════════════

; ── Multimedia Content Blocks ──

rich-image = {
  "t" => "image",
  "alt" => tstr,
  "src" => tstr,
  ? "role" => image-role,
  ? "width" => uint,
  ? "height" => uint,
  ? "format" => tstr,
  ? "size" => uint,
  ? "caption" => tstr,
  ? "ai_description" => tstr,
  ? "srcset" => [+ { "src" => tstr, "width" => uint, * tstr => any }],
  ? "inline_data" => bstr .size (0..10240),
  ? "inline_format" => tstr,
  * tstr => any
}

image-role = "hero" / "product" / "logo" / "icon" / "photo" /
             "screenshot" / "chart" / "diagram" / "avatar" / "decorative" / tstr

video = {
  "t" => "video",
  "src" => tstr,
  "title" => tstr,
  ? "duration" => uint,
  ? "format" => tstr,
  ? "thumbnail" => tstr,
  ? "description" => tstr,
  ? "transcript" => tstr,
  ? "chapters" => [+ chapter],
  ? "lang" => tstr,
  ? "captions" => [+ caption-track],
  ? "size" => uint,
  * tstr => any
}

chapter = {
  "time" => uint,
  "title" => tstr,
  * tstr => any
}

caption-track = {
  "lang" => tstr,
  "src" => tstr,
  * tstr => any
}

audio = {
  "t" => "audio",
  "src" => tstr,
  "title" => tstr,
  ? "duration" => uint,
  ? "format" => tstr,
  ? "transcript" => tstr,
  ? "chapters" => [+ chapter],
  ? "lang" => tstr,
  ? "size" => uint,
  ? "episode" => podcast-episode,
  * tstr => any
}

podcast-episode = {
  ? "number" => uint,
  ? "season" => uint,
  ? "series" => tstr,
  * tstr => any
}

document-block = {
  "t" => "document",
  "src" => tstr,
  "title" => tstr,
  "format" => tstr,
  ? "size" => uint,
  ? "pages" => uint,
  ? "summary" => tstr,
  ? "extracted_text" => tstr,
  ? "lang" => tstr,
  * tstr => any
}

diagram = {
  "t" => "diagram",
  "title" => tstr,
  "syntax" => "mermaid" / "plantuml" / "dot" / "d2" / tstr,
  "source" => tstr,
  ? "rendered" => tstr,
  ? "alt" => tstr,
  * tstr => any
}

live-stream = {
  "t" => "live_stream",
  "src" => tstr,
  "title" => tstr,
  "protocol" => "websocket" / "sse" / "hls" / "dash" / tstr,
  ? "format" => tstr,
  ? "description" => tstr,
  ? "auth_required" => "T0" / "T1" / "T2",
  ? "snapshot" => { * tstr => any },
  * tstr => any
}

; ── index.cbor Channels (Key 8) ──

channel = {
  "id" => tstr,
  "url" => tstr,
  "protocol" => "websocket" / "sse" / "hls" / "dash" / tstr,
  ? "format" => "cbor" / "json" / "text" / "video" / tstr,
  ? "description" => tstr,
  ? "auth_required" => "T0" / "T1" / "T2",
  ? "message_schema" => { * tstr => any },
  * tstr => any
}
```

---

## 10. Agent Processing Guidelines

### 10.1 Text-Only Agents

A text-only agent (no image/video processing) can extract full informational value from multimedia blocks by reading text fields only:

| Block Type | Text Fields to Read | Information Captured |
|-----------|-------------------|---------------------|
| `"image"` | `"alt"`, `"caption"`, `"ai_description"` | What the image shows |
| `"video"` | `"title"`, `"description"`, `"transcript"` | Full video content |
| `"audio"` | `"title"`, `"transcript"` | Full audio content |
| `"document"` | `"title"`, `"summary"`, `"extracted_text"` | Full document content |
| `"diagram"` | `"title"`, `"alt"`, `"source"` | Diagram structure + description |
| `"live_stream"` | `"title"`, `"description"`, `"snapshot"` | Last known state |

### 10.2 Bandwidth-Aware Agents

An agent SHOULD check `"size"` before downloading media:

```python
if block["size"] > agent_max_media_size:
    use block["ai_description"] or block["transcript"] instead
```

### 10.3 Filtering by Role

An agent MAY skip multimedia blocks based on role or purpose:

```python
SKIP_ROLES = {"decorative", "icon", "avatar"}
if block.get("role") in SKIP_ROLES:
    continue  # skip this block
```

---

*CBOR-Web Multimedia Specification v3.0 — Document 3 of 6*

*Deltopide 2026*
