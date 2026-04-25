// cbor-server/worker.js — Cloudflare Edge Worker
// Deployed at: cbor-web.explodev.workers.dev
// Proxies CBOR-Web requests to origin server (cbor.deltopide.com)
//
// Short codes: /<code>/<path> → domain routing  (e.g. /lfr/ → laforetnousregale.fr)
// Full domain:  /s/<domain>/<path>              (e.g. /s/deltopide.com/)
// Uses X-CBOR-Domain header because Host header is blocked by Cloudflare
//
// Requires: compatibility_date "2026-04-25" + global_fetch_strictly_public flag

const SHORT = {
  // 3-letter codes → domain
  lfr: "laforetnousregale.fr",
  dtp: "deltopide.com",
  cbw: "cbor-web.com",
  cb2: "cborweb.com",
  cbo: "cborweb.org",
  cbs: "cbor-web.site",
  cbt: "cbor-web.tech",
  cbf: "cbor-web.fr",
  edv: "explodev.com",
  edf: "explodev.fr",
  edo: "explodev.org",
  eds: "explodev.site",
  edt: "explodev.tech",
  edw: "explodev.website",
  vta: "verdetao.fr",
  vtb: "verdetao.be",
  vtd: "verdetao.de",
  vte: "verdetao.eu",
  vts: "verdetao.es",
  cbd: "californiacbd.fr",
  cbe: "californiacbd.es",
  clc: "californialovecbd.es",
  cls: "californialovecbd.site",
  cle: "californialove.es",
  mjc: "mariejeannecbd.fr",
  mje: "mariejeannecbd.es",
  fcc: "fanaticodelcbd.com",
  fce: "fanaticodelcbd.es",
  bcc: "bienestarcosmeticacbd.es",
  bcf: "bienetrecosmetiquecbd.fr",
  amz: "amazingcbd.es",
  cas: "castelloconviu.es",
  cgm: "cargamipatinete.es",
  ptp: "preciotupatinete.es",
  rtc: "ritueletcalme.com",
  cau: "courtiers-auto.com",
  dts: "deltopide.site",
  wbc: "wellbeingcosmeticcbd.com",
};

// Preserved top-level routes — don't shadow these
const RESERVED = new Set(["health", "doleance", "diff", "s", ""]);

export default {
  async fetch(request) {
    const url = new URL(request.url);
    const method = request.method;
    const corsHeaders = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, HEAD, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type, X-CBOR-Web-Wallet",
      "Access-Control-Expose-Headers": "ETag, Content-Type",
    };
    if (method === "OPTIONS") {
      return new Response(null, { status: 204, headers: corsHeaders });
    }

    let domainOverride = null;
    let outPath = url.pathname;
    let outSearch = url.search;

    // ── Short code routing: /<code>[/path] ──
    const seg = outPath.match(/^\/([a-zA-Z0-9]+)(\/.*)?$/);
    if (seg && SHORT[seg[1]] && !RESERVED.has(seg[1])) {
      if (!seg[2]) {
        return Response.redirect(url.origin + url.pathname + "/" + url.search, 301);
      }
      domainOverride = SHORT[seg[1]];
      outPath = seg[2];
    }

    // ── Full domain routing: /s/<domain>[/path] ──
    if (!domainOverride) {
      const dm = outPath.match(/^\/s\/([a-zA-Z0-9][a-zA-Z0-9._-]*)(\/.*)?$/);
      if (dm) {
        if (!dm[2]) {
          return Response.redirect(url.origin + url.pathname + "/" + url.search, 301);
        }
        domainOverride = dm[1];
        outPath = dm[2];
      }
    }

    const originUrl = "https://cbor.deltopide.com" + outPath + outSearch;

    const fwdHeaders = new Headers();
    for (const [k, v] of request.headers) {
      if (k.toLowerCase() !== "host") {
        fwdHeaders.set(k, v);
      }
    }
    if (domainOverride) {
      fwdHeaders.set("X-CBOR-Domain", domainOverride);
    }

    try {
      const originResp = await fetch(originUrl, {
        method,
        headers: fwdHeaders,
        body: method === "POST" ? request.body : undefined,
      });
      const out = new Headers(originResp.headers);
      for (const [k, v] of Object.entries(corsHeaders)) { out.set(k, v); }
      out.set("X-CBOR-Edge", "Cloudflare");
      if (method === "GET" && originResp.status === 200) {
        out.set("Cache-Control", "public, max-age=3600");
      }
      return new Response(originResp.body, { status: originResp.status, headers: out });
    } catch (e) {
      return new Response("Origin unreachable: " + e.message, { status: 502, headers: corsHeaders });
    }
  },
};
