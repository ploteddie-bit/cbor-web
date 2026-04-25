use axum::{
    extract::{Multipart, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;

// ── Types ──

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct User {
    email: String,
    password_hash: String,
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
        .route("/", get(serve_dashboard))
        .route("/login", get(serve_login_page).post(handle_login))
        .route("/register", get(serve_register_page).post(handle_register))
        .route("/logout", get(handle_logout))
        .route("/upload", post(upload_handler))
        .route("/invoices", get(serve_invoices))
        .route("/invoice/{id}", get(serve_invoice))
        .route("/api/status", get(api_status))
        .route("/admin", get(serve_admin))
        .route("/admin/invoice/{id}/paid", post(mark_invoice_paid))
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
            r#"<div class="nav"><a href="/">Dashboard</a><a href="/invoices">Invoices</a><a href="/api/status">API</a><a href="https://github.com/ploteddie-bit/cbor-web">GitHub</a><span style="float:right;color:#888;font-size:.8rem">{email}</span><a href="/logout" style="float:right;margin-right:1rem">Logout</a></div>"#,
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
    users.push(User { email: email.clone(), password_hash: hash_password(&pw), created_at: now_ts(), upload_count: 0, total_pages: 0, plan: "Trial (5 free)".into() });
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
