# La fin du web tel que nous le connaissons

**Le HTML a été construit pour des yeux. Le prochain web sera construit pour des cerveaux artificiels.**

---

En 2024, un seuil a été franchi en silence. Pour la première fois en dix ans, le trafic automatisé a dépassé le trafic humain sur internet : **51% des requêtes web proviennent désormais de machines**, selon le rapport Imperva Bad Bot Report 2025 (Thales Group). Cloudflare confirme dans son bilan 2025 que le trafic HTML d'origine humaine ne représente plus que 47% des requêtes. L'inversion est consommée.

Dans le même temps, Gartner prévoit une **chute de 25% du volume des moteurs de recherche traditionnels d'ici fin 2026**, remplacés par des agents IA conversationnels. Ce n'est pas une projection marginale : c'est le cabinet qui conseille les directions stratégiques du Fortune 500.

Les chiffres s'accumulent. Les recherches Google qui aboutissent à **zéro clic** — l'utilisateur obtient sa réponse sans jamais visiter un site — ont atteint 69% en mai 2025 selon Similarweb, contre 56% un an plus tôt. Quand une requête déclenche un AI Overview de Google, le taux de zéro-clic monte à **83%** (Seer Interactive, septembre 2025). Le trafic Google vers les éditeurs de contenu a chuté d'un tiers en un an selon Press Gazette. Les éditeurs interrogés anticipent une baisse supplémentaire de 43% sur les trois prochaines années.

## Le navigateur est un outil de compensation

Le navigateur web — Chrome, Firefox, Safari — est un interpréteur visuel. Il reçoit du HTML, applique du CSS, exécute du JavaScript, et produit des pixels sur un écran. Ce processus existe pour une seule raison : les humains lisent avec des yeux.

Un agent IA n'a pas d'yeux. Quand Claude, GPT ou Gemini doivent lire une page web, ils téléchargent le HTML, ignorent le CSS, ignorent le JavaScript, ignorent les images décoratives, ignorent les menus de navigation, ignorent les bannières cookies — et extraient péniblement le texte utile enfoui dans des milliers de balises `<div>`. Sur une page typique de 120 Ko, le contenu exploitable représente **7% du poids total** (HTML Waste Benchmark, CBOR-Web Spec v2.1, mesuré sur 200 sites). L'agent paie pour 100% et utilise 7%.

Ce gaspillage a un coût concret. À raison de 20 000 tokens par page HTML, un agent qui lit 1000 pages par jour dépense environ **36 dollars quotidiens** en tokens chez un fournisseur IA. Le même contenu en format binaire structuré tiendrait en 100 tokens par page — coût : 2 dollars par jour. La différence n'est pas une optimisation. C'est un changement d'ordre de grandeur.

## Les géants préparent la transition

Google a lancé en 2025 le **Universal Commerce Protocol** (UCP), un protocole qui permet aux agents IA de découvrir des produits, authentifier des utilisateurs et finaliser des achats sans navigateur. OpenAI, Anthropic et Perplexity sont entrés dans ce que Gartner qualifie de « guerre des navigateurs IA » — non pas pour améliorer Chrome, mais pour le rendre obsolète.

Le **Model Context Protocol** (MCP), soutenu par Anthropic, OpenAI, Google et Microsoft, est devenu le standard de facto pour connecter les agents IA aux systèmes d'entreprise. Ce n'est plus du HTTP vers du HTML : c'est du machine-à-machine structuré. Gartner prévoit que **40% des applications d'entreprise intégreront des agents IA spécialisés d'ici fin 2026**, contre moins de 5% début 2025.

## Ce qui remplace le navigateur

Le futur n'est pas un meilleur navigateur. C'est l'absence de navigateur.

Un être humain qui veut réserver un vol ne cherchera pas « billet Paris Madrid » sur Google pour comparer 15 onglets. Il dira à son agent : « Réserve-moi un Paris-Madrid mardi prochain, moins de 200 euros. » L'agent interrogera directement les systèmes des compagnies aériennes en format binaire — pas leurs sites web. La transaction se fera en millisecondes, sans qu'un seul pixel ne soit rendu.

Pour que ce futur fonctionne, il manque une pièce : un **format standard** permettant à n'importe quel site d'exposer son contenu en binaire aux machines, en parallèle de son HTML pour les humains. C'est exactement ce que proposent les nouvelles spécifications comme CBOR-Web : un fichier unique (`index.cbor`) à la racine du site, lisible nativement par les agents, pesant 50 Ko là où le HTML en pèse 500.

## La cohabitation, pas le remplacement

Le HTML ne disparaîtra pas demain. Mais il deviendra ce que le fax est devenu après l'email : un format hérité, maintenu par inertie, utilisé par une fraction décroissante du trafic. Les sites continueront à servir du HTML pour les visiteurs humains — tant qu'il en restera assez pour justifier le coût. Parallèlement, le canal binaire deviendra le canal principal, celui par lequel transitent les ventes, les transactions, les recommandations et les décisions.

Le web de 2030 n'aura probablement plus de « pages ». Il aura des **flux de données structurées** consommés par des agents qui agissent au nom d'humains. Le navigateur aura rejoint le Minitel dans les musées de l'informatique. Et la question ne sera plus « quel site visiter ? » mais « quel agent envoyer ? »

---

*Eddie Plot — ExploDev, mars 2026*

**Sources :**
- [Imperva Bad Bot Report 2025 — Thales Group](https://cpl.thalesgroup.com/about-us/newsroom/2025-imperva-bad-bot-report-ai-internet-traffic)
- [Cloudflare Radar 2025 Year in Review](https://blog.cloudflare.com/radar-2025-year-in-review/)
- [Gartner — Search Engine Volume Drop 25% by 2026](https://www.gartner.com/en/newsroom/press-releases/2024-02-19-gartner-predicts-search-engine-volume-will-drop-25-percent-by-2026-due-to-ai-chatbots-and-other-virtual-agents)
- [Gartner — 40% Enterprise Apps with AI Agents by 2026](https://www.gartner.com/en/newsroom/press-releases/2025-08-26-gartner-predicts-40-percent-of-enterprise-apps-will-feature-task-specific-ai-agents-by-2026-up-from-less-than-5-percent-in-2025)
- [Gartner — AI Browsers Disruptive Shift](https://www.gartner.com/en/documents/6860266)
- [Zero-Click Search 58% — Superprompt/Similarweb](https://superprompt.com/blog/zero-click-search-worsens-58-percent-google-no-clicks-november-2025-recovery-strategies)
- [AI Overviews CTR -61% — Seer Interactive via Dataslayer](https://www.dataslayer.ai/blog/google-ai-overviews-the-end-of-traditional-ctr-and-how-to-adapt-in-2025)
- [Google Traffic to Publishers -33% — Press Gazette](https://pressgazette.co.uk/media-audience-and-business-data/google-traffic-down-2025-trends-report-2026/)
