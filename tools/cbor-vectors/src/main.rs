//! CBOR-Web v2.1 — Test Vector Generator (Rust/ciborium)
//! RFC 8949 §4.2.1 Deterministic Encoding

use ciborium::Value;
use clap::Parser;
use sha2::{Digest, Sha256};

#[derive(Parser)]
#[command(name = "cbor-vectors")]
#[command(about = "CBOR-Web v2.1 Test Vector Generator")]
struct Cli {
    #[arg(long, default_value = "./test-vectors")]
    output: String,
}

/// Encode a single CBOR Value to bytes
fn encode_value(v: &Value) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::into_writer(v, &mut buf).unwrap();
    buf
}

/// Sort map entries by RFC 8949 §4.2.1:
/// 1. Shorter encoded key first
/// 2. Among equal-length, bytewise comparison
fn canonical_map(entries: Vec<(Value, Value)>) -> Value {
    let mut pairs: Vec<(Vec<u8>, Value, Value)> = entries
        .into_iter()
        .map(|(k, v)| {
            let encoded_key = encode_value(&k);
            (encoded_key, k, v)
        })
        .collect();

    pairs.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(&b.0)));

    Value::Map(pairs.into_iter().map(|(_, k, v)| (k, v)).collect())
}

fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}
fn u(n: u64) -> Value {
    Value::Integer(ciborium::value::Integer::from(n))
}
fn ii(n: i64) -> Value {
    Value::Integer(ciborium::value::Integer::from(n))
}
fn b(v: bool) -> Value {
    Value::Bool(v)
}
fn epoch(ts: u64) -> Value {
    Value::Tag(1, Box::new(u(ts)))
}
fn self_described(inner: Value) -> Value {
    Value::Tag(55799, Box::new(inner))
}
fn bstr_bytes(data: &[u8]) -> Value {
    Value::Bytes(data.to_vec())
}

/// Inner manifest map (without self-described tag), reused by TV1 and TV4
fn manifest_map() -> Value {
    let site_meta = canonical_map(vec![
        (t("domain"), t("test.example")),
        (t("name"), t("Test")),
        (t("languages"), Value::Array(vec![t("en")])),
        (t("default_language"), t("en")),
    ]);

    let page_entry = canonical_map(vec![
        (t("path"), t("/")),
        (t("title"), t("Home")),
        (t("lang"), t("en")),
        (t("access"), t("public")),
        (t("size"), u(95)),
    ]);

    let meta = canonical_map(vec![
        (t("generated_at"), epoch(1742515200)),
        (t("total_pages"), u(1)),
        (t("total_size"), u(95)),
        (t("bundle_available"), b(false)),
    ]);

    canonical_map(vec![
        (ii(0), t("cbor-web-manifest")),
        (ii(1), u(2)),
        (ii(2), site_meta),
        (ii(3), Value::Array(vec![page_entry])),
        (ii(5), meta),
    ])
}

fn test_vector_1_manifest() -> Value {
    self_described(manifest_map())
}

/// Inner page map (without self-described tag), reused by TV2 and TV4
fn page_map() -> Value {
    let identity = canonical_map(vec![
        (t("path"), t("/")),
        (t("canonical"), t("https://test.example/")),
        (t("lang"), t("en")),
    ]);

    let metadata = canonical_map(vec![(t("title"), t("Welcome"))]);

    let heading = canonical_map(vec![
        (t("t"), t("h")),
        (t("l"), u(1)),
        (t("v"), t("Welcome")),
    ]);

    let paragraph = canonical_map(vec![(t("t"), t("p")), (t("v"), t("Hello, World!"))]);

    canonical_map(vec![
        (ii(0), t("cbor-web-page")),
        (ii(1), u(2)),
        (ii(2), identity),
        (ii(3), metadata),
        (ii(4), Value::Array(vec![heading, paragraph])),
    ])
}

fn test_vector_2_page() -> Value {
    self_described(page_map())
}

fn test_vector_3_product() -> Value {
    let identity = canonical_map(vec![
        (t("path"), t("/products/lions-mane")),
        (
            t("canonical"),
            t("https://verdetao.com/products/lions-mane"),
        ),
        (t("lang"), t("fr")),
        (
            t("alternates"),
            canonical_map(vec![(t("es"), t("/es/productos/melena-de-leon"))]),
        ),
    ]);

    let metadata = canonical_map(vec![
        (t("title"), t("Lion's Mane")),
        (t("category"), t("products")),
        (t("updated"), epoch(1742428800)),
        (
            t("tags"),
            Value::Array(vec![t("champignon"), t("nootropique")]),
        ),
    ]);

    let h1 = canonical_map(vec![
        (t("t"), t("h")),
        (t("l"), u(1)),
        (t("v"), t("Lion's Mane")),
    ]);
    let p1 = canonical_map(vec![
        (t("t"), t("p")),
        (t("v"), t("Extrait de Hericium erinaceus concentre 10:1.")),
    ]);
    let tbl = canonical_map(vec![
        (t("t"), t("table")),
        (
            t("headers"),
            Value::Array(vec![t("Propriete"), t("Valeur")]),
        ),
        (
            t("rows"),
            Value::Array(vec![
                Value::Array(vec![t("Prix"), t("29.90 EUR")]),
                Value::Array(vec![t("Capsules"), t("90")]),
            ]),
        ),
    ]);
    let cta = canonical_map(vec![
        (t("t"), t("cta")),
        (t("v"), t("Ajouter au panier")),
        (t("href"), t("/cart/add/lions-mane")),
    ]);

    let structured = canonical_map(vec![
        (t("type"), t("Product")),
        (t("name"), t("Lion's Mane")),
        (
            t("offers"),
            canonical_map(vec![
                (t("type"), t("Offer")),
                (t("price"), Value::Float(29.90)),
                (t("priceCurrency"), t("EUR")),
                (t("availability"), t("InStock")),
            ]),
        ),
    ]);

    let page = canonical_map(vec![
        (ii(0), t("cbor-web-page")),
        (ii(1), u(2)),
        (ii(2), identity),
        (ii(3), metadata),
        (ii(4), Value::Array(vec![h1, p1, tbl, cta])),
        (ii(6), structured),
    ]);

    self_described(page)
}

/// TV4 — Bundle: minimal bundle document combining manifest (TV1) and page (TV2)
fn test_vector_4_bundle() -> Value {
    let pages = canonical_map(vec![(t("/"), page_map())]);

    self_described(canonical_map(vec![
        (ii(0), t("cbor-web-bundle")),
        (ii(1), u(2)),
        (ii(2), manifest_map()),
        (ii(3), pages),
    ]))
}

/// TV5 — Navigation: manifest with navigation (key 4) containing main and footer arrays
fn test_vector_5_navigation() -> Value {
    let site_meta = canonical_map(vec![
        (t("domain"), t("test.example")),
        (t("name"), t("Test")),
        (t("languages"), Value::Array(vec![t("en")])),
        (t("default_language"), t("en")),
    ]);

    let page_entry = canonical_map(vec![
        (t("path"), t("/")),
        (t("title"), t("Home")),
        (t("lang"), t("en")),
        (t("access"), t("public")),
        (t("size"), u(95)),
    ]);

    let nav_main = Value::Array(vec![
        canonical_map(vec![(t("label"), t("Home")), (t("href"), t("/"))]),
        canonical_map(vec![
            (t("label"), t("Products")),
            (t("href"), t("/products")),
        ]),
    ]);
    let nav_footer = Value::Array(vec![
        canonical_map(vec![(t("label"), t("Privacy")), (t("href"), t("/privacy"))]),
        canonical_map(vec![(t("label"), t("Terms")), (t("href"), t("/terms"))]),
    ]);
    let navigation = canonical_map(vec![(t("main"), nav_main), (t("footer"), nav_footer)]);

    let meta = canonical_map(vec![
        (t("generated_at"), epoch(1742515200)),
        (t("total_pages"), u(1)),
        (t("total_size"), u(95)),
        (t("bundle_available"), b(false)),
    ]);

    self_described(canonical_map(vec![
        (ii(0), t("cbor-web-manifest")),
        (ii(1), u(2)),
        (ii(2), site_meta),
        (ii(3), Value::Array(vec![page_entry])),
        (ii(4), navigation),
        (ii(5), meta),
    ]))
}

/// TV6 — Doléance: agent feedback document with signals
fn test_vector_6_doleance() -> Value {
    let context = canonical_map(vec![
        (t("domain"), t("test.example")),
        (t("page_path"), t("/contact")),
    ]);

    let signal = canonical_map(vec![
        (t("signal"), t("missing_data")),
        (t("block_type"), t("p")),
        (t("details"), t("Missing contact info")),
    ]);

    let agent_info = canonical_map(vec![
        (t("agent"), t("system")),
        (t("timestamp"), epoch(1736951400)),
    ]);

    self_described(canonical_map(vec![
        (ii(0), t("cbor-web-doleance")),
        (ii(1), u(1)),
        (ii(2), context),
        (ii(3), Value::Array(vec![signal])),
        (ii(4), agent_info),
    ]))
}

/// TV7 — Diff Manifest: differential manifest with change entries and stats
fn test_vector_7_diff_manifest() -> Value {
    let base_hash: Vec<u8> = (0u8..32u8).collect();
    let change_hash: Vec<u8> = (32u8..64u8).collect();

    let change = canonical_map(vec![
        (t("path"), t("/about")),
        (t("action"), t("modified")),
        (t("hash"), bstr_bytes(&change_hash)),
        (t("size"), u(120)),
    ]);

    let stats = canonical_map(vec![
        (t("pages_added"), u(0)),
        (t("pages_modified"), u(1)),
        (t("pages_removed"), u(0)),
        (t("total_pages_now"), u(5)),
    ]);

    self_described(canonical_map(vec![
        (ii(0), t("cbor-web-diff-manifest")),
        (ii(1), u(1)),
        (ii(2), bstr_bytes(&base_hash)),
        (ii(3), epoch(1742515200)),
        (ii(4), epoch(1742601600)),
        (ii(5), Value::Array(vec![change])),
        (ii(6), stats),
    ]))
}

/// TV8 — Security Manifest: extends TV1 manifest with key 10 (security)
fn test_vector_8_security_manifest() -> Value {
    let mut entries: Vec<(Value, Value)> = match manifest_map() {
        Value::Map(e) => e,
        _ => unreachable!(),
    };

    let security = canonical_map(vec![
        (t("security_level"), t("S1")),
        (t("token_required"), b(true)),
        (t("contract_address"), t("0x1234...")),
        (t("chain"), t("ethereum")),
        (t("signing_algorithm"), t("EdDSA")),
    ]);

    entries.push((ii(10), security));
    self_described(canonical_map(entries))
}

/// TV9 — Page with Multimedia: extends TV2 page with a rich-image block
fn test_vector_9_page_multimedia() -> Value {
    let entries: Vec<(Value, Value)> = match page_map() {
        Value::Map(e) => e,
        _ => unreachable!(),
    };

    let mut new_entries: Vec<(Value, Value)> = Vec::new();
    for (k, v) in entries {
        if k == ii(4) {
            if let Value::Array(mut blocks) = v {
                let image_block = canonical_map(vec![
                    (t("t"), t("image")),
                    (t("trust"), u(0)),
                    (t("src"), t("https://example.com/photo.jpg")),
                    (t("alt"), t("A photo")),
                    (t("semantic_role"), t("product_photo")),
                    (
                        t("dimensions"),
                        canonical_map(vec![(t("w"), u(800)), (t("h"), u(600))]),
                    ),
                    (t("format"), t("image/jpeg")),
                    (t("file_size"), u(50000)),
                ]);
                blocks.push(image_block);
                new_entries.push((k, Value::Array(blocks)));
            } else {
                new_entries.push((k, v));
            }
        } else {
            new_entries.push((k, v));
        }
    }

    self_described(canonical_map(new_entries))
}

fn print_annotated_hex(label: &str, data: &[u8]) {
    println!("\n--- {} ---", label);
    let hex_str = hex::encode_upper(data);
    for i in (0..hex_str.len()).step_by(64) {
        let end = std::cmp::min(i + 64, hex_str.len());
        println!("  {}", &hex_str[i..end]);
    }
}

fn verify_key_order(label: &str, keys: &[&str]) {
    let mut prev_enc: Option<Vec<u8>> = None;
    let mut prev_name: &str = "";
    println!("\n  Key order for {}:", label);
    for &k in keys {
        let enc = encode_value(&t(k));
        println!(
            "    {} -> \"{}\" ({} bytes)",
            hex::encode_upper(&enc),
            k,
            enc.len()
        );
        if let Some(ref pe) = prev_enc {
            let ok = if pe.len() != enc.len() {
                pe.len() < enc.len()
            } else {
                pe < &enc
            };
            assert!(
                ok,
                "KEY ORDER VIOLATION: \"{}\" should come before \"{}\"",
                prev_name, k
            );
        }
        prev_enc = Some(enc);
        prev_name = k;
    }
    println!("    \u{2705} Order correct");
}

fn main() {
    let cli = Cli::parse();

    println!("\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}");
    println!("\u{2551}  CBOR-Web v2.1 \u{2014} Test Vector Generator (Rust/ciborium) \u{2551}");
    println!("\u{2551}  RFC 8949 \u{00A7}4.2.1 Deterministic Encoding                \u{2551}");
    println!("\u{255A}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255D}");

    // ── TV1: Minimal Manifest ──
    println!("\n========== TEST VECTOR 1: Minimal Manifest ==========");
    let tv1 = test_vector_1_manifest();
    let tv1_bytes = encode_value(&tv1);
    let tv1_sha = hex::encode_upper(Sha256::digest(&tv1_bytes));

    println!("Size: {} bytes", tv1_bytes.len());
    println!("SHA-256: {}", tv1_sha);
    print_annotated_hex("Hex dump", &tv1_bytes);

    verify_key_order(
        "site-metadata",
        &["name", "domain", "languages", "default_language"],
    );
    verify_key_order("page-entry", &["lang", "path", "size", "title", "access"]);
    verify_key_order(
        "meta",
        &[
            "total_size",
            "total_pages",
            "generated_at",
            "bundle_available",
        ],
    );

    // ── TV2: Minimal Page ──
    println!("\n========== TEST VECTOR 2: Minimal Page ==========");
    let tv2 = test_vector_2_page();
    let tv2_bytes = encode_value(&tv2);
    let tv2_sha = hex::encode_upper(Sha256::digest(&tv2_bytes));

    println!("Size: {} bytes", tv2_bytes.len());
    println!("SHA-256: {}", tv2_sha);
    print_annotated_hex("Hex dump", &tv2_bytes);

    verify_key_order("identity", &["lang", "path", "canonical"]);
    verify_key_order("heading-block", &["l", "t", "v"]);

    // Verify self-described tag
    assert_eq!(
        &tv2_bytes[0..3],
        &[0xD9, 0xD9, 0xF7],
        "Missing self-described tag"
    );
    println!("\n  \u{2705} Self-described CBOR tag verified");

    // ── TV3: Product Page ──
    println!("\n========== TEST VECTOR 3: Product Page ==========");
    let tv3 = test_vector_3_product();
    let tv3_bytes = encode_value(&tv3);
    let tv3_sha = hex::encode_upper(Sha256::digest(&tv3_bytes));

    println!("Size: {} bytes", tv3_bytes.len());
    println!("SHA-256: {}", tv3_sha);
    print_annotated_hex(
        "Hex dump (truncated)",
        &tv3_bytes[..std::cmp::min(256, tv3_bytes.len())],
    );

    // ── TV4: Bundle ──
    println!("\n========== TEST VECTOR 4: Bundle ==========");
    let tv4 = test_vector_4_bundle();
    let tv4_bytes = encode_value(&tv4);
    let tv4_sha = hex::encode_upper(Sha256::digest(&tv4_bytes));

    println!("Size: {} bytes", tv4_bytes.len());
    println!("SHA-256: {}", tv4_sha);
    print_annotated_hex("Hex dump", &tv4_bytes);

    // Verify self-described tag on bundle
    assert_eq!(
        &tv4_bytes[0..3],
        &[0xD9, 0xD9, 0xF7],
        "Missing self-described tag"
    );
    println!("\n  \u{2705} Self-described CBOR tag verified");

    // ── TV5: Navigation ──
    println!("\n========== TEST VECTOR 5: Navigation ==========");
    let tv5 = test_vector_5_navigation();
    let tv5_bytes = encode_value(&tv5);
    let tv5_sha = hex::encode_upper(Sha256::digest(&tv5_bytes));

    println!("Size: {} bytes", tv5_bytes.len());
    println!("SHA-256: {}", tv5_sha);
    print_annotated_hex("Hex dump", &tv5_bytes);

    verify_key_order("navigation", &["main", "footer"]);
    verify_key_order("nav-item", &["href", "label"]);

    // ── TV6: Doléance ──
    println!("\n========== TEST VECTOR 6: Doléance ==========");
    let tv6 = test_vector_6_doleance();
    let tv6_bytes = encode_value(&tv6);
    let tv6_sha = hex::encode_upper(Sha256::digest(&tv6_bytes));

    println!("Size: {} bytes", tv6_bytes.len());
    println!("SHA-256: {}", tv6_sha);
    print_annotated_hex("Hex dump", &tv6_bytes);

    assert_eq!(
        &tv6_bytes[0..3],
        &[0xD9, 0xD9, 0xF7],
        "Missing self-described tag"
    );
    println!("\n  \u{2705} Self-described CBOR tag verified");

    verify_key_order("context", &["domain", "page_path"]);
    verify_key_order("signal", &["signal", "details", "block_type"]);
    verify_key_order("agent_info", &["agent", "timestamp"]);

    // ── TV7: Diff Manifest ──
    println!("\n========== TEST VECTOR 7: Diff Manifest ==========");
    let tv7 = test_vector_7_diff_manifest();
    let tv7_bytes = encode_value(&tv7);
    let tv7_sha = hex::encode_upper(Sha256::digest(&tv7_bytes));

    println!("Size: {} bytes", tv7_bytes.len());
    println!("SHA-256: {}", tv7_sha);
    print_annotated_hex("Hex dump", &tv7_bytes);

    assert_eq!(
        &tv7_bytes[0..3],
        &[0xD9, 0xD9, 0xF7],
        "Missing self-described tag"
    );
    println!("\n  \u{2705} Self-described CBOR tag verified");

    verify_key_order("change-entry", &["hash", "path", "size", "action"]);
    verify_key_order(
        "stats",
        &[
            "pages_added",
            "pages_removed",
            "pages_modified",
            "total_pages_now",
        ],
    );

    // ── TV8: Security Manifest ──
    println!("\n========== TEST VECTOR 8: Security Manifest ==========");
    let tv8 = test_vector_8_security_manifest();
    let tv8_bytes = encode_value(&tv8);
    let tv8_sha = hex::encode_upper(Sha256::digest(&tv8_bytes));

    println!("Size: {} bytes", tv8_bytes.len());
    println!("SHA-256: {}", tv8_sha);
    print_annotated_hex("Hex dump", &tv8_bytes);

    assert_eq!(
        &tv8_bytes[0..3],
        &[0xD9, 0xD9, 0xF7],
        "Missing self-described tag"
    );
    println!("\n  \u{2705} Self-described CBOR tag verified");

    verify_key_order(
        "security",
        &[
            "chain",
            "security_level",
            "token_required",
            "contract_address",
            "signing_algorithm",
        ],
    );

    // ── TV9: Page with Multimedia ──
    println!("\n========== TEST VECTOR 9: Page with Multimedia ==========");
    let tv9 = test_vector_9_page_multimedia();
    let tv9_bytes = encode_value(&tv9);
    let tv9_sha = hex::encode_upper(Sha256::digest(&tv9_bytes));

    println!("Size: {} bytes", tv9_bytes.len());
    println!("SHA-256: {}", tv9_sha);
    print_annotated_hex("Hex dump", &tv9_bytes);

    assert_eq!(
        &tv9_bytes[0..3],
        &[0xD9, 0xD9, 0xF7],
        "Missing self-described tag"
    );
    println!("\n  \u{2705} Self-described CBOR tag verified");

    verify_key_order(
        "image-block",
        &[
            "t",
            "alt",
            "src",
            "trust",
            "format",
            "file_size",
            "dimensions",
            "semantic_role",
        ],
    );

    // ── Summary ──
    println!("\n\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}");
    println!("\u{2551}  SUMMARY \u{2014} All test vectors for spec inclusion          \u{2551}");
    println!("\u{2560}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2563}");
    println!(
        "\u{2551}  TV1 Manifest   : {:>4} bytes  SHA-256: {}...\u{2551}",
        tv1_bytes.len(),
        &tv1_sha[..16]
    );
    println!(
        "\u{2551}  TV2 Page       : {:>4} bytes  SHA-256: {}...\u{2551}",
        tv2_bytes.len(),
        &tv2_sha[..16]
    );
    println!(
        "\u{2551}  TV3 Product    : {:>4} bytes  SHA-256: {}...\u{2551}",
        tv3_bytes.len(),
        &tv3_sha[..16]
    );
    println!(
        "\u{2551}  TV4 Bundle     : {:>4} bytes  SHA-256: {}...\u{2551}",
        tv4_bytes.len(),
        &tv4_sha[..16]
    );
    println!(
        "\u{2551}  TV5 Navigation : {:>4} bytes  SHA-256: {}...\u{2551}",
        tv5_bytes.len(),
        &tv5_sha[..16]
    );
    println!(
        "\u{2551}  TV6 Doléance   : {:>4} bytes  SHA-256: {}...\u{2551}",
        tv6_bytes.len(),
        &tv6_sha[..16]
    );
    println!(
        "\u{2551}  TV7 Diff Manif. : {:>4} bytes  SHA-256: {}...\u{2551}",
        tv7_bytes.len(),
        &tv7_sha[..16]
    );
    println!(
        "\u{2551}  TV8 Security   : {:>4} bytes  SHA-256: {}...\u{2551}",
        tv8_bytes.len(),
        &tv8_sha[..16]
    );
    println!(
        "\u{2551}  TV9 Multimedia : {:>4} bytes  SHA-256: {}...\u{2551}",
        tv9_bytes.len(),
        &tv9_sha[..16]
    );
    println!("\u{255A}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255D}");

    // Write binary files
    std::fs::create_dir_all(&cli.output).unwrap();
    let out = &cli.output;
    std::fs::write(format!("{}/tv1_manifest.cbor", out), &tv1_bytes).unwrap();
    std::fs::write(format!("{}/tv2_page.cbor", out), &tv2_bytes).unwrap();
    std::fs::write(format!("{}/tv3_product.cbor", out), &tv3_bytes).unwrap();
    std::fs::write(format!("{}/tv4_bundle.cbor", out), &tv4_bytes).unwrap();
    std::fs::write(format!("{}/tv5_navigation.cbor", out), &tv5_bytes).unwrap();
    std::fs::write(format!("{}/tv6_doleance.cbor", out), &tv6_bytes).unwrap();
    std::fs::write(format!("{}/tv7_diff_manifest.cbor", out), &tv7_bytes).unwrap();
    std::fs::write(format!("{}/tv8_security.cbor", out), &tv8_bytes).unwrap();
    std::fs::write(format!("{}/tv9_multimedia.cbor", out), &tv9_bytes).unwrap();

    // Write a JSON summary for the spec builder
    let summary = format!(
        r#"{{"tv1":{{"size":{},"sha256":"{}"}},"tv2":{{"size":{},"sha256":"{}"}},"tv3":{{"size":{},"sha256":"{}"}},"tv4":{{"size":{},"sha256":"{}"}},"tv5":{{"size":{},"sha256":"{}"}},"tv6":{{"size":{},"sha256":"{}"}},"tv7":{{"size":{},"sha256":"{}"}},"tv8":{{"size":{},"sha256":"{}"}},"tv9":{{"size":{},"sha256":"{}"}}}}"#,
        tv1_bytes.len(),
        tv1_sha,
        tv2_bytes.len(),
        tv2_sha,
        tv3_bytes.len(),
        tv3_sha,
        tv4_bytes.len(),
        tv4_sha,
        tv5_bytes.len(),
        tv5_sha,
        tv6_bytes.len(),
        tv6_sha,
        tv7_bytes.len(),
        tv7_sha,
        tv8_bytes.len(),
        tv8_sha,
        tv9_bytes.len(),
        tv9_sha,
    );
    std::fs::write(format!("{}/test_vectors_summary.json", out), summary).unwrap();

    println!("\n\u{2705} Binary .cbor files written to {}", out);
    println!("\u{2705} Summary written to test_vectors_summary.json");
}
