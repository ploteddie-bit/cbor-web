#!/usr/bin/env python3
"""
OCR-to-CBOR pipeline — extrait texte + métadonnées d'une image
et produit un bloc CBOR-Web multimedia enrichi.

Utilise Tesseract pour l'OCR + Pillow pour les métadonnées.
Gratuit, local, 100+ langues. Aucune API externe.

Usage:
  python3 ocr-to-cbor.py screenshot.png > output.json
"""
import sys, json, io, struct
from pathlib import Path
from PIL import Image
import pytesseract

def dominant_color(img: Image.Image) -> str:
    """Extract dominant color in hex."""
    img_small = img.resize((50, 50))
    pixels = list(img_small.getdata())
    # Most common color
    from collections import Counter
    c = Counter(pixels).most_common(1)[0][0]
    if isinstance(c, tuple) and len(c) >= 3:
        return f"#{c[0]:02x}{c[1]:02x}{c[2]:02x}"
    return "#000000"

def ocr_text(img: Image.Image, lang: str = "fra+eng+spa") -> str:
    """Extract text via Tesseract OCR."""
    return pytesseract.image_to_string(img, lang=lang).strip()

def cbor_encode_value(val, buf: io.BytesIO):
    """Minimal deterministic CBOR encoder for our subset."""
    if val is True: buf.write(b'\xf5')
    elif val is False: buf.write(b'\xf4')
    elif val is None: buf.write(b'\xf6')
    elif isinstance(val, int):
        if val >= 0:
            if val < 24: buf.write(bytes([val]))
            elif val < 256: buf.write(bytes([0x18, val]))
            elif val < 65536: buf.write(struct.pack('>BH', 0x19, val))
            else: buf.write(struct.pack('>BI', 0x1a, val))
        else:
            val = -1 - val
            if val < 24: buf.write(bytes([0x20 | val]))
            elif val < 256: buf.write(bytes([0x38, val]))
            elif val < 65536: buf.write(struct.pack('>BH', 0x39, val))
            else: buf.write(struct.pack('>BI', 0x3a, val))
    elif isinstance(val, str):
        b = val.encode('utf-8')
        l = len(b)
        if l < 24: buf.write(bytes([0x60 | l]))
        elif l < 256: buf.write(bytes([0x78, l]))
        else: buf.write(bytes([0x79]) + struct.pack('>H', l))
        buf.write(b)
    elif isinstance(val, dict):
        # Sort keys by CBOR encoding length then bytewise
        items = sorted(val.items(), key=lambda kv: _cbor_key_sort(kv[0]))
        l = len(items)
        if l < 24: buf.write(bytes([0xa0 | l]))
        elif l < 256: buf.write(bytes([0xb8, l]))
        else: buf.write(bytes([0xb9]) + struct.pack('>H', l))
        for k, v in items:
            cbor_encode_value(k, buf)
            cbor_encode_value(v, buf)
    elif isinstance(val, list):
        l = len(val)
        if l < 24: buf.write(bytes([0x80 | l]))
        elif l < 256: buf.write(bytes([0x98, l]))
        else: buf.write(bytes([0x99]) + struct.pack('>H', l))
        for v in val:
            cbor_encode_value(v, buf)

def _cbor_key_sort(key) -> tuple:
    """Sort key: shortest CBOR encoding first, then bytewise."""
    buf = io.BytesIO()
    cbor_encode_value(key, buf)
    return (len(buf.getvalue()), buf.getvalue())

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 ocr-to-cbor.py <image.png>", file=sys.stderr)
        sys.exit(1)

    path = Path(sys.argv[1])
    if not path.exists():
        print(f"File not found: {path}", file=sys.stderr)
        sys.exit(1)

    img = Image.open(path)
    w, h = img.size
    fmt = Image.MIME.get(img.format, f"image/{img.format.lower()}" if img.format else "image/png")
    fsize = path.stat().st_size

    # Extract
    text = ocr_text(img)
    color = dominant_color(img)
    semantic = "screenshot" if "screen" in path.name.lower() else "illustration"

    # Build CBOR-Web multimedia page
    blocks = []

    # Image block (rich multimedia)
    image_block = {
        "t": "image", "trust": 0,
        "src": str(path), "alt": f"Image: {path.name}",
        "semantic_role": semantic,
        "dimensions": {"w": w, "h": h},
        "format": fmt,
        "file_size": fsize,
        "dominant_color": color,
        "ai_description": text[:500] if text else "",
    }
    blocks.append(image_block)

    # If OCR found text, add as paragraphs
    if text:
        blocks.append({"t": "h", "l": 2, "v": "Texte extrait par OCR"})
        for line in text.split('\n')[:50]:
            line = line.strip()
            if line:
                blocks.append({"t": "p", "v": line})

    # Build page document
    page = {
        0: "cbor-web-page",
        1: 2,
        2: {"path": f"/ocr/{path.name}", "canonical": str(path.absolute()), "lang": "fr"},
        3: {
            "title": f"OCR: {path.name}",
            "description": f"Image {w}x{h}, {fmt}, {fsize} bytes. Texte extrait: {len(text)} chars.",
        },
        4: blocks,
    }

    # Encode to CBOR bytes
    buf = io.BytesIO()
    # Self-described tag
    buf.write(b'\xd9\xd9\xf7')
    cbor_encode_value(page, buf)

    # Output
    cbor_bytes = buf.getvalue()

    # Also output JSON summary for readability
    result = {
        "file": str(path),
        "dimensions": f"{w}x{h}",
        "format": fmt,
        "file_size": fsize,
        "dominant_color": color,
        "ocr_text_length": len(text),
        "ocr_text": text[:300],
        "cbor_bytes": len(cbor_bytes),
        "cbor_hex": cbor_bytes[:200].hex() + "...",
        "blocks_generated": len(blocks),
    }
    print(json.dumps(result, indent=2, ensure_ascii=False))

if __name__ == "__main__":
    main()
