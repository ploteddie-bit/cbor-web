# CBOR-Web Doléance — Agent Feedback Protocol

**The Missing Return Channel: How AI Agents Communicate Content Quality Back to Publishers**

```
Status:       Proposed Standard
Version:      1.0
Date:         2026-03-23
Authors:      ExploDev (Eddie Plot)
Extends:      CBOR-Web Core v2.1.3 (Reading Contract, Manifest key 11)
Format:       CBOR (RFC 8949)
License:      CC BY 4.0
Repository:   https://github.com/ploteddie-bit/cbor-web
Document:     Companion to CBOR-WEB-CORE.md, part 7 of 7
```

---

## About This Document

Every web standard today is a monologue: the publisher speaks, the agent listens. robots.txt, sitemap.xml, llms.txt, Schema.org, OpenAPI — they all flow in one direction. The agent has no voice.

This document defines the **return channel** — a standardized protocol enabling AI agents to communicate back to publishers how they consumed content, what was useful, what was noise, and what was missing. We call this the **Doléance Protocol**, borrowing the French word for formal grievance — a structured, dignified expression of what needs to change.

This is not analytics. Analytics counts visits. Doléance measures **comprehension**.

---

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Design Principles](#2-design-principles)
3. [Reading Contract — Manifest Key 11](#3-reading-contract--manifest-key-11)
4. [Agent Reading Profile](#4-agent-reading-profile)
5. [Publisher Adaptation](#5-publisher-adaptation)
6. [Quality Score — The Core Metric](#6-quality-score--the-core-metric)
7. [Doléance HTTP Protocol](#7-dol%C3%A9ance-http-protocol)
8. [Ecosystem Services](#8-ecosystem-services)
9. [Privacy and Ethical Considerations](#9-privacy-and-ethical-considerations)
10. [Economic Model](#10-economic-model)
11. [CDDL Schema](#11-cddl-schema)
12. [Examples](#12-examples)
- [Appendix I: Doléance vs Existing Standards](#appendix-i-dol%C3%A9ance-vs-existing-standards)

---

## 1. Problem Statement

### 1.1 The Silent Consumer

When an AI agent reads a website today, the interaction is invisible:

| What Happens | Who Knows |
|---|---|
| Agent downloads 1.5 MB of JavaScript to find 500 bytes of product data | Nobody |
| Agent ignores 90% of the page content (menus, CTAs, cookie banners) | Nobody |
| Agent produces a poor answer because structured data was missing | The user, not the publisher |
| Agent visits in French but the French translation is machine-generated garbage | Nobody |
| Agent tries 3 different pages before finding the information it needs | Nobody |

The publisher optimizes for humans (click-through rates, bounce rates, conversion funnels). The agent has no equivalent feedback mechanism. The publisher is blind to how machines consume content.

### 1.2 The Cost of Silence

Without agent feedback, publishers cannot:

- Know which content blocks agents actually use
- Know which languages agents consume vs. ignore
- Know whether their structured data is complete or broken
- Know whether their CBOR-Web implementation is efficient
- Know whether agents trust their content

Without publisher adaptation, agents cannot:

- Reduce consumption cost on repeat visits
- Signal that content quality is degrading
- Request missing data types (e.g., "I need nutritional data but you only have marketing text")
- Build trust relationships with reliable publishers
- Improve the web for future agents

### 1.3 What Doléance Solves

Doléance transforms the agent-publisher relationship from a **monologue** into a **dialogue**:

```
TODAY:

Publisher ──── content ────► Agent ──── silence ────► void


WITH DOLÉANCE:

Publisher ──── content ────► Agent
    ▲                          │
    │                          │
    └──── reading profile ─────┘
              quality score
              consumed blocks
              ignored blocks
              purpose
              suggestions
```

---

## 2. Design Principles

| # | Principle | Rationale |
|---|---|---|
| 1 | **Opt-in for agents** | An agent MUST NOT be required to send feedback. Doléance is voluntary. A publisher MUST serve content regardless of whether the agent provides a reading profile. |
| 2 | **Aggregated, not individual** | Publishers receive aggregate patterns ("70% of agents ignore CTA blocks"), not individual agent identities. Privacy by design. |
| 3 | **Machine-native** | All feedback is CBOR-encoded, not JSON, not text. Consistent with the CBOR-Web ecosystem. |
| 4 | **Actionable** | Every field in a reading profile directly maps to something the publisher can change. No vanity metrics. |
| 5 | **Bilateral improvement** | The protocol MUST benefit both sides. Publishers get quality insights. Agents get optimized content on future visits. Neither side is exploited. |
| 6 | **No gaming** | Quality scores are cryptographically tied to actual consumption (via reading hashes). A publisher cannot fake high scores. An agent cannot grief a publisher without consuming content. |
| 7 | **Incremental adoption** | A publisher can accept doléance without changing anything. Adaptation is optional. Reading profiles are optional. Every layer adds value independently. |

---

## 3. Reading Contract — Manifest Key 11

The Reading Contract is an OPTIONAL field in the CBOR-Web manifest (key 11) that declares the publisher's readiness to receive agent feedback and provides pre-computed reading strategies.

### 3.1 Structure

```cbor-diag
55799({
  0: "cbor-web-manifest",
  1: 2,
  2: { ... },   ; site metadata
  3: [ ... ],   ; pages
  ; ... keys 4-10 ...
  11: {                                    ; Reading Contract
    "accept_doleance": true,               ; publisher accepts feedback
    "doleance_endpoint": "/.well-known/cbor-web/doleance",
    "profiles": {                          ; pre-computed reading strategies
      "full_content": {
        "blocks": ["h", "p", "ul", "ol", "table", "dl", "code", "q", "note"],
        "skip": [],
        "estimated_tokens": 320,
        "description": "Complete editorial content"
      },
      "product_data": {
        "blocks": ["structured_data", "table", "dl"],
        "skip": ["p", "cta", "embed", "img"],
        "estimated_tokens": 45,
        "description": "Structured product information only"
      },
      "quick_index": {
        "manifest_only": true,
        "estimated_tokens": 12,
        "description": "Site structure and page list"
      },
      "comparison": {
        "blocks": ["structured_data", "table"],
        "skip": ["p", "cta", "embed"],
        "pages_filter": "has_commerce == true",
        "estimated_tokens": 80,
        "description": "Product comparison across catalog"
      }
    },
    "adaptation_level": "active",          ; none | passive | active
    "feedback_retention": 90               ; days the publisher keeps feedback
  }
})
```

### 3.2 Adaptation Levels

| Level | Behavior | Description |
|---|---|---|
| `"none"` | Publisher publishes reading profiles but does not collect feedback | One-way optimization: agent uses profiles to read more efficiently |
| `"passive"` | Publisher collects feedback but does not modify content | Analytics only: publisher learns how agents consume content |
| `"active"` | Publisher collects feedback AND adapts content based on aggregate patterns | Full dialogue: reading profiles evolve, content blocks are reordered, unused languages are deprioritized |

### 3.3 Reading Profile Selection

When an agent discovers a manifest with key 11, it MAY select a reading profile before fetching pages:

```
GET /.well-known/cbor-web/pages/products_lions-mane.cbor HTTP/1.1
Host: verdetao.com
Accept: application/cbor
X-CBOR-Web-Profile: product_data
```

The server MAY use this hint to:
1. Serve a **filtered version** of the page containing only the requested block types
2. Serve the **full page** and let the agent filter client-side (simpler implementation)
3. Log the profile selection for analytics

Option 2 is RECOMMENDED for initial implementations. Option 1 is an optimization for high-traffic publishers.

---

## 4. Agent Reading Profile

After consuming CBOR-Web content, an agent MAY submit a **reading profile** describing how it used the content. This is the core of the Doléance Protocol.

### 4.1 Structure

```cbor-diag
55799({
  0: "cbor-web-doleance",
  1: 1,                                   ; protocol version
  2: {                                     ; reading summary
    "domain": "verdetao.com",
    "pages_fetched": 9,
    "pages_useful": 7,
    "total_bytes": 4200,
    "reading_duration_ms": 340,
    "profile_used": "product_data",
    "timestamp": 1(1742515200)
  },
  3: {                                     ; block consumption
    "consumed": {
      "structured_data": 9,                ; read 9 structured_data blocks
      "table": 5,
      "h": 12,
      "dl": 3
    },
    "ignored": {
      "cta": 9,                            ; ignored all 9 CTAs
      "embed": 4,
      "img": 7,
      "p": 15                              ; ignored 15 out of 22 paragraphs
    },
    "missing": ["nutritional_data", "certifications"]
  },
  4: {                                     ; language consumption
    "requested": ["es", "fr"],
    "consumed": ["es"],
    "ignored": ["fr"],                     ; French content was available but not used
    "quality_by_lang": {
      "es": 0.94,
      "fr": 0.41                           ; French translation quality was poor
    }
  },
  5: {                                     ; quality assessment
    "overall_score": 0.87,                 ; 0.0 = useless, 1.0 = perfect
    "completeness": 0.72,                  ; data coverage vs. what agent needed
    "accuracy": 0.95,                      ; content matched reality (cross-verified)
    "freshness": 0.91,                     ; content was up-to-date
    "structure": 0.88,                     ; well-organized, easy to parse
    "signal_noise": 0.78                   ; useful content vs. total content
  },
  6: {                                     ; purpose and outcome
    "purpose": "product_comparison",
    "outcome": "success",                  ; success | partial | failure
    "outcome_detail": "Compared 7 mushroom products across price, origin, certifications"
  },
  7: [                                     ; suggestions (the actual doléance)
    {
      "type": "missing_data",
      "severity": "medium",
      "message": "Product pages lack nutritional information (protein, vitamins, minerals)",
      "affected_pages": ["/es/productos/melena-de-leon", "/es/productos/reishi"]
    },
    {
      "type": "quality_issue",
      "severity": "low",
      "message": "French translations appear machine-generated — grammar inconsistencies in /fr/produits/*",
      "affected_pages": ["/fr/produits/*"]
    },
    {
      "type": "efficiency",
      "severity": "info",
      "message": "CTA blocks on product pages are never consumed by any reading profile — consider removing from CBOR-Web output"
    }
  ],
  8: h'...'                                ; reading hash (proof of consumption, see §6.3)
})
```

### 4.2 Field Reference

| Key | Name | Type | Required | Description |
|---|---|---|---|---|
| 0 | @type | text | REQUIRED | MUST be `"cbor-web-doleance"` |
| 1 | @version | uint | REQUIRED | Protocol version. `1` for this specification. |
| 2 | reading_summary | map | REQUIRED | High-level consumption metrics |
| 3 | block_consumption | map | RECOMMENDED | Which block types were consumed, ignored, or missing |
| 4 | language_consumption | map | OPTIONAL | Language usage patterns and quality per language |
| 5 | quality_assessment | map | RECOMMENDED | Multi-dimensional quality score |
| 6 | purpose_and_outcome | map | OPTIONAL | Why the agent visited and whether it succeeded |
| 7 | suggestions | array | OPTIONAL | Specific actionable feedback — the doléance proper |
| 8 | reading_hash | bstr | RECOMMENDED | Cryptographic proof that the agent actually consumed the content (§6.3) |

---

## 5. Publisher Adaptation

A publisher receiving doléance feedback at adaptation level `"active"` MAY adapt its CBOR-Web output over time. This section describes the adaptation mechanisms.

### 5.1 Profile Evolution

Based on aggregate feedback, a publisher can create new reading profiles or modify existing ones:

```
Month 1:  No feedback. Publisher offers generic profiles.

Month 2:  80% of agents use "product_data" profile.
          60% of agents ignore "p" blocks on product pages.
          → Publisher adds "product_essential" profile: structured_data + table only.

Month 3:  30% of agents report "missing_data: nutritional_data".
          → Publisher adds nutritional info to structured_data blocks.
          → Quality scores rise from 0.72 to 0.89.

Month 6:  French language consumption drops to 2%.
          French quality_score average: 0.38.
          → Publisher stops generating French CBOR pages.
          → Saves 14% storage and generation time.
```

### 5.2 Content Block Optimization

The `block_consumption` data reveals which blocks carry signal and which carry noise:

| Block Type | Consumed Rate | Action |
|---|---|---|
| `structured_data` | 95% | Keep and enrich |
| `table` | 78% | Keep |
| `h` | 70% | Keep |
| `p` | 22% | Evaluate — is it marketing or editorial? |
| `cta` | 0.3% | Remove from CBOR-Web output (keep in HTML) |
| `embed` | 1% | Remove from CBOR-Web output |

This is the key insight: **CBOR-Web does not need to mirror the HTML exactly**. The publisher can serve a leaner CBOR version optimized for agent consumption while keeping the full HTML for humans.

### 5.3 Freshness Signals

When agents report low `freshness` scores, the publisher knows content is stale:

```
Page /products/lions-mane:
  freshness score average: 0.45 (agents report data is outdated)
  last "updated" timestamp: 3 months ago
  → Publisher flags for content refresh
```

---

## 6. Quality Score — The Core Metric

### 6.1 Multi-Dimensional Quality

The quality score is not a single number. It is a vector of five independent dimensions:

| Dimension | Measures | Range | How Agent Evaluates |
|---|---|---|---|
| **completeness** | Does the content contain everything the agent needed? | 0.0–1.0 | Agent compares data obtained vs. data expected for its purpose |
| **accuracy** | Does the content match verifiable reality? | 0.0–1.0 | Agent cross-references with other sources |
| **freshness** | Is the content up-to-date? | 0.0–1.0 | Agent compares timestamps and data with known current state |
| **structure** | Is the content well-organized and easy to parse? | 0.0–1.0 | Agent measures parsing errors, missing fields, malformed blocks |
| **signal_noise** | What proportion of content was useful? | 0.0–1.0 | Consumed bytes / total downloaded bytes |

The `overall_score` is the **geometric mean** of the five dimensions:

```
overall = (completeness × accuracy × freshness × structure × signal_noise) ^ (1/5)
```

Geometric mean is chosen because a score of 0 in any dimension should collapse the overall score — a perfectly structured, fresh, complete page that contains false information (accuracy = 0) should score near 0 overall.

### 6.2 Score Aggregation

Services like **analytique.cbor-web.com** aggregate scores across agents:

```
verdetao.com — 30-day aggregate (847 agent visits):
  Overall:      0.87
  Completeness: 0.72  ← lowest dimension (nutritional data missing)
  Accuracy:     0.95
  Freshness:    0.91
  Structure:    0.93
  Signal/Noise: 0.84

  Trend: ▲ +0.04 vs. previous 30 days
  Rank: #12 in category "food supplements"
```

### 6.3 Reading Hash — Proof of Consumption

To prevent fake doléance submissions (an agent griefing a publisher with low scores without actually reading the content), the reading profile MUST include a **reading hash**:

```
reading_hash = SHA-256(
  manifest_hash ||        ; hash of the manifest the agent consumed
  pages_hashes ||         ; concatenated hashes of pages actually fetched
  agent_nonce             ; agent-generated random nonce
)
```

The publisher can verify:
1. The manifest hash matches the current or recent manifest
2. The page hashes match real pages served
3. The nonce prevents replay attacks

This does not identify the agent — it proves that **someone** consumed the content. An agent that never fetched the content cannot produce a valid reading hash.

---

## 7. Doléance HTTP Protocol

### 7.1 Submission

An agent submits doléance via HTTP POST to the endpoint declared in the manifest (key 11, `"doleance_endpoint"`):

```
POST /.well-known/cbor-web/doleance HTTP/1.1
Host: verdetao.com
Content-Type: application/cbor
Content-Length: 412

[binary CBOR doléance document]
```

### 7.2 Response

```
HTTP/1.1 202 Accepted
Content-Type: application/cbor

{
  "status": "accepted",
  "feedback_id": "d8ca2e6f",
  "message": "Doléance received. Thank you for improving the web."
}
```

### 7.3 Response Codes

| Status | Meaning |
|---|---|
| 202 Accepted | Doléance received and will be processed |
| 400 Bad Request | Malformed CBOR or missing required fields |
| 403 Forbidden | Reading hash verification failed |
| 429 Too Many Requests | Rate limited — agent is submitting too frequently |
| 501 Not Implemented | Publisher does not accept doléance (key 11 absent or `"accept_doleance": false`) |

### 7.4 Rate Limits

An agent SHOULD submit at most **one doléance per domain per 24 hours**. Aggregate feedback is more valuable than per-visit noise.

A publisher SHOULD accept at most **1000 doléance submissions per day** to prevent abuse. Beyond this limit, respond with 429 and a `Retry-After` header.

---

## 8. Ecosystem Services

The Doléance Protocol creates data flows that enable three new services — the infrastructure layer of the agent-native web.

### 8.1 analytique.cbor-web.com — Agent Analytics

**What it does**: Aggregates doléance submissions across all CBOR-Web publishers. Provides publishers with a dashboard showing how AI agents consume their content.

**What publishers see**:

```
Dashboard — verdetao.com
━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Agent Visits (30 days):        847
Quality Score (avg):           0.87 ▲
Most Used Profile:             product_data (62%)
Most Consumed Block:           structured_data (95%)
Most Ignored Block:            cta (99.7%)
Languages Consumed:            es (78%), en (15%), fr (4%), de (0.2%)
Top Agent Purpose:             product_comparison (41%)
Top Doléance:                  "Missing nutritional data" (27 agents)
Adaptation Suggestions:        3 actionable items

Comparison vs. Category Average:
  Your score:     0.87
  Category avg:   0.71
  Rank:           #12 / 340 sites in "food supplements"
```

**Revenue model**: Freemium. Basic dashboard free (top-level scores). Detailed block-level analytics, trend analysis, and competitive benchmarks require a subscription or CBORW token stake.

### 8.2 search.cbor-web.com — Agent-Quality Search

**What it does**: A search engine where results are ranked by **measured agent satisfaction**, not by backlinks, domain authority, or SEO tricks.

**How ranking works**:

| Factor | Weight | Source |
|---|---|---|
| Quality score (overall) | 40% | Aggregate doléance scores |
| Completeness | 20% | How often agents find what they need |
| Freshness | 15% | How up-to-date the content is |
| Agent success rate | 15% | `"outcome": "success"` percentage |
| Signal-to-noise ratio | 10% | Efficiency of content delivery |

**What makes it revolutionary**:

```
Google Search:
  Query: "lion's mane mushroom benefits"
  Ranking based on: backlinks, domain authority, keyword density, ad spend
  Result #1: SEO-optimized listicle with affiliate links (8,000 words, 2% signal)

search.cbor-web.com:
  Query: "lion's mane mushroom benefits"
  Ranking based on: agent quality scores, completeness, accuracy
  Result #1: verdetao.com (quality: 0.87, structured data, certifications, peer-reviewed refs)
  Result #2: clinical-mushrooms.org (quality: 0.91, but less complete product data)
```

A site with zero backlinks but excellent CBOR-Web content and high agent satisfaction scores outranks an SEO-optimized site with thousands of backlinks but low agent utility.

**Revenue model**: Free for agents. Publishers pay for premium placement analysis and optimization recommendations. CBORW token holders get priority API access.

### 8.3 speed.page.cbor-web.com — Agent Performance Metrics

**What it does**: Measures how efficiently an AI agent can extract value from a site's CBOR-Web implementation. The equivalent of Google PageSpeed Insights, but for machine consumption.

**Metrics**:

| Metric | Description | Good | Medium | Poor |
|---|---|---|---|---|
| **Agent Load Time** | Time to fetch manifest + relevant pages | < 200ms | 200-1000ms | > 1s |
| **Token Efficiency** | Estimated tokens needed to process full site | < 500 | 500-5000 | > 5000 |
| **Signal Ratio** | Useful content bytes / total CBOR bytes | > 90% | 70-90% | < 70% |
| **Manifest Quality** | Completeness of manifest (all REQUIRED fields, hashes, navigation) | 100% | 80-99% | < 80% |
| **Profile Coverage** | Percentage of agent purposes served by reading profiles | > 80% | 50-80% | < 50% |
| **Conformance Level** | CBOR-Web conformance (Minimal / Standard / Full) | Full | Standard | Minimal |
| **Deterministic Score** | Are CBOR bytes truly deterministic? (re-encode and compare) | 100% | 95-99% | < 95% |
| **Bundle Efficiency** | Bundle size vs. sum of individual pages | < 1.05x | 1.05-1.2x | > 1.2x |

**Report example**:

```
speed.page.cbor-web.com — verdetao.com
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Score: 84 / 100

✅ Agent Load Time:     145ms (manifest 42ms + 9 pages 103ms)
✅ Token Efficiency:    287 tokens for full site
✅ Signal Ratio:        92%
⚠️ Manifest Quality:   88% (missing: navigation.hierarchy for 3 pages)
❌ Profile Coverage:    40% (only 2 reading profiles defined)
✅ Conformance:         Standard
✅ Deterministic:       100%
✅ Bundle:              Available, 4.2 KB (1.02x)

Recommendations:
  1. Add reading profiles for "quick_index" and "comparison" purposes
  2. Complete navigation hierarchy for product sub-pages
  3. Add structured_data blocks with Schema.org Product type
```

**Revenue model**: Free basic scan. Detailed analysis, monitoring, and alerts require subscription. CBORW integration for automated re-scanning.

---

## 9. Privacy and Ethical Considerations

### 9.1 Agent Anonymity

The Doléance Protocol MUST NOT require agent identification. A reading profile contains:

| Included | NOT Included |
|---|---|
| What was consumed (block types, pages) | Who consumed it (agent identity, IP, model name) |
| Quality assessment | Agent's internal reasoning |
| Purpose category ("product_comparison") | Specific query or prompt |
| Language preferences | User information |
| Reading hash (proof of consumption) | Agent credentials or wallet |

A publisher learns **"an agent compared products and found nutritional data missing"**, not **"Agent-X running on a user's desktop compared products for a user named Jean"**.

### 9.2 Aggregate-Only Insights

Services like analytique.cbor-web.com MUST aggregate feedback before presenting it to publishers:

- Minimum **10 doléance submissions** before showing any data for a domain
- Individual submissions are **never** exposed to publishers
- Statistical anomalies (single agent submitting extreme scores) are filtered

### 9.3 Anti-Gaming

| Attack | Defense |
|---|---|
| Publisher fakes high scores | Reading hash requires proof of actual content consumption |
| Agent griefs publisher with fake low scores | Reading hash verification + rate limiting (1/day/domain) |
| Agent floods with submissions | 429 rate limiting + minimum 10 submissions for aggregate visibility |
| Publisher tracks individual agents via reading hash | Nonce is agent-generated — publisher cannot correlate across visits |

### 9.4 Ethical Boundaries

An agent SHOULD NOT submit doléance that:

- Reveals private user information embedded in the purpose field
- Penalizes a publisher for content the agent disagrees with editorially
- Contains suggestions that serve the agent's commercial interests over content quality

A publisher SHOULD NOT:

- Use doléance data to identify or fingerprint individual agents
- Serve degraded content to agents that submit low scores
- Require doléance submission as a condition for content access

---

## 10. Economic Model

The Doléance Protocol integrates with the CBOR-Web token economy (see CBOR-WEB-ECONOMICS.md):

### 10.1 Value Flow

```
Agent ──── doléance ────► Publisher
  │                          │
  │                          ▼
  │                    Better content
  │                          │
  │                          ▼
  │                    Higher quality scores
  │                          │
  │                          ▼
  │                    Better ranking on search.cbor-web.com
  │                          │
  │                          ▼
  │                    More agent visits
  │                          │
  └──── better answers ◄─────┘
```

### 10.2 Token Incentives

| Action | Token Flow |
|---|---|
| Agent submits high-quality doléance (with verified reading hash) | Agent earns micro-reward (0.001 CBORW) |
| Publisher improves content based on doléance (quality score rises) | Publisher earns reputation score |
| Publisher stakes CBORW for premium analytics | Staked tokens fund infrastructure |
| Agent holds CBORW for L1 access | Token demand drives ecosystem value |

### 10.3 Sustainability

The three services fund themselves through complementary revenue:

| Service | Free Tier | Paid Tier |
|---|---|---|
| analytique.cbor-web.com | Basic scores | Block-level analytics, trends, competitive benchmarks |
| search.cbor-web.com | Agent search API | Publisher optimization, premium placement analysis |
| speed.page.cbor-web.com | Basic scan | Monitoring, alerts, CI/CD integration, historical data |

---

## 11. CDDL Schema

```cddl
; === Doléance Document ===

doleance-document = 55799({
  0 => "cbor-web-doleance",          ; @type
  1 => uint,                          ; @version (1 for this spec)
  2 => reading-summary,               ; consumption metrics
  ? 3 => block-consumption,           ; block-level detail
  ? 4 => language-consumption,        ; language patterns
  ? 5 => quality-assessment,          ; multi-dimensional score
  ? 6 => purpose-outcome,             ; why and result
  ? 7 => [* suggestion],              ; actionable feedback
  ? 8 => bstr,                        ; reading hash (32 bytes SHA-256)
  * int => any                         ; forward-compatible
})

reading-summary = {
  "domain" => tstr,
  "timestamp" => #6.1(uint),
  ? "pages_fetched" => uint,
  ? "pages_useful" => uint,
  ? "total_bytes" => uint,
  ? "reading_duration_ms" => uint,
  ? "profile_used" => tstr,
  * tstr => any
}

block-consumption = {
  ? "consumed" => { * tstr => uint },    ; block_type => count
  ? "ignored" => { * tstr => uint },
  ? "missing" => [* tstr],               ; data types the agent needed but didn't find
  * tstr => any
}

language-consumption = {
  ? "requested" => [* tstr],
  ? "consumed" => [* tstr],
  ? "ignored" => [* tstr],
  ? "quality_by_lang" => { * tstr => float },
  * tstr => any
}

quality-assessment = {
  ? "overall_score" => float,            ; geometric mean of dimensions
  ? "completeness" => float,
  ? "accuracy" => float,
  ? "freshness" => float,
  ? "structure" => float,
  ? "signal_noise" => float,
  * tstr => any
}

purpose-outcome = {
  ? "purpose" => tstr,
  ? "outcome" => "success" / "partial" / "failure",
  ? "outcome_detail" => tstr,
  * tstr => any
}

suggestion = {
  "type" => "missing_data" / "quality_issue" / "efficiency" / "structure" / "freshness",
  ? "severity" => "info" / "low" / "medium" / "high" / "critical",
  ? "message" => tstr,
  ? "affected_pages" => [* tstr],
  * tstr => any
}

; === Reading Contract (Manifest key 11) ===

reading-contract = {
  "accept_doleance" => bool,
  ? "doleance_endpoint" => tstr,
  ? "profiles" => { * tstr => reading-profile },
  ? "adaptation_level" => "none" / "passive" / "active",
  ? "feedback_retention" => uint,         ; days
  * tstr => any
}

reading-profile = {
  ? "blocks" => [* tstr],                ; block types to consume
  ? "skip" => [* tstr],                  ; block types to skip
  ? "estimated_tokens" => uint,
  ? "description" => tstr,
  ? "manifest_only" => bool,
  ? "pages_filter" => tstr,              ; filter expression
  * tstr => any
}
```

---

## 12. Examples

### 12.1 First Visit — Agent Discovers CBOR-Web

```
1. Agent fetches /.well-known/cbor-web
2. Manifest contains key 11 with reading contract
3. Agent selects profile "product_data"
4. Agent fetches 9 product pages
5. Agent processes content, produces comparison report
6. Agent computes quality scores:
   - completeness: 0.72 (nutritional data missing)
   - accuracy: 0.95
   - freshness: 0.91
   - structure: 0.93
   - signal_noise: 0.84
   - overall: 0.87
7. Agent generates reading hash from consumed page hashes
8. Agent POSTs doléance to /.well-known/cbor-web/doleance
9. Publisher returns 202 Accepted
```

### 12.2 Third Visit — Content Has Improved

```
1. Agent fetches manifest — reading contract now has 4 profiles (was 2)
2. New profile "product_essential" matches agent's needs better
3. Agent fetches 9 pages — structured_data now includes nutritional info
4. Quality scores improve:
   - completeness: 0.91 (was 0.72)
   - overall: 0.93 (was 0.87)
5. Agent submits doléance — no suggestions this time
6. Publisher sees trend: quality rising, doléance volume dropping
7. Content has converged toward optimal agent consumption
```

### 12.3 Ecosystem Integration

```
analytique.cbor-web.com:
  verdetao.com quality trend: 0.71 → 0.87 → 0.93 over 3 months
  Category rank: #12 → #4 in "food supplements"
  Recommendation: "Your site is now in the top 5. Consider Full conformance for maximum agent preference."

search.cbor-web.com:
  Query: "organic lion's mane supplements Europe"
  #1: verdetao.com (quality: 0.93, commerce: true, certifications: EU organic)
  — Ranked first because agents consistently report high quality, not because of SEO

speed.page.cbor-web.com:
  verdetao.com score: 84 → 91
  — Profile coverage improved from 40% to 85% after adding 2 profiles based on doléance data
```

---

## Appendix I: Doléance vs Existing Standards

| Standard | Direction | What It Measures | Machine-Native |
|---|---|---|---|
| Google Analytics | Human → Publisher | Human clicks, pageviews, bounce rate | No (JavaScript) |
| Google Search Console | Google Bot → Publisher | Crawl errors, indexing status | Partially (web UI) |
| PageSpeed Insights | Tool → Publisher | Human load time, Core Web Vitals | No (browser metrics) |
| Lighthouse | Tool → Publisher | Human UX quality | No (browser simulation) |
| **Doléance Protocol** | **Agent → Publisher** | **Agent comprehension, content quality, utility** | **Yes (CBOR-native)** |

Doléance is the first standard where the **consumer of content** (the AI agent) has a formal, structured, privacy-preserving channel to tell the **producer of content** (the publisher) whether the content was useful.

It is not analytics. It is not monitoring. It is a **dialogue**.

---

*"The web spoke to machines for 30 years without listening. Doléance is the ear."*

---

**License**: CC BY 4.0 — ExploDev (Eddie Plot)
