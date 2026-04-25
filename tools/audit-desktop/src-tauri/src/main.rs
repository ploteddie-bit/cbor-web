#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod telemetry;
use telemetry::Telemetry;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    telemetry: Mutex<Telemetry>,
}

#[tauri::command]
fn track_scan(domain: String, state: State<AppState>) {
    state.telemetry.lock().unwrap().track_scan(&domain);
}

#[tauri::command]
fn track_crawl(domain: String, elapsed_secs: u64, state: State<AppState>) {
    state.telemetry.lock().unwrap().track_crawl(&domain, elapsed_secs);
}

#[tauri::command]
fn get_telemetry(state: State<AppState>) -> serde_json::Value {
    state.telemetry.lock().unwrap().report()
}

// (license functions kept from before, omitted for brevity)
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
struct License {
    key: String, email: String, plan: String, issued_at: u64, expires_at: u64, verified: bool,
}

#[tauri::command]
fn check_license(key: String, email: String) -> Result<License, String> {
    if !key.starts_with("CBW-") || key.len() < 19 {
        return Err("Invalid license format".into());
    }
    let expected = hex::encode(&Sha256::digest(format!("cbor-web:{}:secret-salt-v2", email).as_bytes())[..8]);
    let provided = key.replace("CBW-", "").replace("-", "").to_lowercase();
    if provided != expected { return Err("Invalid license".into()); }
    let license = License { key, email, plan: "Standard".into(),
        issued_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        expires_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 365 * 86400,
        verified: true };
    if let Ok(json) = serde_json::to_string_pretty(&license) {
        let _ = fs::write(dirs_next::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join(".cbor-web-audit-license.json"), json);
    }
    Ok(license)
}

#[tauri::command]
fn get_saved_license() -> Option<License> {
    fs::read_to_string(dirs_next::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join(".cbor-web-audit-license.json"))
        .ok().and_then(|s| serde_json::from_str(&s).ok())
}

#[tauri::command]
fn audit_url(url: String) -> String {
    format!("{{\"status\":\"ok\",\"url\":\"{}\"}}", url)
}

fn main() {
    let telemetry = Telemetry::load();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState { telemetry: Mutex::new(telemetry) })
        .invoke_handler(tauri::generate_handler![audit_url, check_license, get_saved_license, track_scan, track_crawl, get_telemetry])
        .run(tauri::generate_context!())
        .expect("error running CBOR-Web Audit");
}
