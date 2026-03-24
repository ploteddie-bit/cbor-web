---
name: CBOR-Web — Insight architectural signal propre + embeddings
description: Decision architecturale cle du 22 mars — CBOR-Web ne remplace PAS les embeddings/RAG, il leur fournit un signal 10x plus propre. Pas de taxonomie custom, le combo format+embeddings est la bonne approche.
type: project
---

## Insight architectural (22 mars 2026, session N4000)

### Ce qui a ete elimine

L'idee initiale etait de stocker la spec CBOR-Web dans pgvector/RAG pour que les agents puissent la requeter. Eddie a challenge cette approche en 3 etapes :

1. **"Tu vas transformer le document en binaire comme un CBOR ?"** → Non, le RAG classique c'est du texte chunke, pas du CBOR. Dogfooding = utiliser CBOR-Web lui-meme.

2. **"On vient de supprimer beaucoup de couches logiciel"** → Exact. HTML→chunker→embedding→pgvector→RAG→reranking→LLM (7 couches) remplace par Markdown→text2cbor→CBOR binaire→agent parse (2 couches).

3. **"Il faut indexer le binaire, avoir une architecture de classement normalisee ?"** → Puis Eddie a realise : "ca existe deja, embeddings."

### Decision finale

**CBOR-Web ne remplace PAS les embeddings. Il ameliore la qualite de l'entree.**

```
Aujourd'hui :  HTML sale (7% signal) → embedding → vecteur pollue
CBOR-Web :     Binaire propre (95% signal) → embedding → vecteur precis
```

### Le combo valide

| Couche | Role | Technologie |
|--------|------|-------------|
| **Format** | Signal propre, structure, deterministe | CBOR-Web |
| **Indexation** | Recherche semantique, cross-site, universelle | Embeddings |

- Le manifest CBOR-Web = index **deterministe** (navigation, hierarchie, hash, cross-refs)
- Les embeddings = index **semantique** (trouvez-moi tout ce qui parle de pricing)
- Pas besoin d'inventer une taxonomie custom — les embeddings le font deja

### Argument commercial du token

C'est CA l'argument pour le token : un agent qui indexe via CBOR-Web produit des embeddings 10x plus precis qu'un agent qui parse du HTML. L'acces au contenu propre (L1 token) = meilleure qualite d'indexation = meilleur avantage competitif pour l'agent.

**Why:** Eddie a deraille l'over-engineering (taxonomie normalisee) pour revenir a l'essentiel : CBOR-Web fournit le signal, les embeddings font le reste.

**How to apply:** Ne jamais proposer de remplacer les embeddings par un systeme CBOR-Web custom. Toujours positionner CBOR-Web comme le fournisseur de signal propre pour les systemes d'indexation existants (pgvector, Pinecone, Qdrant, etc.).
