# CBOR-Web Test Vectors

Binary test vectors for validating CBOR-Web implementations. Each file is a self-described CBOR document (tag 55799).

| Vector | Spec Section | Description |
|--------|-------------|-------------|
| `tv1_manifest.cbor` | CORE §3 | Manifest with site metadata, page list, and manifest-meta |
| `tv2_page.cbor` | CORE §4 | Full page with identity, metadata, and content blocks |
| `tv3_product.cbor` | CORE §4 + GENERATIVE §5 | Page with structured data (Schema.org Product) |
| `tv4_bundle.cbor` | CORE §6 | Bundle containing multiple pages keyed by path |
| `tv5_navigation.cbor` | CORE §5 | Manifest with navigation structure at key 4 |

## Verification

```bash
# Check self-described CBOR tag (should start with d9d9f7)
xxd -p -l3 tv1_manifest.cbor

# Generate fresh vectors
cd ../tools/cbor-vectors && cargo run -- --output /tmp/tv
```

## Summary

`test_vectors_summary.json` provides a machine-readable index of all vectors with their SHA-256 hashes.
