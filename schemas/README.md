# CDDL Schemas

This directory contains standalone CDDL (RFC 8610) schema files extracted from the CBOR-Web specification documents.

| File | Source | Description |
|------|--------|-------------|
| `cbor-web-core.cddl` | CBOR-WEB-CORE.md Appendix A | Manifest, page, bundle, content blocks |
| `cbor-web-security.cddl` | CBOR-WEB-SECURITY.md Appendix A | COSE signatures, security levels, access control |
| `cbor-web-multimedia.cddl` | CBOR-WEB-MULTIMEDIA.md Appendix A | Image variants, audio, video, channels |
| `cbor-web-generative.cddl` | CBOR-WEB-GENERATIVE.md Appendix C | Generative blocks, forms, commerce |
| `cbor-web-unified.cddl` | CBOR-WEB-REFERENCE.md §1 | All types in one unified schema |

## Validation

Using [cddl](https://github.com/anweiss/cddl):

```bash
cargo install cddl
cddl validate cbor-web-core.cddl
```

Using [Ruby cddl](https://rubygems.org/gems/cddl):

```bash
gem install cddl
cddl cbor-web-core.cddl
```
