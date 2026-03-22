# DIRECTIVE-CLAUDE-IA — Méthodologie de travail sur les documents CBOR-Web

> Ce document est une directive de travail obligatoire. Tu le lis INTÉGRALEMENT avant de toucher à un document CBOR-Web. Chaque règle existe parce qu'une erreur réelle a été commise sans elle.

---

## Pourquoi cette directive existe

Ce projet produit un standard technique destiné à la publication. Un standard publié avec des incohérences internes perd sa crédibilité — pas demain, pas à la première review, mais au moment exact où un développeur copie un exemple et que ça ne marche pas.

Les boucles de revue-correction précédentes ont montré un pattern : chaque correction résout un problème local mais en introduit un nouveau ailleurs, parce que le correcteur agit sur ce qu'il voit sans vérifier ce qu'il ne voit pas. Ce document casse ce pattern.

---

## Les 3 piliers — à quoi sert chaque correction

Avant de proposer une modification, identifie quel pilier elle protège. Si elle n'en protège aucun, ne la fais pas.

| Pilier | Question | Conséquence d'une violation |
|--------|----------|----------------------------|
| **Rigueur technique** | Un développeur peut-il implémenter à partir de cet exemple sans me poser de question ? | Un implémenteur produit du code non conforme. Deux implémentations sont incompatibles. |
| **Cohérence interne** | Si la même chose est décrite à deux endroits, disent-ils la même chose ? | Un développeur qui lit §5 obtient un résultat différent d'un développeur qui lit l'Appendix. |
| **Complétude structurelle** | Chaque section est-elle trouvable, référencée, et sans artefacts de travail ? | Un développeur ne trouve pas une section, suit un lien cassé, ou doute que le document est finalisé. |

---

## Règle 0 — Comprendre avant d'agir

Avant toute correction, prouve en 3 lignes que tu comprends :
1. Le rôle du document spécifique que tu corriges dans la suite de 6
2. Qui va le lire et dans quel but (développeur implémentant, revieweur IETF, agent IA)
3. Quel pilier chaque correction que tu proposes protège

Si tu ne peux pas le faire, tu n'es pas prêt à corriger.

---

## Règle 1 — Inventaire AVANT correction

Quand tu identifies un problème, tu NE corriges PAS immédiatement. Tu fais d'abord un **inventaire exhaustif** de toutes les occurrences du même pattern dans le document entier.

**Méthode :**
1. Identifier le pattern fautif
2. Scanner le document ENTIER pour toutes les occurrences — prose, exemples, CDDL, appendices, test vectors, tables, commentaires de code
3. Lister chaque occurrence avec son numéro de ligne et son état
4. Corriger TOUTES les occurrences en une seule passe

**Jamais :**
- Corriger une occurrence et oublier les autres
- Corriger le contenu principal et oublier l'appendix qui montre la même chose
- Corriger un champ dans un exemple mais pas le même champ dans un autre exemple

---

## Règle 2 — Ne jamais aggraver

Une correction DOIT être **strictement meilleure** que l'état précédent. Si elle est latérale (aussi bien qu'avant) ou pire, ne la fais pas.

**Cas d'aggravation fréquents :**

| Situation | Ce qu'il ne faut PAS faire | Ce qu'il faut faire |
|-----------|---------------------------|---------------------|
| Hash placeholder annoté `; PLACEHOLDER` | Le remplacer par un faux hash sans annotation | Le laisser tel quel OU calculer le vrai hash |
| Référence cross-document par numéro `§13` | La remplacer par un titre incorrect | La remplacer par le bon titre OU la laisser |
| Commentaire de debug dans la prose | Le supprimer en cassant la phrase | Le reformuler proprement |
| Exemple en ordre non-déterministe | Corriger l'ordre mais casser la structure des données | Corriger l'ordre ET vérifier que les données restent valides |

**Principe : si tu n'es pas sûr que ta correction est strictement meilleure, ne la fais pas et signale le problème sans le corriger.**

---

## Règle 3 — Vérification par balayage après correction

Après avoir appliqué toutes tes corrections, tu fais un balayage complet. Pas optionnel.

### Checklist de balayage

**A. Structure**
- [ ] La Table des Matières liste chaque `## Chapitre` du document
- [ ] Chaque `##` a le bon niveau de heading (pas de `##` pour une sous-section)
- [ ] Les numéros de section sont séquentiels sans trou
- [ ] Chaque référence croisée interne (`§X.Y`) pointe vers une section qui existe

**B. Cohérence des exemples**
- [ ] Chaque bloc de code `cbor-diag` respecte les règles du document (encodage déterministe, tags, types)
- [ ] Si le même concept est illustré à plusieurs endroits (§, Appendix, test vector), les exemples sont identiques
- [ ] Les tailles annoncées (`"size": 127`) correspondent aux hex dumps si un hex dump est présent

**C. CDDL**
- [ ] Chaque type défini en CDDL est cohérent avec la prose qui le décrit
- [ ] Les champs REQUIRED en prose sont sans `?` en CDDL
- [ ] Les champs OPTIONAL en prose ont `?` en CDDL
- [ ] Les maps qui doivent être extensibles (forward-compatibility) ont `* tstr => any` ou `* int => any`

**D. Références cross-document**
- [ ] Chaque `See CBOR-WEB-XYZ.md §N` pointe vers un document et une section qui existent (ou qui sont planifiés)
- [ ] Aucun artefact de développement ne subsiste ("point 12 in corrections", "TODO", "FIXME", "wait, let me check")

**E. Données**
- [ ] Les hash marqués PLACEHOLDER sont annotés explicitement
- [ ] Les hash avec une valeur réelle correspondent à un calcul vérifiable
- [ ] Les timestamps sont cohérents entre eux (un `"updated"` n'est pas antérieur à un `"published"`)

---

## Règle 4 — Format de livraison

Quand tu livres tes corrections, structure-les comme suit :

```
## Corrections appliquées

### [FINDING-ID] — [Titre court]
**Pilier :** [Rigueur / Cohérence / Complétude]
**Lignes affectées :** [N, M, P]
**Occurrences corrigées :** [X / X total]
**Vérification :** [comment vérifier que la correction est correcte]
```

Ceci permet au donneur d'ordre de vérifier sans relire 3000 lignes.

---

## Règle 5 — Scope de travail

Quand on te donne un document à corriger, tu fais UNIQUEMENT ce qui est demandé :

- **"Identifier les incohérences"** → Tu listes, tu prouves, tu proposes. Tu ne modifies RIEN.
- **"Corriger les incohérences"** → Tu appliques les corrections ET tu fais le balayage (Règle 3).
- **"Vérifier les corrections"** → Tu relis le document corrigé et tu vérifies que chaque correction est strictement meilleure et que rien de nouveau n'a été cassé.

Ne déborde jamais du scope. Si tu vois un problème hors scope, signale-le en fin de réponse sous "Observations hors scope" — ne le corrige pas.

---

## Règle 6 — Spécifique CBOR-Web : encodage déterministe

Ce standard repose sur le Core Deterministic Encoding (RFC 8949 §4.2). Chaque diagnostic CBOR dans chaque document DOIT avoir ses clés dans le bon ordre. C'est le bug le plus fréquent.

**Algorithme de tri :**
1. Encoder chaque clé en CBOR
2. Trier par longueur de l'encodage (plus court d'abord)
3. À longueur égale, trier byte par byte (valeur plus basse d'abord)

**Conséquence :** un entier `0` (1 byte) sort toujours avant un texte `"t"` (2 bytes). Un texte `"t"` (2 bytes) sort toujours avant un texte `"alt"` (4 bytes). Parmi les textes de même longueur, c'est le contenu qui départage : `"hash"` (5B, 3ème byte `68`) avant `"lang"` (5B, 3ème byte `6C`).

**Quand tu corriges ou vérifies un diagnostic CBOR :**
1. Liste chaque clé de la map
2. Calcule la taille encodée de chaque clé
3. Trie
4. Vérifie que l'exemple montre cet ordre exact
5. Si une sous-map existe, applique récursivement

---

## Résumé

| Règle | En un mot | Erreur qu'elle prévient |
|-------|-----------|------------------------|
| 0 | Comprendre | Corriger sans savoir pourquoi |
| 1 | Inventorier | Corriger une occurrence sur trois |
| 2 | Ne pas aggraver | Remplacer un problème par un pire |
| 3 | Balayer | Laisser des résidus après correction |
| 4 | Structurer | Corrections invérifiables |
| 5 | Scope | Déborder et introduire du bruit |
| 6 | Tri CBOR | Le bug #1 de ce projet |

---

*Ce document s'applique à tous les fichiers de la suite CBOR-Web : CORE, MULTIMEDIA, GENERATIVE, SECURITY, ECONOMICS, REFERENCE.*
