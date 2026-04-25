# Outils de lecture d'image — Gratuits / Open Source

> Comparatif pour intégration CBOR-Web (extraction texte depuis images, OCR, description)

## Tableau comparatif

| Outil | Type | Gratuit ? | Qualité | Langues | CPU/GPU | Install |
|-------|------|-----------|---------|---------|---------|---------|
| **Tesseract 5** | OCR classique | ✅ 100% gratuit | 85% texte clair | 100+ | CPU | `apt install tesseract-ocr` |
| **EasyOCR** | Deep Learning | ✅ Gratuit | 92% | 80+ | GPU reco | `pip install easyocr` |
| **PaddleOCR** | Deep Learning | ✅ Gratuit | 95% | 80+ | GPU/CPU | `pip install paddleocr` |
| **Surya** | Vision+Texte | ✅ Gratuit | 90% | 90+ | GPU reco | `pip install surya-ocr` |
| **TrOCR** | Transformer | ✅ Gratuit | 93% | Anglais | GPU | `pip install transformers` |
| **DocTR** | Document | ✅ Gratuit | 91% | Multi | GPU/CPU | `pip install doctr` |
| **Tesseract.js** | OCR navigateur | ✅ Gratuit | 80% | 100+ | CPU | `<script>` CDN |
| **Google Vision** | Cloud API | 🟡 1000/mois gratuit | 98% | 200+ | Cloud | Clé API |
| **Azure Vision** | Cloud API | 🟡 5000/mois gratuit | 97% | 70+ | Cloud | Clé API |
| **Claude Vision** | LLM multimodal | ❌ Payant | 99% | Toutes | Cloud | API key |
| **GPT-4o** | LLM multimodal | ❌ Payant | 99% | Toutes | Cloud | API key |

## Recommandation pour CBOR-Web

### Phase 1 — Gratuit immédiat
```bash
pip install easyocr
```

```python
import easyocr
reader = easyocr.Reader(['fr', 'en', 'es'])
result = reader.readtext('screenshot.png')
# → [([[x,y],...], 'texte détecté', 0.98), ...]
```

**Pourquoi EasyOCR :**
- Gratuit, pas de limite
- 80+ langues (français inclus)
- Fonctionne sur CPU (lent) ou GPU (rapide)
- Installation simple : `pip install easyocr`
- Parfait pour extraire du texte de captures d'écran, photos, PDFs

### Phase 2 — Qualité maximale (gratuit aussi)
```bash
pip install paddlepaddle paddleocr
```

PaddleOCR est le meilleur OCR gratuit actuel (95%+ précision, développé par Baidu).

## Intégration CBOR-Web possible

Le bloc `image` CBOR-Web pourrait inclure un champ `ai_description` extrait automatiquement :

```json
{
  "t": "image",
  "src": "screenshot.png",
  "alt": "Capture d'écran Alchemy faucet",
  "ai_description": "Page montrant un formulaire avec champ Wallet Address, sélecteur de chaîne Ethereum, bouton Continue",
  "text_extracted": "Ethereum Sepolia Faucet | Drop in your wallet details..."
}
```

## Script d'exemple

```python
# ocr-cbor.py — extrait le texte d'une image et produit un bloc CBOR-Web
import easyocr, json, sys

reader = easyocr.Reader(['fr', 'en'])
results = reader.readtext(sys.argv[1])

blocks = []
for bbox, text, confidence in results:
    blocks.append({"t": "p", "v": text, "confidence": round(confidence, 2)})

manifest = {
    "@type": "cbor-web-page",
    "blocks": blocks,
    "source": sys.argv[1],
    "extracted_chars": sum(len(b["v"]) for b in blocks)
}

print(json.dumps(manifest, indent=2, ensure_ascii=False))
```

Usage :
```bash
pip install easyocr
python3 ocr-cbor.py "Capture d'écran 2026-04-25.png"
```
