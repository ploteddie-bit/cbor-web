# CBOR-Web Security Architecture v1.0

**Eight-Layer Defense Model for Machine-to-Machine Trust on the Open Web**

```
Status:       Proposed Standard
Version:      1.0 (companion to CBOR-Web Specification v2.0)
Date:         2026-03-21
Authors:      ExploDev (Eddie Plot, Claude)
Extends:      CBOR-Web v2.0 §11 (Security Considerations)
Format:       CBOR (RFC 8949)
Signing:      COSE (RFC 9052)
Identity:     W3C DID / Verifiable Credentials
License:      CC BY 4.0
Repository:   https://github.com/explodev/cbor-web
```

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Threat Model](#2-threat-model)
3. [Security Level Matrix](#3-security-level-matrix)
4. [Layer 1 — Decentralized Identity](#4-layer-1--decentralized-identity)
5. [Layer 2 — Adaptive Proof-of-Work](#5-layer-2--adaptive-proof-of-work)
6. [Layer 3 — Behavioral Analysis and Trust Scoring](#6-layer-3--behavioral-analysis-and-trust-scoring)
7. [Layer 4 — Content Integrity and Authenticity](#7-layer-4--content-integrity-and-authenticity)
8. [Layer 5 — Binary Content Protection](#8-layer-5--binary-content-protection)
9. [Layer 6 — Executable Block Sandbox](#9-layer-6--executable-block-sandbox)
10. [Layer 7 — Injection Prevention](#10-layer-7--injection-prevention)
11. [Layer 8 — Audit and Logging](#11-layer-8--audit-and-logging)
12. [Verifier Marketplace — Economic Model](#12-verifier-marketplace--economic-model)
13. [Authentication Flow](#13-authentication-flow)
14. [IANA and Registry Considerations](#14-iana-and-registry-considerations)
15. [Implementation Guidance](#15-implementation-guidance)
- [Appendix F: CDDL Schema — Security Structures](#appendix-f-cddl-schema--security-structures)
- [Appendix G: Test Vectors — Security](#appendix-g-test-vectors--security)
- [Appendix H: Security Level Decision Tree](#appendix-h-security-level-decision-tree)
- [References](#references)

---

## 1. Introduction

### 1.1 Problem Statement

CBOR-Web v1.0 and v2.0 define how machines read and act on web content. But a machine-readable web without a machine-trust layer is an open invitation to:

- **Content poisoning**: injecting false information that AI agents propagate as truth
- **Agent impersonation**: malicious bots masquerading as legitimate agents
- **Resource exhaustion**: DDoS via bundle downloads consuming publisher bandwidth
- **Code injection**: v2.0 executable blocks introducing malicious code into agent runtimes
- **Economic parasitism**: scraping entire sites without contributing to the ecosystem

The current web's security model (cookies, API keys, CAPTCHAs) was designed for human users with browsers. It is fundamentally incompatible with autonomous AI agents that:

- Have no browser to render CAPTCHAs
- May be distributed across multiple IP addresses
- Execute programmatic workflows, not click-based navigation
- Need machine-verifiable trust, not human-verifiable identity

### 1.2 Solution — Eight-Layer Defense

This specification defines an eight-layer security architecture purpose-built for machine-to-machine trust:

```
┌─────────────────────────────────────────────────────┐
│  LAYER 8 — AUDIT & LOGGING                         │
│  Every interaction recorded in CBOR format          │
├─────────────────────────────────────────────────────┤
│  LAYER 7 — INJECTION PREVENTION                    │
│  Sanitized content, prepared statements             │
├─────────────────────────────────────────────────────┤
│  LAYER 6 — EXECUTABLE SANDBOX (WASM)               │
│  Isolated runtime for generative blocks             │
├─────────────────────────────────────────────────────┤
│  LAYER 5 — BINARY CONTENT PROTECTION               │
│  Depth limits, size limits, type validation         │
├─────────────────────────────────────────────────────┤
│  LAYER 4 — CONTENT INTEGRITY (COSE)                │
│  Cryptographic signatures on manifests & pages      │
├─────────────────────────────────────────────────────┤
│  LAYER 3 — BEHAVIORAL ANALYSIS                     │
│  Trust scoring, pattern detection, adaptive limits  │
├─────────────────────────────────────────────────────┤
│  LAYER 2 — ADAPTIVE PROOF-OF-WORK                  │
│  Computational cost for heavy requests              │
├─────────────────────────────────────────────────────┤
│  LAYER 1 — DECENTRALIZED IDENTITY (DID)            │
│  W3C Verifiable Credentials, no central authority   │
└─────────────────────────────────────────────────────┘
```

Each layer is independent but cumulative. A publisher MAY implement any subset based on the Security Level Matrix (§3). Layers 1-3 protect the **publisher**. Layers 4-5 protect the **agent**. Layer 6 protects the **agent's host**. Layer 7 protects **downstream systems**. Layer 8 protects **everyone**.

### 1.3 Design Principles

| Principle | Description |
|-----------|-------------|
| **No central authority** | Identity and trust are decentralized (DIDs, not OAuth providers) |
| **Progressive security** | Each layer is optional; publishers adopt what they need |
| **Machine-native** | All security structures are CBOR-encoded, not JSON/HTML |
| **Economically sustainable** | Security has a cost; the Verifier Marketplace distributes it |
| **Forward compatible** | Unknown security fields are ignored, not rejected |

---

## 2. Threat Model

### 2.1 Threat Catalog

| ID | Threat | Attacker | Target | Severity | Layer |
|----|--------|----------|--------|----------|-------|
| T1 | **Content poisoning** | Compromised CDN / rogue publisher | Agent | CRITICAL | L4 |
| T2 | **Agent impersonation** | Malicious bot | Publisher | HIGH | L1 |
| T3 | **DDoS via bundle** | Bot swarm | Publisher | HIGH | L2, L3 |
| T4 | **CBOR bomb** | Rogue publisher | Agent | HIGH | L5 |
| T5 | **Malicious executable** | Rogue publisher | Agent host | CRITICAL | L6 |
| T6 | **SQL/command injection** | Rogue publisher | Agent's DB/OS | CRITICAL | L7 |
| T7 | **Manifest falsification** | MITM / compromised cache | Agent | HIGH | L4 |
| T8 | **Economic parasitism** | Scraper | Publisher | MEDIUM | L1, L2 |
| T9 | **Replay attack** | MITM | Publisher | MEDIUM | L1 |
| T10 | **Behavioral evasion** | Sophisticated bot | Publisher | MEDIUM | L3 |
| T11 | **Key compromise** | External attacker | Publisher signing keys | CRITICAL | L4 |
| T12 | **Log tampering** | Insider / attacker | Forensics | MEDIUM | L8 |
| T13 | **Workflow hijacking** | Rogue publisher | Agent (financial) | CRITICAL | L6 |
| T14 | **Rate limit bypass** | Distributed bot | Publisher | MEDIUM | L1, L3 |
| T15 | **Privacy violation** | Rogue publisher/agent | User | HIGH | L7, L8 |

### 2.2 Attacker Profiles

| Profile | Capability | Motivation |
|---------|-----------|------------|
| **Script kiddie** | Low: simple automation, no DID | Scraping, spam |
| **Competitive scraper** | Medium: fake DID, distributed IPs | Content theft |
| **Rogue publisher** | Medium: valid domain, malicious content | SEO manipulation, phishing |
| **Nation-state actor** | High: compromised CAs, CDN access | Disinformation, surveillance |
| **Insider threat** | High: access to publisher signing keys | Content manipulation |

### 2.3 Trust Boundaries

```
┌──────────────┐     HTTPS      ┌──────────────┐
│              │◄──────────────►│              │
│    AGENT     │   Trust        │  PUBLISHER   │
│   Runtime    │   boundary 1   │   Server     │
│              │                │              │
└──────┬───────┘                └──────┬───────┘
       │                               │
       ▼                               ▼
┌──────────────┐                ┌──────────────┐
│  AGENT HOST  │                │     CDN      │
│  (sandbox)   │                │   (cache)    │
│              │                │              │
└──────┬───────┘                └──────┬───────┘
       │                               │
       ▼                               ▼
  Trust boundary 2              Trust boundary 3
  (execution env)               (delivery chain)
       │                               │
       ▼                               ▼
┌──────────────┐                ┌──────────────┐
│  DOWNSTREAM  │                │  VERIFIER    │
│  SYSTEMS     │                │  NETWORK     │
│  (DB, APIs)  │                │  (DID check) │
└──────────────┘                └──────────────┘
```

---

## 3. Security Level Matrix

Publishers and agents choose a security level based on content sensitivity:

### 3.1 Security Levels

| Level | Name | Layers Required | Use Case |
|-------|------|----------------|----------|
| **S0** | Open | L5 only | Public documentation, blogs, static content |
| **S1** | Standard | L3 + L4 + L5 | Business sites, product catalogs, public APIs |
| **S2** | Protected | L1 + L2 + L3 + L4 + L5 + L7 | E-commerce, paid content, sensitive data |
| **S3** | Fortress | All 8 layers | Financial data, healthcare, legal, government |

### 3.2 Content-to-Security Mapping

| Content Type | Minimum Security Level | Rationale |
|-------------|----------------------|-----------|
| Static blog / docs | S0 | Low value, public, no risk |
| Product catalog (read-only) | S1 | Competitive data, moderate value |
| E-commerce (transactions) | S2 | Financial transactions, user data |
| API with write access | S2 | Data modification possible |
| Executable blocks | S2 | Code execution risk |
| Healthcare / legal | S3 | Regulatory compliance required |
| Financial data feeds | S3 | Market manipulation risk |
| Government publications | S3 | National security implications |

### 3.3 Manifest Security Declaration

The manifest MUST declare its security level in key 10:

```cbor-diag
10: {
  "security_level": "S2",
  "layers_active": [1, 2, 3, 4, 5, 7],
  "identity_required": true,
  "pow_required_for": ["bundle", "api_write"],
  "signing_algorithm": "EdDSA",
  "public_key_url": "/.well-known/cbor-web/jwks.cbor",
  "public_key_dns": "_cbor-web.verdetao.com",
  "verifier_endpoint": "https://verify.cbor-web.network/v1/check",
  "content_safety": "sanitized",
  "audit_log_retention_days": 90
}
```

---

## 4. Layer 1 — Decentralized Identity

### 4.1 Rationale

API keys and Bearer tokens require a central issuing authority. In a decentralized agent ecosystem, there is no single authority to issue credentials. W3C Decentralized Identifiers (DIDs) solve this: an agent creates its own identity, registers it on a distributed ledger, and presents verifiable credentials that any party can validate without contacting an authority.

### 4.2 Agent DID Format

Each agent MUST have a DID following the W3C DID specification. CBOR-Web RECOMMENDS the `did:iota` method for zero-fee registration:

```
did:iota:0x1234abcd5678ef901234abcd5678ef901234abcd5678ef901234abcd5678ef90
```

Alternative acceptable methods:
- `did:web` — domain-based, simple but centralized
- `did:key` — ephemeral, self-generated, no ledger registration
- `did:ethr` — Ethereum-based, fees apply

A publisher MUST accept `did:iota` and `did:key` at minimum. A publisher MAY require `did:iota` for protected content (S2+).

### 4.3 CBOR-Web Verifiable Credential

An agent presents a Verifiable Credential (VC) in **native CBOR format** (not JSON-LD) with each request. The VC is carried in the HTTP header:

```
X-CBOR-Web-VC: <base64url-encoded CBOR VC>
```

#### 4.3.1 VC Structure

```cbor-diag
{
  "type": "CborWebAgentCredential",
  "version": 1,
  "issuer": "did:iota:0xABCD...",
  "subject": "did:iota:0x1234...",
  "issued_at": 1(1742515200),
  "expires_at": 1(1742601600),
  "claims": {
    "agent_name": "ExploGeo/1.0",
    "agent_type": "crawler",
    "capabilities": ["read", "commerce", "generative"],
    "organization": "ExploDev",
    "contact": "security@explodev.com",
    "compliance": ["EU-AI-Act", "GDPR"]
  },
  "nonce": h'A1B2C3D4E5F6A7B8',
  "proof": {
    "type": "Ed25519Signature2020",
    "created": 1(1742515200),
    "verification_method": "did:iota:0x1234...#key-1",
    "proof_value": h'SIGNATURE_BYTES_64'
  }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `"type"` | text | REQUIRED | MUST be `"CborWebAgentCredential"` |
| `"version"` | uint | REQUIRED | MUST be `1` |
| `"issuer"` | text | REQUIRED | DID of the entity that issued this credential |
| `"subject"` | text | REQUIRED | DID of the agent presenting this credential |
| `"issued_at"` | tag 1 (uint) | REQUIRED | Issuance timestamp |
| `"expires_at"` | tag 1 (uint) | REQUIRED | Expiration timestamp (max 24h from issuance) |
| `"claims"` | map | REQUIRED | Agent capabilities and metadata |
| `"nonce"` | bstr (8 bytes) | REQUIRED | Random nonce to prevent replay attacks |
| `"proof"` | map | REQUIRED | Cryptographic proof (see §4.3.2) |

#### 4.3.2 Proof Structure

The proof MUST be a COSE_Sign1 (RFC 9052) signature over the VC content (all fields except `"proof"` itself):

| Field | Type | Description |
|-------|------|-------------|
| `"type"` | text | Signature algorithm: `"Ed25519Signature2020"` (RECOMMENDED) or `"ES256"` |
| `"created"` | tag 1 (uint) | Signature creation timestamp |
| `"verification_method"` | text | DID URL pointing to the public key |
| `"proof_value"` | bstr | Raw signature bytes (64 bytes for Ed25519) |

#### 4.3.3 Agent Types

| Type | Description | Typical Permissions |
|------|-------------|-------------------|
| `"crawler"` | Content indexing agent | Read manifest + pages |
| `"commerce"` | Shopping/ordering agent | Read + cart + checkout |
| `"generative"` | Template/code execution agent | Read + execute generative blocks |
| `"monitor"` | Monitoring/watchdog agent | Read manifest + diff only |
| `"full"` | Full-capability agent | All operations |

### 4.4 HTTP Header Protocol

A CBOR-Web request with identity:

```
GET /.well-known/cbor-web HTTP/1.1
Host: verdetao.com
Accept: application/cbor
X-CBOR-Web-DID: did:iota:0x1234...
X-CBOR-Web-VC: <base64url-encoded CBOR VC>
X-CBOR-Web-Nonce: A1B2C3D4E5F6A7B8
```

A server response includes the agent's trust score:

```
HTTP/1.1 200 OK
Content-Type: application/cbor
X-CBOR-Web-Trust-Score: 85
X-CBOR-Web-PoW-Required: false
X-CBOR-Web-Rate-Remaining: 45
X-CBOR-Web-Rate-Reset: 60
```

### 4.5 Anonymous Access

An agent without a DID (no `X-CBOR-Web-DID` header) MUST be treated as anonymous. Anonymous agents:

- MAY access content at security level S0
- MUST NOT access content at S1 or above
- Receive a default trust score of 30 (out of 100)
- Are subject to stricter rate limits (50% of declared rate)
- MUST NOT access generative blocks, forms, or commerce endpoints

### 4.6 DID Resolution

Server-side DID resolution flow:

```
Agent request → Extract DID from header
                    │
                    ▼
            ┌───────────────┐
            │ Local DID     │──── Cache hit ────► Verify VC signature
            │ Cache         │                         │
            └───────┬───────┘                         ▼
                    │ Cache miss              ┌───────────────┐
                    ▼                         │ Signature OK? │
            ┌───────────────┐                 └───────┬───────┘
            │ Resolve DID   │                    Yes │    │ No
            │ (IOTA Tangle  │                       ▼    ▼
            │  / blockchain │                   Accept  Reject
            │  / did:web)   │                   request  (401)
            └───────┬───────┘
                    │
                    ▼
            ┌───────────────┐
            │ Cache DID     │
            │ Document      │
            │ (TTL: 1 hour) │
            └───────────────┘
```

### 4.7 Replay Attack Prevention

The `"nonce"` field in the VC and the `X-CBOR-Web-Nonce` header MUST be:
- Cryptographically random (8 bytes minimum)
- Unique per request
- The server MUST reject VCs with a previously seen nonce within the `expires_at` window
- The server SHOULD maintain a nonce bloom filter with TTL matching the maximum VC lifetime (24h)

---

## 5. Layer 2 — Adaptive Proof-of-Work

### 5.1 Rationale

Rate limiting by IP is ineffective against distributed agents. DID-based identity helps, but a legitimate DID doesn't prevent resource abuse. Proof-of-Work adds a computational cost to expensive operations, making DDoS economically unfeasible while remaining trivial for normal usage.

### 5.2 PoW Requirement Matrix

| Operation | PoW Required | Difficulty |
|-----------|-------------|------------|
| Read manifest | NO | — |
| Read individual page | NO | — |
| Read bundle (< 1 MB) | NO | — |
| Read bundle (1-10 MB) | YES | Low (16-bit) |
| Read bundle (> 10 MB) | YES | Medium (20-bit) |
| API read endpoints | NO | — |
| API write endpoints | YES | Medium (20-bit) |
| Form submission | YES | Low (16-bit) |
| Cart / checkout | YES | High (24-bit) |
| Batch page download (> 50 pages) | YES | Low (16-bit) |

### 5.3 Challenge-Response Protocol

#### Step 1: Agent requests a PoW-protected resource

```
GET /.well-known/cbor-web/bundle HTTP/1.1
Host: verdetao.com
X-CBOR-Web-DID: did:iota:0x1234...
X-CBOR-Web-VC: <base64url VC>
```

#### Step 2: Server responds with a challenge

```
HTTP/1.1 402 Payment Required
Content-Type: application/cbor
X-CBOR-Web-PoW-Challenge: <base64url CBOR challenge>
```

Challenge body:

```cbor-diag
{
  "type": "CborWebPoWChallenge",
  "version": 1,
  "nonce": h'F1E2D3C4B5A69788',
  "difficulty": 20,
  "algorithm": "sha256",
  "target_resource": "/.well-known/cbor-web/bundle",
  "issued_at": 1(1742515200),
  "expires_at": 1(1742515260),
  "server_id": "verdetao.com"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `"nonce"` | bstr (8 bytes) | Server-generated random nonce |
| `"difficulty"` | uint | Number of leading zero bits required in hash |
| `"algorithm"` | text | MUST be `"sha256"` |
| `"expires_at"` | tag 1 | Challenge expiration (max 60 seconds from issuance) |

#### Step 3: Agent computes the proof

The agent finds a `solution` (uint) such that:

```
SHA-256( nonce || solution ) has >= difficulty leading zero bits
```

Concatenation is bytewise: 8 bytes of nonce + 8 bytes of solution (big-endian uint64).

#### Step 4: Agent resubmits with proof

```
GET /.well-known/cbor-web/bundle HTTP/1.1
Host: verdetao.com
X-CBOR-Web-DID: did:iota:0x1234...
X-CBOR-Web-VC: <base64url VC>
X-CBOR-Web-PoW-Proof: <base64url CBOR proof>
```

Proof body:

```cbor-diag
{
  "type": "CborWebPoWProof",
  "challenge_nonce": h'F1E2D3C4B5A69788',
  "solution": 847291,
  "hash": h'00000F3A...32 bytes',
  "computed_at": 1(1742515205),
  "agent_did": "did:iota:0x1234..."
}
```

#### Step 5: Server verifies and responds

The server verifies:
1. Challenge nonce exists and has not expired
2. SHA-256(nonce || solution) has the required leading zeros
3. Challenge has not already been used (one-time use)
4. Agent DID matches the VC

If valid: `200 OK` with the requested resource.
If invalid: `403 Forbidden` with error CBOR.

### 5.4 Difficulty Calibration

| Difficulty (bits) | Approximate Time (single core) | Classification |
|-------------------|-------------------------------|---------------|
| 16 | ~10 ms | Low — form submission, small bundle |
| 20 | ~100 ms | Medium — large bundle, API write |
| 24 | ~1.5 seconds | High — checkout, sensitive operations |
| 28 | ~25 seconds | Very High — reserved for abuse response |
| 32 | ~7 minutes | Extreme — effective ban |

### 5.5 Adaptive Difficulty

The server SHOULD adjust difficulty based on agent behavior:

```
base_difficulty = operation_default_difficulty
agent_score = trust_score (from Layer 3)

if agent_score >= 80:
    effective_difficulty = base_difficulty - 4    # trusted agents get easier PoW
elif agent_score >= 50:
    effective_difficulty = base_difficulty         # normal
elif agent_score >= 20:
    effective_difficulty = base_difficulty + 4    # suspicious agents get harder PoW
else:
    effective_difficulty = base_difficulty + 12   # near-ban: very expensive
```

Minimum difficulty: 8 bits. Maximum difficulty: 32 bits.

### 5.6 Hashcash / IOTA Compatibility

The PoW challenge is compatible with Hashcash (RFC 6376 informational) and can use the IOTA Tangle's PoW mechanism. An agent that already performs IOTA transactions SHOULD reuse its PoW hardware.

---

## 6. Layer 3 — Behavioral Analysis and Trust Scoring

### 6.1 Rationale

Identity (Layer 1) tells you WHO; Proof-of-Work (Layer 2) adds COST. But a sophisticated attacker can have a legitimate DID and compute PoW. Layer 3 analyzes HOW the agent behaves over time and assigns a trust score.

### 6.2 Trust Score

Every agent (identified by DID or IP for anonymous agents) has a trust score from 0 to 100:

| Score Range | Classification | Effect |
|-------------|---------------|--------|
| 80-100 | Trusted | Full access, reduced PoW, higher rate limits |
| 50-79 | Normal | Standard access, standard PoW, standard rate limits |
| 20-49 | Suspicious | Restricted access, increased PoW, lower rate limits |
| 1-19 | Restricted | Manifest-only access, maximum PoW, near-ban |
| 0 | Banned | No access (temporary ban, default 1 hour) |

New agents (first seen) start at score **50** (identified) or **30** (anonymous).

### 6.3 Behavioral Metrics

The server MUST track these metrics per agent DID:

| Metric | Healthy Range | Suspicious Signal | Weight |
|--------|--------------|-------------------|--------|
| **Request rate** | ≤ declared rate_limit | > 2x rate_limit | 25% |
| **Page diversity** | > 30% unique pages / session | Same 3 pages in loop | 15% |
| **Access pattern** | Mix of sequential + targeted | Purely sequential crawl (page 1, 2, 3...) | 10% |
| **Manifest:page ratio** | 1 manifest per 10-50 pages | 1 manifest per page (re-fetching) | 10% |
| **Bundle abuse** | 1 bundle per cooldown period | Rapid bundle re-download | 15% |
| **Error rate** | < 5% 4xx/5xx responses | > 20% errors (probing) | 10% |
| **Respect of robots.txt** | Respects Disallow | Accesses disallowed paths | 15% |

### 6.4 Score Update Algorithm

```
every request:
  for each metric M:
    score_delta = evaluate(M, current_behavior)  # range: -5 to +1
  
  new_score = clamp(0, old_score + sum(score_deltas), 100)
  
  # Slow recovery, fast decline
  if new_score > old_score:
    new_score = old_score + min(new_score - old_score, 2)  # max +2 per request
  
  # Automatic recovery over time (1 point per hour if no requests)
  if time_since_last_request > 1 hour:
    hours_idle = (now - last_request) / 3600
    new_score = min(new_score + hours_idle, 50)  # recovers to 50 max, not full trust
```

### 6.5 HTTP Response Headers

Every CBOR-Web response MUST include:

```
X-CBOR-Web-Trust-Score: 72
X-CBOR-Web-Rate-Remaining: 45
X-CBOR-Web-Rate-Reset: 60
X-CBOR-Web-Rate-Limit: 100
```

| Header | Type | Description |
|--------|------|-------------|
| `X-CBOR-Web-Trust-Score` | uint (0-100) | Agent's current trust score |
| `X-CBOR-Web-Rate-Remaining` | uint | Remaining requests in current window |
| `X-CBOR-Web-Rate-Reset` | uint | Seconds until rate limit window resets |
| `X-CBOR-Web-Rate-Limit` | uint | Total requests allowed in current window |

### 6.6 Ban Response

When an agent's trust score reaches 0:

```
HTTP/1.1 429 Too Many Requests
Content-Type: application/cbor
Retry-After: 3600
```

Response body (CBOR):

```cbor-diag
{
  "type": "CborWebBan",
  "reason": "behavioral_violation",
  "details": "Excessive request rate (247 req/s, limit 10 req/s) and sequential scraping pattern detected",
  "trust_score": 0,
  "ban_duration_seconds": 3600,
  "ban_expires_at": 1(1742518800),
  "appeal": "security@verdetao.com",
  "metrics": {
    "request_rate": 247,
    "rate_limit": 10,
    "unique_pages_ratio": 0.02,
    "error_rate": 0.35
  }
}
```

### 6.7 Rate Limiting Strategy

Rate limits are per-DID, not per-IP. An agent distributed across 100 IPs with a single DID has a single rate limit pool.

| Agent Score | Rate Multiplier | Effective Rate (base 10 req/s) |
|-------------|----------------|-------------------------------|
| 80-100 | 2.0x | 20 req/s |
| 50-79 | 1.0x | 10 req/s |
| 20-49 | 0.5x | 5 req/s |
| 1-19 | 0.1x | 1 req/s |
| 0 | 0x | Banned |

---

## 7. Layer 4 — Content Integrity and Authenticity

### 7.1 Rationale

A CDN, reverse proxy, or MITM attacker can serve modified CBOR-Web content with altered hashes. Without cryptographic signatures, an agent cannot distinguish legitimate content from injected content.

### 7.2 Signing Requirements by Security Level

| Document | S0 | S1 | S2 | S3 |
|----------|----|----|----|----|
| Manifest | NOT REQUIRED | REQUIRED | REQUIRED | REQUIRED |
| Bundle | NOT REQUIRED | RECOMMENDED | REQUIRED | REQUIRED |
| Individual pages | NOT REQUIRED | NOT REQUIRED | RECOMMENDED | REQUIRED |

### 7.3 COSE_Sign1 Format

All signatures MUST use COSE_Sign1 (RFC 9052) with the following parameters:

```cbor-diag
; COSE_Sign1 structure
[
  h'A10127',                          ; protected header: {"alg": "EdDSA"}
  {},                                  ; unprotected header (empty)
  h'PAYLOAD_HASH',                    ; payload: SHA-256 of signed content
  h'SIGNATURE_64_BYTES'               ; Ed25519 signature
]
```

#### 7.3.1 Protected Header

```cbor-diag
{
  1: -8                               ; alg: EdDSA (RFC 9053)
}
```

RECOMMENDED algorithm: **EdDSA (Ed25519)** — fast, compact (64-byte signatures), no padding oracles.

Acceptable alternatives:
- ES256 (ECDSA with P-256) — wider library support
- ES384 — higher security margin

RSA MUST NOT be used (signatures too large for binary-compact protocol).

#### 7.3.2 Payload

For manifests: SHA-256 of the CBOR-encoded keys 0-5 and 7-9 (everything except key 6 which holds the signature and key 10 which holds the security config).

For pages: SHA-256 of the complete CBOR-encoded page document.

For bundles: SHA-256 of the CBOR-encoded keys 0-2 (type, version, manifest). Pages within the bundle are individually verified via their manifest hashes.

### 7.4 Manifest Signature Location

The manifest carries its signature in key 6:

```cbor-diag
55799({
  0: "cbor-web-manifest",
  1: 2,
  2: { ... },
  3: [ ... ],
  4: { ... },
  5: { ... },
  6: [                                ; COSE_Sign1
    h'A10127',                        ; protected: alg=EdDSA
    {},                                ; unprotected
    null,                              ; payload: detached (computed from keys 0-5, 7-9)
    h'SIGNATURE_64_BYTES'             ; signature
  ],
  7: { ... },                         ; capabilities
  10: { ... }                         ; security config
})
```

### 7.5 Public Key Discovery

The publisher's public key MUST be discoverable via at least one of:

#### Method 1: CBOR Key Set (RECOMMENDED)

```
GET /.well-known/cbor-web/keys.cbor HTTP/1.1
Accept: application/cbor
```

Response:

```cbor-diag
{
  "type": "CborWebKeySet",
  "keys": [
    {
      "kid": "cbor-web-signing-2026",
      "kty": "OKP",
      "crv": "Ed25519",
      "x": h'PUBLIC_KEY_32_BYTES',
      "use": "sig",
      "valid_from": 1(1740000000),
      "valid_until": 1(1771536000)
    }
  ]
}
```

#### Method 2: DNS TXT Record

```
_cbor-web.verdetao.com. IN TXT "v=1; alg=EdDSA; key=BASE64URL_PUBLIC_KEY"
```

#### Method 3: DANE/TLSA Record

```
_443._tcp.verdetao.com. IN TLSA 3 1 1 HASH_OF_PUBLIC_KEY
```

### 7.6 Key Rotation

A publisher MUST support key rotation:

1. Generate new key pair
2. Add new key to key set with `"valid_from"` = now
3. Sign new manifests with new key
4. Keep old key in key set for 30 days (agents cache the old key)
5. Remove old key after 30 days

The key set SHOULD contain at most 2 active keys during rotation.

### 7.7 Verification Failure Handling

When an agent detects a signature verification failure:

1. The agent MUST discard the document
2. The agent SHOULD retry from a different network path (different DNS resolver, different CDN edge)
3. If the second attempt also fails, the agent MUST log the failure and alert its operator
4. The agent MUST NOT use any content from the failed document
5. The agent SHOULD report the failure to the Verifier Network (§12) if available

---

## 8. Layer 5 — Binary Content Protection

### 8.1 Parsing Limits

An agent MUST enforce the following limits when parsing any CBOR-Web document:

| Limit | Value | Rationale |
|-------|-------|-----------|
| Max nesting depth | 32 levels | Prevents stack overflow from recursive structures |
| Max decompressed/compressed ratio | 10:1 | Prevents zip bomb equivalent |
| Max elements per array | 100,000 | Prevents memory exhaustion |
| Max elements per map | 100,000 | Prevents memory exhaustion |
| Max text string size | 1 MB | Prevents single-field memory exhaustion |
| Max byte string size | 5 MB | Allows inline images but limits abuse |
| Max manifest size | 5 MB | Defined in v1.0 |
| Max page size | 1 MB | Defined in v1.0 |
| Max bundle size | 50 MB | Defined in v1.0 |
| Max total parse time | 30 seconds | Prevents algorithmic complexity attacks |

### 8.2 Type Validation

A conforming agent MUST perform strict type validation:

```
For each field defined in the CDDL schema:
  1. Verify CBOR major type matches expected type
  2. Verify value constraints (min, max, regex)
  3. Reject documents with type mismatches

Example:
  Field "level" declared as uint .ge 1 .le 6
  Value received: "3" (text string)
  Result: REJECT — expected uint, got text
```

### 8.3 Unknown Tag Handling

CBOR tags not defined in this specification or RFC 8949 MUST be handled as follows:

| Tag Range | Handling |
|-----------|----------|
| 0-5 (RFC 8949 standard) | Process normally |
| 6-23 (RFC 8949 standard) | Process if recognized, ignore content otherwise |
| 55799 (self-described) | REQUIRED — process |
| 256-65535 (registered) | Ignore content, do not process |
| > 65535 (unregistered) | REJECT document — potential exploit vector |

### 8.4 Size Consistency Verification

For every page referenced in the manifest:

```
actual_size = length(fetched_cbor_document)
declared_size = page_entry.size

if abs(actual_size - declared_size) / declared_size > 0.10:
    WARN: size mismatch exceeds 10%
    if security_level >= S2:
        REJECT document
```

### 8.5 CBOR-Web Validator

Publishers and agents SHOULD use the reference validator:

```
cbor-web-validate manifest.cbor --level S2
cbor-web-validate page.cbor --strict
cbor-web-validate bundle.cbor --check-signatures --key-set keys.cbor
```

The validator MUST check:
1. Self-described CBOR tag present
2. Deterministic encoding (keys sorted, minimal integer encoding)
3. All required fields present
4. Type constraints satisfied
5. Size limits respected
6. Hash consistency (manifest hash vs actual page hash)
7. Signature validity (if signed)
8. Nesting depth within limits

---

## 9. Layer 6 — Executable Block Sandbox

### 9.1 Rationale

v2.0 Generative Blocks (§16) introduce executable code into CBOR-Web documents. This is the single most dangerous feature in the specification. Without a mandatory sandbox, a malicious publisher can execute arbitrary code on an agent's host.

### 9.2 Execution Classification

| Block Type | Trust Level | Required Isolation |
|-----------|-------------|-------------------|
| `"template"` | 1 | None — pure string interpolation, no code execution |
| `"schema"` | 0 | None — declarative data, no execution |
| `"constraint"` | 0 | None — declarative rules, limited expression evaluator |
| `"executable"` | 2 | **MANDATORY SANDBOX** |
| `"workflow"` | 3 | **MANDATORY SANDBOX** for execute steps; network isolation for API steps |
| `"api_endpoint"` | 3 | Network isolation — whitelist destination |

### 9.3 Sandbox Requirements

#### 9.3.1 WASM Runtime (RECOMMENDED)

The RECOMMENDED sandbox is **WebAssembly (WASM)**. An agent SHOULD:

1. Compile the executable block code to WASM (or use a WASM-based interpreter)
2. Run the WASM module with:
   - **No network access** (no WASI networking)
   - **No filesystem access** (no WASI filesystem)
   - **No shared memory** (isolated linear memory)
   - **Bounded memory**: maximum as declared in `sandbox_requirements.max_memory_mb`
   - **Bounded time**: maximum as declared in `sandbox_requirements.max_execution_time_ms`
3. Communicate only via the defined `inputs` and `outputs`

#### 9.3.2 Alternative Sandboxes

| Sandbox | Acceptable | Notes |
|---------|-----------|-------|
| WASM (Wasmtime, Wasmer) | RECOMMENDED | Best isolation, cross-platform |
| Docker/OCI container | Acceptable | Heavier, but strong isolation |
| Linux seccomp + namespaces | Acceptable | Linux only, complex to configure |
| Python `RestrictedPython` | NOT RECOMMENDED | Incomplete isolation, known escapes |
| `eval()` in any language | PROHIBITED | No isolation whatsoever |

#### 9.3.3 Default Limits

If the executable block does not declare `sandbox_requirements`, the agent MUST apply these defaults:

| Resource | Default Limit |
|----------|--------------|
| Network | DENIED |
| Filesystem | DENIED |
| Execution time | 5,000 ms |
| Memory | 64 MB |
| CPU | Single thread |

### 9.4 Trust Origin Classification

Beyond the numeric trust level (0-3), v2.0 executable blocks declare trust origin:

| Origin | Value | Meaning | Agent Default |
|--------|-------|---------|---------------|
| `"publisher_signed"` | `"ps"` | Code is signed by the publisher's COSE key | Execute in sandbox |
| `"community_verified"` | `"cv"` | Code hash appears in community verification registry | Execute in sandbox |
| `"unverified"` | `"uv"` | No verification of code origin | Show code + ask user |

```cbor-diag
{
  "t": "executable",
  "trust": 2,
  "trust_origin": "ps",
  "publisher_signature": h'COSE_SIGN1_BYTES',
  ...
}
```

### 9.5 Mandatory User Confirmation

An agent MUST NOT execute an `"unverified"` executable block without:

1. Displaying the full source code to the user
2. Displaying the declared purpose, inputs, and outputs
3. Displaying the sandbox requirements (especially `network` and `filesystem`)
4. Receiving explicit user confirmation

For `"publisher_signed"` blocks, the agent MAY auto-execute in sandbox if:
- The publisher's trust score is >= 80
- The signature is valid
- The sandbox requirements request no network or filesystem access

### 9.6 Capability Model

Each executable block SHOULD declare the capabilities it requires:

```cbor-diag
"capabilities_required": {
  "read_network": false,
  "write_network": false,
  "read_filesystem": false,
  "write_filesystem": false,
  "spawn_process": false,
  "access_env": false,
  "use_crypto": true,
  "use_time": true
}
```

The agent maps these to sandbox permissions:

| Capability | WASM Mapping | Risk Level |
|-----------|-------------|------------|
| `read_network` | WASI sock_recv | HIGH |
| `write_network` | WASI sock_send | HIGH |
| `read_filesystem` | WASI fd_read | HIGH |
| `write_filesystem` | WASI fd_write | HIGH |
| `spawn_process` | WASI proc_exec | CRITICAL — ALWAYS DENY |
| `access_env` | WASI environ_get | MEDIUM |
| `use_crypto` | Pure computation | LOW |
| `use_time` | WASI clock_time_get | LOW |

An agent MUST deny `spawn_process` unconditionally. An agent SHOULD deny all network and filesystem capabilities unless explicitly approved by the user.

---

## 10. Layer 7 — Injection Prevention

### 10.1 Rationale

CBOR-Web text fields can contain SQL, XSS, shell commands, LDAP injection, or template injection payloads. While CBOR is binary (not parsed as HTML), an agent that uses CBOR-Web text values in downstream operations (database queries, email templates, shell commands) is vulnerable.

### 10.2 Publisher Responsibilities

A publisher MUST sanitize content before CBOR encoding:

| Field Type | Sanitization Rule |
|-----------|------------------|
| Page title | Strip HTML tags, limit to 300 characters |
| Page description | Strip HTML tags, limit to 500 characters |
| Paragraph text (`"v"`) | Strip HTML tags; preserve semantic text |
| List items | Strip HTML tags per item |
| Table cells | Strip HTML tags; numeric cells MUST be validated |
| Code blocks | NO sanitization (code is literal content) |
| External URLs | Validate URL scheme (`https://` only) |
| Email addresses | Validate email format (RFC 5322) |
| Form field values | Validate against declared type and constraints |

### 10.3 Agent Responsibilities

An agent MUST treat all CBOR-Web text values as **untrusted input**:

1. **SQL**: MUST use parameterized queries / prepared statements. NEVER concatenate CBOR-Web text into SQL strings.

2. **Shell commands**: MUST NOT pass CBOR-Web text to shell commands. Use programmatic APIs instead.

3. **HTML rendering**: If displaying CBOR-Web text in a web UI, MUST HTML-escape all values.

4. **Template engines**: MUST use auto-escaping template engines. NEVER render raw CBOR-Web text in templates.

5. **Log injection**: MUST sanitize CBOR-Web text before writing to logs (strip newlines, control characters).

### 10.4 Content Safety Declaration

The manifest MAY declare its sanitization status:

```cbor-diag
10: {
  "content_safety": "sanitized",
  "sanitization_version": "1.0",
  "sanitization_rules": ["strip_html", "validate_urls", "validate_emails", "limit_lengths"]
}
```

| Value | Meaning |
|-------|---------|
| `"sanitized"` | Publisher has applied sanitization rules |
| `"raw"` | Content is direct HTML extraction, not sanitized |
| `"user_generated"` | Content includes user-generated text (higher risk) |

An agent processing `"raw"` or `"user_generated"` content MUST apply additional sanitization before downstream use.

### 10.5 URL Validation

All URL fields (`"src"`, `"href"`, `"canonical"`, `"action"`, `"endpoint"`, `"stream_url"`) MUST:

1. Use `https://` scheme (except for `"stream_url"` which MAY use `wss://`)
2. Not contain `javascript:`, `data:`, or `file:` schemes
3. Not contain URL-encoded control characters
4. Resolve to a valid hostname

An agent MUST validate URLs before following them.

---

## 11. Layer 8 — Audit and Logging

### 11.1 Rationale

Without standardized logging, security incidents cannot be investigated, patterns cannot be detected, and the trust scoring system has no data to work with.

### 11.2 Log Format

All CBOR-Web access logs MUST be in CBOR format (not text):

```cbor-diag
{
  "type": "CborWebAccessLog",
  "version": 1,
  "timestamp": 1(1742515200),
  "server": "verdetao.com",
  "agent": {
    "did": "did:iota:0x1234...",
    "ip": "203.0.113.42",
    "user_agent": "ExploGeo/1.0",
    "trust_score": 72
  },
  "request": {
    "method": "GET",
    "path": "/.well-known/cbor-web/pages/products_lions-mane.cbor",
    "headers": {
      "X-CBOR-Web-DID": "did:iota:0x1234...",
      "Accept-Encoding": "br"
    }
  },
  "response": {
    "status": 200,
    "size": 2340,
    "content_type": "application/cbor",
    "encoding": "br",
    "compressed_size": 1870
  },
  "security": {
    "layer_1_identity": "verified",
    "layer_2_pow": "not_required",
    "layer_3_score_delta": -1,
    "layer_4_signature": "valid",
    "layer_5_validation": "passed"
  },
  "processing_time_ms": 12
}
```

### 11.3 Mandatory Fields

| Field | Required | Description |
|-------|----------|-------------|
| `"timestamp"` | REQUIRED | Request timestamp (epoch) |
| `"agent"."did"` | REQUIRED if present | Agent DID (or `"anonymous"`) |
| `"agent"."ip"` | REQUIRED | Source IP address |
| `"agent"."trust_score"` | REQUIRED | Score at time of request |
| `"request"."method"` | REQUIRED | HTTP method |
| `"request"."path"` | REQUIRED | Requested path |
| `"response"."status"` | REQUIRED | HTTP status code |
| `"response"."size"` | REQUIRED | Response body size |
| `"security"` | RECOMMENDED | Security layer results |

### 11.4 Retention

| Security Level | Minimum Retention |
|---------------|------------------|
| S0 | 30 days |
| S1 | 90 days |
| S2 | 180 days |
| S3 | 365 days |

### 11.5 Log Integrity

At security level S3, logs MUST be tamper-resistant:

1. Each log entry includes a hash chain link: `"prev_hash": SHA-256(previous_entry)`
2. The hash chain is anchored to the IOTA Tangle or equivalent append-only ledger every 1000 entries
3. Log files MUST be stored on write-once media or append-only filesystems

### 11.6 Privacy Considerations

Logs containing agent DIDs and IP addresses are **personal data** under GDPR and similar regulations. Publishers MUST:

1. Document the legal basis for logging (legitimate interest: security)
2. Allow agents to request log deletion (right to erasure) after the minimum retention period
3. NOT share raw logs with third parties without the agent operator's consent
4. Anonymize logs older than the retention period (hash the DID, remove IP)

---

## 12. Verifier Marketplace — Economic Model

### 12.1 Vision

The security infrastructure of CBOR-Web requires computational work: verifying DIDs, checking PoW proofs, validating COSE signatures, monitoring behavioral patterns. This work has a real cost. The Verifier Marketplace creates a **sustainable economic model** where this cost is distributed among participants.

### 12.2 The Miner-to-Verifier Transition

The cryptocurrency mining industry has created a global infrastructure of:
- High-performance hash computation hardware
- Low-latency network connectivity
- 24/7 operational teams
- Energy procurement expertise

Currently, this infrastructure produces **empty hashes** — proof-of-work with no utility beyond securing a ledger. CBOR-Web Verifiers repurpose this infrastructure for **useful work**: securing machine-to-machine communication on the open web.

| Aspect | Bitcoin Mining | CBOR-Web Verification |
|--------|---------------|----------------------|
| Work performed | SHA-256 hashes (no utility) | DID verification, PoW checking, signature validation |
| Revenue source | Block rewards + transaction fees | Microtransactions from publishers and agents |
| Hardware | ASICs (specialized, single-purpose) | General compute (can run any verification) |
| Energy use | ~150 TWh/year globally | Orders of magnitude less (verification is cheap) |
| Barrier to entry | Millions in hardware | Standard server + CBOR-Web Verifier software |
| Value created | Ledger security | Web security + trust infrastructure |

### 12.3 Verifier Roles

| Role | Work | Payment Source |
|------|------|---------------|
| **DID Verifier** | Resolve and cache agent DIDs, verify VC signatures | Publisher (per verification) |
| **PoW Verifier** | Validate proof-of-work solutions | Publisher (per verification) |
| **Signature Verifier** | Validate COSE signatures on manifests/pages | Agent (per verification) |
| **Behavioral Analyst** | Process access logs, compute trust scores | Publisher (subscription) |
| **Content Auditor** | Compare CBOR-Web content with HTML for discrepancies | Agent (per audit) |
| **Registry Operator** | Maintain the community verification registry for executable blocks | Community (staking) |

### 12.4 Payment Protocol

Micropayments use **IOTA** (zero-fee transactions on the Tangle):

```
┌─────────┐    verification    ┌──────────┐    payment     ┌──────────┐
│  AGENT   │──── request ─────►│ VERIFIER │◄──── IOTA ─────│PUBLISHER │
│          │◄── result ────────│          │                 │          │
│          │──── IOTA ────────►│          │                 │          │
│ (optional│  (agent pays for  │          │                 │          │
│  premium)│   content audit)  │          │                 │          │
└─────────┘                    └──────────┘                 └──────────┘
```

#### Payment Amounts (Indicative)

| Verification Type | Cost (IOTA) | Equivalent (EUR) |
|-------------------|-------------|-------------------|
| DID resolution + VC check | 1 Mi | ~€0.0001 |
| PoW validation | 0.5 Mi | ~€0.00005 |
| COSE signature check | 1 Mi | ~€0.0001 |
| Content audit (full page) | 10 Mi | ~€0.001 |
| Behavioral analysis (per hour) | 100 Mi | ~€0.01 |
| Community registry listing | 1000 Mi | ~€0.10 |

### 12.5 Verifier Registration

A verifier registers on the CBOR-Web Verifier Network:

```cbor-diag
{
  "type": "CborWebVerifierRegistration",
  "did": "did:iota:0xVERIFIER...",
  "services": ["did_verification", "pow_validation", "signature_check"],
  "endpoint": "https://verify.example.com/v1",
  "stake": 10000,
  "availability": 0.999,
  "region": "EU",
  "registered_at": 1(1742515200)
}
```

Verifiers MUST stake IOTA tokens to register. The stake is slashed if the verifier produces incorrect results (detected by cross-verification from other verifiers).

### 12.6 Verification Request Protocol

A publisher requesting DID verification:

```cbor-diag
{
  "type": "CborWebVerifyRequest",
  "request_id": "req_A1B2C3D4",
  "service": "did_verification",
  "payload": {
    "did": "did:iota:0x1234...",
    "vc": { ... },
    "nonce": h'A1B2C3D4E5F6A7B8'
  },
  "payment": {
    "amount_mi": 1,
    "payer_did": "did:iota:0xPUBLISHER...",
    "transaction_hash": "IOTA_TX_HASH"
  }
}
```

Response:

```cbor-diag
{
  "type": "CborWebVerifyResponse",
  "request_id": "req_A1B2C3D4",
  "result": "valid",
  "confidence": 1.0,
  "details": {
    "did_resolved": true,
    "vc_signature_valid": true,
    "vc_not_expired": true,
    "nonce_unique": true
  },
  "verifier_did": "did:iota:0xVERIFIER...",
  "verifier_signature": h'COSE_SIGN1_BYTES',
  "timestamp": 1(1742515201)
}
```

### 12.7 Cross-Verification

To prevent corrupt verifiers, the network uses probabilistic cross-verification:

1. 10% of verification requests are randomly sent to a second verifier
2. If results disagree, a third verifier breaks the tie
3. The incorrect verifier's stake is slashed
4. Consistent disagreement (> 5% error rate) results in de-registration

### 12.8 Economic Sustainability

The model is economically sustainable because:

1. **Publishers pay for security they already need** — DDoS protection, bot management, identity verification. Currently they pay Cloudflare, Akamai, etc. The Verifier Network is a decentralized alternative.

2. **Agents pay for premium access** — an agent that wants to verify content authenticity pays a tiny fraction of what it saves by using CBOR-Web instead of HTML crawling.

3. **Verifiers earn from useful work** — unlike mining, every verification has a direct customer and measurable value.

4. **Zero-fee transactions via IOTA** — no intermediary takes a cut. The full payment goes to the verifier.

5. **Low barrier to entry** — any server can be a verifier. No specialized hardware needed.

---

## 13. Authentication Flow

### 13.1 Complete Request Flow (S2 Security Level)

```
AGENT                          PUBLISHER                      VERIFIER
  │                                │                              │
  │ 1. Generate request            │                              │
  │    with DID + VC + nonce       │                              │
  │                                │                              │
  ├────── GET /cbor-web ──────────►│                              │
  │       Headers:                 │                              │
  │       X-CBOR-Web-DID           │                              │
  │       X-CBOR-Web-VC            │                              │
  │       X-CBOR-Web-Nonce         │                              │
  │                                │                              │
  │                                │ 2. Verify DID (cache or      │
  │                                │    delegate to verifier)      │
  │                                ├── verify_did ────────────────►│
  │                                │◄── result: valid ────────────┤
  │                                │                              │
  │                                │ 3. Check trust score         │
  │                                │    (Layer 3 evaluation)      │
  │                                │                              │
  │                                │ 4. PoW required?             │
  │                                │    (check matrix vs          │
  │                                │     resource + score)        │
  │                                │                              │
  │  CASE A: No PoW needed         │                              │
  │◄───── 200 OK ─────────────────┤                              │
  │       + Content-Type: cbor     │                              │
  │       + X-CBOR-Web-Trust: 72   │                              │
  │       + COSE-signed manifest   │                              │
  │                                │                              │
  │  CASE B: PoW required          │                              │
  │◄───── 402 + PoW Challenge ────┤                              │
  │                                │                              │
  │ 5. Compute PoW solution        │                              │
  │    SHA-256(nonce || solution)   │                              │
  │    with N leading zeros        │                              │
  │                                │                              │
  ├────── GET /cbor-web ──────────►│                              │
  │       + X-CBOR-Web-PoW-Proof   │                              │
  │                                │                              │
  │                                │ 6. Verify PoW                │
  │                                │    (local or delegate)       │
  │                                ├── verify_pow ────────────────►│
  │                                │◄── result: valid ────────────┤
  │                                │                              │
  │◄───── 200 OK ─────────────────┤                              │
  │       + signed manifest        │                              │
  │                                │                              │
  │ 7. Agent verifies COSE         │                              │
  │    signature on manifest       │                              │
  │    (local or delegate)         │                              │
  ├── verify_signature ───────────────────────────────────────────►│
  │◄── result: valid ─────────────────────────────────────────────┤
  │                                │                              │
  │ 8. Agent processes manifest    │                              │
  │    and fetches pages           │                              │
  │                                │                              │
  │                                │ 9. Log access (Layer 8)      │
  │                                │    CBOR format               │
  ▼                                ▼                              ▼
```

### 13.2 Anonymous Access Flow (S0)

```
AGENT                          PUBLISHER
  │                                │
  ├────── GET /cbor-web ──────────►│
  │       (no identity headers)    │
  │                                │
  │                                │ Trust score: 30 (anonymous)
  │                                │ Rate limit: 50% of declared
  │                                │
  │◄───── 200 OK ─────────────────┤
  │       X-CBOR-Web-Trust: 30     │
  │       (unsigned manifest)      │
  ▼                                ▼
```

### 13.3 Banned Agent Flow

```
AGENT                          PUBLISHER
  │                                │
  ├────── GET /cbor-web ──────────►│
  │       X-CBOR-Web-DID: ...      │
  │                                │
  │                                │ Trust score: 0 (banned)
  │                                │
  │◄───── 429 Too Many Req ───────┤
  │       Content-Type: cbor       │
  │       Retry-After: 3600        │
  │       Body: CborWebBan         │
  │       (reason, duration,       │
  │        metrics, appeal email)  │
  ▼                                ▼
```

---

## 14. IANA and Registry Considerations

### 14.1 HTTP Header Registration

This specification defines the following HTTP headers:

| Header | Type | Direction | Description |
|--------|------|-----------|-------------|
| `X-CBOR-Web-DID` | text | Request | Agent's Decentralized Identifier |
| `X-CBOR-Web-VC` | text (base64url) | Request | Agent's Verifiable Credential (CBOR) |
| `X-CBOR-Web-Nonce` | text (hex) | Request | Request nonce for replay prevention |
| `X-CBOR-Web-PoW-Challenge` | text (base64url) | Response | PoW challenge (CBOR) |
| `X-CBOR-Web-PoW-Proof` | text (base64url) | Request | PoW solution (CBOR) |
| `X-CBOR-Web-Trust-Score` | uint | Response | Agent's current trust score (0-100) |
| `X-CBOR-Web-Rate-Remaining` | uint | Response | Remaining requests in window |
| `X-CBOR-Web-Rate-Reset` | uint | Response | Seconds until rate window reset |
| `X-CBOR-Web-Rate-Limit` | uint | Response | Total requests per window |

### 14.2 Well-Known URI Extensions

| URI Suffix | Content | MIME Type |
|-----------|---------|-----------|
| `cbor-web` | Manifest | `application/cbor` |
| `cbor-web/keys.cbor` | Publisher key set | `application/cbor` |
| `cbor-web/bundle` | Bundle | `application/cbor` |
| `cbor-web/pages/*.cbor` | Individual pages | `application/cbor` |

### 14.3 DNS Record Types

| Record | Format | Example |
|--------|--------|---------|
| `_cbor-web.{domain}` TXT | `v=1; alg=EdDSA; key=BASE64URL` | Publisher signing key |
| `_cbor-web-verify.{domain}` TXT | `v=1; endpoint=https://verify.example.com` | Verifier endpoint |

---

## 15. Implementation Guidance

### 15.1 For Publishers

**S0 (Open):**
1. Implement binary content protection (Layer 5) — parsing limits in your CBOR encoder
2. Serve CBOR-Web over HTTPS
3. That's it. No identity, no PoW, no signing.

**S1 (Standard):**
1. Add COSE signing to your manifest (Layer 4)
2. Publish your key set at `/.well-known/cbor-web/keys.cbor`
3. Implement behavioral analysis (Layer 3) — at minimum, rate limiting per DID

**S2 (Protected):**
1. Require DID identity (Layer 1) for non-anonymous access
2. Implement PoW challenges for heavy operations (Layer 2)
3. Sanitize all content (Layer 7)
4. Start logging in CBOR format (Layer 8)

**S3 (Fortress):**
1. Implement all 8 layers
2. Sign individual pages, not just the manifest
3. Use hash-chained logs with Tangle anchoring
4. Connect to the Verifier Network
5. Implement key rotation with DNS DANE records

### 15.2 For Agents

**Minimum (any security level):**
1. Enforce all Layer 5 parsing limits — this protects YOU
2. Validate URLs before following them
3. Use parameterized queries for all CBOR-Web data

**Standard:**
1. Implement DID generation and VC issuance
2. Implement PoW computation
3. Verify COSE signatures on manifests
4. Monitor your trust score via response headers

**Full:**
1. Sandbox all executable block execution (Layer 6)
2. Implement the full authentication flow (§13)
3. Delegate signature verification to the Verifier Network when available
4. Cross-validate CBOR-Web content against HTML periodically

---

## Appendix F: CDDL Schema — Security Structures

```cddl
; ══════════════════════════════════════════════════════════
; CBOR-Web Security Architecture v1.0 — CDDL Schema
; Extends CBOR-Web v2.0 CDDL with security structures
; ══════════════════════════════════════════════════════════

; ── Manifest Security Key (Key 10) ──

manifest-security = {
  "security_level" => "S0" / "S1" / "S2" / "S3",
  ? "layers_active" => [+ uint .ge 1 .le 8],
  ? "identity_required" => bool,
  ? "pow_required_for" => [+ tstr],
  ? "signing_algorithm" => "EdDSA" / "ES256" / "ES384",
  ? "public_key_url" => tstr,
  ? "public_key_dns" => tstr,
  ? "verifier_endpoint" => tstr,
  ? "content_safety" => "sanitized" / "raw" / "user_generated",
  ? "sanitization_version" => tstr,
  ? "sanitization_rules" => [+ tstr],
  ? "audit_log_retention_days" => uint,
  * tstr => any
}

; ── Verifiable Credential ──

cbor-web-vc = {
  "type" => "CborWebAgentCredential",
  "version" => uint,
  "issuer" => tstr,                   ; DID of issuer
  "subject" => tstr,                  ; DID of agent
  "issued_at" => #6.1(uint),
  "expires_at" => #6.1(uint),
  "claims" => vc-claims,
  "nonce" => bstr .size 8,
  "proof" => vc-proof,
  * tstr => any
}

vc-claims = {
  "agent_name" => tstr,
  "agent_type" => "crawler" / "commerce" / "generative" / "monitor" / "full",
  ? "capabilities" => [+ tstr],
  ? "organization" => tstr,
  ? "contact" => tstr,
  ? "compliance" => [+ tstr],
  * tstr => any
}

vc-proof = {
  "type" => "Ed25519Signature2020" / "ES256",
  "created" => #6.1(uint),
  "verification_method" => tstr,      ; DID URL to public key
  "proof_value" => bstr,              ; signature bytes
  * tstr => any
}

; ── Proof-of-Work ──

pow-challenge = {
  "type" => "CborWebPoWChallenge",
  "version" => 1,
  "nonce" => bstr .size 8,
  "difficulty" => uint .ge 8 .le 32,
  "algorithm" => "sha256",
  "target_resource" => tstr,
  "issued_at" => #6.1(uint),
  "expires_at" => #6.1(uint),
  "server_id" => tstr,
  * tstr => any
}

pow-proof = {
  "type" => "CborWebPoWProof",
  "challenge_nonce" => bstr .size 8,
  "solution" => uint,
  "hash" => bstr .size 32,
  "computed_at" => #6.1(uint),
  "agent_did" => tstr,
  * tstr => any
}

; ── Ban Response ──

ban-response = {
  "type" => "CborWebBan",
  "reason" => tstr,
  "details" => tstr,
  "trust_score" => 0,
  "ban_duration_seconds" => uint,
  "ban_expires_at" => #6.1(uint),
  ? "appeal" => tstr,
  ? "metrics" => { * tstr => any },
  * tstr => any
}

; ── Key Set ──

cbor-web-key-set = {
  "type" => "CborWebKeySet",
  "keys" => [+ cbor-web-key],
  * tstr => any
}

cbor-web-key = {
  "kid" => tstr,                      ; key identifier
  "kty" => "OKP" / "EC",             ; key type
  "crv" => "Ed25519" / "P-256" / "P-384",
  "x" => bstr,                        ; public key bytes
  "use" => "sig",
  "valid_from" => #6.1(uint),
  "valid_until" => #6.1(uint),
  * tstr => any
}

; ── Access Log ──

access-log-entry = {
  "type" => "CborWebAccessLog",
  "version" => 1,
  "timestamp" => #6.1(uint),
  "server" => tstr,
  "agent" => {
    ? "did" => tstr,
    "ip" => tstr,
    ? "user_agent" => tstr,
    "trust_score" => uint .ge 0 .le 100,
    * tstr => any
  },
  "request" => {
    "method" => tstr,
    "path" => tstr,
    ? "headers" => { * tstr => tstr },
    * tstr => any
  },
  "response" => {
    "status" => uint,
    "size" => uint,
    ? "content_type" => tstr,
    ? "encoding" => tstr,
    ? "compressed_size" => uint,
    * tstr => any
  },
  ? "security" => { * tstr => any },
  ? "processing_time_ms" => uint,
  ? "prev_hash" => bstr .size 32,     ; hash chain (S3)
  * tstr => any
}

; ── Verifier Network ──

verifier-registration = {
  "type" => "CborWebVerifierRegistration",
  "did" => tstr,
  "services" => [+ "did_verification" / "pow_validation" / "signature_check" /
                    "behavioral_analysis" / "content_audit" / "registry_operation"],
  "endpoint" => tstr,
  "stake" => uint,                    ; IOTA Mi staked
  ? "availability" => float,          ; SLA (0.0-1.0)
  ? "region" => tstr,                 ; ISO 3166-1 alpha-2
  "registered_at" => #6.1(uint),
  * tstr => any
}

verify-request = {
  "type" => "CborWebVerifyRequest",
  "request_id" => tstr,
  "service" => tstr,
  "payload" => { * tstr => any },
  ? "payment" => {
    "amount_mi" => uint,
    "payer_did" => tstr,
    "transaction_hash" => tstr,
    * tstr => any
  },
  * tstr => any
}

verify-response = {
  "type" => "CborWebVerifyResponse",
  "request_id" => tstr,
  "result" => "valid" / "invalid" / "error",
  ? "confidence" => float,
  ? "details" => { * tstr => any },
  "verifier_did" => tstr,
  "verifier_signature" => bstr,
  "timestamp" => #6.1(uint),
  * tstr => any
}

; ── Executable Trust Origin ──

executable-trust-origin = "ps" / "cv" / "uv"
; ps = publisher_signed
; cv = community_verified
; uv = unverified

; ── Capability Model (for executable blocks) ──

capability-model = {
  ? "read_network" => bool,
  ? "write_network" => bool,
  ? "read_filesystem" => bool,
  ? "write_filesystem" => bool,
  ? "spawn_process" => bool,          ; MUST always be denied
  ? "access_env" => bool,
  ? "use_crypto" => bool,
  ? "use_time" => bool,
  * tstr => bool
}
```

---

## Appendix G: Test Vectors — Security

### G.1 Verifiable Credential (Minimal)

```cbor-diag
{
  "type": "CborWebAgentCredential",
  "version": 1,
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "subject": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issued_at": 1(1742515200),
  "expires_at": 1(1742601600),
  "claims": {
    "agent_name": "TestAgent/1.0",
    "agent_type": "crawler"
  },
  "nonce": h'0102030405060708',
  "proof": {
    "type": "Ed25519Signature2020",
    "created": 1(1742515200),
    "verification_method": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "proof_value": h'DEADBEEF00000000DEADBEEF00000000DEADBEEF00000000DEADBEEF00000000DEADBEEF00000000DEADBEEF00000000DEADBEEF00000000DEADBEEF00000000'
  }
}
```

### G.2 PoW Challenge and Solution

**Challenge:**
```cbor-diag
{
  "type": "CborWebPoWChallenge",
  "version": 1,
  "nonce": h'A1B2C3D4E5F6A7B8',
  "difficulty": 16,
  "algorithm": "sha256",
  "target_resource": "/.well-known/cbor-web/bundle",
  "issued_at": 1(1742515200),
  "expires_at": 1(1742515260),
  "server_id": "test.example"
}
```

**Proof (16-bit difficulty = 2 leading zero bytes):**
```cbor-diag
{
  "type": "CborWebPoWProof",
  "challenge_nonce": h'A1B2C3D4E5F6A7B8',
  "solution": 42719,
  "hash": h'0000E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852',
  "computed_at": 1(1742515203),
  "agent_did": "did:key:z6MkTest..."
}
```

Verification: `SHA-256(h'A1B2C3D4E5F6A7B8' || h'000000000000A6DF') = 0000E3B0...` (first 2 bytes are 0x00).

### G.3 Manifest with Security Declaration

```cbor-diag
55799({
  0: "cbor-web-manifest",
  1: 2,
  2: {
    "domain": "verdetao.com",
    "name": "Verdetao",
    "languages": ["fr", "es"],
    "default_language": "fr"
  },
  3: [
    {"path": "/", "title": "Accueil", "lang": "fr", "size": 1500, "hash": h'AAAA...'}
  ],
  5: {
    "generated_at": 1(1742515200),
    "total_pages": 1,
    "total_size": 1500,
    "bundle_available": true,
    "bundle_url": "/.well-known/cbor-web/bundle"
  },
  6: [
    h'A10127',
    {},
    null,
    h'ED25519_SIGNATURE_64_BYTES'
  ],
  7: {
    "static_content": true,
    "commerce": {"available": true, "currencies": ["EUR"]}
  },
  10: {
    "security_level": "S2",
    "layers_active": [1, 2, 3, 4, 5, 7],
    "identity_required": true,
    "pow_required_for": ["bundle"],
    "signing_algorithm": "EdDSA",
    "public_key_url": "/.well-known/cbor-web/keys.cbor",
    "content_safety": "sanitized",
    "audit_log_retention_days": 180
  }
})
```

---

## Appendix H: Security Level Decision Tree

```
START
  │
  ├── Is the content public, non-commercial, no user data?
  │     YES → S0 (Open)
  │     NO ↓
  │
  ├── Is there commercial value (products, prices, competitive data)?
  │     YES ↓
  │     NO → Does it contain user-generated content?
  │            YES → S1 (Standard)
  │            NO → S0 (Open)
  │
  ├── Are there financial transactions (cart, checkout, payments)?
  │     YES → S2 (Protected) minimum
  │     NO ↓
  │
  ├── Does it contain executable blocks or write-access APIs?
  │     YES → S2 (Protected) minimum
  │     NO → S1 (Standard)
  │
  ├── Is regulatory compliance required (GDPR, HIPAA, PCI-DSS)?
  │     YES → S3 (Fortress)
  │     NO ↓
  │
  ├── Is the content health, financial, or government-related?
  │     YES → S3 (Fortress)
  │     NO → S2 (Protected)
  │
  END
```

---

## References

### Normative References

- **[RFC 2119]** Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, March 1997.
- **[RFC 8174]** Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, May 2017.
- **[RFC 8949]** Bormann, C. and P. Hoffman, "Concise Binary Object Representation (CBOR)", STD 94, December 2020.
- **[RFC 9052]** Schaad, J., "CBOR Object Signing and Encryption (COSE): Structures and Process", STD 96, August 2022.
- **[RFC 9053]** Schaad, J., "CBOR Object Signing and Encryption (COSE): Initial Algorithms", August 2022.
- **[RFC 8610]** Birkholz, H., et al., "Concise Data Definition Language (CDDL)", June 2019.
- **[RFC 6455]** Fette, I. and A. Melnikov, "The WebSocket Protocol", December 2011.
- **[W3C DID]** W3C, "Decentralized Identifiers (DIDs) v1.0", July 2022.
- **[W3C VC]** W3C, "Verifiable Credentials Data Model v2.0", March 2024.

### Informative References

- **[IOTA]** IOTA Foundation, "IOTA Protocol", https://www.iota.org/
- **[WASM]** W3C, "WebAssembly Core Specification", December 2019.
- **[WASI]** Bytecode Alliance, "WebAssembly System Interface", https://wasi.dev/
- **[CBOR-Web v1.0]** ExploDev, "CBOR-Web Specification v1.0", March 2026.
- **[CBOR-Web v2.0]** ExploDev, "CBOR-Web Specification v2.0", March 2026.

---

## Acknowledgments

The CBOR-Web Security Architecture was designed by ExploDev (Eddie Plot and Claude). The eight-layer model draws from decades of network security practice (defense in depth), adapted for the unique challenges of machine-to-machine trust in the age of autonomous AI agents.

The Verifier Marketplace concept — repurposing the global crypto mining infrastructure for useful security work — represents a potential answer to one of blockchain's most persistent criticisms: that proof-of-work is wasted energy. In the CBOR-Web model, every hash verification protects a real transaction between a real agent and a real publisher.

---

*CBOR-Web Security Architecture v1.0 — ExploDev 2026*

*"Trust is the currency of the machine web. This is the mint."*

*"La confiance est la monnaie du web des machines. Voici l'atelier monétaire."*
