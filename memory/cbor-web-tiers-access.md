---
name: cbor-web-tiers-access
description: Decision architecturale CBOR-Web — modele 3 tiers d'acces (public/profondeur/commerce) avec cbor.txt
type: project
---

## Modele d'acces CBOR-Web — 3 Tiers (decision 2026-03-22)

**Why:** Le standard doit etre adopte facilement (pas de barriere) tout en securisant les transactions. Identite obligatoire partout tuerait l'adoption. Inspiré de la cohabitation robots.txt / llms.txt.

**How to apply:** Integrer ce modele dans CBOR-WEB-SECURITY.md et CBOR-WEB-CORE.md (section discovery/cbor.txt).

### Tier 0 — Public (pas de token)
- Comme robots.txt / llms.txt aujourd'hui
- Contenu de surface, metadata, catalogue
- Declare dans `cbor.txt` a la racine du site
- Zero barriere = adoption facile

### Tier 1 — Profondeur (webmaster decide)
- Contenu detaille, API lecture, donnees riches
- Token optionnel selon strategie du webmaster
- Le webmaster configure ce qui est Tier 0 vs Tier 1 dans cbor.txt

### Tier 2 — Commerce / Action (token OBLIGATOIRE)
- Achat, formulaires, paiement, donnees personnelles
- Identite verifiee, transaction tracee
- Non negociable : pas de token = pas d'action
- Aligné avec eIDAS 2.0 (identite numerique EU) etendu aux machines

### cbor.txt
- Fichier a la racine du site (comme robots.txt)
- Declare les regles d'acces par tier
- Cohabite avec robots.txt et llms.txt
- Point d'entree de la relation agent ↔ site
- L'agent le lit EN PREMIER avant toute interaction

### Tier INTERDIT — Ethique (hardcode, non configurable)
- Violence, armes, drogue, contenu criminel, exploitation humaine
- Pas une option webmaster — grave dans le protocole lui-meme
- Triple verrouillage : filtre agent + filtre reseau + signature SHA-512
- Aucun token, aucun tier ne peut debloquer ces contenus
- Un agent CBOR-Web qui detecte ce contenu DOIT refuser et signaler
- Non negociable, non desactivable, non contournable par design
- Ancre au plus profond du code, pas dans un fichier de config

### Principes
- Pas d'anonymat quand il y a de l'argent ou des donnees perso
- Le webmaster garde le controle total sur sa strategie d'acces (SAUF contenu interdit)
- Les hackers contourneront (comme toujours) mais le rapport de force change : chaque acces Tier 2 est trace et imputable
- Rampe d'adoption : commencer en Tier 0 gratuit, activer les tokens progressivement
- L'ethique n'est PAS une couche — c'est le socle. On construit par dessus, pas a cote
