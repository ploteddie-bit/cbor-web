---
name: Document de vision CBOR-Web cree
description: Document fondateur VISION-CBOR-WEB.md cree le 22 mars — contexte obligatoire pour tout agent travaillant sur le projet. Explique la boucle complete spec→text2cbor→crawler→signal propre→embeddings→auto-amelioration→token→autofinancement.
type: project
---

## Document de vision CBOR-Web

Cree le 22 mars 2026, session N4000. Fichier : `~/cbor-web/VISION-CBOR-WEB.md` + copie dans `Downloads/`.

### Contexte de creation

Eddie a constate que chaque collaborateur (Claude.ia, Claude Code, agents) ne voyait qu'un morceau du projet. Personne d'autre ne tenait la boucle complete en tete. Ce document capture la vision d'Eddie pour que tout agent qui le lit travaille avec le bon contexte.

### Points cles du document

1. La boucle complete : publisher → crawler → signal propre → auto-amelioration → token → autofinancement
2. Le crawler est la piece manquante critique (a construire en Rust)
3. CBOR-Web ne remplace PAS les embeddings — il ameliore leur qualite d'entree
4. Le token n'a de valeur que si le crawler existe
5. Instructions specifiques par type d'agent (spec, text2cbor, crawler, token, infra)

### Usage

Ce document doit etre donne en contexte a tout agent ou collaborateur AVANT qu'il travaille sur CBOR-Web. C'est le "CLAUDE.md du projet CBOR-Web".

**Why:** Sans ce document, chaque agent optimise son morceau sans comprendre la boucle. Claude.ia ecrit une spec sans penser au crawler. Le SG propose un RAG alors que le format lui-meme est la reponse.

**How to apply:** Copier-coller ce document en debut de conversation avec tout agent travaillant sur CBOR-Web. Pour Claude.ia : le donner AVANT de toucher a la spec ou aux companions.
