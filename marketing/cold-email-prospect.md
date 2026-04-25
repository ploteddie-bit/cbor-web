# Template — Email de prospection CRM

Utiliser ce template pour contacter les prospects CRM : blogs tech, sites de documentation, agences web, plateformes e-commerce.

---

**Objet :** Les agents IA ne peuvent pas lire {{SITE_NAME}} efficacement — et c'est réglable en 30 minutes

**Corps :**

Bonjour {{PRENOM}},

Je suis tombé sur **{{SITE_NAME}}** en cherchant {{CONTEXTE_DECOUVERTE}} — {{COMPLIMENT_SPECIFIQUE}}.

Je vous écris parce que j'ai identifié un angle mort qui touche tous les sites de contenu aujourd'hui, y compris le vôtre.

Quand ChatGPT, Claude, Perplexity ou un pipeline RAG d'entreprise crawle {{SITE_NAME}}, l'agent télécharge l'intégralité du HTML — menus, CSS, JavaScript, trackers, popups — pour extraire le contenu utile. Résultat : **93 % des tokens sont gaspillés** à stripper du bruit. Pour un site de la taille du vôtre, ça représente des centaines de dollars de coûts d'inférence gaspillés *par mois*, supportés par les agents qui vous lisent — ou pire, ils sautent vos pages parce que c'est trop cher.

**J'ai construit la solution.** CBOR-Web est un protocole ouvert qui permet à n'importe quel site de publier une copie binaire structurée de son contenu, en parallèle du HTML existant. Les humains voient le site normal. Les agents IA reçoivent un flux 100× plus petit, 100 % structuré, sans tokenisation.

**Intégration : 30 minutes, une commande, zéro changement sur votre site.**
```bash
text2cbor generate --input ./votre-site --output ./cbor --domain {{DOMAINE}}
```

Ce qui est déjà en production :
- **38 sites** servis depuis le CDN Cloudflare (300+ villes)
- **8 SDKs** (TypeScript, Python, Go, PHP, Ruby, Java, C++, React) — zéro dépendance
- **IETF Internet-Draft** soumis (`draft-plot-cbor-web-00`)
- **npm** : `@deltopide_edy/cborweb`

**Mon offre :** Je vous propose un essai gratuit de la plateforme SaaS **text2cbor** — je convertis {{NOMBRE_PAGES}} pages de {{SITE_NAME}} en CBOR-Web, je vous montre le dashboard analytics (quels agents lisent quoi, combien de tokens économisés), et si ça vous plaît, on active la version complète. Aucun engagement.

Une visio de 20 minutes cette semaine ou la prochaine ?

Bonne journée,
**Eddie Plot**
Fondateur, ExploDev / Deltopide SL
[cbor-web.explodev.workers.dev](https://cbor-web.explodev.workers.dev)

---

**P.S.** — Un fait qui surprend toujours : la page d'accueil d'un site React moyenne pèse 1,6 Mo (HTML + JS bundle + assets). La même page en CBOR-Web pèse **536 octets**. C'est un ratio de compression de 1 000:1, et l'agent reçoit exactement le même contenu éditorial. Pas de perte. Juste du signal.

---

## Guide d'utilisation

| Variable | Description |
|----------|-------------|
| `{{SITE_NAME}}` | Nom du site prospecté |
| `{{PRENOM}}` | Prénom du contact |
| `{{CONTEXTE_DECOUVERTE}}` | Comment vous avez trouvé leur site (ex: "une recherche sur la documentation OAuth", "une reco HN") |
| `{{COMPLIMENT_SPECIFIQUE}}` | Un compliment sincère et spécifique sur leur contenu (ex: "vos guides API sont remarquablement clairs") |
| `{{DOMAINE}}` | Nom de domaine (ex: docs.stripe.com) |
| `{{NOMBRE_PAGES}}` | Nombre estimé de pages à convertir |

## Règles d'envoi

1. **Personnaliser systématiquement** `COMPLIMENT_SPECIFIQUE` et `CONTEXTE_DECOUVERTE` — jamais générique
2. **Vérifier que le site n'est pas déjà en CBOR-Web** avant d'envoyer (curl `/.well-known/cbor-web`)
3. **Relance à J+7** si pas de réponse, avec un lien vers l'article de blog
4. **Ne pas envoyer** aux prospects qui utilisent déjà llms.txt — ils sont en avance, proposer CBOR-Web comme couche complémentaire
