# cbor-server — CBOR-Web HTTP Server

Reference implementation server for the [CBOR-Web specification](https://github.com/ploteddie-bit/cbor-web).

## Architecture

```
Internet → Cloudflare DNS
              │
              ▼
    Cloudflare Worker (edge)
    ├─ Static: R2 bucket           → HTML, .cbor files
    └─ Dynamic: proxy to origin    → POST doléance, GET diff
              │
              ▼
    serveur-dev:3001 (Rust/axum)
    ├─ GET  /.well-known/cbor-web          → manifest
    ├─ GET  /.well-known/cbor-web/pages/*  → page CBOR
    ├─ GET  /.well-known/cbor-web/bundle   → bundle CBOR
    ├─ POST /.well-known/cbor-web/doleance → agent feedback
    └─ GET  /.well-known/cbor-web/diff     → incremental diff
```

## Quick Start

```bash
# Build
cargo build --release

# Create data directory with CBOR files
mkdir -p data/.well-known/cbor-web/pages
cp manifest.cbor data/.well-known/cbor-web/
cp bundle.cbor data/.well-known/cbor-web/
cp pages/*.cbor data/.well-known/cbor-web/pages/

# Run (open mode)
./cbor-server --data data --listen 0.0.0.0:3001

# Run (with token auth)
./cbor-server --data data --listen 0.0.0.0:3001 --token cbw_your_secret
```

## Endpoints

| Method | Path | Response | Notes |
|--------|------|----------|-------|
| GET | `/.well-known/cbor-web` | `application/cbor` | Manifest (v2.1/v3.0) |
| GET | `/.well-known/cbor-web/pages/*` | `application/cbor` | Individual pages |
| GET | `/.well-known/cbor-web/bundle` | `application/cbor` | Full site bundle |
| POST | `/.well-known/cbor-web/doleance` | `202 Accepted` | Agent feedback |
| GET | `/.well-known/cbor-web/diff?base=HASH` | `application/cbor` | Incremental diff |

## Deployment

```bash
# Deploy to serveur-dev from serveur-build
./deploy.sh

# Or run with systemd
cp cbor-server.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now cbor-server
```

## Rate Limiting

Default: 10 req/s per IP. Configure with `--rate-limit <N>`.

## Auth

Set `--token <value>` to require `X-CBOR-Web-Wallet` header matching the token. Without `--token`, all content is public.
