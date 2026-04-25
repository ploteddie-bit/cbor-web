use axum::{
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let data_dir = PathBuf::from("data");
    std::fs::create_dir_all(data_dir.join("uploads")).expect("Failed to create uploads dir");

    let state = Arc::new(AppState {
        started_at: Instant::now(),
        upload_count: AtomicU64::new(0),
    });

    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/upload", axum::routing::post(upload_handler))
        .route("/api/status", get(api_status))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .expect("Failed to bind");
    tracing::info!("text2cbor-web listening on 0.0.0.0:3002");
    axum::serve(listener, app).await.expect("Server error");
}

struct AppState {
    started_at: Instant,
    upload_count: AtomicU64,
}

fn text2cbor_binary() -> PathBuf {
    let candidates = [
        PathBuf::from("../text2cbor/target/release/text2cbor"),
        PathBuf::from("/usr/local/bin/text2cbor"),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    PathBuf::from("text2cbor")
}

async fn serve_dashboard() -> Response {
    let style = "*{margin:0;padding:0;box-sizing:border-box}body{font-family:system-ui,monospace;background:#0a0a0a;color:#e0e0e0;padding:2rem}h1{color:#f97316;margin-bottom:.5rem}h1 span{color:#fff}.sub{color:#888;margin-bottom:1.5rem}.nav{margin-bottom:2rem}.nav a{margin-right:1.5rem;color:#f97316;text-decoration:none}.nav a:hover{text-decoration:underline}.card{background:#111;border-radius:8px;padding:2rem;max-width:640px;margin-bottom:2rem}.card h2{color:#f97316;margin-bottom:1rem;font-size:1.1rem}.field{margin-bottom:1.2rem}.field label{display:block;color:#888;font-size:.8rem;margin-bottom:.4rem;text-transform:uppercase;letter-spacing:.05em}.field input[type=text],.field input[type=file]{width:100%;padding:.6rem .8rem;background:#1a1a1a;border:1px solid #333;border-radius:6px;color:#e0e0e0;font-family:inherit;font-size:.9rem}.field input[type=file]::file-selector-button{padding:.4rem 1rem;margin-right:.8rem;background:#f97316;color:#000;border:none;border-radius:4px;cursor:pointer;font-weight:600;font-family:inherit}.btn{background:#f97316;color:#000;border:none;padding:.7rem 1.8rem;border-radius:6px;font-weight:700;font-size:.9rem;cursor:pointer;font-family:inherit}.btn:hover{opacity:.9}.btn:disabled{opacity:.4;cursor:not-allowed}#progress{display:none;margin-top:1rem;color:#f97316;font-size:.85rem}#result{display:none;margin-top:1rem;background:#111;padding:1rem;border-radius:6px;border-left:3px solid #28c840}#result a{color:#f97316}#result .err{border-left-color:#ef4444;color:#ef4444}.statb{display:flex;gap:2rem;flex-wrap:wrap;margin:1rem 0 2rem}.stat{background:#111;padding:1rem;border-radius:8px;min-width:120px}.stat .n{font-size:2rem;color:#f97316;font-weight:700}.stat .l{color:#666;font-size:.8rem}.foot{margin-top:3rem;color:#444;font-size:.7rem}";

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>text2cbor · SaaS</title>
<style>{style}</style>
</head>
<body>
<h1>text2<span>cbor</span></h1>
<p class="sub">Convert HTML websites to CBOR-Web v3.0 index.cbor — drag, drop, deploy</p>
<div class="nav">
  <a href="/">Dashboard</a>
  <a href="/api/status">API Status</a>
  <a href="https://github.com/ploteddie-bit/cbor-web/tree/main/tools/text2cbor">CLI Docs</a>
</div>

<div class="card">
  <h2>Upload &amp; Convert</h2>
  <form id="uploadForm" enctype="multipart/form-data">
    <div class="field">
      <label for="domain">Domain name</label>
      <input type="text" id="domain" name="domain" placeholder="example.com" required>
    </div>
    <div class="field">
      <label for="file">HTML site (.zip)</label>
      <input type="file" id="file" name="file" accept=".zip" required>
    </div>
    <button type="submit" class="btn" id="submitBtn">Convert to CBOR</button>
  </form>
  <div id="progress">Processing... this may take a few seconds</div>
  <div id="result"></div>
</div>

<div id="statsContainer" class="statb">
  <div class="stat"><div class="n" id="statUptime">—</div><div class="l">Uptime (s)</div></div>
  <div class="stat"><div class="n" id="statUploads">—</div><div class="l">Uploads Processed</div></div>
</div>

<div class="foot">text2cbor-web v{} — ExploDev 2026</div>

<script>
const form = document.getElementById('uploadForm');
const progress = document.getElementById('progress');
const result = document.getElementById('result');
const submitBtn = document.getElementById('submitBtn');

form.onsubmit = async (e) => {{
  e.preventDefault();
  const data = new FormData(form);
  progress.style.display = 'block';
  result.style.display = 'none';
  submitBtn.disabled = true;

  try {{
    const resp = await fetch('/upload', {{ method: 'POST', body: data }});
    if (resp.ok) {{
      const blob = await resp.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'cbor-output.zip';
      a.click();
      URL.revokeObjectURL(url);
      result.style.display = 'block';
      result.innerHTML = '<div style="color:#28c840">Conversion complete. Download started.</div>';
    }} else {{
      const txt = await resp.text();
      result.style.display = 'block';
      result.innerHTML = '<div class="err">Error: ' + txt + '</div>';
    }}
  }} catch (err) {{
    result.style.display = 'block';
    result.innerHTML = '<div class="err">Network error: ' + err.message + '</div>';
  }} finally {{
    progress.style.display = 'none';
    submitBtn.disabled = false;
  }}
  fetchStats();
}};

async function fetchStats() {{
  try {{
    const r = await fetch('/api/status');
    const d = await r.json();
    document.getElementById('statUptime').textContent = d.uptime_secs;
    document.getElementById('statUploads').textContent = d.uploads_processed;
  }} catch (_) {{}}
}}
fetchStats();
</script>
</body>
</html>"#,
        env!("CARGO_PKG_VERSION")
    );

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
        .into_response()
}

async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Response {
    let mut domain: Option<String> = None;
    let mut zip_data: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "domain" => {
                if let Ok(text) = field.text().await {
                    let d = text.trim().to_string();
                    if !d.is_empty() && is_valid_domain(&d) {
                        domain = Some(d);
                    }
                }
            }
            "file" => {
                if let Ok(bytes) = field.bytes().await {
                    if !bytes.is_empty() {
                        zip_data = Some(bytes.to_vec());
                    }
                }
            }
            _ => {}
        }
    }

    let domain = match domain {
        Some(d) => d,
        None => return (StatusCode::BAD_REQUEST, "Missing or invalid domain name").into_response(),
    };
    let zip_bytes = match zip_data {
        Some(b) => b,
        None => return (StatusCode::BAD_REQUEST, "Missing zip file").into_response(),
    };

    let work_id = uuid::Uuid::new_v4().to_string();
    let tmp_dir = PathBuf::from("data/uploads").join(&work_id);
    let site_dir = tmp_dir.join("site");
    let output_dir = tmp_dir.join("output");

    if let Err(e) = std::fs::create_dir_all(&site_dir) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create temp dir: {e}"))
            .into_response();
    }
    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create output dir: {e}"))
            .into_response();
    }

    if let Err(e) = extract_zip(&zip_bytes, &site_dir) {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return (StatusCode::BAD_REQUEST, format!("Failed to extract zip: {e}")).into_response();
    }

    let binary = text2cbor_binary();
    let cmd_output = Command::new(&binary)
        .args([
            "generate",
            "--input",
            site_dir.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
            "--domain",
            &domain,
        ])
        .output();

    match cmd_output {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::warn!("text2cbor failed for {}: {}", domain, stderr);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("text2cbor failed: {}", stderr.trim()),
            )
                .into_response();
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            tracing::error!("Failed to run text2cbor: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to run text2cbor binary ({binary:?}): {e}"),
            )
                .into_response();
        }
    }

    match create_output_zip(&output_dir) {
        Ok(zip) => {
            state.upload_count.fetch_add(1, Ordering::Relaxed);
            tracing::info!("Converted {} ({}) -> {} byte output", domain, work_id, zip.len());

            let cleanup_dir = tmp_dir.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(3600)).await;
                let _ = tokio::fs::remove_dir_all(&cleanup_dir).await;
            });

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "application/zip"),
                    (
                        header::CONTENT_DISPOSITION,
                        &format!("attachment; filename=\"{}-cbor.zip\"", domain).as_str(),
                    ),
                ],
                zip,
            )
                .into_response()
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create output zip: {e}"))
                .into_response()
        }
    }
}

async fn api_status(State(state): State<Arc<AppState>>) -> Response {
    let body = serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": state.started_at.elapsed().as_secs(),
        "uploads_processed": state.upload_count.load(Ordering::Relaxed),
    });
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_vec_pretty(&body).unwrap_or_default(),
    )
        .into_response()
}

fn is_valid_domain(s: &str) -> bool {
    if s.is_empty() || s.len() > 253 {
        return false;
    }
    for label in s.split('.') {
        if label.is_empty() || label.len() > 63 {
            return false;
        }
        if label.starts_with('-') || label.ends_with('-') {
            return false;
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
    }
    s.contains('.')
}

fn extract_zip(data: &[u8], target: &Path) -> Result<(), String> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| format!("Bad zip: {e}"))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Read entry {i}: {e}"))?;
        let name = file.name().to_string();
        if name.is_empty() || name.contains("..") || name.starts_with('/') || name.starts_with('\\')
        {
            continue;
        }
        let out_path = target.join(&name);
        if file.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| format!("Create dir {}: {e}", out_path.display()))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Create parent {}: {e}", parent.display()))?;
            }
            let mut out_file = std::fs::File::create(&out_path)
                .map_err(|e| format!("Create file {}: {e}", out_path.display()))?;
            std::io::copy(&mut file, &mut out_file)
                .map_err(|e| format!("Write file {}: {e}", out_path.display()))?;
        }
    }
    Ok(())
}

fn create_output_zip(output_dir: &Path) -> Result<Vec<u8>, String> {
    let buf = Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(buf);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let entries = std::fs::read_dir(output_dir)
        .map_err(|e| format!("Read output dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Read entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let fname = path.file_name().unwrap().to_str().unwrap_or("file");
        writer
            .start_file(fname, options)
            .map_err(|e| format!("Start zip entry {fname}: {e}"))?;
        let data =
            std::fs::read(&path).map_err(|e| format!("Read file {}: {e}", path.display()))?;
        writer
            .write_all(&data)
            .map_err(|e| format!("Write zip entry {fname}: {e}"))?;
    }

    let cursor = writer.finish().map_err(|e| format!("Finish zip: {e}"))?;
    Ok(cursor.into_inner())
}
