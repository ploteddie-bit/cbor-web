#!/bin/bash
# cbor-server — deploy script
# Security: uses env vars instead of hardcoded IPs/hostnames
#
# Required env vars:
#   DEPLOY_TARGET   — hostname or IP to deploy to (e.g. my-server.local)
#   DEPLOY_USER     — SSH user (default: current user)
#   DEPLOY_DIR      — target directory (default: /opt/cbor-server)
#
# Usage:
#   DEPLOY_TARGET=my-server ./deploy.sh

set -euo pipefail

: "${DEPLOY_TARGET:?Set DEPLOY_TARGET env var}"
TARGET="${DEPLOY_TARGET}"
USER="${DEPLOY_USER:-$USER}"
TARGET_DIR="${DEPLOY_DIR:-/opt/cbor-server}"
CBOR_WEB_DIR="$(cd "$(dirname "$0")/../.." && pwd)"

echo "=== Building cbor-server (release) ==="
cd "$CBOR_WEB_DIR/tools/cbor-server"
cargo build --release

echo "=== Stopping service on $TARGET ==="
ssh "${USER}@${TARGET}" "sudo systemctl stop cbor-server 2>/dev/null || true"

echo "=== Deploying binary ==="
scp target/release/cbor-server "${USER}@${TARGET}:${TARGET_DIR}/"

echo "=== Deploying service file ==="
ssh "${USER}@${TARGET}" "sudo mkdir -p /etc/systemd/system"
scp cbor-server.service "${USER}@${TARGET}:/tmp/cbor-server.service"
ssh "${USER}@${TARGET}" "sudo mv /tmp/cbor-server.service /etc/systemd/system/ && sudo systemctl daemon-reload"

echo "=== Syncing data directory ==="
if [ -d data/.well-known ]; then
    rsync -avz --delete data/ "${USER}@${TARGET}:${TARGET_DIR}/data/"
else
    echo "  (no local data/ — using existing data on $TARGET)"
fi

echo "=== Starting service ==="
ssh "${USER}@${TARGET}" "sudo systemctl enable --now cbor-server"

echo "=== Done ==="
sleep 1
ssh "${USER}@${TARGET}" "sudo systemctl status cbor-server --no-pager" || true
echo ""
echo "Server: http://${TARGET}:3001"
echo "Test:   curl -s http://localhost:3001/.well-known/cbor-web"
