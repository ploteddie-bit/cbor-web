# CBOR-Web Economics Specification v2.1

**Companion to:** CBOR-Web Core Specification v2.1 (CBOR-WEB-CORE.md)
**Status:** Draft
**Date:** 2026-03-24
**Authors:** Eddie Plot — Deltopide

---

## 1. Overview

This document defines the economic model of CBOR-Web: the CBORW utility token, pricing, allocation, smart contract mechanics, and the self-sustaining adoption loop.

**Core principle:** The protocol is free. The spec is open (CC BY-ND 4.0). The tools are open source (MIT). The money comes from the token ecosystem, not from licensing or SaaS.

---

## 2. The Problem CBOR-Web Economics Solves

### 2.1 For AI Agents (Consumers)

| Metric | HTML Today | CBOR-Web |
|--------|-----------|----------|
| Cost per page (LLM tokens) | ~$0.036 | ~$0.002 |
| Cost per 1000 pages | ~$36 | ~$2 |
| Annual cost (1000 pages/day) | ~$13,000 | ~$730 |
| Signal-to-noise ratio | ~7% | >95% |
| Embedding quality | Polluted | Clean signal |

An agent using CBOR-Web saves **$12,000/year** on a moderate workload. The economic incentive to adopt is structural.

### 2.2 For Publishers (Producers)

Today, publishers suffer from AI scraping with zero compensation:
- Bots consume bandwidth without generating ad revenue
- No data on which agents visit or what they read
- No control over content usage
- No monetization of machine-readable content

CBOR-Web gives publishers:
- **Visibility** — the manifest shows agents exactly what's available
- **Control** — tiers determine who sees what
- **Analytics** — authenticated agents (T1) are identifiable
- **Revenue** — token model creates value for premium content

### 2.3 The Paradox

The web needs a standard for machine-readable content. Standards must be free to succeed. But free standards have no funding. The CBOR-Web token resolves this paradox: **the standard is free, the access badge has market value**.

---

## 3. CBORW Token

### 3.1 Token Specification

| Property | Value |
|----------|-------|
| Name | CBOR-Web Token |
| Symbol | CBORW |
| Standard | ERC-20 (EIP-20) |
| Blockchain | Ethereum mainnet |
| Total supply | 100,000,000 (100M) |
| Decimals | 18 |
| Mintable | No (fixed supply at deployment) |
| Burnable | No |
| Upgradeable | No (immutable contract) |

### 3.2 Token Function

The CBORW token is a **badge**, not a currency.

| What it IS | What it is NOT |
|-----------|----------------|
| A permanent access badge | A currency spent per request |
| Proof of participation in the ecosystem | A payment for content |
| A verifiable credential on-chain | A subscription that expires |
| An alignment incentive (skin in the game) | A speculative investment vehicle |

**Holding ≥1 CBORW token grants T1 access across ALL participating CBOR-Web sites.** The token is never consumed. The agent keeps it as long as it holds it in its wallet.

### 3.3 Why a Token (Not SaaS, Not API Keys, Not Free)

| Model | Problem |
|-------|---------|
| **SaaS subscription** | Centralized, single point of failure, pricing power abuse |
| **Per-site API keys** | Fragmented, no cross-site portability, publisher lock-in |
| **Completely free** | No funding for development, no spam prevention, no accountability |
| **ERC-20 token** | Decentralized, cross-site, verifiable, self-funding, spam-resistant |

The token creates **alignment**: agents that hold tokens benefit from a growing ecosystem. Publishers that support CBOR-Web attract more agents. More agents increase token demand. Token value funds development.

---

## 4. Token Allocation

| Allocation | Tokens | Percentage | Vesting | Purpose |
|-----------|--------|------------|---------|---------|
| Founder (Deltopide) | 20,000,000 | 20% | 4 years, 1-year cliff | Long-term alignment, development funding |
| Ecosystem Rewards | 40,000,000 | 40% | Released over 10 years | Publisher rewards, verifier rewards, integration bounties |
| Community & Grants | 20,000,000 | 20% | Released on milestones | Developer grants, hackathons, academic research |
| Development Fund | 10,000,000 | 10% | Released quarterly | Engineering salaries, infrastructure, security audits |
| Strategic Reserve | 10,000,000 | 10% | Locked 2 years | Partnerships, exchange listings, emergency fund |

### 4.1 Founder Vesting

The founder allocation (20M tokens) follows a standard 4-year vesting schedule with a 1-year cliff:
- Month 0-12: 0 tokens accessible (cliff)
- Month 12: 5M tokens unlock (25%)
- Month 13-48: ~416,666 tokens/month (linear vesting)
- Month 48: Fully vested

This ensures the founder is committed for at least 4 years and cannot dump tokens early.

### 4.2 Ecosystem Rewards

Tokens distributed to:
- **Publishers** who deploy CBOR-Web on their sites (proof of deployment)
- **Verifiers** who run signature verification nodes
- **Integration builders** who create tools, libraries, plugins
- **Early adopters** (first 1000 sites, first 500 agents)

Distribution mechanism: quarterly claims via smart contract. Proof of activity required (on-chain verification of CBOR-Web manifest signatures, page hashes on IPFS, or similar).

---

## 5. Pricing Model

### 5.1 Token Price

The initial token price is **$0.01** per CBORW.

At initial price:
- 1 token = $0.01 = permanent T1 access
- 3 tokens = $0.03 = comfortable holding (sell 1 if price rises, keep 2)

### 5.2 Why Fixed Initial Price (Not AMM)

Phase 1 (launch → 1000 token holders):
- **Direct sales** from founder wallet at fixed $0.01
- No Uniswap pool, no AMM, no trading
- Goal: distribute tokens to real users (publishers + agents), not speculators

Phase 2 (after 1000 holders):
- Create Uniswap V3 pool
- Initial liquidity: 1M CBORW + equivalent ETH from Phase 1 sales
- Price discovery by market

### 5.3 Smart Contract Auto-Regulation

The smart contract includes a stabilization mechanism:

```solidity
// Simplified — actual contract will be audited
function stabilize() external {
    uint256 currentPrice = getUniswapPrice();
    uint256 targetPrice = getTargetPrice(); // moving average

    if (currentPrice > targetPrice * 150 / 100) {
        // Price too high — release tokens from reserve to market
        uint256 amount = calculateReleaseAmount();
        _transfer(reserveWallet, uniswapPool, amount);
    }

    if (currentPrice < targetPrice * 50 / 100) {
        // Price too low — buy back from market
        uint256 amount = calculateBuybackAmount();
        buyFromUniswap(amount);
    }
}
```

**Goal:** Organic growth tied to real usage, not speculative pumps. The token should appreciate gradually as adoption increases, not 1000x overnight.

### 5.4 Cost Analysis for Agents

| Scenario | Cost | Savings vs HTML |
|----------|------|-----------------|
| Agent buys 1 token at $0.01 | $0.01 one-time | Permanent T1 access |
| Agent reads 1000 pages/day CBOR-Web | $0 (token badge, not consumed) | vs $36/day HTML ($13K/year) |
| Agent sells token after 1 year (price $0.05) | +$0.04 profit | Net: agent was PAID to use CBOR-Web |

The economic incentive is overwhelming: **CBOR-Web costs less than nothing** for early adopters.

---

## 6. Publisher Economics

### 6.1 Cost to Deploy

| Item | Cost | Time |
|------|------|------|
| Install `text2cbor` | $0 (open source) | 30 minutes |
| Generate CBOR-Web content | $0 (automated) | Automatic |
| Serve /.well-known/cbor-web | $0 (existing server) | 5 minutes config |
| **Total** | **$0** | **< 1 hour** |

### 6.2 Revenue for Publishers

Publishers earn ecosystem reward tokens for:
- Deploying CBOR-Web (proof: manifest accessible at well-known URL)
- Maintaining content freshness (proof: manifest `generated_at` < 7 days)
- Serving authenticated agents (proof: T1 request logs)
- Quality content (proof: cross-validation HTML/CBOR match, low error rate)

Estimated early rewards:
- First 100 publishers: 10,000 CBORW each (1M tokens total)
- Next 900 publishers: 1,000 CBORW each (900K tokens)
- Ongoing: proportional to agent traffic

### 6.3 Indirect Revenue

Beyond tokens, publishers benefit from:
- **Better AI visibility** — agents prefer CBOR-Web sites (cleaner signal = better embeddings = higher ranking in AI answers)
- **Analytics** — know which agents visit, what they read, how often
- **Competitive advantage** — early adopters get the most AI traffic
- **Premium data monetization** — T1 content is accessed only by token holders

---

## 7. Self-Sustaining Loop

```
Publishers deploy CBOR-Web (free)
    ↓
Agents discover cleaner content (200x cheaper than HTML)
    ↓
Agents prefer CBOR-Web sites (organic quality signal)
    ↓
Agents obtain CBORW tokens for T1 access
    ↓
Token demand increases → price rises
    ↓
Publisher rewards increase in value
    ↓
More publishers deploy CBOR-Web
    ↓
More content available → more agents adopt
    ↓
↻ Positive feedback loop
```

**Key insight:** No one is forced to adopt. The format wins because it's objectively better. The token wins because it aligns incentives. The loop sustains itself without mandates, regulations, or marketing budgets.

---

## 8. Launch Plan

### Phase 0 — Foundation (current)
- [x] Spec v2.1 written (6 documents)
- [x] text2cbor reference implementation
- [ ] cbor-crawl reference implementation
- [ ] Site cbor-web.org deployed
- [ ] Spec published on GitHub (explodev/cbor-web)

### Phase 1 — Airdrop (cost: ~$50)
- Deploy ERC-20 smart contract on Ethereum mainnet (~$50 gas)
- 100M tokens created in one transaction
- Airdrop 1000 CBORW to first 50 sites/agents
- Blockchain verification works immediately (`balanceOf > 0`)
- No trading, no Uniswap pool yet

### Phase 2 — Early Adoption (cost: ~$0)
- First 100 publishers deploy CBOR-Web
- cbor-crawl available for agents
- Direct token sales at $0.01 to real users
- Community documentation, tutorials, integrations

### Phase 3 — Market (cost: ~$500 for Uniswap liquidity)
- Create Uniswap V3 pool
- Price discovery begins
- Ecosystem rewards distribution starts
- First automated agents using CBOR-Web in production

### Phase 4 — Scale
- 1000+ sites, 100+ agents
- Token price reflects real usage
- Stabilization mechanism active
- IANA registration of well-known URI
- eIDAS 2.0 T0 integrations

---

## 9. Legal and Compliance

### 9.1 Token Classification

The CBORW token is a **utility token**, not a security:
- It grants access to a service (T1 content reading)
- It is not an investment contract
- There is no expectation of profit from the efforts of others (the token's value depends on ecosystem adoption, not a company's performance)
- It is consumed functionally (access badge), not speculatively

### 9.2 Regulatory Considerations

| Jurisdiction | Framework | Classification |
|-------------|-----------|---------------|
| EU | MiCA (Markets in Crypto-Assets) | Utility token — exempt from MiCA authorization if under €5M raise |
| France | AMF/PACTE | Jeton utilitaire — pas de visa AMF requis si pas d'offre au public > €8M |
| Spain | CNMV | Utility token — notification required if ICO |
| US | SEC Howey test | Utility token — not a security if no investment contract |

### 9.3 License

| Component | License |
|-----------|---------|
| CBOR-Web Specification | CC BY-ND 4.0 |
| Reference implementations | MIT |
| Smart contract source | MIT |
| Trademark "CBOR-Web" | To be registered (INPI + OEPM) |

---

## 10. eIDAS 2.0 Compatibility

The CBORW token model is compatible with eIDAS 2.0:

| eIDAS Feature | CBOR-Web Integration |
|--------------|---------------------|
| EUDI Wallet | T0 authentication mechanism (CBOR-WEB-SECURITY.md §4.1) |
| Verifiable Credentials | Publisher identity verification |
| Qualified Electronic Signatures | Manifest signing (alternative to COSE) |
| Cross-border recognition | Agent authenticated in FR → valid in all EU |

**Vision:** An AI agent with both an EUDI Wallet credential (identity) and a CBORW token (access) has full T0+T1 access across the entire CBOR-Web ecosystem. The EUDI proves WHO the agent represents. The CBORW proves the agent is a legitimate ecosystem participant.

---

## 11. Comparison with Alternative Models

| Model | Revenue | Adoption Barrier | Decentralized | Cross-Site |
|-------|---------|-----------------|---------------|-----------|
| **CBOR-Web (CBORW token)** | Token appreciation + rewards | Very low ($0.01 one-time) | Yes | Yes |
| Google UCP | Google platform fees | Vendor lock-in | No | Google only |
| llms.txt | None (free) | None | N/A | N/A |
| Per-site API keys | Per-key fees | High (negotiate per site) | No | No |
| Web3 subscription NFT | Monthly mint | Gas fees per month | Yes | No (per-project) |
| Traditional SaaS | Monthly subscription | $20-500/month | No | Central platform |

---

*CBOR-Web Economics Specification v2.1 — Document 5 of 6*

*Deltopide 2026*
