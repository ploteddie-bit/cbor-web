# cbor-crawl — Crawler CBOR-Web

**Client CBOR-Web pour agents IA**

```
Langage :   Rust
Crates :    reqwest, ciborium, clap, tokio
Binaire :   cbor-crawl
Compile :   MacPro
```

---

## Ce que ça fait

```
cbor-crawl https://deltopide.fr
```

1. Vérifie `/.well-known/cbor-web` → récupère le manifest
2. Parse le manifest CBOR → liste des pages, hash, tailles
3. Télécharge le bundle (si dispo) ou les pages individuelles
4. Parse chaque page → extrait les content blocks
5. Sort le contenu propre (texte structuré, prêt pour embedding)

---

## Modes

| Mode | Commande | Usage |
|------|----------|-------|
| **inspect** | `cbor-crawl inspect https://site.com` | Affiche le manifest : pages, langues, tailles, accès |
| **fetch** | `cbor-crawl fetch https://site.com` | Télécharge tout, sort le contenu en JSON ou texte brut |
| **watch** | `cbor-crawl watch https://site.com` | Surveille les changements (hash diff), re-fetch si modifié |
| **verify** | `cbor-crawl verify fichier.cbor` | Vérifie un fichier CBOR local (structure, hash, encoding déterministe) |

---

## Output

Par défaut, JSON structuré sur stdout :

```json
{
  "site": "deltopide.fr",
  "pages": [
    {
      "path": "/",
      "title": "Deltopide — Agence Digitale",
      "lang": "fr",
      "blocks": [
        {"type": "h", "level": 1, "text": "Deltopide — Agence Digitale"},
        {"type": "p", "text": "Nous créons des sites web performants..."},
        {"type": "ul", "items": ["SEO", "Développement", "Design"]}
      ]
    }
  ]
}
```

Options :
- `--format json` (défaut) — structuré, pour pipelines
- `--format text` — texte brut, pour embeddings directs
- `--format cbor` — re-sérialise en CBOR (pour stockage binaire)
- `--output dir/` — un fichier par page

---

## Authentification token

```
cbor-crawl fetch https://site.com --wallet 0x1234... --keyfile ~/.cbor-web/key
```

Si le manifest indique des pages `"access": "token"` :
- Signe la requête avec la clé privée du wallet
- Envoie les headers `X-CBOR-Web-Wallet`, `X-CBOR-Web-Sig`, `X-CBOR-Web-Nonce`
- Si pas de token → récupère seulement les pages publiques (L0)

---

## Cache local

```
~/.cbor-web/cache/
  deltopide.fr/
    manifest.cbor          # dernier manifest
    pages/
      _index.cbor
      about.cbor
      services_seo.cbor
    hashes.json            # hash de chaque page pour diff
```

Au prochain `fetch` :
- Compare les hash du nouveau manifest avec le cache
- Ne télécharge que les pages modifiées
- Respecte `rate_limit` du manifest

---

## Intégration

Le crawler est un outil CLI standard. Il s'intègre dans n'importe quel pipeline :

```bash
# Embedding pipeline
cbor-crawl fetch https://deltopide.fr --format text | embedding-tool --model ada-002

# Stockage dans tiroir
cbor-crawl fetch https://deltopide.fr --format json | tiroir-cli.sh store "deltopide-cbor"

# Monitoring (cron)
cbor-crawl watch https://deltopide.fr --interval 3600 --on-change "notify.sh"
```

---

## Respect du protocole

- HTTPS obligatoire (refuse HTTP sauf localhost)
- Respecte `rate_limit.requests_per_second` du manifest
- Respecte `robots.txt` du site
- Respecte `bundle_cooldown_seconds` entre re-downloads
- User-Agent : `cbor-crawl/0.1.0 (cbor-web)`
- Vérifie le tag 55799 (magic bytes D9 D9 F7) avant parsing
- Vérifie les hash SHA-256 après téléchargement

---

## Compilation

```bash
ssh macpro "cd /data/builds/cbor-crawl && cargo build --release"
```

Même workflow que text2cbor — compilé sur MacPro, binaire copié où nécessaire.
