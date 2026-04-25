//! cbor-server — CBOR-Web HTTP server (reference implementation)
//!
//! Serves CBOR-Web endpoints following the v2.1 specification:
//! - GET  /.well-known/cbor-web          → manifest
//! - GET  /.well-known/cbor-web/pages/*  → individual page CBOR
//! - GET  /.well-known/cbor-web/bundle   → full site bundle
//! - POST /.well-known/cbor-web/doleance → agent feedback receiver
//! - GET  /.well-known/cbor-web/diff     → incremental diff (base=HASH)
//!
//! Designed to run on serveur-dev (10.0.0.201) behind a Cloudflare Worker proxy.

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::Parser;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
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

// ── Middleware state ──

struct RateLimiter {
    max_per_sec: u32,
    buckets: Mutex<HashMap<String, (Instant, u32)>>,
}

impl RateLimiter {
    fn new(max_per_sec: u32) -> Self {
        Self { max_per_sec, buckets: Mutex::new(HashMap::new()) }
    }

    async fn check(&self, ip: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        let entry = buckets.entry(ip.to_string()).or_insert((now, 0));
        if now.duration_since(entry.0) >= Duration::from_secs(1) {
            *entry = (now, 1);
            true
        } else if entry.1 < self.max_per_sec {
            entry.1 += 1;
            true
        } else {
            false
        }
    }
}

struct AppState {
    data_dir: PathBuf,
    token: Option<String>,
    limiter: RateLimiter,
    doléances: Mutex<Vec<serde_json::Value>>,
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

// ── Endpoints ──

async fn serve_manifest(
    State(state): State<Arc<AppState>>,
    method: Method,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }
    let paths = [
        state.data_dir.join(".well-known/cbor-web/manifest.cbor"),
        state.data_dir.join("index.cbor"),
    ];
    for p in &paths {
        if let Some(resp) = serve_file(p).await {
            return resp;
        }
    }
    (StatusCode::NOT_FOUND, "Not Found").into_response()
}

async fn serve_page(
    State(state): State<Arc<AppState>>,
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
    let path = state.data_dir.join(".well-known/cbor-web/pages").join(&safe);
    serve_file(&path).await.unwrap_or_else(|| (StatusCode::NOT_FOUND, "Not Found").into_response())
}

async fn serve_bundle(
    State(state): State<Arc<AppState>>,
    method: Method,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }
    let path = state.data_dir.join(".well-known/cbor-web/bundle.cbor");
    serve_file(&path).await.unwrap_or_else(|| (StatusCode::NOT_FOUND, "Not Found").into_response())
}

async fn serve_file(path: &std::path::Path) -> Option<Response> {
    let data = tokio::fs::read(path).await.ok()?;
    let etag = format!("\"{}\"", sha256_hex(&data));
    Some(cbor_response(&data, &etag))
}

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
        ).into_response();
    }
    let mut entry = serde_json::Map::new();
    entry.insert("received_at".into(), serde_json::json!(
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0)
    ));
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
    }
    (StatusCode::ACCEPTED, "Doléance received").into_response()
}

#[derive(serde::Deserialize)]
struct DiffQuery {
    base: Option<String>,
}

async fn serve_diff(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DiffQuery>,
    method: Method,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }
    let base_hash = match query.base {
        Some(ref h) if h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()) => h.clone(),
        _ => return (StatusCode::BAD_REQUEST, "Missing or invalid ?base=<sha256-hex>").into_response(),
    };
    let paths = [
        state.data_dir.join(".well-known/cbor-web/manifest.cbor"),
        state.data_dir.join("index.cbor"),
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
            None => return (StatusCode::NOT_FOUND, "Not Found").into_response(),
        }
    };
    let current_hash = sha256_hex(&current_bytes);
    if base_hash == current_hash {
        let diff = cbor_canonical_map(vec![
            (ciborium::Value::Integer(0.into()), ciborium::Value::Text("cbor-web-diff".into())),
            (ciborium::Value::Integer(1.into()), ciborium::Value::Integer(1.into())),
            (ciborium::Value::Text("base_version_hash".into()), ciborium::Value::Bytes(hex::decode(&base_hash).unwrap_or_default())),
            (ciborium::Value::Text("current_hash".into()), ciborium::Value::Text(current_hash.clone())),
            (ciborium::Value::Text("stats".into()), cbor_canonical_map(vec![
                (ciborium::Value::Text("pages_added".into()), ciborium::Value::Integer(0.into())),
                (ciborium::Value::Text("pages_modified".into()), ciborium::Value::Integer(0.into())),
                (ciborium::Value::Text("pages_removed".into()), ciborium::Value::Integer(0.into())),
                (ciborium::Value::Text("total_pages_now".into()), ciborium::Value::Integer(0.into())),
            ])),
            (ciborium::Value::Text("changes".into()), ciborium::Value::Array(vec![])),
        ]);
        let mut buf = Vec::new();
        if ciborium::into_writer(&diff, &mut buf).is_ok() {
            return cbor_response(&buf, &format!("\"{}\"", sha256_hex(&buf)));
        }
    }
    let body = serde_json::json!({
        "error": "Diff not available",
        "base": base_hash,
        "current": current_hash,
    });
    (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "application/json")], serde_json::to_vec(&body).unwrap_or_default()).into_response()
}

// ── Helpers ──

fn cbor_response(data: &[u8], etag: &str) -> Response {
    let headers = [
        (header::CONTENT_TYPE, "application/cbor"),
        (header::ETAG, etag),
        (header::CACHE_CONTROL, "public, max-age=3600"),
    ];
    (StatusCode::OK, headers, data.to_vec()).into_response()
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let cli = Cli::parse();

    std::fs::create_dir_all(cli.data.join(".well-known/cbor-web/pages"))
        .expect("Failed to create data directories");

    let state = Arc::new(AppState {
        data_dir: cli.data.clone(),
        token: cli.token.clone(),
        limiter: RateLimiter::new(cli.rate_limit),
        doléances: Mutex::new(Vec::new()),
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::HEAD])
        .allow_headers(Any)
        .allow_origin(Any);

    // Routes: all endpoints under /.well-known/cbor-web
    let app = Router::new()
        .route("/doleance", post(receive_doleance))
        .route("/diff", get(serve_diff).head(serve_diff))
        .nest("/.well-known/cbor-web", Router::new()
            .route("/", get(serve_manifest).head(serve_manifest))
            .route("/pages/:filename", get(serve_page).head(serve_page))
            .route("/bundle", get(serve_bundle).head(serve_bundle))
            .route("/doleance", post(receive_doleance))
            .route("/diff", get(serve_diff).head(serve_diff))
            .layer(middleware::from_fn_with_state(state.clone(), auth_mw))
        )
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit_mw))
        .layer(cors)
        .with_state(state.clone());

    tracing::info!(
        "cbor-server v{} — CBOR-Web HTTP server",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!("Data dir: {}", cli.data.display());
    tracing::info!("Listening on {}", cli.listen);
    if cli.token.is_some() {
        tracing::info!("Token auth ENABLED (X-CBOR-Web-Wallet required for T1/T0)");
    } else {
        tracing::info!("Open mode — all content public");
    }
    tracing::info!("Rate limit: {} req/s per IP", cli.rate_limit);

    let listener = tokio::net::TcpListener::bind(&cli.listen)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app).await.expect("Server error");
}
