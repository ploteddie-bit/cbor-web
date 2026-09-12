#!/usr/bin/env python3
"""Résumé tokenomics — affiche le résumé du benchmark (utilisé par le job CI metrics).

Lit /tmp/tokenomics_metrics.json (produit par scripts/benchmark.sh --json via
tokenomics-dashboard.py) et imprime un résumé par scénario.
"""
import json

METRICS = "/tmp/tokenomics_metrics.json"

try:
    d = json.load(open(METRICS))
except FileNotFoundError:
    print(f"ERREUR: {METRICS} introuvable — lance d'abord scripts/benchmark.sh --json")
    raise SystemExit(1)

for s, v in d["scenarios"].items():
    print(
        f"  {s}: {v['sites']} sites, {v['agents']} agents, "
        f"${v['token_price']:.4f}/token, ${v['monthly_revenue']:.0f}/mo"
    )
