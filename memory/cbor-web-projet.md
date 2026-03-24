---
name: Projet CBOR-Web
description: Specification format web binaire pour IA + outil text2cbor + modele economique token Ethereum — projet fondateur ExploDev
type: project
---

## CBOR-Web — Le TCP/IP de l'IA

Projet initie le 20 mars 2026 par Eddie Plot. Vision : un protocole binaire open source permettant aux agents IA de lire le web sans passer par le HTML.

### Domaines acquis (21 mars 2026)
- **cbor-web.org** — site principal du standard (Infomaniak)
- **cbor-web.com** — site commercial / SaaS (Infomaniak)
- Repo GitHub : **explodev/cbor-web** (cree le 21 mars)

### Etat des documents

| Document | Version | Lignes | Statut |
|----------|---------|--------|--------|
| CBOR-WEB-SPEC | v1.0 | ~800 | Base — contenu textuel statique |
| CBOR-WEB-SPEC | v2.0 | ~1500 | Multimedia, blocs generatifs, commerce, temps reel |
| CBOR-WEB-SECURITY | v1.0 | 1872 | Architecture securite 8 couches, DID, PoW, fail2ban |
| Review critique | - | - | 22 findings (4 critiques, 6 majeurs, 6 importants, 6 mineurs) |
| CBOR-WEB-SPEC | v3.0 | En cours | Fusion v2.0 + securite + corrections + modele economique |
| Business Plan | - | 400+ | Complet — `~/cbor-web-business-plan.md` + `C:\Users\eddie\Downloads\CBOR-WEB-BUSINESS-PLAN.md` |

### Outil text2cbor

Binaire Rust 5.3MB compile sur MacPro (`/data/builds/text2cbor/`). 3 modes : convert, serve, batch. A adapter a la spec v3.0 finale (format manifest/pages/bundle, discovery /.well-known/cbor-web). Mode watch a ajouter.

### Site web cbor-web.org

Site complet 8 pages genere dans `/home/ubuntu/cbor-web-site/` (336 Ko). HTML/CSS/JS pur, dark mode, responsive. A deployer sur Fram cloud. Correction a faire : lien GitHub pointe vers mauvais repo (nicobao → explodev).

### Decisions economiques (session 21 mars nuit)

**Token :**
- Token ERC-20 sur **Ethereum mainnet** (pas IOTA, pas L2) — plus connu, plus de confiance
- Frais gas Ethereum : ~$0.22 par transfert en mars 2026
- Solution : **smart contract prepaye** — depot une seule fois ($10), consommation interne sans gas a chaque requete
- Le depot dans le smart contract = l'identite de l'agent ("synapse neuronale")

**Pricing :**
- **$0.00001 par requete** (un centime pour 1000 pages)
- Prix fixe, pas degressif — commence bas pour maximiser l'adoption
- A ce prix, les frais gas sont absorbes par le depot prepaye

**Allocation tokens :**
- 100M tokens total
- 20% fondateur (Eddie/ExploDev) = 20M tokens
- 40% verificateurs (rewards)
- 20% communaute/grants
- 10% developpement
- 10% reserve strategique

**Licence :**
- Spec : CC BY 4.0 (ouverte, gratuite)
- Outils : MIT (open source)
- L'argent vient de l'ecosysteme (SaaS, tokens, consultance), pas de la licence
- Marque "CBOR-Web" a deposer INPI + OEPM

### Decisions techniques securite (session 21 mars nuit)

**Authentification — modele synapse :**
- L'adresse Ethereum du wallet = l'identite unique de l'agent
- Le depot de tokens dans le smart contract = la "synapse" (connexion permanente)
- Tant que le depot est actif, l'agent est authentifie — pas de re-verification a chaque requete
- Quand le solde = 0, la synapse est coupee

**Zero-Knowledge Proofs :**
- Etudie mais peut-etre pas necessaire si le modele synapse suffit
- ZKP = prouver qu'on a un wallet valide sans reveler lequel
- Probleme : lourd en calcul pour la premiere connexion (~1-5s)
- Solution envisagee : ZKP une fois → pass temporaire 1h → requetes rapides

**Compatibilite eIDAS 2.0 :**
- L'Europe impose le wallet numerique d'identite pour decembre 2026
- CBOR-Web peut etre compatible — le wallet eIDAS comme methode d'authentification alternative
- Avantage : identite verifiee par un Etat europeen = confiance maximale
- Probleme : eIDAS est concu pour les humains (lent), pas pour les machines (rapide)
- Solution : premiere auth eIDAS (lente, une fois) → session CBOR-Web (rapide, illimitee)

**Point ouvert (Eddie fatigue, a reprendre) :**
- Comment le serveur sait que le pass/session appartient bien a cet agent si le serveur ne connait pas l'identite (ZKP) ?
- Le modele synapse (depot = identite) resout peut-etre ca sans ZKP
- A clarifier avec Eddie a la prochaine session

### Chaine de production (mise a jour)

1. Spec v3.0 finalisee (frere Claude sur claude.ai) ← en cours
2. Chapitre securite integre avec corrections des 22 findings ← en cours
3. Chapitre economique + token + eIDAS ← prompt envoye
4. text2cbor adapte a la spec (SG Desktop) ← apres v3.0
5. Site web deploye sur Fram cloud ← fichiers prets dans ~/cbor-web-site/
6. DNS Infomaniak → Fram cloud ← a configurer
7. Deploiement CBOR-Web sur sites deltopide (premiers adopteurs)
8. Token ERC-20 deploye sur Ethereum mainnet
9. Publication spec + outil sur GitHub explodev/cbor-web
10. Depot marque INPI + OEPM

### Tiroirs lies

- #1283 — Vision protocole binaire IA
- #1284 — Probleme du referentiel quantique
- #1300 — text2cbor lance via Framwork
- #1320 — text2cbor compile et operationnel
- #1334 — Business plan complet

### Fichiers cles

- Spec v1.0+v2.0 : `C:\Users\eddie\Downloads\CBOR-WEB-SPEC-v2.0.md`
- Securite : `C:\Users\eddie\Downloads\CBOR-WEB-SECURITY-v1.0.md`
- Business plan : `C:\Users\eddie\Downloads\CBOR-WEB-BUSINESS-PLAN.md` + `~/cbor-web-business-plan.md`
- Site web : `/home/ubuntu/cbor-web-site/` (8 pages, pret a deployer)
- Rapport protocoles : `/home/ubuntu/n4000/rapport-protocoles-m2m-ia.md`
- Rapport securite : `/home/ubuntu/n4000/rapport-securite-cbor-web.md`
- Sources text2cbor : `macpro:/data/builds/text2cbor/`
