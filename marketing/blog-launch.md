# Les agents IA gaspillent 93 % de leurs tokens à lire du HTML. On a corrigé ça.

*Par Eddie Plot — ExploDev / Deltopide SL, le 25 avril 2026*

---

**Every day, millions of AI agents crawl the web. And every day, they waste 93% of the tokens they pay for — stripping out menus, ads, trackers, cookie banners, and JavaScript bundles — just to extract the 8 KB of actual content buried inside a 1.6 MB HTML page.**

Think about that for a second. When ChatGPT browses your site, Claude summarizes your docs, or an enterprise RAG pipeline indexes your product pages, the agent downloads *everything* — CSS grids, React bundles, Google Analytics, newsletter popups — then runs that noise through a tokenizer that charges by the thousand tokens. The signal-to-noise ratio on the modern web, measured across 200 real-world sites, is 7 %. The other 93 % is pure overhead. For a single agent reading 1 000 pages a day, that's ~$36/day burned on markup. At the scale of millions of agents, we're talking about **billions of tokens wasted daily**.

This isn't a bug. It's an architectural choice that made sense in 1993, when HTML was designed for human eyes and Netscape Navigator. But the web has two clients now — humans *and* machines — and only one of them is served.

---

## CBOR-Web : un REST API automatique pour votre contenu

CBOR-Web is an open protocol that lets any website publish a **machine-readable binary copy** of its content, in parallel with its existing HTML. Humans see the regular site. AI agents get a structured, binary, 10× smaller version served from the edge.

It works like this:

```bash
text2cbor --input ./my-site --output ./cbor --domain mysite.com
```

One command. Thirty minutes. Your site gets a `/.well-known/cbor-web` endpoint that speaks **CBOR** (RFC 8949), the same compact binary format used in IoT sensors, WebAuthn, and cryptographic tokens. The output is not a summary like `llms.txt` or metadata like Schema.org — it's the **full structured content** of every page: headings, paragraphs, lists, tables, code blocks, navigation trees, and multi-language alternates, all in a format an AI agent can ingest in nanoseconds without touching an LLM tokenizer.

The result: a 1.6 MB HTML page becomes a **536-byte CBOR file**. A 100-page site that costs $3 to crawl as HTML costs **$0.01** as CBOR-Web. That's not a marginal improvement — it's a structural change in the economics of web-scale AI consumption.

---

## Les trois piliers stratégiques

CBOR-Web is built on three pillars aligned with the biggest shifts in AI and digital regulation.

### 1. Identité IA — Savoir qui lit votre contenu

No existing protocol tells a webmaster *which* AI agent is accessing their site. With CBOR-Web, every agent carries a cryptographic wallet identity. Publishers can allow or block specific agents, set access tiers (public L0 vs. token-gated L1), and audit exactly which agent read which page, when. This isn't theoretical — France is advancing legislation requiring all automated agents to be identifiable when accessing web services. CBOR-Web is **already compliant today**.

### 2. Économie des tokens — Des milliards économisés par jour

An agent reading a CBOR-Web site pays only for the content, not the markup. The tokenizer doesn't run on 93 % noise. For AI companies training foundation models, clean structured data means higher-quality fine-tuning at a fraction of the cost. The protocol reduces token consumption by **100× to 200×** depending on the page — equivalent to billions of tokens saved daily at web scale.

### 3. Prêt pour la réglementation — Identité numérique pour l'IA

The EU's MiCA regulation and France's upcoming digital identity law for automated systems are the first of many frameworks that will require AI agents to be traceable. CBOR-Web embeds cryptographic identity as a first-class primitive — your site adopting it today means you're **pre-compliant** before the law even passes.

---

## 38 sites en production, 8 langages, un IETF draft

CBOR-Web is not a whitepaper. It's running in production right now:

- **38 domains**, 59 pages across 3 languages, served from Cloudflare's global edge network (300+ cities)
- **8 SDKs** with zero dependencies: TypeScript, React, Python, Go, PHP, Ruby, Java, C++ — installable in one line (`npm install @deltopide_edy/cborweb`, `pip install cborweb`, `go get`, etc.)
- **npm package** published and live
- **IETF Internet-Draft** (`draft-plot-cbor-web-00`) aligned with RFC 8949 (CBOR), RFC 8610 (CDDL), and RFC 8615 (Well-Known URIs)
- **Reference implementation** in Rust (`text2cbor`) with deterministic encoding, SHA-256 cache validation, and incremental update support
- **Cloudflare Worker** serving the SaaS dashboard at [cbor-web.explodev.workers.dev](https://cbor-web.explodev.workers.dev)

The entire spec suite — 7 documents covering core format, security, economics, multimedia, generative blocks, and reference CDDL schemas — is open-source under CC BY 4.0. The tools are MIT.

---

## Essayez-le. Forkez-le.

CBOR-Web doesn't replace robots.txt, sitemap.xml, or llms.txt. It complements them — it's the **final layer** that gives AI agents what they actually need: the content.

If you run a tech blog, a documentation site, an e-commerce product catalog, or any content-heavy website, adding CBOR-Web takes 30 minutes and changes how every AI agent on the planet reads your pages. If you build AI agents, the SDK gives you 100× cheaper crawling across any CBOR-Web-enabled site.

➤ **Try the SaaS**: [cbor-web.explodev.workers.dev](https://cbor-web.explodev.workers.dev)
➤ **Star the repo**: [github.com/ploteddie-bit/cbor-web](https://github.com/ploteddie-bit/cbor-web)
➤ **Install the npm package**: `npm install @deltopide_edy/cborweb`

*The web has two clients. It's time to serve both.*
