//! text2cbor — Convert HTML websites to CBOR-Web v2.1
//!
//! Usage:
//!   text2cbor --input ./site --output ./cbor-web --domain example.com
//!   text2cbor --input ./site --output ./cbor-web --domain example.com --bundle

use ciborium::Value;
use clap::Parser;
use scraper::{Html, Selector, ElementRef};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ============================================================
// CLI Arguments
// ============================================================

#[derive(Parser, Debug)]
#[command(name = "text2cbor", version = "0.1.0")]
#[command(about = "Convert HTML to CBOR-Web v2.1")]
struct Args {
    /// Input directory containing HTML files
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for CBOR-Web files
    #[arg(short, long)]
    output: PathBuf,

    /// Site domain (e.g., example.com)
    #[arg(short, long)]
    domain: String,

    /// Site name
    #[arg(long, default_value = "")]
    name: String,

    /// Default language
    #[arg(long, default_value = "en")]
    lang: String,

    /// Generate bundle file
    #[arg(long, default_value_t = false)]
    bundle: bool,

    /// Pages to mark as token-gated (comma-separated paths, e.g. "/products,/api")
    #[arg(long, default_value = "")]
    token_pages: String,
}

// ============================================================
// Canonical CBOR helpers (RFC 8949 §4.2.1)
// ============================================================

fn encode(v: &Value) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::into_writer(v, &mut buf).unwrap();
    buf
}

fn cmap(entries: Vec<(Value, Value)>) -> Value {
    let mut pairs: Vec<(Vec<u8>, Value, Value)> = entries
        .into_iter()
        .map(|(k, v)| { let e = encode(&k); (e, k, v) })
        .collect();
    pairs.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(&b.0)));
    Value::Map(pairs.into_iter().map(|(_, k, v)| (k, v)).collect())
}

fn t(s: &str) -> Value { Value::Text(s.to_string()) }
fn ii(n: i64) -> Value { Value::Integer(ciborium::value::Integer::from(n)) }
fn u(n: u64) -> Value { Value::Integer(ciborium::value::Integer::from(n)) }
fn b(v: bool) -> Value { Value::Bool(v) }
fn epoch(ts: u64) -> Value { Value::Tag(1, Box::new(u(ts))) }
fn sd(inner: Value) -> Value { Value::Tag(55799, Box::new(inner)) }
fn arr(items: Vec<Value>) -> Value { Value::Array(items) }

fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}

// ============================================================
// Path encoding (CBOR-Web v2.1 §6.1 with M-01 fix)
// ============================================================

fn encode_path_to_filename(path: &str) -> String {
    if path == "/" {
        return "_index.cbor".to_string();
    }
    // Step 1: percent-encode literal underscores
    let escaped = path.replace('_', "%5F");
    // Step 2: remove leading slash
    let no_leading = escaped.trim_start_matches('/');
    // Step 3: replace remaining slashes with underscores
    let result = no_leading.replace('/', "_");
    format!("{}.cbor", result)
}

// ============================================================
// HTML Parser → CBOR-Web content blocks
// ============================================================

struct PageContent {
    title: String,
    description: String,
    lang: String,
    blocks: Vec<Value>,
    internal_links: Vec<(String, String)>,
    external_links: Vec<(String, String)>,
}

fn extract_text(el: &ElementRef) -> String {
    el.text().collect::<Vec<_>>().join("").trim().to_string()
}

fn parse_html(html_content: &str, default_lang: &str) -> PageContent {
    let doc = Html::parse_document(html_content);
    let mut blocks = Vec::new();
    let mut title = String::new();
    let mut description = String::new();
    let lang = default_lang.to_string();
    let mut internal_links = Vec::new();
    let mut external_links = Vec::new();

    // Extract <title>
    if let Some(el) = doc.select(&sel("title")).next() {
        title = extract_text(&el);
    }

    // Extract meta description
    if let Some(el) = doc.select(&sel("meta[name=description]")).next() {
        if let Some(content) = el.value().attr("content") {
            description = content.to_string();
        }
    }

    // Extract content from <main> or <body>
    let container_sel = if doc.select(&sel("main")).next().is_some() {
        "main"
    } else if doc.select(&sel("article")).next().is_some() {
        "article"
    } else {
        "body"
    };

    if let Some(container) = doc.select(&sel(container_sel)).next() {
        for child in container.children() {
            if let Some(el) = ElementRef::wrap(child) {
                let tag = el.value().name();
                match tag {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let level: u64 = tag[1..].parse().unwrap_or(1);
                        let text = extract_text(&el);
                        if !text.is_empty() {
                            if title.is_empty() && level == 1 {
                                title = text.clone();
                            }
                            blocks.push(cmap(vec![
                                (t("t"), t("h")),
                                (t("l"), u(level)),
                                (t("v"), t(&text)),
                            ]));
                        }
                    }
                    "p" => {
                        let text = extract_text(&el);
                        if !text.is_empty() {
                            blocks.push(cmap(vec![
                                (t("t"), t("p")),
                                (t("v"), t(&text)),
                            ]));
                        }
                        // Extract links from paragraphs
                        for a in el.select(&sel("a[href]")) {
                            let href = a.value().attr("href").unwrap_or("");
                            let link_text = extract_text(&a);
                            if href.starts_with("http") {
                                external_links.push((href.to_string(), link_text));
                            } else if href.starts_with('/') {
                                internal_links.push((href.to_string(), link_text));
                            }
                        }
                    }
                    "ul" => {
                        let items: Vec<Value> = el.select(&sel("li"))
                            .map(|li| t(&extract_text(&li)))
                            .filter(|v| if let Value::Text(s) = v { !s.is_empty() } else { false })
                            .collect();
                        if !items.is_empty() {
                            blocks.push(cmap(vec![
                                (t("t"), t("ul")),
                                (t("v"), arr(items)),
                            ]));
                        }
                    }
                    "ol" => {
                        let items: Vec<Value> = el.select(&sel("li"))
                            .map(|li| t(&extract_text(&li)))
                            .filter(|v| if let Value::Text(s) = v { !s.is_empty() } else { false })
                            .collect();
                        if !items.is_empty() {
                            blocks.push(cmap(vec![
                                (t("t"), t("ol")),
                                (t("v"), arr(items)),
                            ]));
                        }
                    }
                    "blockquote" => {
                        let text = extract_text(&el);
                        if !text.is_empty() {
                            blocks.push(cmap(vec![
                                (t("t"), t("q")),
                                (t("v"), t(&text)),
                            ]));
                        }
                    }
                    "pre" => {
                        let code_text = if let Some(code_el) = el.select(&sel("code")).next() {
                            extract_text(&code_el)
                        } else {
                            extract_text(&el)
                        };
                        if !code_text.is_empty() {
                            let mut entries = vec![
                                (t("t"), t("code")),
                                (t("v"), t(&code_text)),
                            ];
                            // Detect language from class
                            if let Some(code_el) = el.select(&sel("code")).next() {
                                if let Some(class) = code_el.value().attr("class") {
                                    if let Some(lang) = class.strip_prefix("language-") {
                                        entries.push((t("lang"), t(lang)));
                                    }
                                }
                            }
                            blocks.push(cmap(entries));
                        }
                    }
                    "table" => {
                        let headers: Vec<Value> = el.select(&sel("th"))
                            .map(|th| t(&extract_text(&th)))
                            .collect();
                        let rows: Vec<Value> = el.select(&sel("tbody tr, tr"))
                            .filter(|tr| tr.select(&sel("td")).next().is_some())
                            .map(|tr| {
                                arr(tr.select(&sel("td"))
                                    .map(|td| t(&extract_text(&td)))
                                    .collect())
                            })
                            .collect();
                        if !headers.is_empty() || !rows.is_empty() {
                            blocks.push(cmap(vec![
                                (t("t"), t("table")),
                                (t("headers"), arr(if headers.is_empty() { vec![] } else { headers })),
                                (t("rows"), arr(rows)),
                            ]));
                        }
                    }
                    "img" => {
                        let alt = el.value().attr("alt").unwrap_or("Image").to_string();
                        let src = el.value().attr("src").unwrap_or("").to_string();
                        if !src.is_empty() {
                            blocks.push(cmap(vec![
                                (t("t"), t("img")),
                                (t("alt"), t(&alt)),
                                (t("src"), t(&src)),
                            ]));
                        }
                    }
                    "hr" => {
                        blocks.push(cmap(vec![(t("t"), t("sep"))]));
                    }
                    _ => {}
                }
            }
        }
    }

    PageContent { title, description, lang, blocks, internal_links, external_links }
}

fn sel(s: &str) -> Selector {
    Selector::parse(s).unwrap()
}

// ============================================================
// CBOR-Web document builders
// ============================================================

fn build_page_document(
    path: &str,
    domain: &str,
    content: &PageContent,
) -> Value {
    let canonical = format!("https://{}{}", domain, path);

    let mut identity_entries = vec![
        (t("path"), t(path)),
        (t("canonical"), t(&canonical)),
        (t("lang"), t(&content.lang)),
    ];

    let mut meta_entries = vec![
        (t("title"), t(&content.title)),
    ];
    if !content.description.is_empty() {
        meta_entries.push((t("description"), t(&content.description)));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    meta_entries.push((t("updated"), epoch(now)));

    let mut page_entries = vec![
        (ii(0), t("cbor-web-page")),
        (ii(1), u(2)),
        (ii(2), cmap(identity_entries)),
        (ii(3), cmap(meta_entries)),
        (ii(4), arr(content.blocks.clone())),
    ];

    // Add links if present
    let mut link_entries = Vec::new();
    if !content.internal_links.is_empty() {
        let links: Vec<Value> = content.internal_links.iter()
            .map(|(href, text)| cmap(vec![(t("path"), t(href)), (t("text"), t(text))]))
            .collect();
        link_entries.push((t("internal"), arr(links)));
    }
    if !content.external_links.is_empty() {
        let links: Vec<Value> = content.external_links.iter()
            .map(|(url, text)| cmap(vec![(t("text"), t(text)), (t("url"), t(url))]))
            .collect();
        link_entries.push((t("external"), arr(links)));
    }
    if !link_entries.is_empty() {
        page_entries.push((ii(5), cmap(link_entries)));
    }

    sd(cmap(page_entries))
}

struct PageInfo {
    path: String,
    title: String,
    lang: String,
    access: String,
    size: usize,
    hash: Vec<u8>,
    cbor_bytes: Vec<u8>,
}

fn build_manifest(
    domain: &str,
    site_name: &str,
    lang: &str,
    pages: &[PageInfo],
    bundle_available: bool,
) -> Value {
    let site_meta = cmap(vec![
        (t("domain"), t(domain)),
        (t("name"), t(if site_name.is_empty() { domain } else { site_name })),
        (t("languages"), arr(vec![t(lang)])),
        (t("default_language"), t(lang)),
    ]);

    let page_entries: Vec<Value> = pages.iter().map(|p| {
        cmap(vec![
            (t("path"), t(&p.path)),
            (t("title"), t(&p.title)),
            (t("lang"), t(&p.lang)),
            (t("access"), t(&p.access)),
            (t("hash"), Value::Bytes(p.hash.clone())),
            (t("size"), u(p.size as u64)),
        ])
    }).collect();

    let total_size: usize = pages.iter().map(|p| p.size).sum();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let meta = cmap(vec![
        (t("generated_at"), epoch(now)),
        (t("total_pages"), u(pages.len() as u64)),
        (t("total_size"), u(total_size as u64)),
        (t("bundle_available"), b(bundle_available)),
        (t("generator"), t("text2cbor/0.1.0")),
    ]);

    let mut manifest_entries = vec![
        (ii(0), t("cbor-web-manifest")),
        (ii(1), u(2)),
        (ii(2), site_meta),
        (ii(3), arr(page_entries)),
        (ii(5), meta),
    ];

    sd(cmap(manifest_entries))
}

fn build_bundle(manifest: &Value, pages: &[PageInfo]) -> Value {
    // Bundle pages map: path -> page content (without self-described tag)
    let page_map: Vec<(Value, Value)> = pages.iter().map(|p| {
        // Decode the page and strip the self-described tag
        let decoded: Value = ciborium::from_reader(&p.cbor_bytes[..]).unwrap();
        let inner = match decoded {
            Value::Tag(55799, boxed) => *boxed,
            other => other,
        };
        (t(&p.path), inner)
    }).collect();

    // Extract manifest inner (strip self-described tag)
    let manifest_decoded: Value = ciborium::from_reader(&encode(manifest)[..]).unwrap();
    let manifest_inner = match manifest_decoded {
        Value::Tag(55799, boxed) => *boxed,
        other => other,
    };

    sd(cmap(vec![
        (ii(0), t("cbor-web-bundle")),
        (ii(1), u(2)),
        (ii(2), manifest_inner),
        (ii(3), cmap(page_map)),
    ]))
}

// ============================================================
// Main
// ============================================================

fn main() {
    let args = Args::parse();

    println!("text2cbor v0.1.0 — CBOR-Web v2.1 Publisher");
    println!("Input:  {}", args.input.display());
    println!("Output: {}", args.output.display());
    println!("Domain: {}", args.domain);
    println!();

    // Parse token-gated paths
    let token_paths: Vec<String> = args.token_pages
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    // Create output directories
    let pages_dir = args.output.join("pages");
    std::fs::create_dir_all(&pages_dir).expect("Failed to create output/pages directory");

    // Find and process HTML files
    let mut page_infos: Vec<PageInfo> = Vec::new();

    for entry in walkdir::WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map(|ext| ext == "html" || ext == "htm")
                .unwrap_or(false)
        })
    {
        let rel_path = entry.path().strip_prefix(&args.input).unwrap();

        // Convert file path to URL path
        let url_path = if rel_path.file_stem().unwrap() == "index" {
            let parent = rel_path.parent().unwrap();
            if parent.as_os_str().is_empty() {
                "/".to_string()
            } else {
                format!("/{}", parent.display()).replace('\\', "/")
            }
        } else {
            let without_ext = rel_path.with_extension("");
            format!("/{}", without_ext.display()).replace('\\', "/")
        };

        println!("  Processing: {} → {}", rel_path.display(), url_path);

        // Read and parse HTML
        let html = std::fs::read_to_string(entry.path())
            .expect(&format!("Failed to read {}", entry.path().display()));
        let content = parse_html(&html, &args.lang);

        // Build CBOR-Web page
        let page_doc = build_page_document(&url_path, &args.domain, &content);
        let page_bytes = encode(&page_doc);
        let page_hash = sha256_bytes(&page_bytes);

        // Determine access level
        let access = if token_paths.iter().any(|tp| url_path.starts_with(tp)) {
            "token"
        } else {
            "public"
        };

        // Write page file
        let filename = encode_path_to_filename(&url_path);
        let page_path = pages_dir.join(&filename);
        std::fs::write(&page_path, &page_bytes)
            .expect(&format!("Failed to write {}", page_path.display()));

        let title = if content.title.is_empty() {
            url_path.clone()
        } else {
            content.title.clone()
        };

        println!("    → {} ({} bytes, {} blocks, access={})",
            filename, page_bytes.len(), content.blocks.len(), access);

        page_infos.push(PageInfo {
            path: url_path,
            title,
            lang: content.lang,
            access: access.to_string(),
            size: page_bytes.len(),
            hash: page_hash,
            cbor_bytes: page_bytes,
        });
    }

    if page_infos.is_empty() {
        eprintln!("ERROR: No HTML files found in {}", args.input.display());
        std::process::exit(1);
    }

    // Sort pages by path for deterministic output
    page_infos.sort_by(|a, b| a.path.cmp(&b.path));

    // Build manifest
    let manifest = build_manifest(
        &args.domain,
        &args.name,
        &args.lang,
        &page_infos,
        args.bundle,
    );
    let manifest_bytes = encode(&manifest);
    let manifest_path = args.output.join("manifest.cbor");
    std::fs::write(&manifest_path, &manifest_bytes)
        .expect("Failed to write manifest");

    println!("\n  Manifest: {} bytes, {} pages", manifest_bytes.len(), page_infos.len());

    // Build bundle if requested
    if args.bundle {
        let bundle = build_bundle(&manifest, &page_infos);
        let bundle_bytes = encode(&bundle);
        let bundle_path = args.output.join("bundle.cbor");
        std::fs::write(&bundle_path, &bundle_bytes)
            .expect("Failed to write bundle");
        println!("  Bundle:   {} bytes", bundle_bytes.len());
    }

    // Write a summary JSON
    let summary = serde_json::json!({
        "domain": args.domain,
        "total_pages": page_infos.len(),
        "total_size": page_infos.iter().map(|p| p.size).sum::<usize>(),
        "manifest_size": manifest_bytes.len(),
        "pages": page_infos.iter().map(|p| {
            serde_json::json!({
                "path": p.path,
                "title": p.title,
                "access": p.access,
                "size": p.size,
                "hash": hex::encode(&p.hash),
                "filename": encode_path_to_filename(&p.path),
            })
        }).collect::<Vec<_>>(),
    });

    let summary_path = args.output.join("summary.json");
    std::fs::write(&summary_path, serde_json::to_string_pretty(&summary).unwrap())
        .expect("Failed to write summary");

    println!("\n✅ CBOR-Web v2.1 output generated at {}", args.output.display());
    println!("  manifest.cbor         — serve at /.well-known/cbor-web");
    println!("  pages/*.cbor          — serve at /.well-known/cbor-web/pages/");
    if args.bundle {
        println!("  bundle.cbor           — serve at /.well-known/cbor-web/bundle");
    }
    println!("  summary.json          — human-readable index");
}
