#!/bin/bash
# bootstrap.sh — convert text2cbor output into cbor-server data directory
#
# Usage:
#   cd tools/text2cbor && cargo run -- generate --output ../cbor-server/data ...
#   cd tools/cbor-server && ./bootstrap.sh
#
# Or directly:
#   ./bootstrap.sh /path/to/text2cbor-output

set -e

SRC="${1:-data}"
DEST="data/.well-known/cbor-web"

echo "=== CBOR-Web Server Bootstrap ==="
echo "Source: $SRC"
echo "Dest:   $DEST"

mkdir -p "$DEST/pages"

# v2.1 format: .well-known/cbor-web/ with manifest.cbor, bundle.cbor, pages/
if [ -f "$SRC/.well-known/cbor-web/manifest.cbor" ]; then
    echo "Found v2.1 output"
    cp "$SRC/.well-known/cbor-web/manifest.cbor" "$DEST/"
    cp "$SRC/.well-known/cbor-web/bundle.cbor" "$DEST/" 2>/dev/null || true
    if [ -d "$SRC/.well-known/cbor-web/pages" ]; then
        cp "$SRC/.well-known/cbor-web/pages/"*.cbor "$DEST/pages/" 2>/dev/null || true
    fi

# v3.0 format: index.cbor at root
elif [ -f "$SRC/index.cbor" ]; then
    echo "Found v3.0 output"
    cp "$SRC/index.cbor" "$DEST/manifest.cbor"
    cp "$SRC/index.cbor" "$DEST/bundle.cbor"
    # v3.0 pages are embedded in index.cbor, no separate page files

else
    echo "ERROR: No CBOR output found in $SRC"
    echo "Run text2cbor first:"
    echo "  cd ../text2cbor && cargo run -- generate --output ../cbor-server/data --domain example.com --input ../cbor-web-sites/cbor-web.com --spec-version 2.1"
    exit 1
fi

echo ""
echo "=== Data directory ready ==="
find "$DEST" -type f -exec ls -lh {} \; | sed 's/.*data/data/'
echo ""
echo "Start server: cargo run --release -- --data data"
echo "Test: curl -sI http://localhost:3001/.well-known/cbor-web"
