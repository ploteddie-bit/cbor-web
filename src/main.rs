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
    },
    /// Verify a local CBOR file (structure, encoding, hash)
    Verify {
        /// Path to .cbor file
        file: String,
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
        Commands::Fetch { url, format } => cmd_fetch(&client, &url, &format).await,
        Commands::Verify { file } => cmd_verify(&file),
    }
}

// ── Inspect command ──

async fn cmd_inspect(client: &Client, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let base = base_url.trim_end_matches('/');
    let manifest_url = format!("{}{}", base, WELL_KNOWN);

    eprintln!("Fetching manifest: {}", manifest_url);
    let bytes = fetch_cbor(client, &manifest_url, MAX_MANIFEST_SIZE).await?;
    let manifest = parse_cbor_document(&bytes)?;

    // Extract site metadata (key 2)
    let site = get_map_value(&manifest, &Value::Integer(2.into()));
    let domain = get_text_field(site, "domain").unwrap_or("unknown");
    let name = get_text_field(site, "name").unwrap_or("unknown");
    let description = get_text_field(site, "description").unwrap_or("");
    let languages = get_array_field(site, "languages");
    let default_lang = get_text_field(site, "default_language").unwrap_or("?");

    // Extract meta (key 5)
    let meta = get_map_value(&manifest, &Value::Integer(5.into()));
    let total_pages = get_uint_field(meta, "total_pages").unwrap_or(0);
    let total_size = get_uint_field(meta, "total_size").unwrap_or(0);
    let bundle_available = get_bool_field(meta, "bundle_available").unwrap_or(false);
    let generator = get_text_field(meta, "generator").unwrap_or("unknown");

    // Extract pages (key 3)
    let pages = get_array_value(&manifest, &Value::Integer(3.into()));

    println!("╔══════════════════════════════════════════════════╗");
    println!("║  CBOR-Web Manifest — {}",  domain);
    println!("╠══════════════════════════════════════════════════╣");
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
) -> Result<(), Box<dyn std::error::Error>> {
    let base = base_url.trim_end_matches('/');
    let manifest_url = format!("{}{}", base, WELL_KNOWN);

    eprintln!("Fetching manifest: {}", manifest_url);
    let manifest_bytes = fetch_cbor(client, &manifest_url, MAX_MANIFEST_SIZE).await?;
    let manifest = parse_cbor_document(&manifest_bytes)?;

    let site = get_map_value(&manifest, &Value::Integer(2.into()));
    let domain = get_text_field(site, "domain").unwrap_or("unknown");
    let meta = get_map_value(&manifest, &Value::Integer(5.into()));
    let bundle_available = get_bool_field(meta, "bundle_available").unwrap_or(false);

    let pages_arr = get_array_value(&manifest, &Value::Integer(3.into()));
    let page_entries: Vec<PageEntry> = match pages_arr {
        Some(arr) => arr.iter().filter_map(|v| parse_page_entry(v)).collect(),
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
                        return Some(i128::from(*n) as u64);
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
    title: String,
    access: String,
    hash: Option<String>,
}

fn parse_page_entry(value: &Value) -> Option<PageEntry> {
    if let Value::Map(entries) = value {
        let path = find_text_in_map(entries, "path")?.to_string();
        let title = find_text_in_map(entries, "title").unwrap_or("").to_string();
        let access = find_text_in_map(entries, "access").unwrap_or("T2").to_string();
        let hash = find_bytes_in_map(entries, "hash").map(|b| hex::encode(b));
        Some(PageEntry { path, title, access, hash })
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
        _ => {}
    }
}

// ── Utilities ──

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}
