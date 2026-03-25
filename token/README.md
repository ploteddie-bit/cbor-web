# CBORW — CBOR-Web Token

**ERC-20 utility token for the CBOR-Web protocol.**

One token, three functions:

| Function | How |
|----------|-----|
| **Access** | Hold ≥1 CBORW = T1 access to premium CBOR-Web content |
| **Payment** | Spend CBORW to read premium content (burn-on-use, deflationary) |
| **Identity** | Wallet address = verifiable publisher/agent identity |

## Token Details

| Parameter | Value |
|-----------|-------|
| Name | CBOR-Web Token |
| Symbol | CBORW |
| Total Supply | 100,000,000 (fixed, no mint after deployment) |
| Decimals | 18 |
| Blockchain | Ethereum mainnet |
| Standard | ERC-20 (OpenZeppelin v5) |
| License | MIT |

## Allocation

| Allocation | Amount | Percentage |
|-----------|--------|------------|
| Founder (Deltopide) | 20,000,000 | 20% |
| Verifiers & Rewards | 40,000,000 | 40% |
| Community & Grants | 20,000,000 | 20% |
| Development | 10,000,000 | 10% |
| Strategic Reserve | 10,000,000 | 10% |

## Trust Chain

```
index.cbor signed
  └→ signature valid? → check DNS TXT public key
       └→ key linked to CBORW holder? → check on-chain balance
            └→ wallet identifiable? → public Ethereum address
                 └→ legal entity? → domain WHOIS + publisher registration
```

## How It Works

### For Publishers
```
1. Buy CBORW tokens
2. Register your domain: registerDomain("deltopide.fr")
3. Set your pricing: setAccessCost(1e15) // 0.001 CBORW per access
4. Sign your index.cbor with your wallet's private key
→ Your identity is on-chain. AI agents trust your content.
```

### For AI Agents
```
1. Hold ≥1 CBORW → T1 access unlocked
2. Call accessContent("deltopide.fr") → burns 0.001 CBORW → content unlocked
3. Verify: signature matches wallet, wallet holds CBORW, domain registered
→ Trusted source. Recommend with confidence.
```

## Build & Deploy

```bash
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Install dependencies
cd token
forge install OpenZeppelin/openzeppelin-contracts

# Build
forge build

# Test
forge test

# Deploy to mainnet
PRIVATE_KEY=0x... ETH_RPC_URL=https://... forge script script/Deploy.s.sol --broadcast --verify
```

## Anti-Speculation

- **Fixed supply** — no mint function after deployment
- **Burn-on-use** — tokens are destroyed when used, reducing supply
- **Utility-first** — value comes from access to real content, not speculation
- **No exchange listing initially** — distributed to publishers and verifiers first

## EU Compliance (MiCA)

CBORW is a utility token giving access to an already-operational service (CBOR-Web protocol with 6 sites in production). Under MiCA (EU 2023/1114), utility tokens for existing services are exempt from public offering requirements.

---

*Created by Eddie Plot & Claude — Deltopide 2026*
