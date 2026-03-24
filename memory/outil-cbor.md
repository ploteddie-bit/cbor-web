---
name: outil-cbor
description: Convertisseur CBOR (alias cbor) — commandes et usage pour Eddie
type: reference
---

## Outil `cbor` — Convertisseur fichiers vers CBOR

Installé le 2026-03-22. Alias `cbor` dans .bashrc.

### Commandes

```bash
# Convertir un fichier (coller chemin Windows directement)
cbor "C:\Users\eddie\Downloads\fichier.md"

# Convertir un dossier entier
cbor "C:\Users\eddie\Documents\mon-dossier"

# Chemin Linux aussi
cbor /home/ubuntu/CLAUDE.md

# Voir les fichiers déjà convertis
cbor --list

# Relire un fichier .cbor
cbor --read ~/cbor-output/fichier.cbor

# Mode interactif (sans argument)
cbor
```

### Sortie

Tous les .cbor vont dans **`~/cbor-output/`**

### Gain moyen

~50-70% de compression selon le type de fichier.

### Formats supportés

.md .txt .json .yaml .yml .csv .tsv .docx .xlsx .html .xml
.sh .py .rs .toml .js .ts .tsx .sql .log .conf

### Fichiers source

- `/home/ubuntu/convertir.sh` — script principal (alias cbor)
- `/home/ubuntu/to-cbor.py` — moteur de conversion Python (cbor2 + zlib)
- `/home/ubuntu/cbor-output/` — dossier de sortie
