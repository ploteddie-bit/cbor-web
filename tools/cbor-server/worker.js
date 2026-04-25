// cbor-server/worker.js — Cloudflare Edge Worker
// Deployed at: cbor-web.explodev.workers.dev
// Proxies CBOR-Web requests to origin server (cbor.deltopide.com)
//
// Requires: compatibility_date "2026-04-25" + global_fetch_strictly_public flag

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
    const originUrl = "https://cbor.deltopide.com" + url.pathname + url.search;
    try {
      const originResp = await fetch(originUrl, {
        method,
        headers: request.headers,
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
