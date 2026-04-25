// ============================================================
// DELTOPIDE WEB AUDIT v1.2 — Background Service Worker
// Moteur Rust/WASM — Wrapper JS leger
// Copyright (c) 2026 Deltopide SL
// ============================================================

import init, { run_audit, validate_license, get_version } from "./audit_engine.js";

let wasmReady = false;
let _auditCount = 0;
let _auditWindowStart = Date.now();

// Initialize WASM on load
async function initWasm() {
  try {
    await init();
    wasmReady = true;
    console.log(`[Deltopide] Moteur WASM ${get_version()} charge`);
  } catch (e) {
    console.error("[Deltopide] Erreur chargement WASM:", e);
  }
}

initWasm();

// Side panel behavior
chrome.runtime.onInstalled.addListener(() => {
  chrome.sidePanel.setPanelBehavior({ openPanelOnActionClick: false });
});

// Rate limiting
function checkRateLimit() {
  const now = Date.now();
  if (now - _auditWindowStart > 60000) { _auditCount = 0; _auditWindowStart = now; }
  return ++_auditCount <= 30;
}

// Device ID — stable pour cette installation
async function getDeviceId() {
  const store = await chrome.storage.local.get(["deltopide_device_id"]);
  if (store.deltopide_device_id) return store.deltopide_device_id;
  const id = crypto.randomUUID();
  await chrome.storage.local.set({ deltopide_device_id: id });
  return id;
}

// License check via Worker API
async function checkLicense() {
  const store = await chrome.storage.local.get(["deltopide_license"]);
  const key = store.deltopide_license || "";
  if (!key) return { valid: false, reason: "no_key" };
  if (!wasmReady) return { valid: false, reason: "wasm_loading" };

  // Offline fallback: validate via WASM
  const wasmValid = validate_license(key);
  if (!wasmValid) return { valid: false, reason: "invalid_key" };

  // Online validation: check expiration + device lock
  try {
    const resp = await fetch("https://deltopide-audit-license.explodev.workers.dev/validate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ key, device_id: await getDeviceId() })
    });
    const data = await resp.json();
    if (data.expires_at) {
      await chrome.storage.local.set({ deltopide_expires_at: data.expires_at });
      if (new Date() > new Date(data.expires_at)) {
        return { valid: false, reason: "expired" };
      }
    }
    return data;
  } catch (e) {
    // Offline: use cached expiration
    const cached = await chrome.storage.local.get(["deltopide_expires_at"]);
    if (cached.deltopide_expires_at && new Date() > new Date(cached.deltopide_expires_at)) {
      return { valid: false, reason: "expired" };
    }
    return { valid: true, reason: "ok_offline" };
  }
}

// Fetch helpers
async function fetchText(url) {
  try {
    const r = await fetch(url, { method: "GET", headers: { "User-Agent": "DeltopideAudit/1.2" } });
    if (r.ok) return { ok: true, text: await r.text(), status: r.status };
    return { ok: false, status: r.status };
  } catch (e) { return { ok: false, status: 0 }; }
}

async function fetchBinary(url) {
  try {
    const r = await fetch(url, { method: "GET" });
    if (r.ok) { const buf = await r.arrayBuffer(); return { ok: true, size: buf.byteLength, status: r.status }; }
    return { ok: false, status: r.status };
  } catch (e) { return { ok: false, status: 0 }; }
}

async function fetchHead(url) {
  try {
    const r = await fetch(url, { method: "HEAD" });
    const headers = {};
    r.headers.forEach((v, k) => { headers[k.toLowerCase()] = v; });
    return { ok: true, status: r.status, headers };
  } catch (e) { return { ok: false, status: 0, headers: {} }; }
}

// Message handler
chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg.action === "runFullAudit") {
    (async () => {
      const license = await checkLicense();
      if (!license.valid) {
        sendResponse({ error: "licence_invalide", reason: license.reason,
          message: "Activez votre licence Deltopide." });
        return;
      }
      if (!wasmReady) {
        sendResponse({ error: "wasm_loading", message: "Moteur en cours de chargement. Reessayez." });
        return;
      }
      if (!checkRateLimit()) {
        sendResponse({ error: "rate_limit", message: "Trop de requetes." });
        return;
      }

      // Fetch external resources
      const origin = new URL(msg.url).origin;
      const [robotsData, sitemapData, cborData, headData, llmsData] = await Promise.all([
        fetchText(`${origin}/robots.txt`),
        fetchText(`${origin}/sitemap.xml`),
        fetchBinary(`${origin}/index.cbor`),
        fetchHead(msg.url),
        fetchText(`${origin}/llms.txt`)
      ]);

      const fetchResults = {
        robots_ok: robotsData.ok || false,
        robots_text: robotsData.text || "",
        sitemap_ok: sitemapData.ok || false,
        sitemap_text: sitemapData.text || "",
        cbor_ok: cborData.ok || false,
        cbor_size: cborData.size || 0,
        cbor_status: cborData.status || 0,
        head_ok: headData.ok || false,
        head_headers: headData.headers || {},
        llms_ok: llmsData.ok || false,
        llms_text: llmsData.text || ""
      };

      const dom = msg.domData;
      const domForRust = {
        url: msg.url, domain: dom.domain || "",
        is_https: dom.isHTTPS || false,
        title: (dom.title || "").substring(0, 2000),
        title_length: dom.titleLength || 0,
        meta_desc: (dom.metaDesc || "").substring(0, 2000),
        meta_desc_length: dom.metaDescLength || 0,
        canonical: (dom.canonical || "").substring(0, 2000),
        canonical_mismatch: dom.canonicalMismatch || false,
        canonical_domain_mismatch: dom.canonicalDomainMismatch || false,
        canonical_domain: dom.canonicalDomain || "",
        viewport: dom.viewport || "", lang: dom.lang || "",
        html_size: dom.htmlSize || 0, word_count: dom.wordCount || 0,
        is_spa: dom.isSPA || false, has_favicon: dom.hasFavicon || false,
        last_modified_meta: dom.lastModifiedMeta || "",
        article_date: dom.articleDate || "",
        has_skip_link: dom.hasSkipLink || false,
        aria_landmarks: dom.ariaLandmarks || 0,
        has_llms_txt_link: dom.hasLlmsTxtLink || false,
        headings: dom.headings || { h1: [], h2: [], h3: [] },
        og: dom.og || {},
        schemas: (dom.schemas || []).map(s => ({ type: s.type || "", name: s.name || "" })),
        hreflangs: dom.hreflangs || [],
        images_total: dom.imagesTotal || 0,
        images_without_alt: dom.imagesWithoutAlt || 0,
        internal_links_count: dom.internalLinksCount || 0,
        external_links_count: dom.externalLinksCount || 0,
        social_links: dom.socialLinks || [],
        tables: dom.tables || 0, lists: dom.lists || 0,
        definition_lists: dom.definitionLists || 0,
        blockquotes: dom.blockquotes || 0,
        stats_found: dom.statsFound || 0,
        faq_patterns: dom.faqPatterns || 0,
        has_about_page: dom.hasAboutPage || false,
        has_contact_page: dom.hasContactPage || false,
        has_privacy_page: dom.hasPrivacyPage || false,
        has_author_info: dom.hasAuthorInfo || false,
        has_testimonials: dom.hasTestimonials || false,
        cwv: dom.cwv || {}, perf_data: dom.perfData || {}
      };

      const resultJson = run_audit(JSON.stringify(domForRust), JSON.stringify(fetchResults));
      const result = JSON.parse(resultJson);
      result.timestamp = new Date().toISOString();
      result._build = get_version();
      result._licensed = "full";

      sendResponse(result);
    })();
    return true;
  }

  if (msg.action === "openSidePanel") {
    chrome.sidePanel.open({ tabId: msg.tabId });
    return false;
  }

  if (msg.action === "activateLicense") {
    (async () => {
      await chrome.storage.local.set({ deltopide_license: msg.key });
      const result = await checkLicense();
      sendResponse(result);
    })();
    return true;
  }

  if (msg.action === "checkLicense") {
    checkLicense().then(sendResponse);
    return true;
  }

  if (msg.action === "gscAuth") {
    (async () => {
      try {
        const token = await chrome.identity.getAuthToken({ interactive: true });
        sendResponse({ ok: true, token: token.token });
      } catch (e) {
        sendResponse({ ok: false, error: e.message });
      }
    })();
    return true;
  }

  if (msg.action === "gscRevoke") {
    (async () => {
      try {
        const { token } = await chrome.identity.getAuthToken({ interactive: false });
        if (token) {
          await chrome.identity.removeCachedAuthToken({ token });
          await fetch(`https://accounts.google.com/o/oauth2/revoke?token=${token}`);
        }
        sendResponse({ ok: true });
      } catch (e) {
        sendResponse({ ok: false, error: e.message });
      }
    })();
    return true;
  }

  if (msg.action === "gscListSites") {
    (async () => {
      try {
        const { token } = await chrome.identity.getAuthToken({ interactive: false });
        if (!token) { sendResponse({ ok: false, error: "Non connecte" }); return; }
        const r = await fetch("https://www.googleapis.com/webmasters/v3/sites", {
          headers: { "Authorization": `Bearer ${token}` }
        });
        const data = await r.json();
        sendResponse({ ok: true, sites: data.siteEntry || [] });
      } catch (e) {
        sendResponse({ ok: false, error: e.message });
      }
    })();
    return true;
  }

  if (msg.action === "gscSubmit") {
    (async () => {
      try {
        const { token } = await chrome.identity.getAuthToken({ interactive: true });
        if (!token) { sendResponse({ ok: false, error: "Authentification requise" }); return; }
        const authHeader = { "Authorization": `Bearer ${token}`, "Content-Type": "application/json" };
        const siteUrl = msg.siteUrl;
        const sitemapUrl = msg.sitemapUrl || `${siteUrl}sitemap.xml`;
        const steps = [];

        const addRes = await fetch(
          `https://www.googleapis.com/webmasters/v3/sites/${encodeURIComponent(siteUrl)}`,
          { method: "PUT", headers: authHeader }
        );
        if (addRes.ok || addRes.status === 204) {
          steps.push({ step: "Ajout site GSC", status: "ok" });
        } else {
          const err = await addRes.json().catch(() => ({}));
          steps.push({ step: "Ajout site GSC", status: "error",
            detail: err.error?.message || `HTTP ${addRes.status}` });
        }

        const smCheck = await fetch(sitemapUrl, { method: "HEAD" }).catch(() => null);
        if (smCheck && smCheck.ok) {
          const smRes = await fetch(
            `https://www.googleapis.com/webmasters/v3/sites/${encodeURIComponent(siteUrl)}/sitemaps/${encodeURIComponent(sitemapUrl)}`,
            { method: "PUT", headers: authHeader }
          );
          if (smRes.ok || smRes.status === 204) {
            steps.push({ step: "Soumission sitemap", status: "ok", detail: sitemapUrl });
          } else {
            const err = await smRes.json().catch(() => ({}));
            steps.push({ step: "Soumission sitemap", status: "error",
              detail: err.error?.message || `HTTP ${smRes.status}` });
          }
        } else {
          steps.push({ step: "Soumission sitemap", status: "warn",
            detail: "sitemap.xml non accessible" });
        }

        const indexRes = await fetch(
          `https://indexing.googleapis.com/v3/urlNotifications:publish`,
          { method: "POST", headers: authHeader,
            body: JSON.stringify({ url: siteUrl, type: "URL_UPDATED" }) }
        );
        if (indexRes.ok) {
          steps.push({ step: "Notification indexation", status: "ok" });
        } else {
          steps.push({ step: "Notification indexation", status: "info",
            detail: "Non disponible (API Indexing optionnelle)" });
        }

        sendResponse({ ok: true, steps });
      } catch (e) {
        sendResponse({ ok: false, error: e.message });
      }
    })();
    return true;
  }
});
