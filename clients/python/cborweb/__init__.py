# SPDX-License-Identifier: MIT
# License: MIT — Copyright (c) 2026 ExploDev / Deltopide SL

"""
CBOR-Web Python Client — AI-Agent Ready

Single-import client for discovering, fetching, and consuming
CBOR-Web binary content from any domain.
"""

import hashlib
import struct
import json
from typing import Optional
from urllib.request import Request, urlopen
from urllib.error import HTTPError, URLError


class CBORWebClient:
    """CBOR-Web consumer client for AI agents and tools."""

    WELL_KNOWN = "/.well-known/cbor-web"

    def __init__(self, domain: str, token: str = "", timeout: int = 10):
        self.domain = domain.rstrip("/")
        self.base = f"https://{self.domain}"
        self.token = token
        self.timeout = timeout

    # ── Discovery ──

    def manifest(self) -> dict:
        """Fetch and parse the CBOR-Web manifest."""
        return self._cbor_get(self.WELL_KNOWN)

    def page(self, path: str) -> dict:
        """Fetch a single page by path (e.g. '/about')."""
        filename = self._path_to_filename(path)
        return self._cbor_get(f"{self.WELL_KNOWN}/pages/{filename}")

    def bundle(self) -> dict:
        """Fetch the full site bundle (all pages in one request)."""
        return self._cbor_get(f"{self.WELL_KNOWN}/bundle")

    def search(self, query: str) -> list[dict]:
        """Search across all pages for a term (requires crawl via bundle)."""
        data = self.bundle()
        results = []
        q = query.lower()
        pages = data.get("pages", data)
        if isinstance(pages, dict):
            for path, page in pages.items():
                title = page.get("title", "") or page.get("3", {}).get("title", "")
                blocks = page.get("content") or page.get("4") or page.get("blocks") or []
                if isinstance(blocks, dict):
                    blocks = blocks.get("content") or blocks.get("4") or blocks.get("blocks") or []
                matching = [b for b in blocks if self._block_matches(b, q)]
                if matching or q in title.lower():
                    results.append({"path": path, "title": title, "matches": len(matching), "blocks": matching[:5]})
        return results

    # ── Doléance ──

    def send_feedback(self, page_path: str, signals: list[dict]) -> bool:
        """
        Send Doléance feedback to the publisher.

        signals: [{"signal": "missing_data", "details": "...", "block_type": "p"}, ...]
        """
        payload = {
            "page_path": page_path,
            "signals": signals,
        }
        body = self._json_to_cbor(payload)
        try:
            req = Request(
                f"{self.base}{self.WELL_KNOWN}/doleance",
                data=body,
                headers={"Content-Type": "application/cbor"},
                method="POST",
            )
            resp = urlopen(req, timeout=self.timeout)
            return resp.status == 202
        except Exception:
            return False

    # ── Diff ──

    def diff(self, base_hash: str) -> dict:
        """Get incremental diff since a previous manifest hash."""
        data = self._cbor_get(f"{self.WELL_KNOWN}/diff?base={base_hash}")
        return data

    # ── Reading (simplified protocol) ──

    def read(self, path: str = "/") -> dict:
        """Read a page using the simplified v3.0 protocol (index.cbor)."""
        if path == "/":
            return self._cbor_get("/index.cbor")
        filename = self._path_to_filename(path)
        return self._cbor_get(f"{self.WELL_KNOWN}/pages/{filename}")

    # ── Internal ──

    def _cbor_get(self, path: str) -> dict:
        url = f"{self.base}{path}"
        headers = {"Accept": "application/cbor"}
        if self.token:
            headers["X-CBOR-Web-Wallet"] = self.token
        try:
            req = Request(url, headers=headers)
            resp = urlopen(req, timeout=self.timeout)
            data = resp.read()
            return self._parse_cbor(data)
        except HTTPError as e:
            raise CBORWebError(f"HTTP {e.code}: {e.reason}", e.code) from e
        except URLError as e:
            raise CBORWebError(f"Connection failed: {e.reason}") from e

    def _parse_cbor(self, data: bytes) -> dict:
        """Minimal CBOR parser for the subset used by CBOR-Web (RFC 8949)."""
        if len(data) < 3:
            raise CBORWebError("CBOR data too short")

        # Check self-described CBOR tag (55799 = 0xD9D9F7)
        offset = 0
        if data[:3] == b"\xd9\xd9\xf7":
            offset = 3

        return self._decode_item(data, offset)[0]

    def _decode_item(self, data: bytes, offset: int):
        """Decode one CBOR item, returning (value, new_offset)."""
        if offset >= len(data):
            raise CBORWebError("Unexpected end of CBOR data")
        initial = data[offset]
        major = initial >> 5
        info = initial & 0x1F
        offset += 1

        def get_arg():
            nonlocal offset
            if info < 24:
                return info
            elif info == 24:
                v = data[offset]; offset += 1; return v
            elif info == 25:
                v = struct.unpack(">H", data[offset:offset+2])[0]; offset += 2; return v
            elif info == 26:
                v = struct.unpack(">I", data[offset:offset+4])[0]; offset += 4; return v
            elif info == 27:
                v = struct.unpack(">Q", data[offset:offset+8])[0]; offset += 8; return v
            return 0

        if major == 0:  # uint
            return get_arg(), offset
        elif major == 1:  # nint
            return -1 - get_arg(), offset
        elif major == 2:  # bstr
            length = get_arg()
            val = data[offset:offset+length]; offset += length
            return val, offset
        elif major == 3:  # tstr
            length = get_arg()
            val = data[offset:offset+length].decode("utf-8"); offset += length
            return val, offset
        elif major == 4:  # array
            length = get_arg()
            arr = []
            for _ in range(length):
                item, offset = self._decode_item(data, offset)
                arr.append(item)
            return arr, offset
        elif major == 5:  # map
            length = get_arg()
            obj = {}
            for _ in range(length):
                k, offset = self._decode_item(data, offset)
                v, offset = self._decode_item(data, offset)
                obj[str(k)] = v
            return obj, offset
        elif major == 6:  # tag
            tag = get_arg()
            inner, offset = self._decode_item(data, offset)
            if tag == 1:  # epoch timestamp
                from datetime import datetime, timezone
                return datetime.fromtimestamp(inner, tz=timezone.utc).isoformat(), offset
            return {f"@tag{tag}": inner}, offset
        elif major == 7:  # simple/float
            if info == 20: return False, offset
            elif info == 21: return True, offset
            elif info == 22: return None, offset
            elif info == 25:
                v = struct.unpack(">e", data[offset:offset+2])[0]; offset += 2; return v, offset
            elif info == 26:
                v = struct.unpack(">f", data[offset:offset+4])[0]; offset += 4; return v, offset
            elif info == 27:
                v = struct.unpack(">d", data[offset:offset+8])[0]; offset += 8; return v, offset
            return None, offset
        return None, offset

    def _path_to_filename(self, path: str) -> str:
        """Convert a page path to CBOR-Web filename (§6.1)."""
        if path == "/":
            return "_index.cbor"
        escaped = path.replace("_", "%5F")
        without_slash = escaped.lstrip("/")
        return without_slash.replace("/", "_") + ".cbor"

    def _block_matches(self, block: dict, query: str) -> bool:
        """Check if a content block matches a search query."""
        for field in ("v", "text", "alt", "href", "src", "attr"):
            val = str(block.get(field, "")).lower()
            if query in val:
                return True
        if isinstance(block.get("v"), list):
            for item in block["v"]:
                if query in str(item).lower():
                    return True
        return False

    def _json_to_cbor(self, obj) -> bytes:
        """Encode a simple JSON-like object to CBOR bytes."""
        buf = bytearray()
        self._encode_cbor(obj, buf)
        return bytes(buf)

    def _encode_cbor(self, obj, buf: bytearray):
        """Minimal CBOR encoder."""
        if isinstance(obj, str):
            encoded = obj.encode("utf-8")
            self._write_header(3, len(encoded), buf)
            buf.extend(encoded)
        elif isinstance(obj, int):
            if obj >= 0:
                self._write_header(0, obj, buf)
            else:
                self._write_header(1, -1 - obj, buf)
        elif isinstance(obj, bool):
            buf.append(0xF5 if obj else 0xF4)
        elif obj is None:
            buf.append(0xF6)
        elif isinstance(obj, list):
            self._write_header(4, len(obj), buf)
            for item in obj:
                self._encode_cbor(item, buf)
        elif isinstance(obj, dict):
            self._write_header(5, len(obj), buf)
            for k, v in obj.items():
                self._encode_cbor(k, buf)
                self._encode_cbor(v, buf)
        elif isinstance(obj, bytes):
            self._write_header(2, len(obj), buf)
            buf.extend(obj)

    def _write_header(self, major: int, value: int, buf: bytearray):
        if value < 24:
            buf.append((major << 5) | value)
        elif value < 256:
            buf.append((major << 5) | 24)
            buf.append(value)
        elif value < 65536:
            buf.append((major << 5) | 25)
            buf.extend(struct.pack(">H", value))
        elif value < 4294967296:
            buf.append((major << 5) | 26)
            buf.extend(struct.pack(">I", value))
        else:
            buf.append((major << 5) | 27)
            buf.extend(struct.pack(">Q", value))


class CBORWebError(Exception):
    def __init__(self, message: str, code: int = 0):
        super().__init__(message)
        self.code = code


# ── CLI demo ──
if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: python -m cborweb <domain> [path|search <query>]")
        print("  python -m cborweb deltopide.com")
        print("  python -m cborweb verdetao.com /about")
        print("  python -m cborweb cbor-web.com search CBD")
        sys.exit(1)

    domain = sys.argv[1]
    client = CBORWebClient(domain)

    if len(sys.argv) >= 3 and sys.argv[2] == "search":
        query = sys.argv[3] if len(sys.argv) > 3 else ""
        results = client.search(query)
        print(json.dumps(results, indent=2, ensure_ascii=False))
    elif len(sys.argv) >= 3:
        page = client.page(sys.argv[2])
        print(json.dumps(page, indent=2, ensure_ascii=False))
    else:
        manifest = client.manifest()
        print(json.dumps(manifest, indent=2, ensure_ascii=False))
