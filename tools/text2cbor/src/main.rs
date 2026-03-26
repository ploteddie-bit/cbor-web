//! text2cbor v1.1.0 — Convert HTML websites to CBOR-Web v3.0 index.cbor
//!
//! Usage:
//!   text2cbor generate --input ./site --output ./out --domain example.com
//!   text2cbor watch --site /srv/mysite --domain example.com --interval 60

use ciborium::Value;
use clap::{Parser, Subcommand};
use scraper::{Html, Selector, ElementRef};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================
// CLI — Subcommands
// ============================================================

#[derive(Parser, Debug)]
#[command(name = "text2cbor", version = "1.1.0")]
#[command(about = "CBOR-Web v3.0 — generate and maintain index.cbor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate index.cbor from HTML files (one-shot)
    Generate(GenerateArgs),
    /// Watch site directory and rebuild incrementally when files change
    Watch(WatchArgs),
}

// ============================================================
// Generate Arguments
// ============================================================

#[derive(Parser, Debug, Clone)]
struct GenerateArgs {
    /// Input directory containing HTML files
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for index.cbor + summary.json
    #[arg(short, long)]
    output: PathBuf,

    /// Site domain (e.g., deltopide.fr)
    #[arg(short, long)]
    domain: String,

    /// Site name
    #[arg(long, default_value = "")]
    name: String,

    /// Site description
    #[arg(long, default_value = "")]
    description: String,

    /// Default language (BCP 47)
    #[arg(long, default_value = "en")]
    default_lang: String,

    /// Supported languages (comma-separated, e.g. "fr,en,es")
    #[arg(long, default_value = "")]
    languages: String,

    /// Contact email
    #[arg(long, default_value = "")]
    contact_email: String,

    /// Contact phone
    #[arg(long, default_value = "")]
    contact_phone: String,

    /// Country ISO code (e.g., FR, ES)
    #[arg(long, default_value = "")]
    country: String,

    /// Region (e.g., Castellón, Ile-de-France)
    #[arg(long, default_value = "")]
    region: String,

    /// Default access tier: T0, T1, or T2
    #[arg(long, default_value = "T2")]
    default_access: String,

    /// Paths requiring T1 access (comma-separated, e.g. "/premium,/data")
    #[arg(long, default_value = "")]
    t1_pages: String,

    /// Paths requiring T0 access (comma-separated)
    #[arg(long, default_value = "")]
    t0_pages: String,

    /// Auth mechanisms (comma-separated, e.g. "erc20,apikey")
    #[arg(long, default_value = "")]
    auth_mechanisms: String,

    /// ERC-20 contract address
    #[arg(long, default_value = "")]
    erc20_contract: String,

    /// Rate limit for T1 agents (requests/hour)
    #[arg(long, default_value_t = 50)]
    rate_limit_t1: u64,

    /// Rate limit for T2 agents (requests/hour)
    #[arg(long, default_value_t = 10)]
    rate_limit_t2: u64,

    /// Default crawl priority (0.0-1.0)
    #[arg(long, default_value_t = 0.5)]
    priority: f64,

    /// Default recrawl freshness (realtime|hourly|daily|weekly|monthly)
    #[arg(long, default_value = "monthly")]
    freshness: String,

    /// Main navigation paths (comma-separated)
    #[arg(long, default_value = "")]
    nav_main: String,

    /// Footer navigation paths (comma-separated)
    #[arg(long, default_value = "")]
    nav_footer: String,
}

// ============================================================
// Watch Arguments
// ============================================================

#[derive(Parser, Debug)]
struct WatchArgs {
    /// Site directory to watch (same as --input for generate)
    #[arg(long)]
    site: PathBuf,

    /// Output directory (index.cbor will be written/updated here)
    #[arg(short, long)]
    output: PathBuf,

    /// Site domain
    #[arg(short, long)]
    domain: String,

    /// CBORW publisher token (optional — free mode without)
    #[arg(long, default_value = "")]
    token: String,

    /// Check interval in minutes
    #[arg(long, default_value_t = 60)]
    interval: u64,

    /// Site name
    #[arg(long, default_value = "")]
    name: String,

    /// Default language
    #[arg(long, default_value = "en")]
    default_lang: String,

    /// Supported languages (comma-separated)
    #[arg(long, default_value = "")]
    languages: String,

    /// Contact email
    #[arg(long, default_value = "")]
    contact_email: String,

    /// Contact phone
    #[arg(long, default_value = "")]
    contact_phone: String,

    /// Country ISO code
    #[arg(long, default_value = "")]
    country: String,

    /// Region
    #[arg(long, default_value = "")]
    region: String,

    /// Description
    #[arg(long, default_value = "")]
    description: String,

    /// Default access tier
    #[arg(long, default_value = "T2")]
    default_access: String,

    /// T1 pages (comma-separated)
    #[arg(long, default_value = "")]
    t1_pages: String,

    /// Default priority
    #[arg(long, default_value_t = 0.5)]
    priority: f64,

    /// Default freshness
    #[arg(long, default_value = "monthly")]
    freshness: String,
}

impl WatchArgs {
    fn to_generate_args(&self) -> GenerateArgs {
        GenerateArgs {
            input: self.site.clone(),
            output: self.output.clone(),
            domain: self.domain.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            default_lang: self.default_lang.clone(),
            languages: self.languages.clone(),
            contact_email: self.contact_email.clone(),
            contact_phone: self.contact_phone.clone(),
            country: self.country.clone(),
            region: self.region.clone(),
            default_access: self.default_access.clone(),
            t1_pages: self.t1_pages.clone(),
            t0_pages: String::new(),
            auth_mechanisms: String::new(),
            erc20_contract: String::new(),
            rate_limit_t1: 50,
            rate_limit_t2: 10,
            priority: self.priority,
            freshness: self.freshness.clone(),
            nav_main: String::new(),
            nav_footer: String::new(),
        }
    }
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
fn epoch(ts: u64) -> Value { Value::Tag(1, Box::new(u(ts))) }
fn sd(inner: Value) -> Value { Value::Tag(55799, Box::new(inner)) }
fn arr(items: Vec<Value>) -> Value { Value::Array(items) }
fn float(f: f64) -> Value { Value::Float(f) }

fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}

fn sel(s: &str) -> Selector {
    Selector::parse(s).unwrap()
}

// ============================================================
// HTML Parser → Content extraction
// ============================================================

struct PageContent {
    title: String,
    description: String,
    lang: String,
    blocks: Vec<Value>,
    internal_links: Vec<(String, String)>,
    external_links: Vec<(String, String)>,
    alternates: HashMap<String, String>,
    structured_data: Option<serde_json::Value>,
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
    let mut alternates = HashMap::new();
    let mut structured_data = None;

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

    // Extract hreflang alternates
    for el in doc.select(&sel("link[rel=alternate][hreflang]")) {
        if let (Some(lang), Some(href)) = (el.value().attr("hreflang"), el.value().attr("href")) {
            if lang != "x-default" {
                alternates.insert(lang.to_string(), href.to_string());
            }
        }
    }

    // Extract JSON-LD structured data (first block only)
    for el in doc.select(&sel("script[type='application/ld+json']")) {
        let json_text = extract_text(&el);
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_text) {
            structured_data = Some(parsed);
            break;
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
        extract_blocks_recursive(&container, &mut blocks, &mut internal_links, &mut external_links);
    }

    if title.is_empty() {
        // Fallback: first h1
        for block in &blocks {
            if let Value::Map(pairs) = block {
                let is_h = pairs.iter().any(|(k, v)| {
                    matches!(k, Value::Text(s) if s == "t") && matches!(v, Value::Text(s) if s == "h")
                });
                if is_h {
                    if let Some((_, Value::Text(text))) = pairs.iter().find(|(k, _)| matches!(k, Value::Text(s) if s == "v")) {
                        title = text.clone();
                        break;
                    }
                }
            }
        }
    }

    PageContent { title, description, lang, blocks, internal_links, external_links, alternates, structured_data }
}

fn extract_blocks_recursive(
    container: &ElementRef,
    blocks: &mut Vec<Value>,
    internal_links: &mut Vec<(String, String)>,
    external_links: &mut Vec<(String, String)>,
) {
    for child in container.children() {
        if let Some(el) = ElementRef::wrap(child) {
            let tag = el.value().name();
            match tag {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level: u64 = tag[1..].parse().unwrap_or(1);
                    let text = extract_text(&el);
                    if !text.is_empty() {
                        blocks.push(cmap(vec![
                            (t("l"), u(level)),
                            (t("t"), t("h")),
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
                            (t("headers"), arr(if headers.is_empty() { vec![] } else { headers })),
                            (t("rows"), arr(rows)),
                            (t("t"), t("table")),
                        ]));
                    }
                }
                "img" => {
                    let alt = el.value().attr("alt").unwrap_or("Image").to_string();
                    let src = el.value().attr("src").unwrap_or("").to_string();
                    if !src.is_empty() {
                        blocks.push(cmap(vec![
                            (t("alt"), t(&alt)),
                            (t("src"), t(&src)),
                            (t("t"), t("img")),
                        ]));
                    }
                }
                "hr" => {
                    blocks.push(cmap(vec![(t("t"), t("sep"))]));
                }
                // Recurse into container elements
                "div" | "section" | "main" | "article" | "aside" | "header" | "footer" | "nav" | "figure" | "figcaption" | "details" | "summary" | "dl" => {
                    // For dl, handle specially
                    if tag == "dl" {
                        let mut defs = Vec::new();
                        let mut current_term = String::new();
                        for dl_child in el.children() {
                            if let Some(dl_el) = ElementRef::wrap(dl_child) {
                                match dl_el.value().name() {
                                    "dt" => { current_term = extract_text(&dl_el); }
                                    "dd" => {
                                        let def = extract_text(&dl_el);
                                        if !current_term.is_empty() && !def.is_empty() {
                                            defs.push(cmap(vec![
                                                (t("def"), t(&def)),
                                                (t("term"), t(&current_term)),
                                            ]));
                                        }
                                        current_term.clear();
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if !defs.is_empty() {
                            blocks.push(cmap(vec![
                                (t("t"), t("dl")),
                                (t("v"), arr(defs)),
                            ]));
                        }
                    } else {
                        extract_blocks_recursive(&el, blocks, internal_links, external_links);
                    }
                }
                "a" => {
                    // CTA detection: standalone links with button-like classes
                    let href = el.value().attr("href").unwrap_or("").to_string();
                    let text = extract_text(&el);
                    let class = el.value().attr("class").unwrap_or("");
                    if !text.is_empty() && !href.is_empty() {
                        if class.contains("btn") || class.contains("cta") || class.contains("button") {
                            blocks.push(cmap(vec![
                                (t("href"), t(&href)),
                                (t("t"), t("cta")),
                                (t("v"), t(&text)),
                            ]));
                        } else if href.starts_with("http") {
                            external_links.push((href, text));
                        } else if href.starts_with('/') {
                            internal_links.push((href, text));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// ============================================================
// JSON-LD → CBOR conversion
// ============================================================

fn json_to_cbor(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                ii(i)
            } else if let Some(f) = n.as_f64() {
                float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => t(s),
        serde_json::Value::Array(a) => arr(a.iter().map(json_to_cbor).collect()),
        serde_json::Value::Object(o) => {
            // Filter out @context (not needed in CBOR)
            let entries: Vec<(Value, Value)> = o.iter()
                .filter(|(k, _)| *k != "@context")
                .map(|(k, v)| (t(k), json_to_cbor(v)))
                .collect();
            cmap(entries)
        }
    }
}

// ============================================================
// Index builders (v3.0)
// ============================================================

fn build_site_metadata(args: &GenerateArgs) -> Value {
    let mut entries = vec![
        (t("domain"), t(&args.domain)),
    ];

    let name = if args.name.is_empty() { &args.domain } else { &args.name };
    entries.push((t("name"), t(name)));

    if !args.description.is_empty() {
        entries.push((t("description"), t(&args.description)));
    }

    // Languages
    let langs: Vec<Value> = if args.languages.is_empty() {
        vec![t(&args.default_lang)]
    } else {
        args.languages.split(',').map(|s| t(s.trim())).collect()
    };
    entries.push((t("languages"), arr(langs)));
    entries.push((t("default_language"), t(&args.default_lang)));

    // Contact
    let mut contact = Vec::new();
    if !args.contact_email.is_empty() {
        contact.push((t("email"), t(&args.contact_email)));
    }
    if !args.contact_phone.is_empty() {
        contact.push((t("phone"), t(&args.contact_phone)));
    }
    if !contact.is_empty() {
        entries.push((t("contact"), cmap(contact)));
    }

    // Geo
    let mut geo = Vec::new();
    if !args.country.is_empty() {
        geo.push((t("country"), t(&args.country)));
    }
    if !args.region.is_empty() {
        geo.push((t("region"), t(&args.region)));
    }
    if !geo.is_empty() {
        entries.push((t("geo"), cmap(geo)));
    }

    cmap(entries)
}

fn build_security(args: &GenerateArgs) -> Value {
    let mut entries = vec![
        (t("default_access"), t(&args.default_access)),
    ];

    // Auth mechanisms
    if !args.auth_mechanisms.is_empty() {
        let mechs: Vec<Value> = args.auth_mechanisms.split(',').map(|s| t(s.trim())).collect();
        let mut auth_entries = vec![
            (t("mechanisms"), arr(mechs)),
        ];
        if !args.erc20_contract.is_empty() {
            auth_entries.push((t("erc20"), cmap(vec![
                (t("chain"), t("ethereum")),
                (t("contract_address"), t(&args.erc20_contract)),
            ])));
        }
        entries.push((t("auth"), cmap(auth_entries)));
    }

    // Rate limits
    entries.push((t("rate_limit"), cmap(vec![
        (t("T1"), u(args.rate_limit_t1)),
        (t("T2"), u(args.rate_limit_t2)),
    ])));

    cmap(entries)
}

fn build_navigation(args: &GenerateArgs, pages: &[PageEntry]) -> Value {
    let mut entries = Vec::new();

    if !args.nav_main.is_empty() {
        let paths: Vec<Value> = args.nav_main.split(',').map(|s| t(s.trim())).collect();
        entries.push((t("main"), arr(paths)));
    } else {
        // Auto-detect: top-level pages as main nav
        let main: Vec<Value> = pages.iter()
            .filter(|p| p.path.matches('/').count() <= 1)
            .map(|p| t(&p.path))
            .collect();
        if !main.is_empty() {
            entries.push((t("main"), arr(main)));
        }
    }

    if !args.nav_footer.is_empty() {
        let paths: Vec<Value> = args.nav_footer.split(',').map(|s| t(s.trim())).collect();
        entries.push((t("footer"), arr(paths)));
    }

    // Auto-detect hierarchy from paths
    let mut hierarchy: HashMap<String, Vec<String>> = HashMap::new();
    for page in pages {
        if page.path.matches('/').count() >= 2 {
            let parent = page.path.rsplitn(2, '/').last().unwrap_or("").to_string();
            if !parent.is_empty() {
                hierarchy.entry(parent).or_default().push(page.path.clone());
            }
        }
    }
    if !hierarchy.is_empty() {
        let h_entries: Vec<(Value, Value)> = hierarchy.into_iter()
            .map(|(parent, children)| {
                (t(&parent), arr(children.into_iter().map(|c| t(&c)).collect()))
            })
            .collect();
        entries.push((t("hierarchy"), cmap(h_entries)));
    }

    cmap(entries)
}

fn build_meta(pages: &[PageEntry], total_size: usize) -> Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    cmap(vec![
        (t("generated_at"), epoch(now)),
        (t("generator"), t("text2cbor/1.0.0")),
        (t("total_pages"), u(pages.len() as u64)),
        (t("total_size"), u(total_size as u64)),
    ])
}

// ============================================================
// Page entry builder
// ============================================================

struct PageEntry {
    path: String,
    title: String,
    description: String,
    lang: String,
    access: String,
    blocks: Vec<Value>,
    hash: Vec<u8>,
    content_size: usize,
    internal_links: Vec<(String, String)>,
    external_links: Vec<(String, String)>,
    alternates: HashMap<String, String>,
    structured_data: Option<serde_json::Value>,
    priority: f64,
    freshness: String,
}

// ============================================================
// Intelligent navigation: _describe + _l
// ============================================================

fn enrich_block_with_describe(block: &Value) -> Value {
    if let Value::Map(pairs) = block {
        let mut new_pairs: Vec<(Value, Value)> = pairs.clone();

        // Extract block type and value
        let block_type = pairs.iter()
            .find(|(k, _)| matches!(k, Value::Text(s) if s == "t"))
            .and_then(|(_, v)| if let Value::Text(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("");

        let block_value = pairs.iter()
            .find(|(k, _)| matches!(k, Value::Text(s) if s == "v"))
            .and_then(|(_, v)| if let Value::Text(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("");

        let block_level = pairs.iter()
            .find(|(k, _)| matches!(k, Value::Text(s) if s == "l"))
            .and_then(|(_, v)| match v {
                Value::Integer(i) => { let n: i128 = (*i).into(); Some(n as u64) }
                _ => None
            })
            .unwrap_or(0);

        // Generate _describe
        let describe = match block_type {
            "h" => {
                let text = if block_value.len() > 50 { &block_value[..50] } else { block_value };
                format!("Heading level {}: {}", block_level, text)
            }
            "p" => {
                if block_value.len() > 100 {
                    let words: Vec<&str> = block_value.split_whitespace().take(30).collect();
                    format!("{}...", words.join(" "))
                } else {
                    String::new() // Short paragraphs don't need _describe
                }
            }
            "table" => {
                let headers = pairs.iter()
                    .find(|(k, _)| matches!(k, Value::Text(s) if s == "headers"))
                    .and_then(|(_, v)| if let Value::Array(a) = v {
                        Some(a.iter().filter_map(|h| if let Value::Text(s) = h { Some(s.as_str()) } else { None }).collect::<Vec<_>>().join(", "))
                    } else { None })
                    .unwrap_or_default();
                let rows = pairs.iter()
                    .find(|(k, _)| matches!(k, Value::Text(s) if s == "rows"))
                    .and_then(|(_, v)| if let Value::Array(a) = v { Some(a.len()) } else { None })
                    .unwrap_or(0);
                format!("Table: {}. {} rows.", headers, rows)
            }
            "ul" | "ol" => {
                let count = pairs.iter()
                    .find(|(k, _)| matches!(k, Value::Text(s) if s == "v"))
                    .and_then(|(_, v)| if let Value::Array(a) = v { Some(a.len()) } else { None })
                    .unwrap_or(0);
                format!("List: {} items.", count)
            }
            "cta" => {
                let href = pairs.iter()
                    .find(|(k, _)| matches!(k, Value::Text(s) if s == "href"))
                    .and_then(|(_, v)| if let Value::Text(s) = v { Some(s.as_str()) } else { None })
                    .unwrap_or("");
                format!("Call to action: {} -> {}", block_value, href)
            }
            "img" => {
                let alt = pairs.iter()
                    .find(|(k, _)| matches!(k, Value::Text(s) if s == "alt"))
                    .and_then(|(_, v)| if let Value::Text(s) = v { Some(s.as_str()) } else { None })
                    .unwrap_or("Image");
                format!("Image: {}", alt)
            }
            "q" => format!("Quote: {}...", if block_value.len() > 40 { &block_value[..40] } else { block_value }),
            "code" => {
                let lang = pairs.iter()
                    .find(|(k, _)| matches!(k, Value::Text(s) if s == "lang"))
                    .and_then(|(_, v)| if let Value::Text(s) = v { Some(s.as_str()) } else { None })
                    .unwrap_or("unknown");
                format!("Code block: {}", lang)
            }
            _ => String::new(),
        };

        // Generate _l (depth level)
        let level: u64 = match block_type {
            "h" if block_level == 1 => 0,        // Identity
            "h" if block_level == 2 => 1,         // Essential
            "table" | "cta" => 1,                  // Essential
            "p" | "ul" | "ol" => 2,                // Detail
            "h" => 2,                              // h3+ = detail
            "q" | "code" | "dl" => 3,              // Complete
            "img" | "sep" | "embed" => 4,          // Enrichment
            _ => 2,
        };

        // Add _describe if non-empty
        if !describe.is_empty() {
            new_pairs.push((t("_describe"), t(&describe)));
        }
        new_pairs.push((t("_l"), u(level)));

        cmap(new_pairs)
    } else {
        block.clone()
    }
}

fn enrich_blocks(blocks: &[Value]) -> Vec<Value> {
    blocks.iter().map(enrich_block_with_describe).collect()
}

fn build_page_entry(page: &PageEntry) -> Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut entries: Vec<(Value, Value)> = vec![
        (t("access"), t(&page.access)),
        (t("content"), arr(enrich_blocks(&page.blocks))),
        (t("hash"), Value::Bytes(page.hash.clone())),
        (t("lang"), t(&page.lang)),
        (t("path"), t(&page.path)),
        (t("title"), t(&page.title)),
        (t("updated"), epoch(now)),
    ];

    if !page.description.is_empty() {
        entries.push((t("description"), t(&page.description)));
    }

    // Visibility options
    if page.priority != 0.5 {
        entries.push((t("priority"), float(page.priority)));
    }
    if page.freshness != "monthly" {
        entries.push((t("freshness"), t(&page.freshness)));
    }

    // Alternates
    if !page.alternates.is_empty() {
        let alt_entries: Vec<(Value, Value)> = page.alternates.iter()
            .map(|(lang, href)| (t(lang), t(href)))
            .collect();
        entries.push((t("alternates"), cmap(alt_entries)));
    }

    // Structured data (JSON-LD → CBOR)
    if let Some(ref sd) = page.structured_data {
        entries.push((t("structured_data"), json_to_cbor(sd)));
    }

    // Links
    let mut link_entries = Vec::new();
    if !page.internal_links.is_empty() {
        let links: Vec<Value> = page.internal_links.iter()
            .map(|(href, text)| cmap(vec![(t("path"), t(href)), (t("text"), t(text))]))
            .collect();
        link_entries.push((t("internal"), arr(links)));
    }
    if !page.external_links.is_empty() {
        let links: Vec<Value> = page.external_links.iter()
            .map(|(url, text)| cmap(vec![(t("text"), t(text)), (t("url"), t(url))]))
            .collect();
        link_entries.push((t("external"), arr(links)));
    }
    if !link_entries.is_empty() {
        entries.push((t("links"), cmap(link_entries)));
    }

    cmap(entries)
}

// ============================================================
// Generate — one-shot build (core logic)
// ============================================================

fn run_generate(args: &GenerateArgs) -> (Vec<u8>, Vec<PageEntry>) {
    let t1_paths: Vec<String> = args.t1_pages.split(',')
        .filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect();
    let t0_paths: Vec<String> = args.t0_pages.split(',')
        .filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect();

    std::fs::create_dir_all(&args.output).expect("Failed to create output directory");

    let mut pages: Vec<PageEntry> = Vec::new();

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
        let url_path = if rel_path.file_stem().unwrap() == "index" {
            let parent = rel_path.parent().unwrap();
            if parent.as_os_str().is_empty() { "/".to_string() }
            else { format!("/{}", parent.display()).replace('\\', "/") }
        } else {
            let without_ext = rel_path.with_extension("");
            format!("/{}", without_ext.display()).replace('\\', "/")
        };

        println!("  Processing: {} → {}", rel_path.display(), url_path);

        let html = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|_| panic!("Failed to read {}", entry.path().display()));
        let content = parse_html(&html, &args.default_lang);

        let access = if t0_paths.iter().any(|p| url_path.starts_with(p)) { "T0" }
            else if t1_paths.iter().any(|p| url_path.starts_with(p)) { "T1" }
            else { &args.default_access };

        let content_cbor = encode(&arr(content.blocks.clone()));
        let hash = sha256_bytes(&content_cbor);
        let title = if content.title.is_empty() { url_path.clone() } else { content.title.clone() };

        println!("    → {} blocks, access={}", content.blocks.len(), access);

        pages.push(PageEntry {
            path: url_path, title, description: content.description,
            lang: content.lang, access: access.to_string(), blocks: content.blocks,
            hash, content_size: content_cbor.len(),
            internal_links: content.internal_links, external_links: content.external_links,
            alternates: content.alternates, structured_data: content.structured_data,
            priority: args.priority, freshness: args.freshness.clone(),
        });
    }

    if pages.is_empty() {
        eprintln!("ERROR: No HTML files found in {}", args.input.display());
        std::process::exit(1);
    }

    pages.sort_by(|a, b| a.path.cmp(&b.path));

    let total_content_size: usize = pages.iter().map(|p| p.content_size).sum();
    let page_values: Vec<Value> = pages.iter().map(build_page_entry).collect();

    let mut index_entries = vec![
        (ii(0), t("cbor-web")),
        (ii(1), u(3)),
        (ii(2), build_site_metadata(args)),
        (ii(3), build_security(args)),
    ];

    let nav = build_navigation(args, &pages);
    if let Value::Map(ref pairs) = nav {
        if !pairs.is_empty() { index_entries.push((ii(4), nav)); }
    }

    index_entries.push((ii(5), arr(page_values)));
    index_entries.push((ii(6), build_meta(&pages, total_content_size)));

    let index_doc = sd(cmap(index_entries));
    let index_bytes = encode(&index_doc);

    // Write index.cbor
    std::fs::write(args.output.join("index.cbor"), &index_bytes)
        .expect("Failed to write index.cbor");

    // Write summary.json
    let summary = serde_json::json!({
        "type": "cbor-web", "version": 3, "domain": args.domain,
        "pages": pages.iter().map(|p| serde_json::json!({
            "path": p.path, "title": p.title, "access": p.access,
            "blocks": p.blocks.len(), "hash": hex::encode(&p.hash),
        })).collect::<Vec<_>>(),
        "stats": {
            "total_pages": pages.len(),
            "index_cbor_bytes": index_bytes.len(),
        }
    });
    std::fs::write(args.output.join("summary.json"),
        serde_json::to_string_pretty(&summary).unwrap())
        .expect("Failed to write summary.json");

    (index_bytes, pages)
}

// ============================================================
// Watch — incremental rebuild daemon
// ============================================================

fn compute_site_hash(site_dir: &std::path::Path) -> Vec<u8> {
    let mut hasher = Sha256::new();
    let mut files: Vec<_> = walkdir::WalkDir::new(site_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension()
            .map(|ext| ext == "html" || ext == "htm")
            .unwrap_or(false))
        .collect();
    files.sort_by(|a, b| a.path().cmp(b.path()));
    for entry in &files {
        // Hash filename + modification time + size
        hasher.update(entry.path().to_string_lossy().as_bytes());
        if let Ok(meta) = entry.metadata() {
            hasher.update(meta.len().to_le_bytes());
            if let Ok(modified) = meta.modified() {
                let secs = modified.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                hasher.update(secs.to_le_bytes());
            }
        }
    }
    hasher.finalize().to_vec()
}

fn verify_token(token: &str) -> bool {
    if token.is_empty() {
        println!("  [FREE MODE] No token — T2 only, no signature");
        return true;
    }
    // For now: accept any non-empty token prefixed with "cbw_"
    // Future: call cbor-web.com/api/verify or on-chain verification
    if token.starts_with("cbw_") {
        println!("  [TOKEN OK] Publisher token verified");
        true
    } else {
        eprintln!("  [TOKEN INVALID] Token must start with cbw_");
        false
    }
}

fn run_watch(args: &WatchArgs) {
    println!("text2cbor v1.1.0 — CBOR-Web v3.0 Watch Mode");
    println!("Site:     {}", args.site.display());
    println!("Output:   {}", args.output.display());
    println!("Domain:   {}", args.domain);
    println!("Interval: {} min", args.interval);
    println!();

    // Verify token
    if !verify_token(&args.token) {
        std::process::exit(1);
    }

    let gen_args = args.to_generate_args();
    let mut last_site_hash: Vec<u8> = Vec::new();

    // Initial build
    println!("[INIT] Building index.cbor...");
    let (bytes, pages) = run_generate(&gen_args);
    last_site_hash = compute_site_hash(&args.site);
    println!("[INIT] Done — {} bytes, {} pages\n", bytes.len(), pages.len());

    // Watch loop
    loop {
        std::thread::sleep(std::time::Duration::from_secs(args.interval * 60));

        // Quick check: has anything changed?
        let current_hash = compute_site_hash(&args.site);
        if current_hash == last_site_hash {
            let now = chrono_now();
            println!("[{}] No changes detected", now);
            continue;
        }

        // Something changed — rebuild
        println!("[REBUILD] Changes detected, rebuilding...");
        let (bytes, pages) = run_generate(&gen_args);
        last_site_hash = current_hash;

        // Count changed pages by comparing hashes
        let now = chrono_now();
        println!("[{}] Rebuilt — {} bytes, {} pages", now, bytes.len(), pages.len());
    }
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    // Simple HH:MM:SS from epoch (UTC)
    let h = (now % 86400) / 3600;
    let m = (now % 3600) / 60;
    let s = now % 60;
    format!("{:02}:{:02}:{:02} UTC", h, m, s)
}

// ============================================================
// Main — dispatch subcommands
// ============================================================

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(args) => {
            println!("text2cbor v1.1.0 — CBOR-Web v3.0 Generate");
            println!("Input:  {}", args.input.display());
            println!("Output: {}", args.output.display());
            println!("Domain: {}", args.domain);
            println!();
            let (bytes, pages) = run_generate(&args);
            println!("\n  index.cbor: {} bytes, {} pages", bytes.len(), pages.len());
            println!("  summary.json: human-readable breakdown");
            println!("\n✅ Serve at https://{}/index.cbor", args.domain);
        }
        Commands::Watch(args) => {
            run_watch(&args);
        }
    }
}
