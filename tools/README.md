# CBOR-Web Tools

Reference implementations for the CBOR-Web specification.

## cbor-crawl

CBOR-Web crawler for AI agents — discovers, fetches, and outputs structured content from CBOR-Web endpoints.

```bash
cargo run --bin cbor-crawl -- inspect https://verdetao.com
cargo run --bin cbor-crawl -- fetch https://verdetao.com --format text
cargo run --bin cbor-crawl -- fetch https://verdetao.com --output ./pages
cargo run --bin cbor-crawl -- search https://verdetao.com "cbd"
cargo run --bin cbor-crawl -- verify manifest.cbor
cargo run --bin cbor-crawl -- doleance https://verdetao.com --feedback '{"signals":[{"signal":"missing_data","details":"Price not found","block_type":"table"}],"page_path":"/products/item1"}'
cargo run --bin cbor-crawl -- diff https://verdetao.com --base-version <hex_hash>
```

Commands:
- **inspect** — Display manifest info (pages, languages, sizes, access tiers)
- **fetch** — Fetch full site content as JSON or text (with optional `--output` dir)
- **verify** — Verify a local CBOR file (structure, encoding, hash)
- **search** — Search for a term across all CBOR-Web pages
- **doleance** — Send Doléance feedback to a CBOR-Web publisher
- **diff** — Fetch and display a diff manifest (incremental changes)

## text2cbor

Convert HTML websites to CBOR-Web index.cbor.

```bash
cargo run --bin text2cbor -- generate --input ./site --output ./out --domain example.com --spec-version 2.1
cargo run --bin text2cbor -- validate --file ./out/.well-known/cbor-web/manifest.cbor
cargo run --bin text2cbor -- watch --site /srv/mysite --output ./out --domain example.com
cargo run --bin text2cbor -- doleance --url https://example.com --feedback '{"signals":[...]}'
```

Commands:
- **generate** — Generate CBOR-Web files from HTML directory (`--spec-version 2.1` or `3.0`)
- **validate** — Validate a CBOR file against CBOR-Web spec (deterministic encoding check)
- **watch** — Watch site directory and rebuild incrementally
- **doleance** — Send Doléance feedback to a CBOR-Web publisher

## cbor-vectors

Test vector generator for CBOR-Web v2.1 — generates binary `.cbor` files and SHA-256 hashes.

```bash
cargo run --bin cbor-vectors -- --output ./test-vectors
```

Generates:
- `tv1_manifest.cbor` — Minimal manifest
- `tv2_page.cbor` — Minimal page
- `tv3_product.cbor` — Product page with structured data
- `tv4_bundle.cbor` — Bundle with manifest + page
- `tv5_navigation.cbor` — Manifest with navigation

All vectors use RFC 8949 §4.2.1 deterministic encoding.
