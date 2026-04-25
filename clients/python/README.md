# CBOR-Web Python Client

```bash
pip install cborweb
```

```python
from cborweb import CBORWebClient

client = CBORWebClient("example.com")

# Discover
manifest = client.manifest()
print(f"{manifest['site_name']} — {manifest['pages_count']} pages")

# Fetch a page
page = client.page("/about")
for block in page["blocks"]:
    print(f"[{block['type']}] {block.get('text', '')}")

# Get everything at once
site = client.bundle()

# Search across all pages
results = client.search("contact")

# Send feedback (Doléance)
client.send_feedback("/pricing", [
    {"signal": "missing_data", "details": "No price in EUR", "block_type": "table"}
])
```
