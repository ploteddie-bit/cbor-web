#!/bin/bash
# cbor-server — deploy script
# Run on: serveur-build (10.0.0.135, MacPro, Rust 1.94)
# Deploys to: serveur-dev (10.0.0.201)

set -e

TARGET="serveur-dev"
TARGET_DIR="/home/eddie/cbor-server"
TOOLS_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Building cbor-server (release) ==="
cd "$(dirname "$0")"
cargo build --release

echo "=== Stopping service on $TARGET ==="
ssh "$TARGET" "systemctl --user stop cbor-server 2>/dev/null || true"

echo "=== Deploying binary ==="
scp target/release/cbor-server "$TARGET:$TARGET_DIR/"

echo "=== Deploying data directory ==="
rsync -avz --delete data/ "$TARGET:$TARGET_DIR/data/"

echo "=== Restarting service ==="
ssh "$TARGET" "systemctl --user start cbor-server"

echo "=== Done ==="
ssh "$TARGET" "systemctl --user status cbor-server --no-pager" || true
echo ""
echo "Server running on http://$TARGET:3001"
echo "Test: curl http://$TARGET:3001/.well-known/cbor-web"
