# Contributing to CBOR-Web

Thank you for your interest in improving CBOR-Web. This document describes the process for contributing.

## How to Contribute

1. **Open an issue** describing the problem or improvement before submitting a PR
2. **Fork** the repository and create a feature branch
3. **Submit a pull request** referencing the issue

## Submission Checklist

Before submitting a PR, verify:

- [ ] **Cross-document consistency** — if your change affects a CDDL type, check all 7 specification documents for references
- [ ] **Deterministic encoding** — all CBOR examples use Core Deterministic Encoding (RFC 8949 §4.2.1) with keys in correct sort order
- [ ] **CDDL validation** — CDDL schemas in the document match the prose description (REQUIRED fields have no `?`, OPTIONAL fields have `?`)
- [ ] **No orphaned references** — every `§X.Y` cross-reference points to an existing section
- [ ] **No internal artifacts** — no TODO, FIXME, or development notes remain in the text
- [ ] **Test vectors** — if you add or modify a data structure, update the corresponding test vector

## Commit Convention

```
type: short description

type: fix | feat | docs | refactor | chore
```

Examples:
- `fix: correct key ordering in page-entry CDDL`
- `docs: add missing section reference in CORE §5`
- `feat: add new content block type for footnotes`

## Review Criteria

Maintainers will verify:

1. The change is **strictly better** than the previous state (no lateral or worse changes)
2. All occurrences of the affected pattern are updated (no partial fixes)
3. The CDDL schema and prose remain consistent
4. No development artifacts or internal notes are introduced

## Scope

- **Bug fixes and clarifications** — always welcome
- **New features or block types** — open an issue for discussion first
- **Structural changes** — require consensus before implementation

## License

By contributing, you agree that your contributions will be licensed under [CC BY 4.0](LICENSE).
