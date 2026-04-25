#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

#[tauri::command]
fn audit_url(url: String) -> String {
    format!("{{\"status\":\"analyzing\",\"url\":\"{}\",\"message\":\"CBOR-Web Audit Engine v2.0 — Rust/WASM\"}}", url)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![audit_url])
        .run(tauri::generate_context!())
        .expect("error while running CBOR-Web Audit");
}
