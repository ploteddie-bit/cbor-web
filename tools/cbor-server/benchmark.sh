#!/bin/bash
# benchmark.sh — CBOR-Web compression ratio benchmarks
# Measures: HTML size → CBOR size → compression ratio
# Run from: tools/cbor-server/

set -e

OUT="benchmark-$(date +%Y%m%d-%H%M%S).json"
echo "=== CBOR-Web Benchmark ==="

# Test sites (HTML dir → domain)
declare -A SITES=(
    ["../deltopide-site"]="deltopide.com"
)

# Build text2cbor if needed
cd "$(dirname "$0")/../text2cbor"
cargo build --release 2>/dev/null

T2CBOR="$(pwd)/target/release/text2cbor"
TMP="/tmp/cbor-benchmark"
rm -rf "$TMP"
mkdir -p "$TMP"

RESULTS='{"benchmarks":[]}'
ITEMS=""

for HTML_DIR in "${!SITES[@]}"; do
    DOMAIN="${SITES[$HTML_DIR]}"
    FULL_HTML_DIR="$(cd "$HTML_DIR" 2>/dev/null && pwd)" || continue

    echo ""
    echo "📊 $DOMAIN"

    # Measure total HTML size
    HTML_BYTES=$(find "$FULL_HTML_DIR" -name "*.html" -exec cat {} + | wc -c)
    HTML_FILES=$(find "$FULL_HTML_DIR" -name "*.html" | wc -l)

    # Generate CBOR
    OUT_DIR="$TMP/$DOMAIN"
    $T2CBOR generate -i "$FULL_HTML_DIR" -o "$OUT_DIR" -d "$DOMAIN" --default-lang en --spec-version 2.1 2>/dev/null

    # Measure CBOR sizes
    CBOR_BYTES=0
    if [ -d "$OUT_DIR/.well-known/cbor-web/pages" ]; then
        CBOR_BYTES=$(find "$OUT_DIR/.well-known/cbor-web" -name "*.cbor" -exec cat {} + | wc -c)
    fi

    if [ "$HTML_BYTES" -gt 0 ]; then
        RATIO=$(echo "scale=1; $HTML_BYTES / $CBOR_BYTES" | bc 2>/dev/null || echo "0")
        REDUCTION=$(echo "scale=1; 100 - ($CBOR_BYTES * 100 / $HTML_BYTES)" | bc 2>/dev/null || echo "0")
    else
        RATIO="0"
        REDUCTION="0"
    fi

    echo "  HTML: $HTML_BYTES bytes ($HTML_FILES files)"
    echo "  CBOR: $CBOR_BYTES bytes"
    echo "  Ratio: ${RATIO}:1 (${REDUCTION}% reduction)"

    if [ -n "$ITEMS" ]; then ITEMS="$ITEMS,"; fi
    ITEMS="$ITEMS{\"domain\":\"$DOMAIN\",\"html_bytes\":$HTML_BYTES,\"cbor_bytes\":$CBOR_BYTES,\"ratio\":$RATIO,\"reduction_pct\":$REDUCTION,\"pages\":$HTML_FILES}"
done

echo "{\"benchmarks\":[$ITEMS]}" > "$OUT"
echo ""
echo "=== Results saved to $OUT ==="
cat "$OUT" | python3 -m json.tool 2>/dev/null || cat "$OUT"
