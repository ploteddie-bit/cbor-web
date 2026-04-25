#!/usr/bin/env bash
# smart-scraper.sh — Intelligent CBOR-Web prospect scraper
# Usage:
#   ./smart-scraper.sh domains.txt           # file: one domain per line
#   ./smart-scraper.sh domain1.com domain2.com  # inline args
#   cat domains.txt | ./smart-scraper.sh -    # stdin pipe
# Output: CSV to stdout, progress to stderr

set -euo pipefail
TIMEOUT=12
UA="Mozilla/5.0 (compatible; CBOR-Web SmartScraper/1.0)"
CBORCRAWL="${CBORCRAWL:-cbor-crawl}"
EMAIL_RE='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'

# ── Collect domains ──
DOMAINS=()
if [[ $# -eq 1 && "$1" == "-" ]]; then
    while IFS= read -r line; do
        line="$(echo "$line" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
        [[ -n "$line" && "$line" != \#* ]] && DOMAINS+=("$line")
    done
elif [[ $# -eq 1 && -f "$1" ]]; then
    while IFS= read -r line; do
        line="$(echo "$line" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
        [[ -n "$line" && "$line" != \#* ]] && DOMAINS+=("$line")
    done < "$1"
else
    for arg in "$@"; do
        DOMAINS+=("$arg")
    done
fi

[[ ${#DOMAINS[@]} -eq 0 ]] && { echo "No domains provided" >&2; exit 1; }

# ── CSV header ──
echo "domain,pages_found,homepage_size_kb,tokens_estimated,has_cborweb,has_blog,has_shop,contact_email,priority_score"

# ── Process each domain ──
for domain in "${DOMAINS[@]}"; do
    # Strip protocol/path if present
    domain="${domain#https://}"
    domain="${domain#http://}"
    domain="${domain%%/*}"
    domain="${domain##www.}"
    [[ -z "$domain" ]] && continue

    echo "[$(date +%H:%M:%S)] Scanning $domain" >&2

    # ── 1. Check CBOR-Web support via cbor-crawl inspect ──
    has_cborweb="no"
    pages_found=0
    cbor_out="$($CBORCRAWL inspect "https://$domain" 2>/dev/null || true)"
    if echo "$cbor_out" | grep -qE 'pages|manifest|CBOR'; then
        has_cborweb="yes"
        pages_found=$(echo "$cbor_out" | grep -oP 'pages?[:\s]+\K\d+' | head -1 || echo 0)
    fi

    # ── 2. Download homepage ──
    html="$(curl -sL --max-time "$TIMEOUT" -A "$UA" "https://$domain" 2>/dev/null || true)"
    [[ -z "$html" ]] && html="$(curl -sL --max-time "$TIMEOUT" -A "$UA" "http://$domain" 2>/dev/null || true)"
    [[ -z "$html" ]] && { echo "domain,pages_found,homepage_size_kb,tokens_estimated,has_cborweb,has_blog,has_shop,contact_email,priority_score" | tail -1; echo "$domain,0,0,0,no,no,no,,1" >&1; continue; }

    # ── 3. Analyze homepage ──
    # Size
    size_bytes=$(echo -n "$html" | wc -c)
    size_kb=$(awk "BEGIN {printf \"%.1f\", $size_bytes/1024}")

    # Token estimate (~4 chars per token)
    tokens_est=$(awk "BEGIN {printf \"%.0f\", $size_bytes/4}")

    # Link count
    link_count=$(echo "$html" | grep -oi '<a[ >]' | wc -l)

    # Has structured data (Schema.org JSON-LD)
    has_structured="no"
    echo "$html" | grep -qi 'application/ld+json' && has_structured="yes"

    # Has blog
    has_blog="no"
    echo "$html" | grep -qiE '(wp-content|/blog/|/articles/|/news/|wordpress|ghost|medium|substack|rss)' && has_blog="yes"

    # Has shop
    has_shop="no"
    echo "$html" | grep -qiE '(woocommerce|shopify|/shop/|/product/|/cart|add-to-cart|checkout|commerce|prestashop|magento|bigcommerce)' && has_shop="yes"

    # Contact email
    contact_email=""
    contact_email="$(echo "$html" | grep -oP "$EMAIL_RE" | head -1 || true)"
    [[ -z "$contact_email" ]] && contact_email="contact@$domain"

    # ── 4. Pages estimate (from sitemap or link count) ──
    if [[ "$pages_found" -eq 0 ]]; then
        # Try sitemap
        sitemap_url=""
        sitemap_url="$(echo "$html" | grep -oP 'https?://[^"\047<>]+sitemap[^"\047<>]*\.xml' | head -1 || true)"
        if [[ -n "$sitemap_url" ]]; then
            sitemap_content="$(curl -sL --max-time 10 -A "$UA" "$sitemap_url" 2>/dev/null || true)"
            pages_found=$(echo "$sitemap_content" | grep -o '<url>' | wc -l)
        fi
        # Fallback: unique internal links / 3
        [[ "$pages_found" -eq 0 ]] && pages_found=$(awk "BEGIN {printf \"%.0f\", $link_count / 3}")
    fi

    # ── 5. Priority score (1-5) ──
    priority=1
    [[ "$pages_found" -gt 50 ]] && priority=$((priority + 1))
    [[ "$pages_found" -gt 200 ]] && priority=$((priority + 1))
    [[ "$has_blog" == "yes" || "$has_shop" == "yes" ]] && priority=$((priority + 1))
    [[ "$has_cborweb" == "yes" ]] && priority=$((priority + 1))
    [[ "$priority" -gt 5 ]] && priority=5

    # ── 6. Output CSV row ──
    printf '%s,%s,%s,%s,%s,%s,%s,%s,%d\n' \
        "$domain" "$pages_found" "$size_kb" "$tokens_est" \
        "$has_cborweb" "$has_blog" "$has_shop" "$contact_email" "$priority"
done
