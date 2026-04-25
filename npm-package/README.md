# CBOR-Web Client SDK

Zero-dependency client for CBOR-Web — the binary web protocol for AI agents.

```ts
import { CBORWebClient } from "cborweb";

const client = new CBORWebClient("https://cbor.deltopide.com");
const manifest = await client.manifest();
console.log(`${manifest.site_name} — ${manifest.pages_count} pages`);
```

## API

- `new CBORWebClient(baseUrl)` — create client
- `manifest()` — fetch site manifest
- `page(path)` — fetch single page
- `bundle()` — fetch full site in one request

## What is CBOR-Web?

CBOR-Web (RFC 8949) serves structured web content in binary format for AI agents — 10x smaller than HTML, 95% signal ratio.

Spec: https://github.com/ploteddie-bit/cbor-web
