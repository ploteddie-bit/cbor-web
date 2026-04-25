// SPDX-License-Identifier: MIT
// License: MIT — Copyright (c) 2026 ExploDev / Deltopide SL
// Repository: https://github.com/ploteddie-bit/cbor-web

// CBOR-Web TypeScript Client SDK — v2.1
// Zero dependencies. Uses fetch() and a minimal CBOR decoder.
// Protocol: GET {base}/.well-known/cbor-web (manifest),
//           GET {base}/.well-known/cbor-web/pages/{file}.cbor (page),
//           GET {base}/.well-known/cbor-web/bundle (bundle)

const MAX_DECODE_SIZE = 50 * 1024 * 1024;  // 50 MB
const MAX_DECODE_DEPTH = 50;               // max nesting
const MAX_ARRAY_ITEMS = 100_000;           // max items per array
const MAX_MAP_ENTRIES = 100_000;           // max entries per map

type CBORValue = number | string | boolean | null | Uint8Array | CBORValue[] | { [key: string]: CBORValue } | { [key: number]: CBORValue };

class CBorDecodeError extends Error {
  constructor(msg: string) { super(msg); this.name = "CBorDecodeError"; }
}

function decodeCBOR(data: Uint8Array): CBORValue {
  if (data.length > MAX_DECODE_SIZE) throw new CBorDecodeError(`input too large: ${data.length} > ${MAX_DECODE_SIZE}`);
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 0;
  let depth = 0;

  function readArg(ib: number): number {
    const info = ib & 0x1F;
    if (info < 24) return info;
    if (info === 24) { if (offset >= data.length) throw new CBorDecodeError("unexpected end"); offset++; return data[offset - 1]; }
    if (info === 25) { if (offset + 2 > data.length) throw new CBorDecodeError("unexpected end"); const v = dv.getUint16(offset); offset += 2; return v; }
    if (info === 26) { if (offset + 4 > data.length) throw new CBorDecodeError("unexpected end"); const v = dv.getUint32(offset); offset += 4; return v; }
    if (info === 27) { if (offset + 8 > data.length) throw new CBorDecodeError("unexpected end"); const v = Number(dv.getBigUint64(offset)); offset += 8; return v; }
    throw new CBorDecodeError(`unsupported argument encoding: ${info}`);
  }

  function decode(): CBORValue {
    if (offset >= data.length) throw new CBorDecodeError("unexpected end of data");
    depth++;
    if (depth > MAX_DECODE_DEPTH) throw new CBorDecodeError(`max depth ${MAX_DECODE_DEPTH} exceeded`);
    const ib = data[offset++];
    const major = (ib >> 5) & 0x07;
    const info = ib & 0x1F;

    let result: CBORValue;
    if (major === 0) { result = readArg(ib); }                    // uint
    else if (major === 1) { result = -1 - readArg(ib); }          // nint
    else if (major === 2) {                                       // bstr
      const len = readArg(ib);
      if (offset + len > data.length) throw new CBorDecodeError("bstr length exceeds data");
      result = data.slice(offset, offset + len);
      offset += len;
    }
    else if (major === 3) {                                       // tstr
      const len = readArg(ib);
      if (offset + len > data.length) throw new CBorDecodeError("tstr length exceeds data");
      result = new TextDecoder().decode(data.slice(offset, offset + len));
      offset += len;
    }
    else if (major === 4) {                                       // array
      const items: CBORValue[] = [];
      if (info === 31) {
        while (data[offset] !== 0xFF) { if (items.length >= MAX_ARRAY_ITEMS) throw new CBorDecodeError("max array items exceeded"); items.push(decode()); }
        offset++;
      } else {
        const len = readArg(ib);
        if (len > MAX_ARRAY_ITEMS) throw new CBorDecodeError(`array too large: ${len} > ${MAX_ARRAY_ITEMS}`);
        for (let i = 0; i < len; i++) items.push(decode());
      }
      result = items;
    }
    else if (major === 5) {                                       // map
      const map: Record<number | string, CBORValue> = {};
      if (info === 31) {
        let count = 0;
        while (data[offset] !== 0xFF) {
          if (count++ >= MAX_MAP_ENTRIES) throw new CBorDecodeError("max map entries exceeded");
          const k = decode(); const v = decode();
          map[typeof k === "string" ? k : Number(k)] = v;
        }
        offset++;
      } else {
        const len = readArg(ib);
        if (len > MAX_MAP_ENTRIES) throw new CBorDecodeError(`map too large: ${len} > ${MAX_MAP_ENTRIES}`);
        for (let i = 0; i < len; i++) {
          const k = decode(); const v = decode();
          map[typeof k === "string" ? k : Number(k)] = v;
        }
      }
      result = map;
    }
    else if (major === 6) {                                       // tag
      const tag = readArg(ib);
      const inner = decode();
      if (tag === 55799) result = inner;
      else result = { _tag: tag, _value: inner };
    }
    else if (major === 7) {
      if (info === 20) result = false;
      else if (info === 21) result = true;
      else if (info === 22) result = null;
      else if (info === 25) {
        if (offset + 2 > data.length) throw new CBorDecodeError("unexpected end");
        const bits = dv.getUint16(offset); offset += 2;
        const sign = (bits & 0x8000) ? -1 : 1;
        const exp = (bits >> 10) & 0x1F;
        const mant = bits & 0x3FF;
        if (exp === 0) result = sign * mant / 1024 * 2 ** -14;
        else if (exp === 31) result = mant ? NaN : sign * Infinity;
        else result = sign * (1 + mant / 1024) * 2 ** (exp - 15);
      }
      else if (info === 26) { if (offset + 4 > data.length) throw new CBorDecodeError("unexpected end"); result = dv.getFloat32(offset); offset += 4; }
      else if (info === 27) { if (offset + 8 > data.length) throw new CBorDecodeError("unexpected end"); result = dv.getFloat64(offset); offset += 8; }
      else throw new CBorDecodeError(`unsupported simple: ${info}`);
    }
    else { throw new CBorDecodeError(`unsupported major type: ${major}/${info}`); }
    depth--;
    return result!;
  }

  const result = decode();
  if (offset !== data.length) throw new CBorDecodeError(`trailing bytes: ${data.length - offset}`);
  return result;
}

function encodePagePath(path: string): string {
  if (path === "/") return "_index";
  let s = path.replace(/_/g, "%5F");
  s = s.replace(/^\//, "");
  s = s.replace(/\//g, "_");
  return s;
}

function decodePagePath(filename: string): string {
  if (filename === "_index") return "/";
  let s = filename.replace(/_/g, "/");
  s = decodeURIComponent(s);
  return "/" + s;
}

export class CBORWebClient {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
  }

  async manifest(): Promise<CBORValue> {
    const resp = await fetch(`${this.baseUrl}/.well-known/cbor-web`, {
      headers: { "Accept": "application/cbor" }
    });
    if (!resp.ok) throw new Error(`manifest fetch failed: ${resp.status}`);
    const buf = await resp.arrayBuffer();
    return decodeCBOR(new Uint8Array(buf));
  }

  async page(path: string): Promise<CBORValue> {
    const filename = encodePagePath(path);
    const resp = await fetch(`${this.baseUrl}/.well-known/cbor-web/pages/${filename}.cbor`, {
      headers: { "Accept": "application/cbor" }
    });
    if (!resp.ok) throw new Error(`page fetch failed (${path}): ${resp.status}`);
    const buf = await resp.arrayBuffer();
    return decodeCBOR(new Uint8Array(buf));
  }

  async bundle(): Promise<CBORValue> {
    const resp = await fetch(`${this.baseUrl}/.well-known/cbor-web/bundle`, {
      headers: { "Accept": "application/cbor" }
    });
    if (!resp.ok) throw new Error(`bundle fetch failed: ${resp.status}`);
    const buf = await resp.arrayBuffer();
    return decodeCBOR(new Uint8Array(buf));
  }
}

export { encodePagePath, decodePagePath, decodeCBOR };
