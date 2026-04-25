//! cbor-server — CBOR-Web HTTP server (reference implementation)
//!
//! Serves CBOR-Web endpoints following the v2.1 specification:
//! - GET  /.well-known/cbor-web              → manifest
//! - GET  /.well-known/cbor-web/pages/:file  → individual page CBOR
//! - GET  /.well-known/cbor-web/bundle       → full site bundle
//! - POST /.well-known/cbor-web/doleance     → agent feedback receiver
//! - GET  /.well-known/cbor-web/doleance/list → retrieve collected feedback
//! - GET  /.well-known/cbor-web/diff         → incremental diff (?base=HASH)
//!
//! Designed to run on serveur-dev (10.0.0.201) behind a Cloudflare Worker proxy.

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use clap::Parser;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "cbor-server", version, about = "CBOR-Web HTTP server")]
struct Cli {
    #[arg(short, long, default_value = "data")]
    data: PathBuf,

    #[arg(short, long, default_value = "0.0.0.0:3001")]
    listen: String,

    #[arg(long)]
    token: Option<String>,

    #[arg(long, default_value = "10")]
    rate_limit: u32,
}

// ── Short code mapping (sync with worker.js SHORT) ──

const SHORT_CODES: &[(&str, &str)] = &[
    ("lfr", "laforetnousregale.fr"),
    ("dtp", "deltopide.com"),
    ("cbw", "cbor-web.com"),
    ("cb2", "cborweb.com"),
    ("cbo", "cborweb.org"),
    ("cbs", "cbor-web.site"),
    ("cbt", "cbor-web.tech"),
    ("cbf", "cbor-web.fr"),
    ("edv", "explodev.com"),
    ("edf", "explodev.fr"),
    ("edo", "explodev.org"),
    ("eds", "explodev.site"),
    ("edt", "explodev.tech"),
    ("edw", "explodev.website"),
    ("vta", "verdetao.fr"),
    ("vtb", "verdetao.be"),
    ("vtd", "verdetao.de"),
    ("vte", "verdetao.eu"),
    ("vts", "verdetao.es"),
    ("cbd", "californiacbd.fr"),
    ("cbe", "californiacbd.es"),
    ("clc", "californialovecbd.es"),
    ("cls", "californialovecbd.site"),
    ("cle", "californialove.es"),
    ("mjc", "mariejeannecbd.fr"),
    ("mje", "mariejeannecbd.es"),
    ("fcc", "fanaticodelcbd.com"),
    ("fce", "fanaticodelcbd.es"),
    ("bcc", "bienestarcosmeticacbd.es"),
    ("bcf", "bienetrecosmetiquecbd.fr"),
    ("amz", "amazingcbd.es"),
    ("cas", "castelloconviu.es"),
    ("cgm", "cargamipatinete.es"),
    ("ptp", "preciotupatinete.es"),
    ("rtc", "ritueletcalme.com"),
    ("cau", "courtiers-auto.com"),
    ("dts", "deltopide.site"),
    ("wbc", "wellbeingcosmeticcbd.com"),
];

fn short_for_domain(domain: &str) -> Option<&'static str> {
    SHORT_CODES
        .iter()
        .find(|(_, d)| *d == domain)
        .map(|(c, _)| *c)
}

// ── State ──

struct RateLimiter {
    max_per_sec: u32,
    buckets: Mutex<HashMap<String, (Instant, u32)>>,
    last_prune: Mutex<Instant>,
}

impl RateLimiter {
    fn new(max_per_sec: u32) -> Self {
        Self {
            max_per_sec,
            buckets: Mutex::new(HashMap::new()),
            last_prune: Mutex::new(Instant::now()),
        }
    }

    async fn check(&self, ip: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        let entry = buckets.entry(ip.to_string()).or_insert((now, 0));
        if now.duration_since(entry.0) >= Duration::from_secs(1) {
            *entry = (now, 1);
            self.prune_if_needed(&mut buckets, now);
            true
        } else if entry.1 < self.max_per_sec {
            entry.1 += 1;
            true
        } else {
            false
        }
    }

    fn prune_if_needed(&self, buckets: &mut HashMap<String, (Instant, u32)>, now: Instant) {
        if buckets.len() < 5000 {
            return;
        }
        let mut last_prune = match self.last_prune.try_lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if now.duration_since(*last_prune) < Duration::from_secs(60) {
            return;
        }
        *last_prune = now;
        buckets.retain(|_, (t, _)| now.duration_since(*t) < Duration::from_secs(300));
    }
}

struct AppState {
    data_dir: PathBuf,
    token: Option<String>,
    limiter: RateLimiter,
    doléances: Mutex<Vec<serde_json::Value>>,
    doléance_path: PathBuf,
    page_snapshots: Mutex<HashMap<String, HashMap<String, String>>>,
    started_at: std::time::Instant,
    hit_count: AtomicU64,
}

// ── Rate-limit middleware ──

async fn rate_limit_mw(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let ip = headers
        .get("CF-Connecting-IP")
        .or_else(|| headers.get("X-Forwarded-For"))
        .or_else(|| headers.get("X-Real-IP"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .split(',')
        .next()
        .unwrap_or("unknown")
        .trim()
        .to_string();
    if !state.limiter.check(&ip).await {
        return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
    }
    next.run(request).await
}

// ── Auth middleware ──

async fn auth_mw(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    if state.token.is_none() {
        return next.run(request).await;
    }
    let provided = headers
        .get("X-CBOR-Web-Wallet")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if provided != state.token.as_deref().unwrap_or("") {
        return (StatusCode::PAYMENT_REQUIRED, "Token required").into_response();
    }
    next.run(request).await
}

// ── Hit counter middleware ──

async fn hit_counter_mw(
    State(state): State<Arc<AppState>>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    state.hit_count.fetch_add(1, Ordering::Relaxed);
    next.run(request).await
}

// ── Manifest endpoint ──

async fn serve_manifest(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    method: Method,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }
    let dir = host_dir(&state, &headers);
    let paths = [
        dir.join(".well-known/cbor-web/manifest.cbor"),
        dir.join("index.cbor"),
    ];
    for p in &paths {
        if let Some(resp) = serve_file(p).await {
            return resp;
        }
    }
    (StatusCode::NOT_FOUND, "Not Found").into_response()
}

async fn serve_manifest_json(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let dir = host_dir(&state, &headers);
    let paths = [
        dir.join(".well-known/cbor-web/manifest.cbor"),
        dir.join("index.cbor"),
    ];
    for p in &paths {
        if let Ok(data) = tokio::fs::read(p).await {
            if let Ok(value) = ciborium::from_reader::<ciborium::Value, _>(data.as_slice()) {
                let json_val = cbor_to_json(&value);
                let json_str = serde_json::to_string_pretty(&json_val).unwrap_or_default();
                return (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
                    json_str,
                )
                    .into_response();
            }
        }
    }
    (StatusCode::NOT_FOUND, "Not Found").into_response()
}

// ── Page endpoint ──

async fn serve_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(filename): Path<String>,
    method: Method,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }
    let safe: String = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '.' || *c == '-')
        .collect();
    if safe != filename {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }
    let dir = host_dir(&state, &headers);
    let path = dir.join(".well-known/cbor-web/pages").join(&safe);
    serve_file(&path)
        .await
        .unwrap_or_else(|| (StatusCode::NOT_FOUND, "Not Found").into_response())
}

// ── Bundle endpoint ──

async fn serve_bundle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    method: Method,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }
    let dir = host_dir(&state, &headers);
    let path = dir.join(".well-known/cbor-web/bundle.cbor");
    serve_file(&path)
        .await
        .unwrap_or_else(|| (StatusCode::NOT_FOUND, "Not Found").into_response())
}

// ── Doléance endpoint ──

async fn receive_doleance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    method: Method,
    body: Bytes,
) -> Response {
    if method != Method::POST {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if content_type != "application/cbor" {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Expected Content-Type: application/cbor",
        )
            .into_response();
    }
    let mut entry = serde_json::Map::new();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    entry.insert("received_at".into(), serde_json::json!(ts));
    entry.insert("size".into(), serde_json::json!(body.len()));
    entry.insert("hash".into(), serde_json::json!(sha256_hex(&body)));

    match ciborium::from_reader::<ciborium::Value, _>(&body[..]) {
        Ok(value) => {
            entry.insert("doleance".into(), cbor_to_json(&value));
            tracing::info!("Doléance accepted: {} bytes", body.len());
        }
        Err(e) => {
            entry.insert("parse_error".into(), serde_json::json!(e.to_string()));
            tracing::warn!("Doléance parse failed: {}", e);
        }
    }
    {
        let mut dols = state.doléances.lock().await;
        dols.push(serde_json::Value::Object(entry));
        if dols.len() > 10_000 {
            dols.drain(..1_000);
        }
        // Persist to disk every 10 doléances
        if dols.len() % 10 == 0 {
            let path = state.doléance_path.clone();
            let data = serde_json::to_vec_pretty(&dols.clone()).unwrap_or_default();
            tokio::spawn(async move {
                let _ = tokio::fs::write(&path, &data).await;
            });
        }
    }
    (StatusCode::ACCEPTED, "Doléance received").into_response()
}

// ── Doléance list endpoint (GET) ──

#[derive(serde::Deserialize)]
struct DoleanceListQuery {
    limit: Option<usize>,
}

async fn list_doleances(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DoleanceListQuery>,
    method: Method,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }
    let dols = state.doléances.lock().await;
    let limit = query.limit.unwrap_or(100).min(dols.len());
    let result: Vec<_> = dols.iter().rev().take(limit).cloned().collect();
    let body = serde_json::json!({
        "count": dols.len(),
        "doleances": result,
    });
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_vec_pretty(&body).unwrap_or_default(),
    )
        .into_response()
}

// ── Health endpoint (GET /health) ──

async fn health_check(State(state): State<Arc<AppState>>, method: Method) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }
    let uptime = state.started_at.elapsed().as_secs();
    let dol_count = state.doléances.lock().await.len();
    let hits = state.hit_count.load(Ordering::Relaxed);
    let body = serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": uptime,
        "doleances_received": dol_count,
        "requests_total": hits,
    });
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_vec_pretty(&body).unwrap_or_default(),
    )
        .into_response()
}

// ── Diff endpoint ──

#[derive(serde::Deserialize)]
struct DiffQuery {
    base: Option<String>,
}

async fn serve_diff(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<DiffQuery>,
    method: Method,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }
    let base_hash = match query.base {
        Some(ref h) if h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()) => h.clone(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                "Missing or invalid ?base=<sha256-hex>",
            )
                .into_response()
        }
    };
    let dir = host_dir(&state, &headers);
    let paths = [
        dir.join(".well-known/cbor-web/manifest.cbor"),
        dir.join("index.cbor"),
    ];
    let current_bytes = {
        let mut found = None;
        for p in &paths {
            if let Ok(b) = tokio::fs::read(p).await {
                found = Some(b);
                break;
            }
        }
        match found {
            Some(b) => b,
            None => return not_found(),
        }
    };
    let current_hash = sha256_hex(&current_bytes);

    // Extract current pages
    let current_pages = extract_pages_from_manifest(&current_bytes);

    // Save snapshot for future diffs
    {
        let mut snapshots = state.page_snapshots.lock().await;
        if !snapshots.contains_key(&current_hash) {
            snapshots.insert(current_hash.clone(), current_pages.clone());
        }
    }

    // Find base snapshot
    let base_pages = {
        let snapshots = state.page_snapshots.lock().await;
        snapshots.get(&base_hash).cloned()
    };

    let (added, modified, removed, changes) = match base_pages {
        Some(ref base) => {
            let mut added_list = Vec::new();
            let mut modified_list = Vec::new();
            let mut removed_list = Vec::new();

            for (path, hash) in &current_pages {
                match base.get(path) {
                    None => added_list.push((path.clone(), "added")),
                    Some(old_hash) if old_hash != hash => {
                        modified_list.push((path.clone(), "modified"))
                    }
                    _ => {}
                }
            }
            for path in base.keys() {
                if !current_pages.contains_key(path) {
                    removed_list.push((path.clone(), "removed"));
                }
            }

            let changes: Vec<ciborium::Value> = added_list
                .iter()
                .map(|(p, _)| change_entry(p, "added"))
                .chain(
                    modified_list
                        .iter()
                        .map(|(p, _)| change_entry(p, "modified")),
                )
                .chain(removed_list.iter().map(|(p, _)| change_entry(p, "removed")))
                .collect();

            (
                added_list.len() as u64,
                modified_list.len() as u64,
                removed_list.len() as u64,
                changes,
            )
        }
        None => (0, 0, 0, vec![]),
    };

    let diff = cbor_canonical_map(vec![
        (cv_int(0), cv_text("cbor-web-diff")),
        (cv_int(1), cv_int(1)),
        (
            cv_text("base_version_hash"),
            ciborium::Value::Bytes(hex::decode(&base_hash).unwrap_or_default()),
        ),
        (cv_text("current_hash"), cv_text(&current_hash)),
        (
            cv_text("stats"),
            cbor_canonical_map(vec![
                (cv_text("pages_added"), cv_int(added)),
                (cv_text("pages_modified"), cv_int(modified)),
                (cv_text("pages_removed"), cv_int(removed)),
                (
                    cv_text("total_pages_now"),
                    cv_int(current_pages.len() as u64),
                ),
            ]),
        ),
        (cv_text("changes"), ciborium::Value::Array(changes)),
    ]);
    let mut buf = Vec::new();
    if ciborium::into_writer(&diff, &mut buf).is_ok() {
        return cbor_response(&buf, &format!("\"{}\"", sha256_hex(&buf)));
    }
    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode diff").into_response()
}

fn change_entry(path: &str, action: &str) -> ciborium::Value {
    cbor_canonical_map(vec![
        (cv_text("path"), cv_text(path)),
        (cv_text("action"), cv_text(action)),
    ])
}

fn extract_pages_from_manifest(data: &[u8]) -> HashMap<String, String> {
    let doc = match ciborium::from_reader::<ciborium::Value, _>(data) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };
    let doc = match doc {
        ciborium::Value::Tag(55799, inner) | ciborium::Value::Tag(_, inner) => *inner,
        other => other,
    };
    let pages = if let ciborium::Value::Map(ref entries) = doc {
        // Try v3.0 key 5, then v2.1 key 3
        get_map_key(entries, &ciborium::Value::Integer(5.into()))
            .or_else(|| get_map_key(entries, &ciborium::Value::Integer(3.into())))
    } else {
        None
    };

    let mut result = HashMap::new();
    if let Some(ciborium::Value::Array(arr)) = pages {
        for entry in arr {
            if let ciborium::Value::Map(ref pairs) = entry {
                let path = find_text(pairs, "path").unwrap_or("?").to_string();
                let hash = find_text(pairs, "hash")
                    .or_else(|| find_bytes(pairs, "hash").map(|_| "bytes"))
                    .unwrap_or("?")
                    .to_string();
                result.insert(path, hash);
            }
        }
    }
    result
}

fn get_map_key<'a>(
    entries: &'a [(ciborium::Value, ciborium::Value)],
    key: &ciborium::Value,
) -> Option<&'a ciborium::Value> {
    entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn find_text<'a>(pairs: &'a [(ciborium::Value, ciborium::Value)], key: &str) -> Option<&'a str> {
    for (k, v) in pairs {
        if let (ciborium::Value::Text(k_str), ciborium::Value::Text(v_str)) = (k, v) {
            if k_str == key {
                return Some(v_str.as_str());
            }
        }
    }
    None
}

fn find_bytes<'a>(pairs: &'a [(ciborium::Value, ciborium::Value)], key: &str) -> Option<&'a [u8]> {
    for (k, v) in pairs {
        if let (ciborium::Value::Text(k_str), ciborium::Value::Bytes(b)) = (k, v) {
            if k_str == key {
                return Some(b.as_slice());
            }
        }
    }
    None
}

// ── Helpers ──

fn cv_int(n: u64) -> ciborium::Value {
    ciborium::Value::Integer(n.into())
}
fn cv_text(s: &str) -> ciborium::Value {
    ciborium::Value::Text(s.into())
}

fn cbor_response(data: &[u8], etag: &str) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/cbor"),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        data.to_vec(),
    )
        .into_response()
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "Not Found").into_response()
}

fn host_dir(state: &AppState, headers: &HeaderMap) -> PathBuf {
    let base = state.data_dir.parent().unwrap_or(&state.data_dir);

    // Check X-CBOR-Domain override header (for edge proxy path-based routing)
    // If present but site doesn't exist, return data_dir (→ 404) instead of falling back to Host
    if let Some(domain) = headers.get("X-CBOR-Domain").and_then(|v| v.to_str().ok()) {
        let site_dir = base.join("sites").join(domain);
        if site_dir.exists() {
            return site_dir;
        }
        return state.data_dir.clone(); // X-CBOR-Domain set but no such site → 404
    }

    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_string();

    // Try exact hostname first
    let site_dir = base.join("sites").join(&host);
    if site_dir.exists() {
        return site_dir;
    }
    // Try stripping www. prefix
    if let Some(stripped) = host.strip_prefix("www.") {
        let site_dir = base.join("sites").join(stripped);
        if site_dir.exists() {
            return site_dir;
        }
    }
    // Try stripping subdomain (e.g. cbor.deltopide.com → deltopide.com)
    if let Some(dot_pos) = host.find('.') {
        let parent_domain = &host[dot_pos + 1..];
        if !parent_domain.is_empty() {
            let site_dir = base.join("sites").join(parent_domain);
            if site_dir.exists() {
                return site_dir;
            }
        }
    }
    state.data_dir.clone()
}

async fn serve_file(path: &std::path::Path) -> Option<Response> {
    let data = tokio::fs::read(path).await.ok()?;
    Some(cbor_response(&data, &format!("\"{}\"", sha256_hex(&data))))
}

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

fn cbor_canonical_map(entries: Vec<(ciborium::Value, ciborium::Value)>) -> ciborium::Value {
    let mut pairs: Vec<(Vec<u8>, ciborium::Value, ciborium::Value)> = entries
        .into_iter()
        .map(|(k, v)| {
            let mut buf = Vec::new();
            ciborium::into_writer(&k, &mut buf).expect("failed to encode CBOR map key");
            (buf, k, v)
        })
        .collect();
    pairs.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(&b.0)));
    ciborium::Value::Map(pairs.into_iter().map(|(_, k, v)| (k, v)).collect())
}

fn cbor_to_json(value: &ciborium::Value) -> serde_json::Value {
    match value {
        ciborium::Value::Integer(n) => serde_json::json!(i128::from(*n)),
        ciborium::Value::Text(s) => serde_json::json!(s),
        ciborium::Value::Bool(b) => serde_json::json!(b),
        ciborium::Value::Null => serde_json::json!(null),
        ciborium::Value::Float(f) => serde_json::json!(f),
        ciborium::Value::Bytes(b) => serde_json::json!(hex::encode(b)),
        ciborium::Value::Array(arr) => {
            serde_json::json!(arr.iter().map(cbor_to_json).collect::<Vec<_>>())
        }
        ciborium::Value::Map(entries) => {
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                let key = match k {
                    ciborium::Value::Text(s) => s.clone(),
                    ciborium::Value::Integer(n) => format!("{}", i128::from(*n)),
                    _ => format!("{:?}", k),
                };
                map.insert(key, cbor_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        ciborium::Value::Tag(tag, inner) => {
            if *tag == 1 {
                cbor_to_json(inner)
            } else {
                serde_json::json!({"_tag": tag, "_value": cbor_to_json(inner)})
            }
        }
        _ => serde_json::json!(null),
    }
}

// ── Dashboard (GET /) ──

// ── Dashboard (A) ──

async fn serve_dashboard(State(state): State<Arc<AppState>>) -> Response {
    let base = state.data_dir.parent().unwrap_or(&state.data_dir);
    let sites_dir = base.join("sites");
    let mut sites = Vec::new();
    let mut total_pages: u64 = 0;
    let mut total_size: u64 = 0;

    if let Ok(mut entries) = tokio::fs::read_dir(&sites_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let manifest = entry.path().join(".well-known/cbor-web/manifest.cbor");
            let index = entry.path().join("index.cbor");
            let has_manifest = manifest.exists();
            let has_index = index.exists();
            if has_manifest || has_index {
                let dir_path = entry.path();
                let mut page_count: u64 = 0;
                let pages_dir = dir_path.join(".well-known/cbor-web/pages");
                if let Ok(mut pents) = tokio::fs::read_dir(&pages_dir).await {
                    while let Ok(Some(_)) = pents.next_entry().await {
                        page_count += 1;
                    }
                }
                let size = if has_manifest {
                    tokio::fs::metadata(&manifest)
                        .await
                        .map(|m| m.len())
                        .unwrap_or(0)
                } else {
                    tokio::fs::metadata(&index)
                        .await
                        .map(|m| m.len())
                        .unwrap_or(0)
                };
                let format = if has_manifest { "v2.1" } else { "index" };
                let code = short_for_domain(&name).unwrap_or("-");
                total_pages += page_count;
                total_size += size;
                sites.push((name, format.to_string(), size, page_count, code.to_string()));
            }
        }
    }
    sites.sort_by(|a, b| a.0.cmp(&b.0));

    let hits = state.hit_count.load(Ordering::Relaxed);

    let style = "*{margin:0;padding:0;box-sizing:border-box}body{font-family:system-ui,monospace;background:#0a0a0a;color:#e0e0e0;padding:2rem}h1{color:#f97316;margin-bottom:.5rem}h1 span{color:#fff}.sub{color:#888;margin-bottom:1.5rem}.statb{display:flex;gap:2rem;flex-wrap:wrap;margin:1rem 0}.stat{background:#111;padding:1rem;border-radius:8px;min-width:120px}.stat .n{font-size:2rem;color:#f97316;font-weight:700}.stat .l{color:#666;font-size:.8rem}table{width:100%;border-collapse:collapse;margin:1.5rem 0}th{text-align:left;padding:.5rem .8rem;color:#888;border-bottom:1px solid #333;font-size:.75rem;text-transform:uppercase}td{padding:.5rem .8rem;border-bottom:1px solid #1a1a1a;font-size:.85rem}td a,a{color:#f97316;text-decoration:none}td a:hover,a:hover{text-decoration:underline}.code{background:#1a1a1a;color:#28c840;padding:1px 6px;border-radius:3px;font-size:.75rem;font-family:monospace}.nav{margin-bottom:1.5rem}.nav a{margin-right:1.5rem}.foot{margin-top:3rem;color:#444;font-size:.7rem}";

    let mut html = format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1.0\"><title>CBOR-Web Server</title><style>{style}</style></head><body><h1>CBOR-<span>Web</span></h1><p class=\"sub\">Binary Web Content for AI Agents — RFC 8949 — 38 sites live</p><div class=\"nav\"><a href=\"/\">Dashboard</a><a href=\"/codes\">Short Codes</a><a href=\"/health\">Health</a><a href=\"/.well-known/cbor-web/manifest.json\">Manifest (JSON)</a><a href=\"https://github.com/ploteddie-bit/cbor-web\">GitHub</a></div><div class=\"statb\"><div class=\"stat\"><div class=\"n\">{}</div><div class=\"l\">Sites</div></div><div class=\"stat\"><div class=\"n\">{}</div><div class=\"l\">Pages CBOR</div></div><div class=\"stat\"><div class=\"n\">{}</div><div class=\"l\">Total</div></div><div class=\"stat\"><div class=\"n\">{}</div><div class=\"l\">Requests</div></div></div>",
        sites.len(), total_pages, format_size(total_size), hits
    );

    html.push_str("<table><tr><th>Domain</th><th>Format</th><th>Pages</th><th>Short</th><th>Endpoints</th></tr>");
    for (domain, format, _size, pages, code) in &sites {
        let code_html = if code != "-" {
            format!("<span class=\"code\">/{}/</span>", code)
        } else {
            "-".to_string()
        };
        html.push_str(&format!(
            "<tr><td><a href=\"https://{0}\">{0}</a></td><td>{1}</td><td>{2}</td><td>{3}</td><td><a href=\"/.well-known/cbor-web\">manifest</a> <a href=\"/.well-known/cbor-web/bundle\">bundle</a></td></tr>",
            domain, format, pages, code_html
        ));
    }
    html.push_str("</table>");
    html.push_str(&format!(
        "<div class=\"foot\">cbor-server v{} — ExploDev 2026</div></body></html>",
        env!("CARGO_PKG_VERSION")
    ));

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
        .into_response()
}

// ── Short codes page (B) ──

async fn serve_codes(State(state): State<Arc<AppState>>) -> Response {
    let base = state.data_dir.parent().unwrap_or(&state.data_dir);
    let sites_dir = base.join("sites");
    let mut deployed: Vec<String> = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&sites_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            deployed.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    let style = "*{margin:0;padding:0;box-sizing:border-box}body{font-family:system-ui,monospace;background:#0a0a0a;color:#e0e0e0;padding:2rem}h1{color:#f97316;margin-bottom:.5rem}.sub{color:#888;margin-bottom:1.5rem}.nav{margin-bottom:1.5rem}.nav a{margin-right:1.5rem;color:#f97316;text-decoration:none}table{width:100%;border-collapse:collapse}th{text-align:left;padding:.5rem;color:#888;border-bottom:1px solid #333;font-size:.75rem}td{padding:.5rem;border-bottom:1px solid #1a1a1a;font-size:.85rem}code{color:#28c840;font-size:.85rem}a{color:#f97316;text-decoration:none}a:hover{text-decoration:underline}.on{color:#28c840}.off{color:#666}.foot{margin-top:3rem;color:#444;font-size:.7rem}";

    let mut html = format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1.0\"><title>Short Codes · CBOR-Web</title><style>{style}</style></head><body><h1>Short <span style=\"color:#fff\">Codes</span></h1><p class=\"sub\">3-letter aliases → Edge Worker CDN URLs</p><div class=\"nav\"><a href=\"/\">Dashboard</a><a href=\"/codes\">Short Codes</a><a href=\"/health\">Health</a></div><p style=\"color:#888;margin-bottom:1rem\">Worker URL pattern: <code>https://cbor-web.explodev.workers.dev/<b>code</b>/path</code></p><table><tr><th>Code</th><th>Domain</th><th>Status</th><th>Quick Links</th></tr>"
    );

    for (code, domain) in SHORT_CODES {
        let deployed = deployed.iter().any(|d| d == domain);
        let status = if deployed {
            "<span class=\"on\">deployed</span>"
        } else {
            "<span class=\"off\">pending</span>"
        };
        let links = if deployed {
            format!("<a href=\"https://cbor-web.explodev.workers.dev/{0}/\">/{0}/</a> <a href=\"https://cbor-web.explodev.workers.dev/{0}/.well-known/cbor-web\">manifest</a>", code)
        } else {
            "-".to_string()
        };
        html.push_str(&format!(
            "<tr><td><code>{0}</code></td><td><a href=\"https://{1}\">{1}</a></td><td>{2}</td><td>{3}</td></tr>",
            code, domain, status, links
        ));
    }
    html.push_str("</table>");
    html.push_str(&format!(
        "<div class=\"foot\">cbor-server v{} — ExploDev 2026</div></body></html>",
        env!("CARGO_PKG_VERSION")
    ));

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
        .into_response()
}

// ── Cross-domain search (D) ──

#[derive(serde::Serialize)]
struct SearchResult {
    domain: String,
    path: String,
    snippet: String,
}

async fn serve_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let q = match params.get("q").filter(|s| !s.is_empty()) {
        Some(v) => v.to_lowercase(),
        None => return (StatusCode::BAD_REQUEST, "Missing ?q= parameter").into_response(),
    };
    if q.len() < 2 {
        return (StatusCode::BAD_REQUEST, "Query too short (min 2 chars)").into_response();
    }

    let base = state.data_dir.parent().unwrap_or(&state.data_dir);
    let sites_dir = base.join("sites");
    let mut results: Vec<SearchResult> = Vec::new();

    if let Ok(mut entries) = tokio::fs::read_dir(&sites_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let domain = entry.file_name().to_string_lossy().to_string();
            let pages_dir = entry.path().join(".well-known/cbor-web/pages");
            if !pages_dir.exists() {
                continue;
            }
            if let Ok(mut pents) = tokio::fs::read_dir(&pages_dir).await {
                while let Ok(Some(pent)) = pents.next_entry().await {
                    let path = pent.file_name().to_string_lossy().to_string();
                    if let Ok(data) = tokio::fs::read(pent.path()).await {
                        let text = String::from_utf8_lossy(&data).to_lowercase();
                        if let Some(pos) = text.find(&q) {
                            let start = pos.saturating_sub(30);
                            let end = (pos + q.len() + 50).min(text.len());
                            let raw_snippet = &text[start..end];
                            let snippet = format!("...{}...", raw_snippet.replace('\n', " "));
                            results.push(SearchResult {
                                domain: domain.clone(),
                                path: path.clone(),
                                snippet,
                            });
                            if results.len() >= 20 {
                                break;
                            }
                        }
                    }
                }
            }
            if results.len() >= 20 {
                break;
            }
        }
    }

    let json = serde_json::to_string(&serde_json::json!({
        "query": q,
        "results": results.len(),
        "hits": results,
    }))
    .unwrap_or_default();

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        json,
    )
        .into_response()
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    std::fs::create_dir_all(cli.data.join(".well-known/cbor-web/pages"))
        .expect("Failed to create data directories");

    let doléance_path = cli.data.join(".doleances.json");
    let doléances = if doléance_path.exists() {
        std::fs::read_to_string(&doléance_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<serde_json::Value>>(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    tracing::info!("Loaded {} persisted doléances", doléances.len());

    let state = Arc::new(AppState {
        data_dir: cli.data.clone(),
        token: cli.token.clone(),
        limiter: RateLimiter::new(cli.rate_limit),
        doléances: Mutex::new(doléances),
        doléance_path: doléance_path.clone(),
        page_snapshots: Mutex::new(HashMap::new()),
        started_at: std::time::Instant::now(),
        hit_count: AtomicU64::new(0),
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::HEAD])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/health", get(health_check).head(health_check))
        .route("/codes", get(serve_codes))
        .route("/search", get(serve_search))
        .route("/doleance", axum::routing::post(receive_doleance))
        .route("/diff", get(serve_diff).head(serve_diff))
        .nest(
            "/.well-known/cbor-web",
            Router::new()
                .route("/", get(serve_manifest).head(serve_manifest))
                .route("/manifest.json", get(serve_manifest_json))
                .route("/pages/:filename", get(serve_page).head(serve_page))
                .route("/bundle", get(serve_bundle).head(serve_bundle))
                .route("/doleance", axum::routing::post(receive_doleance))
                .route("/doleance/list", get(list_doleances))
                .route("/diff", get(serve_diff).head(serve_diff))
                .layer(middleware::from_fn_with_state(state.clone(), auth_mw)),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            hit_counter_mw,
        ))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit_mw))
        .layer(CompressionLayer::new())
        .layer(cors)
        .with_state(state.clone());

    tracing::info!(
        "cbor-server v{} — CBOR-Web HTTP server",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!("Data dir: {}", cli.data.display());
    tracing::info!("Listening on {}", cli.listen);
    if cli.token.is_some() {
        tracing::info!("Token auth ENABLED (X-CBOR-Web-Wallet required)");
    } else {
        tracing::info!("Open mode — all content public");
    }
    tracing::info!("Rate limit: {} req/s per IP", cli.rate_limit);

    let listener = tokio::net::TcpListener::bind(&cli.listen)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    fn test_state() -> Arc<AppState> {
        let _ = std::fs::create_dir_all("data/.well-known/cbor-web/pages");
        Arc::new(AppState {
            data_dir: PathBuf::from("data"),
            token: None,
            limiter: RateLimiter::new(1000),
            doléances: Mutex::new(Vec::new()),
            doléance_path: PathBuf::from("data/.doleances.json"),
            page_snapshots: Mutex::new(HashMap::new()),
            started_at: std::time::Instant::now(),
            hit_count: AtomicU64::new(0),
        })
    }

    fn test_router(state: Arc<AppState>) -> Router {
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::HEAD])
            .allow_headers(Any)
            .allow_origin(Any);

        Router::new()
            .route("/", get(serve_dashboard))
            .route("/codes", get(serve_codes))
            .route("/search", get(serve_search))
            .nest(
                "/.well-known/cbor-web",
                Router::new()
                    .route("/", get(serve_manifest).head(serve_manifest))
                .route("/manifest.json", get(serve_manifest_json))
                    .route("/pages/:filename", get(serve_page).head(serve_page))
                    .route("/bundle", get(serve_bundle).head(serve_bundle))
                    .route("/doleance", axum::routing::post(receive_doleance))
                    .route("/doleance/list", get(list_doleances))
                    .route("/diff", get(serve_diff).head(serve_diff)),
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                hit_counter_mw,
            ))
            .layer(cors)
            .with_state(state)
    }

    #[tokio::test]
    async fn manifest_returns_200() {
        let app = test_router(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/cbor-web")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn manifest_returns_cbor_content_type() {
        let app = test_router(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/cbor-web")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "application/cbor"
        );
    }

    #[tokio::test]
    async fn bundle_returns_200() {
        let app = test_router(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/cbor-web/bundle")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn nonexistent_page_returns_404() {
        let app = test_router(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/cbor-web/pages/zzz_nope.cbor")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn diff_without_base_returns_400() {
        let app = test_router(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/cbor-web/diff")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn doleance_wrong_method_returns_405() {
        let app = test_router(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/.well-known/cbor-web/doleance")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn doleance_list_returns_200() {
        let app = test_router(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/cbor-web/doleance/list")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn doleance_post_without_cbor_type_returns_415() {
        let app = test_router(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/.well-known/cbor-web/doleance")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn doleance_post_accepts_and_list_grows() {
        let state = test_state();
        let app = test_router(state.clone());

        // Create a minimal valid CBOR payload: empty map = 0xA0
        let cbor_body = vec![0xA0u8];
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/.well-known/cbor-web/doleance")
                    .header("content-type", "application/cbor")
                    .body(Body::from(cbor_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        // Verify it's in the list
        let dols = state.doléances.lock().await;
        assert_eq!(dols.len(), 1);
    }

    #[tokio::test]
    async fn diff_with_valid_base_returns_cbor() {
        let app = test_router(test_state());
        // Use a 64-char hex hash (arbitrary but valid format)
        let resp = app.oneshot(
            Request::builder()
                .uri("/.well-known/cbor-web/diff?base=0000000000000000000000000000000000000000000000000000000000000000")
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();
        // Should return CBOR diff or 404 if base not found
        let status = resp.status();
        assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);
    }
}
