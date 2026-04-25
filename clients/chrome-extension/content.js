// ============================================================
// DELTOPIDE WEB AUDIT — Content Script
// Analyse le DOM de la page courante
// ============================================================

(() => {
  // Éviter double injection
  if (window.__deltopideAuditInjected) return;
  window.__deltopideAuditInjected = true;

  chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
    if (msg.action === "analyzeDOM") {
      try {
        const result = analyzePage();
        sendResponse({ success: true, data: result });
      } catch (e) {
        sendResponse({ success: false, error: e.message });
      }
    }
    return true; // async response
  });

  function analyzePage() {
    const doc = document;
    const html = doc.documentElement.outerHTML;

    // --- META TAGS ---
    const title = doc.title || "";
    const metaDesc = doc.querySelector('meta[name="description"]')?.content || "";
    const canonical = doc.querySelector('link[rel="canonical"]')?.href || "";
    const viewport = doc.querySelector('meta[name="viewport"]')?.content || "";
    const charset = doc.querySelector('meta[charset]')?.getAttribute("charset") ||
                    doc.querySelector('meta[http-equiv="Content-Type"]')?.content || "";
    const lang = doc.documentElement.lang || "";

    // --- OPEN GRAPH ---
    const og = {};
    doc.querySelectorAll('meta[property^="og:"]').forEach(m => {
      og[m.getAttribute("property")] = m.content;
    });
    const twitter = {};
    doc.querySelectorAll('meta[name^="twitter:"]').forEach(m => {
      twitter[m.getAttribute("name")] = m.content;
    });

    // --- HEADINGS ---
    const headings = { h1: [], h2: [], h3: [] };
    doc.querySelectorAll("h1").forEach(h => headings.h1.push(h.textContent.trim()));
    doc.querySelectorAll("h2").forEach(h => headings.h2.push(h.textContent.trim()));
    doc.querySelectorAll("h3").forEach(h => headings.h3.push(h.textContent.trim()));

    // Limit headings to prevent oversized messages
    for (const key of ['h1', 'h2', 'h3']) {
      headings[key] = headings[key].slice(0, 20).map(h => h.substring(0, 200));
    }

    // --- VISIBLE TEXT ---
    const body = doc.body?.cloneNode(true);
    if (body) {
      body.querySelectorAll("script, style, noscript, svg, iframe").forEach(el => el.remove());
    }
    const visibleText = body ? body.textContent.replace(/\s+/g, " ").trim() : "";
    const wordCount = visibleText.split(/\s+/).filter(w => w.length > 0).length;

    // --- SSR DETECTION ---
    const hasEmptyRoot = !!doc.querySelector('#root:empty, #app:empty, #__next:empty');
    const htmlSize = html.length;
    const isSPA = hasEmptyRoot && htmlSize < 15000;

    // --- SCHEMA.ORG (JSON-LD) ---
    const schemas = [];
    doc.querySelectorAll('script[type="application/ld+json"]').forEach(s => {
      try {
        const data = JSON.parse(s.textContent);
        const items = Array.isArray(data) ? data : [data];
        items.forEach(item => {
          if (item["@graph"]) {
            item["@graph"].forEach(g => schemas.push(extractSchema(g)));
          } else {
            schemas.push(extractSchema(item));
          }
        });
      } catch (e) { /* malformed JSON-LD */ }
    });

    // Limit schemas
    const limitedSchemas = schemas.slice(0, 20);

    // --- LINKS ---
    const internalLinks = [];
    const externalLinks = [];
    const currentHost = location.hostname;
    doc.querySelectorAll("a[href]").forEach(a => {
      try {
        const url = new URL(a.href, location.href);
        if (url.hostname === currentHost) {
          internalLinks.push(url.pathname);
        } else {
          externalLinks.push(url.hostname);
        }
      } catch (e) { /* invalid URL */ }
    });

    // --- IMAGES ---
    const images = [];
    doc.querySelectorAll("img").forEach(img => {
      images.push({
        src: img.src?.substring(0, 200),
        alt: img.alt || "",
        hasAlt: img.hasAttribute("alt"),
        loading: img.loading || "",
        width: img.naturalWidth || img.width,
        height: img.naturalHeight || img.height
      });
    });
    const imagesWithoutAlt = images.filter(i => !i.hasAlt || i.alt === "").length;
    // Limit images to prevent oversized messages
    const limitedImages = images.slice(0, 100);

    // --- GEO: CITABLE DATA ---
    const tables = doc.querySelectorAll("table").length;
    const lists = doc.querySelectorAll("ul, ol").length;
    const definitionLists = doc.querySelectorAll("dl").length;
    const blockquotes = doc.querySelectorAll("blockquote").length;
    const codeBlocks = doc.querySelectorAll("pre, code").length;

    // Stats / numbers in text
    const statsPattern = /\d+[\.,]?\d*\s*(%|€|\$|£|millions?|milliards?|billions?|users?|clients?)/gi;
    const statsFound = (visibleText.match(statsPattern) || []).length;

    // FAQ patterns
    const faqPatterns = doc.querySelectorAll('[itemtype*="FAQPage"], [itemtype*="Question"], details, .faq, .accordion, [id*="faq"]').length;

    // --- E-E-A-T SIGNALS ---
    const hasAboutPage = !!doc.querySelector('a[href*="about"], a[href*="a-propos"], a[href*="qui-sommes"], a[href*="equipe"]');
    const hasContactPage = !!doc.querySelector('a[href*="contact"]');
    const hasPrivacyPage = !!doc.querySelector('a[href*="privacy"], a[href*="confidentialite"], a[href*="politique"], a[href*="rgpd"], a[href*="legal"]');
    const hasAuthorInfo = !!doc.querySelector('[rel="author"], .author, [itemprop="author"], .byline');
    const hasTestimonials = !!doc.querySelector('.testimonial, .review, .avis, [itemprop="review"]');

    // --- PERFORMANCE HINTS ---
    const perfData = {};
    if (window.performance) {
      const nav = performance.getEntriesByType("navigation")[0];
      if (nav) {
        perfData.domContentLoaded = Math.round(nav.domContentLoadedEventEnd - nav.startTime);
        perfData.loadComplete = Math.round(nav.loadEventEnd - nav.startTime);
        perfData.ttfb = Math.round(nav.responseStart - nav.startTime);
        perfData.domInteractive = Math.round(nav.domInteractive - nav.startTime);
        perfData.transferSize = nav.transferSize;
      }
    }

    // --- HTTPS ---
    const isHTTPS = location.protocol === "https:";

    // --- CANONICAL COHERENCE ---
    const canonicalMismatch = canonical && canonical !== location.href && canonical !== location.href.replace(/\/$/, '');
    const canonicalDomain = canonical ? new URL(canonical).hostname : "";
    const currentDomain = location.hostname;
    const canonicalDomainMismatch = canonicalDomain && canonicalDomain !== currentDomain && canonicalDomain !== `www.${currentDomain}` && `www.${canonicalDomain}` !== currentDomain;

    // --- HREFLANG ---
    const hreflangs = [];
    doc.querySelectorAll('link[rel="alternate"][hreflang]').forEach(l => {
      hreflangs.push({ lang: l.hreflang, href: l.href });
    });

    // --- CORE WEB VITALS ---
    const cwv = {};
    try {
      const paintEntries = performance.getEntriesByType("paint");
      const fcp = paintEntries.find(e => e.name === "first-contentful-paint");
      if (fcp) cwv.fcp = Math.round(fcp.startTime);

      const lcpEntries = performance.getEntriesByType("largest-contentful-paint");
      if (lcpEntries.length > 0) cwv.lcp = Math.round(lcpEntries[lcpEntries.length - 1].startTime);

      // CLS from layout-shift entries
      const lsEntries = performance.getEntriesByType("layout-shift");
      if (lsEntries.length > 0) {
        cwv.cls = lsEntries.reduce((sum, e) => sum + (e.hadRecentInput ? 0 : e.value), 0);
        cwv.cls = Math.round(cwv.cls * 1000) / 1000;
      }
    } catch(e) { /* CWV unavailable */ }

    // --- FAVICON ---
    const hasFavicon = !!doc.querySelector('link[rel="icon"], link[rel="shortcut icon"], link[rel="apple-touch-icon"]');

    // --- CONTENT FRESHNESS ---
    const lastModifiedMeta = doc.querySelector('meta[name="last-modified"]')?.content || doc.querySelector('meta[http-equiv="last-modified"]')?.content || "";
    const articleDate = doc.querySelector('time[datetime]')?.getAttribute("datetime") || doc.querySelector('[itemprop="datePublished"]')?.content || "";

    // --- SOCIAL LINKS ---
    const socialPlatforms = ["linkedin", "facebook", "twitter", "instagram", "youtube", "tiktok", "github"];
    const socialLinks = [...new Set(
      Array.from(doc.querySelectorAll('a[href]'))
        .map(a => { try { return new URL(a.href).hostname; } catch(e) { return ""; } })
        .filter(h => socialPlatforms.some(p => h.includes(p)))
    )];

    // --- AI SIGNALS ---
    const hasLlmsTxtLink = !!doc.querySelector('link[rel="llms-txt"], a[href*="llms.txt"]');

    // --- BASIC A11Y ---
    const hasSkipLink = !!doc.querySelector('a[href="#main"], a[href="#content"], .skip-link, .skip-nav');
    const ariaLandmarks = doc.querySelectorAll('[role="main"], [role="navigation"], [role="banner"], [role="contentinfo"], main, nav, header, footer').length;

    return {
      url: location.href,
      domain: location.hostname,
      protocol: location.protocol,
      isHTTPS,
      title,
      titleLength: title.length,
      metaDesc,
      metaDescLength: metaDesc.length,
      canonical,
      viewport,
      charset,
      lang,
      og,
      twitter,
      headings,
      visibleText: visibleText.substring(0, 500), // truncated for message size
      wordCount,
      htmlSize,
      isSPA,
      schemas: limitedSchemas,
      internalLinksCount: internalLinks.length,
      externalLinksCount: externalLinks.length,
      externalDomains: [...new Set(externalLinks)].slice(0, 20),
      imagesTotal: images.length,
      imagesWithoutAlt,
      imagesLazy: limitedImages.filter(i => i.loading === "lazy").length,
      tables, lists, definitionLists, blockquotes, codeBlocks,
      statsFound, faqPatterns,
      hasAboutPage, hasContactPage, hasPrivacyPage, hasAuthorInfo, hasTestimonials,
      perfData,
      canonicalMismatch, canonicalDomainMismatch, canonicalDomain,
      hreflangs,
      cwv,
      hasFavicon,
      lastModifiedMeta, articleDate,
      socialLinks,
      hasLlmsTxtLink,
      hasSkipLink, ariaLandmarks
    };
  }

  function extractSchema(item) {
    return {
      type: item["@type"] || "unknown",
      name: item.name || item.headline || "",
      url: item.url || "",
      keys: Object.keys(item).slice(0, 15)
    };
  }
})();
