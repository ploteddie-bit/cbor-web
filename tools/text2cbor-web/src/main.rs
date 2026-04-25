use axum::{
    extract::{Multipart, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;

// ── Types ──

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct User {
    email: String,
    password_hash: String,
    api_token: String,
    created_at: u64,
    upload_count: u64,
    total_pages: u64,
    plan: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Session {
    token: String,
    email: String,
    expires_at: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Invoice {
    id: String,
    user_email: String,
    domain: String,
    pages: usize,
    cbor_size: usize,
    amount_eur: u32,
    timestamp: u64,
    paid: bool,
}

struct AppState {
    users: RwLock<Vec<User>>,
    sessions: RwLock<Vec<Session>>,
    invoices: RwLock<Vec<Invoice>>,
    data_dir: PathBuf,
    started_at: Instant,
    upload_count: AtomicU64,
}

// ── Main ──

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let data_dir = PathBuf::from("data");
    std::fs::create_dir_all(data_dir.join("uploads")).ok();
    std::fs::create_dir_all(data_dir.join("invoices")).ok();

    let users = load_json(&data_dir.join("users.json")).unwrap_or_default();
    let sessions: Vec<Session> = vec![];
    let invoices = load_json(&data_dir.join("invoices.json")).unwrap_or_default();

    let state = Arc::new(AppState {
        users: RwLock::new(users),
        sessions: RwLock::new(sessions),
        invoices: RwLock::new(invoices),
        data_dir,
        started_at: Instant::now(),
        upload_count: AtomicU64::new(0),
    });

    let app = Router::new()
        .layer(CorsLayer::new().allow_origin(Any).allow_methods([Method::GET, Method::POST]).allow_headers(Any)),
        .route("/", get(serve_dashboard))
        .route("/login", get(serve_login_page).post(handle_login))
        .route("/register", get(serve_register_page).post(handle_register))
        .route("/logout", get(handle_logout))
        .route("/upload", post(upload_handler))
        .route("/api/upload", post(api_upload_handler))
        .route("/api/key", get(api_key_handler))
        .route("/invoices", get(serve_invoices))
        .route("/invoice/{id}", get(serve_invoice))
        .route("/api/status", get(api_status))
        .route("/admin", get(serve_admin))
        .route("/admin/invoice/{id}/paid", post(mark_invoice_paid))
        .route("/crm", get(serve_crm))
        .route("/crm/prospects", get(api_prospects))
        .route("/crm/prospects/add", post(api_add_prospect))
        .route("/crm/prospects/{id}/status", post(api_update_status))
        .route("/crm/prospects/{id}/note", post(api_add_note))
        .route("/crm/scrape", post(api_trigger_scrape))
        .route("/crm/export", get(api_export_csv))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3002").await.expect("bind");
    tracing::info!("text2cbor-web SaaS listening on 127.0.0.1:3002");
    axum::serve(listener, app).await.ok();
}

// ── Session helpers ──

fn hash_password(pw: &str) -> String {
    hex::encode(Sha256::digest(format!("cbor-web-salt:{pw}").as_bytes()))
}

fn make_token() -> String {
    hex::encode(Sha256::digest(format!("session:{}:{}", uuid::Uuid::new_v4(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()).as_bytes()))
}

fn now_ts() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() }

async fn get_session_user(state: &AppState, token: &str) -> Option<User> {
    let sessions = state.sessions.read().await;
    let email = sessions.iter().find(|s| s.token == token && s.expires_at > now_ts())?.email.clone();
    let users = state.users.read().await;
    users.iter().find(|u| u.email == email).cloned()
}

fn auth_cookie(token: &str) -> String {
    format!("session={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=86400")
}

// ── Pages ──

const STYLE: &str = "*{margin:0;padding:0;box-sizing:border-box}body{font-family:system-ui,monospace;background:#0a0a0a;color:#e0e0e0;padding:2rem}h1{color:#f97316;margin-bottom:.5rem}h1 span{color:#fff}h2{color:#f97316;margin:1.5rem 0 .8rem}.sub{color:#888;margin-bottom:.5rem}.nav{margin-bottom:2rem}.nav a{margin-right:1.5rem;color:#f97316;text-decoration:none}.nav a:hover{text-decoration:underline}.card{background:#111;border-radius:8px;padding:2rem;max-width:640px;margin-bottom:2rem}.card h2{color:#f97316;margin-bottom:1rem;font-size:1.1rem}.field{margin-bottom:1.2rem}.field label{display:block;color:#888;font-size:.8rem;margin-bottom:.4rem;text-transform:uppercase;letter-spacing:.05em}.field input{width:100%;padding:.6rem .8rem;background:#1a1a1a;border:1px solid #333;border-radius:6px;color:#e0e0e0;font-family:inherit;font-size:.9rem}.field input[type=file]::file-selector-button{padding:.4rem 1rem;margin-right:.8rem;background:#f97316;color:#000;border:none;border-radius:4px;cursor:pointer;font-weight:600}.btn{background:#f97316;color:#000;border:none;padding:.7rem 1.8rem;border-radius:6px;font-weight:700;font-size:.9rem;cursor:pointer;font-family:inherit}.btn:hover{opacity:.9}.btn:disabled{opacity:.4}.btn-sm{padding:.3rem .8rem;font-size:.75rem}.flash{background:#1a1a1a;border-left:3px solid #f97316;padding:1rem;margin-bottom:1rem}.flash-err{border-left-color:#ef4444}.statb{display:flex;gap:2rem;flex-wrap:wrap;margin:1rem 0}.stat{background:#111;padding:1rem;border-radius:8px;min-width:120px}.stat .n{font-size:2rem;color:#f97316;font-weight:700}.stat .l{color:#666;font-size:.8rem}table{width:100%;border-collapse:collapse;margin:1rem 0}th{text-align:left;padding:.5rem .8rem;color:#888;border-bottom:1px solid #333;font-size:.75rem;text-transform:uppercase}td{padding:.5rem .8rem;border-bottom:1px solid #1a1a1a;font-size:.85rem}.badge{padding:2px 8px;border-radius:4px;font-size:.7rem;font-weight:700}.badge-ok{background:#28c840;color:#000}.badge-wait{background:#f97316;color:#000}.foot{margin-top:3rem;color:#444;font-size:.7rem}#result{display:none;margin-top:1rem;background:#111;padding:1rem;border-radius:6px;border-left:3px solid #28c840}#result a{color:#f97316}code{color:#28c840;background:#1a1a1a;padding:1px 4px;border-radius:3px}";

fn nav_html(authenticated: bool, email: &str) -> String {
    if authenticated {
        format!(
            r#"<div class="nav"><a href="/">Dashboard</a><a href="/invoices">Invoices</a><a href="/crm">CRM</a><a href="/api/status">API</a><a href="https://github.com/ploteddie-bit/cbor-web">GitHub</a><span style="float:right;color:#888;font-size:.8rem">{email}</span><a href="/logout" style="float:right;margin-right:1rem">Logout</a></div>"#,
        )
    } else {
        r#"<div class="nav"><a href="/login">Login</a><a href="/register">Register</a><a href="https://github.com/ploteddie-bit/cbor-web">GitHub</a></div>"#.to_string()
    }
}

async fn serve_dashboard(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = get_cookie(&headers);
    let user = get_session_user(&state, &token).await;
    let authenticated = user.is_some();
    let email = user.as_ref().map(|u| u.email.as_str()).unwrap_or("");

    let nav = nav_html(authenticated, email);

    let content = if authenticated {
        let u = user.unwrap();
        format!(
            r#"<div class="statb">
  <div class="stat"><div class="n">{}</div><div class="l">Uploads</div></div>
  <div class="stat"><div class="n">{}</div><div class="l">Pages Processed</div></div>
  <div class="stat"><div class="n">{}</div><div class="l">Plan</div></div>
</div>
<div class="card">
  <h2>Convert HTML → CBOR</h2>
  <form id="uploadForm" enctype="multipart/form-data">
    <div class="field">
      <label>Domain name</label>
      <input type="text" id="domain" name="domain" placeholder="example.com" required>
    </div>
    <div class="field">
      <label>HTML site (.zip)</label>
      <input type="file" id="file" name="file" accept=".zip" required>
    </div>
    <button type="submit" class="btn">Convert to CBOR</button>
  </form>
  <div id="progress" style="display:none;margin-top:1rem;color:#f97316">Processing...</div>
  <div id="result"></div>
</div>"#,
            u.upload_count, u.total_pages, u.plan,
        )
    } else {
        r#"<div class="flash">Create an account to convert your website to CBOR-Web. Free trial — 5 conversions included.</div>
<div class="card"><h2>How it works</h2>
<p>1. Register with your email</p><p>2. Upload your HTML site as a .zip</p><p>3. Download the CBOR-Web output</p><p>4. Deploy alongside your existing site</p>
<p style="margin-top:1rem;color:#888">After 5 conversions, invoice-based billing applies — you receive an invoice by email, payable by bank transfer.</p>
</div>"#.to_string()
    };

    let html = format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>text2cbor — SaaS</title><style>{STYLE}</style></head><body><h1>text2<span>cbor</span></h1><p class="sub">Convert HTML to CBOR-Web — binary web for AI agents</p>{nav}{content}<div class="foot">text2cbor SaaS v{version} — ExploDev 2026</div><script>{script}</script></body></html>"#,
        version = env!("CARGO_PKG_VERSION"),
        script = UPLOAD_SCRIPT,
    );

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}

const UPLOAD_SCRIPT: &str = r#"
const form=document.getElementById('uploadForm');if(form){form.onsubmit=async(e)=>{e.preventDefault();const d=new FormData(form);document.getElementById('progress').style.display='block';document.getElementById('result').style.display='none';try{const r=await fetch('/upload',{method:'POST',body:d});if(r.ok){const b=await r.blob();const u=URL.createObjectURL(b);const a=document.createElement('a');a.href=u;a.download='cbor-output.zip';a.click();URL.revokeObjectURL(u);document.getElementById('result').style.display='block';document.getElementById('result').innerHTML='<div style=\"color:#28c840\">Done. Download started. <a href=\"/invoices\">View invoice →</a></div>'}else{const t=await r.text();document.getElementById('result').style.display='block';document.getElementById('result').innerHTML='<div style=\"color:#ef4444\">Error: '+t+'</div>'}}catch(err){document.getElementById('result').style.display='block';document.getElementById('result').innerHTML='<div style=\"color:#ef4444\">Network error</div>'}finally{document.getElementById('progress').style.display='none'}}}"#;

// ── Auth handlers ──

fn get_cookie(headers: &axum::http::HeaderMap) -> String {
    headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .find(|c| c.trim().starts_with("session="))
        .map(|c| c.trim().strip_prefix("session=").unwrap_or(""))
        .unwrap_or("")
        .to_string()
}

async fn serve_login_page(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let token = get_cookie(&headers);
    if get_session_user(&state, &token).await.is_some() {
        return Redirect::to("/").into_response();
    }
    let error = params.get("error").cloned().unwrap_or_default();
    let err_html = if !error.is_empty() { format!(r#"<div class="flash flash-err">{error}</div>"#) } else { String::new() };
    let html = format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>Login — text2cbor</title><style>{STYLE}</style></head><body><h1>text2<span>cbor</span></h1><div class="card" style="max-width:400px"><h2>Login</h2>{err_html}<form method="post" action="/login"><div class="field"><label>Email</label><input name="email" type="email" required></div><div class="field"><label>Password</label><input name="password" type="password" required></div><button class="btn">Login</button></form><p style="margin-top:1rem;color:#888">No account? <a href="/register">Register →</a></p></div></body></html>"#,
    );
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}

async fn serve_register_page() -> Response {
    let html = format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>Register — text2cbor</title><style>{STYLE}</style></head><body><h1>text2<span>cbor</span></h1><div class="card" style="max-width:400px"><h2>Register</h2><form method="post" action="/register"><div class="field"><label>Email</label><input name="email" type="email" required></div><div class="field"><label>Password</label><input name="password" type="password" minlength="6" required></div><button class="btn">Create Account</button></form><p style="margin-top:1rem;color:#888">Free trial — 5 conversions included. After that, invoice-based billing.</p><p style="color:#888">Already have an account? <a href="/login">Login →</a></p></div></body></html>"#,
    );
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}

async fn handle_login(
    State(state): State<Arc<AppState>>,
    mut form: axum::extract::Form<HashMap<String, String>>,
) -> Response {
    let email = form.remove("email").unwrap_or_default().to_lowercase().trim().to_string();
    let pw = form.remove("password").unwrap_or_default();
    if email.is_empty() || pw.is_empty() {
        return Redirect::to("/login?error=Email+and+password+required").into_response();
    }
    let users = state.users.read().await;
    let user = users.iter().find(|u| u.email == email && u.password_hash == hash_password(&pw));
    if user.is_none() {
        return Redirect::to("/login?error=Invalid+email+or+password").into_response();
    }
    let token = make_token();
    state.sessions.write().await.push(Session { token: token.clone(), email, expires_at: now_ts() + 86400 });
    let mut resp = Redirect::to("/").into_response();
    resp.headers_mut().insert(header::SET_COOKIE, auth_cookie(&token).parse().unwrap());
    resp
}

async fn handle_register(
    State(state): State<Arc<AppState>>,
    mut form: axum::extract::Form<HashMap<String, String>>,
) -> Response {
    let email = form.remove("email").unwrap_or_default().to_lowercase().trim().to_string();
    let pw = form.remove("password").unwrap_or_default();
    if email.is_empty() || pw.len() < 6 || !email.contains('@') {
        return Redirect::to("/register?error=Valid+email+and+password+(6%2B+chars)+required").into_response();
    }
    let mut users = state.users.write().await;
    if users.iter().any(|u| u.email == email) {
        return Redirect::to("/register?error=Email+already+registered").into_response();
    }
    users.push(User { email: email.clone(), password_hash: hash_password(&pw), api_token: hex::encode(Sha256::digest(format!("api:{}:{}", &email, now_ts()).as_bytes())), created_at: now_ts(), upload_count: 0, total_pages: 0, plan: "Trial (5 free)".into() });
    let users_clone = users.clone();
    drop(users);
    save_json(&state.data_dir.join("users.json"), &users_clone);
    let token = make_token();
    state.sessions.write().await.push(Session { token: token.clone(), email, expires_at: now_ts() + 86400 });
    let mut resp = Redirect::to("/").into_response();
    resp.headers_mut().insert(header::SET_COOKIE, auth_cookie(&token).parse().unwrap());
    resp
}

async fn handle_logout(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = get_cookie(&headers);
    if !token.is_empty() {
        let mut s = state.sessions.write().await;
        s.retain(|s| s.token != token);
    }
    let mut resp = Redirect::to("/").into_response();
    resp.headers_mut().insert(header::SET_COOKIE, "session=; HttpOnly; Path=/; Max-Age=0".parse().unwrap());
    resp
}

// ── Upload (authenticated) ──

async fn upload_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let token = get_cookie(&headers);
    let user = match get_session_user(&state, &token).await {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Login required").into_response(),
    };

    let mut domain: Option<String> = None;
    let mut zip_data: Option<Vec<u8>> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "domain" => { if let Ok(t) = field.text().await { let d = t.trim().to_string(); if !d.is_empty() && is_valid_domain(&d) { domain = Some(d); } } }
            "file" => { if let Ok(b) = field.bytes().await { if !b.is_empty() { zip_data = Some(b.to_vec()); } } }
            _ => {}
        }
    }
    let domain = match domain { Some(d) => d, None => return (StatusCode::BAD_REQUEST, "Missing domain").into_response() };
    let zip_bytes = match zip_data { Some(b) => b, None => return (StatusCode::BAD_REQUEST, "Missing zip file").into_response() };

    let work_id = uuid::Uuid::new_v4().to_string();
    let tmp_dir = state.data_dir.join("uploads").join(&work_id);
    let site_dir = tmp_dir.join("site");
    let output_dir = tmp_dir.join("output");
    std::fs::create_dir_all(&site_dir).ok();
    std::fs::create_dir_all(&output_dir).ok();

    if let Err(e) = extract_zip(&zip_bytes, &site_dir) {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return (StatusCode::BAD_REQUEST, format!("Bad zip: {e}")).into_response();
    }

    let binary = text2cbor_binary();
    let out = Command::new(&binary).args(["generate","--input",site_dir.to_str().unwrap(),"--output",output_dir.to_str().unwrap(),"--domain",&domain]).output();
    match out {
        Ok(o) if o.status.success() => {}
        Ok(o) => { let _ = std::fs::remove_dir_all(&tmp_dir); return (StatusCode::INTERNAL_SERVER_ERROR, format!("text2cbor failed: {}", String::from_utf8_lossy(&o.stderr).trim())).into_response(); }
        Err(e) => { let _ = std::fs::remove_dir_all(&tmp_dir); return (StatusCode::INTERNAL_SERVER_ERROR, format!("text2cbor error: {e}")).into_response(); }
    }

    let output_zip = match create_output_zip(&output_dir) {
        Ok(z) => z,
        Err(e) => { let _ = std::fs::remove_dir_all(&tmp_dir); return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(); }
    };

    // Count pages from CBOR output
    let page_count = std::fs::read_dir(&output_dir).map(|d| d.filter(|e| e.as_ref().map(|f| f.path().extension().map(|x| x == "cbor").unwrap_or(false)).unwrap_or(false)).count()).unwrap_or(0);
    let cbor_size = output_zip.len();

    // Generate invoice
    let invoice = Invoice {
        id: format!("INV-{}", &work_id[..8].to_uppercase()),
        user_email: user.email.clone(),
        domain: domain.clone(),
        pages: page_count,
        cbor_size,
        amount_eur: if user.upload_count < 5 { 0 } else { 49 },
        timestamp: now_ts(),
        paid: user.upload_count < 5,
    };
    state.invoices.write().await.push(invoice.clone());
    let invoices_clone = state.invoices.read().await.clone();
    save_json(&state.data_dir.join("invoices.json"), &invoices_clone);
    save_invoice_html(&state.data_dir, &invoice);
    send_invoice_email(&invoice);

    // Update user stats
    {
        let mut users = state.users.write().await;
        if let Some(u) = users.iter_mut().find(|u| u.email == user.email) {
            u.upload_count += 1;
            u.total_pages += page_count as u64;
        }
        let users_clone2 = users.clone();
        save_json(&state.data_dir.join("users.json"), &users_clone2);
    }

    state.upload_count.fetch_add(1, Ordering::Relaxed);
    tracing::info!("Converted {} for {} ({} pages, {} bytes)", domain, user.email, page_count, cbor_size);

    let cleanup_dir = tmp_dir.clone();
    tokio::spawn(async move { tokio::time::sleep(Duration::from_secs(3600)).await; let _ = tokio::fs::remove_dir_all(&cleanup_dir).await; });

    (StatusCode::OK, [(header::CONTENT_TYPE, "application/zip"), (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}-cbor.zip\"", domain).as_str())], output_zip).into_response()
}

// ── API upload (programmatic, with Authorization header) ──

async fn api_upload_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let api_token = headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    let users = state.users.read().await;
    let user = match users.iter().find(|u| u.api_token == api_token) {
        Some(u) => u.clone(),
        None => return (StatusCode::UNAUTHORIZED, "Invalid API key. Get yours at /api/key").into_response(),
    };
    drop(users);

    let mut domain: Option<String> = None;
    let mut zip_data: Option<Vec<u8>> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "domain" => { if let Ok(t) = field.text().await { let d = t.trim().to_string(); if !d.is_empty() && is_valid_domain(&d) { domain = Some(d); } } }
            "file" => { if let Ok(b) = field.bytes().await { if !b.is_empty() { zip_data = Some(b.to_vec()); } } }
            _ => {}
        }
    }
    let domain = match domain { Some(d) => d, None => return (StatusCode::BAD_REQUEST, "Missing domain").into_response() };
    let zip_bytes = match zip_data { Some(b) => b, None => return (StatusCode::BAD_REQUEST, "Missing zip file").into_response() };

    // Reuse upload logic — same as upload_handler from here
    let work_id = uuid::Uuid::new_v4().to_string();
    let tmp_dir = state.data_dir.join("uploads").join(&work_id);
    let site_dir = tmp_dir.join("site");
    let output_dir = tmp_dir.join("output");
    std::fs::create_dir_all(&site_dir).ok();
    std::fs::create_dir_all(&output_dir).ok();
    if let Err(e) = extract_zip(&zip_bytes, &site_dir) { let _ = std::fs::remove_dir_all(&tmp_dir); return (StatusCode::BAD_REQUEST, format!("Bad zip: {e}")).into_response(); }
    let binary = text2cbor_binary();
    let out = Command::new(&binary).args(["generate","--input",site_dir.to_str().unwrap(),"--output",output_dir.to_str().unwrap(),"--domain",&domain]).output();
    match out { Ok(o) if o.status.success() => {}, Ok(o) => { let _ = std::fs::remove_dir_all(&tmp_dir); return (StatusCode::INTERNAL_SERVER_ERROR, format!("text2cbor: {}", String::from_utf8_lossy(&o.stderr).trim())).into_response(); }, Err(e) => { let _ = std::fs::remove_dir_all(&tmp_dir); return (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {e}")).into_response(); } }
    let output_zip = match create_output_zip(&output_dir) { Ok(z) => z, Err(e) => { let _ = std::fs::remove_dir_all(&tmp_dir); return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(); } };
    let page_count = std::fs::read_dir(&output_dir).map(|d| d.filter(|e| e.as_ref().map(|f| f.path().extension().map(|x| x == "cbor").unwrap_or(false)).unwrap_or(false)).count()).unwrap_or(0);
    let cbor_size = output_zip.len();
    let invoice = Invoice { id: format!("INV-{}", &work_id[..8].to_uppercase()), user_email: user.email.clone(), domain: domain.clone(), pages: page_count, cbor_size, amount_eur: if user.upload_count < 5 { 0 } else { 49 }, timestamp: now_ts(), paid: user.upload_count < 5 };
    state.invoices.write().await.push(invoice.clone());
    let inv_clone = state.invoices.read().await.clone();
    save_json(&state.data_dir.join("invoices.json"), &inv_clone);
    save_invoice_html(&state.data_dir, &invoice);
    let mut users_w = state.users.write().await;
    if let Some(u) = users_w.iter_mut().find(|u| u.email == user.email) { u.upload_count += 1; u.total_pages += page_count as u64; }
    let users_clone = users_w.clone(); drop(users_w);
    save_json(&state.data_dir.join("users.json"), &users_clone);
    state.upload_count.fetch_add(1, Ordering::Relaxed);
    send_invoice_email(&invoice);
    tracing::info!("API: Converted {} for {} ({} pages)", domain, user.email, page_count);
    let cleanup_dir = tmp_dir;
    tokio::spawn(async move { tokio::time::sleep(Duration::from_secs(3600)).await; let _ = tokio::fs::remove_dir_all(&cleanup_dir).await; });
    Json(serde_json::json!({"status":"ok","domain":domain,"pages":page_count,"size":cbor_size,"invoice":invoice.id,"download_url":format!("/api/download/{}", work_id)})).into_response()
}

async fn api_key_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = get_cookie(&headers);
    let user = match get_session_user(&state, &token).await {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Login required").into_response(),
    };
    Json(serde_json::json!({"api_key": user.api_token, "usage": format!("curl -H 'Authorization: Bearer {}' -F domain=example.com -F file=@site.zip http://127.0.0.1:3002/api/upload", user.api_token)})).into_response()
}

fn send_invoice_email(invoice: &Invoice) {
    let subject = format!("text2cbor Invoice {} — {}€", invoice.id, invoice.amount_eur);
    let body = format!(
        "text2cbor SaaS — Invoice {}\n\nDomain: {}\nPages: {}\nAmount: {}€\nStatus: {}\nDate: {}\n\nPayment by bank transfer.\nDeltopide SL — CIF B05356202 — Calle San Juan 4, 03110 Mutxamel, Alicante\n\nThank you for using text2cbor!",
        invoice.id, invoice.domain, invoice.pages, invoice.amount_eur,
        if invoice.paid { "PAID" } else { "PENDING" },
        chrono::DateTime::from_timestamp(invoice.timestamp as i64, 0).map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default()
    );
    // Try sendmail, fail silently if not configured
    let _ = std::process::Command::new("sendmail")
        .args(["-f", "noreply@text2cbor.com", &invoice.user_email])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map(|mut child| {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(format!("Subject: {subject}\nFrom: text2cbor <noreply@text2cbor.com>\nTo: {to}\nContent-Type: text/plain; charset=utf-8\n\n{body}", to = invoice.user_email).as_bytes());
            }
            let _ = child.wait();
        });
}

// ── API upload (programmatic) ──

// ── Invoices ──

async fn serve_invoices(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = get_cookie(&headers);
    let user = match get_session_user(&state, &token).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let invoices = state.invoices.read().await;
    let my_invoices: Vec<_> = invoices.iter().filter(|inv| inv.user_email == user.email).collect();

    let rows: String = my_invoices.iter().map(|inv| {
        let paid_badge = if inv.paid { r#"<span class="badge badge-ok">Paid</span>"# } else { r#"<span class="badge badge-wait">Pending</span>"# };
        format!(r#"<tr><td><a href="/invoice/{}">{}</a></td><td>{}</td><td>{} pages</td><td>{}€</td><td>{}</td><td>{}</td></tr>"#,
            inv.id, inv.id, inv.domain, inv.pages, inv.amount_eur, paid_badge,
            chrono::DateTime::from_timestamp(inv.timestamp as i64, 0).map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default())
    }).collect();

    let html = format!(r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>Invoices — text2cbor</title><style>{STYLE}</style></head><body><h1>text2<span>cbor</span></h1>{}<h2>Your Invoices</h2><table><tr><th>Invoice</th><th>Domain</th><th>Pages</th><th>Amount</th><th>Status</th><th>Date</th></tr>{}</table><p class="sub" style="margin-top:1rem">Invoices payable by bank transfer. You will receive the invoice by email after each conversion past the free trial.</p><div class="foot">text2cbor SaaS — ExploDev 2026</div></body></html>"#,
        nav_html(true, &user.email), rows);
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}

async fn serve_invoice(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let token = get_cookie(&headers);
    if get_session_user(&state, &token).await.is_none() {
        return Redirect::to("/login").into_response();
    }
    // Read invoice HTML from disk
    let path = state.data_dir.join("invoices").join(format!("{}.html", id));
    match tokio::fs::read_to_string(&path).await {
        Ok(html) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Invoice not found").into_response(),
    }
}

async fn serve_admin(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = get_cookie(&headers);
    let user = match get_session_user(&state, &token).await {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Login required").into_response(),
    };
    // Only admin can access (first registered user is admin)
    let users = state.users.read().await;
    let is_admin = users.first().map(|u| u.email == user.email).unwrap_or(false);
    if !is_admin { return (StatusCode::FORBIDDEN, "Admin only").into_response(); }

    let user_rows: String = users.iter().map(|u| format!(r#"<tr><td>{}</td><td>{} uploads</td><td>{} pages</td><td>{}</td><td>{}</td></tr>"#,
        u.email, u.upload_count, u.total_pages, u.plan, chrono::DateTime::from_timestamp(u.created_at as i64, 0).map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default())).collect();

    let invoices = state.invoices.read().await;
    let inv_rows: String = invoices.iter().map(|inv| {
        let paid_btn = if !inv.paid { format!(r#"<form method="post" action="/admin/invoice/{}/paid" style="display:inline"><button class="btn btn-sm">Mark Paid</button></form>"#, inv.id) } else { "✓".to_string() };
        format!(r#"<tr><td>{}</td><td>{}</td><td>{} pages</td><td>{}€</td><td>{}</td><td>{}</td></tr>"#,
            inv.id, inv.user_email, inv.pages, inv.amount_eur, paid_btn, inv.domain)
    }).collect();

    let html = format!(r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>Admin — text2cbor</title><style>{STYLE}</style></head><body><h1>text2<span>cbor</span></h1>{}<h2>Users ({})</h2><table><tr><th>Email</th><th>Uploads</th><th>Pages</th><th>Plan</th><th>Since</th></tr>{}</table><h2>Invoices ({})</h2><table><tr><th>ID</th><th>User</th><th>Pages</th><th>Amount</th><th>Paid</th><th>Domain</th></tr>{}</table><div class="foot">text2cbor Admin — ExploDev 2026</div></body></html>"#,
        nav_html(true, &user.email), users.len(), user_rows, invoices.len(), inv_rows);
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}

async fn mark_invoice_paid(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let mut invoices = state.invoices.write().await;
    if let Some(inv) = invoices.iter_mut().find(|i| i.id == id) {
        inv.paid = true;
        let invoices_clone = invoices.clone();
        save_json(&state.data_dir.join("invoices.json"), &invoices_clone);
        save_invoice_html(&state.data_dir, &invoices_clone.iter().find(|i| i.id == id).unwrap());
    }
    Redirect::to("/admin").into_response()
}

fn save_invoice_html(data_dir: &Path, inv: &Invoice) {
    let paid = if inv.paid { "PAID" } else { "PENDING — Payable by bank transfer" };
    let html = format!(r#"<!DOCTYPE html><html lang="fr"><head><meta charset="UTF-8"><title>Invoice {} — text2cbor</title><style>body{{font-family:Arial,sans-serif;padding:40px;max-width:700px;margin:auto}}h1{{color:#f97316}}table{{width:100%;border-collapse:collapse;margin:20px 0}}th,td{{padding:8px 12px;border:1px solid #ddd;text-align:left}}th{{background:#f5f5f5}}.total{{font-size:1.3em;font-weight:700}}.footer{{margin-top:40px;font-size:.8em;color:#888}}.stamp{{color:red;font-size:2em;transform:rotate(-15deg);position:absolute;margin-top:-40px;opacity:.5}}</style></head><body><h1>text2cbor SaaS — Invoice {id}</h1><div class="stamp">{paid}</div><table><tr><th>Date</th><td>{date}</td></tr><tr><th>Customer</th><td>{email}</td></tr><tr><th>Service</th><td>HTML → CBOR-Web conversion — {domain} ({pages} pages)</td></tr><tr><th>Amount</th><td class="total">{amount}€</td></tr><tr><th>Status</th><td>{paid}</td></tr></table><div class="footer"><p>Deltopide SL — CIF B05356202 — Calle San Juan 4, 03110 Mutxamel, Alicante, España</p><p>Payment by bank transfer — IBAN provided upon request</p><p>Invoice generated automatically by text2cbor SaaS — ExploDev 2026</p></div></body></html>"#,
        id = inv.id, paid = paid, date = chrono::DateTime::from_timestamp(inv.timestamp as i64, 0).map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
        email = inv.user_email, domain = inv.domain, pages = inv.pages, amount = inv.amount_eur,
    );
    let _ = std::fs::write(data_dir.join("invoices").join(format!("{}.html", inv.id)), html);
}

async fn api_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let users = state.users.read().await;
    Json(serde_json::json!({
        "status": "ok", "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": state.started_at.elapsed().as_secs(),
        "uploads_processed": state.upload_count.load(Ordering::Relaxed),
        "users": users.len(),
    }))
}

// ── Helpers ──

fn text2cbor_binary() -> PathBuf {
    ["../text2cbor/target/release/text2cbor", "/usr/local/bin/text2cbor"]
        .iter().find(|p| Path::new(p).exists()).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("text2cbor"))
}

fn is_valid_domain(s: &str) -> bool {
    if s.is_empty() || s.len() > 253 { return false; }
    for label in s.split('.') {
        if label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-') { return false; }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') { return false; }
    }
    s.contains('.')
}

fn extract_zip(data: &[u8], target: &Path) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(data)).map_err(|e| format!("Bad zip: {e}"))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Read entry {i}: {e}"))?;
        let name = file.name().to_string();
        if name.is_empty() || name.contains("..") || name.starts_with('/') || name.starts_with('\\') { continue; }
        let out = target.join(&name);
        if file.is_dir() { std::fs::create_dir_all(&out).map_err(|e| format!("Create dir: {e}"))?; }
        else {
            if let Some(p) = out.parent() { std::fs::create_dir_all(p).map_err(|e| format!("Create parent: {e}"))?; }
            let mut f = std::fs::File::create(&out).map_err(|e| format!("Create file: {e}"))?;
            std::io::copy(&mut file, &mut f).map_err(|e| format!("Write: {e}"))?;
        }
    }
    Ok(())
}

fn create_output_zip(dir: &Path) -> Result<Vec<u8>, String> {
    use std::io::Write;
    let buf = Cursor::new(Vec::new());
    let mut w = zip::ZipWriter::new(buf);
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for entry in std::fs::read_dir(dir).map_err(|e| format!("Read dir: {e}"))? {
        let e = entry.map_err(|e| format!("Entry: {e}"))?;
        if !e.path().is_file() { continue; }
        let name = e.file_name().to_str().unwrap_or("file").to_string();
        w.start_file(&name, opts).map_err(|e| format!("Zip start: {e}"))?;
        let data = std::fs::read(e.path()).map_err(|e| format!("Read: {e}"))?;
        w.write_all(&data).map_err(|e| format!("Zip write: {e}"))?;
    }
    Ok(w.finish().map_err(|e| format!("Zip finish: {e}"))?.into_inner())
}

fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_json<T: serde::Serialize>(path: &Path, data: &T) {
    if let Ok(json) = serde_json::to_vec_pretty(data) {
        let _ = std::fs::write(path, json);
    }
}

// ── CRM (SQLite) ──

fn crm_db(state: &AppState) -> rusqlite::Connection {
    let db_path = state.data_dir.join("crm.db");
    let conn = rusqlite::Connection::open(&db_path).expect("crm.db");
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS prospects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain TEXT UNIQUE NOT NULL,
            name TEXT,
            site_type TEXT,
            pages_est INTEGER DEFAULT 0,
            tokens_saved_year INTEGER DEFAULT 0,
            contact_email TEXT,
            contact_name TEXT,
            priority INTEGER DEFAULT 1,
            status TEXT DEFAULT 'new',
            notes TEXT DEFAULT '',
            source_url TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS outreach (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            prospect_id INTEGER REFERENCES prospects(id),
            action TEXT,
            detail TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
    ").ok();
    conn
}

async fn serve_crm(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let token = get_cookie(&headers);
    let user = match get_session_user(&state, &token).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };
    let status_filter = params.get("status").cloned().unwrap_or_default();
    let conn = crm_db(&state);

    let status_counts: Vec<(String, i64)> = {
        let mut stmt = conn.prepare("SELECT status, COUNT(*) FROM prospects GROUP BY status").unwrap();
        stmt.query_map([], |row| Ok((row.get::<_,String>(0).unwrap_or_default(), row.get::<_,i64>(1).unwrap_or(0))))
            .unwrap().filter_map(|r| r.ok()).collect()
    };

    let sql = if status_filter.is_empty() { "SELECT * FROM prospects ORDER BY priority DESC, id DESC LIMIT 200".to_string() } else { format!("SELECT * FROM prospects WHERE status = '{}' ORDER BY priority DESC, id DESC LIMIT 200", status_filter.replace('\'', "''")) };
    let mut stmt = conn.prepare(&sql).unwrap();
    let prospects: Vec<HashMap<String, String>> = stmt.query_map([], |row| {
        let mut m = HashMap::new();
        for i in 0..14 {
            if let Ok(v) = row.get::<_, String>(i) { m.insert(format!("c{i}"), v); }
        }
        Ok(m)
    }).unwrap().filter_map(|r| r.ok()).collect();

    let total: i64 = conn.query_row("SELECT COUNT(*) FROM prospects", [], |r| r.get(0)).unwrap_or(0);
    let contacted: i64 = conn.query_row("SELECT COUNT(*) FROM prospects WHERE status != 'new'", [], |r| r.get(0)).unwrap_or(0);

    let mut rows = String::new();
    for p in &prospects {
        let status_class = match p.get("c8").map(|s| s.as_str()).unwrap_or("new") {
            "converted" => "badge-ok", "contacted" | "replied" => "badge-wait", _ => "",
        };
        rows.push_str(&format!(
            r#"<tr><td><a href="/crm/prospects?id={id}">{domain}</a></td><td>{name}</td><td>{stype}</td><td>{pages} p</td><td>{tokens} tk/j</td><td>{contact}</td><td><span class="badge {sc}">{status}</span></td><td><form method="post" action="/crm/prospects/{id}/status" style="display:inline"><select name="status" onchange="this.form.submit()"><option value="">→</option><option value="contacted">contacted</option><option value="replied">replied</option><option value="converted">converted</option><option value="ignored">ignored</option></select></form></td></tr>"#,
            id = p.get("c0").unwrap_or(&"?".into()),
            domain = p.get("c1").unwrap_or(&"?".into()),
            name = truncate(&p.get("c2").unwrap_or(&"?".into()), 40),
            stype = p.get("c3").unwrap_or(&"?".into()),
            pages = p.get("c4").unwrap_or(&"0".into()),
            tokens = fmt_tokens(p.get("c5").unwrap_or(&"0".into()).parse().unwrap_or(0)),
            contact = p.get("c7").unwrap_or(&"-".into()),
            status = p.get("c8").unwrap_or(&"new".into()),
            sc = status_class,
        ));
    }

    let status_links: String = [("new","New"),("contacted","Contacted"),("replied","Replied"),("converted","Converted")].iter()
        .map(|(s,l)| format!(r#"<a href="/crm?status={s}" style="margin-right:1rem;color:{}">{l} ({})</a>"#,
            if status_filter == *s { "#f97316" } else { "#888" },
            status_counts.iter().find(|(st,_)| st == s).map(|(_,c)| c).unwrap_or(&0)
        )).collect();

    let html = format!(r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>CRM — text2cbor</title><style>{STYLE}</style></head><body><h1>text2<span>cbor</span> <span style="font-size:.6em;color:#888">CRM</span></h1>{nav}<div class="statb"><div class="stat"><div class="n">{total}</div><div class="l">Prospects</div></div><div class="stat"><div class="n">{contacted}</div><div class="l">Contacted</div></div><div class="stat"><div class="n" style="font-size:1rem;line-height:2rem"><form method="post" action="/crm/scrape"><button class="btn btn-sm">🔄 Scrape</button></form></div><div class="l">Auto-enrich</div></div></div><div class="nav">{status_links}</div><form method="post" action="/crm/prospects/add" style="margin-bottom:1rem"><input name="domain" placeholder="domain.com" required style="padding:.4rem;background:#1a1a1a;border:1px solid #333;color:#e0e0e0;border-radius:4px;margin-right:.5rem"><input name="name" placeholder="Company name" style="padding:.4rem;background:#1a1a1a;border:1px solid #333;color:#e0e0e0;border-radius:4px;margin-right:.5rem"><button class="btn btn-sm">+ Add</button></form><table><tr><th>Domain</th><th>Name</th><th>Type</th><th>Pages</th><th>Tokens</th><th>Contact</th><th>Status</th><th>Action</th></tr>{rows}</table><div class="foot"><a href="/crm/export">Export CSV</a> — text2cbor CRM — ExploDev 2026</div></body></html>"#,
        nav = nav_html(true, &user.email), total = total, contacted = contacted,
        rows = rows, status_links = status_links,
    );
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}

async fn api_prospects(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let conn = crm_db(&state);
    let mut stmt = conn.prepare("SELECT id, domain, name, site_type, priority, status FROM prospects ORDER BY id DESC LIMIT 50").unwrap();
    let list: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_,i64>(0)?, "domain": row.get::<_,String>(1)?, "name": row.get::<_,String>(2)?,
            "type": row.get::<_,String>(3)?, "priority": row.get::<_,i64>(4)?, "status": row.get::<_,String>(5)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    Json(serde_json::json!({"prospects": list}))
}

async fn api_add_prospect(
    State(state): State<Arc<AppState>>,
    mut form: axum::extract::Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let domain = form.remove("domain").unwrap_or_default().trim().to_string();
    let name = form.remove("name").unwrap_or_default().trim().to_string();
    if domain.is_empty() { return Redirect::to("/crm"); }
    let conn = crm_db(&state);
    conn.execute("INSERT OR IGNORE INTO prospects (domain, name, site_type, priority) VALUES (?1, ?2, 'manual', 3)", rusqlite::params![domain, name]).ok();
    Redirect::to("/crm")
}

async fn api_update_status(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    mut form: axum::extract::Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let status = form.remove("status").unwrap_or_default();
    if status.is_empty() { return Redirect::to("/crm"); }
    let conn = crm_db(&state);
    conn.execute("UPDATE prospects SET status = ?1, updated_at = datetime('now') WHERE id = ?2", rusqlite::params![status, id]).ok();
    conn.execute("INSERT INTO outreach (prospect_id, action, detail) VALUES (?1, ?2, ?3)", rusqlite::params![id, format!("status: {status}"), ""]).ok();
    Redirect::to("/crm")
}

async fn api_add_note(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    mut form: axum::extract::Form<HashMap<String, String>>,
) -> Response {
    let note = form.remove("note").unwrap_or_default();
    if note.is_empty() { let url = format!("/crm?id={id}"); return Redirect::to(&url).into_response(); }
    let conn = crm_db(&state);
    conn.execute("UPDATE prospects SET notes = notes || ?1 || '\n', updated_at = datetime('now') WHERE id = ?2", rusqlite::params![format!("[{}] {note}", chrono::Utc::now().format("%Y-%m-%d")), id]).ok();
    let url = format!("/crm?id={id}");
    Redirect::to(&url).into_response()
}

async fn api_trigger_scrape(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = crm_db(&state);
    let mut added = 0u32;

    let sources: Vec<(&str, &str, &str)> = vec![
        ("techcrunch.com", "TechCrunch", "tech_blog"),
        ("theverge.com", "The Verge", "tech_blog"),
        ("arstechnica.com", "Ars Technica", "tech_blog"),
        ("dev.to", "DEV Community", "tech_blog"),
        ("css-tricks.com", "CSS-Tricks", "tech_blog"),
        ("smashingmagazine.com", "Smashing Magazine", "tech_blog"),
        ("react.dev", "React Docs", "docs"),
        ("docs.python.org", "Python Docs", "docs"),
        ("nodejs.org", "Node.js Docs", "docs"),
        ("kubernetes.io", "Kubernetes Docs", "docs"),
        ("docs.docker.com", "Docker Docs", "docs"),
        ("docs.github.com", "GitHub Docs", "docs"),
        ("rust-lang.org", "Rust Docs", "docs"),
        ("golang.org", "Go Docs", "docs"),
        ("php.net", "PHP Docs", "docs"),
        ("digitad.fr", "Digitad", "agency"),
        ("alsacreations.fr", "Alsacréations", "agency"),
        ("octo.com", "OCTO Technology", "agency"),
        ("synbioz.com", "Synbioz", "agency"),
        ("leboncoin.fr", "Leboncoin", "ecommerce"),
        ("ldlc.com", "LDLC", "ecommerce"),
    ];

    for (domain, name, stype) in &sources {
        let url = format!("https://{domain}");
        let html = match scrape_fetch(&url) {
            Some(h) => h,
            None => {
                let html2 = match scrape_fetch(&format!("http://{domain}")) {
                    Some(h) => h, None => continue,
                };
                html2
            }
        };

        let pages_est = scrape_pages(&html);
        let tokens = pages_est as i64 * 19000 * 1000 * 365;
        let contact = scrape_email(&html, domain);
        let has_blog = scrape_has_blog(&html);
        let has_shop = scrape_has_shop(&html);

        let resolved_type = if has_shop { "ecommerce" } else if has_blog { "tech_blog" } else { *stype };
        let priority = calc_priority(resolved_type, pages_est);

        let result = conn.execute(
            "INSERT OR IGNORE INTO prospects (domain, name, site_type, pages_est, tokens_saved_year, contact_email, priority, source_url) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![domain, name, resolved_type, pages_est, tokens, contact, priority, format!("https://{domain}")],
        );
        if result.is_ok() && conn.changes() > 0 { added += 1; }

        tracing::info!(
            "Scraped {}: type={}, pages={}, tokens={}, contact={}, priority={}",
            domain, resolved_type, pages_est, tokens, contact, priority
        );
    }

    tracing::info!("Scrape: enriched {} prospects", added);
    Redirect::to("/crm")
}

fn scrape_fetch(url: &str) -> Option<String> {
    let out = Command::new("curl")
        .args(["-sL", "--max-time", "12", "-A", "Mozilla/5.0 (compatible; CBOR-Web CRM/1.0)", url])
        .output()
        .ok()?;
    if out.status.success() { String::from_utf8(out.stdout).ok() } else { None }
}

fn scrape_pages(html: &str) -> usize {
    let link_count = html.matches("<a ").count() + html.matches("<a>").count() + html.matches("href=").count() / 2;
    let link_est = (link_count as f64 / 3.0).ceil() as usize;

    let re_sitemap = Regex::new(r#"https?://[^"'\s<>]+sitemap[^"'\s<>]*\.xml"#).unwrap();
    if let Some(sitemap_url) = re_sitemap.find(html) {
        if let Some(sitemap_xml) = scrape_fetch(sitemap_url.as_str()) {
            let url_count = sitemap_xml.matches("<url>").count();
            if url_count > 0 { return url_count; }
        }
    }
    link_est.max(1)
}

fn scrape_email(html: &str, domain: &str) -> String {
    let re = Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap();
    re.find(html).map(|m| m.as_str().to_string()).unwrap_or_else(|| format!("contact@{domain}"))
}

fn scrape_has_blog(html: &str) -> bool {
    let lower = html.to_lowercase();
    lower.contains("wp-content") || lower.contains("/blog/") || lower.contains("/articles/")
        || lower.contains("wordpress") || lower.contains("ghost") || lower.contains("rss")
}

fn scrape_has_shop(html: &str) -> bool {
    let lower = html.to_lowercase();
    lower.contains("woocommerce") || lower.contains("shopify") || lower.contains("/shop/")
        || lower.contains("/product/") || lower.contains("add-to-cart") || lower.contains("checkout")
        || lower.contains("prestashop") || lower.contains("magento")
}

fn calc_priority(site_type: &str, pages: usize) -> i64 {
    let mut p = 1i64;
    if pages > 50 { p += 1; }
    if pages > 200 { p += 1; }
    match site_type {
        "tech_blog" | "agency" | "ecommerce" => p += 1,
        "docs" => p += 2,
        _ => {}
    }
    p.min(5)
}

async fn api_export_csv(State(state): State<Arc<AppState>>) -> Response {
    let conn = crm_db(&state);
    let mut stmt = conn.prepare("SELECT domain, name, site_type, pages_est, tokens_saved_year, contact_email, priority, status FROM prospects ORDER BY priority DESC").unwrap();
    let mut csv = String::from("domain,name,type,pages,tokens_saved_year,contact,priority,status\n");
    let rows = stmt.query_map([], |row| {
        Ok(format!("{},{},{},{},{},{},{},{}",
            row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?,
            row.get::<_,i64>(3)?, row.get::<_,i64>(4)?, row.get::<_,String>(5)?,
            row.get::<_,i64>(6)?, row.get::<_,String>(7)?,
        ))
    }).unwrap();
    for r in rows { if let Ok(line) = r { csv.push_str(&line); csv.push('\n'); } }
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/csv; charset=utf-8"), (header::CONTENT_DISPOSITION, "attachment; filename=\"cbor-crm-prospects.csv\"")], csv).into_response()
}

fn fmt_tokens(n: i64) -> String {
    if n >= 1_000_000_000 { format!("{:.1}B", n as f64 / 1e9) }
    else if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1e6) }
    else if n >= 1_000 { format!("{}K", n / 1000) }
    else { n.to_string() }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max { s.to_string() } else { format!("{}...", s.chars().take(max).collect::<String>()) }
}
