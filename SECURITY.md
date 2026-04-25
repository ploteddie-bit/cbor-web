# Security Policy

## Reporting a Vulnerability

Please do **not** open a public issue. Report vulnerabilities privately to:

- **Email**: security@deltopide.com
- **GitHub**: Use the "Report a vulnerability" button on the Security tab

We aim to acknowledge reports within 48 hours and provide a fix within 7 days.

## Supported Versions

| Version | Supported |
|---------|-----------|
| v2.2.x  | ✅ Active |
| v2.1.x  | ✅ Security fixes |
| v2.0.x  | ❌ End of life |
| v1.x    | ❌ End of life |

## Security Design Principles

CBOR-Web follows these security principles:

1. **Zero trust** — every request is rate-limited and authenticated independently
2. **Least privilege** — the reference server binds to localhost by default; use a reverse proxy for public exposure
3. **Defense in depth** — multiple security layers (rate limiting, auth middleware, size limits, CORS restrictions)
4. **Fail closed** — in production, missing authentication denies access by default
5. **Input validation** — all CBOR parsers enforce size, depth, and item count limits against DoS

## Production Deployment Checklist

- [ ] Set `CBOR_TOKEN` environment variable (never hardcode in config files)
- [ ] Bind to `127.0.0.1` and use nginx/Caddy as reverse proxy with TLS
- [ ] Enable firewall: only expose ports 80/443 publicly
- [ ] Use Cloudflare Tunnel or similar for the origin server (no open ports)
- [ ] Enable GitHub secret scanning and push protection on the repository
- [ ] Run `cargo audit` on every CI build
- [ ] Review worker.js header forwarding: only allow safe headers

## Known Protections

| Layer | Implementation |
|-------|---------------|
| Rate limiting | Per-IP sliding window, configurable req/s |
| Auth | Static token OR token server wallet verification |
| CBOR DoS | Max 50MB input, max depth 50, max 100K array/map items |
| CORS | Read-only (GET/HEAD) on edge worker, no POST to origin |
| Header filtering | Worker only forwards safe headers to origin |
| Binary validation | All CBOR documents must have self-described tag (55799) |

## Dependency Security

All Rust dependencies are audited via `cargo audit` in CI. Critical/High severity vulnerabilities block the build.

## Responsible Disclosure Hall of Fame

_None yet. Be the first._
