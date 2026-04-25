#!/bin/bash
# cbor-server — deploy script
# Run on: serveur-build (10.0.0.135, MacPro, Rust 1.94)
# Deploys to: serveur-dev (10.0.0.201)
#
# Prerequisites:
#   git clone https://github.com/ploteddie-bit/cbor-web.git /home/eddie/cbor-web

set -e

TARGET="serveur-dev"
TARGET_DIR="/home/eddie/cbor-server"
CBOR_WEB_DIR="$(cd "$(dirname "$0")/../.." && pwd)"

# Ensure cargo is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

echo "=== Building cbor-server (release) ==="
cd "$CBOR_WEB_DIR/tools/cbor-server"
cargo build --release

echo "=== Stopping service on $TARGET ==="
ssh "$TARGET" "systemctl --user stop cbor-server 2>/dev/null || true"

echo "=== Deploying binary ==="
scp target/release/cbor-server "$TARGET:$TARGET_DIR/"

echo "=== Deploying service file ==="
ssh "$TARGET" "mkdir -p ~/.config/systemd/user"
scp cbor-server.service "$TARGET:~/.config/systemd/user/"
ssh "$TARGET" "systemctl --user daemon-reload"

echo "=== Deploying data directory ==="
ssh "$TARGET" "mkdir -p $TARGET_DIR/data/.well-known/cbor-web/pages"
if [ -d data/.well-known ]; then
    rsync -avz --delete data/ "$TARGET:$TARGET_DIR/data/"
else
    echo "  (no local data/ — using existing data on $TARGET)"
fi

echo "=== Enabling linger (keep alive after SSH logout) ==="
ssh "$TARGET" "loginctl enable-linger 2>/dev/null || true"

echo "=== Starting service ==="
ssh "$TARGET" "systemctl --user enable --now cbor-server"

echo "=== Done ==="
sleep 1
ssh "$TARGET" "systemctl --user status cbor-server --no-pager" || true
echo ""
echo "Server running on http://$TARGET:3001"
echo "Test: curl -sI http://$TARGET:3001/.well-known/cbor-web"
