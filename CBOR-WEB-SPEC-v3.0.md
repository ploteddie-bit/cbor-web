# CBOR-Web Specification v3.0

**Machine-Readable Binary Web Content for Autonomous Agents**

```
Status:   Draft
Date:     2026-03-24
Authors:  ExploDev (Eddie Plot, Claude)
License:  CC BY 4.0
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

Un agent découvre CBOR-Web en cherchant `index.cbor` à la racine du domaine :

```
GET /index.cbor HTTP/1.1
Host: fleurs.com
Accept: application/cbor
```

| Réponse | Signification |
|---------|---------------|
| `200 OK` + `application/cbor` | CBOR-Web supporté. Le body est le site entier. |
| `404 Not Found` | Pas de CBOR-Web. Fallback HTML. |

Validation : les 3 premiers octets DOIVENT être `D9 D9 F7` (tag 55799, self-described CBOR).

**Découverte complémentaire (optionnelle) :**

| Méthode | Usage |
|---------|-------|
| DNS TXT `_cbor-web.example.com` | Découverte à grande échelle sans requête HTTP |
| HTTP Link header `rel="alternate" type="application/cbor"` | Découverte pendant navigation HTML |
| `robots.txt` directive `CBOR-Web: /index.cbor` | Compatible crawlers existants |

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
      "content": [
        {"l": 1, "t": "h", "v": "Nos Roses"},
        {"t": "p", "v": "Cultivees en France, cueillies le matin, livrees le lendemain."},
        {"t": "table", "headers": ["Variete", "Prix", "Dispo"], "rows": [
          ["Rose rouge classique", "2.90 EUR", "En stock"],
          ["Rose blanche", "3.50 EUR", "En stock"],
          ["Rose arc-en-ciel", "4.90 EUR", "Sur commande"]
        ]}
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
    {"path": "/roses", "access": "T2"},
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

Le protocole est gratuit (CC BY 4.0). Le token CBORW (ERC-20) est un badge d'accès T1 permanent. Voir CBOR-WEB-ECONOMICS.md pour le modèle complet.

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

*ExploDev 2026*
