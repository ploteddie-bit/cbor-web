// CBOR-Web Audit v2.0 — Full Site Crawler
// Crawls all internal links, audits each page, builds sitemap, detects issues.

class SiteCrawler {
  constructor(baseUrl) {
    this.base = new URL(baseUrl);
    this.visited = new Set();
    this.queue = [baseUrl];
    this.results = [];
    this.broken = [];
    this.maxPages = 500;
  }

  async crawl() {
    while (this.queue.length > 0 && this.visited.size < this.maxPages) {
      const batch = this.queue.splice(0, 5);
      const pages = await Promise.all(batch.map(url => this.fetchPage(url)));
      for (const page of pages) {
        if (page) {
          this.visited.add(page.url);
          this.results.push(page);
          // Extract internal links
          for (const link of page.internalLinks || []) {
            const abs = new URL(link, page.url).href;
            if (!this.visited.has(abs) && new URL(abs).hostname === this.base.hostname) {
              this.queue.push(abs);
            }
          }
        }
      }
    }
    return this.buildReport();
  }

  async fetchPage(url) {
    try {
      const resp = await fetch(url, { signal: AbortSignal.timeout(8000) });
      if (!resp.ok) {
        this.broken.push({ url, status: resp.status });
        return null;
      }
      const html = await resp.text();
      const parser = new DOMParser();
      const doc = parser.parseFromString(html, 'text/html');
      
      const links = [...doc.querySelectorAll('a[href]')].map(a => a.getAttribute('href'));
      const internal = links.filter(l => l.startsWith('/') || l.startsWith(this.base.origin));
      const external = links.filter(l => l.startsWith('http') && !l.startsWith(this.base.origin));
      
      return {
        url,
        title: doc.querySelector('title')?.textContent || '',
        metaDesc: doc.querySelector('meta[name="description"]')?.getAttribute('content') || '',
        h1: [...doc.querySelectorAll('h1')].map(h => h.textContent),
        wordCount: (doc.body?.textContent || '').split(/\s+/).length,
        hasSchemaOrg: !!doc.querySelector('script[type="application/ld+json"]'),
        hasCBORWeb: html.includes('/.well-known/cbor-web') || html.includes('application/cbor'),
        hasLLMsTxt: false, // would need separate fetch
        internalLinks: internal,
        externalLinks: external,
        imageCount: doc.querySelectorAll('img').length,
        imagesWithoutAlt: [...doc.querySelectorAll('img:not([alt])')].length,
        status: resp.status,
        size: html.length,
        loadTime: 0,
      };
    } catch (e) {
      this.broken.push({ url, status: 0, error: e.message });
      return null;
    }
  }

  buildReport() {
    const total = this.results.length;
    const avgWordCount = Math.round(this.results.reduce((s,p) => s + p.wordCount, 0) / total);
    const withSchema = this.results.filter(p => p.hasSchemaOrg).length;
    const withCBOR = this.results.filter(p => p.hasCBORWeb).length;
    const imagesNoAlt = this.results.reduce((s,p) => s + p.imagesWithoutAlt, 0);
    const avgSize = Math.round(this.results.reduce((s,p) => s + p.size, 0) / total);
    
    const issues = [];
    this.results.forEach(p => {
      if (!p.title) issues.push({ url: p.url, issue: 'Missing title', severity: 'high' });
      if (!p.metaDesc) issues.push({ url: p.url, issue: 'Missing meta description', severity: 'medium' });
      if (p.wordCount < 300) issues.push({ url: p.url, issue: 'Thin content (<300 words)', severity: 'medium' });
      if (p.imagesWithoutAlt > 0) issues.push({ url: p.url, issue: `${p.imagesWithoutAlt} images without alt`, severity: 'low' });
    });
    
    const prioritized = issues.sort((a,b) => ['high','medium','low'].indexOf(a.severity) - ['high','medium','low'].indexOf(b.severity));
    
    return {
      summary: { totalPages: total, brokenPages: this.broken.length, avgWordCount, withSchemaOrg: withSchema, withCBORWeb: withCBOR, imagesNoAlt, avgPageSize: avgSize },
      issues: prioritized.slice(0, 100),
      pages: this.results.map(p => ({ url: p.url, title: p.title, wordCount: p.wordCount, hasSchema: p.hasSchemaOrg, hasCBOR: p.hasCBORWeb })),
      broken: this.broken,
      score: Math.round(Math.max(0, 100 - prioritized.length * 2 - this.broken.length * 5)),
    };
  }
}
