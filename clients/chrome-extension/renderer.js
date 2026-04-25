// ============================================================
// DELTOPIDE WEB AUDIT — Shared Renderer
// Used by both popup.html and sidepanel.html
// ============================================================

const PILLAR_LABELS = {
  TECH: "Fondations Techniques",
  SEO: "Contenu & Autorité",
  GEO: "Visibilité IA",
  SXG: "SXG / CBOR-Web"
};

const PILLAR_ICONS = {
  TECH: "⚙",
  SEO: "📝",
  GEO: "🤖",
  SXG: "🔗"
};

function scoreClass(pct) {
  if (pct >= 70) return "high";
  if (pct >= 40) return "mid";
  return "low";
}

function statusIcon(status) {
  if (status === "OK") return '<div class="check-icon ok">✓</div>';
  if (status === "A_CORRIGER") return '<div class="check-icon warn">!</div>';
  return '<div class="check-icon absent">✗</div>';
}

function badgeClass(cat) {
  return `badge-${cat.toLowerCase()}`;
}

function renderHeader(url) {
  return `
    <div class="header">
      <div class="header-brand">
        <div class="header-logo">D</div>
        <div class="header-title"><span>Deltopide</span> Audit</div>
      </div>
      <div class="header-url">${escapeHtml(url)}</div>
    </div>
  `;
}

function renderScores(summary) {
  const cats = ["TECH", "SEO", "GEO", "SXG"];
  let html = '<div class="score-grid">';

  for (const cat of cats) {
    const s = summary[cat];
    const cls = scoreClass(s.pct);
    html += `
      <div class="score-card fade-in stagger-${cats.indexOf(cat) + 1}">
        <div class="score-label">${PILLAR_ICONS[cat]} ${PILLAR_LABELS[cat]}</div>
        <div class="score-value ${cls}">${s.pct}<span>%</span></div>
        <div class="score-bar">
          <div class="score-bar-fill ${cls}" style="width: ${s.pct}%"></div>
        </div>
      </div>
    `;
  }

  // Total
  const t = summary.TOTAL;
  const tcls = scoreClass(t.pct);
  html += `
    <div class="score-card total fade-in stagger-5">
      <div class="score-label">Score Global</div>
      <div class="score-value ${tcls}" style="font-size: 32px;">${t.pct}<span>% — ${t.ok}/${t.total}</span></div>
      <div class="score-bar">
        <div class="score-bar-fill ${tcls}" style="width: ${t.pct}%"></div>
      </div>
    </div>
  `;

  html += '</div>';
  return html;
}

function renderPriorities(priorities) {
  if (!priorities || priorities.length === 0) return '';

  let html = `
    <div class="priorities">
      <div class="section-title">Actions Prioritaires</div>
  `;

  for (const p of priorities) {
    html += `
      <div class="priority-item">
        <div class="priority-badge ${badgeClass(p.category)}">${p.category}</div>
        <div class="priority-text"><strong>${escapeHtml(p.name)}</strong> — ${escapeHtml(p.detail)}</div>
      </div>
    `;
  }

  html += '</div>';
  return html;
}

function renderChecks(checks) {
  const cats = ["TECH", "SEO", "GEO", "SXG"];
  let html = '<div class="checks-section">';

  for (const cat of cats) {
    const catChecks = checks.filter(c => c.category === cat);
    if (catChecks.length === 0) continue;

    const okCount = catChecks.filter(c => c.status === "OK").length;

    html += `
      <div class="pillar-group">
        <div class="pillar-header" onclick="this.parentElement.classList.toggle('collapsed')">
          <span class="pillar-name">${PILLAR_ICONS[cat]} ${PILLAR_LABELS[cat]}</span>
          <span class="pillar-count">${okCount}/${catChecks.length}</span>
        </div>
        <div class="pillar-checks">
    `;

    for (const c of catChecks) {
      html += `
        <div class="check-item">
          ${statusIcon(c.status)}
          <div class="check-info">
            <div class="check-name">${escapeHtml(c.name)}</div>
            <div class="check-detail">${escapeHtml(c.detail)}</div>
          </div>
        </div>
      `;
    }

    html += '</div></div>';
  }

  html += '</div>';
  return html;
}

function renderFooter(result) {
  const stamp = result ? new Date(result.timestamp).toLocaleString("fr-FR") : "";
  return `
    <div class="footer">
      <span class="footer-text">Deltopide v1.1${stamp ? ` — ${stamp}` : ""}</span>
      <a href="https://deltopide.fr" target="_blank" class="footer-link">deltopide.fr</a>
    </div>
  `;
}

function renderLoading() {
  return `
    <div class="loading">
      <div class="spinner"></div>
      <div class="loading-text">Audit en cours<span class="loading-dots"></span></div>
    </div>
  `;
}

function renderIdle(mode) {
  const btnExtra = mode === "popup"
    ? `<button class="btn btn-secondary btn-sm" id="btn-sidepanel">Ouvrir le panneau</button>`
    : '';

  return `
    <div class="idle">
      <div class="idle-icon">🔍</div>
      <div class="idle-text">Cliquez pour auditer cette page</div>
      <button class="btn btn-primary btn-full" id="btn-audit">Lancer l'audit</button>
      ${btnExtra}
    </div>
  `;
}

function renderGSCButton() {
  return `
    <div class="gsc-section">
      <div class="section-title">Google Search Console</div>
      <button class="btn btn-gsc btn-full" id="btn-gsc-submit">
        <span class="gsc-icon">G</span> Soumettre a Google Search Console
      </button>
      <div id="gsc-status"></div>
    </div>
  `;
}

function renderGSCStatus(steps) {
  let html = '<div class="gsc-results">';
  for (const s of steps) {
    const icon = s.status === "ok" ? '<span class="gsc-step-ok">&#10003;</span>'
      : s.status === "error" ? '<span class="gsc-step-err">&#10007;</span>'
      : s.status === "warn" ? '<span class="gsc-step-warn">!</span>'
      : '<span class="gsc-step-info">i</span>';
    html += `<div class="gsc-step">${icon} ${escapeHtml(s.step)}`;
    if (s.detail) html += ` <span class="gsc-detail">${escapeHtml(s.detail)}</span>`;
    html += `</div>`;
  }
  html += '</div>';
  return html;
}

function renderFullReport(result, mode) {
  let html = renderHeader(result.url);
  html += renderScores(result.summary);
  html += renderPriorities(result.priorities);

  if (mode === "popup") {
    html += `
      <div class="actions">
        <button class="btn btn-secondary btn-sm btn-full" id="btn-sidepanel">Vue detaillee</button>
        <button class="btn btn-secondary btn-sm btn-full" id="btn-export">Exporter JSON</button>
      </div>
    `;
    html += renderGSCButton();
  } else {
    html += renderChecks(result.checks);
    html += `
      <div class="actions">
        <button class="btn btn-secondary btn-sm btn-full" id="btn-export">Exporter JSON</button>
        <button class="btn btn-secondary btn-sm btn-full" id="btn-export-csv">Exporter CSV</button>
        <button class="btn btn-secondary btn-sm btn-full" id="btn-reaudit">Relancer</button>
      </div>
    `;
    html += renderGSCButton();
  }

  html += renderFooter(result);
  return html;
}

function escapeHtml(str) {
  if (!str) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function exportJSON(result) {
  const blob = new Blob([JSON.stringify(result, null, 2)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `audit-${result.domain}-${new Date().toISOString().slice(0, 10)}.json`;
  a.click();
  URL.revokeObjectURL(url);
}

function exportCSV(result) {
  const rows = [["Pilier", "Check", "Statut", "Detail"]];
  for (const c of result.checks) {
    rows.push([c.category, c.name, c.status, c.detail.replace(/"/g, '""')]);
  }
  const csv = rows.map(r => r.map(v => `"${v}"`).join(",")).join("\n");
  const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `audit-${result.domain}-${new Date().toISOString().slice(0, 10)}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}
