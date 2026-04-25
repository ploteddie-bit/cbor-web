# CBOR-Web Economics Specification

**Token Economics, Pricing, Launch Plan, Regulation, and Smart Contract**

```
Status:       Proposed Standard
Version:      2.1
Date:         2026-03-21
Authors:      ExploDev (Eddie Plot)
License:      CC BY 4.0
Repository:   https://github.com/ploteddie-bit/cbor-web
Document:     5 of 6 — CBOR-WEB-ECONOMICS.md
Companion:    CBOR-WEB-CORE.md, CBOR-WEB-MULTIMEDIA.md,
              CBOR-WEB-GENERATIVE.md, CBOR-WEB-SECURITY.md,
              CBOR-WEB-REFERENCE.md
```

---

## About This Document

This document is **part 5 of 7** of the CBOR-Web v2.1 specification suite. It defines the **economic model** behind the CBOR-Web token (CBORW): allocation, pricing, stabilization, launch plan, regulation, and the smart contract interface.

This is NOT a whitepaper or marketing document — it is a technical specification of the economic mechanisms that support the CBOR-Web access control model defined in CBOR-WEB-SECURITY.md §4.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Token Properties](#2-token-properties)
3. [Token Allocation](#3-token-allocation)
4. [Vesting Schedules](#4-vesting-schedules)
5. [Price Stabilization Mechanism](#5-price-stabilization-mechanism)
6. [Pricing Model](#6-pricing-model)
7. [Launch Plan](#7-launch-plan)
8. [Self-Financing Through Appreciation](#8-self-financing-through-appreciation)
9. [Regulation — MiCA Compliance](#9-regulation--mica-compliance)
10. [Anti-Speculation Design](#10-anti-speculation-design)
11. [Risk Analysis](#11-risk-analysis)
12. [Comparison with Existing Tokens](#12-comparison-with-existing-tokens)
- [Appendix A: Smart Contract (Solidity)](#appendix-a-smart-contract-solidity)
- [Appendix B: Financial Projections](#appendix-b-financial-projections)
- [Appendix C: MiCA Legal Guide](#appendix-c-mica-legal-guide)
- [References](#references)

---

## 1. Overview

### 1.1 The Token as Access Badge

The CBOR-Web Token (CBORW) is a **utility token** — it provides access to a service (CBOR-Web content across the web). It is NOT:

- A security token (no dividends, no profit-sharing, no governance rights)
- A payment token (not spent per request, not used as currency)
- A stablecoin (not pegged to fiat)
- A governance token (no voting rights)

It IS:
- A **badge of membership** — hold ≥ 1 token, access all CBOR-Web content everywhere
- A **one-time acquisition** — buy once, hold forever, access permanently
- A **universal key** — works on every CBOR-Web site, not per-site

### 1.2 Economic Design Principles

| Principle | Description |
|-----------|-------------|
| **Badge, not payment** | The token is held, not spent. No per-request cost. No burning. No staking requirement. |
| **Universal access** | One token works on ALL participating sites. No per-site subscriptions. |
| **Self-financing** | The project finances itself through token appreciation as adoption grows. No VC, no ads, no subscription fees. |
| **Stability** | An automated smart contract stabilizes the price to prevent wild speculation. |
| **Low barrier** | Initial token price targets < $0.01. Cost of entry: cents, not dollars. |
| **Legal compliance** | Designed as a utility token under MiCA (EU Markets in Crypto-Assets Regulation). |

---

## 2. Token Properties

| Property | Value |
|----------|-------|
| Standard | ERC-20 (Ethereum mainnet) |
| Name | CBOR-Web Token |
| Symbol | CBORW |
| Total Supply | 100,000,000 (fixed, no minting after deployment) |
| Decimals | 18 (standard ERC-20 precision) |
| Blockchain | Ethereum mainnet |
| Contract | To be deployed (see §7) |
| Usage | Hold ≥ 1 token to access L1 (full access) CBOR-Web content |

### 2.1 Why ERC-20 on Ethereum?

| Criterion | Ethereum | Alternative Considered | Why Ethereum Wins |
|-----------|----------|----------------------|-------------------|
| **Maturity** | 10+ years, battle-tested | Solana (3 years), Polygon (newer) | Smart contract security is critical for trust |
| **Tooling** | ethers.js, web3.py, alloy, foundry | Less mature ecosystems | Publisher integration must be easy |
| **Adoption** | Largest smart contract ecosystem | Smaller user bases | More potential agents already have ETH wallets |
| **Decentralization** | Highly decentralized | More centralized alternatives | No single point of failure for token verification |
| **Cost** | Gas fees ~$0.50-5 per transaction | Lower fees on L2/alt chains | Agents trade infrequently — gas is a minor cost |
| **Verifiability** | Any node can verify balanceOf | Same | Standard RPC infrastructure |

**Future consideration:** A Layer 2 deployment (Arbitrum, Optimism, Base) MAY be added to reduce gas costs for frequent traders. The L1 contract remains the canonical source of truth.

### 2.2 Why Not a Custom Blockchain?

Creating a custom blockchain (or using IOTA, as originally considered) would:
- Require agents to support a new network (high adoption barrier)
- Reduce liquidity (no DEX integration)
- Increase security risk (smaller validator set)
- Add operational complexity (running validator nodes)

Ethereum provides all needed functionality via a standard ERC-20 contract. No custom chain is necessary.

---

## 3. Token Allocation

| Allocation | Percentage | Tokens | Purpose |
|-----------|-----------|--------|---------|
| **Founder** (ExploDev) | 20% | 20,000,000 | Founder value, project leadership, long-term commitment |
| **Airdrop** (initial adoption) | 10% | 10,000,000 | Free distribution to first 50-100 adopters (sites + agents) |
| **Stabilization Reserve** | 30% | 30,000,000 | Controlled by smart contract for automatic price stabilization |
| **Community / Grants** | 20% | 20,000,000 | Developer rewards, hackathons, integration bounties, ecosystem growth |
| **Development** | 10% | 10,000,000 | Finance development of tools (text2cbor, validators, libraries) |
| **Future Team** | 10% | 10,000,000 | Recruitment, partnerships, advisory |

### 3.1 Allocation Rationale

**Founder 20%**: Standard for utility token projects. Vested over 2 years to demonstrate long-term commitment. The founder cannot dump tokens immediately.

**Airdrop 10%**: Critical for bootstrapping. The first 50 sites and 50 agents receive free tokens to create initial supply and demand. Without an airdrop, there is a chicken-and-egg problem (no sites → no reason for agents to get tokens → no sites).

**Stabilization 30%**: The largest allocation — intentionally. The smart contract uses this reserve as a buffer to absorb price shocks. See §5 for the mechanism.

**Community 20%**: Unlocked progressively as the ecosystem grows. Used for developer grants (e.g., "build a CBOR-Web plugin for WordPress → receive 10,000 CBORW"), hackathon prizes, and integration rewards.

**Development 10%**: Finances the tools that make CBOR-Web usable: `text2cbor` (Rust), CBOR-Web validators, language-specific libraries, documentation sites.

**Future team 10%**: Locked for 1 year. Available for hiring engineers, business development, or strategic partnerships.

---

## 4. Vesting Schedules

| Allocation | Vesting | Cliff | Schedule |
|-----------|---------|-------|----------|
| Founder | 2 years | None | Linear: 1/24th unlocked monthly |
| Airdrop | None | None | Immediately available |
| Stabilization | N/A | N/A | Controlled by smart contract (not human-accessible) |
| Community | Progressive | None | Unlocked as grants are approved (multi-sig governance) |
| Development | 2 years | 1 year | Nothing for 12 months, then 1/12th monthly |
| Future team | 2 years | 1 year | Nothing for 12 months, then 1/12th monthly |

**Linear vesting example (Founder):**

```
Month 0:  0 tokens available (all locked)
Month 1:  833,333 tokens unlocked (20M / 24)
Month 2:  1,666,667 tokens unlocked
...
Month 12: 10,000,000 tokens unlocked (50%)
...
Month 24: 20,000,000 tokens unlocked (100%)
```

**Why no cliff for founder?** The founder is also the sole developer. A 1-year cliff would mean zero liquidity for the project for the first year. Linear vesting from day 1 allows the founder to fund development while demonstrating long-term commitment.

---

## 5. Price Stabilization Mechanism

### 5.1 The Problem with Unregulated Token Prices

Without stabilization, a utility token's price is at the mercy of speculation:
- Hype drives price up 10x → agents can't afford tokens → adoption stalls
- Panic drives price down 90% → perception of failure → publishers leave

A utility token needs **price stability** — the price should reflect usage, not speculation.

### 5.2 Smart Contract Auto-Regulator

The stabilization reserve (30M tokens) is controlled by a smart contract that acts as an **automatic market maker with stabilization logic**:

```
Monitor: 7-day rolling average price (from Uniswap TWAP oracle)

IF price increased > 20% in 7 days:
  → Release tokens from stabilization reserve to Uniswap pool
  → Increased supply on the market → price pressure downward → stabilizes
  → Amount released: proportional to the excess increase

IF price decreased > 10% in 7 days:
  → Buy tokens from Uniswap pool using reserve ETH
  → Decreased supply on the market → price pressure upward → recovers
  → Amount purchased: proportional to the excess decrease

Monthly: recalculate thresholds based on 30-day average network usage
  (number of unique agents, number of sites, total requests)
```

### 5.3 Stabilization Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Price increase trigger | > 20% in 7 days | Allow organic growth, dampen speculation |
| Price decrease trigger | > 10% in 7 days | React faster to drops (protect confidence) |
| Release amount (up) | 1% of reserve per 5% excess | Gradual intervention, not dumping |
| Purchase amount (down) | Uses available ETH reserve | Limited by ETH balance |
| Recalculation period | Monthly | Adapt to changing conditions |
| Oracle | Uniswap V3 TWAP (time-weighted average) | Manipulation-resistant price feed |

### 5.4 When the Stabilization Reserve Runs Out

The 30M token reserve is finite. If the project succeeds massively and the reserve is depleted:

1. The token price is now determined purely by market forces
2. By this point, the ecosystem should be large enough that organic supply/demand provides stability
3. The smart contract can be upgraded (via governance) to implement alternative stabilization (e.g., fee-based reserve replenishment)

---

## 6. Pricing Model

### 6.1 No Per-Request Cost

The CBOR-Web token is NOT spent per request. This is a fundamental design decision:

| Model | Per-Request | Badge (CBOR-Web) |
|-------|------------|-------------------|
| Cost per request | $0.001 per page | $0 per page |
| Monthly cost (10K pages) | $10 | $0 |
| Token balance after 1 year | Depleted | Same as day 1 |
| Predictability | Unpredictable costs | Zero ongoing cost |
| Agent complexity | Track balance, manage top-ups | Hold token, forget |

**Why badge over per-request?**
1. **Simplicity**: No accounting, no balance tracking, no top-up logic
2. **Predictability**: Agent operators know the exact lifetime cost: buy once
3. **Adoption incentive**: The value proposition is clear — "buy one token, access everything forever"
4. **No meter anxiety**: Agents don't hesitate to crawl because there's no cost

### 6.2 Target Price Range

| Phase | Target Price | Market Cap | Rationale |
|-------|-------------|-----------|-----------|
| Launch | $0.001 | $100K | Accessible to anyone. 1 token = $0.001. |
| Early adoption (6 months) | $0.01 | $1M | 10x growth from initial adoption |
| Growth (1 year) | $0.03-0.10 | $3-10M | Ecosystem matures, more sites/agents |
| Maturity (2+ years) | $0.10-1.00 | $10-100M | Utility token at scale |

These are **estimates**, not guarantees. Actual price depends on adoption.

---

## 7. Launch Plan

### 7.1 Phase 1 — Contract Deployment (~$50)

```
Timeline: Day 1
Cost:     ~$50 (Ethereum gas for contract deployment)
Actions:
  1. Deploy ERC-20 smart contract on Ethereum mainnet
  2. Mint 100,000,000 CBORW tokens in one transaction
  3. Transfer allocations to vesting contracts
  4. Lock stabilization reserve in stabilization contract
  5. Publish contract address on cbor-web.org and in all spec documents
```

### 7.2 Phase 2 — Airdrop (~$100)

```
Timeline: Week 1-4
Cost:     ~$100 (gas for 50-100 token transfers)
Actions:
  1. Identify first 50 sites willing to adopt CBOR-Web
  2. Identify first 50 agents/agent developers
  3. Distribute 100,000 CBORW tokens each (free, no cost to recipient)
  4. Provide integration support (text2cbor setup, agent wallet creation)
  5. First CBOR-Web sites go live with token verification
```

### 7.3 Phase 3 — Organic Growth (no cost)

```
Timeline: Months 1-6
Cost:     $0 (growth is organic)
Actions:
  1. More sites adopt CBOR-Web (drawn by the spec + working tools)
  2. More agents discover CBOR-Web sites and want tokens
  3. Founder sells tokens directly at fixed price ($0.001-0.01)
  4. Word-of-mouth in AI/developer community
  5. Token has organic demand
```

### 7.4 Phase 4 — Market ($5K-10K when ready)

```
Timeline: When demand justifies (est. 6 months)
Cost:     $5,000-10,000 (initial liquidity for Uniswap pool)
Actions:
  1. Create Uniswap V3 liquidity pool (CBORW/ETH)
  2. Token freely tradeable on DEX
  3. Price determined by supply and demand
  4. Activate stabilization smart contract
  5. Price oracle connected
```

---

## 8. Self-Financing Through Appreciation

### 8.1 The Model

The project finances itself through a natural economic cycle:

```
Step 1: Founder deploys contract, creates tokens (cost: $50)
Step 2: Airdrop creates initial ecosystem (cost: $100)
Step 3: Adoption grows → demand for tokens increases → price rises
Step 4: Founder's 20% allocation appreciates in value
Step 5: Founder sells a fraction to fund development
Step 6: Development improves the protocol → more adoption → step 3
```

### 8.2 Early Adopter Economics

```
Early adopter buys 3 tokens at $0.001 each = $0.003 total cost
  
6 months later, token price = $0.01:
  → Portfolio value: 3 × $0.01 = $0.03
  → Sell 1 token = $0.01 (3x return on $0.003 investment)
  → Keep 2 tokens = permanent free L1 access
  → Net cost: $0.003 - $0.01 = -$0.007 (PROFIT)

The earlier you join, the cheaper it is. This creates an adoption race.
```

### 8.3 Why This Works

| Factor | Effect |
|--------|--------|
| Fixed supply (100M) | Scarcity increases as demand grows |
| Increasing utility | More sites = more valuable to hold a token |
| Network effects | Each new site makes the token more useful for agents |
| No inflation | No new tokens minted — supply is permanently fixed |
| Stabilization dampens bubbles | Price grows with adoption, not with hype |

---

## 9. Regulation — MiCA Compliance

### 9.1 MiCA Classification

The EU Markets in Crypto-Assets Regulation (MiCA), effective June 2024 for stablecoins and December 2024 for other crypto-assets, classifies tokens into categories:

| MiCA Category | CBORW Applicability | Requirement |
|---------------|-------------------|-------------|
| E-money token | ❌ Not applicable | CBORW is not pegged to fiat |
| Asset-referenced token | ❌ Not applicable | CBORW is not backed by assets |
| **Utility token** | **✅ Applicable** | Exemption available if service exists at launch |
| Crypto-asset (general) | Fallback if utility exemption fails | White paper + notification to authority |

### 9.2 Utility Token Exemption (Article 4(3))

MiCA provides an exemption for utility tokens that:

1. **Provide access to an existing service** — CBORW provides access to CBOR-Web content (the spec and tools exist at launch) ✅
2. **Are accepted only by the issuer or a limited group** — CBORW is accepted by CBOR-Web publishers (a defined group) ✅
3. **Are not primarily acquired for speculative purposes** — the token's primary purpose is access, not investment ✅

**To qualify for the exemption:**
- The CBOR-Web specification and tools MUST be functional before the token launch
- The token MUST have demonstrable utility (access to content) at the time of sale
- Marketing MUST NOT promote the token as an investment opportunity

### 9.3 Compliance Checklist

```
☐ Specification published and functional (CBOR-WEB-CORE.md et al.)
☐ Reference implementation available (text2cbor)
☐ At least 5 sites serving CBOR-Web content before token launch
☐ Token utility documented (this document)
☐ No investment claims in marketing materials
☐ Terms of service explicitly state "utility token, not investment"
☐ Legal opinion from EU crypto-asset attorney (recommended)
```

---

## 10. Anti-Speculation Design

### 10.1 Design Choices That Discourage Speculation

| Feature | Anti-Speculation Effect |
|---------|----------------------|
| **Fixed supply, no burn** | No deflationary pressure to drive artificial scarcity |
| **No staking rewards** | No yield farming incentive to accumulate |
| **No governance** | No voting power to attract governance token collectors |
| **Stabilization contract** | Actively dampens price swings |
| **Low price target** | $0.001-0.10 range is unexciting for speculators |
| **Badge model** | Only 1 token needed — no incentive to hold thousands |
| **Utility-first messaging** | All documentation emphasizes utility, not investment |

### 10.2 What If Speculation Happens Anyway?

If CBORW is listed on major exchanges and experiences speculative trading:

1. **Stabilization contract activates**: releases reserve tokens to suppress price spikes
2. **Utility remains unchanged**: 1 token still = full access, regardless of price
3. **Price floor from utility**: even if speculation collapses, the token retains utility value (access to content is still valuable)

---

## 11. Risk Analysis

### 11.1 Risks and Mitigations

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| **Low adoption** — few sites/agents adopt | HIGH | MEDIUM | Open-source tools, free airdrop, specification quality |
| **Ethereum gas costs spike** — transactions become expensive | MEDIUM | LOW | Token is held, not transacted frequently. L2 bridge possible. |
| **Smart contract bug** — vulnerability in token contract | CRITICAL | LOW | Audited contract, standard ERC-20 (well-tested pattern) |
| **Regulatory change** — MiCA exemption revoked | HIGH | LOW | Prepare white paper and notification as fallback |
| **Competitor** — another protocol emerges | MEDIUM | MEDIUM | First-mover advantage, spec quality, open standard |
| **Token price crashes to 0** — no demand | MEDIUM | LOW | Utility floor (access still works), community grants |
| **Stabilization reserve depleted** — no more stabilization | LOW | LOW | By that point, market should be self-stabilizing |
| **Key person risk** — founder leaves | HIGH | LOW | Open-source spec, community governance of grants |

### 11.2 Worst-Case Scenario

If CBORW fails completely (token price = 0, no adoption):
- The **specification** remains valuable as an open standard (CC BY 4.0)
- The **tools** (text2cbor, validators) remain usable without tokens
- Publishers can still serve CBOR-Web content at S0 (no token required)
- The token model can be replaced by a different access mechanism in a future spec version

The specification is designed to be **valuable independent of the token**. The token enables the economic model but is not required for the core technology.

---

## 12. Comparison with Existing Tokens

| Token | Type | Usage | Price Model | CBORW Comparison |
|-------|------|-------|-------------|-----------------|
| **LINK** (Chainlink) | Utility | Pay oracle operators per query | Per-request | CBORW: badge, not per-request |
| **BAT** (Brave) | Utility | Reward users for viewing ads | Advertising-based | CBORW: access-based, no ads |
| **FIL** (Filecoin) | Utility | Pay for storage | Per-GB storage | CBORW: no storage, content access |
| **ENS** (Ethereum Name Service) | Utility/Governance | Register .eth domains | Per-domain annual fee | CBORW: one-time, no renewal |
| **GRT** (The Graph) | Utility | Pay for API queries | Per-query | CBORW: badge, unlimited queries |
| **CBORW** | Utility | Access CBOR-Web content | **Badge (hold ≥ 1)** | **Unique: one token = universal access** |

The badge model is unique among utility tokens. Most utility tokens are spent per-use (LINK per oracle query, FIL per GB stored, GRT per API call). CBORW is held permanently — closer to a membership card than a prepaid balance.

---

## Appendix A: Smart Contract (Solidity)

The CBORW token uses a standard ERC-20 implementation with the addition of a stabilization interface. The full contract will be deployed on Ethereum mainnet after audit.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title CBORWebToken
 * @dev ERC-20 token for CBOR-Web access control.
 *      Hold >= 1 token (1e18 wei) to access L1 content.
 */
contract CBORWebToken is ERC20, Ownable {
    uint256 public constant TOTAL_SUPPLY = 100_000_000 * 1e18;

    // Allocation addresses (set at deployment)
    address public stabilizationReserve;
    address public communityGrants;
    address public developmentFund;
    address public futureTeam;

    constructor(
        address _founder,
        address _airdrop,
        address _stabilization,
        address _community,
        address _development,
        address _futureTeam
    ) ERC20("CBOR-Web Token", "CBORW") Ownable(_founder) {
        stabilizationReserve = _stabilization;
        communityGrants = _community;
        developmentFund = _development;
        futureTeam = _futureTeam;

        // Mint all tokens at deployment (no further minting possible)
        _mint(_founder,        20_000_000 * 1e18);  // 20% founder
        _mint(_airdrop,        10_000_000 * 1e18);  // 10% airdrop
        _mint(_stabilization,  30_000_000 * 1e18);  // 30% stabilization
        _mint(_community,      20_000_000 * 1e18);  // 20% community
        _mint(_development,    10_000_000 * 1e18);  // 10% development
        _mint(_futureTeam,     10_000_000 * 1e18);  // 10% future team
    }

    /**
     * @dev Check if an address holds enough tokens for L1 access.
     *      Publishers call this function (via RPC, not on-chain) to verify access.
     * @param account The wallet address to check
     * @return hasAccess True if balance >= 1 token (1e18 wei)
     */
    function hasL1Access(address account) external view returns (bool hasAccess) {
        return balanceOf(account) >= 1e18;
    }

    // No mint function — supply is permanently fixed at 100M
    // No burn function — tokens are not consumed
    // No pause function — access should always be available
}
```

**Note:** The stabilization logic (§5) is implemented in a **separate contract** that interacts with Uniswap V3. This separation keeps the token contract simple and auditable.

---

## Appendix B: Financial Projections

### B.1 Conservative Scenario

| Timeline | Sites | Agents | Token Price | Market Cap |
|----------|-------|--------|-------------|-----------|
| Launch | 5 | 5 | $0.001 | $100K |
| 3 months | 20 | 30 | $0.002 | $200K |
| 6 months | 50 | 100 | $0.005 | $500K |
| 1 year | 100 | 300 | $0.01 | $1M |
| 2 years | 500 | 1,000 | $0.03 | $3M |

### B.2 Realistic Scenario

| Timeline | Sites | Agents | Token Price | Market Cap |
|----------|-------|--------|-------------|-----------|
| Launch | 10 | 10 | $0.001 | $100K |
| 3 months | 50 | 100 | $0.005 | $500K |
| 6 months | 200 | 500 | $0.02 | $2M |
| 1 year | 1,000 | 2,000 | $0.10 | $10M |
| 2 years | 5,000 | 10,000 | $0.50 | $50M |

### B.3 Optimistic Scenario

| Timeline | Sites | Agents | Token Price | Market Cap |
|----------|-------|--------|-------------|-----------|
| Launch | 20 | 20 | $0.001 | $100K |
| 3 months | 200 | 500 | $0.01 | $1M |
| 6 months | 1,000 | 5,000 | $0.10 | $10M |
| 1 year | 10,000 | 50,000 | $1.00 | $100M |
| 2 years | 100,000 | 500,000 | $5.00 | $500M |

**Disclaimer:** These are illustrative projections, not guarantees. Actual outcomes depend on adoption, market conditions, and competition.

---

## Appendix C: MiCA Legal Guide

### C.1 Steps for Utility Token Compliance in the EU

1. **Document utility**: This specification suite serves as documentation that the token provides access to an existing service.

2. **Launch service before token**: The CBOR-Web specification, tools (text2cbor), and at least 5 sites MUST be operational before any token sale. The airdrop (Phase 2) occurs after the service is live.

3. **No investment language**: Marketing materials MUST NOT use language like "investment opportunity", "guaranteed returns", "appreciation", or "profit". Use: "access badge", "utility token", "membership".

4. **Prepare white paper (optional but recommended)**: If the exemption is challenged, having a MiCA-compliant white paper demonstrates good faith. The white paper should contain:
   - Token description and utility
   - Issuer identity
   - Technology description
   - Rights and obligations
   - Risk disclosure

5. **Notify competent authority**: Under MiCA, issuers of crypto-assets must notify their national competent authority at least 20 working days before offering the token. For a Spanish issuer (ExploDev / Deltopide SL), this is the CNMV (Comision Nacional del Mercado de Valores).

6. **Maintain compliance**: Ongoing obligations include keeping the white paper updated, honoring the described utility, and reporting material changes.

---

## References

### Normative References

- **[ERC-20]** Vogelsteller, F. and V. Buterin, "EIP-20: Token Standard", November 2015.
- **[MiCA]** European Parliament, "Regulation (EU) 2023/1114 on markets in crypto-assets", June 2023.

### Informative References

- **[CBOR-WEB-SECURITY.md]** CBOR-Web Security Specification v2.1 — token access control.
- **[CBOR-WEB-CORE.md]** CBOR-Web Core Specification v2.1.
- **[Uniswap V3]** "Uniswap V3 Core", https://uniswap.org/whitepaper-v3.pdf
- **[OpenZeppelin]** "OpenZeppelin Contracts", https://docs.openzeppelin.com/contracts/

---

*CBOR-Web Economics Specification v2.1 — Document 5 of 6*

*ExploDev 2026*
