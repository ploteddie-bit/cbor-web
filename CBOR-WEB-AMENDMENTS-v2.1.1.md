# CBOR-Web Amendments v2.1.1

**Applies to:** CBOR-Web Core Specification v2.1
**Date:** 2026-03-24
**Status:** Proposed amendments for integration into v2.2

---

## Amendment 1 — Inline Annotations (§8.4 extension)

### Problem
Stripping `<strong>`, `<em>`, `<a>` from paragraphs loses semantic meaning. Example: "This is **not** recommended" → "This is not recommended" loses emphasis that changes meaning.

### Solution
Add an OPTIONAL `"annotations"` key to paragraph blocks:

```cbor-diag
{
  "t": "p",
  "v": "This is not recommended for pregnant women.",
  "annotations": [
    {"start": 8, "end": 11, "type": "strong"},
    {"start": 32, "end": 46, "type": "link", "href": "/safety"}
  ]
}
```

### Annotation Types

| Type | Meaning | Extra Keys |
|------|---------|-----------|
| `"strong"` | Strong emphasis (was `<strong>`) | — |
| `"em"` | Emphasis (was `<em>`) | — |
| `"link"` | Inline link (was `<a>`) | `"href"`: destination path or URL |
| `"code"` | Inline code (was `<code>`) | — |
| `"abbr"` | Abbreviation (was `<abbr>`) | `"title"`: expansion |
| `"sup"` | Superscript | — |
| `"sub"` | Subscript | — |

### CDDL

```cddl
paragraph = {
  "t" => "p",
  "v" => tstr,
  ? "annotations" => [+ annotation],
  * tstr => any
}

annotation = {
  "start" => uint,       ; byte offset in UTF-8 "v" string
  "end" => uint,         ; byte offset (exclusive)
  "type" => "strong" / "em" / "link" / "code" / "abbr" / "sup" / "sub" / tstr,
  ? "href" => tstr,      ; for "link" type
  ? "title" => tstr,     ; for "abbr" type
  * tstr => any
}
```

### Backward Compatibility
- `"annotations"` is OPTIONAL — v2.1 agents ignore it (forward-compatibility rule §3.6)
- `"v"` still contains the complete plain text — agents that don't process annotations get the full content
- Publishers MAY omit annotations entirely (Minimal conformance)

---

## Amendment 2 — Table Column Types (§8.8 extension)

### Problem
Table cells are always text strings. An agent must guess if "29.90" is a price, a temperature, or a score.

### Solution
Add an OPTIONAL `"col_types"` key to table blocks:

```cbor-diag
{
  "t": "table",
  "headers": ["Product", "Price", "Stock", "Last Updated"],
  "col_types": ["text", "currency:EUR", "number", "date"],
  "rows": [
    ["Lion's Mane", "29.90", "142", "2026-03-20"],
    ["Reishi", "24.90", "89", "2026-03-19"]
  ]
}
```

### Column Type Values

| Type | Format | Example |
|------|--------|---------|
| `"text"` | Plain text (default) | `"Lion's Mane"` |
| `"number"` | Numeric | `"142"`, `"3.14"` |
| `"currency:XXX"` | Price with ISO 4217 code | `"29.90"` (currency in type) |
| `"date"` | ISO 8601 date | `"2026-03-20"` |
| `"datetime"` | ISO 8601 datetime | `"2026-03-20T14:30:00Z"` |
| `"boolean"` | True/false | `"true"`, `"false"` |
| `"url"` | URL | `"https://..."` |
| `"email"` | Email address | `"contact@..."` |
| `"percent"` | Percentage | `"95.2"` (% implicit) |

### CDDL

```cddl
data-table = {
  "t" => "table",
  "headers" => [+ tstr],
  "rows" => [+ [+ tstr]],
  ? "col_types" => [+ tstr],    ; same length as "headers"
  * tstr => any
}
```

### Backward Compatibility
- `"col_types"` is OPTIONAL — v2.1 agents ignore it
- Cell values remain text strings regardless of type hint
- `"col_types"` array MUST have the same length as `"headers"`

---

## Amendment 3 — Non-UTF8 Encoding Handling (§3.3 extension)

### Problem
§3.3 requires UTF-8 but doesn't specify what to do when a publisher encounters non-UTF8 source content (Latin-1, Windows-1252 — common on old sites).

### Solution
Add to §3.3:

**Publisher behavior with non-UTF8 sources:**

A publisher converting HTML to CBOR-Web MUST:
1. Detect the source encoding (from `<meta charset>`, HTTP `Content-Type`, or byte analysis)
2. Convert to UTF-8 before CBOR encoding
3. If conversion fails for a specific character → replace with U+FFFD (REPLACEMENT CHARACTER)
4. Log a warning for each replacement

**Agent behavior with invalid UTF-8:**

An agent receiving a CBOR-Web document containing invalid UTF-8 sequences SHOULD:
1. Attempt to process the document (fail open for content)
2. Replace invalid sequences with U+FFFD locally
3. Log a warning
4. NOT reject the entire document for encoding errors in text values

An agent MUST reject a document where CBOR structural elements (map keys, block type codes) contain invalid UTF-8 — these indicate a broken encoder, not a content issue.

### Encoding Detection Priority

| Source | Detection Method |
|--------|-----------------|
| HTML `<meta charset="...">` | Authoritative if present |
| HTTP `Content-Type: text/html; charset=...` | Authoritative if present |
| BOM (Byte Order Mark) | Reliable for UTF-8/UTF-16 |
| Byte analysis (heuristic) | Fallback — chardet/uchardet |

---

## Amendment 4 — Feedback Endpoint (new §14.8)

### Problem
No mechanism for agents to report errors, stale content, or inconsistencies to publishers.

### Solution
Define an OPTIONAL feedback endpoint:

```
POST /.well-known/cbor-web/feedback
Content-Type: application/cbor
```

Body:

```cbor-diag
{
  "type": "error",
  "path": "/products/lions-mane",
  "issue": "hash_mismatch",
  "details": "Page hash does not match manifest hash. Manifest may be stale.",
  "agent": "cbor-crawl/1.0.0",
  "timestamp": 1(1742598400)
}
```

Feedback types:

| Type | Meaning |
|------|---------|
| `"error"` | Technical error (hash mismatch, malformed CBOR, broken link) |
| `"stale"` | Content appears outdated vs HTML |
| `"inconsistency"` | CBOR and HTML content diverge |
| `"suggestion"` | Improvement suggestion |

**Publisher response:**

```
HTTP/1.1 202 Accepted
```

Or `404` if the publisher does not support feedback.

Publishers MAY use feedback to trigger content regeneration (`text2cbor` re-run).

---

## Amendment 5 — DNS TXT Discovery (new §4.10)

### Problem
No mechanism for agents to discover CBOR-Web support across many sites at scale without hitting each site individually.

### Solution
A publisher MAY declare CBOR-Web support via DNS TXT record:

```
_cbor-web.example.com. 3600 IN TXT "v=2.1; manifest=/.well-known/cbor-web; tiers=T1,T2; pages=25"
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `v` | REQUIRED | Spec version |
| `manifest` | OPTIONAL | Manifest URL (default: `/.well-known/cbor-web`) |
| `tiers` | OPTIONAL | Supported access tiers |
| `pages` | OPTIONAL | Approximate page count |
| `kid` | OPTIONAL | Signing key ID (for signature verification) |
| `pk` | OPTIONAL | Base64url-encoded public key |

### Agent behavior

An agent scanning many domains can:
1. Batch DNS TXT lookups for `_cbor-web.*.` across target domains
2. Filter: only domains with the TXT record support CBOR-Web
3. Fetch manifests only for confirmed domains

This eliminates thousands of 404 responses when scanning for CBOR-Web adoption.

---

*CBOR-Web Amendments v2.1.1 — Proposed for v2.2*

*ExploDev 2026*
