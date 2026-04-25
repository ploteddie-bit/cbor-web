// CBOR-Web Audit v2.0 — Competitor Comparison

class CompetitorCompare {
  constructor(yourData, competitorUrl) {
    this.yourData = yourData;
    this.compUrl = competitorUrl;
  }

  async compare() {
    const crawler = new SiteCrawler(this.compUrl);
    const compData = await crawler.crawl();
    
    return {
      yourSite: this.summarize(this.yourData),
      competitor: this.summarize(compData),
      gaps: this.findGaps(this.yourData, compData),
      advantage: this.findAdvantages(this.yourData, compData),
    };
  }

  summarize(data) {
    const s = data.summary || data;
    return { pages: s.totalPages || 0, avgWords: s.avgWordCount || 0, schemaOrg: s.withSchemaOrg || 0, cborWeb: s.withCBORWeb || 0, score: s.score || 0 };
  }

  findGaps(yours, theirs) {
    const yourUrls = new Set((yours.pages || []).map(p => p.url));
    const theirUrls = new Set((theirs.pages || []).map(p => p.url));
    // Content gaps: pages they have that you don't
    const theirWords = new Set();
    (theirs.pages || []).forEach(p => (p.title || '').toLowerCase().split(/\s+/).forEach(w => w.length > 3 && theirWords.add(w)));
    const yourWords = new Set();
    (yours.pages || []).forEach(p => (p.title || '').toLowerCase().split(/\s+/).forEach(w => w.length > 3 && yourWords.add(w)));
    const missing = [...theirWords].filter(w => !yourWords.has(w)).slice(0, 20);
    return { missingKeywords: missing, theyHavePages: theirUrls.size - [...theirUrls].filter(u => yourUrls.has(u)).length };
  }

  findAdvantages(yours, theirs) {
    const advantages = [];
    if ((yours.summary?.withCBORWeb || 0) > (theirs.summary?.withCBORWeb || 0)) advantages.push('More CBOR-Web pages → AI agents prefer your site');
    if ((yours.summary?.score || 0) > (theirs.summary?.score || 0)) advantages.push('Higher overall score');
    if ((yours.summary?.avgWordCount || 0) > (theirs.summary?.avgWordCount || 0)) advantages.push('Richer content (higher word count)');
    return advantages;
  }
}
