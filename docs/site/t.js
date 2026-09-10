/* t.js — télémétrie first-party anonyme (agent-webmaster cborweb.com)
   Aucun cookie, aucune IP stockée, aucune donnée personnelle. RGPD-friendly.
   Envoi : pageview (load + hashchange SPA), clics (liens/boutons), batch sendBeacon. */
(function () {
  "use strict";
  var ENDPOINT = "https://telemetry.cborweb.com/collect";
  var sid;
  try {
    sid = sessionStorage.getItem("tw-sid");
    if (!sid) { sid = Math.random().toString(36).slice(2) + Date.now().toString(36); sessionStorage.setItem("tw-sid", sid); }
  } catch (e) { sid = "nose-" + Date.now().toString(36); }

  var q = [];
  function ev(type, page, element) {
    var p = new URLSearchParams(location.search);
    q.push({
      type: type, page: page || (location.hash || "#home"), element: (element || "").slice(0, 120),
      referrer: document.referrer.slice(0, 200),
      utm_source: p.get("utm_source") || "", utm_medium: p.get("utm_medium") || "", utm_campaign: p.get("utm_campaign") || "",
      lang: navigator.language || "", screen: screen.width + "x" + screen.height
    });
    if (q.length >= 10) flush();
  }
  function flush() {
    if (!q.length) return;
    var body = JSON.stringify({ sid: sid, events: q.splice(0) });
    if (navigator.sendBeacon && navigator.sendBeacon(ENDPOINT, new Blob([body], { type: "application/json" }))) return;
    fetch(ENDPOINT, { method: "POST", headers: { "Content-Type": "application/json" }, body: body, keepalive: true }).catch(function () {});
  }

  ev("pageview");
  window.addEventListener("hashchange", function () { ev("route"); });
  document.addEventListener("click", function (e) {
    var el = e.target.closest("a,button,[role=button]");
    if (!el) return;
    var label = (el.getAttribute("aria-label") || el.textContent || "").trim().slice(0, 60);
    ev("click", null, (el.tagName.toLowerCase()) + ":" + (el.getAttribute("href") || label || "?"));
  }, { passive: true });
  setInterval(flush, 5000);
  window.addEventListener("beforeunload", flush);
})();
