// CBOR-Web Audit Desktop — with License

async function activateLicense() {
  const email = document.getElementById('licenseEmail').value;
  const key = document.getElementById('licenseKey').value;
  if (!email || !key) { document.getElementById('licenseError').textContent = 'Email and license key required'; return; }
  
  try {
    const result = await window.__TAURI_INTERNALS__.invoke('check_license', { key, email });
    if (result.verified) {
      document.getElementById('licenseScreen').style.display = 'none';
      document.getElementById('mainScreen').style.display = 'block';
      document.getElementById('userInfo').textContent = `Licensed to: ${result.email} · Plan: ${result.plan} · Expires: ${new Date(result.expires_at * 1000).toLocaleDateString()}`;
      document.getElementById('licUser').textContent = result.email;
      document.getElementById('planBadge').textContent = result.plan;
    }
  } catch (e) {
    document.getElementById('licenseError').textContent = typeof e === 'string' ? e : 'Invalid license';
  }
}

// Auto-check saved license on startup
(async () => {
  try {
    const saved = await window.__TAURI_INTERNALS__.invoke('get_saved_license');
    if (saved && saved.verified) {
      document.getElementById('licenseScreen').style.display = 'none';
      document.getElementById('mainScreen').style.display = 'block';
      document.getElementById('userInfo').textContent = `Licensed to: ${saved.email} · Plan: ${saved.plan}`;
      document.getElementById('licUser').textContent = saved.email;
      document.getElementById('planBadge').textContent = saved.plan;
    }
  } catch(e) {}
})();

// Audit functions (same as before)
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
    const hasCBOR = html.includes('/.well-known/cbor-web');
    const links = [...doc.querySelectorAll('a[href]')].length;
    let score = 100;
    const issues = [];
    if (!title) { score -= 15; issues.push('Missing title'); }
    if (!metaDesc) { score -= 10; issues.push('Missing meta description'); }
    if (h1s === 0) { score -= 10; issues.push('No H1 tag'); }
    if (h1s > 1) { score -= 5; issues.push('Multiple H1 tags'); }
    if (imgsNoAlt > 0) { score -= imgsNoAlt * 2; issues.push(imgsNoAlt + ' images without alt'); }
    if (words < 300) { score -= 10; issues.push('Thin content'); }
    if (!hasSchema) { score -= 5; issues.push('No Schema.org'); }
    score = Math.max(0, Math.min(100, score));
    document.getElementById('score').innerHTML = `<span style="color:${score>70?'#10b981':score>40?'#f59e0b':'#ef4444'}">${score}</span>/100`;
    document.getElementById('cbor-check').textContent = hasCBOR ? '✅ Detected' : '❌ Not detected';
    document.getElementById('page-count').textContent = words + ' words';
    document.getElementById('issue-count').textContent = issues.length;
    document.getElementById('output').textContent = 'URL: ' + url + '\nTitle: ' + title + '\nWords: ' + words + '\nSchema.org: ' + hasSchema + '\nCBOR-Web: ' + hasCBOR + '\n\nIssues:\n' + issues.map(i => '  • ' + i).join('\n');
  } catch (e) { document.getElementById('output').textContent = 'Error: ' + e.message; }
}

async function runCrawl() {
  const base = document.getElementById('url').value;
  document.getElementById('output').textContent = 'Crawling ' + base + '...';
  const visited = new Set(); const queue = [base]; const results = [];
  while (queue.length > 0 && visited.size < 50) {
    const batch = queue.splice(0, 5);
    const pages = await Promise.all(batch.map(async u => {
      try { const r = await fetch(u, { signal: AbortSignal.timeout(5000) }); const t = await r.text(); const d = new DOMParser().parseFromString(t,'text/html');
      return { url: u, title: d.querySelector('title')?.textContent||'', words: (d.body?.textContent||'').split(/\s+/).length, status: r.status,
        links: [...d.querySelectorAll('a[href]')].map(a => a.getAttribute('href')) }; } catch(e) { return null; }
    }));
    for (const p of pages) { if (p && !visited.has(p.url)) { visited.add(p.url); results.push(p);
      (p.links||[]).forEach(l => { try { const abs = new URL(l, p.url).href; if (!visited.has(abs) && new URL(abs).hostname === new URL(base).hostname) queue.push(abs); } catch(e){} }); } }
  }
  document.getElementById('page-count').textContent = results.length + ' pages';
  document.getElementById('issue-count').textContent = results.filter(p => p.status !== 200).length + ' broken';
  document.getElementById('output').textContent = 'Crawled ' + results.length + ' pages\n\n' + results.map(p => p.url + ' — ' + (p.title||'No title')).join('\n');
}

function exportHTML() {
  const html = `<!DOCTYPE html><html><head><title>CBOR-Web Audit Report</title><style>body{font-family:Arial;max-width:800px;margin:40px auto}h1{color:#d4540b}</style></head><body><h1>CBOR-Web Audit Report</h1><pre>${document.getElementById('output').textContent}</pre></body></html>`;
  const blob = new Blob([html],{type:'text/html'}); const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = 'audit-report.html'; a.click();
}
