// ============================================================
// DELTOPIDE WEB AUDIT — Popup Controller
// ============================================================

const app = document.getElementById("app");
let currentResult = null;

// License activation screen
function showLicenseScreen(reason) {
  const msg = reason === "expired" ? "Licence expirée. Contactez Deltopide pour renouveler."
    : reason === "invalid_key" ? "Clé invalide. Vérifiez et réessayez."
    : "Activez votre licence pour utiliser Deltopide Audit.";
  app.innerHTML = `
    <div class="header">
      <div class="header-brand">
        <div class="header-logo">D</div>
        <div class="header-title"><span>Deltopide</span> Audit</div>
      </div>
    </div>
    <div class="idle">
      <div class="idle-icon">🔐</div>
      <div class="idle-text">${escapeHtml(msg)}</div>
      <input type="text" id="license-key" placeholder="Clé de licence DELTOPIDE-XXXX"
        style="width:100%;padding:10px;margin:10px 0;border:1px solid var(--border);border-radius:var(--radius);background:var(--bg-input);color:var(--text-primary);font-family:'JetBrains Mono',monospace;font-size:12px;text-align:center;">
      <button class="btn btn-primary btn-full" id="btn-activate">Activer</button>
      <div id="license-error" style="color:var(--absent);font-size:11px;margin-top:8px;display:none;"></div>
    </div>
  `;
  document.getElementById("btn-activate").addEventListener("click", async () => {
    const key = document.getElementById("license-key").value.trim();
    if (!key) return;
    const result = await chrome.runtime.sendMessage({ action: "activateLicense", key });
    if (result.valid) {
      init(); // Reload
    } else {
      const errEl = document.getElementById("license-error");
      errEl.textContent = "Clé invalide ou expirée.";
      errEl.style.display = "block";
    }
  });
  // Allow Enter key
  document.getElementById("license-key").addEventListener("keydown", (e) => {
    if (e.key === "Enter") document.getElementById("btn-activate").click();
  });
}

// Check if we have a cached result for this tab
async function init() {
  // License gate — FREE mode (désactivé pour développement)

  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.url || tab.url.startsWith("chrome://")) {
    app.innerHTML = `
      <div class="header">
        <div class="header-brand">
          <div class="header-logo">D</div>
          <div class="header-title"><span>Deltopide</span> Audit</div>
        </div>
      </div>
      <div class="idle">
        <div class="idle-icon">🚫</div>
        <div class="idle-text">Impossible d'auditer cette page.<br>Naviguez vers un site web.</div>
      </div>
    `;
    return;
  }

  // Check for cached result
  const cached = await chrome.storage.session.get(tab.url);
  if (cached[tab.url]) {
    currentResult = cached[tab.url];
    showResult();
  } else {
    showIdle(tab);
  }
}

function showIdle(tab) {
  app.innerHTML = `
    <div class="header">
      <div class="header-brand">
        <div class="header-logo">D</div>
        <div class="header-title"><span>Deltopide</span> Audit</div>
      </div>
      <div class="header-url">${escapeHtml(tab.url)}</div>
    </div>
    ${renderIdle("popup")}
  `;

  document.getElementById("btn-audit").addEventListener("click", () => runAudit(tab));

  const sideBtn = document.getElementById("btn-sidepanel");
  if (sideBtn) {
    sideBtn.addEventListener("click", () => openSidePanel(tab));
  }
}

async function runAudit(tab) {
  app.innerHTML = `
    <div class="header">
      <div class="header-brand">
        <div class="header-logo">D</div>
        <div class="header-title"><span>Deltopide</span> Audit</div>
      </div>
      <div class="header-url">${escapeHtml(tab.url)}</div>
    </div>
    ${renderLoading()}
  `;

  try {
    // Inject content script if needed
    await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      files: ["content.js"]
    });

    // Small delay for injection
    await new Promise(r => setTimeout(r, 200));

    // Get DOM data
    const domResponse = await chrome.tabs.sendMessage(tab.id, { action: "analyzeDOM" });
    if (!domResponse?.success) throw new Error(domResponse?.error || "DOM analysis failed");

    // Run full audit via background
    const result = await chrome.runtime.sendMessage({
      action: "runFullAudit",
      url: tab.url,
      domData: domResponse.data
    });

    // Handle security errors
    if (result.error === "licence_invalide") {
      showLicenseScreen(result.reason);
      return;
    }
    if (result.error === "integrity_fail") {
      app.innerHTML = `<div class="idle"><div class="idle-icon">🛑</div><div class="idle-text">Extension corrompue. Réinstallez depuis deltopide.fr</div></div>`;
      return;
    }
    if (result.error) {
      throw new Error(result.message || result.error);
    }

    currentResult = result;

    // Cache result
    await chrome.storage.session.set({ [tab.url]: result });

    showResult();
  } catch (err) {
    app.innerHTML = `
      <div class="header">
        <div class="header-brand">
          <div class="header-logo">D</div>
          <div class="header-title"><span>Deltopide</span> Audit</div>
        </div>
      </div>
      <div class="idle">
        <div class="idle-icon">⚠️</div>
        <div class="idle-text">Erreur: ${escapeHtml(err.message)}</div>
        <button class="btn btn-primary" id="btn-retry">Réessayer</button>
      </div>
    `;
    document.getElementById("btn-retry")?.addEventListener("click", () => runAudit(tab));
  }
}

function showResult() {
  app.innerHTML = renderFullReport(currentResult, "popup");

  // Bind buttons
  document.getElementById("btn-sidepanel")?.addEventListener("click", async () => {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    openSidePanel(tab);
  });

  document.getElementById("btn-export")?.addEventListener("click", () => {
    exportJSON(currentResult);
  });

  // GSC Submit button
  document.getElementById("btn-gsc-submit")?.addEventListener("click", async () => {
    const statusEl = document.getElementById("gsc-status");
    const btn = document.getElementById("btn-gsc-submit");
    btn.disabled = true;
    btn.textContent = "Soumission en cours...";
    statusEl.innerHTML = '<div class="gsc-loading">Connexion a Google...</div>';

    const siteUrl = new URL(currentResult.url).origin + "/";
    const result = await chrome.runtime.sendMessage({
      action: "gscSubmit",
      siteUrl: siteUrl,
      sitemapUrl: siteUrl + "sitemap.xml"
    });

    if (result.ok) {
      statusEl.innerHTML = renderGSCStatus(result.steps);
      btn.textContent = "Soumis";
      btn.classList.add("btn-gsc-done");
    } else {
      statusEl.innerHTML = `<div class="gsc-error">${escapeHtml(result.error)}</div>`;
      btn.textContent = "Reessayer";
      btn.disabled = false;
    }
  });
}

async function openSidePanel(tab) {
  chrome.runtime.sendMessage({ action: "openSidePanel", tabId: tab.id });
}

init();
