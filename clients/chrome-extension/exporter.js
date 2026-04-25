// CBOR-Web Audit v2.0 — Report Exporter

class ReportExporter {
  constructor(auditData) {
    this.data = auditData;
  }

  toCSV() {
    const rows = ['url,title,wordCount,hasSchema,hasCBORWeb,issueCount'];
    for (const page of (this.data.pages || [])) {
      const issues = (this.data.issues || []).filter(i => i.url === page.url).length;
      rows.push(`${page.url},${page.title},${page.wordCount || 0},${page.hasSchema || false},${page.hasCBOR || false},${issues}`);
    }
    return rows.join('\n');
  }

  toJSON() { return JSON.stringify(this.data, null, 2); }

  toHTML() {
    const s = this.data.summary || {};
    return `<!DOCTYPE html><html><head><title>CBOR-Web Audit Report</title>
<style>body{font-family:Arial,sans-serif;max-width:800px;margin:auto;padding:40px}h1{color:#d4540b}.score{font-size:3rem;font-weight:700}.good{color:#10b981}.medium{color:#f59e0b}.bad{color:#ef4444}table{width:100%;border-collapse:collapse;margin:20px 0}th,td{padding:8px;border:1px solid #ddd;text-align:left}th{background:#f5f5f5}</style></head><body>
<h1>CBOR-Web Audit Report</h1><p>Domain: ${this.data.domain || 'N/A'} | Date: ${new Date().toISOString().split('T')[0]}</p>
<div class="score ${s.score > 80 ? 'good' : s.score > 50 ? 'medium' : 'bad'}">${s.score || 0}/100</div>
<h2>Summary</h2><table><tr><th>Total pages</th><td>${s.totalPages || 0}</td></tr>
<tr><th>Broken pages</th><td>${s.brokenPages || 0}</td></tr>
<tr><th>Avg word count</th><td>${s.avgWordCount || 0}</td></tr>
<tr><th>Schema.org</th><td>${s.withSchemaOrg || 0} pages</td></tr>
<tr><th>CBOR-Web</th><td>${s.withCBORWeb || 0} pages</td></tr></table>
<h2>Issues</h2><table><tr><th>URL</th><th>Issue</th><th>Severity</th></tr>
${(this.data.issues || []).slice(0, 50).map(i => `<tr><td>${i.url}</td><td>${i.issue}</td><td style="color:${i.severity==='high'?'#ef4444':i.severity==='medium'?'#f59e0b':'#10b981'}">${i.severity}</td></tr>`).join('')}</table>
</body></html>`;
  }

  downloadCSV() { this._download(this.toCSV(), 'audit-report.csv', 'text/csv'); }
  downloadJSON() { this._download(this.toJSON(), 'audit-report.json', 'application/json'); }
  downloadHTML() { this._download(this.toHTML(), 'audit-report.html', 'text/html'); }

  _download(content, filename, type) {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a'); a.href = url; a.download = filename; a.click();
    URL.revokeObjectURL(url);
  }
}
