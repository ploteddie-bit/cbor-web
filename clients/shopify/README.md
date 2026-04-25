# CBOR-Web for Shopify

Make your Shopify store readable by AI agents — 10x faster, at a fraction of the cost.

This is a single Liquid snippet. Copy, paste, done.

---

## What this does

- **CBOR-Web manifest** — a machine-readable index of every product, collection, page, and blog post in your store.
- **JSON-LD structured data** — rich product data on every product page, already in CBOR-Web compatible format.
- AI agents (ChatGPT, Claude, Gemini, enterprise RAG) discover your products instantly.
- Better SEO via structured data that Google, Bing, and AI crawlers consume.

---

## Installation (2 minutes)

### Step 1 — Copy the snippet

1. Go to **Online Store → Themes → Actions → Edit code**
2. In the `snippets/` folder, click **Add a new snippet**
3. Name it `cbor-web-shopify` (no `.liquid` — Shopify adds it)
4. Copy the contents of [`cbor-web-shopify.liquid`](cbor-web-shopify.liquid) and paste
5. Click **Save**

### Step 2 — Create the manifest page template

1. In the code editor, under `templates/`, click **Add a new template**
2. Choose **page** as the template type
3. Name it `cbor-web`
4. Replace all content with these two lines:

```liquid
{%- layout none -%}
{%- render 'cbor-web-shopify' -%}
```

5. Click **Save**

### Step 3 — Add JSON-LD to product pages

1. Open `layout/theme.liquid`
2. Find the closing `</head>` tag
3. Add this line just before it:

```liquid
{%- render 'cbor-web-shopify' -%}
```

4. Click **Save**

---

## Verify it works

1. Visit `https://yourstore.com/pages/cbor-web` — you should see a JSON manifest
2. Visit any product page and view source — you should see `<script type="application/ld+json">` with product data

---

## Keeping the manifest page hidden

The `/pages/cbor-web` URL is meant for machines, not humans. To hide it from your store navigation:

- Don't add it to any menu
- Add this to `robots.txt.liquid` (or your theme's robots.txt):
  ```
  Disallow: /pages/cbor-web
  ```
- It stays accessible to AI agents but won't appear in search results

---

## Benefits for your store

| What | How |
|------|-----|
| **AI agents find your products** | ChatGPT, Claude, Gemini crawl the CBOR-Web manifest and index every product instantly |
| **Better SEO** | JSON-LD structured data on every product page (Google, Bing, AI crawlers) |
| **Zero maintenance** | Auto-updates — new products appear in the manifest automatically |
| **Zero performance impact** | Manifest is a single JSON file, generated server-side, no JavaScript |
| **No CMS change** | Your store looks and works exactly the same for human customers |
| **Future-proof** | CBOR-Web is an open protocol (IETF RFC 8949 compatible) — your content is ready for the next generation of AI agents |

---

## Advanced: Custom fields (metafields)

Want to expose extra product data to AI agents? Add metafields to your products:

1. Go to **Settings → Custom data → Products → Add definition**
2. Create a metafield namespace `cbor_web` with any of these keys:
   - `rating_value` — numeric rating (e.g. 4.5)
   - `rating_count` — number of reviews
   - `specs` — JSON string of product specifications

The JSON-LD snippet automatically includes these if present.

---

## Support

- **GitHub**: https://github.com/ploteddie-bit/cbor-web
- **npm**: `@deltopide_edy/cborweb`
- **Live dashboard**: https://cbor-web.explodev.workers.dev

---

*CBOR-Web Shopify v2.1 — ExploDev / Deltopide SL — MIT License*
