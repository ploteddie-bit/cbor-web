# CBOR-Web Specification v0.1

**Machine-Readable Binary Web Content for Autonomous Agents**

> Status: Draft — v0.1
> Date: 2026-03-21
> Authors: ExploDev (Eddie Plot, Claude)
> Format: CBOR (RFC 8949)
> License: CC BY 4.0

---

## 1. Introduction

### 1.1 Problème

Le web a été conçu pour les humains. Les pages HTML contiennent du contenu mêlé à de la présentation (CSS, JavaScript, balises décoratives). Quand un agent IA autonome navigue le web, il doit :

1. Télécharger du HTML lourd (DOM, CSS inline, scripts)
2. Parser le markup pour en extraire le contenu
3. Tokenizer du texte pollué par des artefacts visuels
4. Deviner la structure de navigation

Ce processus est **lent, coûteux en tokens, et peu fiable**. Un agent consomme 10 à 50x plus de tokens qu'il n'en a besoin pour accéder à l'information utile.

### 1.2 Solution

CBOR-Web définit un format binaire standardisé permettant à un site web d'exposer une **copie machine-native** de son contenu. Cette copie :

- Est en **CBOR** (Concise Binary Object Representation, RFC 8949) — binaire, compact, auto-descriptif
- Contient **uniquement le contenu structuré** — pas de CSS, pas de JS, pas de markup décoratif
- Expose une **navigation explicite** — l'agent sait où aller sans parser de `<nav>` ou de `<a href>`
- Est **transparente pour l'utilisateur humain** — le site HTML reste identique, CBOR-Web est un canal parallèle

### 1.3 Positionnement

| Standard | Format | Contenu | Cible |
|----------|--------|---------|-------|
| robots.txt | Texte | Règles d'accès | Crawlers |
| sitemap.xml | XML | Liste d'URLs | Crawlers |
| llms.txt | Markdown | Résumé + liens | Agents IA (texte) |
| **CBOR-Web** | **Binaire** | **Contenu structuré complet** | **Agents IA (natif)** |

CBOR-Web ne remplace pas les standards existants. Il les complète en offrant le **contenu réel** en binaire, là où llms.txt offre un résumé texte et sitemap.xml une liste d'URLs.

---

## 2. Terminologie

| Terme | Définition |
|-------|-----------|
| **Manifest** | Document CBOR décrivant un site : métadonnées, index des pages, structure de navigation |
| **Page** | Document CBOR contenant le contenu structuré d'une page web |
| **Bundle** | Document CBOR contenant le manifest + toutes les pages (optionnel, pour petits sites) |
| **Agent** | Logiciel autonome (IA ou non) qui consomme du contenu CBOR-Web |
| **Publisher** | Outil ou service qui génère les documents CBOR-Web à partir d'un site HTML |

---

## 3. Discovery — Comment un agent trouve le canal CBOR-Web

Un agent DOIT pouvoir découvrir la présence de CBOR-Web via au moins l'un des mécanismes suivants (par ordre de priorité) :

### 3.1 Well-Known URL (RECOMMANDÉ)

```
GET /.well-known/cbor-web → Manifest CBOR
```

Le serveur DOIT répondre avec `Content-Type: application/cbor` et le manifest du site.

### 3.2 HTTP Link Header

Toute page HTML PEUT inclure un header :

```
Link: </.well-known/cbor-web>; rel="alternate"; type="application/cbor"
```

### 3.3 HTML Meta Tag

```html
<link rel="alternate" type="application/cbor" href="/.well-known/cbor-web">
```

### 3.4 Entrée robots.txt

```
# CBOR-Web endpoint
CBOR-Web: /.well-known/cbor-web
```

### 3.5 Entrée llms.txt

```
## Machine-Readable Content
- CBOR-Web Manifest: /.well-known/cbor-web
```

---

## 4. Manifest — Index du site

Le manifest est le point d'entrée. Un agent le lit en premier et décide quelles pages récupérer.

### 4.1 Structure

```
CBOR Map {
  0: "cbor-web-manifest"          // @type (texte)
  1: 1                            // @version (entier)
  2: {                            // site (map)
    "domain": "deltopide.fr"
    "name": "Deltopide - Agence Digitale"
    "description": "Agence web franco-espagnole..."
    "languages": ["fr", "es", "en"]
    "default_language": "fr"
    "contact": {
      "email": "contact@deltopide.fr"
      "phone": "+34 ..."
    }
    "geo": {
      "country": "ES"
      "region": "Comunidad Valenciana"
      "coordinates": [38.9942, -0.1660]
    }
  }
  3: [                            // pages (array)
    {
      "path": "/"
      "title": "Accueil"
      "lang": "fr"
      "updated": 1(1742515200)    // CBOR tag 1 = epoch timestamp
      "hash": h'a3f2...'          // SHA-256 du contenu page
      "size": 1847                // taille en bytes du document page
      "alternates": {
        "es": "/es/"
        "en": "/en/"
      }
    }
    {
      "path": "/services/developpement-web"
      "title": "Développement Web"
      "lang": "fr"
      "updated": 1(1742428800)
      "hash": h'b7c1...'
      "size": 3204
      "alternates": {
        "es": "/es/servicios/desarrollo-web"
        "en": "/en/services/web-development"
      }
    }
    // ... toutes les pages
  ]
  4: {                            // navigation (map)
    "main": ["/", "/services", "/portfolio", "/blog", "/contact"]
    "footer": ["/mentions-legales", "/politique-confidentialite"]
    "hierarchy": {
      "/services": [
        "/services/developpement-web"
        "/services/seo"
        "/services/design"
      ]
    }
  }
  5: {                            // meta (map)
    "generator": "cbor-web-publisher/1.0"
    "generated_at": 1(1742515200)
    "total_pages": 80
    "total_size": 127400          // taille totale en bytes
    "bundle_available": true      // un bundle tout-en-un est disponible
    "bundle_url": "/.well-known/cbor-web/bundle"
  }
}
```

### 4.2 Clés entières compactes

Pour minimiser la taille binaire, les clés de premier niveau du manifest utilisent des entiers :

| Clé | Signification |
|-----|---------------|
| 0 | @type |
| 1 | @version |
| 2 | site |
| 3 | pages |
| 4 | navigation |
| 5 | meta |

Les clés de second niveau restent en texte pour la lisibilité de debug.

---

## 5. Page — Contenu structuré d'une page

### 5.1 Accès

```
GET /.well-known/cbor-web/pages/{path-encodé}.cbor
```

Exemples :
- `/` → `/.well-known/cbor-web/pages/_index.cbor`
- `/services/seo` → `/.well-known/cbor-web/pages/services_seo.cbor`

### 5.2 Structure

```
CBOR Map {
  0: "cbor-web-page"              // @type
  1: 1                            // @version
  2: {                            // identity (map)
    "path": "/services/developpement-web"
    "canonical": "https://deltopide.fr/services/developpement-web"
    "lang": "fr"
    "alternates": {
      "es": "/es/servicios/desarrollo-web"
      "en": "/en/services/web-development"
    }
  }
  3: {                            // metadata (map)
    "title": "Développement Web Sur Mesure"
    "description": "Création de sites web et applications..."
    "author": "Deltopide SL"
    "published": 1(1740000000)
    "updated": 1(1742428800)
    "tags": ["web", "développement", "react", "node.js"]
    "category": "services"
    "reading_time_seconds": 180
  }
  4: [                            // content (array de blocs)
    {
      "t": "h"                    // heading
      "l": 1                      // level
      "v": "Développement Web Sur Mesure"
    }
    {
      "t": "p"                    // paragraph
      "v": "Nous concevons des sites web performants, accessibles et optimisés pour les moteurs de recherche. Notre approche combine design moderne et technologies éprouvées."
    }
    {
      "t": "h"
      "l": 2
      "v": "Nos Technologies"
    }
    {
      "t": "ul"                   // unordered list
      "v": [
        "React / Next.js"
        "Node.js / Express"
        "PostgreSQL / Redis"
        "Caddy / Nginx"
      ]
    }
    {
      "t": "q"                    // quote / testimonial
      "v": "Deltopide a transformé notre présence en ligne."
      "attr": "Client, Entreprise XYZ"
    }
    {
      "t": "table"                // tableau de données
      "headers": ["Forfait", "Prix", "Inclus"]
      "rows": [
        ["Starter", "990€", "5 pages, responsive, SEO de base"]
        ["Pro", "2490€", "15 pages, multilingue, analytics"]
        ["Premium", "4990€", "Sur mesure, API, maintenance 1 an"]
      ]
    }
    {
      "t": "cta"                  // call to action
      "v": "Demander un devis gratuit"
      "href": "/contact"
    }
  ]
  5: {                            // links (map)
    "internal": [
      {"path": "/contact", "text": "Contact"}
      {"path": "/portfolio", "text": "Nos réalisations"}
    ]
    "external": [
      {"url": "https://reactjs.org", "text": "React"}
    ]
  }
  6: {                            // structured_data (map) — Schema.org compatible
    "type": "Service"
    "provider": {
      "type": "Organization"
      "name": "Deltopide SL"
      "url": "https://deltopide.fr"
    }
    "area_served": ["France", "Espagne", "Europe"]
    "price_range": "€€"
  }
}
```

### 5.3 Types de blocs de contenu

| Code | Type | Champs |
|------|------|--------|
| `h` | Heading | `l` (level 1-6), `v` (texte) |
| `p` | Paragraph | `v` (texte) |
| `ul` | Unordered list | `v` (array de textes) |
| `ol` | Ordered list | `v` (array de textes) |
| `q` | Citation / quote | `v` (texte), `attr` (attribution, optionnel) |
| `code` | Bloc de code | `v` (code), `lang` (langage, optionnel) |
| `table` | Tableau | `headers` (array), `rows` (array d'arrays) |
| `img` | Image (référence) | `alt` (description), `src` (URL), `caption` (optionnel) |
| `cta` | Call to action | `v` (texte), `href` (destination) |
| `embed` | Contenu embarqué | `type` (video/map/form), `src` (URL), `description` |
| `sep` | Séparateur | aucun |
| `dl` | Definition list | `v` (array de `{"term": ..., "def": ...}`) |
| `note` | Note/avertissement | `v` (texte), `level` (info/warn/important) |

### 5.4 Clés compactes des pages

| Clé | Signification |
|-----|---------------|
| 0 | @type |
| 1 | @version |
| 2 | identity |
| 3 | metadata |
| 4 | content |
| 5 | links |
| 6 | structured_data |

---

## 6. Bundle — Tout le site en un fichier

Pour les sites de moins de ~500 pages, un bundle optionnel permet à l'agent de tout récupérer en une seule requête.

### 6.1 Accès

```
GET /.well-known/cbor-web/bundle → Bundle CBOR complet
```

### 6.2 Structure

```
CBOR Map {
  0: "cbor-web-bundle"
  1: 1
  2: { ... }                     // manifest complet (même structure que §4)
  3: {                            // pages (map path → contenu page)
    "/": { ... }                  // contenu page accueil
    "/services/seo": { ... }      // contenu page SEO
    // ... toutes les pages
  }
}
```

---

## 7. Compression et transport

### 7.1 Encodage CBOR

- Encodage déterministe (RFC 8949 §4.2) pour garantir des hash reproductibles
- Clés de premier niveau en entiers (§4.2, §5.4)
- Timestamps en CBOR tag 1 (epoch)
- Hash en CBOR byte strings

### 7.2 Compression HTTP

Le serveur DEVRAIT supporter :

```
Accept-Encoding: br, gzip
Content-Encoding: br
```

Brotli (br) est RECOMMANDÉ — taux de compression supérieur à gzip sur du CBOR.

### 7.3 Cache et conditional requests

```
ETag: "a3f2c8..."
Last-Modified: Fri, 21 Mar 2026 10:00:00 GMT
```

L'agent PEUT utiliser `If-None-Match` / `If-Modified-Since` pour éviter de retélécharger un contenu inchangé. Le champ `hash` dans le manifest permet une vérification côté client sans requête.

### 7.4 Content-Type

```
Content-Type: application/cbor
```

MIME type enregistré par RFC 8949. Pas de sous-type custom nécessaire.

---

## 8. Intégrité et sécurité

### 8.1 HTTPS obligatoire

CBOR-Web DOIT être servi uniquement via HTTPS. Un agent DOIT refuser un manifest servi en HTTP.

### 8.2 Hash de contenu

Chaque page dans le manifest inclut un hash SHA-256 de son contenu CBOR. L'agent PEUT vérifier l'intégrité après téléchargement.

### 8.3 Rate limiting

Le publisher DEVRAIT implémenter du rate limiting. Recommandation : 10 requêtes/seconde par agent. Le manifest inclut un champ optionnel :

```
"rate_limit": {
  "requests_per_second": 10
  "bundle_cooldown_seconds": 3600
}
```

### 8.4 Respect du robots.txt

Un agent CBOR-Web DOIT respecter robots.txt. Si `Disallow: /secret/` est présent, les pages sous `/secret/` NE DOIVENT PAS être incluses dans le manifest.

---

## 9. Versioning

- Champ `@version` (clé 1) dans chaque document
- Version actuelle : `1`
- Un agent DOIT ignorer les clés qu'il ne reconnaît pas (forward compatibility)
- Changement non rétrocompatible = incrément de version majeure
- Ajout de clés optionnelles = pas de changement de version

---

## 10. Gains attendus

### 10.1 Taille

Exemple : page "Services" de deltopide.fr

| Format | Taille | Tokens estimés |
|--------|--------|---------------|
| HTML complet | 47 KB | ~12 000 |
| HTML minifié | 31 KB | ~8 000 |
| llms.txt (texte) | 2.1 KB | ~500 |
| **CBOR-Web** | **1.4 KB** | **0** (binaire, pas de tokenization) |

### 10.2 Requêtes

| Approche | Requêtes pour indexer un site 80 pages |
|----------|---------------------------------------|
| Crawl HTML | 80+ (1 par page + assets) |
| Sitemap + crawl | 81 (sitemap + 80 pages) |
| llms.txt | 1 (mais contenu résumé) |
| **CBOR-Web bundle** | **1** (contenu complet structuré) |

### 10.3 Coût

Pour un agent utilisant Claude (tokens d'entrée) :

| Approche | Tokens / site 80 pages | Coût estimé |
|----------|----------------------|-------------|
| Crawl HTML | ~960 000 | ~$2.88 |
| llms.txt | ~40 000 | ~$0.12 |
| **CBOR-Web** | **0 tokens** (parsing binaire) | **$0.00** |

*L'agent parse le CBOR en mémoire et n'envoie au LLM que les données pertinentes à sa requête.*

---

## 11. Implémentation de référence

### 11.1 Publisher (convertisseur HTML → CBOR-Web)

Langage : Rust
Entrée : URL d'un site web ou répertoire de fichiers HTML
Sortie : Manifest + pages CBOR dans `/.well-known/cbor-web/`

```
cbor-web-publish https://deltopide.fr --output ./well-known/cbor-web/
cbor-web-publish ./site-html/ --output ./well-known/cbor-web/
```

### 11.2 Agent Reader (consommateur CBOR-Web)

Bibliothèque Rust/Python qui :
1. Découvre le manifest (`/.well-known/cbor-web`)
2. Parse le CBOR
3. Expose le contenu structuré via une API simple

```python
from cbor_web import Site

site = Site.from_url("https://deltopide.fr")
print(site.name)           # "Deltopide - Agence Digitale"
print(site.languages)      # ["fr", "es", "en"]

for page in site.pages(lang="fr"):
    print(page.title)
    for block in page.content:
        if block.type == "p":
            print(block.value)
```

### 11.3 Intégration serveur web

Plugin Caddy (prioritaire — ExploDev utilise Caddy) :

```caddy
deltopide.fr {
    cbor_web {
        source /var/www/deltopide.fr
        bundle true
        refresh 1h
    }
}
```

---

## 12. Roadmap

| Phase | Livrable | Statut |
|-------|----------|--------|
| 0.1 | Spécification draft | Ce document |
| 0.2 | Publisher CLI (Rust) | À développer |
| 0.3 | Déploiement sur deltopide.fr | À planifier |
| 0.4 | Agent Reader (Python) | À développer |
| 0.5 | Plugin Caddy | À développer |
| 0.6 | GeoScore v2 — mesure avant/après | À développer |
| 1.0 | Spécification stable + publication | Objectif |

---

## 13. FAQ

**Pourquoi CBOR et pas Protocol Buffers ?**
CBOR est auto-descriptif (pas besoin de fichier .proto), standardisé IETF, et plus flexible pour du contenu web variable. Protobuf est optimal pour des schémas fixes type RPC, pas pour du contenu web hétérogène.

**Pourquoi pas juste du JSON compressé ?**
JSON nécessite du parsing texte, de l'échappement de caractères, et des guillemets autour de chaque clé. CBOR encode les mêmes structures en binaire natif, 30-50% plus compact sans compression, et significativement plus rapide à parser.

**Pourquoi pas étendre llms.txt ?**
llms.txt est du Markdown texte — il sera toujours tokenizé par le LLM. CBOR-Web est parsé en mémoire par l'agent *avant* d'interroger le LLM. L'agent n'envoie au LLM que la fraction pertinente à la question posée.

**Est-ce que ça remplace le HTML ?**
Non. Le HTML reste pour les humains. CBOR-Web est un canal parallèle pour les machines. Un même serveur sert les deux. L'analogie : un restaurant a un menu pour les clients (HTML) et un bon de commande pour la cuisine (CBOR-Web).

**Comment gérer le contenu dynamique ?**
CBOR-Web est pensé pour le contenu relativement stable (pages, articles, produits). Le contenu hautement dynamique (feed temps réel, résultats de recherche) reste en API classique. Le champ `updated` et les hash permettent le refresh incrémental.

---

## Références

- [RFC 8949 — Concise Binary Object Representation (CBOR)](https://www.rfc-editor.org/rfc/rfc8949)
- [RFC 8610 — CDDL: Concise Data Definition Language](https://www.rfc-editor.org/rfc/rfc8610) (pour formaliser le schéma)
- [llms.txt — LLM-readable content](https://llmstxt.org/)
- [robots.txt — Robots Exclusion Protocol](https://www.rfc-editor.org/rfc/rfc9309)

---

*CBOR-Web Specification — ExploDev 2026*
*"Le web a deux clients : les humains et les machines. Il est temps de servir les deux."*
