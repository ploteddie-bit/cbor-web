# CBOR-Web Security Specification v2.1

**Companion to:** CBOR-Web Core Specification v2.1 (CBOR-WEB-CORE.md)
**Status:** Draft
**Date:** 2026-03-24
**Authors:** Eddie Plot & Claude — Deltopide

---

## 1. Overview

This document defines the security architecture for CBOR-Web: access control, authentication, content integrity, privacy, and threat mitigation. It complements CBOR-WEB-CORE.md and is referenced throughout the core specification.

CBOR-Web security follows one principle: **the level of trust required matches the risk of the action**. Reading a product description requires no identity. Buying a product requires verified identity. Accessing government data requires institutional-grade authentication.

---

## 2. Access Tiers

CBOR-Web defines three access tiers plus one hardcoded prohibition. Tiers are ordered from most restricted (T0) to most open (T2).

### 2.1 Tier Summary

| Tier | Name | Access Level | Authentication Required | Use Cases |
|------|------|-------------|------------------------|-----------|
| **T0** | Institutional | Full — sensitive data, identity-verified actions, delegated purchases | eIDAS 2.0 / X.509 EV / OAuth institutional | Government APIs, medical records, financial KYC, purchasing on behalf of a human |
| **T1** | Authenticated | Depth — premium content, commerce, financial transactions, full API | ERC-20 token badge / API key / OAuth 2.1 M2M | Product data, wholesale pricing, transaction APIs, generative blocks |
| **T2** | Open | Surface — metadata, catalogue, navigation, public pages | None | Search indexing, AI crawling, content discovery (equivalent to robots.txt) |
| **Interdit** | Prohibited | Blocked — hardcoded in protocol, non-configurable | N/A — no access regardless of credentials | Violence, weapons trafficking, exploitation, illegal content |

### 2.2 Tier Declaration

A publisher declares tier requirements per page in the manifest page entry (CBOR-WEB-CORE.md §5.4):

```cbor-diag
{
  "path": "/products/lions-mane",
  "title": "Lion's Mane Details",
  "access": "T2",
  "...": "..."
}
```

The `"access"` field accepts:

| Value | Tier | Meaning |
|-------|------|---------|
| `"T0"` | Institutional | Requires institutional-grade authentication |
| `"T1"` | Authenticated | Requires token, API key, or M2M OAuth |
| `"T2"` | Open | No authentication required |
| `"public"` | Alias for T2 | Backward compatibility with core spec v2.1 |
| `"token"` | Alias for T1 | Backward compatibility with core spec v2.1 |

**Migration from core spec v2.1:** The values `"public"` and `"token"` remain valid and map to T2 and T1 respectively. New implementations SHOULD use `"T0"`, `"T1"`, `"T2"`.

### 2.3 Tier Inheritance

If a page entry does not specify `"access"`, the default is determined by the manifest's security configuration (key 10):

```cbor-diag
10: {
  "default_access": "T2",
  "...": "..."
}
```

If key 10 is absent or `"default_access"` is not specified, the default is `"T2"` (open).

### 2.4 Tier Interdit — Ethical Prohibition

The following content categories are **prohibited by the protocol itself**. This is not a publisher configuration — it is hardcoded into conforming agents and publishers.

A conforming CBOR-Web agent MUST NOT:
- Serve, index, cache, or propagate content that promotes or facilitates: violence against persons, weapons trafficking, child exploitation, human trafficking, or production of illegal substances.

A conforming CBOR-Web publisher MUST NOT:
- Expose such content via CBOR-Web endpoints, regardless of tier level.

A conforming agent that detects prohibited content in a CBOR-Web document MUST:
1. Reject the entire document
2. Log the detection with the site domain and timestamp
3. NOT cache or forward the content

**Implementation:** This prohibition is enforced at the agent level, not the protocol level. The CBOR format itself cannot prevent content — but conforming implementations MUST include content policy checks. The reference implementations (`text2cbor`, `cbor-crawl`) include a content policy filter as a REQUIRED component.

---

## 3. Authentication Mechanisms

CBOR-Web supports multiple authentication mechanisms to maximize adoption while maintaining security guarantees. A publisher declares which mechanisms it accepts in the manifest security configuration (key 10).

### 3.1 Mechanism Registry

| ID | Mechanism | Tier | Standard | Maturity |
|----|-----------|------|----------|----------|
| `"eidas2"` | eIDAS 2.0 EUDI Wallet | T0 | EU 2024/1183, OpenID4VC, ISO mDL | Mandatory EU Dec 2026 |
| `"did"` | W3C Decentralized Identifier | T0 | W3C DID Core 1.0 (2022), Verifiable Credentials | Standard W3C |
| `"x509"` | X.509 Client Certificate (EV/OV) | T0 | RFC 5280, TLS 1.3 | Mature (30+ years) |
| `"oauth2_institutional"` | OAuth 2.1 via institutional IdP | T0 | RFC 6749, RFC 9068, OpenID Connect | Mature |
| `"erc20"` | ERC-20 Token Badge (CBORW) | T1 | EIP-20, EIP-712 | Designed for CBOR-Web |
| `"oauth2_m2m"` | OAuth 2.1 Client Credentials | T1 | RFC 6749 §4.4, RFC 9068 | Mature, MCP standard |
| `"apikey"` | API Key (bearer token) | T1 | RFC 6750 | Universal, simple |
| `"none"` | No authentication | T2 | — | — |

### 3.2 Manifest Security Configuration (Key 10)

```cbor-diag
10: {
  "auth_mechanisms": ["erc20", "apikey", "oauth2_m2m"],
  "default_access": "T2",
  "erc20": {
    "chain": "ethereum",
    "contract_address": "0x...",
    "min_balance": 1
  },
  "oauth2_m2m": {
    "issuer": "https://auth.example.com",
    "token_endpoint": "https://auth.example.com/oauth/token",
    "scopes": ["cbor-web:read", "cbor-web:commerce"]
  },
  "apikey": {
    "registration_url": "https://example.com/developer/api-keys"
  },
  "t0_mechanisms": ["eidas2", "x509"],
  "eidas2": {
    "accepted_countries": ["FR", "DE", "ES", "IT", "NL", "BE"],
    "required_attributes": ["family_name", "given_name", "date_of_birth"]
  },
  "rate_limits": {
    "T0": {"requests_per_second": 100},
    "T1": {"requests_per_second": 50},
    "T2": {"requests_per_second": 10}
  }
}
```

**CDDL:**

```cddl
manifest-security = {
  "auth_mechanisms" => [+ auth-mechanism-id],
  ? "default_access" => "T0" / "T1" / "T2",
  ? "erc20" => erc20-config,
  ? "oauth2_m2m" => oauth2-config,
  ? "apikey" => apikey-config,
  ? "t0_mechanisms" => [+ auth-mechanism-id],
  ? "eidas2" => eidas2-config,
  ? "x509" => x509-config,
  ? "rate_limits" => { * tstr => rate-limit-info },
  * tstr => any
}

auth-mechanism-id = "eidas2" / "x509" / "oauth2_institutional" /
                    "erc20" / "oauth2_m2m" / "apikey" / "none" / tstr

erc20-config = {
  "chain" => tstr,
  "contract_address" => tstr,
  ? "min_balance" => uint,
  * tstr => any
}

oauth2-config = {
  "issuer" => tstr,
  ? "token_endpoint" => tstr,
  ? "scopes" => [+ tstr],
  * tstr => any
}

apikey-config = {
  ? "registration_url" => tstr,
  * tstr => any
}

eidas2-config = {
  ? "accepted_countries" => [+ tstr],
  ? "required_attributes" => [+ tstr],
  * tstr => any
}

x509-config = {
  ? "accepted_cas" => [+ tstr],
  ? "min_validation_level" => "DV" / "OV" / "EV",
  * tstr => any
}
```

---

## 4. T0 — Institutional Authentication

T0 protects sensitive data and actions where legal identity is required: government APIs, medical records, financial KYC, or an AI agent purchasing on behalf of a human.

### 4.1 eIDAS 2.0 (EUDI Wallet)

The European Digital Identity Wallet (EUDI) will be mandatory for all EU member states by December 2026 (Regulation EU 2024/1183). It allows citizens to store verified attributes (identity, diplomas, licenses) in a wallet on their device.

**For CBOR-Web T0:**

An AI agent acting on behalf of a human presents the human's EUDI Wallet credentials. The flow:

```
1. Agent requests T0 page
2. Server returns 401 + challenge (OpenID4VP request)
3. Agent forwards challenge to human's EUDI Wallet app
4. Human approves disclosure of required attributes
5. Wallet signs a Verifiable Presentation (VP)
6. Agent presents VP to server
7. Server verifies VP against trusted issuer registry
8. Server grants T0 access for session duration
```

**HTTP headers:**

```
POST /.well-known/cbor-web/auth/eidas2
Content-Type: application/json

{
  "vp_token": "<Verifiable Presentation JWT>",
  "presentation_submission": { ... }
}
```

**Response (success):**

```
HTTP/1.1 200 OK
Content-Type: application/json

{
  "access_token": "cbw_t0_...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "tier": "T0"
}
```

The returned `access_token` is a session bearer token valid for 1 hour. Subsequent requests use:

```
Authorization: Bearer cbw_t0_...
```

**Key constraint:** The human MUST approve each disclosure. The AI agent cannot bypass consent. This aligns with eIDAS 2.0's user-centric design.

**Supported protocols:** OpenID for Verifiable Presentations (OpenID4VP), ISO/IEC 18013-5 (mDL), W3C Verifiable Credentials Data Model 2.0.

### 4.2 W3C DID (Decentralized Identifiers)

DID (W3C Recommendation, juillet 2022) fournit une identité décentralisée vérifiable sans autorité centrale. Un agent ou un publisher est identifié par un DID, et prouve son identité via un Verifiable Credential (VC).

**Méthodes DID supportées pour T0 :**

| Méthode | Format | Résolution | Usage |
|---------|--------|-----------|-------|
| `did:web` | `did:web:agent.deltopide.com` | HTTPS (fichier `.well-known/did.json`) | Agents liés à un domaine |
| `did:ethr` | `did:ethr:0x742d35Cc...` | Ethereum blockchain | Agents avec wallet Ethereum |
| `did:key` | `did:key:z6Mkf5rG...` | Auto-contenu (clé dans l'identifiant) | Agents autonomes sans domaine |

**Flow d'authentification T0 via DID :**

```
1. Agent présente son DID : did:web:agent.deltopide.com
2. Server résout le DID → récupère le DID Document (clé publique)
3. Agent signe un challenge avec sa clé privée
4. Server vérifie la signature contre la clé du DID Document
5. Agent présente un Verifiable Credential (ex: "organisation gouvernementale")
6. Server vérifie le VC contre l'émetteur de confiance
7. Si valide → accès T0
```

**HTTP headers :**

```
GET /index.cbor HTTP/1.1
Host: fleurs.com
Accept: application/cbor
X-CBOR-Web-Auth: did
X-CBOR-Web-DID: did:web:agent.deltopide.com
X-CBOR-Web-DID-Sig: <signature du challenge>
X-CBOR-Web-VC: <Verifiable Credential JWT>
```

**Manifest configuration (key 10) :**

```cbor-diag
"did": {
  "accepted_methods": ["did:web", "did:ethr", "did:key"],
  "trusted_issuers": [
    "did:web:franceconnect.gouv.fr",
    "did:web:eidas.europa.eu",
    "did:web:deltopide.com"
  ],
  "required_vc_types": ["GovernmentOrganization", "VerifiedAgent"]
}
```

**CDDL :**

```cddl
did-config = {
  ? "accepted_methods" => [+ tstr],
  ? "trusted_issuers" => [+ tstr],
  ? "required_vc_types" => [+ tstr],
  * tstr => any
}
```

**Compatibilité eIDAS 2.0 :** L'EUDI Wallet peut émettre des Verifiable Credentials au format W3C. Un agent authentifié via eIDAS peut aussi présenter son credential en DID — les deux mécanismes sont complémentaires, pas concurrents.

**Avantage DID pour les agents IA :** Un agent IA autonome (sans humain derrière) peut posséder un DID propre (`did:key:...`). Contrairement à eIDAS (conçu pour les humains), DID est nativement compatible avec les identités machines. Un agent peut prouver qu'il est bien "l'agent X de l'organisation Y" sans qu'un humain intervienne dans la boucle d'authentification.

### 4.3 X.509 Client Certificates

For non-EU institutional access, X.509 client certificates (TLS mutual authentication) provide equivalent trust.

**Requirements:**
- Certificate MUST be Extended Validation (EV) or Organization Validation (OV)
- Domain Validation (DV) certificates are NOT sufficient for T0
- Certificate MUST chain to a CA trusted by the server
- Server MAY restrict to specific CAs (declared in key 10 `"x509"."accepted_cas"`)

**Flow:** Standard TLS client authentication (RFC 8446 §4.4.2). No CBOR-Web-specific logic.

```
Client                                  Server
  |                                      |
  |  TLS ClientHello                     |
  |─────────────────────────────────────>|
  |                                      |
  |  TLS ServerHello + CertificateRequest|
  |<─────────────────────────────────────|
  |                                      |
  |  TLS Certificate (client EV cert)    |
  |  TLS CertificateVerify              |
  |─────────────────────────────────────>|
  |                                      |
  |  TLS Finished (mutual auth OK)       |
  |<─────────────────────────────────────|
  |                                      |
  |  GET /.well-known/cbor-web/pages/... |
  |─────────────────────────────────────>|
  |  200 OK (T0 content)                 |
  |<─────────────────────────────────────|
```

### 4.3 OAuth 2.1 Institutional

For government platforms (France Connect, Itsme, BankID, etc.):

**Flow:** OAuth 2.1 Authorization Code with PKCE (human-in-the-loop for consent), then the resulting token is used for M2M access.

The publisher declares its accepted institutional IdPs in key 10:

```cbor-diag
"oauth2_institutional": {
  "accepted_issuers": [
    "https://franceconnect.gouv.fr",
    "https://login.microsoftonline.com/{tenant}",
    "https://accounts.google.com"
  ]
}
```

---

## 5. T1 — Authenticated Access

T1 protects premium content and enables commerce. Three authentication mechanisms, all equally valid:

### 5.1 ERC-20 Token Badge (CBORW)

The CBOR-Web token is an ERC-20 utility token on Ethereum mainnet. Holding ≥1 token grants T1 access across all participating sites.

**The token is a badge, not a currency.** It is not spent per request. An agent that holds 1 token has permanent T1 access until it transfers the token away.

#### 5.1.1 Authentication Protocol

```
1. Agent signs request hash with wallet private key (EIP-712)
2. Server verifies signature → recovers wallet address
3. Server queries ERC-20 contract: balanceOf(wallet) >= 1?
4. If yes → T1 access. If no → 402 Payment Required.
```

#### 5.1.2 EIP-712 Typed Data Signature

The agent signs a structured message containing:

```json
{
  "types": {
    "EIP712Domain": [
      {"name": "name", "type": "string"},
      {"name": "version", "type": "string"},
      {"name": "chainId", "type": "uint256"}
    ],
    "CBORWebRequest": [
      {"name": "domain", "type": "string"},
      {"name": "path", "type": "string"},
      {"name": "nonce", "type": "uint256"},
      {"name": "timestamp", "type": "uint256"}
    ]
  },
  "primaryType": "CBORWebRequest",
  "domain": {
    "name": "CBOR-Web",
    "version": "2.1",
    "chainId": 1
  },
  "message": {
    "domain": "verdetao.com",
    "path": "/.well-known/cbor-web/pages/products_lions-mane.cbor",
    "nonce": 42,
    "timestamp": 1742598400
  }
}
```

#### 5.1.3 HTTP Headers

**Request:**

```
GET /.well-known/cbor-web/pages/products_lions-mane.cbor HTTP/1.1
Host: verdetao.com
Accept: application/cbor
X-CBOR-Web-Auth: erc20
X-CBOR-Web-Wallet: 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18
X-CBOR-Web-Sig: 0x<EIP-712 signature hex>
X-CBOR-Web-Nonce: 42
X-CBOR-Web-Timestamp: 1742598400
```

**Server validation:**

1. Verify timestamp is within 300 seconds of server time (replay window)
2. Recover signer address from EIP-712 signature
3. Verify recovered address matches `X-CBOR-Web-Wallet`
4. Query `balanceOf(wallet)` on ERC-20 contract (cache ≤ 60 seconds)
5. If balance ≥ `min_balance` (default 1) → grant T1 access

**Response (success):**

```
HTTP/1.1 200 OK
Content-Type: application/cbor
X-CBOR-Web-Verified: true
X-CBOR-Web-Token-Balance: 3
```

**Response (insufficient tokens):**

```
HTTP/1.1 402 Payment Required
Content-Type: application/cbor

{
  "error": "token_required",
  "message": "This page requires a CBOR-Web token (T1 access).",
  "contract_address": "0x...",
  "chain": "ethereum",
  "get_token_url": "https://cbor-web.org/get-token",
  "alternative_auth": ["apikey", "oauth2_m2m"]
}
```

Note: The `"alternative_auth"` field tells the agent that other T1 mechanisms are available, so it does not need to acquire a token if it has an API key or OAuth credentials.

### 5.2 API Key (Bearer Token)

For developers who prefer simplicity over blockchain:

```
GET /.well-known/cbor-web/pages/products_lions-mane.cbor HTTP/1.1
Host: verdetao.com
Accept: application/cbor
Authorization: Bearer cbw_k1_a3f2c442...
```

The publisher issues API keys via a registration endpoint (declared in key 10 `"apikey"."registration_url"`). API keys are opaque strings prefixed with `cbw_k1_` for identification.

**Validation:** Server checks the key against its database. No blockchain involved.

**Trade-offs vs ERC-20:**

| Aspect | ERC-20 Token | API Key |
|--------|-------------|---------|
| Cross-site | Yes (one token, all sites) | No (one key per site) |
| Decentralized | Yes (blockchain) | No (server database) |
| Setup effort | Requires Ethereum wallet | HTTP registration form |
| Revocation | Transfer token away | Publisher revokes key |
| Privacy | Wallet is cross-site identifier | Key is site-specific |

### 5.3 OAuth 2.1 Machine-to-Machine

For enterprise integrations:

```
POST /oauth/token HTTP/1.1
Host: auth.example.com
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
&client_id=cbor_agent_42
&client_secret=<secret>
&scope=cbor-web:read cbor-web:commerce
```

Response:

```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "cbor-web:read cbor-web:commerce"
}
```

Then:

```
GET /.well-known/cbor-web/pages/products_lions-mane.cbor HTTP/1.1
Authorization: Bearer eyJ...
```

This follows the standard OAuth 2.1 Client Credentials flow (RFC 6749 §4.4). It is the same mechanism used by MCP servers (Model Context Protocol) for AI agent authentication.

---

## 6. T2 — Open Access

T2 requires no authentication. An agent requests content with no special headers:

```
GET /.well-known/cbor-web HTTP/1.1
Host: example.com
Accept: application/cbor
User-Agent: cbor-crawl/1.0.0
```

The server SHOULD include a `User-Agent` for identification and rate limiting purposes, but it is not required.

T2 pages are equivalent to what Google sees when it crawls a site. The manifest, page metadata, and public pages are all T2.

**Rate limiting for T2:** Servers SHOULD enforce rate limits for anonymous agents (declared in key 10 `"rate_limits"."T2"`). Default: 10 requests/second per IP.

---

## 7. Content Integrity

### 7.1 Hash Verification

Every page in the manifest includes a SHA-256 hash (CBOR-WEB-CORE.md §10.2). An agent MUST verify the hash after downloading a page:

```python
page_bytes = fetch(page_url)
assert page_bytes[0:3] == b'\xD9\xD9\xF7'  # self-described CBOR
computed = sha256(page_bytes)
assert computed == manifest_entry.hash
```

If the hash does not match:
1. Reject the page — do NOT process content from an unverified page
2. Re-fetch the manifest (it may be stale)
3. If the manifest hash also fails → treat the site as compromised

### 7.2 Manifest Signature (COSE_Sign1)

At Full conformance (CBOR-WEB-CORE.md §11.3), the manifest MUST be signed using COSE_Sign1 (RFC 9052).

Manifest key 6 contains a **byte string** wrapping a serialized COSE_Sign1 structure:

```cbor-diag
6: h'<serialized COSE_Sign1>'
```

The signature covers manifest keys 0-5 and 7-10 (everything except key 6 itself).

#### 7.2.1 Signing Algorithm

```
payload = canonical_cbor_encode({
  0: manifest[0],  ; @type
  1: manifest[1],  ; @version
  2: manifest[2],  ; site
  3: manifest[3],  ; pages
  5: manifest[5],  ; meta
  7: manifest[7],  ; capabilities (if present)
  8: manifest[8],  ; channels (if present)
  9: manifest[9],  ; diff (if present)
  10: manifest[10] ; security (if present)
})

signature = COSE_Sign1(
  protected: {1: -7},       ; alg: ES256 (ECDSA with P-256 + SHA-256)
  unprotected: {4: key_id}, ; kid: key identifier
  payload: payload
)

manifest[6] = serialize(signature)  ; store as bstr
```

**Required algorithm:** ES256 (ECDSA with NIST P-256 curve, SHA-256 hash). This is the most widely supported COSE algorithm, used in WebAuthn, FIDO2, and passkeys.

#### 7.2.2 Key Discovery

The publisher's public key is discoverable via:

1. **Keys endpoint (preferred):**
   ```
   GET /.well-known/cbor-web/keys.cbor
   ```
   Returns a COSE_KeySet containing the publisher's public key(s).

2. **DNS TXT record (fallback):**
   ```
   _cbor-web.example.com TXT "v=2.1; kid=key-2026-03; pk=<base64url-encoded-public-key>"
   ```

3. **Key 10 inline (minimal):**
   ```cbor-diag
   10: {
     "signing_key": h'<COSE_Key bytes>',
     "...": "..."
   }
   ```

#### 7.2.3 Key Rotation

A publisher SHOULD rotate signing keys at least annually. The rotation process:

1. Generate new key pair
2. Add new public key to keys.cbor (alongside old key)
3. Start signing new manifests with new key
4. After 30 days, remove old key from keys.cbor

Agents SHOULD cache public keys for up to 7 days (the Cache-Control for keys.cbor, see CBOR-WEB-CORE.md §9.6).

### 7.3 Content Cross-Validation

At Full conformance, an agent SHOULD periodically compare CBOR-Web content against the HTML version:

1. Fetch the HTML page at the same path
2. Extract visible text content from HTML
3. Compare with CBOR-Web page content (key 4 blocks)
4. If divergence > 20% by word count → log a warning

This detects:
- Stale CBOR-Web content (publisher forgot to regenerate)
- Malicious content injection (CBOR-Web serves different content than HTML)
- Translation drift (CBOR-Web and HTML in different languages)

---

## 8. Trust Levels

Content blocks in CBOR-Web carry implicit or explicit trust levels that determine how an agent should process them.

### 8.1 Trust Level Definitions

| Level | Name | Description | Agent Behavior |
|-------|------|-------------|---------------|
| 0 | Declarative | Pure data, no side effects. Safe to process without sandboxing. | Process directly |
| 1 | Template | Contains variable substitution. Output depends on context. | Validate template syntax before rendering |
| 2 | Executable | Contains code that can run in a sandbox. Potential side effects. | Execute ONLY in isolated sandbox |
| 3 | Interactive | Requires network access or user interaction. Full side effects. | Execute ONLY with explicit agent policy and user consent |

### 8.2 Block Type Trust Assignments

**Core blocks (CBOR-WEB-CORE.md §8):**
All core blocks are trust level 0 (implicit — no `"trust"` key needed).

**Multimedia blocks (CBOR-WEB-MULTIMEDIA.md):**
All multimedia blocks are trust level 0.

**Generative blocks (CBOR-WEB-GENERATIVE.md):**

| Block Type | Trust Level | Rationale |
|-----------|-------------|-----------|
| `"schema"` | 0 | Pure data description |
| `"constraint"` | 0 | Declarative business rule |
| `"product"` | 0 | Product data |
| `"template"` | 1 | Variable substitution |
| `"executable"` | 2 | Sandboxed code execution |
| `"api_endpoint"` | 3 | Network access required |
| `"workflow"` | 3 | Multi-step with side effects |
| `"form"` | 3 | User data submission |
| `"cart_action"` | 3 | Financial transaction |

### 8.3 Agent Trust Policy

An agent MUST define a trust policy before processing content:

```
Example policy:
  trust_level_0: always process
  trust_level_1: process if template syntax validates
  trust_level_2: process only in Docker/Wasm sandbox
  trust_level_3: process only with explicit human approval
```

An agent MUST NOT process a block at a trust level it does not support. Unknown trust levels MUST be treated as level 3 (maximum restriction).

---

## 9. Privacy

### 9.1 Cross-Site Tracking Risk

The ERC-20 token authentication uses a wallet address that is identical across all CBOR-Web sites. This creates a cross-site tracking risk equivalent to third-party cookies.

### 9.2 Mitigations

| Mitigation | Responsibility | Mechanism |
|------------|---------------|-----------|
| **Dedicated wallets per site** | Agent | Generate a new wallet per publisher domain |
| **Dedicated API keys per site** | Agent | Use `"apikey"` auth (inherently site-specific) |
| **Privacy proxy** | Agent/Service | Third-party service holds token, authenticates on agent's behalf |
| **Minimal logging** | Publisher | MUST NOT log wallet addresses beyond rate limiting needs |
| **No sharing** | Publisher | MUST NOT share authentication identifiers with third parties |
| **Session tokens** | Publisher | Issue short-lived session tokens after initial auth, use session for subsequent requests |

### 9.3 Recommended Agent Behavior

An agent SHOULD:
- Use API key authentication when cross-site privacy matters
- Use a different wallet address per publisher domain when using ERC-20
- Prefer OAuth 2.1 M2M (client credentials are site-specific by nature)

### 9.4 Publisher Privacy Requirements

A publisher MUST:
- NOT log wallet addresses for longer than 24 hours
- NOT correlate wallet addresses across sites
- NOT use CBOR-Web authentication headers for fingerprinting
- NOT sell or share authentication identifiers

A publisher SHOULD:
- Offer API key and OAuth alternatives alongside ERC-20
- Support session tokens to reduce per-request wallet exposure

---

## 10. Threat Model

### 10.1 Threats and Mitigations

| Threat | Description | Mitigation |
|--------|------------|------------|
| **CBOR Bomb** | Malicious manifest declares 10 MB but contains 10 GB of nested data | Agent MUST enforce size limits: manifest ≤ 5 MB, page ≤ 1 MB, bundle ≤ 50 MB. Reject before full parsing. |
| **Manifest Poisoning** | CDN or MITM serves modified manifest with altered hashes | COSE signature verification (§7.2). HTTPS required (CBOR-WEB-CORE.md §9.1). |
| **Content Injection** | CBOR-Web serves different content than HTML (SEO cloaking for AI) | Content cross-validation (§7.3). |
| **Replay Attack** | Attacker replays a valid signed request | Nonce + timestamp window (300 seconds). Server rejects stale nonces. |
| **Token Theft** | Attacker steals wallet private key | Standard Ethereum key management. Hardware wallets recommended for high-value agents. |
| **API Key Leak** | API key exposed in logs or code | Keys are revocable. Publishers MUST support key rotation. |
| **DoS via Manifest** | Giant manifest (millions of pages) exhausts agent memory | Agent MUST reject manifests > 5 MB. Sub-manifest pagination (CBOR-WEB-CORE.md §5.8). |
| **Malicious Links** | External links in page content point to malicious sites | Agent MUST NOT auto-follow external links without explicit policy (CBOR-WEB-CORE.md §6.7). |
| **Cross-Site Tracking** | Wallet address used to track agent across sites | Privacy mitigations (§9.2). |

### 10.2 Size Limits (Enforcement)

An agent MUST enforce these limits BEFORE parsing:

| Document | Max Size | Action on Exceed |
|----------|---------|-----------------|
| Manifest | 5 MB | Reject entirely |
| Page | 1 MB | Reject page, continue with others |
| Bundle | 50 MB | Reject, fall back to individual pages |
| Sub-manifest page | 5 MB | Reject page, try next |
| Inline image | 10 KB | Skip image block |
| Content block text | 50,000 chars | Truncate with warning |

---

## 11. External Link Security

### 11.1 Policy

Links in page content (key 5) are informational. An agent MUST NOT automatically follow external links without an explicit security policy.

### 11.2 Agent Link Policy

An agent SHOULD define a link follow policy:

| Policy Level | Behavior |
|-------------|----------|
| **Strict** | Never follow external links |
| **Domain-whitelist** | Follow only links to whitelisted domains |
| **Same-site** | Follow internal links, reject external |
| **Permissive** | Follow all links (NOT RECOMMENDED) |

Default: **Same-site**.

---

## 12. Binary Watermarking (OPTIONAL)

A publisher MAY embed an invisible watermark in CBOR-Web content to trace unauthorized redistribution.

### 12.1 Mechanism

The publisher embeds a unique code at a random position in a text block value. The code is invisible to content processing (zero-width Unicode characters or statistically insignificant word variations).

**HTTP headers:**

```
X-CBOR-Web-Code-Position: 42    ; server tells agent where the watermark is
X-CBOR-Web-Code: <watermark>    ; agent can echo it back to prove authenticity
```

### 12.2 Agent Behavior

An agent SHOULD NOT strip watermarks from CBOR-Web content. An agent that redistributes CBOR-Web content SHOULD preserve watermarks to maintain the chain of trust.

---

## 13. cbor.txt — Access Rules Declaration

### 13.1 Purpose

`cbor.txt` is a plain text file at the root of a website (like `robots.txt`) that declares CBOR-Web access rules. It coexists with `robots.txt` and `/.well-known/cbor-web`.

```
GET /cbor.txt HTTP/1.1
Host: example.com
```

### 13.2 Format

```
# cbor.txt — CBOR-Web access rules
# Version: 2.1

Manifest: /.well-known/cbor-web
Default-Access: T2

# Pages requiring institutional access
T0: /government/*
T0: /medical/*

# Pages requiring authentication
T1: /products/*/pricing
T1: /api/*
T1: /wholesale/*

# Open pages (default, but explicit)
T2: /
T2: /about
T2: /blog/*
T2: /products/*

# Rate limits
Rate-Limit-T0: 100/s
Rate-Limit-T1: 50/s
Rate-Limit-T2: 10/s
```

### 13.3 Relationship with Manifest

`cbor.txt` is a **human-readable declaration** of access rules. The manifest (key 10) is the **machine-readable implementation**. They SHOULD be consistent. If they conflict, the manifest is authoritative.

An agent MAY read `cbor.txt` before fetching the manifest to quickly determine if the site's access model matches its capabilities.

### 13.4 Relationship with robots.txt

`cbor.txt` does NOT replace `robots.txt`. A CBOR-Web agent MUST still respect `robots.txt` directives. The relationship:

| File | Controls | Read By |
|------|---------|---------|
| `robots.txt` | Crawl permissions (what paths can be accessed) | All crawlers |
| `cbor.txt` | CBOR-Web access tiers (what auth is needed) | CBOR-Web agents |
| `/.well-known/cbor-web` | Actual CBOR-Web content and manifest | CBOR-Web agents |

---

## 14. Conformance

### 14.1 Publisher Security Conformance

| Level | Requirements |
|-------|-------------|
| **Minimal** | HTTPS. `"access"` field in page entries. T2 by default. |
| **Standard** | Minimal + key 10 in manifest. SHA-256 hashes. Rate limits declared. At least 2 auth mechanisms for T1. |
| **Full** | Standard + COSE manifest signature. Key discovery endpoint. Content cross-validation support. T0 mechanisms declared. cbor.txt published. |

### 14.2 Agent Security Conformance

| Level | Requirements |
|-------|-------------|
| **Minimal** | Respect access tiers. HTTPS only. Size limits enforced. |
| **Standard** | Minimal + hash verification. Rate limit compliance. Trust level policy. |
| **Full** | Standard + COSE signature verification. Content cross-validation. Privacy mitigations. |

---

## References

- **[RFC 5280]** Cooper, D., et al., "Internet X.509 PKI Certificate and CRL Profile", May 2008.
- **[RFC 6749]** Hardt, D., "The OAuth 2.0 Authorization Framework", October 2012.
- **[RFC 6750]** Jones, M. and D. Hardt, "The OAuth 2.0 Authorization Framework: Bearer Token Usage", October 2012.
- **[RFC 8446]** Rescorla, E., "The Transport Layer Security (TLS) Protocol Version 1.3", August 2018.
- **[RFC 9052]** Schaad, J., "CBOR Object Signing and Encryption (COSE)", August 2022.
- **[RFC 9068]** Bertocci, V., "JSON Web Token (JWT) Profile for OAuth 2.0 Access Tokens", October 2021.
- **[EU 2024/1183]** European Parliament, "European Digital Identity Framework (eIDAS 2)", April 2024.
- **[EIP-20]** Vogelsteller, F. and V. Buterin, "ERC-20 Token Standard", November 2015.
- **[EIP-712]** Steiner, R., et al., "Ethereum typed structured data hashing and signing", September 2017.
- **[W3C DID]** Sporny, M., et al., "Decentralized Identifiers (DIDs) v1.0", W3C Recommendation, July 2022. https://www.w3.org/TR/did-core/
- **[W3C VC]** Sporny, M., et al., "Verifiable Credentials Data Model v2.0", W3C Recommendation, 2024. https://www.w3.org/TR/vc-data-model-2.0/
- **[OpenID4VP]** Terbu, O., et al., "OpenID for Verifiable Presentations", 2023.

---

*CBOR-Web Security Specification v2.1 — Document 2 of 6*

*Deltopide 2026*
