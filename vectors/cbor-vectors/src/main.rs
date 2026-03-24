//! CBOR-Web v2.1 — Test Vector Generator (Rust/ciborium)
//! RFC 8949 §4.2.1 Deterministic Encoding

use ciborium::Value;
use sha2::{Sha256, Digest};

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
    
    pairs.sort_by(|a, b| {
        a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(&b.0))
    });
    
    Value::Map(pairs.into_iter().map(|(_, k, v)| (k, v)).collect())
}

fn t(s: &str) -> Value { Value::Text(s.to_string()) }
fn u(n: u64) -> Value { Value::Integer(ciborium::value::Integer::from(n)) }
fn ii(n: i64) -> Value { Value::Integer(ciborium::value::Integer::from(n)) }
fn b(v: bool) -> Value { Value::Bool(v) }
fn epoch(ts: u64) -> Value { Value::Tag(1, Box::new(u(ts))) }
fn self_described(inner: Value) -> Value { Value::Tag(55799, Box::new(inner)) }

fn test_vector_1_manifest() -> Value {
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
    
    let manifest = canonical_map(vec![
        (ii(0), t("cbor-web-manifest")),
        (ii(1), u(2)),
        (ii(2), site_meta),
        (ii(3), Value::Array(vec![page_entry])),
        (ii(5), meta),
    ]);
    
    self_described(manifest)
}

fn test_vector_2_page() -> Value {
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
    
    let paragraph = canonical_map(vec![
        (t("t"), t("p")),
        (t("v"), t("Hello, World!")),
    ]);
    
    let page = canonical_map(vec![
        (ii(0), t("cbor-web-page")),
        (ii(1), u(2)),
        (ii(2), identity),
        (ii(3), metadata),
        (ii(4), Value::Array(vec![heading, paragraph])),
    ]);
    
    self_described(page)
}

fn test_vector_3_product() -> Value {
    let identity = canonical_map(vec![
        (t("path"), t("/products/lions-mane")),
        (t("canonical"), t("https://verdetao.com/products/lions-mane")),
        (t("lang"), t("fr")),
        (t("alternates"), canonical_map(vec![
            (t("es"), t("/es/productos/melena-de-leon")),
        ])),
    ]);
    
    let metadata = canonical_map(vec![
        (t("title"), t("Lion's Mane")),
        (t("category"), t("products")),
        (t("updated"), epoch(1742428800)),
        (t("tags"), Value::Array(vec![t("champignon"), t("nootropique")])),
    ]);
    
    let h1 = canonical_map(vec![
        (t("t"), t("h")), (t("l"), u(1)),
        (t("v"), t("Lion's Mane")),
    ]);
    let p1 = canonical_map(vec![
        (t("t"), t("p")),
        (t("v"), t("Extrait de Hericium erinaceus concentre 10:1.")),
    ]);
    let tbl = canonical_map(vec![
        (t("t"), t("table")),
        (t("headers"), Value::Array(vec![t("Propriete"), t("Valeur")])),
        (t("rows"), Value::Array(vec![
            Value::Array(vec![t("Prix"), t("29.90 EUR")]),
            Value::Array(vec![t("Capsules"), t("90")]),
        ])),
    ]);
    let cta = canonical_map(vec![
        (t("t"), t("cta")),
        (t("v"), t("Ajouter au panier")),
        (t("href"), t("/cart/add/lions-mane")),
    ]);
    
    let structured = canonical_map(vec![
        (t("type"), t("Product")),
        (t("name"), t("Lion's Mane")),
        (t("offers"), canonical_map(vec![
            (t("type"), t("Offer")),
            (t("price"), Value::Float(29.90)),
            (t("priceCurrency"), t("EUR")),
            (t("availability"), t("InStock")),
        ])),
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
        println!("    {} -> \"{}\" ({} bytes)", hex::encode_upper(&enc), k, enc.len());
        if let Some(ref pe) = prev_enc {
            let ok = if pe.len() != enc.len() {
                pe.len() < enc.len()
            } else {
                pe < &enc
            };
            assert!(ok, "KEY ORDER VIOLATION: \"{}\" should come before \"{}\"", prev_name, k);
        }
        prev_enc = Some(enc);
        prev_name = k;
    }
    println!("    ✅ Order correct");
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  CBOR-Web v2.1 — Test Vector Generator (Rust/ciborium) ║");
    println!("║  RFC 8949 §4.2.1 Deterministic Encoding                ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    
    // ── TV1: Minimal Manifest ──
    println!("\n========== TEST VECTOR 1: Minimal Manifest ==========");
    let tv1 = test_vector_1_manifest();
    let tv1_bytes = encode_value(&tv1);
    let tv1_sha = hex::encode_upper(Sha256::digest(&tv1_bytes));
    
    println!("Size: {} bytes", tv1_bytes.len());
    println!("SHA-256: {}", tv1_sha);
    print_annotated_hex("Hex dump", &tv1_bytes);
    
    verify_key_order("site-metadata", &["name", "domain", "languages", "default_language"]);
    verify_key_order("page-entry", &["lang", "path", "size", "title", "access"]);
    verify_key_order("meta", &["total_size", "total_pages", "generated_at", "bundle_available"]);
    
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
    assert_eq!(&tv2_bytes[0..3], &[0xD9, 0xD9, 0xF7], "Missing self-described tag");
    println!("\n  ✅ Self-described CBOR tag verified");
    
    // ── TV3: Product Page ──
    println!("\n========== TEST VECTOR 3: Product Page ==========");
    let tv3 = test_vector_3_product();
    let tv3_bytes = encode_value(&tv3);
    let tv3_sha = hex::encode_upper(Sha256::digest(&tv3_bytes));
    
    println!("Size: {} bytes", tv3_bytes.len());
    println!("SHA-256: {}", tv3_sha);
    print_annotated_hex("Hex dump (truncated)", &tv3_bytes[..std::cmp::min(256, tv3_bytes.len())]);
    
    // ── Summary ──
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  SUMMARY — All test vectors for spec inclusion          ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  TV1 Manifest : {:>4} bytes  SHA-256: {}...║", tv1_bytes.len(), &tv1_sha[..16]);
    println!("║  TV2 Page     : {:>4} bytes  SHA-256: {}...║", tv2_bytes.len(), &tv2_sha[..16]);
    println!("║  TV3 Product  : {:>4} bytes  SHA-256: {}...║", tv3_bytes.len(), &tv3_sha[..16]);
    println!("╚══════════════════════════════════════════════════════════╝");
    
    // Write binary files
    std::fs::write("/home/claude/tv1_manifest.cbor", &tv1_bytes).unwrap();
    std::fs::write("/home/claude/tv2_page.cbor", &tv2_bytes).unwrap();
    std::fs::write("/home/claude/tv3_product.cbor", &tv3_bytes).unwrap();
    
    // Write a JSON summary for the spec builder
    let summary = format!(
        r#"{{"tv1":{{"size":{},"sha256":"{}"}},"tv2":{{"size":{},"sha256":"{}"}},"tv3":{{"size":{},"sha256":"{}"}}}}"#,
        tv1_bytes.len(), tv1_sha,
        tv2_bytes.len(), tv2_sha,
        tv3_bytes.len(), tv3_sha,
    );
    std::fs::write("/home/claude/test_vectors_summary.json", summary).unwrap();
    
    println!("\n✅ Binary .cbor files written to /home/claude/");
    println!("✅ Summary written to test_vectors_summary.json");
}
