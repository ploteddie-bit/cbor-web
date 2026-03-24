# Rapport : Bourse, Crypto, Tokens et Modele CBOR-Web

> Rapport de recherche — 21 mars 2026
> Redige pour quelqu'un qui n'a jamais fait de trading.
> Tous les chiffres et exemples sont reels et verifies.

---

## TABLE DES MATIERES

1. [Fonctionnement de la Bourse — les bases](#1-fonctionnement-de-la-bourse--les-bases)
2. [Crypto / Tokens — specifiquement](#2-crypto--tokens--specifiquement)
3. [Mecanismes de stabilisation du prix](#3-mecanismes-de-stabilisation-du-prix)
4. [Application a un token utilitaire comme CBOR-Web](#4-application-a-un-token-utilitaire-comme-cbor-web)
5. [Le modele specifique CBOR-Web — analyse](#5-le-modele-specifique-cbor-web--analyse)

---

## 1. FONCTIONNEMENT DE LA BOURSE — LES BASES

### 1.1 Comment un prix monte ou baisse (offre et demande)

Le prix d'un actif (action, crypto, token) se resume a une seule chose : **l'equilibre entre acheteurs et vendeurs**.

- **Plus d'acheteurs que de vendeurs** → les acheteurs se font concurrence, acceptent de payer plus cher → **le prix monte**
- **Plus de vendeurs que d'acheteurs** → les vendeurs se font concurrence, acceptent de vendre moins cher → **le prix baisse**

Le prix affiche a un instant T represente **le dernier accord entre un acheteur et un vendeur**. C'est la derniere transaction executee.

**Exemple concret :** imagine un marche aux puces. Un vendeur propose une lampe a 50 EUR. Personne n'en veut. Il baisse a 40 EUR. Un acheteur dit "OK a 35 EUR". Le vendeur accepte. Le "prix du marche" de cette lampe est maintenant 35 EUR. Si 10 personnes veulent la meme lampe, le vendeur n'a pas besoin de baisser — il peut meme monter a 60 EUR.

### 1.2 Le carnet d'ordres (order book)

Le carnet d'ordres est **la liste en temps reel de tous les ordres d'achat et de vente** en attente sur un marche.

Il se compose de :

| Terme | Definition |
|-------|-----------|
| **Bid** (offre d'achat) | Le prix le plus haut qu'un acheteur est pret a payer |
| **Ask** (offre de vente) | Le prix le plus bas qu'un vendeur est pret a accepter |
| **Spread** | L'ecart entre le bid et l'ask |
| **Last price** | Le prix de la derniere transaction executee |

**Comment ca marche :**
1. Un acheteur pose un ordre : "Je veux acheter 100 tokens a 2,50 EUR"
2. Un vendeur pose un ordre : "Je veux vendre 50 tokens a 2,55 EUR"
3. Tant que le bid (2,50) est inferieur a l'ask (2,55), aucune transaction ne se fait
4. Quand un acheteur accepte le prix du vendeur (ou inversement), **la transaction s'execute**
5. Ce nouveau prix devient le "prix du marche"

**Le spread** (ecart bid/ask) est un indicateur de liquidite. Un spread etroit (0,01 EUR) = marche liquide et actif. Un spread large (0,50 EUR) = marche peu liquide, peu d'activite.

### 1.3 Achat (buy) et vente (sell) — comment ca fonctionne concretement

Il existe deux types d'ordres principaux :

**Ordre au marche (market order) :**
- "J'achete/je vends MAINTENANT au meilleur prix disponible"
- Execution immediate, mais tu ne controles pas le prix exact
- Utilise quand la vitesse compte plus que le prix

**Ordre limite (limit order) :**
- "J'achete/je vends SEULEMENT a ce prix ou mieux"
- Pas d'execution garantie — ton ordre attend dans le carnet
- Utilise quand le prix exact compte plus que la vitesse

**Exemple :** le token X vaut 10 EUR (dernier prix).
- **Market buy** : tu achetes immediatement au meilleur ask disponible (peut-etre 10,02 EUR)
- **Limit buy a 9,80 EUR** : ton ordre attend. Si le prix descend a 9,80, il s'execute. Sinon, rien ne se passe.

### 1.4 Les market makers — qui sont-ils et quel role

Un **market maker** (teneur de marche) est une entite qui **se tient prete a acheter ET vendre en permanence**. Son role est d'assurer qu'il y a toujours quelqu'un en face quand tu veux acheter ou vendre.

**Comment ils gagnent de l'argent :**
- Ils affichent un prix d'achat (bid) et un prix de vente (ask) en meme temps
- Le spread entre les deux est leur marge
- Exemple : ils achetent a 9,98 EUR et vendent a 10,02 EUR → ils gagnent 0,04 EUR par transaction

**Pourquoi c'est important :**
- Sans market makers, tu pourrais vouloir vendre et **n'avoir personne en face**
- Ils reduisent la volatilite (les prix bougent moins brutalement)
- Ils assurent la liquidite — tu peux toujours entrer ou sortir du marche

**Exemples reels :** Citadel Securities, Virtu Financial, Jump Trading sont parmi les plus grands market makers au monde. Dans la crypto, Wintermute, GSR Markets, et DWF Labs jouent ce role.

### 1.5 Bull market vs bear market

| Terme | Definition | Caracteristiques |
|-------|-----------|-----------------|
| **Bull market** (marche haussier) | Les prix montent de facon soutenue (+20% ou plus) | Confiance elevee, beaucoup d'acheteurs, medias enthousiastes |
| **Bear market** (marche baissier) | Les prix baissent de facon soutenue (-20% ou plus) | Pessimisme, ventes massives, medias negatifs |

**Pourquoi ces noms ?** Le taureau (bull) frappe de bas en haut avec ses cornes. L'ours (bear) frappe de haut en bas avec ses griffes.

**En crypto specifiquement :**
- Les bull markets montent plus vite et plus haut qu'en bourse traditionnelle
- Les bear markets descendent plus fort et durent plus longtemps psychologiquement
- Historiquement, un cycle complet (bull + bear) dure environ 3-4 ans en crypto
- Exemple : Bitcoin est passe de 69 000 $ (novembre 2021) a 15 500 $ (novembre 2022) — un bear market de -77%

### 1.6 La volatilite — pourquoi les prix bougent

La **volatilite** mesure **l'amplitude des variations de prix** sur une periode donnee. Un actif tres volatil peut gagner ou perdre 10% en un jour. Un actif peu volatil bouge de 0,5%.

**Causes de la volatilite :**
- **Actualites** : une nouvelle regulatoire, un hack, un tweet d'un personnage influent
- **Liquidite faible** : moins il y a d'acheteurs/vendeurs, plus un seul gros ordre fait bouger le prix
- **Speculation** : les traders parient sur la hausse ou la baisse, amplifiant les mouvements
- **Effet de levier** : les traders empruntent pour amplifier leurs positions → liquidations en cascade
- **Sentiment de marche** : la peur (Fear) et la cupidite (Greed) sont mesurables (Fear & Greed Index)

**Chiffres concrets :**
- La volatilite annualisee du Bitcoin est d'environ 50-80%
- Celle du S&P 500 (bourse US) est d'environ 15-20%
- Un petit token peut avoir une volatilite de 200-500%

### 1.7 Les indicateurs de base

| Indicateur | Definition | Pourquoi c'est important |
|-----------|-----------|------------------------|
| **Volume** | Nombre de tokens echanges sur une periode (24h, 7j) | Volume eleve = interet reel. Volume faible = danger (facile a manipuler) |
| **Capitalisation (market cap)** | Prix du token x nombre de tokens en circulation | Donne la "taille" du projet. Bitcoin : ~1 300 Mds $. Un petit token : quelques millions |
| **Liquidite** | Facilite avec laquelle tu peux acheter/vendre sans impacter le prix | Faible liquidite = slippage (tu paies plus cher ou vends moins cher que prevu) |
| **FDV (Fully Diluted Valuation)** | Prix x supply totale (tous tokens, meme ceux pas encore en circulation) | Montre la valorisation "reelle" si tous les tokens etaient deja distribues |

**Exemple concret :**
- Chainlink (LINK) : market cap ~9 Mds $, volume 24h ~400 M $, supply circulante 678 M sur 1 Md total
- Un token obscur : market cap 500 000 $, volume 24h 2 000 $. Tu ne peux pas vendre 10 000 $ d'un coup sans faire chuter le prix.

---

## 2. CRYPTO / TOKENS — SPECIFIQUEMENT

### 2.1 Comment un token ERC-20 est cree et mis en circulation

Un token **ERC-20** est un standard technique sur la blockchain Ethereum. "ERC" = Ethereum Request for Comments, "20" = numero de la proposition.

**Ce que le standard definit :**
- Les fonctions de base : transferer des tokens, verifier un solde, approuver une depense
- La compatibilite avec tous les wallets Ethereum (MetaMask, etc.)
- La compatibilite avec tous les exchanges et protocoles DeFi

**Etapes de creation :**
1. **Ecrire le smart contract** en Solidity (langage de programmation Ethereum)
   - On definit : nom, symbole, supply totale, decimales
   - On peut utiliser OpenZeppelin (bibliotheque open source auditee) pour les fonctions standard
2. **Deployer le contrat** sur Ethereum mainnet
   - Cout en gas : entre 100 $ et 500 $ selon la congestion reseau (en 2025-2026, plutot 100-200 $ grace aux optimisations recentes)
   - Le deploiement cree le contrat sur la blockchain — c'est irreversible
3. **Les tokens existent** a l'adresse du deploieur
   - La supply totale est creee d'un coup ou progressivement (selon le code)
   - Le createur peut ensuite les distribuer, les vendre, les envoyer

**Cout reel en 2025-2026 :**
- Deploiement du contrat : 100-500 $
- Audit de securite (recommande) : 5 000 - 50 000 $ selon la complexite
- Si on utilise un framework comme OpenZeppelin, le code de base est gratuit et eprouve

### 2.2 Comment un token est liste sur un exchange

Il y a deux chemins tres differents :

**DEX (exchange decentralise) — Uniswap, PancakeSwap, SushiSwap :**
- **Aucune autorisation necessaire** — c'est "permissionless"
- Etapes :
  1. Aller sur Uniswap, connecter son wallet
  2. Coller l'adresse du contrat du token
  3. Creer un "liquidity pool" (paire token/ETH ou token/USDC)
  4. Deposer des tokens + de l'ETH (ou USDC) pour creer le marche
- Delai : **quelques minutes**
- Cout : le gas pour creer le pool (50-200 $) + la liquidite initiale que tu deposes
- Le token est tradable immediatement apres

**CEX (exchange centralise) — Binance, Coinbase, Kraken :**
- **Processus de candidature** formel
- Etapes :
  1. Remplir un formulaire de listing (business plan, tokenomics, equipe, audit)
  2. Due diligence par l'exchange (verification legale, technique, financiere)
  3. Negociation des frais de listing
  4. Integration technique
- Delai : **quelques semaines a plusieurs mois**
- Cout :
  - Exchanges "tier 2" (MEXC, Gate.io) : 50 000 - 200 000 $
  - Binance : **1 000 000 $+ (non officiel, varie enormement)**
  - Coinbase : processus "gratuit" mais tres selectif

**Strategie typique :** Lancer sur DEX d'abord (cout faible, acces immediat) → prouver la traction → postuler aux CEX pour la visibilite.

### 2.3 Les liquidity pools — comment ca marche

Un **liquidity pool** (pool de liquidite) est un **reservoir de deux tokens verrouilles dans un smart contract** qui permet aux gens d'echanger entre ces deux tokens sans carnet d'ordres.

**Fonctionnement :**
1. Quelqu'un (le "liquidity provider") depose une paire de tokens. Exemple : 10 000 CBOR + 5 ETH
2. Le smart contract devient un "marche" ou n'importe qui peut echanger du CBOR contre de l'ETH et vice-versa
3. Le prix est determine automatiquement par une formule mathematique (pas par un humain)
4. Chaque echange paie une commission (typiquement 0,3% sur Uniswap) qui va aux liquidity providers

**Exemple concret sur Uniswap :**
- Pool : 100 000 USDC + 50 000 tokens CBOR
- Prix implicite : 1 CBOR = 2 USDC
- Tu veux acheter 1 000 CBOR → tu deposes des USDC, le pool te donne des CBOR
- Apres ton achat, il y a moins de CBOR dans le pool → le prix du CBOR monte

### 2.4 Le concept de Liquidity Provider (LP)

Un **Liquidity Provider** (fournisseur de liquidite) est quelqu'un qui **depose ses tokens dans un pool pour permettre aux autres de trader**.

**Pourquoi quelqu'un ferait ca ?**
- Il gagne une part des commissions de trading (0,3% de chaque echange, repartie entre tous les LP)
- Sur un pool actif avec beaucoup de volume, les gains peuvent etre significatifs

**Le risque : l'Impermanent Loss (perte temporaire)**
- Si le prix d'un des deux tokens change beaucoup par rapport a l'autre, le LP peut se retrouver avec moins de valeur que s'il avait simplement garde ses tokens
- Exemple : tu deposes 50/50 ETH+CBOR. Si ETH double et CBOR ne bouge pas, tu aurais gagne plus en gardant tes ETH plutot qu'en les mettant dans le pool
- Ce phenomene s'appelle "impermanent loss" car la perte disparait si les prix reviennent a l'equilibre initial

### 2.5 Comment le prix est determine sur un DEX (formule x * y = k)

La formule **x * y = k** est le coeur du systeme **AMM (Automated Market Maker)** invente par Uniswap.

**Les variables :**
- **x** = quantite du token A dans le pool
- **y** = quantite du token B dans le pool
- **k** = une constante (le produit des deux ne change jamais)

**Exemple chiffre :**
```
Pool initial : 10 000 CBOR (x) et 50 ETH (y)
k = 10 000 x 50 = 500 000

Prix implicite : 50 / 10 000 = 0,005 ETH par CBOR

Quelqu'un veut acheter 1 000 CBOR :
- Nouveau x = 9 000 CBOR (10 000 - 1 000)
- k doit rester 500 000
- Nouveau y = 500 000 / 9 000 = 55,56 ETH
- L'acheteur doit deposer : 55,56 - 50 = 5,56 ETH

Prix moyen paye : 5,56 ETH / 1 000 CBOR = 0,00556 ETH par CBOR
(11% plus cher que le prix initial de 0,005 — c'est le "slippage")
```

**Points cles :**
- Plus le pool est gros (beaucoup de liquidite), moins le prix bouge a chaque echange
- Plus le pool est petit, plus un seul achat/vente fait bouger le prix
- Le prix s'ajuste automatiquement a chaque transaction — pas besoin de market maker humain

### 2.6 Initial DEX Offering (IDO) — comment lancer un token

Un **IDO** est une methode de levee de fonds ou un projet lance ses tokens **directement sur un exchange decentralise**.

**Le processus :**
1. **Preparation** : le projet definit sa tokenomics, son smart contract, son whitepaper
2. **Choix d'une plateforme de lancement (launchpad)** : DAO Maker, Polkastarter, Seedify, Fjord Foundry
3. **Whitelist** : les investisseurs s'inscrivent. La demande depasse souvent l'offre → seuls certains sont selectionnes (allocations de quelques centaines de dollars)
4. **Token Generation Event (TGE)** : le smart contract s'active, les tokens sont distribues
5. **Creation du pool de liquidite** : une partie des fonds leves est automatiquement pairee avec les tokens pour creer le marche
6. **Trading immediat** : les tokens sont echangeables des la fin de l'IDO

**Alternative pour petit budget — le Liquidity Bootstrapping Pool (LBP) :**
- Invente par Balancer (protocole DeFi)
- Commence avec un ratio desequilibre (ex : 99% token / 1% ETH)
- Le ratio s'inverse progressivement sur quelques jours
- Le prix commence haut et descend naturellement
- Les acheteurs entrent quand le prix atteint un niveau acceptable → **decouverte du prix par le marche**
- Avantage : **necessite tres peu de capital initial**

### 2.7 Tokenomics — comment structurer l'offre, la distribution, le vesting

La **tokenomics** ("token economics") est l'architecture economique d'un token : combien, pour qui, quand, pourquoi.

**Les elements a definir :**

| Element | Definition | Benchmark 2025 |
|---------|-----------|----------------|
| **Total supply** | Nombre total de tokens qui existeront | Variable (100M a 10Mds typiquement) |
| **Circulating supply** | Tokens effectivement en circulation | 10-30% au lancement, typiquement |
| **Allocation equipe/fondateurs** | Part reservee aux createurs | 18-20% (avec vesting !) |
| **Allocation investisseurs** | Part vendue aux investisseurs prives | 12-18% |
| **Tresorerie/Reserve** | Reserve pour le futur | 20-25% |
| **Ecosysteme/Communaute** | Recompenses, airdrops, incentives | 35-45% |
| **Vente publique** | Tokens vendus au public (IDO, etc.) | 1-5% |
| **Advisors/Partners** | Conseillers et partenaires | 1-3% |

**Le vesting :**
Le **vesting** est le calendrier de deblocage progressif des tokens.
- **Cliff** : periode initiale ou aucun token n'est debloque (typiquement 6-12 mois)
- **Linear vesting** : deblocage mensuel regulier apres le cliff (sur 24-36 mois)
- Exemple courant : "12 mois de cliff + 36 mois de vesting lineaire"
  - Mois 1 a 12 : 0 token debloque
  - Mois 12 : 25% debloques d'un coup
  - Mois 13 a 48 : deblocage mensuel regulier des 75% restants

**Pourquoi c'est crucial :** 70% des projets utilisent le vesting lineaire. Les projets sans vesting voient souvent un dump (vente massive) des qu'un gros detenteur peut vendre.

### 2.8 Token utilitaire vs security token (implications legales)

| Critere | Token utilitaire | Security token |
|---------|-----------------|----------------|
| **Objectif** | Donner acces a un service/produit | Representer un investissement |
| **Droits** | Usage du service | Dividendes, parts, vote |
| **Regulation** | Legere (MiCA : "non-financier") | Lourde (lois sur les valeurs mobilieres) |
| **Exemples** | BAT, LINK, FIL | tZERO, Securitize |

**Le test de Howey (USA) :**
Un token est considere comme un "security" si ces 4 criteres sont remplis :
1. Il y a un investissement d'argent
2. Dans une entreprise commune
3. Avec une attente de profits
4. Provenant des efforts d'un tiers

**Attention :** en 2022, 98,7% des tokens commercialises comme "utilitaires" ont ete reclasses en "securities" par la SEC americaine, avec 1,2 Md $ d'amendes. L'etiquette ne suffit pas — c'est la realite du fonctionnement qui compte.

**En Europe (MiCA) :** l'approche est plus claire. MiCA definit explicitement les tokens utilitaires comme "non-financiers" (Article 3(9)). Les obligations sont allegees : pas de prospectus complet si le token donne acces a un service existant et fonctionnel.

---

## 3. MECANISMES DE STABILISATION DU PRIX

### 3.1 Comment les stablecoins maintiennent leur prix

Les stablecoins montrent qu'il est possible de **maintenir un prix stable** dans le monde crypto. Deux modeles dominants :

**USDC (Circle) — adosse a des reserves :**
- Chaque USDC en circulation est couvert par 1 $ de reserves (cash + bons du Tresor US)
- Audits independants reguliers (BlackRock publie les rapports)
- Mecanisme d'arbitrage : si USDC tombe a 0,99 $, des traders l'achetent et le convertissent en 1 $ reel aupres de Circle → profit de 0,01 $ → le prix remonte
- Disponible sur 23 blockchains nativement (en 2025)

**DAI (MakerDAO) — algorithmique et sur-collateralise :**
- Pas adosse a du cash, mais a des **crypto-actifs deposes en garantie**
- Pour creer 100 DAI, tu deposes 150 $ d'ETH (sur-collateralisation de 150%)
- Des smart contracts ajustent automatiquement les taux d'interet pour gerer l'offre et la demande
- Un "Peg Stability Module" agit comme market maker automatique entre DAI et d'autres stablecoins

**Lecon pour un token utilitaire :** ces mecanismes montrent que la stabilite vient de (1) une valeur sous-jacente reelle, (2) des mecanismes d'arbitrage clairs, et (3) de la transparence.

### 3.2 Buyback and Burn (racheter et detruire des tokens)

Le **buyback and burn** consiste a :
1. Le projet utilise une partie de ses **revenus** pour racheter ses propres tokens sur le marche
2. Les tokens rachetes sont envoyes a une "burn address" (adresse sans cle privee) → **detruits definitivement**
3. La supply totale diminue → avec une demande constante, le prix monte (loi de l'offre et de la demande)

**Exemples reels :**
- **Binance (BNB)** : utilise 20% de ses profits pour racheter et bruler du BNB chaque trimestre. En octobre 2021, le 17e burn a detruit 1 335 888 BNB (~600 M $ a l'epoque)
- **Stellar (XLM)** : en 2019, a brule 55 milliards de XLM (plus de 50% de la supply). Le prix a bondi de 25% en une journee.
- **Sky DAO (ex-MakerDAO)** : en fevrier 2025, a lance un programme de rachat de 1 M $ par jour de tokens SKY

**Attention :** le burn ne cree pas de valeur magiquement. Si personne n'utilise le token, bruler des tokens ne sauvera pas le projet. C'est un outil complementaire, pas une solution miracle.

### 3.3 Le staking — bloquer des tokens pour gagner des recompenses

Le **staking** consiste a **verrouiller ses tokens pendant une periode** en echange de recompenses.

**Deux formes principales :**

1. **Staking de validation (Proof of Stake)** :
   - Tu bloques tes tokens pour participer a la validation des transactions de la blockchain
   - Le reseau te recompense avec de nouveaux tokens
   - Exemple : Ethereum → ~4-5% APY, Solana → ~6-7% APY, Cardano → ~2,8-4,5% APY

2. **Staking d'ecosysteme** :
   - Tu bloques tes tokens dans un protocole pour gagner des recompenses
   - Le but : reduire la supply en circulation (et donc stabiliser/augmenter le prix)
   - Chainlink : les node operators stakent du LINK comme garantie de bonne conduite

**Pourquoi c'est utile pour un nouveau token :**
- Reduit la pression de vente (tokens verrouilles = tokens non vendables)
- Cree un engagement a long terme des detenteurs
- Donne une raison de conserver le token au-dela de l'usage

### 3.4 Les vesting schedules — deblocage progressif

Deja couvert en section 2.7, voici les **regles d'or** :

- **Fondateurs/equipe** : 12 mois de cliff minimum + 36 mois de vesting lineaire
- **Ne jamais debloquer tous les tokens en meme temps** pour differentes categories (equipe, investisseurs, advisors) — echelonner les deblocages
- Un mauvais vesting a detruit des projets : si equipe + investisseurs + advisors debloquent le meme mois, c'est un dump garanti

**Chiffre cle :** selon les benchmarks 2025, 70% des projets serieux utilisent un vesting lineaire avec cliff de 12 mois.

### 3.5 Les bonding curves — prix automatique base sur l'offre

Une **bonding curve** est une **formule mathematique qui lie le prix a la quantite de tokens en circulation**.

**Fonctionnement :**
- Le prix est gere par un smart contract, pas par un marche ouvert
- Quand tu achetes un token, il est "minte" (cree) et le prix monte selon la courbe
- Quand tu vends un token, il est "brule" (detruit) et le prix descend
- L'argent est stocke dans un reserve pool automatique

**Types de courbes :**
- **Lineaire** : prix = a x supply. Le prix monte de facon reguliere
- **Exponentielle** : le prix monte de plus en plus vite a mesure que la supply augmente (recompense les premiers acheteurs)
- **Logarithmique** : le prix monte vite au debut puis se stabilise

**Avantages :**
- Liquidite garantie en permanence (tu peux toujours acheter ou vendre)
- Pas besoin de market maker ni de pool de liquidite
- Les premiers acheteurs paient moins → incitatif a l'adoption precoce

**Inconvenient :** moins de liquidite qu'un pool Uniswap bien garni, et le modele est moins connu des investisseurs.

### 3.6 Comment eviter le pump and dump

Un **pump and dump** est un schema frauduleux :
1. Un groupe achete massivement un token a bas prix (pump)
2. Ils creent du battage mediatique (hype sur les reseaux sociaux)
3. Des acheteurs naifs entrent, faisant monter le prix
4. Les manipulateurs vendent tout d'un coup (dump)
5. Le prix s'effondre, les derniers acheteurs perdent tout

**Mecanismes de protection pour un projet serieux :**
- **Vesting strict** : empeche les gros detenteurs de vendre d'un coup
- **Lock-up des fondateurs** : 12+ mois minimum sans pouvoir vendre
- **Liquidite profonde** : plus il y a de liquidite, plus il faut de capital pour manipuler le prix
- **Distribution large** : eviter que 2-3 wallets detiennent 50%+ de la supply
- **Transparence** : publier les adresses des wallets fondateurs, montrer les transactions on-chain
- **Volume minimum** : ne pas lister sur des exchanges a tres faible volume

**Chiffre alarmant :** selon CoinGecko, 53% des tokens lances depuis 2021 sont devenus des "dead coins". En Q1 2025 seul, 1,8 million de tokens se sont effondres.

---

## 4. APPLICATION A UN TOKEN UTILITAIRE COMME CBOR-WEB

### 4.1 Comment structurer le lancement d'un token utilitaire

**Phase 1 — Pre-lancement (3-6 mois avant) :**
- Smart contract audite (budget : 5 000-20 000 $)
- Whitepaper detaille (tokenomics, roadmap, cas d'usage)
- Communaute initiale (Discord, Twitter, blog technique)
- Partenariats avec des projets complementaires

**Phase 2 — Lancement :**
- Option A : **IDO via launchpad** (DAO Maker, Fjord Foundry) — visibilite + processus encadre
- Option B : **Liquidity Bootstrapping Pool** (Balancer) — faible capital requis, decouverte du prix par le marche
- Option C : **Listing direct sur Uniswap** — le plus simple et le moins cher

**Phase 3 — Post-lancement :**
- Monitoring du pool de liquidite
- Communication reguliere (updates produit, metriques d'adoption)
- Programme de staking si pertinent
- Candidature aux CEX quand le volume le justifie

### 4.2 Comment creer de la liquidite initiale sans gros capital

**Methode 1 : Liquidity Bootstrapping Pool (LBP)**
- Necessite seulement 1-5% du capital total en token de collateral (ETH/USDC)
- Ratio de depart : 95/5 ou 99/1 (token/ETH)
- Le marche decouvre le prix naturellement sur 3-7 jours
- Cout total : quelques milliers de dollars en ETH

**Methode 2 : Bootstrapping communautaire**
- Distribuer des tokens a des contributeurs (developpeurs, beta-testeurs, ambassadeurs)
- Ces detenteurs creent organiquement de l'offre et de la demande
- Airdrop cible (pas massif) aux utilisateurs potentiels du service

**Methode 3 : Seeding progressif**
- Commencer avec un petit pool Uniswap (ex : 5 000 $ en ETH + equivalent en tokens)
- Augmenter la liquidite progressivement avec les revenus du projet
- Ajout de paires (CBOR/USDC, CBOR/DAI) au fur et a mesure

**Chiffre de reference :** un pool Uniswap avec 20 000-50 000 $ de liquidite est un minimum pour eviter un slippage excessif sur des ordres de quelques centaines de dollars.

### 4.3 Comment le prix peut monter naturellement avec l'adoption

Le **cercle vertueux** d'un token utilitaire :
1. Le service est utile → des gens veulent l'utiliser
2. Pour l'utiliser, ils doivent acheter le token
3. Plus d'acheteurs que de vendeurs → le prix monte
4. Le prix monte → plus de visibilite → plus d'utilisateurs
5. Plus d'utilisateurs → plus de demande de tokens → le prix continue de monter

**Conditions necessaires :**
- Le service doit avoir une **valeur reelle** et verifiable
- Le token doit etre **necessaire** pour acceder au service (pas optionnel)
- L'offre de tokens doit etre **limitee** ou **deflationniste**
- Le vesting empeche les gros detenteurs de casser le cycle

**Difference cruciale avec la speculation :**
- **Speculation** : le prix monte parce que les gens pensent qu'il va monter (bulle)
- **Adoption** : le prix monte parce que plus de gens utilisent reellement le service (valeur fondamentale)

### 4.4 Exemples de tokens utilitaires qui ont reussi

**Chainlink (LINK) :**
- **Usage** : payer les fournisseurs de donnees (oracles) qui alimentent les smart contracts
- **Supply** : 1 milliard total, ~678 millions en circulation (2025)
- **Distribution** : 35% vente publique, 35% node operators/ecosysteme, 30% equipe
- **Market cap** : ~9-17 Mds $ (fluctuant)
- **Ce qui a marche** : utilite reelle (des milliers de projets DeFi dependent de Chainlink), staking comme garantie de qualite, adoption institutionnelle
- **Debut 2025** : lancement de "Chainlink Reserve" — les revenus des entreprises servent a acheter du LINK (buyback)

**Basic Attention Token (BAT) :**
- **Usage** : recompenser les utilisateurs du navigateur Brave pour regarder des pubs
- **Supply** : 1,5 milliard total (tout en circulation)
- **Ce qui a marche** : model simple a comprendre (tu regardes des pubs, tu gagnes des BAT), integration directe dans un produit reel (Brave, 60+ millions d'utilisateurs mensuels)
- **Ce qui a limite** : le prix du BAT n'a pas explose car la demande publicitaire n'a pas suivi autant qu'espere

**Filecoin (FIL) :**
- **Usage** : payer pour du stockage de fichiers decentralise
- **Supply** : 2 milliards max, emission progressive
- **Ce qui a marche** : alternative credible a AWS/Dropbox, les mineurs stakent du FIL comme garantie
- **Probleme rencontre** : supply debloquee trop vite initialement → pression de vente → prix passe de 237 $ (avril 2021) a 3 $ (fin 2022)

### 4.5 Les erreurs a eviter

| Erreur | Consequence | Comment eviter |
|--------|-------------|----------------|
| **Trop de tokens fondateur (>25%)** | Les investisseurs se mefient, risque de dump | Max 18-20%, avec vesting 12+36 mois |
| **Pas de vesting** | Dump massif des que les tokens sont debloques | Vesting lineaire obligatoire pour tous les insiders |
| **Liquidite trop faible** | Manipulation facile, slippage enorme | Min 20-50K $ dans le pool initial |
| **Token sans utilite reelle** | Personne ne l'achete, prix → 0 | Le token DOIT etre necessaire pour le service |
| **Pas d'audit du smart contract** | Hack, vol, perte de confiance | Audit par une firme reconnue (Certik, OpenZeppelin, Hacken) |
| **Supply trop grande sans mecanisme deflationniste** | Dilution → prix ne monte jamais | Burn, buyback, ou supply fixe |
| **Communication opaque** | Perte de confiance communaute | Transparence on-chain, rapports reguliers |
| **Deblocage simultane de plusieurs categories** | Chute brutale du prix | Echelonner les unlocks |

### 4.6 Les obligations legales en Europe (MiCA 2024-2025)

**MiCA (Markets in Crypto-Assets)** est le cadre reglementaire europeen pour les crypto-actifs, pleinement en vigueur depuis le 30 decembre 2024.

**Ce que MiCA dit sur les tokens utilitaires :**

1. **Definition** : un token utilitaire est un crypto-actif "destine uniquement a fournir l'acces a un bien ou service fourni par son emetteur" (Article 3(9))
2. **Statut** : les tokens utilitaires sont classes comme "non-financiers" — donc soumis a des regles plus legeres que les security tokens

**Exemption importante pour les tokens utilitaires :**
- Si le token donne acces a **un service existant et fonctionnel** (pas un projet futur), il est **exempt des exigences de whitepaper MiCA** et des obligations d'offre publique
- Condition : le service doit etre "actuellement disponible sur le marche ou en cours d'utilisation"
- Attention : cette exemption ne s'applique PAS si le token est utilise pour lever des fonds

**Si pas exempte (service pas encore existant) :**
- Publication d'un whitepaper (crypto-asset white paper) au format iXBRL (depuis decembre 2025)
- Notification a l'autorite competente (en France : l'AMF)
- Le whitepaper doit contenir : description du projet, droits et obligations, risques, tokenomics, impact environnemental

**Autres exemptions MiCA :**
- Levee de moins de 1 million EUR → pas besoin de whitepaper
- Offre a moins de 150 personnes par Etat membre → pas besoin de whitepaper
- Offre uniquement a des investisseurs qualifies → pas besoin de whitepaper

**En pratique pour CBOR-Web :** si le service CBOR-Web est **deja fonctionnel** quand le token est lance, il beneficie de l'exemption MiCA. Sinon, il faut un whitepaper conforme.

---

## 5. LE MODELE SPECIFIQUE CBOR-WEB — ANALYSE

### 5.1 Rappel du modele propose

| Parametre | Valeur |
|-----------|--------|
| Standard | ERC-20 sur Ethereum mainnet |
| Supply totale | 100 000 000 (100M) tokens |
| Mecanisme d'acces | Detention = acces (pas de depense par requete) |
| Modele | "Token gating" — tu detiens X tokens = tu as acces au contenu CBOR-Web |
| Dynamique de prix | Plus d'adoption → plus de demande → prix monte |
| Allocation fondateur | 20% (20M tokens) |

### 5.2 Analyse de viabilite

**Points forts du modele :**

1. **Le "token gating" est un modele eprouve** :
   - Des projets comme Doodles, Adidas ALTS, Lyrical Lemonade utilisent deja le "hold-to-access"
   - Le concept est simple a expliquer : "tu detiens le token = tu as acces"
   - Pas de friction d'usage (pas besoin de depenser a chaque requete)

2. **Le modele par detention reduit la pression de vente** :
   - Les utilisateurs DOIVENT garder leurs tokens pour maintenir l'acces
   - Contrairement a un modele "pay-per-use" ou les tokens sont constamment vendus
   - Cela cree un "plancher" naturel de demande

3. **100M supply est un chiffre raisonnable** :
   - Comparable a des projets reussis
   - Assez grand pour eviter des prix unitaires trop eleves au debut
   - Assez petit pour creer de la rarete si l'adoption decolle

**Points de vigilance :**

1. **Le paradoxe du "hold-to-access"** :
   - Si les utilisateurs achetent des tokens et les gardent sans jamais les vendre → pas de volume d'echange
   - Pas de volume = pas de revenus de trading = difficulte a attirer des market makers
   - Solution : definir un seuil minimum de detention (ex: 100 tokens = acces basique, 1000 = acces premium)

2. **Pas de mecanisme de burn/deflation prevu** :
   - Avec 100M tokens fixes et un modele de detention, la supply ne diminue jamais
   - Le prix ne monte que par la demande accrue
   - Recommandation : ajouter un mecanisme de buyback & burn finance par les revenus du service

3. **20% fondateur — a la limite haute** :
   - Le benchmark 2025 est 18-20% pour l'equipe
   - 20% est acceptable UNIQUEMENT avec un vesting strict (12 mois cliff + 36 mois lineaire)
   - Sans vesting, les investisseurs fuiront (signal de "money grab")

### 5.3 Risques identifies

| Risque | Probabilite | Impact | Mitigation |
|--------|-------------|--------|-----------|
| **Pas assez d'utilisateurs** | Elevee | Critique | Le service doit avoir une valeur reelle AVANT le lancement du token |
| **Classification en security token** | Moyenne | Critique | Lancer le token APRES que le service existe (exemption MiCA). Ne JAMAIS promettre de profits |
| **Liquidite insuffisante** | Elevee | Eleve | LBP au lancement + programme LP incentive |
| **Dump du fondateur** | Faible (si vesting) | Critique | Vesting 12+36, adresses publiques, transparence |
| **Hack du smart contract** | Faible (si audite) | Critique | Audit par firme reconnue (5-20K $) |
| **Concurrence** | Moyenne | Moyen | Differenciation technique (CBOR est une niche) |
| **Reglementation future** | Moyenne | Moyen | Suivre l'evolution de MiCA, consulter un avocat crypto |

### 5.4 Proposition de structure de lancement

**Allocation recommandee pour CBOR-Web (100M tokens) :**

| Categorie | % | Tokens | Vesting |
|-----------|---|--------|---------|
| Ecosysteme / Communaute | 40% | 40M | Liberation progressive sur 48 mois, incentives, airdrops cibles |
| Fondateur + Equipe | 18% | 18M | 12 mois cliff + 36 mois lineaire |
| Tresorerie / Reserve | 20% | 20M | Verrouille, utilise pour buyback, partenariats, urgences |
| Liquidite initiale (DEX) | 10% | 10M | Pairee avec ETH/USDC au lancement, lock 12 mois |
| Vente publique (IDO/LBP) | 8% | 8M | 25% au TGE + 75% veste sur 6 mois |
| Advisors / Partenaires | 4% | 4M | 6 mois cliff + 24 mois lineaire |

**Calendrier suggere :**

| Phase | Duree | Action |
|-------|-------|--------|
| T-6 mois | Mois 1-6 | Developpement smart contract, audit, whitepaper, communaute |
| T-3 mois | Mois 7-9 | Beta du service CBOR-Web (sans token), testnet du token |
| T-1 mois | Mois 10 | Service CBOR-Web lance et fonctionnel (crucial pour exemption MiCA) |
| T=0 | Mois 11 | Lancement du token via LBP (Balancer/Fjord) ou IDO |
| T+1 mois | Mois 12 | Listing Uniswap V3, programme LP incentive |
| T+3 mois | Mois 14 | Premier buyback & burn avec les revenus |
| T+6 mois | Mois 17 | Candidature CEX tier 2 (MEXC, Gate.io) si volume suffisant |

**Budget minimum estime :**

| Poste | Cout |
|-------|------|
| Smart contract (OpenZeppelin + custom) | 2 000 - 5 000 $ |
| Audit de securite | 5 000 - 20 000 $ |
| Deploiement Ethereum mainnet | 100 - 500 $ |
| Liquidite initiale (ETH dans le pool) | 10 000 - 50 000 $ |
| Marketing/communaute | 5 000 - 20 000 $ |
| Conseil juridique (MiCA) | 5 000 - 15 000 $ |
| **Total minimum** | **~27 000 - 110 000 $** |

### 5.5 Comparaison avec des modeles similaires

| Projet | Modele | Supply | Resultat |
|--------|--------|--------|----------|
| **BAT (Brave)** | Utility / attention | 1,5 Md | Succes modere. 60M+ users, mais prix stagne (~0,20 $). Le service marche, la tokenomics manque de mecanisme deflationniste |
| **LINK (Chainlink)** | Utility / oracle | 1 Md | Grand succes. Service indispensable a l'ecosysteme DeFi. Prix 9-25 $ selon les periodes |
| **FIL (Filecoin)** | Utility / stockage | 2 Md | Techniquement solide, mais deblocage trop rapide → chute de 237 $ a 3 $. Le vesting est crucial |
| **ENS (Ethereum Name Service)** | Utility / nommage | 100 M | Bonne reference pour CBOR-Web. Supply similaire (100M). Token de gouvernance + acces. Prix 8-25 $ |
| **CBOR-Web (hypothetique)** | Utility / acces contenu | 100 M | A comparer avec ENS (meme supply, modele acces/detention). Viable SI le service a une vraie adoption |

### 5.6 Verdict

**Est-ce viable ?** Oui, sous conditions :

1. **Le service CBOR-Web doit exister et etre utile AVANT le token.** Un token sans produit est un meme coin.
2. **Le vesting du fondateur est non-negociable.** 12 mois cliff + 36 mois lineaire minimum.
3. **La liquidite initiale doit etre suffisante.** Minimum 20-50K $ pour eviter la manipulation.
4. **Un mecanisme deflationniste est fortement recommande.** Buyback & burn avec les revenus du service.
5. **Ne jamais promettre de profits** — ca transforme un utility token en security token (legal et fiscal).
6. **Lancer le service AVANT le token** — exemption MiCA + credibilite + adoption organique.
7. **Definir des niveaux d'acces clairs** — ex : 100 CBOR = acces lecture, 1000 CBOR = acces premium, 10 000 CBOR = acces API complet.

**Le modele "hold-to-access" est l'un des plus sains** car il aligne les interets : les utilisateurs gardent les tokens (pas de pression de vente), le fondateur est incentive a ameliorer le service (plus d'utilisateurs = plus de demande = prix monte), et la regulation est favorable (utility token avec service existant = exemption MiCA).

**Le risque principal est classique : le produit doit marcher.** Aucune tokenomics au monde ne sauve un service que personne n'utilise.

---

## SOURCES

### Fonctionnement de la Bourse
- [Supply and Demand in Stocks](https://www.heygotrade.com/en/blog/supply-and-demand-in-stocks)
- [What Is an Order Book](https://www.heygotrade.com/en/blog/what-is-an-order-book)
- [Order Book - Corporate Finance Institute](https://corporatefinanceinstitute.com/resources/career-map/sell-side/capital-markets/order-book/)
- [What Is a Market Maker? - Citadel Securities](https://www.citadelsecurities.com/what-we-do/what-is-a-market-maker/)
- [Market Maker - Britannica Money](https://www.britannica.com/money/what-is-a-market-maker)
- [Market Makers - Investor.gov](https://www.investor.gov/introduction-investing/investing-basics/glossary/market-makers)
- [Bull and Bear Markets - Coinbase](https://www.coinbase.com/learn/crypto-basics/what-is-a-bull-or-bear-market)
- [Bull vs Bear Markets - Changelly](https://changelly.com/blog/bears-vs-bulls-in-crypto-market-players/)

### Crypto / Tokens
- [ERC-20 Token Development 2025](https://www.blockchainappfactory.com/blog/erc20-token-costs-and-practices/)
- [List Your Token on an Exchange - Matcha](https://blog.matcha.xyz/article/list-a-token-on-exchanges)
- [How Uniswap Works - Uniswap Docs](https://docs.uniswap.org/contracts/v2/concepts/protocol-overview/how-uniswap-works)
- [What is a Liquidity Pool? - Uniswap](https://support.uniswap.org/hc/en-us/articles/8829880740109-What-is-a-liquidity-pool)
- [Constant Product Formula - Bitget](https://www.bitget.com/wiki/what-is-the-constant-product-formula-of-uniswap-v1--v2)
- [Tokenomics 2025 Guide - InnMind](https://blog.innmind.com/creating-a-successful-tokenomics-key-metrics-to-consider/)
- [Token Vesting Complete Guide - Tokenomics.com](https://tokenomics.com/articles/token-vesting-complete-guide-to-vesting-schedules-cliffs-and-unlock-mechanisms)
- [2025 Tokenomics Playbook - Medium](https://medium.com/@izaguirre.john/the-2025-tokenomics-playbook-vesting-allocations-and-a-new-institutional-era-1d3062697d5b)
- [Security vs Utility Tokens - DeFi Bunker](https://defibunker.com/utility-tokens-vs-security-tokens-key-differences-risks-opportunities)
- [IDO Explained - CryptoPotato](https://cryptopotato.com/what-is-an-initial-dex-offering-ido-how-is-it-different-than-ico-ieo/)

### Stabilisation du prix
- [How USDC Maintains Its Peg - Eco](https://eco.com/support/en/articles/11855034-how-does-usdc-maintain-its-peg-complete-guide-to-stablecoin-stability-mechanisms)
- [Top 3 Stablecoins 2025 - Cryptal](https://cryptal.com/en/blog/top-3-stablecoins-2025-usd-usdc-dai)
- [Buyback and Burn - Tokenomics Learning](https://tokenomics-learning.com/en/buyback-and-burn-2/)
- [Token Buybacks - DWF Labs](https://www.dwf-labs.com/research/547-token-buybacks-in-web3)
- [Bonding Curves - TokenMinds](https://tokenminds.co/blog/knowledge-base/crypto-bonding-curve)
- [What is a Bonding Curve? - Coinbase](https://www.coinbase.com/learn/advanced-trading/what-is-a-bonding-curve)
- [Staking 101 - Grayscale](https://research.grayscale.com/reports/staking-101-secure-the-blockchain-earn-rewards)
- [What is Staking? - Coinbase](https://www.coinbase.com/learn/crypto-basics/what-is-staking)

### Token utilitaires et exemples
- [What Are Utility Tokens - Debut Infotech](https://www.debutinfotech.com/blog/what-are-utility-tokens)
- [Chainlink Tokenomics - Shrimpy Academy](https://academy.shrimpy.io/post/chainlink-tokenomics-explained)
- [Chainlink Economics](https://chain.link/economics)
- [Token Launch Mistakes - Onchain Magazine](https://onchain.org/magazine/avoid-these-common-token-launch-mistakes/)
- [5 Rules for Token Launches - a16z](https://a16zcrypto.com/posts/article/5-rules-for-token-launches/)
- [Liquidity Bootstrapping Pools - Balancer](https://medium.com/balancer-protocol/a-primer-on-fair-token-launches-and-liquidity-bootstrapping-pools-11bab5ff33a2)

### Reglementation MiCA
- [MiCA Regulation - ESMA](https://www.esma.europa.eu/esmas-activities/digital-finance-and-innovation/markets-crypto-assets-regulation-mica)
- [MiCA Explained - Legal Nodes](https://www.legalnodes.com/article/mica-regulation-explained)
- [MiCA Compliance 2025 - Hacken](https://hacken.io/discover/mica-regulation/)
- [MiCA Whitepaper Requirements - Paul Hastings](https://www.paulhastings.com/insights/client-alerts/mica-crypto-white-papers-comply-or-be-de-listed)
- [Utility Tokens Under MiCA - SpringerLink](https://link.springer.com/chapter/10.1007/978-3-031-74889-9_10)

### Token Gating
- [What Is Token Gating? - BitDegree](https://www.bitdegree.org/crypto/tutorials/token-gating)
- [Token Gating Guide - NFT News Today](https://nftnewstoday.com/2025/02/13/what-is-token-gating-your-complete-guide-to-exclusive-web3-access)
- [Token Gating - Ledger Academy](https://www.ledger.com/academy/topics/crypto/what-is-token-gating)

### Deploiement et couts
- [ERC-20 Token Cost 2025 - Blockchain App Factory](https://www.blockchainappfactory.com/blog/how-much-does-it-cost-to-create-erc-20-token/)
- [Ethereum Gas Fees 2025 - SQ Magazine](https://sqmagazine.co.uk/ethereum-gas-fees-statistics/)
- [Top Tokenomics Mistakes - iLink](https://ilink.dev/blog/top-tokenomics-mistakes-how-not-to-lose-liquidity-and-users-in-the-first-weeks)
