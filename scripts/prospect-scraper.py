#!/usr/bin/env python3
"""
Prospect scraper for text2cbor SaaS.
Finds companies that would benefit from CBOR-Web:
  - Tech media (high page count, need AI agents to read them)
  - Open-source docs (large documentation sites)
  - Web agencies / webmasters (clients for text2cbor)
  - Developer blogs / portfolios

Outputs CSV with: domain, name, type, pages_estimated, contact_email, priority (1-5)
"""
import json, csv, sys, re
from urllib.request import urlopen, Request
from urllib.error import URLError
from urllib.parse import urljoin
import ssl

ssl_ctx = ssl.create_default_context()
ssl_ctx.check_hostname = False
ssl_ctx.verify_mode = ssl.CERT_NONE

HEADERS = {"User-Agent": "Mozilla/5.0 (compatible; CBOR-Web Crawler/1.0)"}
TIMEOUT = 10

def fetch(url):
    try:
        req = Request(url, headers=HEADERS)
        resp = urlopen(req, timeout=TIMEOUT, context=ssl_ctx)
        return resp.read().decode("utf-8", errors="ignore")
    except Exception:
        return ""

def count_pages(html):
    """Estimate page count from sitemap, nav links, or content structure."""
    links = re.findall(r'href=["\']([^"\']+)["\']', html)
    internal = [l for l in links if l.startswith("/") or l.startswith("http") and not "google" in l and not "facebook" in l]
    return min(len(set(internal)), 999)

def find_email(html, domain):
    """Extract contact email from page."""
    emails = re.findall(r'[\w\.-]+@[\w\.-]+\.\w+', html)
    return emails[0] if emails else f"contact@{domain}"

def estimate_tokens_saved(pages):
    """Estimate annual token savings for this site."""
    # ~20K tokens/page for HTML vs ~100 tokens for CBOR
    # ~1000 agent visits per day (avg for mid-size tech site)
    daily = pages * 19000 * 1000
    annual = daily * 365
    return annual

def score(site_type, pages):
    """Priority score 1-5."""
    s = 1
    if site_type in ("tech_blog", "agency"):
        s += 1
    if site_type in ("docs", "ecommerce"):
        s += 2
    if pages > 50:
        s += 1
    if pages > 200:
        s += 1
    return min(s, 5)

# ── Source lists ──

TECH_BLOGS = [
    "https://techcrunch.com", "https://www.theverge.com", "https://arstechnica.com",
    "https://www.zdnet.com", "https://thenextweb.com", "https://venturebeat.com",
    "https://www.infoq.com", "https://dev.to", "https://css-tricks.com",
    "https://www.smashingmagazine.com", "https://alistapart.com",
]

DOCS_SITES = [
    "https://docs.python.org", "https://react.dev", "https://nodejs.org/en/docs",
    "https://docs.docker.com", "https://kubernetes.io/docs", "https://docs.gitlab.com",
    "https://docs.github.com/en", "https://www.rust-lang.org/learn",
    "https://golang.org/doc", "https://ruby-doc.org", "https://www.php.net/docs.php",
]

AGENCIES = [
    "https://www.digitad.fr", "https://www.anthedesign.fr", "https://www.alsacreations.fr",
    "https://www.lunaweb.fr", "https://www.synbioz.com", "https://www.octo.com",
]

ECOMMERCE = [
    "https://www.leboncoin.fr", "https://www.ldlc.com", "https://www.materiel.net",
]

# ── Main ──

def main():
    writer = csv.writer(sys.stdout)
    writer.writerow(["domain", "name", "type", "pages_est", "tokens_saved_year", "contact", "priority"])

    sources = []
    sources.extend([(d, "tech_blog") for d in TECH_BLOGS])
    sources.extend([(d, "docs") for d in DOCS_SITES])
    sources.extend([(d, "agency") for d in AGENCIES])
    sources.extend([(d, "ecommerce") for d in ECOMMERCE])

    seen = set()
    for url, stype in sources:
        try:
            domain = re.sub(r'^https?://(www\.)?', '', url).rstrip('/')
            if domain in seen:
                continue
            seen.add(domain)

            print(f"  Scanning {domain}...", file=sys.stderr)
            html = fetch(url)
            if not html:
                continue

            pages = count_pages(html)
            tokens = estimate_tokens_saved(pages)
            contact = find_email(html, domain)
            name = re.search(r'<title>(.*?)</title>', html, re.I)
            name = name.group(1)[:80] if name else domain
            prio = score(stype, pages)

            writer.writerow([domain, name, stype, pages, tokens, contact, prio])
        except Exception as e:
            print(f"  ERR {domain}: {e}", file=sys.stderr)

if __name__ == "__main__":
    print("Domain, Name, Type, Pages, TokensSaved/Year, Contact, Priority", file=sys.stderr)
    main()
