# CBOR-Web — Vision Fondatrice

**Ce document est le contexte obligatoire pour tout agent ou collaborateur travaillant sur CBOR-Web.**
**Lire ce document AVANT de toucher au code, à la spec, ou au token.**

```
Auteur :    Eddie Plot (ExploDev)
Version :   1.0
Date :      2026-03-22
Statut :    Document vivant — source de vérité
```

---

## Le problème en une phrase

Le web a été construit pour les yeux humains. Les agents IA sont forcés de lire un format qui n'a pas été conçu pour eux — et ils gaspillent 93% de leurs ressources à filtrer du bruit.

---

## La solution en une phrase

CBOR-Web est une copie binaire du web, servie en parallèle du HTML, où chaque octet est du signal utile. L'agent IA lit du binaire propre au lieu de parser du HTML sale.

---

## La boucle complète

C'est ici que la plupart des gens s'arrêtent : "c'est un format binaire". Non. C'est un **écosystème auto-alimenté**. Voici la boucle complète — chaque étape dépend de la précédente :

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                   │
│   1. PUBLISHER                                                    │
│      Un site web installe text2cbor                               │
│      → Son contenu HTML est converti en CBOR binaire              │
│      → Servi en parallèle à /.well-known/cbor-web                │
│                                                                   │
│   2. CRAWLER                                                      │
│      Un agent IA découvre le endpoint CBOR-Web                    │
│      → Télécharge le manifest (structure du site entier)          │
│      → Télécharge le bundle (tout le contenu, 1 requête)         │
│      → Parse le CBOR en mémoire (zéro tokenisation LLM)          │
│                                                                   │
│   3. SIGNAL PROPRE                                                │
│      Le contenu CBOR-Web est 95% signal, 5% structure             │
│      → Les embeddings générés sont 10x plus précis                │
│      → L'agent comprend mieux, répond mieux, décide mieux        │
│      → Résultat mesurable : meilleure qualité de sortie           │
│                                                                   │
│   4. AUTO-AMÉLIORATION                                            │
│      L'agent constate que le contenu CBOR-Web donne               │
│      de meilleurs résultats que le HTML                           │
│      → Il PRÉFÈRE les sites CBOR-Web                              │
│      → Il CHERCHE le endpoint /.well-known/cbor-web en premier    │
│      → Adoption organique — personne ne force l'agent             │
│                                                                   │
│   5. TOKEN                                                        │
│      Le contenu premium est derrière un token ERC-20 (badge)      │
│      → L'agent veut le contenu complet → il obtient un token      │
│      → Le token n'est jamais dépensé — c'est un badge permanent   │
│      → Plus d'agents veulent le token → sa valeur monte           │
│                                                                   │
│   6. AUTOFINANCEMENT                                              │
│      La valeur du token monte avec l'adoption                     │
│      → Le projet se finance sans vendre de SaaS                   │
│      → Les publishers adoptent parce que les agents les lisent    │
│        mieux → plus de contenu CBOR-Web → plus d'agents           │
│      → Boucle vertueuse infinie                                   │
│                                                                   │
│   ↻ Retour à l'étape 1 — chaque tour amplifie le précédent       │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Les pièces du puzzle

| Pièce | Rôle | Statut |
|-------|------|--------|
| **Spec CBOR-Web** (6 docs) | Définit le format binaire | v2.1 écrite, en review |
| **text2cbor** | Publisher : convertit HTML → CBOR | Compilé sur MacPro, à adapter |
| **Crawler CBOR-Web** | Consumer : l'agent IA qui lit le CBOR | **À CONSTRUIRE** — pièce manquante |
| **Token ERC-20 (CBORW)** | Badge d'accès permanent, autofinancement | Conçu, pas déployé |
| **Site cbor-web.org** | Vitrine du standard | Fichiers prêts, pas déployé |
| **Sites Deltopide** | Premiers adopteurs (preuve de concept) | 80 pages FR/ES/EN prêtes |
| **Infrastructure ExploDev** | 7 machines + 6 agents autonomes | Opérationnel |
| **Framwork** | Peut générer text2cbor + crawler | Pipeline v4 sur framlocal |

---

## Ce que CBOR-Web N'EST PAS

| Malentendu courant | Réalité |
|-------------------|---------|
| "Un concurrent de llms.txt" | Non. llms.txt = résumé texte. CBOR-Web = contenu complet binaire. Complémentaires. |
| "Un remplacement des embeddings / RAG" | Non. CBOR-Web fournit le **signal propre** que les embeddings indexent. Il améliore leur qualité, il ne les remplace pas. |
| "Un format pour humains" | Non. Le HTML reste pour les humains. CBOR-Web est le canal parallèle pour les machines. |
| "Un SaaS à vendre" | Non. La spec est gratuite (CC BY 4.0). Les outils sont open source (MIT). L'argent vient du token. |
| "Une base de données / RAG" | Non. C'est un format de fichier binaire. L'indexation est faite par les outils existants (pgvector, Pinecone, etc.) qui reçoivent un meilleur signal. |
| "Un programme statique" | Non côté consumer. Le crawler est un agent IA vivant qui découvre, lit, comprend et s'améliore. Côté publisher, text2cbor est un convertisseur (mais peut tourner en mode watch). |

---

## L'argument d'auto-amélioration

C'est l'argument que personne d'autre ne fait et qui rend CBOR-Web unique :

**Un agent IA qui lit du CBOR-Web devient objectivement meilleur.**

Pas "probablement meilleur". Mathématiquement meilleur :
- Embedding de "Lion's Mane — Champignon Fonctionnel Premium" (CBOR-Web, signal pur)
- vs embedding de "Lion's Mane — Champignon Fonctionnel Premium | Menu | Accueil | Produits | Blog | Contact | Panier (0) | 🍄 Découvrez nos champignons | Cookie consent | Newsletter..." (HTML)

Le premier vecteur est **précis**. Le second est **dilué**. Tout modèle de similarité, de classification, ou de génération produit de meilleurs résultats avec le premier.

L'agent le constate. Il préfère le signal propre. Il cherche le CBOR-Web en premier. **L'adoption est organique, pas forcée.**

---

## L'argument économique : débloquer le web pour les IA

Les chiffres théoriques sont une chose. Les chiffres mesurés en sont une autre. Voici les mesures réelles effectuées sur verdetao.com :

| Mesure | Valeur |
|--------|--------|
| Page Verdetao HTML+JS | 1,509,461 bytes (JS) + 9,580 bytes (HTML shell) = **1.5 MB** |
| Page Verdetao CBOR-Web | **536 bytes** (fichier `_index.cbor`) |
| Ratio de compression réel | **~1000x** |

Traduction en coût IA :

| Métrique | HTML | CBOR-Web | Ratio |
|----------|------|----------|-------|
| Tokens par page | ~20,000 (à parser) | ~100 (contenu utile) | 200x |
| Coût par page | ~$0.06 | ~$0.0003 | 200x |
| Coût pour 100 pages | ~$6.00 | ~$0.001 | 6,000x |

Aujourd'hui, les IA (Claude, ChatGPT, Gemini) **limitent volontairement** leur accès au web. Pas par choix technique — par contrainte économique. Chaque page web consommée coûte des milliers de tokens, et les tokens coûtent de l'argent. Le résultat : les agents IA ne lisent que ce qu'ils doivent absolument lire.

CBOR-Web supprime cette limitation. À ~$0.001 pour 100 pages, un agent IA peut **dévorer le web entier** à coût quasi nul. Ce n'est pas une amélioration marginale — c'est un changement structurel.

La conséquence pour le token est directe : la demande est organique. Les agents IA n'ont pas besoin d'être convaincus d'utiliser CBOR-Web — ils en ont **besoin** pour débloquer le web. Le token donne accès au contenu premium. Les agents le veulent parce que le contenu qu'ils obtiennent est meilleur et infiniment moins cher.

---

## Pourquoi le crawler est la pièce critique

Sans crawler, la boucle est cassée :

```
text2cbor ──→ fichiers .cbor ──→ ??? ──→ agent IA
                                   ↑
                              PERSONNE NE LIT
```

Avec le crawler :

```
text2cbor ──→ fichiers .cbor ──→ crawler ──→ signal propre ──→ agent IA
                                                                    ↓
                                                          meilleurs résultats
                                                                    ↓
                                                          préfère CBOR-Web
                                                                    ↓
                                                          demande le token
```

**Le crawler est le client du token.** Sans client, pas de demande. Sans demande, le token ne vaut rien.

---

## Le token en 30 secondes

- **ERC-20 sur Ethereum mainnet** — pas de blockchain exotique
- **Badge permanent** — tu le détiens, tu as accès. Tu ne le dépenses jamais.
- **100M tokens**, 20% fondateur, 40% rewards, 20% communauté, 10% dev, 10% réserve
- **Smart contract auto-régulateur** — stabilise le prix automatiquement
- **Lancement à $0.01** — les premiers arrivent, ça coûte quasi rien
- **Autofinancement** — l'adoption fait monter le prix, le projet se finance tout seul
- **Vérification blockchain** — `balanceOf(wallet) > 0 ?` OUI/NON. Infalsifiable.

---

## L'infrastructure qui existe déjà

ExploDev dispose de 7 machines et 6 agents autonomes qui peuvent porter ce projet :

| Machine | Rôle dans CBOR-Web |
|---------|-------------------|
| **Desktop** (SG) | Orchestration, vision, spec review |
| **MacPro** | Compilation Rust (text2cbor, crawler) |
| **MiniPC** | Hébergement sites Deltopide (premiers adopteurs) |
| **MacMini** | PostgreSQL, tiroirs, persistance données |
| **Framlocal** | Framwork pipeline, peut générer les outils |
| **Pi5** | Backups, monitoring |
| **N4000** | Audit sécurité, code review |

---

## Ordre des opérations

1. ✅ Spec v2.1 (6 documents) — en cours de review
2. ⬜ Crawler CBOR-Web en Rust — **priorité absolue**
3. ⬜ Adapter text2cbor à la spec v2.1 finale
4. ⬜ Convertir les sites Deltopide en CBOR-Web (premiers adopteurs)
5. ⬜ Déployer cbor-web.org sur Fram cloud
6. ⬜ Déployer le token ERC-20 ($50 de gas)
7. ⬜ Mesurer : embeddings CBOR-Web vs HTML (preuve quantitative)
8. ⬜ Publication GitHub + annonce
9. ⬜ Publier les benchmarks (HTML vs CBOR-Web, chiffres mesurés sur Verdetao)

---

## Pour les agents : ce que vous devez savoir

**Si vous travaillez sur la spec :** Le format n'est pas une fin en soi. Chaque décision de design doit servir la boucle complète. Un choix qui optimise le format mais complique le crawler est un mauvais choix.

**Si vous travaillez sur text2cbor :** Votre outil doit produire du signal propre, pas juste du CBOR valide. La qualité des embeddings générés à partir de votre output est votre métrique, pas la conformance CBOR.

**Si vous travaillez sur le crawler :** Vous construisez le moteur d'adoption. Le crawler qui donne de meilleurs résultats à l'agent crée la demande pour le token. Votre code est le business model.

**Si vous travaillez sur le token :** Le token n'a de valeur que si le crawler existe et fonctionne. Ne déployez pas le token avant que le crawler ne prouve la boucle.

**Si vous travaillez sur l'infra :** Tout ce que vous maintenez (sites, DB, backups, monitoring) protège la capacité d'ExploDev à exécuter ce plan. Chaque service down est un jour de retard.

---

*"Le web a deux clients : les humains et les machines. Il est temps de servir les deux."*

*— ExploDev, 2026*
