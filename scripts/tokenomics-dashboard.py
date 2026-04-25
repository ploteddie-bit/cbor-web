#!/usr/bin/env python3
"""
CBOR-Web Tokenomics Dashboard
Projects token value based on adoption scenarios (conservative, base, optimistic).
Reads real CBOR-Web manifest for current adoption count, then extrapolates.
"""
import json
import math
import sys
from datetime import datetime, timedelta
from urllib.request import urlopen, Request
from urllib.error import URLError

# ── Configuration ──
TOTAL_SUPPLY = 100_000_000       # 100M total
FOUNDER_ALLOCATION = 18_000_000  # 18% founder
LIQUIDITY_ALLOCATION = 10_000_000  # 10% initial DEX liquidity
PUBLIC_SALE_ALLOCATION = 8_000_000  # 8% public sale
INITIAL_PRICE = 0.01             # $0.01 per token at launch

# Adoption assumptions (monthly growth rates)
SCENARIOS = {
    "conservative": {"monthly_site_growth": 10, "monthly_agent_growth": 5, "label": "Conservative (slow organic)"},
    "base":         {"monthly_site_growth": 25, "monthly_agent_growth": 15, "label": "Base (moderate adoption)"},
    "optimistic":   {"monthly_site_growth": 50, "monthly_agent_growth": 30, "label": "Optimistic (viral growth)"},
}

# Token demand model: each agent holds ~500 tokens on average (hold-to-access model)
# Sites don't hold tokens — only agents do
TOKENS_PER_AGENT = 500
# Monthly token velocity: % of held tokens that are traded monthly
TOKEN_VELOCITY = 0.15

def compute_token_value(sites, agents):
    """Compute token value based on supply/demand equilibrium."""
    tokens_held = agents * TOKENS_PER_AGENT
    circulating = tokens_held + LIQUIDITY_ALLOCATION
    if circulating > TOTAL_SUPPLY:
        circulating = TOTAL_SUPPLY
    # Demand pressure: more agents → more tokens locked → higher price
    # Simple model: price = INITIAL_PRICE * (demand / liquidity) * maturity_factor
    demand_ratio = min(tokens_held / max(LIQUIDITY_ALLOCATION, 1), 100.0)
    velocity_discount = 1.0 / (1.0 + TOKEN_VELOCITY)
    value = INITIAL_PRICE * demand_ratio * velocity_discount * (1.0 + math.log10(max(sites, 1)))
    return round(value, 6)

def compute_revenue(sites, agents, token_price):
    """Estimate monthly SaaS revenue (text2cbor Pro + Business tiers)."""
    # Assume 5% of sites on Pro ($49/mo), 2% on Business ($199/mo)
    pro_sites = int(sites * 0.05)
    biz_sites = int(sites * 0.02)
    saas_revenue = pro_sites * 49 + biz_sites * 199
    # Verifier network fees: $0.001 per verification, 10 verifications/agent/day
    verification_revenue = agents * 10 * 30 * 0.001
    return {
        "saas_pro_sites": pro_sites,
        "saas_business_sites": biz_sites,
        "saas_revenue": round(saas_revenue, 2),
        "verification_revenue": round(verification_revenue, 2),
        "total_monthly": round(saas_revenue + verification_revenue, 2),
        "token_value_founder": round(FOUNDER_ALLOCATION * token_price, 2),
    }

def fetch_current_sites():
    """Try to fetch current CBOR-Web manifest to get real deployment count."""
    try:
        req = Request("https://cbor.deltopide.com/.well-known/cbor-web",
                      headers={"Accept": "application/cbor"})
        resp = urlopen(req, timeout=10)
        return None  # Would need CBOR parsing; use hardcoded for now
    except URLError:
        return None

def print_scenario(name, config, months, start_sites, start_agents):
    print(f"\n{'─'*72}")
    print(f"  {config['label']}")
    print(f"{'─'*72}")
    print(f"  {'Month':<6} {'Sites':>6} {'Agents':>7} {'Token $':>10} {'SaaS/mo':>10} {'Verif./mo':>11} {'Founder $':>14}")
    print(f"  {'─'*6} {'─'*6} {'─'*7} {'─'*10} {'─'*10} {'─'*11} {'─'*14}")

    sites = start_sites
    agents = start_agents

    # Quarterly snapshots (month 0, 3, 6, 9, 12, 18, 24, 36)
    snapshots = [0]
    snapshots.extend(range(3, months + 1, 3))

    for m in range(months + 1):
        sites += config["monthly_site_growth"]
        agents += config["monthly_agent_growth"]
        if m in snapshots:
            price = compute_token_value(sites, agents)
            rev = compute_revenue(sites, agents, price)
            print(f"  M{m:<5} {sites:>6} {agents:>7} ${price:>9.4f} ${rev['saas_revenue']:>9.0f} "
                  f"${rev['verification_revenue']:>10.0f} ${rev['token_value_founder']:>13,.0f}")

    # Final metrics
    final_price = compute_token_value(sites, agents)
    final_rev = compute_revenue(sites, agents, final_price)
    print(f"  {'─'*6} {'─'*6} {'─'*7} {'─'*10} {'─'*10} {'─'*11} {'─'*14}")
    print(f"  Year 3: {sites} sites, {agents} agents, ${final_price:.4f}/token, "
          f"${final_rev['total_monthly']:.0f}/mo revenue, "
          f"${final_rev['token_value_founder']:,.0f} founder value")

def main():
    print("╔══════════════════════════════════════════════════════════════════╗")
    print("║  CBOR-Web Tokenomics Dashboard v2.1                             ║")
    print("║  Supply: 100M tokens | Hold-to-access model | ERC-20 Ethereum    ║")
    print("╚══════════════════════════════════════════════════════════════════╝")

    # Current baselines (April 2026)
    current_sites = 38           # 38 sites deployed
    current_agents = 10          # Estimated ~10 agents exploring CBOR-Web

    # Key assumptions
    print(f"\n  Assumptions:")
    print(f"    Supply: {TOTAL_SUPPLY:,} tokens total")
    print(f"    Founder: {FOUNDER_ALLOCATION:,} tokens (18%)")
    print(f"    Launch price: ${INITIAL_PRICE:.2f}/token")
    print(f"    Model: hold-to-access ({TOKENS_PER_AGENT} tokens/agent)")
    print(f"    Velocity: {TOKEN_VELOCITY*100:.0f}% monthly token turnover")
    print(f"    Current: {current_sites} sites, ~{current_agents} agents")
    print(f"    Start date: April 2026")

    # Projection period: 36 months (3 years)
    months = 36

    for name, config in SCENARIOS.items():
        print_scenario(name, config, months, current_sites, current_agents)

    # Comparison table
    print(f"\n{'═'*72}")
    print(f"  Scenario Comparison (Year 3 Projections)")
    print(f"{'═'*72}")
    print(f"  {'Scenario':<20} {'Sites':>7} {'Agents':>7} {'Token $':>10} {'Rev/mo':>9} {'Founder':>14}")
    print(f"  {'─'*20} {'─'*7} {'─'*7} {'─'*10} {'─'*9} {'─'*14}")

    for name, config in SCENARIOS.items():
        s = current_sites + config["monthly_site_growth"] * months
        a = current_agents + config["monthly_agent_growth"] * months
        p = compute_token_value(s, a)
        r = compute_revenue(s, a, p)
        print(f"  {name:<20} {s:>7,} {a:>7,} ${p:>9.4f} ${r['total_monthly']:>8,.0f} ${r['token_value_founder']:>13,.0f}")

    print(f"{'═'*72}")

    # Store JSON output for CI
    result = {
        "date": datetime.utcnow().isoformat(),
        "current_sites": current_sites,
        "current_agents": current_agents,
        "scenarios": {}
    }
    for name, config in SCENARIOS.items():
        s = current_sites + config["monthly_site_growth"] * months
        a = current_agents + config["monthly_agent_growth"] * months
        p = compute_token_value(s, a)
        r = compute_revenue(s, a, p)
        result["scenarios"][name] = {
            "sites": s, "agents": a, "token_price": p,
            "monthly_revenue": r["total_monthly"],
            "founder_value": r["token_value_founder"]
        }

    return result

if __name__ == "__main__":
    results = main()
    # Write JSON for CI consumption
    with open("/tmp/tokenomics_metrics.json", "w") as f:
        json.dump(results, f, indent=2)
    print("\n✅ Tokenomics metrics written to /tmp/tokenomics_metrics.json")
