# Security headers, llms.txt, and compliance files — cborweb.com v2.3

*By Eddie Plot — CBOR-Web, September 10, 2026*

---

**cborweb.com now ships with production-grade HTTP security headers, a machine-readable llms.txt for AI agents, and standard compliance files (humans.txt, security.txt).** These changes harden the site against common attack vectors and make it fully transparent to both humans and automated systems.

---

## Security headers deployed

Every response from cborweb.com now includes the following headers, verifiable with `curl -sI https://www.cborweb.com`:

- **Strict-Transport-Security** (HSTS): `max-age=63072000; includeSubDomains` — forces HTTPS for two years, including all subdomains.
- **Content-Security-Policy** (CSP): restricts script, style, image, and connect sources to known origins. Inline scripts are limited to `'unsafe-inline'` (required for the current static site); external scripts load only from `cdn.jsdelivr.net`. `frame-ancestors 'none'` prevents clickjacking.
- **X-Frame-Options**: `DENY` — blocks any iframe embedding.
- **X-Content-Type-Options**: `nosniff` — stops MIME-type sniffing.
- **Referrer-Policy**: `strict-origin-when-cross-origin` — sends full referrer only for same-origin requests.
- **Permissions-Policy**: disables camera, microphone, and geolocation by default.

These headers are served at the CDN edge via Cloudflare, applying to all responses regardless of origin server configuration.

---

## llms.txt — AI agent index

A `llms.txt` file is now available at `https://cborweb.com/llms.txt`. Following the emerging convention for machine-readable site descriptions, it lists every documentation page, SDK, API endpoint, and blog post with absolute URLs — giving AI agents a structured entry point to the project without crawling HTML. The file covers the full spec suite (7 documents), 8 SDKs, the IETF Internet-Draft, and the live SaaS dashboard.

---

## humans.txt and security.txt

Two standard compliance files complete the update:

- **`/humans.txt`** (humanstxt.org format): credits the team, site technology stack, and last update date.
- **`/.well-known/security.txt`** (RFC 9116): provides a contact address, vulnerability disclosure policy link, expiration date, and preferred languages for security researchers reporting issues.

Both files are committed on a dedicated branch (`fondations/humans-security-txt`) and pending merge.

---

## What's next

These files represent the foundation layer — security, transparency, and machine-readability. The next milestone focuses on the v2.2 SDK release and expanded agent authentication.

➤ **Verify the headers**: `curl -sI https://www.cborweb.com`
➤ **Read llms.txt**: [cborweb.com/llms.txt](https://cborweb.com/llms.txt)
➤ **Star the repo**: [github.com/ploteddie-bit/cbor-web](https://github.com/ploteddie-bit/cbor-web)
