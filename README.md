# CBOR-Web

[![License: CC BY 4.0](https://img.shields.io/badge/License-CC%20BY%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by/4.0/)
[![Version](https://img.shields.io/badge/Version-v2.1.3-blue.svg)]()
[![CI](https://img.shields.io/badge/CI-passing-brightgreen.svg)]()
[![Tests](https://img.shields.io/badge/tests-29%20passed-brightgreen.svg)]()
[![Status](https://img.shields.io/badge/status-production%20%7C%2038%20sites-brightgreen.svg)]()

**Binary Web Content for Autonomous AI Agents — CBOR (RFC 8949)**

## What Is CBOR-Web?

CBOR-Web is a binary format standard that lets websites expose a machine-native copy of their content alongside HTML. AI agents consume structured content directly — no HTML parsing, no DOM traversal, no token waste.

A real-world benchmark on a 49-page site: **1.63 MB HTML → 878 KB CBOR bundle** (1.8:1 compression, 45% smaller). Single-page content achieves 10:1+.

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
| 49-page site | ~1.6 MB (multiple requests) | ~878 KB (one bundle) |
| Incremental updates | Full re-crawl | SHA-256 diff |

## Key Features

- **Binary CBOR format** (RFC 8949) — compact, typed, deterministic encoding
- **Single-request indexing** — bundle endpoint delivers entire site in one request
- **SHA-256 incremental updates** — only re-download pages that changed
- **13+ content block types** — h, p, ul, ol, q, code, table, img, cta, embed, sep, dl, note
- **Schema.org structured data** — native CBOR, not JSON-LD-inside-script-inside-HTML
- **Forward compatible** — unknown keys are ignored, not errors
- **Doléance Protocol** — agents feed back content quality to publishers
- **Access control** — token-based (X-CBOR-Web-Wallet header)

## Tools

| Tool | Language | Purpose |
|------|----------|---------|
| [`tools/text2cbor`](tools/text2cbor/) | Rust | Convert HTML websites → CBOR-Web (manifest, bundle, pages) |
| [`tools/cbor-crawl`](tools/cbor-crawl/) | Rust | AI agent crawler: discover, fetch, search, diff, send doléance |
| [`tools/cbor-server`](tools/cbor-server/) | Rust | Reference HTTP server: well-known endpoints, doléance receiver, rate limiting |
| [`tools/cbor-vectors`](tools/cbor-vectors/) | Rust | Generate binary test vectors |
| [`clients/python/cborweb`](clients/python/) | Python | Zero-dependency AI agent client (manifest, bundle, search, feedback) |
| [`tools/cbor-server/worker.js`](tools/cbor-server/worker.js) | JS | Cloudflare Worker for edge distribution + R2 |

## Quick Start

```bash
git clone https://github.com/ploteddie-bit/cbor-web.git
cd cbor-web

# Generate CBOR from HTML
cd tools/text2cbor && cargo build --release
./target/release/text2cbor generate \
  -i /path/to/html/site -o /tmp/output -d example.com \
  --name "My Site" --default-lang en --spec-version 2.1

# Serve it
cd ../cbor-server && ./bootstrap.sh /tmp/output
cargo run --release -- --data data --listen 0.0.0.0:3001
# → http://localhost:3001/.well-known/cbor-web

# Crawl it
cd ../cbor-crawl && cargo build --release
./target/release/cbor-crawl inspect https://cbor.deltopide.com

# Python client (AI agents)
cd ../../clients/python
python3 -c "
from cborweb import CBORWebClient
client = CBORWebClient('cbor.deltopide.com')
manifest = client.manifest()
print(f'{manifest[\"site_name\"]} — {manifest[\"pages_count\"]} pages')
"
```

## Specification Documents (7-part suite)

| # | Document | Version | Description |
|---|----------|---------|-------------|
| 1 | [CBOR-WEB-CORE.md](CBOR-WEB-CORE.md) | v2.1.3 | Core format: manifest, page, bundle, content blocks, CDDL |
| 2 | [CBOR-WEB-MULTIMEDIA.md](CBOR-WEB-MULTIMEDIA.md) | v2.1 | Image, audio, video, streaming channels |
| 3 | [CBOR-WEB-GENERATIVE.md](CBOR-WEB-GENERATIVE.md) | v2.1 | Generative blocks, forms, commerce |
| 4 | [CBOR-WEB-SECURITY.md](CBOR-WEB-SECURITY.md) | v2.1.1 | COSE signatures, access control, rate limiting |
| 5 | [CBOR-WEB-ECONOMICS.md](CBOR-WEB-ECONOMICS.md) | v2.1 | Token economics, smart contracts |
| 6 | [CBOR-WEB-REFERENCE.md](CBOR-WEB-REFERENCE.md) | v2.1 | Unified CDDL, test vectors, glossary |
| 7 | [CBOR-WEB-DOLEANCE.md](CBOR-WEB-DOLEANCE.md) | v1.0 | Agent feedback protocol |

## Production Deployment

**Live at:** [`https://cbor.deltopide.com`](https://cbor.deltopide.com) — 38 CBOR-Web enabled sites.

**Edge proxy:** [`https://cbor-web.explodev.workers.dev`](https://cbor-web.explodev.workers.dev) — Cloudflare Worker global CDN.

```
Browser / AI Agent
       │
       ▼
 Cloudflare Worker (cbor-web.explodev.workers.dev)
       │  Short codes: /lfr/ /dtp/ /cbw/ ... (3-letter domain alias)
       │  Full path:   /s/<domain>/...              (any domain)
       │
       ▼
 Cloudflare Tunnel → serveur-dev:3001 (Rust/axum)
       │
       ├─ sites/deltopide.com/    (49 pages, full spec)
       ├─ sites/cbor-web.com/     (8 pages, full spec)
       ├─ sites/laforetnousregale.fr/  (2 pages)
       ├─ sites/verdetao.fr/      (index.cbor)
       └─ ... 34 more showcase sites
```

### Endpoints

#### Edge Worker (Cloudflare CDN — any domain reachable)
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/<code>/pages/:file` | Short-code access (e.g. `/lfr/pages/accueil.cbor`) |
| `GET` | `/<code>/` | Manifest via short code |
| `GET` | `/<code>/bundle` | Bundle via short code |
| `GET` | `/s/<domain>/pages/:file` | Full domain path access |
| `GET` | `/health` | Health check |
| `GET` | `/diff?base=HASH` | Incremental diff |

#### Origin Server (direct)
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | HTML dashboard listing all 38 sites |
| `GET` | `/health` | JSON health check |
| `GET` | `/.well-known/cbor-web` | Manifest (multi-domain via Host header or X-CBOR-Domain) |
| `GET` | `/.well-known/cbor-web/bundle` | Full site bundle |
| `GET` | `/.well-known/cbor-web/pages/:file` | Individual pages |
| `POST` | `/.well-known/cbor-web/doleance` | Agent feedback (persisted to disk) |
| `GET` | `/.well-known/cbor-web/doleance/list` | Collected feedback |
| `GET` | `/.well-known/cbor-web/diff?base=HASH` | Incremental diff |

### Domain Routing

The server supports two methods to serve per-domain content:

1. **Host header** (direct access) — `Host: deltopide.com` → serves from `sites/deltopide.com/`. Includes `www.` stripping and subdomain fallback (`cbor.x.com` → `x.com`).

2. **X-CBOR-Domain header** (edge proxy) — `X-CBOR-Domain: laforetnousregale.fr` → serves from `sites/laforetnousregale.fr/`. Used by the Worker for path-based and short-code access when DNS isn't available.

### Short Code Reference

The Worker maps 3-letter codes to domain names for concise URLs:

| Code | Domain | Code | Domain |
|------|--------|------|--------|
| `lfr` | laforetnousregale.fr | `dtp` | deltopide.com |
| `cbw` | cbor-web.com | `cb2` | cborweb.com |
| `cbo` | cborweb.org | `cbs` | cbor-web.site |
| `cbt` | cbor-web.tech | `cbf` | cbor-web.fr |
| `edv` | explodev.com | `edf` | explodev.fr |
| `edo` | explodev.org | `eds` | explodev.site |
| `edt` | explodev.tech | `edw` | explodev.website |
| `vta` | verdetao.fr | `vtb` | verdetao.be |
| `vtd` | verdetao.de | `vte` | verdetao.eu |
| `vts` | verdetao.es | `cbd` | californiacbd.fr |
| `cbe` | californiacbd.es | `clc` | californialovecbd.es |
| `cls` | californialovecbd.site | `cle` | californialove.es |
| `mjc` | mariejeannecbd.fr | `mje` | mariejeannecbd.es |
| `fcc` | fanaticodelcbd.com | `fce` | fanaticodelcbd.es |
| `bcc` | bienestarcosmeticacbd.es | `bcf` | bienetrecosmetiquecbd.fr |
| `amz` | amazingcbd.es | `cas` | castelloconviu.es |
| `cgm` | cargamipatinete.es | `ptp` | preciotupatinete.es |
| `rtc` | ritueletcalme.com | `cau` | courtiers-auto.com |
| `dts` | deltopide.site | `wbc` | wellbeingcosmeticcbd.com |

Example:
```
https://cbor-web.explodev.workers.dev/lfr/pages/root.cbor
                                    ^^^  ^^^^^^^^^^^^^^^^^^^^
                                    code  CBOR-Web page path
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
| Agent feedback | No | No | No | No | **Doléance Protocol** |
| Reference server | — | — | — | — | **cbor-server (Rust)** |
| Python client | — | — | — | — | **cborweb (zero deps)** |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All Rust tools pass `cargo clippy -D warnings` and have test coverage.

## License

[Creative Commons Attribution 4.0 International (CC BY 4.0)](LICENSE)

---

*"The web has two clients: humans and machines. It's time to serve both."*
