# CBOR-Web

**The binary protocol that replaces HTML for AI agents.**

Created by [Eddie Plot](https://deltopide.fr) & [Claude](https://claude.ai) — [Deltopide](https://deltopide.com)

```
fleurs.com/index.html  →  for browsers
fleurs.com/index.cbor  →  for AI agents (entire site in one request)
```

One file. One request. Every page, every product, every price — structured, signed, verified.

---

## Why

HTML was built for humans in 1993. AI agents waste **hundreds of TWh/year** parsing it — stripping ads, cookies banners, navigation menus — just to find the actual content.

CBOR-Web is a binary format (RFC 8949) purpose-built for machines. An AI agent fetches `index.cbor` and gets the **entire site** in a single structured request. No parsing. No guessing. No wasted energy.

| | HTML | CBOR-Web |
|---|---|---|
| **Format** | Text (~100 KB/page) | Binary (~2 KB/page) |
| **Requests per site** | 50-200 | **1** |
| **Structure** | CSS selectors, guesswork | Native typed blocks |
| **Integrity** | None | SHA-256 per page |
| **Identity** | SSL certificate | DNS + Ed25519 signature |
| **Multilingual** | hreflang (often broken) | `"alternates": {"en": "/en/page"}` |
| **Access control** | robots.txt (honor system) | Cryptographic tiers (T0/T1/T2) |

---

## 30-second example

```cbor-diag
55799({
  0: "cbor-web",
  1: 3,
  2: {
    "domain": "fleurs.com",
    "name": "Fleurs.com — Flower delivery",
    "languages": ["fr", "en"],
    "default_language": "fr"
  },
  5: [
    {
      "path": "/roses",
      "title": "Our Roses",
      "lang": "fr",
      "access": "T2",
      "priority": 0.9,
      "freshness": "daily",
      "updated": 1(1742428800),
      "content": [
        {"l": 1, "t": "h", "v": "Our Roses"},
        {"t": "p", "v": "Grown in France, picked fresh every morning."},
        {"t": "table", "headers": ["Variety", "Price", "Stock"], "rows": [
          ["Classic Red Rose", "2.90 EUR", "In stock"],
          ["White Rose", "3.50 EUR", "In stock"],
          ["Rainbow Rose", "4.90 EUR", "On order"]
        ]}
      ],
      "structured_data": {
        "type": "Product",
        "offers": {"lowPrice": 2.90, "highPrice": 4.90, "priceCurrency": "EUR"}
      }
    }
  ]
})
```

An AI agent reads this and instantly knows: what the site sells, at what price, in what language, and when it was last updated. No DOM parsing. No CSS selectors. No hallucination.

---

## Spec

| Document | Description |
|----------|-------------|
| **[CBOR-WEB-SPEC-v3.0.md](CBOR-WEB-SPEC-v3.0.md)** | The specification — start here |
| [CBOR-WEB-SECURITY.md](CBOR-WEB-SECURITY.md) | Security architecture (DNS identity, COSE signatures, access tiers) |
| [CBOR-WEB-ECONOMICS.md](CBOR-WEB-ECONOMICS.md) | Token economy (CBORW ERC-20, pricing, allocation) |
| [CBOR-WEB-MULTIMEDIA.md](CBOR-WEB-MULTIMEDIA.md) | Multimedia blocks (image, video, audio, live stream) |
| [CBOR-WEB-GENERATIVE.md](CBOR-WEB-GENERATIVE.md) | Generative blocks (forms, products, cart, API endpoints) |

---

## Visibility options — why merchants adopt

CBOR-Web doesn't just structure content. It gives publishers **control over how AI agents prioritize their pages**.

| Feature | Free | Publisher (paid token) |
|---------|------|----------------------|
| `"priority"` | 0.5 | 0.0 — 1.0 (you choose) |
| `"freshness"` | monthly | realtime — monthly |
| `"boost"` | — | Temporary page promotion with expiry |
| `"alternates"` | 1 language | Unlimited |

Unlike `sitemap.xml` where `<priority>` is ignored by Google, **CBOR-Web crawlers respect these signals**. The publisher has verified their identity via DNS and signed their file — the protocol rewards that investment.

> CBOR-Web doesn't sell advertising. It sells **machine readability**.

---

## Tools

### cbor-crawl (Rust)

AI-side crawler that fetches and processes `index.cbor` files.

```bash
cargo build --release
./target/release/cbor-crawl https://fleurs.com/index.cbor
```

### text2cbor (Rust)

Publisher-side tool that converts an HTML website to `index.cbor`.

```bash
cd tools/text2cbor
cargo build --release
./target/release/text2cbor --url https://fleurs.com --output index.cbor
```

---

## Test vectors

Pre-built CBOR files for validator testing:

| File | Content |
|------|---------|
| `vectors/tv1_manifest.cbor` | Site manifest with identity and navigation |
| `vectors/tv2_page.cbor` | Single page with content blocks |
| `vectors/tv3_product.cbor` | Product page with structured data |

---

## How it works

```
Publisher                          AI Agent
   |                                  |
   |  1. Create DNS TXT record        |
   |     _cbor-web.fleurs.com         |
   |                                  |
   |  2. Generate index.cbor          |
   |     (via cbor-web.com API        |
   |      or text2cbor CLI)           |
   |                                  |
   |  3. Serve at /index.cbor         |
   |                                  |
   |         GET /index.cbor     <----|
   |                                  |
   |----> 200 OK (application/cbor)   |
   |      Entire site in 1 request    |
   |                                  |
   |      4. Agent verifies:          |
   |         - Tag 55799              |
   |         - DNS signature          |
   |         - Content hashes         |
   |         - Access tier            |
   |                                  |
   |      5. Agent processes pages    |
   |         by priority, freshness   |
```

---

## Positioning

| Standard | What it does | CBOR-Web |
|----------|-------------|----------|
| `robots.txt` | Crawl rules | Complementary |
| `sitemap.xml` | URL list | **Replaced** by `index.cbor` |
| `llms.txt` | Text summary for LLMs | Complementary |
| `index.html` | Human-readable page | Parallel — `index.cbor` is for machines |

---

## Economy

The protocol is free (CC BY-ND 4.0). Revenue comes from:

- **Publisher tokens** — annual subscription for generation service + visibility options
- **CBORW token (ERC-20)** — permanent T1 access badge for AI agents
- **Boost** — temporary page promotion (product launches, seasonal sales)
- **On-demand generation** — extra regenerations beyond free quota

---

## The math

A single HTML page: ~100 KB of markup, ads, scripts, styles.
The same content in CBOR-Web: ~2 KB of structured binary.

**50x smaller. 1 request instead of 200. Zero parsing.**

If 1% of the world's AI traffic switches to CBOR-Web, that's **~40 TWh/year saved** — equivalent to the annual electricity consumption of Denmark.

---

## Status

```
Spec v3.0:      Draft (March 2026)
text2cbor:      v0.1.0 (working, needs v3.0 adaptation)
cbor-crawl:     v0.1.0 (working)
Token:          Not yet deployed
Website:        cbor-web.org / cbor-web.com (coming soon)
```

---

## License

- Specification: **CC BY-ND 4.0** (open, free to use, no derivatives without permission)
- Tools (text2cbor, cbor-crawl): **MIT**
- Trademark "CBOR-Web": registration pending (INPI + OEPM)

---

*CBOR-Web — the entire site in one file.*

*Deltopide 2026*
