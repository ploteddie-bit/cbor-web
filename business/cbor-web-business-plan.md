# CBOR-Web — Business Plan
## Le standard ouvert du web binaire pour l'intelligence artificielle

**Date :** 21 mars 2026
**Auteur :** Eddie Plot — ExploDev / Deltopide SL / Logic Ingénierie
**Confidentiel**

---

## 1. Résumé exécutif

CBOR-Web est un protocole ouvert qui permet aux agents IA de lire les sites web en format binaire au lieu de HTML. C'est comme passer du courrier papier à l'email — même contenu, transport 10x plus efficace.

Le marché des agents IA explose (ChatGPT, Claude, Gemini, agents autonomes). Tous ces agents lisent le web en HTML — un format conçu pour les humains il y a 30 ans. 90% de ce qu'ils téléchargent est du bruit (CSS, JS, menus, publicités). Ils paient au token chez les fournisseurs IA, donc 90% de bruit = 90% de gaspillage.

CBOR-Web résout ce problème avec un format binaire standardisé, auto-descriptif, sécurisé. Un site qui adopte CBOR-Web sert son contenu aux machines en binaire (parallèlement au HTML pour les humains). Le coût de lecture pour un agent IA est divisé par 10 à 50.

**Positionnement :** ExploDev est le créateur du protocole, le premier implémenteur, et l'opérateur du réseau de vérification.

---

## 2. Le problème

### 2.1 Le coût de lecture du web pour les IA

| Métrique | HTML (aujourd'hui) | CBOR-Web |
|----------|-------------------|----------|
| Taille d'une page typique | 50-200 Ko | 2-10 Ko |
| Signal utile | ~10% | >95% |
| Tokens consommés (LLM) | ~12 000 | ~600 |
| Coût lecture 1 page (Claude) | ~$0.036 | ~$0.002 |
| Coût crawl complet 80 pages | ~$2.88 | ~$0.16 |

**Chiffre clé :** un agent qui lit 1000 pages/jour en HTML dépense ~$36/jour en tokens. En CBOR-Web, ~$2/jour. Économie : $12 000/an par agent.

### 2.2 Le problème des publishers

Les sites web subissent le scraping massif par les IA sans aucune contrepartie :
- Pas de revenus publicitaires (l'agent ne voit pas les pubs)
- Pas de contrôle sur qui scrappe quoi
- Bande passante consommée sans retour
- Aucune donnée sur les agents qui visitent

### 2.3 Le problème de sécurité

Le web M2M (machine-to-machine) n'a pas de modèle de sécurité :
- Les CAPTCHA ne fonctionnent pas pour les agents
- Les API keys sont centralisées et volables
- Pas d'identité machine standard
- Pas de modèle de confiance entre agents et sites

---

## 3. La solution

### 3.1 Le protocole CBOR-Web

Un standard ouvert (comme TCP/IP, HTTP, SMTP) qui définit :
- **Un format binaire** basé sur CBOR (RFC 8949, standard IETF)
- **Une découverte automatique** via `/.well-known/cbor-web`
- **3 niveaux de documents** : Manifest (index du site), Pages (contenu), Bundle (tout en 1 requête)
- **Du contenu structuré** : headings, paragraphes, liens, images, vidéos, commerce
- **Des blocs génériques** : templates, schemas, endpoints API, code exécutable
- **Une sécurité 8 couches** : identité blockchain, Proof-of-Work, signatures, sandbox
- **Un modèle économique** : token utilitaire pour rémunérer les vérificateurs

### 3.2 L'outil text2cbor

Un outil open source (Rust) qui convertit automatiquement un site HTML en CBOR-Web. Le publisher installe `text2cbor`, pointe vers son site, et obtient une version binaire parallèle.

### 3.3 Le réseau de vérificateurs

Un réseau décentralisé qui vérifie l'identité des agents, valide les preuves PoW, et signe les documents. Rémunéré en tokens CBOR-Web.

---

## 4. Le marché

### 4.1 Taille du marché

| Segment | Taille estimée 2026-2027 |
|---------|-------------------------|
| Agents IA commerciaux (ChatGPT, Claude, Gemini, Perplexity) | 4 acteurs majeurs, millions d'agents |
| Agents IA entreprise (crawlers internes, RAG, veille) | Marché entreprise IA : $150B+ |
| Sites web mondiaux | 200M+ sites actifs |
| Coût scraping IA estimé | $500M-1B/an en tokens gaspillés |

### 4.2 Segments cibles

**Phase 1 — Early adopters (2026)**
- Développeurs d'agents IA (startups, indépendants)
- Sites de documentation technique (déjà pensés pour les machines)
- E-commerce (intérêt direct à être lu par les IA shopping)

**Phase 2 — Croissance (2027)**
- Agences digitales (comme Deltopide — proposent CBOR-Web à leurs clients)
- Médias et éditeurs de contenu
- SaaS et plateformes

**Phase 3 — Masse (2028+)**
- CMS (WordPress, Shopify intègrent CBOR-Web nativement)
- Moteurs de recherche IA
- Standards industriels (santé, finance, gouvernement)

### 4.3 Concurrence

| Concurrent | Format | Modèle | Faiblesse |
|-----------|--------|--------|-----------|
| **llms.txt** | Texte markdown | Gratuit, ouvert | Résumé seulement, pas de contenu complet |
| **Google A2A** | JSON | Google contrôle | Fermé, peut changer les règles |
| **robots.txt** | Texte | Standard historique | Juste des règles d'accès, pas de contenu |
| **sitemap.xml** | XML | Standard historique | Juste des URLs, pas de contenu |
| **OpenAPI/Swagger** | JSON/YAML | APIs uniquement | Ne couvre pas le contenu web |
| **CBOR-Web** | **Binaire CBOR** | **Ouvert + token** | **Nouveau, adoption à construire** |

**Avantage compétitif :** CBOR-Web est le seul standard qui combine contenu complet + binaire + sécurité + modèle économique. Et il est ouvert — pas de vendor lock-in.

---

## 5. Modèle de revenus

### 5.1 Sources de revenus

| Source | Type | Revenus estimés Y1 | Revenus estimés Y3 |
|--------|------|--------------------|--------------------|
| **text2cbor SaaS** | Abonnement mensuel | $5K-20K/mois | $50K-200K/mois |
| **Tokens fondateur** | Appréciation si adoption | Variable | Potentiel significatif |
| **Verifier Network** | Commission par vérification | $1K-5K/mois | $20K-100K/mois |
| **Consultance** | Journées expert | $2K-10K/mois | $5K-20K/mois |
| **Support entreprise** | Contrats de support | $0 | $10K-50K/mois |

### 5.2 text2cbor SaaS — Détail

| Plan | Prix | Contenu |
|------|------|---------|
| **Starter** | Gratuit | 10 pages, conversion manuelle, format S0 |
| **Pro** | €49/mois | 500 pages, conversion auto, webhook on-change, S1 |
| **Business** | €199/mois | 5000 pages, bundle, multilingual, API, S2 |
| **Enterprise** | Sur devis | Illimité, sécurité S3, SLA, support dédié |

### 5.3 Token CBOR-Web — Économie

| Métrique | Valeur |
|----------|--------|
| Supply initiale | 100M tokens |
| Allocation fondateur (Eddie/ExploDev) | 20% (20M tokens) |
| Allocation vérificateurs (rewards) | 40% (40M tokens) |
| Allocation communauté/grants | 20% (20M tokens) |
| Allocation développement | 10% (10M tokens) |
| Réserve stratégique | 10% (10M tokens) |
| Prix initial par token | $0.01 |
| Valorisation initiale | $1M |

Si le réseau atteint 10 000 sites et 1000 agents actifs : demande estimée ~$100K/mois en tokens = pression haussière naturelle.

---

## 6. Go-to-market

### Phase 1 : Proof of Concept (maintenant — juin 2026)

| Action | Responsable | Coût |
|--------|------------|------|
| Finaliser spec v3.0 | Claude (claude.ai) | $0 (abonnement Max existant) |
| Adapter text2cbor à la spec | Claude (SG Desktop) | $0 |
| Déployer sur les sites Deltopide (FR/ES/EN) | Eddie + Claude | $0 |
| Publier la spec sur GitHub | Eddie | $0 |
| Article blog + post LinkedIn | Eddie | $0 |
| Soumettre un draft à l'IETF | Eddie | $0 |

**Coût Phase 1 : $0** — tout est faisable avec l'infra existante.

### Phase 2 : Early Adoption (juillet — décembre 2026)

| Action | Responsable | Coût |
|--------|------------|------|
| Plugin WordPress cbor-web | Développement | €2K-5K |
| text2cbor SaaS (hébergé) | Infra Cloudflare Workers | €20/mois |
| Déployer le token sur IOTA/L2 | Smart contract | €100-500 |
| Premier verifier node sur l'infra ExploDev | Pi5 + MacPro | $0 (déjà payé) |
| Démarcher 50 sites early adopters | Eddie | Temps |
| Talk à un meetup/conférence tech | Eddie | Déplacement |

**Coût Phase 2 : €3K-6K**

### Phase 3 : Croissance (2027)

| Action | Coût estimé |
|--------|-------------|
| Embaucher 1 dev Rust (mi-temps ou freelance) | €30K-50K/an |
| Marketing + communauté | €10K/an |
| Infra Verifier Network (scaling) | €5K/an |
| Certification sécurité (si marché enterprise) | €10K-20K |

**Coût Phase 3 : €55K-85K/an** — financé par les revenus SaaS + token.

---

## 7. Équipe et ressources

### 7.1 Fondateur

**Eddie Plot**
- Entrepreneur franco-espagnol (Deltopide SL Espagne, Logic Ingénierie France)
- Ingénieur systèmes, architecte infrastructure
- Infrastructure opérationnelle : 7 machines, 6 agents IA autonomes, broker M2M
- Créateur du concept CBOR-Web et du protocole

### 7.2 Infrastructure existante (déjà payée)

| Ressource | Usage CBOR-Web |
|-----------|---------------|
| MacPro | Build server Rust (compilation text2cbor) |
| MiniPC + Caddy | Hébergement sites Deltopide = premiers adopteurs |
| Pi5 | Premier Verifier node |
| FramLocal | Mistral 24B pour analyse/enrichissement |
| N4000 | Code review automatisée |
| 6 agents Claude Code | Automatisation dev/test/déploiement |

### 7.3 IA comme co-équipier

| Instance | Rôle projet |
|----------|-------------|
| Claude SG (Desktop) | Développement text2cbor, intégration, déploiement |
| Claude claude.ai | Rédaction spec, review, fusion docs |
| Claude reviewer | Audit qualité spec |
| Framwork Pipeline | Génération outils Rust |

---

## 8. Propriété intellectuelle

| Élément | Protection |
|---------|-----------|
| Spécification CBOR-Web | CC BY 4.0 (ouverte, attribution requise) |
| text2cbor (outil) | MIT License (open source) |
| Marque "CBOR-Web" | À déposer (INPI France + OEPM Espagne) |
| Token CBOR-Web | Smart contract publié (blockchain = preuve d'antériorité) |
| Nom de domaine | cbor-web.org / cbor-web.com à réserver |

**Pourquoi open source ?** Un standard fermé ne sera pas adopté. TCP/IP, HTTP, SMTP sont ouverts — c'est pour ça que tout le monde les utilise. L'argent ne vient pas de la spec mais de l'écosystème autour (outils, services, tokens, expertise).

---

## 9. Risques

| Risque | Probabilité | Impact | Mitigation |
|--------|------------|--------|------------|
| Google impose A2A comme standard | Moyenne | Fort | CBOR-Web est ouvert et indépendant — complémentaire, pas concurrent |
| IOTA disparaît ou change | Faible | Moyen | Fallback did:key + support multi-blockchain |
| Pas d'adoption | Moyenne | Fort | Les sites Deltopide sont les premiers adopteurs (dog-fooding) |
| Un concurrent copie l'idée | Moyenne | Faible | CC BY 4.0 = attribution obligatoire + avance de 6-12 mois |
| Réglementation crypto/token | Faible | Moyen | Token utilitaire (pas un security token) = régulation légère |
| Le marché des agents IA ralentit | Très faible | Fort | Tendance lourde, tous les indicateurs en hausse |

---

## 10. Projections financières

### Scénario conservateur

| Métrique | Y1 (2026) | Y2 (2027) | Y3 (2028) |
|----------|-----------|-----------|-----------|
| Sites CBOR-Web | 10 (Deltopide) | 200 | 5 000 |
| Agents actifs | 5 | 100 | 2 000 |
| Revenu SaaS | €2K | €30K | €200K |
| Revenu tokens | €0 | €5K | €50K |
| Revenu consultance | €5K | €20K | €40K |
| **Revenu total** | **€7K** | **€55K** | **€290K** |
| Coûts | €1K | €60K | €90K |
| **Résultat** | **+€6K** | **-€5K** | **+€200K** |

### Scénario optimiste (un CMS majeur adopte CBOR-Web)

| Métrique | Y1 | Y2 | Y3 |
|----------|-----|-----|-----|
| Sites CBOR-Web | 10 | 2 000 | 100 000 |
| Revenu total | €7K | €200K | €2M+ |
| Valeur token (si adoption massive) | $0.01 | $0.10 | $1+ |
| Valeur allocation fondateur (20M tokens) | $200K | $2M | $20M+ |

---

## 11. Prochaines étapes immédiates

| # | Action | Quand | Qui |
|---|--------|-------|-----|
| 1 | Finaliser spec v3.0 (fusion + corrections + économie) | Cette semaine | Claude claude.ai |
| 2 | Adapter text2cbor à la spec v3.0 | Après spec | Claude SG |
| 3 | Déployer CBOR-Web sur deltopide.fr/es/com | Après text2cbor | Eddie + Claude |
| 4 | Réserver cbor-web.org et cbor-web.com | Immédiat | Eddie |
| 5 | Publier spec + outil sur GitHub | Après déploiement | Eddie |
| 6 | Déposer marque CBOR-Web (INPI + OEPM) | Avril 2026 | Eddie |
| 7 | Premier post LinkedIn / Hacker News | Après GitHub | Eddie |
| 8 | Déployer token sur testnet IOTA | Quand spec économie finalisée | Claude SG |

---

*"Le meilleur moment pour planter un arbre c'était il y a 20 ans. Le deuxième meilleur moment c'est maintenant."*

Eddie a planté l'arbre le 20 mars 2026.
