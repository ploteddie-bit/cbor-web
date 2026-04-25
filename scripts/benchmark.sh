# SPDX-License-Identifier: MIT
# License: MIT — Copyright (c) 2026 ExploDev

#!/usr/bin/env bash
# CBOR-Web Compression Benchmark Suite
# Measures HTML→CBOR compression ratios on test fixtures
# Usage: ./scripts/benchmark.sh [--json]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TEXT2CBOR="$PROJECT_DIR/tools/text2cbor/target/debug/text2cbor"
FIXTURES="$PROJECT_DIR/test-fixtures"
TMPDIR=$(mktemp -d)
JSON_OUTPUT="${1:-}"

# Build if needed
if [ ! -f "$TEXT2CBOR" ]; then
  echo "Building text2cbor..." >&2
  (cd "$PROJECT_DIR/tools/text2cbor" && cargo build 2>/dev/null)
fi

# Token estimation: ~4 chars per token for English, ~3.5 for mixed content
TOKEN_RATIO=3.8
# LLM cost per 1K tokens (Claude Sonnet ~$3/1M tokens)
COST_PER_1K_TOKENS=0.003

benchmark_site() {
  local name="$1"
  local input_dir="$2"
  local domain="$3"
  local lang="${4:-en}"
  local spec_version="${5:-2.1}"

  local out_dir="$TMPDIR/$name"
  $TEXT2CBOR generate \
    -i "$input_dir" -o "$out_dir" \
    -d "$domain" --name "$name" \
    --default-lang "$lang" \
    --spec-version "$spec_version" >/dev/null 2>/dev/null

  # Count HTML sizes
  local html_total=0 html_pages=0
  while IFS= read -r -d '' f; do
    html_total=$((html_total + $(wc -c < "$f")))
    html_pages=$((html_pages + 1))
  done < <(find "$input_dir" -name "*.html" -print0 2>/dev/null)

  # Count CBOR sizes
  local cbor_total=0 cbor_pages=0 cbor_manifest=0 cbor_bundle=0
  local pages_dir="$out_dir/.well-known/cbor-web/pages"
  if [ -d "$pages_dir" ]; then
    while IFS= read -r -d '' f; do
      cbor_total=$((cbor_total + $(wc -c < "$f")))
      cbor_pages=$((cbor_pages + 1))
    done < <(find "$pages_dir" -name "*.cbor" -print0 2>/dev/null)
  fi
  if [ -f "$out_dir/.well-known/cbor-web/manifest.cbor" ]; then
    cbor_manifest=$(wc -c < "$out_dir/.well-known/cbor-web/manifest.cbor")
    cbor_total=$((cbor_total + cbor_manifest))
  fi
  if [ -f "$out_dir/.well-known/cbor-web/bundle.cbor" ]; then
    cbor_bundle=$(wc -c < "$out_dir/.well-known/cbor-web/bundle.cbor")
  fi

  local ratio=0
  if [ "$cbor_total" -gt 0 ]; then
    ratio=$(echo "scale=1; $html_total / $cbor_total" | bc 2>/dev/null | sed 's/^\./0./' || echo "N/A")
  fi

  local html_tokens=$(echo "scale=0; $html_total / $TOKEN_RATIO" | bc 2>/dev/null || echo "0")
  local cbor_tokens=$(echo "scale=0; $cbor_total / $TOKEN_RATIO" | bc 2>/dev/null || echo "0")
  # CBOR content is ~95% signal; HTML is ~7%
  local html_signal_tokens=$(echo "scale=0; $html_tokens * 7 / 100" | bc 2>/dev/null || echo "0")
  local cbor_signal_tokens=$(echo "scale=0; $cbor_tokens * 95 / 100" | bc 2>/dev/null || echo "0")
  local html_cost=$(echo "scale=4; $html_tokens * $COST_PER_1K_TOKENS / 1000" | bc 2>/dev/null | sed 's/^\./0./;s/^-\./-0./' || echo "0")
  local cbor_cost=$(echo "scale=4; $cbor_tokens * $COST_PER_1K_TOKENS / 1000" | bc 2>/dev/null | sed 's/^\./0./;s/^-\./-0./' || echo "0")
  local savings=$(echo "scale=4; $html_cost - $cbor_cost" | bc 2>/dev/null | sed 's/^\./0./;s/^-\./-0./' || echo "0")

  if [ "$JSON_OUTPUT" = "--json" ]; then
    echo "  {"
    echo "    \"name\": \"$name\","
    echo "    \"domain\": \"$domain\","
    echo "    \"pages\": $html_pages,"
    echo "    \"html_bytes\": $html_total,"
    echo "    \"cbor_bytes\": $cbor_total,"
    echo "    \"cbor_manifest_bytes\": $cbor_manifest,"
    echo "    \"cbor_bundle_bytes\": $cbor_bundle,"
    echo "    \"compression_ratio\": \"${ratio}x\","
    echo "    \"html_tokens_est\": $html_tokens,"
    echo "    \"html_signal_tokens\": $html_signal_tokens,"
    echo "    \"cbor_tokens_est\": $cbor_tokens,"
    echo "    \"cbor_signal_tokens\": $cbor_signal_tokens,"
    echo "    \"llm_cost_html\": $html_cost,"
    echo "    \"llm_cost_cbor\": $cbor_cost,"
    echo "    \"llm_cost_savings\": $savings"
    echo "  }"
  else
    printf "%-20s %3s pgs  HTML:%7sB → CBOR:%7sB  %5sx  Tokens:%5s→%4s  Cost:\$%s→\$%s\n" \
      "$name" "$html_pages" "$(printf "%'d" $html_total)" "$(printf "%'d" $cbor_total)" \
      "${ratio}x" "$(printf "%'d" $html_tokens)" "$(printf "%'d" $cbor_tokens)" \
      "$html_cost" "$cbor_cost"
  fi
}

if [ "$JSON_OUTPUT" = "--json" ]; then
  echo "{"
  echo "  \"benchmark_date\": \"$(date -Iseconds)\","
  echo "  \"cborweb_version\": \"2.1.3\","
  echo "  \"results\": ["
  benchmark_site "small-blog"      "$FIXTURES/small-blog"     "blog.example.com"     "en" "2.1"
  echo "  ,"
  benchmark_site "product-page"    "$FIXTURES/product-page"   "shop.example.com"     "fr" "2.1"
  echo "  ,"
  benchmark_site "multilingual"    "$FIXTURES/multilingual"   "multi.example.com"    "fr" "2.1"
  echo "  ]"
  echo "}"
else
  echo "═════════════════════════════════════════════════════════════════════════════════"
  echo "  CBOR-Web Compression Benchmark"
  echo "  $(date)"
  echo "═════════════════════════════════════════════════════════════════════════════════"
  printf "%-20s %-7s %-22s %6s  %-20s %-10s\n" "Site" "Pages" "HTML→CBOR" "Ratio" "Tokens (HTML→CBOR)" "Cost"
  echo "─────────────────────────────────────────────────────────────────────────────────"
  benchmark_site "small-blog"      "$FIXTURES/small-blog"     "blog.example.com"     "en" "2.1"
  benchmark_site "product-page"    "$FIXTURES/product-page"   "shop.example.com"     "fr" "2.1"
  benchmark_site "multilingual"    "$FIXTURES/multilingual"   "multi.example.com"    "fr" "2.1"
  echo "═════════════════════════════════════════════════════════════════════════════════"
fi

rm -rf "$TMPDIR"
