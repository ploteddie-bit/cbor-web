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

const MAX_RESPONSE_SIZE = 50 * 1024 * 1024; // 50 MB
const FETCH_TIMEOUT_MS = 30000;              // 30 seconds

// ── Minimal CBOR decoder (RFC 8949 subset) ──
function decodeCBOR(data) {
  let offset = 0;
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);

  function readArg(ib) {
    const info = ib & 0x1F;
    if (info < 24) return info;
    if (info === 24) { offset++; return data[offset - 1]; }
    if (info === 25) { const v = dv.getUint16(offset); offset += 2; return v; }
    if (info === 26) { const v = dv.getUint32(offset); offset += 4; return v; }
    if (info === 27) { const v = Number(dv.getBigUint64(offset)); offset += 8; return v; }
    throw new Error("bad CBOR arg: " + info);
  }

  function decode() {
    if (offset >= data.length) throw new Error("CBOR underflow");
    const ib = data[offset++];
    const major = (ib >> 5) & 0x07;
    const info = ib & 0x1F;

    if (major === 0) return readArg(ib);
    if (major === 1) return -1 - readArg(ib);
    if (major === 2) { const len = readArg(ib); const v = data.slice(offset, offset + len); offset += len; return v; }
    if (major === 3) { const len = readArg(ib); const v = new TextDecoder().decode(data.slice(offset, offset + len)); offset += len; return v; }
    if (major === 4) {
      const items = [];
      if (info === 31) { while (data[offset] !== 0xFF) items.push(decode()); offset++; }
      else { const len = readArg(ib); for (let i = 0; i < len; i++) items.push(decode()); }
      return items;
    }
    if (major === 5) {
      const obj = {};
      const len = info === 31 ? Infinity : readArg(ib);
      let count = 0;
      while ((info === 31 && data[offset] !== 0xFF) || (info !== 31 && count < len)) {
        const k = decode(); const v = decode(); obj[k] = v; count++;
      }
      if (info === 31) offset++;
      return obj;
    }
    if (major === 6) {
      const tag = readArg(ib); const inner = decode();
      if (tag === 55799) return inner; // self-described CBOR-Web
      if (tag === 1) return inner; // timestamp
      return { ["@tag" + tag]: inner };
    }
    if (major === 7) {
      if (info === 20) return false;
      if (info === 21) return true;
      if (info === 22) return null;
      if (info === 25) { const v = dv.getFloat16(offset); offset += 2; return v; }
      if (info === 26) { const v = dv.getFloat32(offset); offset += 4; return v; }
      if (info === 27) { const v = dv.getFloat64(offset); offset += 8; return v; }
      return null;
    }
    throw new Error("bad CBOR major: " + major);
  }

  // Skip self-described tag if present
  if (data.length >= 3 && data[0] === 0xD9 && data[1] === 0xD9 && data[2] === 0xF7) {
    offset = 3;
  }
  return decode();
}

// ── HTML escape ──
function esc(s) {
  if (!s) return "";
  s = String(s);
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

export default {
  async fetch(request) {
    const url = new URL(request.url);
    const method = request.method;
    const corsHeaders = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, HEAD, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type, Accept",
      "Access-Control-Expose-Headers": "ETag, Content-Type",
    };
    // Block dangerous methods in CORS preflight
    if (method === "OPTIONS") {
      return new Response(null, { status: 204, headers: corsHeaders });
    }
    // Reject POST/PUT/DELETE unless explicitly allowed
    if (method !== "GET" && method !== "HEAD") {
      return new Response("Method not allowed", { status: 405, headers: corsHeaders });
    }

    let domainOverride = null;
    let outPath = url.pathname;
    let outSearch = url.search;

    // ── Short code routing: /<code>[/path] ──
    const seg = outPath.match(/^\/([a-zA-Z0-9]+)(\/.*)?$/);
    if (seg) {
      const code = seg[1].toLowerCase();
      if (SHORT[code] && !RESERVED.has(code)) {
        if (!seg[2]) {
          return Response.redirect(url.origin + url.pathname + "/" + url.search, 301);
        }
        domainOverride = SHORT[code];
        outPath = seg[2];
      }
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

    // ── Dashboard with inline manifest (root path) ──
    if (outPath === "/" || outPath === "") {
      const domain = domainOverride || "deltopide.com";
      try {
        const c2 = new AbortController();
        const t2 = setTimeout(() => c2.abort(), FETCH_TIMEOUT_MS);
        const manifestResp = await fetch(`https://cbor.deltopide.com/.well-known/cbor-web`, {
          headers: { "X-CBOR-Domain": domain, "Accept": "application/cbor" },
          signal: c2.signal,
        });
        clearTimeout(t2);
        if (manifestResp.ok) {
          const cborBytes = new Uint8Array(await manifestResp.arrayBuffer());
          const manifest = decodeCBOR(cborBytes);
          const site = manifest[2] || {};
          const pages = manifest[3] || [];
          const meta = manifest[5] || {};
          let pagesHTML = pages.slice(0, 100).map(p => {
            const path = p.path || p.file || "/";
            const hash = p.hash ? Array.from(new Uint8Array(p.hash.slice(0, 4))).map(b => b.toString(16).padStart(2, "0")).join("") : "-";
            const size = p.size || 0;
            return `<tr><td><code>${esc(path)}</code></td><td>${size} B</td><td>${hash}...</td></tr>`;
          }).join("");
          const dashHTML = `<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>CBOR-Web — ${esc(site.name || domain)}</title><style>*{margin:0;padding:0;box-sizing:border-box}body{font-family:system-ui,monospace;background:#0a0a0a;color:#e0e0e0;padding:2rem}h1{color:#f97316;margin-bottom:.5rem}h1 span{color:#fff}h2{color:#f97316;margin:1.5rem 0 .5rem}.sub{color:#888;margin-bottom:.5rem}table{width:100%;border-collapse:collapse;margin:1rem 0}th{text-align:left;padding:.5rem .8rem;color:#888;border-bottom:1px solid #333;font-size:.75rem;text-transform:uppercase}td{padding:.5rem .8rem;border-bottom:1px solid #1a1a1a;font-size:.85rem}a{color:#f97316;text-decoration:none}.info{color:#888;margin:.5rem 0}.foot{margin-top:3rem;color:#444;font-size:.7rem}code{color:#28c840}.badges{display:flex;gap:.5rem;flex-wrap:wrap;margin:1rem 0}.badge{display:inline-block;padding:3px 8px;border-radius:4px;font-size:.7rem;font-weight:700;text-decoration:none}.b-rust{background:#000;color:#fff}.b-ts{background:#3178C6;color:#fff}.b-react{background:#61DAFB;color:#000}.b-python{background:#3776AB;color:#fff}.b-go{background:#00ADD8;color:#fff}.b-cpp{background:#00599C;color:#fff}.b-php{background:#777BB4;color:#fff}.b-ruby{background:#CC342D;color:#fff}.b-java{background:#ED8B00;color:#fff}.b-npm{background:#CB3837;color:#fff}.b-cf{background:#F38020;color:#fff}</style></head><body><h1>CBOR-<span>Web</span></h1><p class="sub">${esc(site.name || "")} — ${esc(site.domain || domain)} — ${pages.length} pages</p><div class="badges"><span class="badge b-rust">🦀 Rust</span><span class="badge b-ts">TS</span><span class="badge b-react">⚛️ React</span><span class="badge b-python">🐍 Python</span><span class="badge b-go">🔵 Go</span><span class="badge b-cpp">C++</span><span class="badge b-php">🐘 PHP</span><span class="badge b-ruby">💎 Ruby</span><span class="badge b-java">☕ Java</span><span class="badge b-npm">📦 npm</span><span class="badge b-cf">☁️ Cloudflare</span></div><p class="info">${esc(site.description || "")}</p><p class="info">Languages: ${esc((site.languages || []).join(", "))} | Default: ${esc(site.default_language || "")} | Bundle: ${meta.bundle_available ? "available" : "N/A"} | ${meta.total_size || 0} bytes total</p><h2>Pages</h2><table><tr><th>Path</th><th>Size</th><th>Hash</th></tr>${pagesHTML}</table><div class="foot">CBOR-Web v2.2 — <a href="/codes">Short Codes</a> — <a href="https://github.com/ploteddie-bit/cbor-web">GitHub</a></div></body></html>`;
          return new Response(dashHTML, {
            status: 200,
            headers: { ...corsHeaders, "Content-Type": "text/html; charset=utf-8", "Cache-Control": "public, max-age=300" },
          });
        }
      } catch (e) { /* fall through to origin */ }
    }

    // ── Default domain when no short code and no /s/ — use deltopide.com ──
    if (!domainOverride && !url.pathname.startsWith("/s/")) {
      domainOverride = "deltopide.com";
    }

    const originUrl = "https://cbor.deltopide.com" + outPath + outSearch;

    const SAFE_FWD_HEADERS = new Set(["accept", "accept-encoding", "accept-language", "if-none-match", "if-modified-since", "content-type", "x-cbor-domain", "x-cbor-web-wallet"]);
    const fwdHeaders = new Headers();
    for (const [k, v] of request.headers) {
      if (SAFE_FWD_HEADERS.has(k.toLowerCase())) {
        fwdHeaders.set(k, v);
      }
    }
    // If-None-Match / If-Modified-Since pass through here for 304 support
    if (domainOverride) {
      fwdHeaders.set("X-CBOR-Domain", domainOverride);
    }

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), FETCH_TIMEOUT_MS);

    try {
      const originResp = await fetch(originUrl, {
        method,
        headers: fwdHeaders,
        body: method === "POST" ? request.body : undefined,
        signal: controller.signal,
      });

      // Reject responses larger than 50 MB
      const contentLength = originResp.headers.get("Content-Length");
      if (contentLength && parseInt(contentLength, 10) > MAX_RESPONSE_SIZE) {
        console.log(
          "Size exceeded: " + contentLength + " bytes (" +
          Math.round(parseInt(contentLength, 10) / (1024 * 1024)) +
          " MB) for " + originUrl
        );
        return new Response("Response too large", { status: 502, headers: corsHeaders });
      }

      const out = new Headers(originResp.headers);
      for (const [k, v] of Object.entries(corsHeaders)) { out.set(k, v); }
      out.set("X-CBOR-Edge", "Cloudflare");
      if (method === "GET" && (originResp.status === 200 || originResp.status === 304)) {
        out.set("Cache-Control", "public, max-age=3600");
      }
      return new Response(originResp.body, { status: originResp.status, headers: out });
    } catch (e) {
      if (e.name === "AbortError") {
        console.log("Timeout (" + FETCH_TIMEOUT_MS / 1000 + "s) fetching: " + originUrl);
        return new Response("Origin request timed out", { status: 504, headers: corsHeaders });
      }
      console.log("Origin unreachable: " + e.message + " for " + originUrl);
      return new Response("Origin unreachable: " + e.message, { status: 502, headers: corsHeaders });
    } finally {
      clearTimeout(timer);
    }
  },
};
