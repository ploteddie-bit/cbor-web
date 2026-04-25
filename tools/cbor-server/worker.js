// cbor-server/worker.js — Cloudflare Edge Worker
// Routes:
//   Static content → R2 Assets (HTML, CBOR files)
//   Dynamic endpoints → Origin server (serveur-dev via CF Tunnel)
//
// Deploy with: wrangler deploy
// Requires: R2 bucket bound as ASSETS, origin server bound as ORIGIN

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const path = url.pathname;
    const method = request.method;

    // CORS preflight
    if (method === "OPTIONS") {
      return new Response(null, {
        status: 204,
        headers: corsHeaders(),
      });
    }

    // ── Dynamic endpoints → proxy to origin ──
    if (shouldProxy(path, method)) {
      return proxyToOrigin(request, env, url);
    }

    // ── Static: resolve index.html for root paths ──
    let assetPath = path;
    if (assetPath === "/") {
      // Route to the domain-specific index.html
      const host = request.headers.get("host") || "cbor-web.com";
      assetPath = `/${host.replace("www.", "")}/index.html`;
    } else if (assetPath.startsWith("/.well-known/cbor-web/pages/")) {
      // Pages: serve from R2 from static build output
      const host = request.headers.get("host") || "cbor-web.com";
      const pageFile = path.split("/").pop() || "index.cbor";
      assetPath = `/${host.replace("www.", "")}/.well-known/cbor-web/pages/${pageFile}`;
    } else if (assetPath === "/.well-known/cbor-web") {
      // Manifest: serve from R2
      const host = request.headers.get("host") || "cbor-web.com";
      assetPath = `/${host.replace("www.", "")}/.well-known/cbor-web/manifest.cbor`;
    } else if (assetPath === "/.well-known/cbor-web/bundle") {
      const host = request.headers.get("host") || "cbor-web.com";
      assetPath = `/${host.replace("www.", "")}/.well-known/cbor-web/bundle.cbor`;
    } else if (assetPath === "/index.cbor") {
      // Simplified read protocol: serve from domain subfolder
      const host = request.headers.get("host") || "cbor-web.com";
      assetPath = `/${host.replace("www.", "")}/index.cbor`;
    }

    // ── Serve from R2 Assets ──
    try {
      const asset = await env.ASSETS.fetch(
        new URL(assetPath, request.url),
        { method: "GET" }
      );

      if (asset.ok) {
        let headers = new Headers(asset.headers);
        headers.set("Access-Control-Allow-Origin", "*");
        headers.set("Access-Control-Expose-Headers", "ETag, Content-Type");

        // Set correct content type for CBOR files
        if (assetPath.endsWith(".cbor")) {
          headers.set("Content-Type", "application/cbor");
          // Cache CBOR for 1h at edge
          headers.set("Cache-Control", "public, max-age=3600");
        }

        return new Response(asset.body, {
          status: asset.status,
          headers,
        });
      }
    } catch (e) {
      // Fall through to origin or 404
    }

    // ── Fallback: try origin server ──
    const originResp = await proxyToOrigin(request, env, url);
    if (originResp.status !== 404) {
      return originResp;
    }

    return new Response("Not Found", {
      status: 404,
      headers: corsHeaders(),
    });
  },
};

// ── Helpers ──

function corsHeaders() {
  return {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, POST, HEAD, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type, X-CBOR-Web-Wallet, X-CBOR-Web-Sig, X-CBOR-Web-Nonce",
    "Access-Control-Expose-Headers": "ETag, Content-Type, X-CBOR-Web-Tier",
  };
}

function shouldProxy(path, method) {
  // Dynamic endpoints that need server-side logic
  if (path === "/.well-known/cbor-web/doleance" && method === "POST") return true;
  if (path === "/.well-known/cbor-web/diff") return true;

  // Token-gated content (T0/T1) — always proxy for auth check
  // Worker can't verify ERC-20 balances, so delegate to origin
  return false;
}

async function proxyToOrigin(request, env, url) {
  try {
    const originUrl = new URL(url.pathname + url.search, env.ORIGIN_URL || "http://origin");
    const originReq = new Request(originUrl, {
      method: request.method,
      headers: request.headers,
      body: request.body,
      redirect: "manual",
    });

    const resp = await fetch(originReq);
    let headers = new Headers(resp.headers);
    headers.set("Access-Control-Allow-Origin", "*");
    headers.set("Access-Control-Expose-Headers", "ETag, Content-Type, X-CBOR-Web-Tier");

    return new Response(resp.body, {
      status: resp.status,
      headers,
    });
  } catch (e) {
    return new Response("Origin unreachable", {
      status: 502,
      headers: corsHeaders(),
    });
  }
}
