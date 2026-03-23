# CBOR-Web Reference

**Unified Schema, Complete Test Vectors, Glossary, Field Index, and Changelog**

```
Status:       Proposed Standard
Version:      2.1
Date:         2026-03-21
Authors:      ExploDev (Eddie Plot, Claude)
Format:       CBOR (RFC 8949)
Schema:       CDDL (RFC 8610)
License:      CC BY 4.0
Repository:   https://github.com/explodev/cbor-web
Document:     6 of 6 — CBOR-WEB-REFERENCE.md
Companion:    CBOR-WEB-CORE.md, CBOR-WEB-MULTIMEDIA.md,
              CBOR-WEB-GENERATIVE.md, CBOR-WEB-SECURITY.md,
              CBOR-WEB-ECONOMICS.md
```

---

## About This Document

This document is **part 6 of 6** — the **reference companion** for the entire CBOR-Web v2.1 specification suite. It aggregates all schemas, test vectors, definitions, and cross-references into a single lookup document.

An implementer uses this document alongside whichever specific document covers their current task. When in doubt about a type definition, field name, or test vector, this document is the authoritative reference.

| Section | Purpose |
|---------|---------|
| §1 Unified CDDL | Complete schema in one block — core + multimedia + generative + security |
| §2 Complete Test Vectors | All test vectors with hex dumps (deterministic, cross-validated) |
| §3 Glossary | Every term used across all 6 documents |
| §4 Field Index | Every field name, which document, which section |
| §5 Comparison with Standards | CBOR-Web vs llms.txt, A2A, OpenAPI, sitemap, robots.txt |
| §6 Changelog | v1.0 → v2.0 → v2.1 complete history |
| §7 References | All normative and informative references |

---

## Table of Contents

1. [Unified CDDL Schema](#1-unified-cddl-schema)
2. [Complete Test Vectors](#2-complete-test-vectors)
3. [Glossary](#3-glossary)
4. [Field Index](#4-field-index)
5. [Comparison with Existing Standards](#5-comparison-with-existing-standards)
6. [Changelog](#6-changelog)
7. [References](#7-references)

---

## 1. Unified CDDL Schema

This is the **single, complete, validatable CDDL** covering all CBOR-Web v2.1 types. It merges the schemas from:
- CBOR-WEB-CORE.md Appendix A
- CBOR-WEB-MULTIMEDIA.md Appendix A
- CBOR-WEB-GENERATIVE.md Appendix C
- CBOR-WEB-SECURITY.md Appendix A

This schema can be validated using `cddl-rs` or any RFC 8610-compliant CDDL validator.

```cddl
; ══════════════════════════════════════════════════════════════════════
; CBOR-Web Specification v2.1 — UNIFIED CDDL SCHEMA
; Document: CBOR-WEB-REFERENCE.md §1
;
; This is the SINGLE authoritative CDDL for the ENTIRE specification.
; All corrections from v2.1 review applied.
; Cross-validated against: cddl-rs (Rust CDDL validator)
; ══════════════════════════════════════════════════════════════════════


; ═══════════════════════════════
; PART 1: DOCUMENT TYPES
; (from CBOR-WEB-CORE.md)
; ═══════════════════════════════

cbor-web-document = #6.55799(manifest / sub-manifest / page / bundle)

; ── Manifest ──

manifest = {
  0 => "cbor-web-manifest",
  1 => uint,                          ; @version: MUST be 2
  2 => site-metadata,
  3 => [+ page-entry],
  ? 4 => navigation,
  5 => manifest-meta,
  ? 6 => bstr,                        ; COSE_Sign1 signature (serialized)
  ? 7 => capabilities,
  ? 8 => [+ channel],
  ? 9 => diff-manifest,
  ? 10 => manifest-security,
  * int => any
}

sub-manifest = {
  0 => "cbor-web-manifest",
  1 => uint,
  ? 2 => site-metadata,              ; OPTIONAL in sub-manifests
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

language-code = tstr .regexp "[a-z]{2}(-[A-Z]{2})?"

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


; ── Page ──

page = {
  0 => "cbor-web-page",
  1 => uint,
  2 => page-identity,
  3 => page-metadata,
  4 => [+ content-block],
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

structured-data = { "type" => tstr, * tstr => any }


; ── Bundle ──

bundle = {
  0 => "cbor-web-bundle",
  1 => uint,
  2 => manifest,
  3 => { + tstr => page },
  * int => any
}


; ═══════════════════════════════
; PART 2: CONTENT BLOCKS — CORE
; (from CBOR-WEB-CORE.md §8)
; ═══════════════════════════════

content-block = heading / paragraph / unordered-list / ordered-list /
                quote / code-block / data-table / image-ref /
                call-to-action / embed / separator / definition-list /
                note-block /
                ; multimedia (CBOR-WEB-MULTIMEDIA.md)
                rich-image / video-block / audio-block / document-block /
                diagram-block / live-stream-block

heading = { "t" => "h", "l" => uint .ge 1 .le 6, "v" => tstr }
paragraph = { "t" => "p", "v" => tstr }
unordered-list = { "t" => "ul", "v" => [+ tstr] }
ordered-list = { "t" => "ol", "v" => [+ tstr] }
quote = { "t" => "q", "v" => tstr, ? "attr" => tstr }
code-block = { "t" => "code", "v" => tstr, ? "lang" => tstr }
data-table = { "t" => "table", "headers" => [+ tstr], "rows" => [+ [+ tstr]] }
image-ref = { "t" => "img", "alt" => tstr, "src" => tstr, ? "caption" => tstr }
call-to-action = { "t" => "cta", "v" => tstr, "href" => tstr }
embed = { "t" => "embed", "src" => tstr, ? "description" => tstr }
separator = { "t" => "sep" }
definition-list = { "t" => "dl", "v" => [+ { "term" => tstr, "def" => tstr }] }
note-block = { "t" => "note", "v" => tstr, ? "level" => "info" / "warn" / "important" }


; ═══════════════════════════════
; PART 3: MULTIMEDIA BLOCKS
; (from CBOR-WEB-MULTIMEDIA.md)
; ═══════════════════════════════

semantic-role = "logo" / "product_photo" / "hero" / "illustration" /
                "screenshot" / "avatar" / "diagram" / "decorative" /
                "infographic" / "photo_editorial" / tstr

rich-image = {
  "t" => "image", "trust" => 0,
  "src" => tstr, "alt" => tstr, "semantic_role" => semantic-role,
  ? "dimensions" => { "w" => uint, "h" => uint },
  ? "format" => tstr, ? "file_size" => uint,
  ? "dominant_color" => tstr, ? "ai_description" => tstr,
  ? "caption" => tstr, ? "exif" => { * tstr => any },
  ? "inline_data" => bstr / null,
  * tstr => any
}

transcription = plain-transcription / timestamped-transcription

plain-transcription = {
  "format" => "plain",
  "lang" => language-code,
  "text" => tstr,
  * tstr => any
}

timestamped-transcription = {
  "format" => "timestamped",
  "lang" => language-code,
  "segments" => [+ segment],
  * tstr => any
}

segment = { "start" => uint, "end" => uint, "text" => tstr }

video-block = {
  "t" => "video", "trust" => 0,
  "src" => tstr, "title" => tstr, "duration_seconds" => uint,
  ? "resolution" => { "w" => uint, "h" => uint },
  ? "codec" => tstr, ? "file_size" => uint, ? "thumbnail_url" => tstr,
  ? "transcription" => transcription,
  ? "chapters" => [+ { "timestamp" => uint, "title" => tstr }],
  * tstr => any
}

audio-block = {
  "t" => "audio", "trust" => 0,
  "src" => tstr, "title" => tstr, "duration_seconds" => uint,
  ? "format" => tstr, ? "file_size" => uint,
  ? "transcription" => transcription,
  ? "speakers" => [+ { "id" => tstr, "name" => tstr, ? "role" => tstr }],
  ? "diarization" => [+ { "start" => uint, "end" => uint, "speaker" => tstr, "text" => tstr }],
  * tstr => any
}

document-block = {
  "t" => "document", "trust" => 0,
  "src" => tstr, "title" => tstr, "mime_type" => tstr,
  ? "file_size" => uint, ? "page_count" => uint,
  ? "text_extract" => tstr,
  ? "table_of_contents" => [+ { "page" => uint, "title" => tstr }],
  ? "lang" => language-code,
  * tstr => any
}

diagram-block = {
  "t" => "diagram", "trust" => 0,
  ? "src" => tstr, ? "format" => tstr,
  "description" => tstr,
  ? "diagram_type" => "flowchart" / "sequence" / "entity_relationship" /
                      "architecture" / "timeline" / "mindmap" / "organizational" / tstr,
  ? "entities" => [+ tstr],
  ? "relationships" => [+ { "from" => tstr, "to" => tstr, ? "label" => tstr }],
  * tstr => any
}

live-stream-block = {
  "t" => "live_stream", "trust" => 0,
  "title" => tstr, "stream_url" => tstr,
  "stream_format" => "hls" / "dash" / "icecast" / "rtmp" / "webrtc" / tstr,
  ? "current_show" => { "title" => tstr, ? "host" => tstr, ? "started_at" => #6.1(uint), ? "description" => tstr, * tstr => any },
  ? "schedule" => [+ { "time" => tstr, "title" => tstr, ? "host" => tstr }],
  * tstr => any
}

channel = {
  "channel_id" => tstr, "url" => tstr, "purpose" => tstr,
  "protocol" => "cbor-web-stream",
  ? "auth" => { "type" => tstr, ? "description" => tstr, * tstr => any },
  "message_schema" => { + tstr => tstr },
  "frequency" => "on_change" / "event_driven" / "periodic",
  ? "periodic_interval_ms" => uint,
  ? "example_message" => { + tstr => any },
  * tstr => any
}


; ═══════════════════════════════
; PART 4: GENERATIVE BLOCKS
; (from CBOR-WEB-GENERATIVE.md)
; ═══════════════════════════════

variable-def = {
  "type" => "string" / "number" / "integer" / "boolean" / "array" / "object",
  "required" => bool,
  "description" => tstr,
  ? "default" => any, ? "enum" => [+ any], ? "items" => tstr,
  ? "min" => number, ? "max" => number,
  ? "min_length" => uint, ? "max_length" => uint,
  ? "pattern" => tstr, ? "format" => tstr,
  ? "fields" => { + tstr => variable-def },
  * tstr => any
}

generative-block = template-block / schema-block / api-endpoint-block /
                   executable-block / workflow-block / constraint-block

template-block = {
  "t" => "template", "trust" => 1,
  "template_id" => tstr, "purpose" => tstr,
  "variables" => { + tstr => variable-def },
  ? "output_template" => tstr,
  ? "example_instantiation" => { * tstr => any },
  * tstr => any
}

schema-block = {
  "t" => "schema", "trust" => 0,
  "schema_id" => tstr, "purpose" => tstr,
  ? "version" => uint,
  "fields" => { + tstr => variable-def },
  ? "primary_key" => tstr,
  ? "required" => [+ tstr],
  ? "indexes" => [+ tstr],
  * tstr => any
}

api-endpoint-block = {
  "t" => "api_endpoint", "trust" => 3,
  "endpoint_id" => tstr, "purpose" => tstr,
  "method" => "GET" / "POST" / "PUT" / "PATCH" / "DELETE",
  "url" => tstr,
  ? "url_params" => { + tstr => variable-def },
  ? "query_params" => { + tstr => variable-def },
  ? "body" => { + tstr => variable-def },
  ? "headers" => { + tstr => tstr },
  ? "auth" => { "type" => tstr, ? "description" => tstr, * tstr => any },
  "response" => { "content_type" => tstr, ? "schema_ref" => tstr, ? "example" => { * tstr => any }, * tstr => any },
  ? "rate_limit" => { * tstr => uint },
  ? "errors" => [+ { "code" => uint, "description" => tstr }],
  * tstr => any
}

sandbox-requirements = {
  ? "network" => bool, ? "filesystem" => bool,
  ? "max_execution_time_ms" => uint, ? "max_memory_mb" => uint,
  ? "required_packages" => [+ tstr],
  * tstr => any
}

test-case = { "inputs" => { * tstr => any }, "expected_output" => { * tstr => any }, * tstr => any }

executable-block = {
  "t" => "executable", "trust" => 2,
  "exec_id" => tstr, "purpose" => tstr, "lang" => tstr,
  "inputs" => { + tstr => variable-def },
  "outputs" => { + tstr => variable-def },
  "code" => tstr,
  ? "test_cases" => [+ test-case],
  "sandbox_requirements" => sandbox-requirements,
  * tstr => any
}

workflow-step = {
  "step_id" => tstr,
  "action" => "api_call" / "execute" / "transform" / "user_choice" / "user_confirmation" / "validate",
  "purpose" => tstr,
  ? "endpoint_ref" => tstr, ? "exec_ref" => tstr, ? "schema_ref" => tstr,
  ? "params" => { * tstr => any }, ? "body" => { * tstr => any },
  ? "input_var" => tstr, ? "output_var" => tstr,
  ? "condition" => tstr, ? "display" => tstr,
  * tstr => any
}

workflow-block = {
  "t" => "workflow", "trust" => 3,
  "workflow_id" => tstr, "purpose" => tstr,
  "steps" => [+ workflow-step],
  ? "error_handling" => { * tstr => tstr },
  * tstr => any
}

constraint-rule = {
  "rule_id" => tstr, "condition" => tstr,
  "action" => "reject" / "apply" / "require" / "warn",
  ? "effect" => tstr, "message" => tstr,
  * tstr => any
}

constraint-block = {
  "t" => "constraint", "trust" => 0,
  "constraint_id" => tstr, "purpose" => tstr,
  "rules" => [+ constraint-rule],
  * tstr => any
}

form-field = {
  "name" => tstr,
  "type" => "text" / "textarea" / "email" / "tel" / "number" / "select" /
            "multi_select" / "checkbox" / "date" / "file" / "hidden",
  "label" => tstr, "required" => bool,
  ? "max_length" => uint, ? "min_length" => uint,
  ? "min" => number, ? "max" => number, ? "step" => number,
  ? "pattern" => tstr, ? "validation" => tstr,
  ? "placeholder" => tstr, ? "default" => any,
  ? "options" => [+ { "value" => tstr, "label" => tstr }],
  ? "min_selections" => uint, ? "max_selections" => uint,
  ? "accepted_types" => [+ tstr], ? "max_size" => uint,
  ? "value" => any, ? "min_date" => tstr, ? "max_date" => tstr,
  * tstr => any
}

form-block = {
  "t" => "form", "trust" => 3,
  "form_id" => tstr, "purpose" => tstr,
  "action" => tstr, "method" => "POST" / "PUT" / "PATCH",
  "submit_format" => "cbor" / "json" / "form",
  "fields" => [+ form-field],
  ? "success_message" => tstr,
  ? "captcha" => { "type" => tstr, ? "note" => tstr },
  * tstr => any
}

availability-status = "in_stock" / "low_stock" / "out_of_stock" / "pre_order" / "discontinued"

product-variant = {
  "variant_id" => tstr, "name" => tstr,
  "price" => number, "availability" => availability-status,
  ? "quantity_available" => uint, * tstr => any
}

product-block = {
  "t" => "product", "trust" => 0,
  "product_id" => tstr, "name" => tstr, "slug" => tstr,
  "description" => tstr, "price" => number, "currency" => tstr,
  "availability" => availability-status,
  ? "quantity_available" => uint,
  ? "variants" => [+ product-variant],
  ? "images" => [+ { "src" => tstr, "semantic_role" => tstr, "alt" => tstr }],
  ? "categories" => [+ tstr], ? "specs" => { * tstr => any },
  ? "certifications" => [+ tstr],
  ? "rating" => { "average" => number, "count" => uint },
  * tstr => any
}

cart-action-block = {
  "t" => "cart_action", "trust" => 3,
  "action" => tstr, "endpoint" => tstr, "method" => "POST" / "PUT",
  "body_schema" => { + tstr => variable-def },
  ? "auth" => { "type" => tstr, ? "description" => tstr, * tstr => any },
  * tstr => any
}

commerce-data = {
  ? "products" => [+ product-block],
  ? "cart_actions" => [+ cart-action-block],
  ? "checkout_url" => tstr,
  ? "payment_methods" => [+ tstr],
  ? "shipping_zones" => [+ { "zone" => tstr, "countries" => [+ tstr], "base_cost" => number, ? "free_above" => number / null }],
  * tstr => any
}

capabilities = {
  ? "static_content" => bool,
  ? "multimedia" => { ? "images" => bool, ? "video" => bool, ? "audio" => bool, ? "documents" => bool, ? "live_streams" => bool, * tstr => any },
  ? "api" => { ? "available" => bool, ? "auth_required" => bool, ? "endpoint_count" => uint, ? "docs_url" => tstr, * tstr => any },
  ? "generative" => { ? "templates" => bool, ? "schemas" => bool, ? "executables" => bool, ? "workflows" => bool, ? "constraints" => bool, * tstr => any },
  ? "live" => bool,
  ? "commerce" => { ? "available" => bool, ? "currencies" => [+ tstr], ? "checkout_type" => tstr, ? "product_count" => uint, * tstr => any },
  ? "forms" => { ? "available" => bool, ? "types" => [+ tstr], * tstr => any },
  ? "languages" => [+ tstr],
  ? "conformance" => "minimal" / "standard" / "full",
  * tstr => any
}


; ═══════════════════════════════
; PART 5: SECURITY
; (from CBOR-WEB-SECURITY.md)
; ═══════════════════════════════

manifest-security = {
  "security_level" => "S0" / "S1" / "S2",
  "token_required" => bool,
  ? "contract_address" => tstr,
  ? "chain" => "ethereum",
  ? "signing_algorithm" => "EdDSA" / "ES256" / "ES384",
  ? "public_key_url" => tstr,
  * tstr => any
}
```

---

## 2. Complete Test Vectors

All test vectors have been generated using **deterministic CBOR encoding** (RFC 8949 §4.2.1) and **cross-validated** by two independent implementations producing **byte-identical output**:
- **Rust**: ciborium 0.2.2 (`cbor-vectors/` in the repository)
- **Python**: cbor2 (canonical=True)

### 2.1 TV1 — Minimal Manifest

**Size: 206 bytes** | **SHA-256: `6536295FAA254EBD03CC61A0B338A582D25422BF8685EE57691FBA9603511C9F`**

```cbor-diag
55799({
  0: "cbor-web-manifest",
  1: 2,
  2: {"name": "Test", "domain": "test.example", "languages": ["en"], "default_language": "en"},
  3: [{"lang": "en", "path": "/", "size": 95, "title": "Home", "access": "public"}],
  5: {"total_size": 95, "total_pages": 1, "generated_at": 1(1742515200), "bundle_available": false}
})
```

**Hex dump (annotated):**
```
D9 D9F7                              -- tag(55799) self-described CBOR
  A5                                  -- map(5) — 5 top-level keys
    00                                -- key: 0 (@type)
    71 63626F722D7765622D6D616E696665
       7374                           -- text(17) "cbor-web-manifest"
    01                                -- key: 1 (@version)
    02                                -- unsigned(2)
    02                                -- key: 2 (site)
    A4                                -- map(4)
      64 6E616D65                     -- "name"
      64 54657374                     -- "Test"
      66 646F6D61696E                 -- "domain"
      6C 746573742E6578616D706C65     -- "test.example"
      69 6C616E677561676573           -- "languages"
      81 62 656E                      -- ["en"]
      70 64656661756C745F6C616E677561
         6765                         -- "default_language"
      62 656E                         -- "en"
    03                                -- key: 3 (pages)
    81                                -- array(1)
      A5                              -- map(5)
        64 6C616E67                   -- "lang"
        62 656E                       -- "en"
        64 70617468                   -- "path"
        61 2F                         -- "/"
        64 73697A65                   -- "size"
        18 5F                         -- unsigned(95)
        65 7469746C65                 -- "title"
        64 486F6D65                   -- "Home"
        66 616363657373               -- "access"
        66 7075626C6963               -- "public"
    05                                -- key: 5 (meta)
    A4                                -- map(4)
      6A 746F74616C5F73697A65        -- "total_size"
      18 5F                           -- unsigned(95)
      6B 746F74616C5F7061676573      -- "total_pages"
      01                              -- unsigned(1)
      6C 67656E6572617465645F6174    -- "generated_at"
      C1 1A 67DCAC00                  -- tag(1) unsigned(1742515200)
      70 62756E646C655F617661696C6162
         6C65                         -- "bundle_available"
      F4                              -- false
```

**Key ordering verification:**
- Top-level: 0 (1B) < 1 (1B) < 2 (1B) < 3 (1B) < 5 (1B) ✅
- site-metadata: `name` (5B) < `domain` (7B) < `languages` (10B) < `default_language` (17B) ✅
- page-entry: `lang` (5B) < `path` (5B,`70>6C`) < `size` (5B,`73>70`) < `title` (6B) < `access` (7B) ✅
- meta: `total_size` (11B) < `total_pages` (12B) < `generated_at` (13B) < `bundle_available` (17B) ✅

### 2.2 TV2 — Minimal Page

**Size: 127 bytes** | **SHA-256: `D8CAD2E6E8D06A0EF4E5B22C7394E4AC2B48FDE517DBB012C705DB26D163CEB7`**

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

**Hex dump (annotated):**
```
D9 D9F7                              -- tag(55799)
  A5                                  -- map(5)
    00                                -- key: 0
    6D 63626F722D7765622D70616765     -- "cbor-web-page"
    01                                -- key: 1
    02                                -- unsigned(2)
    02                                -- key: 2
    A3                                -- map(3) — identity
      64 6C616E67                     -- "lang"
      62 656E                         -- "en"
      64 70617468                     -- "path"
      61 2F                           -- "/"
      69 63616E6F6E6963616C           -- "canonical"
      75 68747470733A2F2F746573742E65
         78616D706C652F               -- "https://test.example/"
    03                                -- key: 3
    A1                                -- map(1) — metadata
      65 7469746C65                   -- "title"
      67 57656C636F6D65               -- "Welcome"
    04                                -- key: 4
    82                                -- array(2) — content blocks
      A3                              -- map(3) — heading
        61 6C                         -- "l"
        01                            -- unsigned(1)
        61 74                         -- "t"
        61 68                         -- "h"
        61 76                         -- "v"
        67 57656C636F6D65             -- "Welcome"
      A2                              -- map(2) — paragraph
        61 74                         -- "t"
        61 70                         -- "p"
        61 76                         -- "v"
        6D 48656C6C6F2C20576F726C6421 -- "Hello, World!"
```

**Key ordering verification:**
- identity: `lang` (5B,`6C`) < `path` (5B,`70`) < `canonical` (10B) ✅
- heading block: `l` (2B,`6C`) < `t` (2B,`74`) < `v` (2B,`76`) ✅ (same length, bytewise)
- paragraph block: `t` (2B,`74`) < `v` (2B,`76`) ✅

### 2.3 TV3 — Product Page with Structured Data

**Size: 541 bytes** | **SHA-256: `9FC41CE55481DEB75F02B545C8B3FC24977AC30A3A70C489F78E4B56035BA68F`**

Binary file available at `test-vectors/tv3_product.cbor` in the repository.

### 2.4 Test Vector Summary

| Vector | Description | Size | SHA-256 |
|--------|-------------|------|---------|
| TV1 | Minimal Manifest | 206 B | `6536295FAA254EBD03CC61A0B338A582D25422BF8685EE57691FBA9603511C9F` |
| TV2 | Minimal Page | 127 B | `D8CAD2E6E8D06A0EF4E5B22C7394E4AC2B48FDE517DBB012C705DB26D163CEB7` |
| TV3 | Product Page | 541 B | `9FC41CE55481DEB75F02B545C8B3FC24977AC30A3A70C489F78E4B56035BA68F` |

All binary `.cbor` files are in the `test-vectors/` directory of the repository.

---

## 3. Glossary

Complete alphabetical glossary of all terms used across the 6 CBOR-Web documents.

| Term | Definition | Primary Document |
|------|-----------|-----------------|
| **Agent** | Any autonomous software that consumes CBOR-Web content (AI assistant, crawler, pipeline) | CORE §2.2.3 |
| **Badge** | The token holding model — hold ≥ 1 token for permanent access, not spent per request | ECONOMICS §1.1 |
| **Block** | See Content Block | CORE §8 |
| **Block Type Code** | Short string identifying a content block type (`"h"`, `"p"`, `"ul"`, `"image"`, `"template"`, etc.) | CORE §8.2 |
| **Bundle** | CBOR document containing manifest + all pages in one file | CORE §7 |
| **Capability** | A declared feature of a CBOR-Web site (multimedia, API, generative, commerce, etc.) | GENERATIVE §14 |
| **CBOR** | Concise Binary Object Representation (RFC 8949) — the encoding format for all CBOR-Web documents | CORE §2.2.1 |
| **CBORW** | Symbol of the CBOR-Web ERC-20 utility token | ECONOMICS §2 |
| **CDDL** | Concise Data Definition Language (RFC 8610) — schema language for CBOR | CORE §2.2.1 |
| **Channel** | A real-time content stream exposed via WebSocket with CBOR messages | MULTIMEDIA §8 |
| **Conformance Level** | Implementation completeness: Minimal, Standard, or Full | CORE §11 |
| **Constraint** | A declarative business rule in a non-Turing-complete expression language | GENERATIVE §10 |
| **Content Block** | A typed unit of page content (heading, paragraph, list, table, image, etc.) | CORE §8 |
| **COSE** | CBOR Object Signing and Encryption (RFC 9052) — used for manifest signatures | SECURITY §6 |
| **Declarative (trust 0)** | A block containing only data — safe to process without restrictions | GENERATIVE §2 |
| **Deterministic Encoding** | RFC 8949 §4.2 rules ensuring identical binary output for identical data | CORE §3.1 |
| **Diarization** | Speaker-attributed audio transcription (who said what, when) | MULTIMEDIA §4.5 |
| **Diff Manifest** | A manifest fragment containing only changes since a previous version | CORE §10.5 |
| **Discovery** | The process of finding a CBOR-Web manifest on a website | CORE §4 |
| **ecrecover** | Ethereum function to recover a wallet address from a signed message | SECURITY §5.3 |
| **Editorial Block** | Content block containing pure content signal (h, p, ul, ol, q, code, table, dl, note, sep) | CORE §8.15 |
| **ERC-20** | Ethereum token standard (EIP-20) | ECONOMICS §2 |
| **Executable (trust 2)** | A block containing runnable code — MUST be sandboxed | GENERATIVE §8 |
| **Forward Compatibility** | Rule that agents MUST ignore unknown keys, enabling spec evolution | CORE §3.6 |
| **Full Access (L1)** | All CBOR-Web content, available to token holders | SECURITY §4.1 |
| **Generative Block** | A block containing productive intelligence (template, schema, API, executable, workflow, constraint) | GENERATIVE §1 |
| **Interactive (trust 3)** | A block requiring network interaction (API call, form submit, cart action) | GENERATIVE §2 |
| **Manifest** | CBOR document describing a site: metadata, page index, navigation, capabilities, security | CORE §5 |
| **MiCA** | EU Markets in Crypto-Assets Regulation (2023/1114) | ECONOMICS §9 |
| **Mustache Subset** | The template language used in CBOR-Web template blocks (logic-less, safe) | GENERATIVE §5 |
| **Non-Editorial Block** | Content block containing marketing/navigation elements (cta, embed, img) | CORE §8.15 |
| **Nonce** | Anti-replay timestamp included in every authenticated request | SECURITY §5.4 |
| **Page** | CBOR document containing structured content of a single web page | CORE §6 |
| **Path Encoding** | Bijective transformation from URL paths to CBOR-Web filenames (§6.1) | CORE §6.1 |
| **Publisher** | Tool or service that generates CBOR-Web documents from HTML | CORE §2.2.3 |
| **Sandbox** | Isolated execution environment (WASM recommended) for executable blocks | SECURITY §8 |
| **Schema Block** | Declarative data structure definition that an agent can use for code generation | GENERATIVE §6 |
| **Self-Described CBOR** | Tag 55799 (`D9 D9 F7`) prefixed to every CBOR-Web document | CORE §3.2 |
| **Semantic Role** | Purpose classification of a multimedia element (logo, product_photo, decorative, etc.) | MULTIMEDIA §2.4 |
| **Signal-to-Noise Ratio** | Proportion of useful content bytes vs total document bytes | CORE §2.2.5 |
| **SSRF** | Server-Side Request Forgery — attack via URL fields pointing to internal resources | SECURITY §10 |
| **Storefront (L0)** | Public content visible without a token: manifest, public pages, all metadata | SECURITY §4.1 |
| **Sub-Manifest** | Paginated fragment of the manifest for sites > 500 pages | CORE §5.8 |
| **Template (trust 1)** | A block using Mustache string interpolation — no code execution, safe | GENERATIVE §4 |
| **Token** | CBOR-Web ERC-20 utility token (CBORW) on Ethereum mainnet | SECURITY §4, ECONOMICS §2 |
| **Token Holder** | Agent whose Ethereum wallet holds ≥ 1 CBORW token | SECURITY §4 |
| **Transcription** | Text representation of audio/video content (plain or timestamped) | MULTIMEDIA §3.4 |
| **Trust Level** | Security classification: 0=declarative, 1=template, 2=executable, 3=interactive | GENERATIVE §2 |
| **TWAP** | Time-Weighted Average Price — Uniswap oracle for stabilization | ECONOMICS §5.3 |
| **Variable Type System** | Shared type system for template/schema/API/executable variables | GENERATIVE §3 |
| **Well-Known URL** | `/.well-known/cbor-web` — canonical discovery endpoint | CORE §4.1 |
| **Workflow** | Multi-step autonomous process chaining API calls, executables, and user interactions | GENERATIVE §9 |

---

## 4. Field Index

Every field name across all CBOR-Web documents, with the document and section where it is defined.

### 4.1 Top-Level Document Keys (Integer)

| Key | Name | Used In | Document | Section |
|-----|------|---------|----------|---------|
| 0 | @type | Manifest, Page, Bundle | CORE | §5.2, §6.3, §7.5 |
| 1 | @version | Manifest, Page, Bundle | CORE | §5.2, §6.3, §7.5 |
| 2 | site / identity | Manifest, Page | CORE | §5.3, §6.4 |
| 3 | pages / metadata | Manifest, Page | CORE | §5.4, §6.5 |
| 4 | navigation / content | Manifest, Page | CORE | §5.5, §6.6 |
| 5 | meta / links | Manifest, Page | CORE | §5.6, §6.7 |
| 6 | signature / structured_data | Manifest, Page | SECURITY §6 / CORE §6.8 |
| 7 | capabilities / generative | Manifest, Page | GENERATIVE §14 / §15 |
| 8 | channels / forms | Manifest, Page | MULTIMEDIA §8 / GENERATIVE §12 |
| 9 | diff / commerce | Manifest, Page | CORE §10.5 / GENERATIVE §13 |
| 10 | security | Manifest only | SECURITY §14 |

### 4.2 Content Block Keys

| Key | Name | Type | Used In | Document |
|-----|------|------|---------|----------|
| `"t"` | type | text | All blocks | CORE §8.2 |
| `"v"` | value | text/array | h, p, ul, ol, q, code, cta, dl, note | CORE §8.2 |
| `"l"` | level | uint | h | CORE §8.3 |
| `"trust"` | trust level | uint | All multimedia + generative | GENERATIVE §2 |
| `"alt"` | alt text | text | img, image | CORE §8.9, MULTIMEDIA §2.3 |
| `"src"` | source URL | text | img, image, video, audio, document, diagram, embed | Various |
| `"attr"` | attribution | text | q | CORE §8.6 |
| `"lang"` | language | text | code | CORE §8.7 |
| `"headers"` | table headers | array | table | CORE §8.8 |
| `"rows"` | table rows | array | table | CORE §8.8 |
| `"href"` | link destination | text | cta | CORE §8.10 |
| `"caption"` | caption | text | img, image | CORE §8.9, MULTIMEDIA §2.3 |
| `"description"` | description | text | embed, diagram | CORE §8.11, MULTIMEDIA §6.3 |
| `"level"` | severity | text | note | CORE §8.14 |

### 4.3 Multimedia-Specific Keys

| Key | Block | Type | Document | Section |
|-----|-------|------|----------|---------|
| `"semantic_role"` | image | text | MULTIMEDIA | §2.4 |
| `"dimensions"` | image, video | map | MULTIMEDIA | §2.7, §3.3 |
| `"format"` | image, audio | text | MULTIMEDIA | §2.3, §4.3 |
| `"file_size"` | image, video, audio, document | uint | MULTIMEDIA | §2.3, §3.3, §4.3, §5.3 |
| `"dominant_color"` | image | text | MULTIMEDIA | §2.3 |
| `"ai_description"` | image | text | MULTIMEDIA | §2.3 |
| `"inline_data"` | image | bstr/null | MULTIMEDIA | §2.5 |
| `"exif"` | image | map | MULTIMEDIA | §2.6 |
| `"duration_seconds"` | video, audio | uint | MULTIMEDIA | §3.3, §4.3 |
| `"transcription"` | video, audio | map | MULTIMEDIA | §3.4 |
| `"chapters"` | video | array | MULTIMEDIA | §3.5 |
| `"speakers"` | audio | array | MULTIMEDIA | §4.4 |
| `"diarization"` | audio | array | MULTIMEDIA | §4.5 |
| `"text_extract"` | document | text | MULTIMEDIA | §5.3 |
| `"table_of_contents"` | document | array | MULTIMEDIA | §5.3 |
| `"entities"` | diagram | array | MULTIMEDIA | §6.3 |
| `"relationships"` | diagram | array | MULTIMEDIA | §6.3 |
| `"stream_url"` | live_stream | text | MULTIMEDIA | §7.3 |
| `"stream_format"` | live_stream | text | MULTIMEDIA | §7.3 |

### 4.4 Generative-Specific Keys

| Key | Block | Type | Document | Section |
|-----|-------|------|----------|---------|
| `"template_id"` | template | text | GENERATIVE | §4.3 |
| `"variables"` | template | map | GENERATIVE | §4.3 |
| `"output_template"` | template | text | GENERATIVE | §4.3 |
| `"schema_id"` | schema | text | GENERATIVE | §6.3 |
| `"fields"` | schema | map | GENERATIVE | §6.3 |
| `"endpoint_id"` | api_endpoint | text | GENERATIVE | §7.3 |
| `"method"` | api_endpoint, form | text | GENERATIVE | §7.3, §12.3 |
| `"url"` | api_endpoint | text | GENERATIVE | §7.3 |
| `"exec_id"` | executable | text | GENERATIVE | §8.3 |
| `"code"` | executable | text | GENERATIVE | §8.3 |
| `"sandbox_requirements"` | executable | map | GENERATIVE | §8.4 |
| `"test_cases"` | executable | array | GENERATIVE | §8.6 |
| `"workflow_id"` | workflow | text | GENERATIVE | §9.2 |
| `"steps"` | workflow | array | GENERATIVE | §9.2 |
| `"constraint_id"` | constraint | text | GENERATIVE | §10.2 |
| `"rules"` | constraint | array | GENERATIVE | §10.2 |
| `"form_id"` | form | text | GENERATIVE | §12.3 |
| `"action"` | form, cart_action | text | GENERATIVE | §12.3, §13.5 |
| `"product_id"` | product | text | GENERATIVE | §13.3 |
| `"availability"` | product | text | GENERATIVE | §13.4 |

---

## 5. Comparison with Existing Standards

### 5.1 Comprehensive Feature Matrix

| Feature | robots.txt | sitemap.xml | llms.txt | Schema.org | OpenAPI | A2A (Google) | **CBOR-Web** |
|---------|-----------|-------------|----------|------------|---------|-------------|-------------|
| **Format** | Text | XML | Markdown | JSON-LD | JSON/YAML | JSON | **Binary CBOR** |
| **Purpose** | Crawl rules | URL list | AI summary | Entity data | API spec | Agent protocol | **Full content** |
| **Content delivery** | No | No | Summary | Metadata only | No | No | **Yes (all text)** |
| **Navigation** | No | Flat URLs | Flat links | No | No | No | **Typed hierarchy** |
| **Multilingual** | No | hreflang | No | Yes | No | No | **Per-page + manifest** |
| **Structured data** | No | No | No | Yes | No | No | **Native CBOR** |
| **Incremental updates** | No | lastmod | No | No | No | No | **SHA-256 + diffs** |
| **Single-request index** | No | Partial | Yes | No | Yes | No | **Yes (bundle)** |
| **Access control** | No | No | No | No | API keys | OAuth | **ERC-20 token** |
| **Binary efficiency** | N/A | N/A | N/A | N/A | N/A | N/A | **Zero tokenization** |
| **Multimedia metadata** | No | No | No | Partial | No | No | **Full (transcriptions)** |
| **API discovery** | No | No | No | No | Yes | Yes | **Yes (api_endpoint)** |
| **Forms** | No | No | No | No | No | No | **Yes (form block)** |
| **Commerce** | No | No | No | Partial | No | No | **Yes (product+cart)** |
| **Generative** | No | No | No | No | No | No | **Yes (6 block types)** |
| **Real-time** | No | No | No | No | No | No | **Yes (channels)** |

### 5.2 When to Use What

| Need | Best Tool | Why |
|------|----------|-----|
| Tell crawlers what to avoid | robots.txt | Industry standard, universal support |
| Give search engines URL list | sitemap.xml | Universal search engine support |
| Give AI agents a text summary | llms.txt | Quick to create, good for LLM context windows |
| Expose product/org metadata | Schema.org / JSON-LD | Google, Bing, and other engines consume it |
| Document your API | OpenAPI / Swagger | Developer ecosystem, code generation tools |
| Enable agent-to-agent communication | A2A (Google) | Agent coordination and discovery |
| **Give AI agents your full content in binary** | **CBOR-Web** | **Complete content + structure, zero noise** |

These standards are **complementary**, not competing. The recommended stack:

```
robots.txt         ← Crawl permissions (keep this)
sitemap.xml        ← URL discovery (keep this)
Schema.org/JSON-LD ← Structured metadata (keep this)
llms.txt           ← AI text summary (add this)
CBOR-Web           ← Full binary content (add this)
```

Each layer adds value. CBOR-Web is the richest layer — it provides everything an agent needs in one binary download.

---

## 6. Changelog

### 6.1 v1.0 (2026-03-21) — Initial Release

- Core specification: manifest, pages, bundles
- 13 core content block types (h, p, ul, ol, q, code, table, img, cta, embed, sep, dl, note)
- Discovery protocol (well-known URL, HTTP Link, HTML meta, robots.txt, llms.txt)
- Hash-based caching and incremental updates
- HTTPS transport requirement
- Three conformance levels (Minimal, Standard, Full)
- CDDL schema and test vectors
- 8-layer security model (DID/IOTA/PoW/trust score — later replaced in v2.1)

### 6.2 v2.0 (2026-03-21) — Extensions

- 6 multimedia block types (image, video, audio, document, diagram, live_stream)
- 6 generative block types (template, schema, api_endpoint, executable, workflow, constraint)
- Capability declaration (manifest key 7)
- Form blocks (page key 8)
- Commerce protocol (page key 9)
- Real-time streaming channels (manifest key 8)
- Differential manifest updates (manifest key 9)
- Trust level classification for blocks (0-3)
- Binary data encoding rules (raw bstr, not base64)

### 6.3 v2.1 (2026-03-21) — Consolidated Standard

**This is the current version.** It consolidates all prior documents and applies corrections from the inter-document review.

**Security model replacement:**
- **REMOVED**: 8-layer security model (DID W3C, IOTA Tangle, proof-of-work adaptive, trust score 0-100, Verifier Marketplace)
- **ADDED**: Token-based access control (ERC-20 CBORW on Ethereum mainnet)
- Two access levels: L0 (storefront/public) and L1 (token holder/full)
- Wallet identity via ecrecover signature verification
- Nonce-based replay protection with 60-second window
- Server-side balanceOf caching (1-hour TTL)

**Critical fixes (from cross-document review):**

| ID | Fix | Description |
|----|-----|-------------|
| C-01 | Test vectors | All hex dumps regenerated with RFC 8949 §4.2.1 deterministic encoding. Cross-validated Rust ↔ Python. |
| C-02 | Security model | Unified to token badge ERC-20. Removed DID/IOTA/PoW. |
| C-03 | Manifest key 10 | Added `manifest-security` type to CDDL. |
| C-04 | v3.0 reference | Removed reference to non-existent v3.0 document. |
| C-05 | Key 6 type | Aligned signature as bstr wrapping COSE_Sign1 (not raw array). |

**Major fixes:**

| ID | Fix | Description |
|----|-----|-------------|
| M-01 | Path encoding | Bijective encoding: percent-encode `_` → `%5F` before `/` → `_`. |
| M-02 | Bundle hash | Clarified: prepend tag 55799 before hashing bundled pages. |
| M-03 | Sub-manifest | Distinct CDDL type where key 2 (site-metadata) is OPTIONAL. |
| M-04 | Navigation | Key 4 corrected to OPTIONAL in CDDL (was REQUIRED). |
| M-05 | Templates | Adopted Mustache subset with formal EBNF grammar. |
| M-06 | Transcription | Split into `plain-transcription` / `timestamped-transcription` in CDDL. |

**Threat model additions:**

| ID | Threat | Section |
|----|--------|---------|
| T16 | Prompt injection | SECURITY §11 |
| T17 | SSRF via all URL fields | SECURITY §10 |
| T18 | Workflow DDoS | SECURITY §8.7 |

**Minor fixes:**
- Removed duplicate `"conformance"` field (now only in capabilities)
- Corrected `"inline_data"` as raw bstr, not base64
- Added `"access"` field to page-entry CDDL (REQUIRED)
- Binary watermark documented honestly as friction mechanism
- page-links arrays allow empty (`[*` instead of `[+`)

---

## 7. References

### 7.1 Normative References

| Reference | Title |
|-----------|-------|
| [RFC 2119] | Key words for use in RFCs to Indicate Requirement Levels |
| [RFC 8174] | Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words |
| [RFC 8949] | Concise Binary Object Representation (CBOR), STD 94 |
| [RFC 8610] | Concise Data Definition Language (CDDL) |
| [RFC 9052] | CBOR Object Signing and Encryption (COSE): Structures and Process, STD 96 |
| [RFC 9053] | COSE: Initial Algorithms |
| [RFC 8615] | Well-Known Uniform Resource Identifiers (URIs) |
| [RFC 9309] | Robots Exclusion Protocol |
| [RFC 1918] | Address Allocation for Private Internets |
| [RFC 6890] | Special-Purpose IP Address Registries |
| [ERC-20] | EIP-20: Token Standard (Ethereum) |
| [EIP-191] | Signed Data Standard (Ethereum) |

### 7.2 Informative References

| Reference | Title | URL |
|-----------|-------|-----|
| [llms.txt] | A proposed standard for making websites readable by LLMs | https://llmstxt.org/ |
| [Schema.org] | Shared vocabulary for structured data | https://schema.org/ |
| [Mustache] | Logic-less templates | https://mustache.github.io/ |
| [MiCA] | EU Regulation 2023/1114 on markets in crypto-assets | EUR-Lex |
| [eIDAS 2.0] | EU framework for European Digital Identity | EUR-Lex |
| [OpenAPI] | OpenAPI Specification 3.1 | https://spec.openapis.org/oas/v3.1.0 |
| [A2A] | Google Agent-to-Agent Protocol | https://google.github.io/A2A/ |
| [OpenZeppelin] | OpenZeppelin Contracts (ERC-20) | https://docs.openzeppelin.com/contracts/ |
| [Uniswap V3] | Uniswap V3 Core | https://uniswap.org/whitepaper-v3.pdf |
| [OWASP SSRF] | Server-Side Request Forgery Prevention | https://cheatsheetseries.owasp.org/ |

---

*CBOR-Web Reference v2.1 — Document 6 of 6*

*ExploDev 2026 — "The web has two clients: humans and machines. It's time to serve both."*
