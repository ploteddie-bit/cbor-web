=== CBOR-Web ===
Contributors: explodev
Tags: cbor, ai, machine-readable, agents, structured-data, binary, web-protocol
Requires at least: 6.0
Tested up to: 6.7
Requires PHP: 8.0
Stable tag: 2.1.0
License: MIT
License URI: https://opensource.org/licenses/MIT

Publish machine-readable CBOR versions of your content. AI agents get structured binary data at 1/100th the cost of parsing HTML.

== Description ==

CBOR-Web is an open protocol (RFC 8949) that lets any website expose a machine-native copy of its content — in parallel with existing HTML. Humans see the regular site. AI agents get a binary, structured, 10x smaller version.

= Key Features =

* **Automatic CBOR generation** — on post/page save, generates `.cbor` files from your content
* **Well-known endpoint** — serves `/.well-known/cbor-web` manifest and page files
* **AI agent discovery** — agents automatically discover your structured content
* **Token-gated access** — support for T0 (public), T1/T2 (token-required) access levels
* **Deterministic encoding** — RFC 8949 §4.2 compliant, hash-reproducible binary output
* **Bundle support** — single-request full-site indexing at `/.well-known/cbor-web/bundle`

= Why CBOR-Web? =

An AI agent crawling conventional HTML wastes 93% of tokens stripping markup. CBOR-Web pages are ~2 KB vs ~1.5 MB for a typical React SPA. That's a 1000:1 compression ratio — real measured results from production sites.

= How It Works =

1. Activate the plugin
2. Configure access level and optionally a token wallet under Settings → CBOR-Web
3. Save any page or post — CBOR files are generated automatically
4. Agents discover your CBOR content at `/.well-known/cbor-web`

= Endpoints =

* `/.well-known/cbor-web` — Site manifest (metadata, page index)
* `/.well-known/cbor-web/pages/{file}.cbor` — Individual page CBOR
* `/.well-known/cbor-web/bundle` — Full site bundle
* `/.well-known/cbor-web-token` — Token configuration (plain text)

== Installation ==

1. Upload the `cbor-web` folder to `/wp-content/plugins/`
2. Activate the plugin through the 'Plugins' menu
3. Go to Settings → CBOR-Web to configure
4. Save a page or post to trigger initial generation
5. Verify by visiting `https://yoursite.com/.well-known/cbor-web`

== Frequently Asked Questions ==

= Does this affect my regular website? =

No. CBOR-Web is a parallel channel. Human visitors see your normal HTML site unchanged. Only AI agents requesting `application/cbor` receive the binary version.

= What content block types are supported? =

Headings (h1-h6), paragraphs, unordered/ordered lists, blockquotes, code blocks, images, and separators. Additional block types from the CBOR-Web specification are available in the full spec at the project repository.

= Is a token required for access? =

No token is required by default (T0 access). You can optionally set T1 or T2 to gate content behind the CBOR-Web token, or leave it at T0 for full public visibility.

= Where are CBOR files stored? =

In `wp-content/cbor-web/` — `manifest.cbor`, `bundle.cbor`, and a `pages/` directory with individual `.cbor` files.

= What if I change my permalink structure? =

Regenerate all CBOR files from the Settings page. File paths are derived from permalink paths.

== Changelog ==

= 2.1.0 =
* Initial WordPress plugin release
* Automatic CBOR generation on post/page save
* Settings page with enable/disable toggle, access levels, token wallet
* Well-known endpoint: manifest, pages, bundle, token info
* Admin notice with generation stats
* Deterministic CBOR encoding per RFC 8949 §4.2
* HTML content block extraction (headings, paragraphs, lists, quotes, code, images)

== Upgrade Notice ==

= 2.1.0 =
First release. No upgrade path needed.

== Credits ==

CBOR-Web protocol: ExploDev / Deltopide SL https://github.com/ploteddie-bit/cbor-web

WordPress plugin: ExploDev
