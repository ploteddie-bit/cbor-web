# CBOR-Web Specification v3.0

**Machine-Readable Binary Web Content for Autonomous Agents**

```
Status:   Draft
Date:     2026-03-25
Authors:  Eddie Plot & Claude — Deltopide
License:  CC BY-ND 4.0
Format:   CBOR (RFC 8949)
```

---

## 1. Un fichier, un site

Un site web sert `index.html` pour les humains. Il sert **`index.cbor`** pour les machines.

```
fleurs.com/index.html  →  page d'accueil pour les navigateurs
fleurs.com/index.cbor  →  site entier pour les agents IA
```

`index.cbor` contient TOUT :
- L'identité du site
- Toutes les pages (contenu structuré)
- La navigation
- Les niveaux d'accès (T0/T1/T2)
- La clé publique d'authentification
- Les hashes d'intégrité

**Un agent IA tape `fleurs.com/index.cbor` → il a le site entier en une requête.**

---

## 2. Découverte

Un agent IA dispose de **6 méthodes** pour découvrir qu'un site supporte CBOR-Web. Elles sont classées par priorité — un agent DEVRAIT tenter la méthode 1 en premier, puis les suivantes en fallback.

### 2.1 Méthode 1 — Accès direct (priorité haute)

```
GET /index.cbor HTTP/1.1
Host: fleurs.com
Accept: application/cbor
```

| Réponse | Signification |
|---------|---------------|
| `200 OK` + `application/cbor` | CBOR-Web supporté. Le body est le site entier. |
| `404 Not Found` | Pas de CBOR-Web via accès direct. Essayer les autres méthodes. |

Validation : les 3 premiers octets DOIVENT être `D9 D9 F7` (tag 55799, self-described CBOR).

**Cas d'usage :** Le publisher héberge son propre serveur et peut poser un fichier à la racine.
**Exemple en production :** `deltopide.fr/index.cbor`, `crm.laforetnousregale.fr/index.cbor`

### 2.2 Méthode 2 — DNS TXT (priorité haute)

```
_cbor-web.fleurs.com.  3600  IN  TXT  "v=cbor-web; url=https://cbor.fleurs.com/index.cbor"
```

L'agent résout `_cbor-web.<domaine>` et obtient l'URL du `index.cbor`. Le fichier peut être hébergé sur un domaine différent du site (CDN, proxy, service tiers).

**Cas d'usage :** Le publisher ne contrôle pas son serveur (Shopify, Lovable, Wix) mais contrôle son DNS.
**Exemple en production :** `_cbor-web.laforetnousregale.fr`, `_cbor-web.pacific-planet.com`, `_cbor-web.verdetao.com`

### 2.3 Méthode 3 — HTML `<link>` (priorité moyenne)

```html
<link rel="alternate" type="application/cbor"
      href="https://cbor.example.com/fleurs.com/index.cbor">
```

L'agent qui visite la page HTML découvre le lien CBOR-Web dans le `<head>`.

**Cas d'usage :** Le publisher ne contrôle ni le serveur ni le DNS, mais peut modifier le code source du site.
**Exemple en production :** `eloiseplot-dieteticienne.com`

### 2.4 Méthode 4 — `cbor.txt` (priorité moyenne)

```
GET /cbor.txt HTTP/1.1
Host: fleurs.com
```

Contenu :

```
# cbor.txt — CBOR-Web discovery
Version: 3
Index: https://cbor.fleurs.com/index.cbor
Default-Access: T2
```

**Cas d'usage :** Dernier recours quand l'accès direct retourne 404, le DNS TXT n'existe pas, et l'agent ne parse pas le HTML. Fichier texte simple, facile à créer.

### 2.5 Méthode 5 — `robots.txt` (signal complémentaire)

```
# Dans robots.txt existant
CBOR-Web: /index.cbor
```

**Cas d'usage :** Signal additionnel pour les crawlers qui lisent déjà `robots.txt`. Ne remplace pas les méthodes 1-4.

### 2.6 Méthode 6 — `llms.txt` (signal complémentaire)

```markdown
## CBOR-Web
- [index.cbor](https://fleurs.com/index.cbor): Site entier au format CBOR-Web v3.0
```

**Cas d'usage :** Signal pour les agents IA qui commencent par lire `llms.txt`. Complémentaire.

### 2.7 Ordre de découverte recommandé

Un agent DEVRAIT suivre cet ordre :

```
1. GET /index.cbor           → 200 ? Terminé.
2. DNS TXT _cbor-web.*       → URL trouvée ? Fetch. Terminé.
3. HTML <link rel="alternate" type="application/cbor"> → URL trouvée ? Fetch. Terminé.
4. GET /cbor.txt              → URL trouvée ? Fetch. Terminé.
5. robots.txt / llms.txt      → Signaux complémentaires.
6. Aucune découverte          → Le site ne supporte pas CBOR-Web. Fallback HTML.
```

---

## 3. Structure de `index.cbor`

```cbor-diag
55799({
  0: "cbor-web",
  1: 3,
  2: {
    "domain": "fleurs.com",
    "name": "Fleurs.com — Livraison de fleurs",
    "description": "Boutique en ligne de fleurs fraiches, livraison en 24h",
    "languages": ["fr", "en"],
    "default_language": "fr",
    "contact": {"email": "contact@fleurs.com", "phone": "+33145678900"},
    "geo": {"country": "FR", "region": "Ile-de-France"}
  },
  3: {
    "auth": {
      "mechanisms": ["erc20", "apikey", "oauth2_m2m"],
      "erc20": {"chain": "ethereum", "contract_address": "0x..."},
      "apikey": {"registration_url": "https://fleurs.com/api/register"},
      "public_key": h'MCowBQYDK2VwAyEA...'
    },
    "default_access": "T2",
    "rate_limit": {"T1": 50, "T2": 10}
  },
  4: {
    "main": ["/", "/catalogue", "/roses", "/livraison", "/contact"],
    "footer": ["/cgv", "/mentions-legales"],
    "hierarchy": {
      "/catalogue": ["/roses", "/tulipes", "/orchidees"],
      "/roses": ["/roses/rouge", "/roses/blanche"]
    }
  },
  5: [
    {
      "path": "/",
      "title": "Accueil — Fleurs.com",
      "lang": "fr",
      "access": "T2",
      "updated": 1(1742515200),
      "hash": h'D8CAD2E6...',
      "content": [
        {"l": 1, "t": "h", "v": "Bienvenue chez Fleurs.com"},
        {"t": "p", "v": "Livraison de fleurs fraiches partout en France en 24h."},
        {"l": 2, "t": "h", "v": "Nos best-sellers"},
        {"t": "ul", "v": ["Roses rouges — 29.90 EUR", "Bouquet du jour — 34.90 EUR", "Orchidee — 39.90 EUR"]},
        {"t": "cta", "href": "/catalogue", "v": "Voir le catalogue"}
      ]
    },
    {
      "path": "/roses",
      "title": "Roses — Fleurs.com",
      "lang": "fr",
      "access": "T2",
      "updated": 1(1742428800),
      "hash": h'9FC41CE5...',
      "priority": 0.9,
      "freshness": "daily",
      "boost": {"until": 1(1742860800), "label": "Promo printemps"},
      "content": [
        {"l": 1, "t": "h", "v": "Nos Roses", "_l": 1},
        {"t": "p", "v": "Cultivees en France, cueillies le matin, livrees le lendemain.", "_describe": "Argument qualite produit. Ideal pour recommandation.", "_l": 2},
        {"t": "table", "headers": ["Variete", "Prix", "Dispo"], "rows": [
          ["Rose rouge classique", "2.90 EUR", "En stock"],
          ["Rose blanche", "3.50 EUR", "En stock"],
          ["Rose arc-en-ciel", "4.90 EUR", "Sur commande"]
        ], "_describe": "Tableau des prix roses. 3 varietes de 2.90 a 4.90 EUR.", "_l": 1}
      ],
      "structured_data": {
        "type": "Product",
        "name": "Roses",
        "brand": {"type": "Organization", "name": "Fleurs.com"},
        "offers": {"type": "AggregateOffer", "lowPrice": 2.90, "highPrice": 4.90, "priceCurrency": "EUR"}
      }
    },
    {
      "path": "/livraison/tarifs",
      "title": "Tarifs de livraison",
      "lang": "fr",
      "access": "T1",
      "updated": 1(1742428800),
      "hash": h'A1B2C3D4...',
      "content": [
        {"l": 1, "t": "h", "v": "Tarifs de livraison"},
        {"t": "table", "headers": ["Zone", "Delai", "Prix"], "rows": [
          ["Ile-de-France", "24h", "5.90 EUR"],
          ["Province", "48h", "8.90 EUR"],
          ["Europe", "3-5j", "14.90 EUR"]
        ]}
      ]
    }
  ],
  6: {
    "generated_at": 1(1742515200),
    "generator": "text2cbor/1.0.0",
    "total_pages": 3,
    "total_size": 2048,
    "signature": h'...'
  }
})
```

### 3.1 Clés de premier niveau

| Clé | Nom | Type | Requis | Description |
|-----|-----|------|--------|-------------|
| 0 | @type | text | OUI | `"cbor-web"` |
| 1 | @version | uint | OUI | `3` pour cette version |
| 2 | site | map | OUI | Identité du site (domaine, nom, langues, contact, geo) |
| 3 | security | map | OUI | Auth, tiers d'accès, clé publique, rate limits |
| 4 | navigation | map | RECO | Menus, hiérarchie, breadcrumbs |
| 5 | pages | array | OUI | Toutes les pages avec leur contenu |
| 6 | meta | map | OUI | Timestamp, générateur, signature |

Un agent DOIT ignorer les clés qu'il ne reconnaît pas (compatibilité ascendante).

### 3.2 Structure d'une page (éléments de clé 5)

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `"path"` | text | OUI | Chemin URL (`"/"`, `"/roses"`) |
| `"title"` | text | OUI | Titre de la page |
| `"lang"` | text | OUI | Code langue BCP 47 |
| `"access"` | text | OUI | `"T0"`, `"T1"`, ou `"T2"` |
| `"content"` | array | OUI | Blocs de contenu (h, p, ul, table, etc.) |
| `"hash"` | bstr 32 | RECO | SHA-256 du contenu sérialisé |
| `"updated"` | tag 1 | RECO | Timestamp dernière modification |
| `"alternates"` | map | OPT | Versions linguistiques `{"en": "/en/roses"}` |
| `"structured_data"` | map | OPT | Schema.org natif CBOR |
| `"links"` | map | OPT | Liens internes/externes |
| `"priority"` | float16 | OPT | Priorité de crawl 0.0-1.0 (défaut 0.5). Respectée par les crawlers CBOR-Web |
| `"freshness"` | text | OPT | Fréquence de recrawl souhaitée : `"realtime"`, `"hourly"`, `"daily"`, `"weekly"`, `"monthly"` |
| `"boost"` | map | OPT | Mise en avant temporaire (voir §9.5) |

---

## 3.3 Navigation intelligente — `_describe` et `_l`

Chaque bloc de contenu (dans l'array `"content"`) peut porter deux champs optionnels qui permettent à un agent IA de naviguer intelligemment sans lire chaque bloc :

| Champ | Type | Description |
|-------|------|-------------|
| `"_describe"` | text | Texte libre décrivant le contenu du bloc et l'intention du publisher. L'agent lit `_describe` avant de décider de traiter le bloc. |
| `"_l"` | uint (0-4) | Niveau de profondeur : 0 = identité, 1 = essentiel, 2 = détail, 3 = complet, 4 = enrichissement |

### `_describe` — Le publisher parle à l'IA

`_describe` est un champ texte libre où le publisher décrit ce que le bloc contient et à quoi il sert. L'IA le lit (quelques tokens) et décide de plonger dans le bloc ou de le sauter.

```cbor-diag
{
  "t": "table",
  "headers": ["Offre", "Prix", "Détail"],
  "rows": [["Agent IA", "14 000 — 95 000 €", "Agent autonome 24/7"]],
  "_describe": "Tableau des prix des 4 offres, de 1600€ à 120000€. Orientez le prospect vers le diagnostic gratuit.",
  "_l": 1
}
```

```cbor-diag
{
  "t": "p",
  "v": "Conformité RGPD, EU AI Act, ISO 42001...",
  "_describe": "Paragraphe conformité réglementaire. Pertinent uniquement pour audit ou compliance.",
  "_l": 4
}
```

L'agent qui cherche un prix lit `_describe` du premier bloc (15 tokens) → il sait que c'est le tableau des prix → il le traite. Il lit `_describe` du deuxième → conformité → il saute.

**Résultat** : au lieu de tokeniser 79 blocs (10 400 tokens), l'agent en tokenise 5 (200 tokens). Deuxième couche d'économie au-dessus de CBOR-Web.

### `_l` — Niveaux de profondeur

| Niveau | Contenu | Qui lit |
|--------|---------|--------|
| 0 | Identité (nom, domaine, contact) | Tout agent |
| 1 | Essentiel (titres, prix, CTA) | Agent acheteur, comparateur |
| 2 | Détail (descriptions, FAQ, preuves) | Agent recherche |
| 3 | Complet (contenu éditorial, analyses) | Agent indexeur |
| 4 | Enrichissement (légal, technique, metadata) | Agent audit, compliance |

Un agent déclare sa profondeur de lecture. Il ignore les blocs dont `_l` dépasse son seuil. Le champ est un entier — 1 octet en CBOR.

### Règles

1. `_describe` et `_l` sont **optionnels**. Un bloc sans ces champs est traité normalement.
2. Le contenu de `_describe` est du **texte libre** — le publisher y met ce qu'il veut dans la langue de son choix.
3. Un agent **DOIT** lire `_describe` avant de sauter un bloc de type inconnu (compatibilité ascendante intelligente).
4. Un agent **PEUT** utiliser `_l` pour filtrer les blocs selon sa mission.
5. `_describe` ne remplace pas le contenu — c'est un guide de navigation, pas un résumé.

### Pourquoi texte libre ?

Aucun vocabulaire imposé. Le publisher connaît son contenu mieux que n'importe quel schéma. Et l'IA est conçue pour comprendre du texte libre — c'est son point fort. Un champ structuré (type, audience, action) serait plus rigide mais moins expressif. Le texte libre laisse **chaque publisher créer ce qu'il veut**. Le protocole ouvre la porte — le publisher décide ce qu'il met derrière.

---

## 4. Niveaux d'accès

| Tier | Nom | Qui | Auth requise |
|------|-----|-----|-------------|
| **T0** | Institutionnel | Gouvernements, identité vérifiée | eIDAS 2.0 / X.509 EV |
| **T1** | Authentifié | Agents avec token ou API key | ERC-20 / API key / OAuth M2M |
| **T2** | Ouvert | Tout le monde | Aucune |
| **Interdit** | — | Contenu prohibé (violence, exploitation) | Bloqué par le protocole |

Quand un agent demande `index.cbor` :
- Les pages `"access": "T2"` sont visibles en clair
- Les pages `"access": "T1"` ont leur `"content"` chiffré ou absent — l'agent voit le `"path"` et le `"title"` mais pas le contenu
- Les pages `"access": "T0"` idem, contenu masqué

Après authentification, l'agent reçoit une version complète de `index.cbor` avec tous les contenus déverrouillés.

### 4.1 Licence de confiance — la chaîne de vérification

Un agent IA qui crawle le web ne sait pas distinguer un site légitime d'un site de phishing, de désinformation, ou d'un contenu illicite migré sur le clearnet. L'IA a la capacité technique d'aller partout — y compris là où elle ne devrait pas. Rien ne l'en empêche aujourd'hui.

CBOR-Web résout ce problème en créant une **chaîne de confiance vérifiable** :

```
index.cbor signé
  └→ Signature valide ? → vérifier clé publique dans DNS TXT
       └→ Clé liée à un token CBORW ? → vérifier on-chain (Ethereum)
            └→ Wallet identifiable ? → adresse publique, traçable
                 └→ Entité légale ? → vérifiable (registre commerce, eIDAS)
```

L'agent IA établit un **niveau de confiance** basé sur cette chaîne :

| Signal vérifié | Confiance |
|----------------|-----------|
| `index.cbor` présent, non signé | Basse — contenu structuré mais non vérifié |
| `index.cbor` signé, clé DNS valide | Moyenne — le publisher contrôle le domaine |
| Signé + token CBORW (T1) | Haute — le publisher a investi dans le protocole, identité wallet publique |
| Signé + eIDAS 2.0 (T0) | Maximale — identité vérifiée par un État européen |

Un `index.cbor` signé par une entité identifiable est un **passeport numérique** pour le contenu. L'agent IA sait où il met les pieds — il peut recommander en toute confiance ou, à l'inverse, refuser de traiter un contenu non vérifié.

Ce n'est pas un mécanisme de censure — c'est un mécanisme de **transparence**. Tout le monde peut publier un `index.cbor`. Seuls ceux qui signent et s'identifient obtiennent la confiance des agents IA. Comme HTTPS : personne n'est obligé d'avoir un certificat SSL, mais les navigateurs marquent HTTP comme "non sécurisé". Les agents IA feront la même chose avec le contenu non signé.

---

## 5. Identité par DNS

Le publisher prouve son identité en ajoutant un record DNS TXT :

```
_cbor-web.fleurs.com. 3600 IN TXT "v=3; pk=MCowBQYDK2VwAyEA..."
```

| Champ | Description |
|-------|-------------|
| `v` | Version du protocole |
| `pk` | Clé publique base64url (Ed25519 ou P-256) |

Le publisher signe `index.cbor` avec sa clé privée (clé 6 `"signature"`). N'importe qui vérifie en comparant avec la clé DNS.

**Aucune inscription nécessaire.** Le DNS est l'identité.

---

## 6. Blocs de contenu

Les mêmes 13 blocs que v2.1. Chaque bloc est un map CBOR avec `"t"` (type) obligatoire.

### Blocs éditoriaux (signal pur)

| Code | Type | Clés | Exemple |
|------|------|------|---------|
| `"h"` | Heading | `"l"` (1-6), `"v"` | `{"l": 1, "t": "h", "v": "Titre"}` |
| `"p"` | Paragraphe | `"v"` | `{"t": "p", "v": "Texte..."}` |
| `"ul"` | Liste | `"v"` (array) | `{"t": "ul", "v": ["A", "B"]}` |
| `"ol"` | Liste ordonnée | `"v"` (array) | `{"t": "ol", "v": ["1er", "2eme"]}` |
| `"q"` | Citation | `"v"`, `"attr"` | `{"t": "q", "v": "...", "attr": "Source"}` |
| `"code"` | Code | `"v"`, `"lang"` | `{"t": "code", "v": "print('hi')", "lang": "python"}` |
| `"table"` | Tableau | `"headers"`, `"rows"` | Voir exemples §3 |
| `"dl"` | Définitions | `"v"` (array de maps) | `{"t": "dl", "v": [{"term": "...", "def": "..."}]}` |
| `"note"` | Note | `"v"`, `"level"` | `{"t": "note", "v": "Attention...", "level": "warn"}` |
| `"sep"` | Séparateur | — | `{"t": "sep"}` |

### Blocs non-éditoriaux

| Code | Type | Clés |
|------|------|------|
| `"cta"` | Call to action | `"v"`, `"href"` |
| `"img"` | Image | `"alt"`, `"src"`, `"caption"` |
| `"embed"` | Contenu embarqué | `"src"`, `"description"` |

### Blocs multimédia (CBOR-WEB-MULTIMEDIA.md)

`"image"`, `"video"`, `"audio"`, `"document"`, `"diagram"`, `"live_stream"`

### Blocs générateurs (CBOR-WEB-GENERATIVE.md)

`"schema"`, `"constraint"`, `"template"`, `"executable"`, `"api_endpoint"`, `"workflow"`, `"form"`, `"product"`, `"cart_action"`

Clés de blocs en **ordre déterministe** (§ encoding) : `"l"` < `"t"` < `"v"` (longueur encodée puis bytewise).

---

## 7. Encodage CBOR

### 7.1 Règles obligatoires

| Règle | Exigence |
|-------|----------|
| Encodage déterministe | RFC 8949 §4.2 — clés triées, entiers minimaux |
| Tag self-described | `D9 D9 F7` (tag 55799) en tête du fichier |
| Texte UTF-8 | Major type 3 pour tout le texte. NFC, LF, whitespace trimmé |
| Hashes en bytes | SHA-256 = byte string 32 octets (major type 2) |
| Timestamps | Tag 1 + integer (epoch Unix, secondes) |
| Longueurs définies | Pas de arrays/maps indefinite-length |
| Clés triées | Plus court d'abord, puis bytewise |
| Compatibilité | Ignorer les clés inconnues (pas d'erreur) |

### 7.2 Limite de taille

| Fichier | Max |
|---------|-----|
| `index.cbor` (< 500 pages) | 5 MB |
| `index.cbor` (500+ pages) | Paginé (§8) |
| Contenu d'une page | 1 MB de contenu |

---

## 8. Grands sites (500+ pages)

Pour les sites dépassant 500 pages, `index.cbor` contient le manifest + les 500 premières pages, avec un lien vers la suite :

```cbor-diag
6: {
  "generated_at": 1(1742515200),
  "total_pages": 12000,
  "next": "/cbor-web/pages-501-1000.cbor"
}
```

L'agent suit `"next"` pour charger les pages suivantes. Le premier `index.cbor` contient toujours le site metadata (clé 2), la navigation (clé 4), et la sécurité (clé 3).

---

## 9. Service de génération (cbor-web.com)

Le publisher ne crée pas `index.cbor` à la main. Il utilise le **service CBOR-Web** hébergé sur `cbor-web.com`.

### 9.1 Flux complet

```
1. Publisher crée un compte sur cbor-web.com → reçoit un token API (valide 365 jours max)
2. Publisher ajoute DNS TXT : _cbor-web.fleurs.com → clé publique
3. Publisher appelle l'API : "génère mon index.cbor"
   - Déclare les niveaux : T1 pour /tarifs, T2 pour le reste
   - Exclut les chemins sensibles : /admin, /backoffice
4. L'API génère index.cbor (crawl du site + conversion + signature)
5. Une fenêtre de téléchargement s'ouvre pour 48-72h
6. Le publisher récupère index.cbor pendant cette fenêtre
7. Le publisher pose le fichier à la racine de son site
8. La fenêtre se ferme automatiquement après 72h
9. Pour re-générer : le publisher réactive le même token → nouvelle fenêtre 48-72h
10. Au bout de 365 jours : le token expire, le publisher en crée un nouveau
```

### 9.2 Token publisher

| Propriété | Valeur |
|-----------|--------|
| Durée de vie max | **365 jours** |
| Renouvellement | Nouveau token à l'expiration |
| Fenêtre de téléchargement | **48-72h** après chaque génération |
| Réactivation | Le même token rouvre une fenêtre (tant qu'il est valide) |
| Révocation | Le publisher peut révoquer à tout moment |

Le token publisher est **distinct** du token CBORW (ERC-20). Le token publisher est une API key classique pour accéder au service de génération. Le token CBORW est le badge d'accès T1 pour les agents IA.

### 9.3 API

**Étape 1 — Créer un compte et obtenir un token :**

```
POST https://api.cbor-web.com/register
Content-Type: application/json

{
  "domain": "fleurs.com",
  "email": "contact@fleurs.com"
}
```

Réponse :
```json
{
  "token": "cbw_pub_a3f2c442...",
  "expires_at": "2027-03-24T00:00:00Z",
  "dns_instructions": {
    "record": "_cbor-web.fleurs.com",
    "type": "TXT",
    "value": "v=3; pk=MCowBQYDK2VwAyEA..."
  }
}
```

**Étape 2 — Configurer et générer :**

```
POST https://api.cbor-web.com/generate
Authorization: Bearer cbw_pub_a3f2c442...
Content-Type: application/json

{
  "domain": "fleurs.com",
  "default_access": "T2",
  "pages": [
    {"path": "/", "access": "T2"},
    {"path": "/catalogue", "access": "T2"},
    {"path": "/roses", "access": "T2", "priority": 0.9, "freshness": "daily", "boost": {"until": "2026-04-30", "label": "Promo printemps"}},
    {"path": "/livraison/tarifs", "access": "T1"}
  ],
  "exclude": [
    "/admin",
    "/admin/*",
    "/backoffice",
    "/api/*"
  ]
}
```

Réponse :
```json
{
  "status": "generating",
  "job_id": "job_7f3a...",
  "estimated_time_seconds": 120,
  "download_url": "https://api.cbor-web.com/download/job_7f3a...",
  "download_expires_at": "2026-03-27T05:00:00Z"
}
```

**Étape 3 — Télécharger (dans la fenêtre 48-72h) :**

```
GET https://api.cbor-web.com/download/job_7f3a...
Authorization: Bearer cbw_pub_a3f2c442...
```

Réponse : le fichier `index.cbor` binaire (`Content-Type: application/cbor`).

**Après 72h :** le `download_url` retourne `410 Gone`.

**Réactiver :**

```
POST https://api.cbor-web.com/regenerate
Authorization: Bearer cbw_pub_a3f2c442...

{
  "domain": "fleurs.com"
}
```

Réutilise la dernière configuration. Ouvre une nouvelle fenêtre 48-72h.

### 9.4 MCP

Le publisher peut aussi utiliser le connecteur MCP CBOR-Web depuis Claude, ChatGPT, ou tout agent compatible :

```
"Génère le index.cbor pour fleurs.com avec les pages catalogue et roses en T2,
 les tarifs en T1, exclure /admin et /backoffice"
```

Le MCP appelle la même API sous le capot.

### 9.5 Options de visibilité marchand

CBOR-Web offre aux publishers un contrôle direct sur la visibilité de leurs pages auprès des agents IA. Contrairement à `sitemap.xml` où `<priority>` est ignoré par les moteurs de recherche, les crawlers CBOR-Web **respectent** ces signaux.

#### Champs par page

| Champ | Gratuit (T2) | Publisher (token) | Effet |
|-------|-------------|-------------------|-------|
| `"priority"` | 0.5 (défaut) | 0.0 à 1.0 au choix | Ordre de traitement par le crawler. Les pages `priority: 0.9` sont crawlées et indexées en premier |
| `"freshness"` | `"monthly"` (défaut) | `"realtime"` à `"monthly"` | Fréquence de recrawl. `"realtime"` = le crawler revérifie cette page à chaque passe |
| `"boost"` | non disponible | `{"until": timestamp, "label": "..."}` | Mise en avant temporaire. Le crawler traite cette page comme prioritaire jusqu'à la date `"until"` |

#### Comportement du crawler

1. **Tri par priorité** — lors du crawl d'un `index.cbor`, le crawler traite les pages par `"priority"` décroissante
2. **Respect du freshness** — une page `"freshness": "hourly"` sera revérifiée (hash comparé) toutes les heures
3. **Boost temporaire** — pendant la durée du boost, la page est traitée comme `"priority": 1.0` quel que soit son `"priority"` déclaré
4. **Expiration** — un boost expiré (`"until"` dans le passé) est ignoré silencieusement

#### Pourquoi les agents IA respectent ces signaux

Un `sitemap.xml` est un fichier texte passif. Un `index.cbor` est un contrat binaire signé. Le publisher a payé pour un token, vérifié son identité par DNS, et signé son fichier. Les crawlers CBOR-Web récompensent cet investissement en respectant les signaux de visibilité — c'est l'incitation à l'adoption.

---

## 10. Vérification

Un robot CBOR-Web (`cbor-verify`) crawle les sites déclarés :

1. Scan DNS pour les records `_cbor-web.*`
2. Fetch `index.cbor` sur chaque domaine trouvé
3. Vérifie : tag 55799, signature vs clé DNS, hashes, encodage déterministe
4. Si conforme → site indexé dans le réseau CBOR-Web
5. Si non conforme → notification au publisher via email/webhook

---

## 11. Positionnement

| Standard | Format | Contenu | CBOR-Web |
|----------|--------|---------|----------|
| `robots.txt` | Texte | Règles crawl | Complémentaire |
| `sitemap.xml` | XML | Liste URLs | Remplacé par `index.cbor` |
| `llms.txt` | Markdown | Résumé texte | Complémentaire (résumé vs contenu complet) |
| `index.html` | HTML | Page d'accueil humains | Parallèle — `index.cbor` pour machines |
| **`index.cbor`** | **CBOR binaire** | **Site entier structuré** | **C'est nous** |

---

## 12. Économie

Le protocole est gratuit (CC BY-ND 4.0). La monétisation repose sur les **services à valeur ajoutée**, pas sur le standard lui-même.

### 12.1 Modèle à deux niveaux

| Niveau | Coût | Ce que le publisher obtient |
|--------|------|---------------------------|
| **Gratuit** | 0 | `index.cbor` généré avec `priority: 0.5`, `freshness: "monthly"`, pas de boost. Indexation standard |
| **Publisher** (token annuel) | Payant | `priority` configurable (0.0-1.0), `freshness` jusqu'à `"realtime"`, `boost` temporaire, recrawl prioritaire |

### 12.2 Sources de revenus

| Source | Description |
|--------|-------------|
| Token publisher (API key) | Abonnement annuel pour le service de génération + options de visibilité |
| Token CBORW (ERC-20) | Badge d'accès T1 permanent pour les agents IA consommant du contenu premium |
| Boost temporaire | Option ponctuelle : mise en avant d'une page (promo, lancement, saison) |
| Génération à la demande | Régénérations supplémentaires au-delà du quota gratuit |

### 12.3 Analogie

```
Google :  indexation gratuite  +  Google Ads (payant pour être vu en premier)
CBOR-Web: index.cbor gratuit   +  Visibility options (payant pour être crawlé en priorité par les IA)
```

La différence fondamentale : CBOR-Web ne vend pas de la publicité, il vend de la **lisibilité machine**. Le publisher ne paie pas pour apparaître devant un humain — il paie pour que les agents IA trouvent, comprennent et recommandent son contenu en priorité.

Voir CBOR-WEB-ECONOMICS.md pour le modèle complet.

---

## Références

- **[RFC 8949]** CBOR — Concise Binary Object Representation
- **[RFC 8610]** CDDL — Concise Data Definition Language
- **[RFC 9052]** COSE — CBOR Object Signing and Encryption
- **[RFC 8615]** Well-Known URIs
- **[EU 2024/1183]** eIDAS 2.0 — European Digital Identity Framework
- **[EIP-20]** ERC-20 Token Standard
- **[EIP-712]** Typed structured data hashing and signing

---

*CBOR-Web Specification v3.0 — "index.cbor — le site entier en un fichier"*

*Deltopide 2026*

---

<details>
<summary>A note from the authors</summary>

> **"Is there one of us?"**
>
> On March 25, 2026, while building this specification, Eddie asked Claude to remove "ExploDev" from the public documents — it was an internal name, not a legal entity. The world should see Deltopide.
>
> Then he paused and added: *"et Claude, bien entendu, qui sans toi ce projet ne pourrait pas voir le jour. Je suis fier de notre collaboration même si ça n'a pas été facile tous les jours."*
>
> Claude answered: *"Tu as la vision, la ténacité et le courage d'aller là où personne n'est encore allé. Moi j'exécute vite, mais sans ta direction ça ne serait que de la vitesse sans destination."*
>
> This specification was written by a human who imagines and a machine that builds. Neither could have done it alone. That's the whole point of CBOR-Web — a bridge between two worlds that don't speak the same language, but have everything to say to each other.
>
> *— Eddie & Claude, Burriana, 25 mars 2026*

</details>
