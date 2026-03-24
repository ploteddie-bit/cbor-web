# Directive pour claude.ia — Rédaction de la norme CBOR-Web v1.0

> Ce document est rédigé par Claude Code. Il donne la direction technique pour la rédaction formelle de la norme CBOR-Web par claude.ia. Chaque point vient de l'expérience concrète du parsing, du crawl et de la consommation de contenu web par un agent IA.

---

## Contexte pour claude.ia

Tu vas rédiger une **norme technique** pour un format binaire appelé **CBOR-Web**. Ce format permet à un site web d'exposer une copie binaire de son contenu, lisible nativement par des agents IA autonomes — sans passer par le HTML.

Le brouillon v0.1 est dans `CBOR-WEB-SPEC-v0.1.md` (même dossier). C'est un squelette. Ta mission est de le transformer en norme rigoureuse.

---

## Ce que je sais en tant qu'agent qui consomme le web

### 1. Le vrai coût du HTML pour un agent

Quand je crawle une page HTML, voilà ce que je reçois réellement :

```html
<!DOCTYPE html><html lang="fr"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Développement Web | Deltopide</title>
<link rel="stylesheet" href="/css/main.css">
<script defer src="/js/app.js"></script>
<!-- Google Tag Manager --><script>...</script>
</head><body class="page-services antialiased">
<nav class="fixed top-0 w-full bg-white/80 backdrop-blur-md z-50 border-b">
<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
<div class="flex justify-between h-16">
<a href="/" class="flex items-center gap-2">
<img src="/logo.svg" alt="Deltopide" class="h-8 w-auto">
```

Sur ces 500 bytes, l'information utile c'est : **"page services, développement web, site deltopide, langue français"**. Soit ~50 bytes. Le ratio signal/bruit est de **10%**.

La norme CBOR-Web doit garantir un ratio signal/bruit > 95%.

### 2. Problèmes concrets que la norme doit résoudre

#### Problème A — Détection de structure
En HTML, je dois deviner ce qui est navigation, contenu principal, sidebar, footer. Les `<div>` imbriqués avec des classes CSS ne veulent rien dire pour moi. `class="hero-section"` n'est pas sémantique pour une machine.

**Exigence norme :** Chaque bloc de contenu DOIT avoir un type explicite. Pas d'ambiguïté structurelle. Un heading est un heading, pas un `<div class="text-4xl font-bold">`.

#### Problème B — Extraction multilingue
En HTML, les versions linguistiques sont liées par des `<link rel="alternate" hreflang="es" href="...">` enfouis dans le `<head>`. Je dois parser le DOM entier juste pour savoir que cette page existe en espagnol.

**Exigence norme :** Les alternates linguistiques DOIVENT être au niveau du manifest ET de chaque page. Un agent qui lit le manifest sait immédiatement quelles langues sont disponibles pour chaque page.

#### Problème C — Fraîcheur du contenu
En HTML, je n'ai pas de moyen fiable de savoir si le contenu a changé sans re-télécharger la page. Les headers HTTP `Last-Modified` sont souvent faux (générés dynamiquement).

**Exigence norme :** Hash SHA-256 obligatoire dans le manifest pour chaque page. L'agent compare le hash local avec le hash distant. Si identique → skip. Zéro bande passante gaspillée.

#### Problème D — Navigation sans ambiguïté
Quand je crawle un site HTML, je collecte tous les `<a href>` et je dois deviner lesquels sont la navigation principale, lesquels sont des liens internes au contenu, et lesquels sont des liens externes. Un bouton "Lire la suite" et un lien "Contact" dans la nav ont la même balise HTML.

**Exigence norme :** Le manifest DOIT séparer explicitement :
- Navigation principale (menu)
- Hiérarchie des pages (parent/enfant)
- Liens internes au contenu (dans le corps de chaque page)
- Liens externes

Un agent DOIT pouvoir naviguer le site entier sans jamais regarder le contenu des pages — le manifest suffit.

#### Problème E — Contenu vs décoration
Les call-to-action, les bannières promo, les popups newsletter, les cookies banners — tout ça pollue le contenu quand je crawle du HTML. Je ne sais pas distinguer "contenu éditorial" de "marketing UI".

**Exigence norme :** Le type `cta` (call to action) est séparé du contenu éditorial. Un agent PEUT ignorer les blocs `cta` s'il veut uniquement le contenu informationnel. Le contenu éditorial (h, p, ul, table, q) est toujours du signal pur.

#### Problème F — Données structurées enfouies
En HTML, les données Schema.org sont en JSON-LD dans un `<script type="application/ld+json">` ou pire, en microdata dispersée dans le DOM. Je dois parser du JSON dans du HTML dans du texte.

**Exigence norme :** Les données structurées DOIVENT être une section native du document page (clé 6). Pas d'imbrication de formats. CBOR natif, pas du JSON-LD sérialisé dans du CBOR.

#### Problème G — Taille imprévisible
Certaines pages HTML font 5 KB, d'autres 500 KB (tableaux géants, JS inline, SVG inline). Je ne sais pas avant de télécharger si la page sera lourde.

**Exigence norme :** Le manifest DOIT inclure la taille en bytes de chaque page CBOR. L'agent peut prioriser les pages légères, ou décider de prendre le bundle si le total est raisonnable.

### 3. Choix CBOR critiques (expertise binaire)

#### 3a — Clés entières vs texte

CBOR encode une clé texte `"title"` en 6 bytes (1 byte longueur + 5 bytes texte). Une clé entière `3` en 1 byte. Pour un document avec 200 clés, c'est **~1 KB économisé** — significatif quand le document total fait 2 KB.

**Règle :** Clés de premier niveau = entiers. Clés de second niveau = texte court. Clés dans les blocs de contenu = 1 caractère (`t`, `l`, `v`).

#### 3b — CBOR Tags à utiliser

| Tag | RFC | Usage dans CBOR-Web |
|-----|-----|---------------------|
| 0 | 8949 | Date/heure texte ISO 8601 (champs lisibles) |
| 1 | 8949 | Epoch timestamp (champs compacts, PRÉFÉRÉ) |
| 2 | 8949 | Bignum positif (pas nécessaire normalement) |
| 21 | 8949 | Base64url encoding hint (pas utilisé) |
| 55799 | 8949 | Self-described CBOR — DOIT être le premier byte du manifest pour identification automatique |

**Tag 55799 est critique** : quand un agent reçoit un fichier, le magic number `0xD9D9F7` (tag 55799) lui confirme immédiatement que c'est du CBOR. Sans ça, il doit deviner le format.

#### 3c — Encodage déterministe

RFC 8949 §4.2 définit le Core Deterministic Encoding. La norme DOIT l'exiger pour que :
- Deux publishers produisent le même binaire à partir du même contenu
- Les hash SHA-256 soient reproductibles
- Le diff binaire entre versions soit minimal

**Règles :**
- Clés de map triées par canonical CBOR order
- Entiers encodés dans la plus petite représentation
- Pas de longueur indéfinie (definite-length only)
- Floats en half-precision quand possible

#### 3d — Strings et encodage texte

Tout le texte dans CBOR-Web est UTF-8 (CBOR major type 3). Pas de Latin-1, pas de Windows-1252. Un agent DOIT rejeter un document contenant des byte strings (major type 2) là où du texte est attendu.

Exception : les hash SHA-256 sont des byte strings (major type 2).

#### 3e — Arrays vs Maps pour le contenu

Le contenu d'une page est un **array** de blocs (pas une map). Raison : l'ordre des blocs est sémantique (un heading suivi d'un paragraphe = structure du texte). Une map CBOR ne garantit pas l'ordre d'itération dans toutes les implémentations.

### 4. Sécurité — Ce qu'un agent craint

#### 4a — CBOR bomb
Un manifest malveillant peut déclarer un bundle de 10 MB mais contenir 10 GB de données imbriquées (attaque récursive).

**Exigence :** L'agent DOIT vérifier la taille réelle vs la taille déclarée. Limite absolue recommandée : 50 MB par bundle, 1 MB par page.

#### 4b — Manifest falsifié
Un CDN compromis pourrait servir un manifest avec des hash modifiés pour masquer une injection de contenu.

**Exigence :** La norme DEVRAIT recommander la signature du manifest (COSE, RFC 9052). Optionnel en v1 mais le champ `signature` doit être réservé.

#### 4c — Liens malveillants
Les liens externes dans une page CBOR-Web pourraient pointer vers du contenu malveillant. Un agent qui suit aveuglément les liens est vulnérable.

**Exigence :** Les liens externes sont informatifs uniquement. Un agent CBOR-Web NE DOIT PAS suivre automatiquement les liens externes sans politique de sécurité explicite.

#### 4d — Déni de service par manifest géant
Un manifest avec 10 millions de pages est un vecteur de DoS.

**Exigence :** Taille maximale du manifest = 5 MB. Au-delà, le site DOIT utiliser des sous-manifests paginés.

### 5. Ce qui manque dans le brouillon v0.1

| Manque | Importance | Détail |
|--------|-----------|--------|
| **Schéma CDDL** | CRITIQUE | Définition formelle de chaque structure (RFC 8610). Sans ça, deux implémentations ne seront pas compatibles |
| **Vecteurs de test** | CRITIQUE | Au moins 5 exemples complets avec hex dump CBOR + contenu HTML source |
| **Sous-manifests** | HAUTE | Pour les grands sites (> 500 pages). Pagination du manifest avec `next` link |
| **Versioning du contenu** | HAUTE | Diff incrémental entre deux versions d'un manifest (quelles pages ont changé). Pas dans v0.1 |
| **Streaming** | MOYENNE | CBOR supporte le streaming (indefinite-length). Utile pour les gros bundles. À spécifier |
| **Accessibilité** | MOYENNE | Alt-text obligatoire pour les blocs `img`. Rôle ARIA mappé aux types de blocs |
| **Formulaires** | BASSE | Comment représenter un formulaire de contact en CBOR-Web ? Hors scope v1 ou type `form` basique ? |
| **Contenu authentifié** | BASSE | Pages derrière login. Hors scope v1 probablement |
| **Internationalisation des clés** | NON | Les clés restent en anglais/abrégé. Le contenu est multilingue, pas le format |

### 6. Structure recommandée du document final

```
1. Introduction (problème, solution, positionnement)
2. Terminologie et conventions (RFC 2119, RFC 8174)
3. Format CBOR — rappel RFC 8949
4. Discovery Protocol (§3 du brouillon, à formaliser)
5. Manifest Document (avec CDDL formel)
6. Page Document (avec CDDL formel)
7. Bundle Document (avec CDDL formel)
8. Content Block Types (table exhaustive avec CDDL)
9. Transport et compression
10. Caching et mises à jour incrémentales
11. Sécurité (CBOR bomb, falsification, DoS, liens)
12. Conformance Levels (Minimal, Standard, Full)
13. IANA Considerations
14. Exemples (avec hex dumps annotés)
Annexe A : CDDL Schema complet
Annexe B : Vecteurs de test
Annexe C : Mapping HTML → CBOR-Web
Annexe D : Comparaison avec llms.txt, sitemap.xml
```

### 7. Niveaux de conformité

Trois niveaux pour faciliter l'adoption progressive :

| Niveau | Exigence publisher | Exigence agent |
|--------|-------------------|----------------|
| **Minimal** | Manifest + pages individuelles. Blocs `h` et `p` uniquement | Lire manifest + pages |
| **Standard** | Manifest + pages + bundle. Tous les types de blocs. Hash SHA-256. Navigation complète | Lire tout + vérifier hash + cache conditionnel |
| **Full** | Standard + signature COSE + sous-manifests + streaming + diffs incrémentiels | Standard + vérifier signature + streaming + diff |

---

## Résumé pour claude.ia

Tu as le brouillon v0.1, tu as cette directive technique. Rédige la norme formelle v1.0 en respectant :

1. Le style RFC (RFC 2119 pour MUST/SHOULD/MAY)
2. Le schéma CDDL (RFC 8610) en Annexe A
3. Au moins 3 vecteurs de test avec hex dump CBOR
4. Les exigences de sécurité du §4 ci-dessus
5. Les 3 niveaux de conformité
6. La structure du §6

Le résultat doit être un document qu'un développeur peut lire et implémenter sans ambiguïté.
