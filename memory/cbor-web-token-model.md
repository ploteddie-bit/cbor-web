---
name: CBOR-Web Token — Modele valide par Eddie
description: Modele economique token CBOR-Web valide le 21 mars 2026 — smart contract auto-regulateur + verification blockchain infalsifiable + 2 niveaux acces
type: project
---

## Modele token valide par Eddie (21 mars 2026 soir)

### 2 niveaux d'acces (pas 3)

| Niveau | Acces | Token |
|--------|-------|-------|
| **Sans token** | Vitrine seulement (ce que Google voit) | Aucun |
| **Avec token** | Tout le contenu, illimite | Detenir 1+ token |

Le token n'est PAS depense a chaque requete. C'est un **badge d'acces permanent** — tu le detiens, tu as acces. Tu ne le perds pas en naviguant.

### Smart contract auto-regulateur

Le prix du token est stabilise automatiquement par le smart contract :
- Si le prix monte trop vite → le smart contract libere des tokens de la reserve vers le marche
- Si le prix baisse → le smart contract rachete des tokens sur le marche
= Prix stable pour les utilisateurs
= Valeur croissante (mais stable) pour le fondateur

Modele des stablecoins gouvernes — pas de x1000 en une nuit, croissance liee a l'usage reel.

### Verification infalsifiable par la blockchain

Comment un site web verifie qu'un bot IA a le droit d'acceder au contenu :

1. Bot envoie son adresse wallet dans le header HTTP
2. Site interroge le smart contract Ethereum : `balanceOf(0x1234) > 0 ?`
3. Blockchain repond : OUI ou NON
4. Si OUI → acces complet. Si NON → vitrine seulement.

Pourquoi c'est infalsifiable :
- Inventer un faux token → impossible, le smart contract est la seule verite
- Utiliser le token de quelqu'un d'autre → impossible sans sa cle privee
- Falsifier la reponse de la blockchain → impossible, elle est publique et verifiable par tous

Pas de serveur central. La blockchain EST le serveur de verification.

### Mecanisme d'auto-financement du token

L'adoption fait monter le prix → le token se paie tout seul :
- Acheter 3 tokens a $0.01 = $0.03
- 1 mois plus tard chaque token vaut $0.03
- Vendre 1 token = $0.03 (mise recuperee)
- Garder 2 tokens = acces gratuit pour toujours

Plus tu arrives tot, moins ca coute. Ca cree une course a l'adoption.

### Point ouvert — identification du wallet

Le bot envoie son adresse wallet dans le header HTTP. Question non resolue : est-ce que l'adresse publique expose trop d'information ? ZKP envisage mais peut-etre pas necessaire si la signature de requete suffit (la cle privee ne quitte jamais la machine).

### Reevaluation mensuelle

Prevu : reevaluation mensuelle du prix par le smart contract pour maintenir la stabilite. Mecanisme exact a definir apres l'etude du rapport trading/tokenomics.

**Why:** Eddie veut un modele economique auto-suffisant ou l'adoption finance le projet sans avoir besoin de vendre un service SaaS.

**How to apply:** Le token est cree Day 1. Pas de SaaS obligatoire. Le service cbor-web.com est optionnel — le vrai business c'est le token.

### Plan de lancement valide (21 mars soir)

Phase 1 — Airdrop (cout $50) :
- Deployer le smart contract ERC-20 sur Ethereum (~$50 gas)
- 100M tokens crees en une transaction
- Distribuer 1000 tokens gratuits aux 50 premiers sites/agents (airdrop)
- La verification blockchain fonctionne immediatement (balanceOf > 0)
- Pas de pool Uniswap, pas de marche au debut

Phase 2 — Vente (quand la demande depasse l'airdrop) :
- Les nouveaux agents veulent un token → Eddie fixe le prix ($0.01)
- Vente directe ou creation pool Uniswap
- Le token a maintenant un prix de marche
- Marge = prix de vente - cout quasi nul de creation

Cout total lancement : ~$50. Pas de capital necessaire pour la liquidite au debut.
