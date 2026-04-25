//! cbor-crawl — CBOR-Web crawler for AI agents
//!
//! Discovers, fetches, and outputs structured content from CBOR-Web endpoints.
//! Reference consumer implementation for the CBOR-Web specification v2.1.

use ciborium::Value;
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::{json, Map, Value as JsonValue};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Parser)]
#[command(name = "cbor-crawl", version, about = "CBOR-Web crawler for AI agents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display manifest info: pages, languages, sizes, access tiers
    Inspect {
        /// Site URL (e.g. https://deltopide.fr)
        url: String,
    },
    /// Fetch full site content and output as JSON or text
    Fetch {
        /// Site URL
        url: String,
        /// Output format: json, text
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Output directory for saving page files
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Verify a local CBOR file (structure, encoding, hash)
    Verify {
        /// Path to .cbor file
        file: String,
    },
    /// Search for a term across all CBOR-Web pages
    Search {
        /// Site URL
        url: String,
        /// Search query (case-insensitive)
        query: String,
    },
    /// Send Doléance feedback to a CBOR-Web publisher
    Doleance {
        /// Site URL
        url: String,
        /// Feedback JSON: {"signals":[{"signal":"missing_data","details":"...","block_type":"..."}],"page_path":"/..."}
        #[arg(long)]
        feedback: String,
    },
    /// Fetch and display a diff manifest (incremental changes)
    Diff {
        /// Site URL
        url: String,
        /// Base version hash (hex SHA-256 of previous manifest)
        #[arg(long)]
        base_version: Option<String>,
    },
}

// ── CBOR-Web constants ──

const WELL_KNOWN: &str = "/.well-known/cbor-web";
const WELL_KNOWN_PAGES: &str = "/.well-known/cbor-web/pages/";
const WELL_KNOWN_BUNDLE: &str = "/.well-known/cbor-web/bundle";
const SELF_DESCRIBED_TAG: u64 = 55799;
const MAX_MANIFEST_SIZE: usize = 5 * 1024 * 1024;
const MAX_PAGE_SIZE: usize = 1024 * 1024;
const MAX_BUNDLE_SIZE: usize = 50 * 1024 * 1024;

// ── Main ──

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = Client::builder()
        .user_agent("cbor-crawl/0.1.0 (cbor-web)")
        .build()?;

    match cli.command {
        Commands::Inspect { url } => cmd_inspect(&client, &url).await,
        Commands::Fetch { url, format, output } => cmd_fetch(&client, &url, &format, output).await,
        Commands::Verify { file } => cmd_verify(&file),
        Commands::Search { url, query } => cmd_search(&client, &url, &query).await,
        Commands::Doleance { url, feedback } => cmd_doleance(&client, &url, &feedback).await,
        Commands::Diff { url, base_version } => cmd_diff(&client, &url, base_version).await,
    }
}

// ── Inspect command ──

async fn cmd_inspect(client: &Client, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let base = base_url.trim_end_matches('/');
    let manifest_url = format!("{}{}", base, WELL_KNOWN);

    eprintln!("Fetching manifest: {}", manifest_url);
    let bytes = fetch_cbor(client, &manifest_url, MAX_MANIFEST_SIZE).await?;
    let manifest = parse_cbor_document(&bytes)?;

    // Extract document type and version
    let doc_type = get_map_value(&manifest, &Value::Integer(0.into()));
    let (type_name, spec_version) = match doc_type {
        Some(Value::Text(s)) if s == "cbor-web" => ("cbor-web (unified)", "v3.0"),
        Some(Value::Text(s)) if s == "cbor-web-manifest" => ("cbor-web-manifest", "v2.1"),
        Some(Value::Text(s)) => (s.as_str(), "unknown"),
        _ => ("unknown", "unknown"),
    };
    // Extract site metadata (key 2)
    let site = get_map_value(&manifest, &Value::Integer(2.into()));
    let domain = get_text_field(site, "domain").unwrap_or("unknown");
    let name = get_text_field(site, "name").unwrap_or("unknown");
    let description = get_text_field(site, "description").unwrap_or("");
    let languages = get_array_field(site, "languages");
    let default_lang = get_text_field(site, "default_language").unwrap_or("?");

    // Extract meta (key 5 for v2.1, key 6 for v3.0)
    let meta_key = if type_name.contains("unified") { 6 } else { 5 };
    let pages_key = if type_name.contains("unified") { 5 } else { 3 };
    let meta = get_map_value(&manifest, &Value::Integer(meta_key.into()));
    let total_pages = get_uint_field(meta, "total_pages").unwrap_or(0);
    let total_size = get_uint_field(meta, "total_size").unwrap_or(0);
    let bundle_available = get_bool_field(meta, "bundle_available").unwrap_or(false);
    let generator = get_text_field(meta, "generator").unwrap_or("unknown");

    // Extract pages
    let pages = get_array_value(&manifest, &Value::Integer(pages_key.into()));

    println!("╔══════════════════════════════════════════════════╗");
    println!("║  CBOR-Web Manifest — {}",  domain);
    println!("╠══════════════════════════════════════════════════╣");
    println!("║  Format:     {} ({})", type_name, spec_version);
    println!("║  Name:       {}", name);
    if !description.is_empty() {
        println!("║  Description:{}", truncate(description, 50));
    }
    println!("║  Languages:  {} (default: {})", languages.join(", "), default_lang);
    println!("║  Pages:      {}", total_pages);
    println!("║  Total size: {} bytes ({:.1} KB)", total_size, total_size as f64 / 1024.0);
    println!("║  Bundle:     {}", if bundle_available { "available" } else { "not available" });
    println!("║  Generator:  {}", generator);
    println!("╠══════════════════════════════════════════════════╣");
    println!("║  Pages:");

    if let Some(pages_arr) = pages {
        for page in pages_arr {
            if let Value::Map(ref entries) = page {
                let path = find_text_in_map(entries, "path").unwrap_or("/");
                let title = find_text_in_map(entries, "title").unwrap_or("(no title)");
                let access = find_text_in_map(entries, "access").unwrap_or("?");
                let size = find_uint_in_map(entries, "size").unwrap_or(0);
                let lang = find_text_in_map(entries, "lang").unwrap_or("?");
                println!("║    {} [{}] ({}) — {} bytes, {}",
                    path, lang, access, size, truncate(title, 40));
            }
        }
    }

    println!("╚══════════════════════════════════════════════════╝");
    Ok(())
}

// ── Fetch command ──

async fn cmd_fetch(
    client: &Client,
    base_url: &str,
    format: &str,
    output_dir: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let base = base_url.trim_end_matches('/');
    let manifest_url = format!("{}{}", base, WELL_KNOWN);

    eprintln!("Fetching manifest: {}", manifest_url);
    let manifest_bytes = fetch_cbor(client, &manifest_url, MAX_MANIFEST_SIZE).await?;
    let manifest = parse_cbor_document(&manifest_bytes)?;

    let site = get_map_value(&manifest, &Value::Integer(2.into()));
    let domain = get_text_field(site, "domain").unwrap_or("unknown");

    let doc_type = get_map_value(&manifest, &Value::Integer(0.into()));
    let is_unified = matches!(doc_type, Some(Value::Text(s)) if s == "cbor-web");
    let meta_key: u64 = if is_unified { 6 } else { 5 };
    let pages_key: u64 = if is_unified { 5 } else { 3 };

    let meta = get_map_value(&manifest, &Value::Integer(meta_key.into()));
    let bundle_available = get_bool_field(meta, "bundle_available").unwrap_or(false);

    let pages_arr = get_array_value(&manifest, &Value::Integer(pages_key.into()));
    let page_entries: Vec<PageEntry> = match pages_arr {
        Some(arr) => arr.iter().filter_map(parse_page_entry).collect(),
        None => vec![],
    };

    eprintln!("Found {} pages, bundle: {}", page_entries.len(), bundle_available);

    // Try bundle first if available
    let mut page_contents: BTreeMap<String, JsonValue> = BTreeMap::new();

    if bundle_available {
        let bundle_url = format!("{}{}", base, WELL_KNOWN_BUNDLE);
        eprintln!("Fetching bundle: {}", bundle_url);
        match fetch_cbor(client, &bundle_url, MAX_BUNDLE_SIZE).await {
            Ok(bundle_bytes) => {
                let bundle = parse_cbor_document(&bundle_bytes)?;
                // Bundle key 3 = pages map
                if let Some(Value::Map(pages_map)) = get_map_value_owned(&bundle, &Value::Integer(3.into())) {
                    for (key, value) in pages_map {
                        if let Value::Text(path) = key {
                            let content = extract_page_content(&value);
                            page_contents.insert(path, content);
                        }
                    }
                }
                eprintln!("Bundle parsed: {} pages extracted", page_contents.len());
            }
            Err(e) => {
                eprintln!("Bundle fetch failed ({}), falling back to individual pages", e);
            }
        }
    }

    // Fetch individual pages not in bundle
    if page_contents.is_empty() {
        for entry in &page_entries {
            if entry.access == "T0" || entry.access == "token" {
                eprintln!("Skipping {} (requires auth: {})", entry.path, entry.access);
                continue;
            }
            let filename = path_to_filename(&entry.path);
            let page_url = format!("{}{}{}", base, WELL_KNOWN_PAGES, filename);
            eprintln!("Fetching: {}", entry.path);

            match fetch_cbor(client, &page_url, MAX_PAGE_SIZE).await {
                Ok(page_bytes) => {
                    // Verify hash if available
                    if let Some(ref expected_hash) = entry.hash {
                        let computed = sha256_hex(&page_bytes);
                        if computed != *expected_hash {
                            eprintln!("  WARN: hash mismatch for {} (expected {}, got {})",
                                entry.path, expected_hash, computed);
                        }
                    }
                    let page = parse_cbor_document(&page_bytes)?;
                    let content = extract_page_content(&page);
                    page_contents.insert(entry.path.clone(), content);
                }
                Err(e) => {
                    eprintln!("  ERROR fetching {}: {}", entry.path, e);
                }
            }

            // Basic rate limiting (100ms between requests)
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    // Output
    match format {
        "json" => {
            let output = json!({
                "site": domain,
                "pages_count": page_contents.len(),
                "pages": page_contents,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "text" => {
            for (path, content) in &page_contents {
                println!("=== {} ===", path);
                if let Some(blocks) = content.get("blocks") {
                    if let Some(arr) = blocks.as_array() {
                        for block in arr {
                            print_block_text(block);
                        }
                    }
                }
                println!();
            }
        }
        _ => eprintln!("Unknown format: {}. Use json or text.", format),
    }

    // Save to output directory if specified
    if let Some(ref dir) = output_dir {
        std::fs::create_dir_all(dir)?;
        for (path, content) in &page_contents {
            let filename_str = path.trim_start_matches('/')
                .replace('/', "_")
                .trim_start_matches('_')
                .to_string();
            let filename = if filename_str.is_empty() { "index" } else { &filename_str };
            let filepath = std::path::Path::new(dir).join(format!("{}.json", filename));
            std::fs::write(&filepath, serde_json::to_string_pretty(content)?)?;
            eprintln!("  Saved: {}", filepath.display());
        }
        eprintln!("Saved {} pages to {}", page_contents.len(), dir);
    }

    Ok(())
}

// ── Verify command ──

fn cmd_verify(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read(file)?;
    println!("File: {} ({} bytes)", file, bytes.len());

    // Check self-described tag
    if bytes.len() < 3 || bytes[0] != 0xD9 || bytes[1] != 0xD9 || bytes[2] != 0xF7 {
        println!("ERROR: Missing self-described CBOR tag (D9 D9 F7)");
        return Ok(());
    }
    println!("Self-described tag: OK (D9 D9 F7)");

    // Parse
    let doc = parse_cbor_document(&bytes)?;

    // Identify document type
    if let Some(Value::Text(doc_type)) = get_map_value_owned(&doc, &Value::Integer(0.into())) {
        println!("Document type: {}", doc_type);
        match doc_type.as_str() {
            "cbor-web-manifest" => println!("Valid manifest structure"),
            "cbor-web-page" => println!("Valid page structure"),
            "cbor-web-bundle" => println!("Valid bundle structure"),
            other => println!("Unknown document type: {}", other),
        }
    }

    // Version
    if let Some(Value::Integer(ver)) = get_map_value_owned(&doc, &Value::Integer(1.into())) {
        println!("Version: {}", i128::from(ver));
    }

    // SHA-256
    let hash = sha256_hex(&bytes);
    println!("SHA-256: {}", hash);

    println!("Verification: PASS");
    Ok(())
}

// ── HTTP helpers ──

async fn fetch_cbor(
    client: &Client,
    url: &str,
    max_size: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let resp = client
        .get(url)
        .header("Accept", "application/cbor")
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status).into());
    }

    let bytes = resp.bytes().await?.to_vec();
    if bytes.len() > max_size {
        return Err(format!("Response too large: {} > {} bytes", bytes.len(), max_size).into());
    }

    // Validate magic bytes
    if bytes.len() >= 3 && bytes[0] == 0xD9 && bytes[1] == 0xD9 && bytes[2] == 0xF7 {
        Ok(bytes)
    } else {
        Err("Not a self-described CBOR document (missing D9 D9 F7)".into())
    }
}

// ── CBOR parsing helpers ──

fn parse_cbor_document(bytes: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    let value: Value = ciborium::from_reader(bytes)?;
    // Unwrap self-described tag
    match value {
        Value::Tag(SELF_DESCRIBED_TAG, inner) => Ok(*inner),
        _ => Ok(value),
    }
}

fn get_map_value<'a>(map: &'a Value, key: &Value) -> Option<&'a Value> {
    if let Value::Map(entries) = map {
        for (k, v) in entries {
            if k == key {
                return Some(v);
            }
        }
    }
    None
}

fn get_map_value_owned(map: &Value, key: &Value) -> Option<Value> {
    get_map_value(map, key).cloned()
}

fn get_array_value<'a>(map: &'a Value, key: &Value) -> Option<&'a Vec<Value>> {
    match get_map_value(map, key) {
        Some(Value::Array(arr)) => Some(arr),
        _ => None,
    }
}

fn get_text_field<'a>(parent: Option<&'a Value>, key: &str) -> Option<&'a str> {
    let parent = parent?;
    if let Value::Map(entries) = parent {
        for (k, v) in entries {
            if let (Value::Text(k_str), Value::Text(v_str)) = (k, v) {
                if k_str == key {
                    return Some(v_str.as_str());
                }
            }
        }
    }
    None
}

fn get_uint_field(parent: Option<&Value>, key: &str) -> Option<u64> {
    let parent = parent?;
    if let Value::Map(entries) = parent {
        for (k, v) in entries {
            if let Value::Text(k_str) = k {
                if k_str == key {
                    if let Value::Integer(n) = v {
                        return i128::from(*n).try_into().ok();
                    }
                }
            }
        }
    }
    None
}

fn get_bool_field(parent: Option<&Value>, key: &str) -> Option<bool> {
    let parent = parent?;
    if let Value::Map(entries) = parent {
        for (k, v) in entries {
            if let Value::Text(k_str) = k {
                if k_str == key {
                    if let Value::Bool(b) = v {
                        return Some(*b);
                    }
                }
            }
        }
    }
    None
}

fn get_bytes_field<'a>(parent: Option<&'a Value>, key: &str) -> Option<&'a [u8]> {
    let parent = parent?;
    if let Value::Map(entries) = parent {
        for (k, v) in entries {
            if let (Value::Text(k_str), Value::Bytes(b)) = (k, v) {
                if k_str == key {
                    return Some(b.as_slice());
                }
            }
        }
    }
    None
}

fn get_array_field(parent: Option<&Value>, key: &str) -> Vec<String> {
    let parent = match parent {
        Some(p) => p,
        None => return vec![],
    };
    if let Value::Map(entries) = parent {
        for (k, v) in entries {
            if let Value::Text(k_str) = k {
                if k_str == key {
                    if let Value::Array(arr) = v {
                        return arr
                            .iter()
                            .filter_map(|item| {
                                if let Value::Text(s) = item {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                }
            }
        }
    }
    vec![]
}

fn find_text_in_map<'a>(entries: &'a [(Value, Value)], key: &str) -> Option<&'a str> {
    for (k, v) in entries {
        if let (Value::Text(k_str), Value::Text(v_str)) = (k, v) {
            if k_str == key {
                return Some(v_str.as_str());
            }
        }
    }
    None
}

fn find_uint_in_map(entries: &[(Value, Value)], key: &str) -> Option<u64> {
    for (k, v) in entries {
        if let Value::Text(k_str) = k {
            if k_str == key {
                if let Value::Integer(n) = v {
                    return Some(i128::from(*n) as u64);
                }
            }
        }
    }
    None
}

// ── Page entry parsing ──

struct PageEntry {
    path: String,
    access: String,
    hash: Option<String>,
}

fn parse_page_entry(value: &Value) -> Option<PageEntry> {
    if let Value::Map(entries) = value {
        let path = find_text_in_map(entries, "path")?.to_string();
        let access = find_text_in_map(entries, "access").unwrap_or("T2").to_string();
        let hash = find_bytes_in_map(entries, "hash").map(hex::encode);
        Some(PageEntry { path, access, hash })
    } else {
        None
    }
}

fn find_bytes_in_map<'a>(entries: &'a [(Value, Value)], key: &str) -> Option<&'a [u8]> {
    for (k, v) in entries {
        if let Value::Text(k_str) = k {
            if k_str == key {
                if let Value::Bytes(b) = v {
                    return Some(b.as_slice());
                }
            }
        }
    }
    None
}

// ── Content extraction ──

fn extract_page_content(page: &Value) -> JsonValue {
    let mut result = Map::new();

    // Identity (key 2)
    if let Some(identity) = get_map_value(page, &Value::Integer(2.into())) {
        if let Some(path) = get_text_field(Some(identity), "path") {
            result.insert("path".into(), json!(path));
        }
        if let Some(lang) = get_text_field(Some(identity), "lang") {
            result.insert("lang".into(), json!(lang));
        }
    }

    // Metadata (key 3)
    if let Some(metadata) = get_map_value(page, &Value::Integer(3.into())) {
        if let Some(title) = get_text_field(Some(metadata), "title") {
            result.insert("title".into(), json!(title));
        }
        if let Some(desc) = get_text_field(Some(metadata), "description") {
            result.insert("description".into(), json!(desc));
        }
    }

    // Content blocks (key 4)
    let mut blocks = Vec::new();
    if let Some(content) = get_array_value(page, &Value::Integer(4.into())) {
        for block in content {
            blocks.push(cbor_block_to_json(block));
        }
    }
    result.insert("blocks".into(), json!(blocks));

    // Structured data (key 6)
    if let Some(sd) = get_map_value(page, &Value::Integer(6.into())) {
        result.insert("structured_data".into(), cbor_to_json(sd));
    }

    JsonValue::Object(result)
}

fn cbor_block_to_json(block: &Value) -> JsonValue {
    if let Value::Map(entries) = block {
        let block_type = find_text_in_map(entries, "t").unwrap_or("unknown");
        let mut obj = Map::new();
        obj.insert("type".into(), json!(block_type));

        match block_type {
            "h" => {
                if let Some(level) = find_uint_in_map(entries, "l") {
                    obj.insert("level".into(), json!(level));
                }
                if let Some(text) = find_text_in_map(entries, "v") {
                    obj.insert("text".into(), json!(text));
                }
            }
            "p" => {
                if let Some(text) = find_text_in_map(entries, "v") {
                    obj.insert("text".into(), json!(text));
                }
            }
            "ul" | "ol" => {
                for (k, v) in entries {
                    if let Value::Text(key) = k {
                        if key == "v" {
                            if let Value::Array(items) = v {
                                let strs: Vec<&str> = items
                                    .iter()
                                    .filter_map(|i| if let Value::Text(s) = i { Some(s.as_str()) } else { None })
                                    .collect();
                                obj.insert("items".into(), json!(strs));
                            }
                        }
                    }
                }
            }
            "q" => {
                if let Some(text) = find_text_in_map(entries, "v") {
                    obj.insert("text".into(), json!(text));
                }
                if let Some(attr) = find_text_in_map(entries, "attr") {
                    obj.insert("attribution".into(), json!(attr));
                }
            }
            "code" => {
                if let Some(text) = find_text_in_map(entries, "v") {
                    obj.insert("text".into(), json!(text));
                }
                if let Some(lang) = find_text_in_map(entries, "lang") {
                    obj.insert("language".into(), json!(lang));
                }
            }
            "table" => {
                for (k, v) in entries {
                    if let Value::Text(key) = k {
                        if key == "headers" {
                            if let Value::Array(arr) = v {
                                let strs: Vec<&str> = arr.iter()
                                    .filter_map(|i| if let Value::Text(s) = i { Some(s.as_str()) } else { None })
                                    .collect();
                                obj.insert("headers".into(), json!(strs));
                            }
                        }
                        if key == "rows" {
                            if let Value::Array(rows) = v {
                                let json_rows: Vec<Vec<&str>> = rows.iter().map(|row| {
                                    if let Value::Array(cells) = row {
                                        cells.iter()
                                            .filter_map(|c| if let Value::Text(s) = c { Some(s.as_str()) } else { None })
                                            .collect()
                                    } else {
                                        vec![]
                                    }
                                }).collect();
                                obj.insert("rows".into(), json!(json_rows));
                            }
                        }
                    }
                }
            }
            "cta" => {
                if let Some(text) = find_text_in_map(entries, "v") {
                    obj.insert("text".into(), json!(text));
                }
                if let Some(href) = find_text_in_map(entries, "href") {
                    obj.insert("href".into(), json!(href));
                }
            }
            "img" | "image" => {
                if let Some(alt) = find_text_in_map(entries, "alt") {
                    obj.insert("alt".into(), json!(alt));
                }
                if let Some(src) = find_text_in_map(entries, "src") {
                    obj.insert("src".into(), json!(src));
                }
            }
            "sep" => {}
            "dl" => {
                for (k, v) in entries {
                    if let Value::Text(key) = k {
                        if key == "v" {
                            if let Value::Array(items) = v {
                                let terms: Vec<JsonValue> = items.iter().map(|item| {
                                    if let Value::Map(pairs) = item {
                                        let mut entry = Map::new();
                                        for (ik, iv) in pairs {
                                            if let Value::Text(ik_str) = ik {
                                                if let Value::Text(iv_str) = iv {
                                                    entry.insert(ik_str.to_string(), json!(iv_str));
                                                }
                                            }
                                        }
                                        JsonValue::Object(entry)
                                    } else {
                                        json!(null)
                                    }
                                }).collect();
                                obj.insert("items".into(), json!(terms));
                            }
                        }
                    }
                }
            }
            "note" => {
                if let Some(text) = find_text_in_map(entries, "v") {
                    obj.insert("text".into(), json!(text));
                }
                if let Some(level) = find_text_in_map(entries, "level") {
                    obj.insert("level".into(), json!(level));
                }
            }
            "embed" => {
                if let Some(src) = find_text_in_map(entries, "src") {
                    obj.insert("src".into(), json!(src));
                }
                if let Some(desc) = find_text_in_map(entries, "description") {
                    obj.insert("description".into(), json!(desc));
                }
            }
            // Forward compatibility: preserve unknown blocks
            _ => {
                obj.insert("_raw".into(), cbor_to_json(block));
            }
        }

        JsonValue::Object(obj)
    } else {
        json!(null)
    }
}

fn cbor_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Integer(n) => json!(i128::from(*n)),
        Value::Text(s) => json!(s),
        Value::Bool(b) => json!(b),
        Value::Null => json!(null),
        Value::Float(f) => json!(f),
        Value::Bytes(b) => json!(hex::encode(b)),
        Value::Array(arr) => json!(arr.iter().map(cbor_to_json).collect::<Vec<_>>()),
        Value::Map(entries) => {
            let mut map = Map::new();
            for (k, v) in entries {
                let key = match k {
                    Value::Text(s) => s.clone(),
                    Value::Integer(n) => format!("{}", i128::from(*n)),
                    _ => format!("{:?}", k),
                };
                map.insert(key, cbor_to_json(v));
            }
            JsonValue::Object(map)
        }
        Value::Tag(tag, inner) => {
            if *tag == 1 {
                // Epoch timestamp
                cbor_to_json(inner)
            } else {
                json!({"_tag": tag, "_value": cbor_to_json(inner)})
            }
        }
        _ => json!(null),
    }
}

// ── Path encoding (§6.1) ──

fn path_to_filename(path: &str) -> String {
    if path == "/" {
        return "_index.cbor".to_string();
    }
    // Step 1: escape literal underscores
    let escaped = path.replace('_', "%5F");
    // Step 2: remove leading slash
    let without_slash = escaped.trim_start_matches('/');
    // Step 3: replace remaining slashes with underscores
    let filename = without_slash.replace('/', "_");
    // Step 5: append .cbor
    format!("{}.cbor", filename)
}

// ── Text output ──

fn print_block_text(block: &JsonValue) {
    let btype = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match btype {
        "h" => {
            let level = block.get("level").and_then(|v| v.as_u64()).unwrap_or(1);
            let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let prefix = "#".repeat(level as usize);
            println!("\n{} {}", prefix, text);
        }
        "p" => {
            let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
            println!("{}", text);
        }
        "ul" => {
            if let Some(items) = block.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    println!("  • {}", item.as_str().unwrap_or(""));
                }
            }
        }
        "ol" => {
            if let Some(items) = block.get("items").and_then(|v| v.as_array()) {
                for (i, item) in items.iter().enumerate() {
                    println!("  {}. {}", i + 1, item.as_str().unwrap_or(""));
                }
            }
        }
        "q" => {
            let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let attr = block.get("attribution").and_then(|v| v.as_str()).unwrap_or("");
            println!("  > {}", text);
            if !attr.is_empty() {
                println!("    — {}", attr);
            }
        }
        "code" => {
            let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let lang = block.get("language").and_then(|v| v.as_str()).unwrap_or("");
            println!("```{}", lang);
            println!("{}", text);
            println!("```");
        }
        "table" => {
            if let Some(headers) = block.get("headers").and_then(|v| v.as_array()) {
                let hdr: Vec<&str> = headers.iter().filter_map(|h| h.as_str()).collect();
                println!("| {} |", hdr.join(" | "));
                println!("|{}|", hdr.iter().map(|h| "-".repeat(h.len() + 2)).collect::<Vec<_>>().join("|"));
            }
            if let Some(rows) = block.get("rows").and_then(|v| v.as_array()) {
                for row in rows {
                    if let Some(cells) = row.as_array() {
                        let vals: Vec<&str> = cells.iter().filter_map(|c| c.as_str()).collect();
                        println!("| {} |", vals.join(" | "));
                    }
                }
            }
        }
        "cta" => {
            let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let href = block.get("href").and_then(|v| v.as_str()).unwrap_or("");
            println!("[{}] → {}", text, href);
        }
        "sep" => println!("---"),
        "dl" => {
            if let Some(items) = block.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    let term = item.get("term").and_then(|v| v.as_str()).unwrap_or("");
                    let def = item.get("def").and_then(|v| v.as_str()).unwrap_or("");
                    println!("  {} — {}", term, def);
                }
            }
        }
        "note" => {
            let level = block.get("level").and_then(|v| v.as_str()).unwrap_or("info");
            let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let prefix = match level {
                "important" => "!!",
                "warn" => "⚠",
                _ => "ℹ",
            };
            println!("[{}] {}", prefix, text);
        }
        "embed" => {
            let src = block.get("src").and_then(|v| v.as_str()).unwrap_or("");
            let desc = block.get("description").and_then(|v| v.as_str()).unwrap_or("");
            if desc.is_empty() {
                println!("[embed] {}", src);
            } else {
                println!("[embed] {} — {}", src, desc);
            }
        }
        _ => {}
    }
}

fn block_matches_content(block: &JsonValue, query_lower: &str) -> bool {
    if let Some(block_type) = block.get("type").and_then(|v| v.as_str()) {
        if block_type.to_lowercase().contains(query_lower) {
            return true;
        }
    }
    // Search text-bearing fields without full JSON serialization
    for field in &["text", "alt", "attribution", "href", "src", "description"] {
        if let Some(v) = block.get(field).and_then(|v| v.as_str()) {
            if v.to_lowercase().contains(query_lower) {
                return true;
            }
        }
    }
    // Search list items
    if let Some(items) = block.get("items").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(s) = item.as_str() {
                if s.to_lowercase().contains(query_lower) {
                    return true;
                }
            }
            if let Some(term) = item.get("term").and_then(|v| v.as_str()) {
                if term.to_lowercase().contains(query_lower) { return true; }
            }
            if let Some(def) = item.get("def").and_then(|v| v.as_str()) {
                if def.to_lowercase().contains(query_lower) { return true; }
            }
        }
    }
    // Search table headers and rows
    if let Some(headers) = block.get("headers").and_then(|v| v.as_array()) {
        for h in headers {
            if h.as_str().is_some_and(|s| s.to_lowercase().contains(query_lower)) {
                return true;
            }
        }
    }
    if let Some(rows) = block.get("rows").and_then(|v| v.as_array()) {
        for row in rows {
            if let Some(cells) = row.as_array() {
                for cell in cells {
                    if cell.as_str().is_some_and(|s| s.to_lowercase().contains(query_lower)) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// ── Search command ──

async fn cmd_search(
    client: &Client,
    base_url: &str,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base = base_url.trim_end_matches('/');
    let query_lower = query.to_lowercase();

    let manifest_url = format!("{}{}", base, WELL_KNOWN);
    eprintln!("Fetching manifest: {}", manifest_url);
    let manifest_bytes = fetch_cbor(client, &manifest_url, MAX_MANIFEST_SIZE).await?;
    let manifest = parse_cbor_document(&manifest_bytes)?;

    let site = get_map_value(&manifest, &Value::Integer(2.into()));
    let domain = get_text_field(site, "domain").unwrap_or("unknown");

    let doc_type = get_map_value(&manifest, &Value::Integer(0.into()));
    let is_unified = matches!(doc_type, Some(Value::Text(s)) if s == "cbor-web");
    let meta_key: u64 = if is_unified { 6 } else { 5 };
    let pages_key: u64 = if is_unified { 5 } else { 3 };

    let meta = get_map_value(&manifest, &Value::Integer(meta_key.into()));
    let bundle_available = get_bool_field(meta, "bundle_available").unwrap_or(false);

    let mut page_contents: BTreeMap<String, JsonValue> = BTreeMap::new();

    if bundle_available {
        let bundle_url = format!("{}{}", base, WELL_KNOWN_BUNDLE);
        if let Ok(bundle_bytes) = fetch_cbor(client, &bundle_url, MAX_BUNDLE_SIZE).await {
            let bundle = parse_cbor_document(&bundle_bytes)?;
            if let Some(Value::Map(pages_map)) = get_map_value_owned(&bundle, &Value::Integer(3.into())) {
                for (key, value) in pages_map {
                    if let Value::Text(path) = key {
                        page_contents.insert(path, extract_page_content(&value));
                    }
                }
            }
        }
    }

    if page_contents.is_empty() {
        let pages_arr = get_array_value(&manifest, &Value::Integer(pages_key.into()));
        if let Some(arr) = pages_arr {
            for page_val in arr {
                if let Value::Map(entries) = page_val {
                    let path = find_text_in_map(entries, "path").unwrap_or("/").to_string();
                    let access = find_text_in_map(entries, "access").unwrap_or("T2").to_string();
                    if access == "T0" || access == "token" { continue; }
                    let filename = path_to_filename(&path);
                    let page_url = format!("{}{}{}", base, WELL_KNOWN_PAGES, filename);
                    if let Ok(page_bytes) = fetch_cbor(client, &page_url, MAX_PAGE_SIZE).await {
                        let page = parse_cbor_document(&page_bytes)?;
                        page_contents.insert(path, extract_page_content(&page));
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    println!("Search: \"{}\" on {}", query, domain);
    println!("{}", "─".repeat(60));

    let mut match_count = 0;
    for (path, content) in &page_contents {
        let mut page_matches = Vec::new();
        if let Some(blocks) = content.get("blocks").and_then(|b| b.as_array()) {
            for block in blocks {
                if block_matches_content(block, &query_lower) {
                    page_matches.push(block.clone());
                }
            }
        }
        if content.get("title").and_then(|t| t.as_str()).unwrap_or("").to_lowercase().contains(&query_lower) {
            page_matches.push(json!({"type": "title", "text": content.get("title")}));
        }
        if !page_matches.is_empty() {
            match_count += page_matches.len();
            println!("\n  {} ({} matches)", path, page_matches.len());
            for m in &page_matches {
                if let Some(text) = m.get("text").and_then(|t| t.as_str()) {
                    let preview = truncate(text, 80);
                    println!("    → {}", preview);
                }
            }
        }
    }

    println!("\n{} total match(es) across {} pages", match_count, page_contents.len());
    Ok(())
}

// ── Doléance command ──

async fn cmd_doleance(
    client: &Client,
    base_url: &str,
    feedback_json: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let feedback: serde_json::Value = serde_json::from_str(feedback_json)
        .map_err(|e| format!("Invalid JSON feedback: {}", e))?;

    let base = base_url.trim_end_matches('/');
    let domain = base.trim_start_matches("https://").trim_start_matches("http://");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let page_path = feedback.get("page_path")
        .and_then(|v| v.as_str())
        .unwrap_or("/");

    let signals: Vec<Value> = feedback.get("signals")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter().map(|s| {
                let signal = s.get("signal").and_then(|v| v.as_str()).unwrap_or("missing_data");
                let details = s.get("details").and_then(|v| v.as_str()).unwrap_or("");
                let block_type = s.get("block_type").and_then(|v| v.as_str()).unwrap_or("");
                cbor_canonical_map(vec![
                    (Value::Text("signal".into()), Value::Text(signal.into())),
                    (Value::Text("details".into()), Value::Text(details.into())),
                    (Value::Text("block_type".into()), Value::Text(block_type.into())),
                ])
            }).collect::<Vec<_>>()
        }).unwrap_or_default();

    let doleance = cbor_canonical_map(vec![
        (Value::Integer(0.into()), Value::Text("cbor-web-doleance".into())),
        (Value::Integer(1.into()), Value::Integer(1.into())),
        (Value::Integer(2.into()), cbor_canonical_map(vec![
            (Value::Text("domain".into()), Value::Text(domain.into())),
            (Value::Text("page_path".into()), Value::Text(page_path.into())),
        ])),
        (Value::Integer(3.into()), Value::Array(signals)),
        (Value::Integer(4.into()), cbor_canonical_map(vec![
            (Value::Text("agent".into()), Value::Text("cbor-crawl/0.2.0".into())),
            (Value::Text("timestamp".into()), Value::Tag(1, Box::new(Value::Integer(now.into())))),
        ])),
    ]);

    let tagged = Value::Tag(55799, Box::new(doleance));

    let mut payload = Vec::new();
    ciborium::into_writer(&tagged, &mut payload)?;

    let doleance_url = format!("{}{}/doleance", base, WELL_KNOWN);
    eprintln!("Sending Doléance feedback to: {}", doleance_url);

    let resp = client
        .post(&doleance_url)
        .header("Content-Type", "application/cbor")
        .body(payload)
        .send()
        .await?;

    println!("Status: {}", resp.status());
    if resp.status().is_success() {
        println!("Doléance feedback accepted");
    } else {
        println!("Feedback rejected or endpoint not available");
    }

    Ok(())
}

// ── Diff command ──

async fn cmd_diff(
    client: &Client,
    base_url: &str,
    base_version: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let base = base_url.trim_end_matches('/');

    let manifest_url = format!("{}{}", base, WELL_KNOWN);
    eprintln!("Fetching manifest: {}", manifest_url);
    let manifest_bytes = fetch_cbor(client, &manifest_url, MAX_MANIFEST_SIZE).await?;
    let manifest = parse_cbor_document(&manifest_bytes)?;

    let current_hash = sha256_hex(&manifest_bytes);

    // Check if manifest has diff-manifest (key 9)
    if let Some(diff) = get_map_value(&manifest, &Value::Integer(9.into())) {
        println!("╔══════════════════════════════════════════════════╗");
        println!("║  CBOR-Web Diff Manifest");
        println!("╠══════════════════════════════════════════════════╣");

        if let Some(v) = get_uint_field(Some(diff), "diff_version") {
            println!("║  Diff version:  {}", v);
        }
        if let Some(base_hash) = get_bytes_field(Some(diff), "base_version_hash") {
            println!("║  Base hash:     {}...", hex::encode(&base_hash[..16.min(base_hash.len())]));
        }
        if let Some(stats) = get_map_value(diff, &Value::Text("stats".into())) {
            let added = get_uint_field(Some(stats), "pages_added").unwrap_or(0);
            let modified = get_uint_field(Some(stats), "pages_modified").unwrap_or(0);
            let removed = get_uint_field(Some(stats), "pages_removed").unwrap_or(0);
            let total = get_uint_field(Some(stats), "total_pages_now").unwrap_or(0);
            println!("║  Pages added:    {}", added);
            println!("║  Pages modified: {}", modified);
            println!("║  Pages removed:  {}", removed);
            println!("║  Total pages:    {}", total);
        }

        if let Some(changes) = get_array_value(diff, &Value::Text("changes".into())) {
            println!("╠══════════════════════════════════════════════════╣");
            println!("║  Changes:");
            for change in changes {
                if let Value::Map(entries) = change {
                    let path = find_text_in_map(entries, "path").unwrap_or("?");
                    let action = find_text_in_map(entries, "action").unwrap_or("?");
                    let icon = match action {
                        "added" => "+",
                        "modified" => "~",
                        "removed" => "-",
                        _ => "?",
                    };
                    println!("║  {} {} [{}]", icon, path, action);
                }
            }
        }
        println!("╚══════════════════════════════════════════════════╝");
        return Ok(());
    }

    // Try fetching the diff endpoint if base_version was provided
    if let Some(ref bv) = base_version {
        let diff_url = format!("{}{}/diff?base={}", base, WELL_KNOWN, bv);
        eprintln!("Fetching diff endpoint: {}", diff_url);
        match fetch_cbor(client, &diff_url, MAX_MANIFEST_SIZE).await {
            Ok(diff_bytes) => {
                let diff_doc = parse_cbor_document(&diff_bytes)?;
                println!("Current manifest hash: {}...", &current_hash[..16.min(current_hash.len())]);
                println!("Diff document received ({} bytes)", diff_bytes.len());
                if let Some(Value::Text(dt)) = get_map_value_owned(&diff_doc, &Value::Integer(0.into())) {
                    println!("Document type: {}", dt);
                }
            }
            Err(e) => {
                println!("No diff endpoint available: {}", e);
                println!("Current manifest hash: {}", current_hash);
            }
        }
    } else {
        println!("No diff-manifest found in current manifest.");
        println!("Current manifest hash: {}", current_hash);
        println!("Tip: use --base-version <hash> to request a diff from a previous version");
    }

    Ok(())
}

// ── Utilities ──

fn cbor_canonical_map(entries: Vec<(Value, Value)>) -> Value {
    let mut pairs: Vec<(Vec<u8>, Value, Value)> = entries
        .into_iter()
        .map(|(k, v)| {
            let mut buf = Vec::new();
            ciborium::into_writer(&k, &mut buf).expect("failed to encode CBOR map key");
            (buf, k, v)
        })
        .collect();
    pairs.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(&b.0)));
    Value::Map(pairs.into_iter().map(|(_, k, v)| (k, v)).collect())
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect::<String>() + "..."
    }
}
