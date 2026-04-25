// CBOR-Web Audit Desktop — UI Logic
async function runAudit() {
  const url = document.getElementById('url').value;
  document.getElementById('output').textContent = 'Analyzing ' + url + '...';
  
  try {
    const resp = await fetch(url, { signal: AbortSignal.timeout(10000) });
    const html = await resp.text();
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, 'text/html');
    
    const title = doc.querySelector('title')?.textContent || 'No title';
    const metaDesc = doc.querySelector('meta[name="description"]')?.getAttribute('content') || '';
    const h1s = [...doc.querySelectorAll('h1')].length;
    const imgs = doc.querySelectorAll('img').length;
    const imgsNoAlt = [...doc.querySelectorAll('img:not([alt])')].length;
    const words = (doc.body?.textContent || '').split(/\s+/).length;
    const hasSchema = !!doc.querySelector('script[type="application/ld+json"]');
    const hasCBOR = html.includes('/.well-known/cbor-web') || html.includes('application/cbor');
    const links = [...doc.querySelectorAll('a[href]')].length;
    const internalLinks = [...doc.querySelectorAll('a[href^="/"],a[href^="'+new URL(url).origin+'"]')].length;
    
    let score = 100;
    const issues = [];
    if (!title) { score -= 15; issues.push('Missing title'); }
    if (title.length < 10 || title.length > 70) { score -= 5; issues.push('Title length suboptimal'); }
    if (!metaDesc) { score -= 10; issues.push('Missing meta description'); }
    if (h1s === 0) { score -= 10; issues.push('No H1 tag'); }
    if (h1s > 1) { score -= 5; issues.push('Multiple H1 tags'); }
    if (imgsNoAlt > 0) { score -= imgsNoAlt * 2; issues.push(imgsNoAlt + ' images without alt'); }
    if (words < 300) { score -= 10; issues.push('Thin content (<300 words)'); }
    if (!hasSchema) { score -= 5; issues.push('No Schema.org structured data'); }
    if (!hasCBOR) { score -= 0; issues.push('CBOR-Web not detected (optional)'); }
    if (links < 5) { score -= 5; issues.push('Very few links'); }
    
    score = Math.max(0, Math.min(100, score));
    
    document.getElementById('score').innerHTML = `<span style="color:${score>70?'#10b981':score>40?'#f59e0b':'#ef4444'}">${score}</span>/100`;
    document.getElementById('cbor-check').textContent = hasCBOR ? '✅ CBOR-Web detected' : '❌ Not detected';
    document.getElementById('page-count').textContent = words + ' words · ' + imgs + ' images · ' + links + ' links';
    document.getElementById('issue-count').textContent = issues.length + ' issues found';
    document.getElementById('output').textContent = 'URL: ' + url + '\nTitle: ' + title + '\nWords: ' + words + '\nImages: ' + imgs + '\nSchema.org: ' + hasSchema + '\nCBOR-Web: ' + hasCBOR + '\n\nIssues:\n' + issues.map(i => '  • ' + i).join('\n') + '\n\nScore: ' + score + '/100';
  } catch (e) {
    document.getElementById('output').textContent = 'Error: ' + e.message;
  }
}

async function runCrawl() {
  const base = document.getElementById('url').value;
  document.getElementById('output').textContent = 'Crawling site starting from ' + base + '...';
  const crawler = { visited: new Set(), queue: [base], results: [], maxPages: 50, async fetch(url) {
    try { const r = await fetch(url, { signal: AbortSignal.timeout(5000) }); const t = await r.text(); const d = new DOMParser().parseFromString(t,'text/html');
    return { url, title: d.querySelector('title')?.textContent||'', words: (d.body?.textContent||'').split(/\s+/).length, links: [...d.querySelectorAll('a[href]')].map(a=>a.getAttribute('href')), status: r.status }; } catch(e) { return null; }
  }};
  while (crawler.queue.length > 0 && crawler.visited.size < crawler.maxPages) {
    const batch = crawler.queue.splice(0, 5);
    const pages = await Promise.all(batch.map(u => crawler.fetch(u)));
    for (const p of pages) { if (p) { crawler.visited.add(p.url); crawler.results.push(p);
      (p.links||[]).forEach(l => { try { const abs = new URL(l, p.url).href; if (!crawler.visited.has(abs) && new URL(abs).hostname === new URL(base).hostname) crawler.queue.push(abs); } catch(e){} }); } }
  }
  const broken = crawler.results.filter(p => p.status !== 200);
  document.getElementById('page-count').textContent = crawler.results.length + ' pages crawled';
  document.getElementById('issue-count').textContent = broken.length + ' broken';
  document.getElementById('output').textContent = 'Crawled ' + crawler.results.length + ' pages\nBroken: ' + broken.length + '\n\n' + crawler.results.map(p => p.url + ' — ' + (p.title||'No title') + ' (' + p.words + ' words)').join('\n');
}

function exportHTML() {
  const html = `<!DOCTYPE html><html><head><title>CBOR-Web Audit Report</title><style>body{font-family:Arial;max-width:800px;margin:40px auto}h1{color:#d4540b}.score{font-size:3rem}</style></head><body><h1>CBOR-Web Audit Report</h1><pre>${document.getElementById('output').textContent}</pre></body></html>`;
  const blob = new Blob([html],{type:'text/html'}); const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = 'audit-report.html'; a.click();
}
