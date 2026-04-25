// CBOR-Web TypeScript Client SDK — v2.1
// Zero dependencies. Uses fetch() and a minimal CBOR decoder.
// Protocol: GET {base}/.well-known/cbor-web (manifest),
//           GET {base}/.well-known/cbor-web/pages/{file}.cbor (page),
//           GET {base}/.well-known/cbor-web/bundle (bundle)

type CBORValue = number | string | boolean | null | Uint8Array | CBORValue[] | { [key: string]: CBORValue } | { [key: number]: CBORValue };

class CBorDecodeError extends Error {
  constructor(msg: string) { super(msg); this.name = "CBorDecodeError"; }
}

function decodeCBOR(data: Uint8Array): CBORValue {
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let offset = 0;

  function readArg(ib: number): number {
    const info = ib & 0x1F;
    if (info < 24) return info;
    if (info === 24) { offset++; return data[offset - 1]; }
    if (info === 25) { const v = dv.getUint16(offset); offset += 2; return v; }
    if (info === 26) { const v = dv.getUint32(offset); offset += 4; return v; }
    if (info === 27) { const v = Number(dv.getBigUint64(offset)); offset += 8; return v; }
    throw new CBorDecodeError(`unsupported argument encoding: ${info}`);
  }

  function decode(): CBORValue {
    const ib = data[offset++];
    const major = (ib >> 5) & 0x07;
    const info = ib & 0x1F;

    if (major === 0) { return readArg(ib); }                    // uint
    if (major === 1) { return -1 - readArg(ib); }               // nint
    if (major === 2) {                                          // bstr
      const len = readArg(ib);
      const val = data.slice(offset, offset + len);
      offset += len;
      return val;
    }
    if (major === 3) {                                          // tstr
      const len = readArg(ib);
      const val = new TextDecoder().decode(data.slice(offset, offset + len));
      offset += len;
      return val;
    }
    if (major === 4) {                                          // array
      const items: CBORValue[] = [];
      if (info === 31) {
        while (data[offset] !== 0xFF) items.push(decode());
        offset++;
      } else {
        const len = readArg(ib);
        for (let i = 0; i < len; i++) items.push(decode());
      }
      return items;
    }
    if (major === 5) {                                          // map
      const map: Record<number | string, CBORValue> = {};
      if (info === 31) {
        while (data[offset] !== 0xFF) {
          const k = decode(); const v = decode();
          map[typeof k === "string" ? k : Number(k)] = v;
        }
        offset++;
      } else {
        const len = readArg(ib);
        for (let i = 0; i < len; i++) {
          const k = decode(); const v = decode();
          map[typeof k === "string" ? k : Number(k)] = v;
        }
      }
      return map;
    }
    if (major === 6) {                                          // tag
      const tag = readArg(ib);
      const inner = decode();
      if (tag === 55799) return inner;  // self-described CBOR-Web: unwrap
      return { _tag: tag, _value: inner };
    }
    if (major === 7) {
      if (info === 20) return false;
      if (info === 21) return true;
      if (info === 22) return null;
      if (info === 25) {
        const bits = dv.getUint16(offset); offset += 2;
        const sign = (bits & 0x8000) ? -1 : 1;
        const exp = (bits >> 10) & 0x1F;
        const mant = bits & 0x3FF;
        if (exp === 0) return sign * mant / 1024 * 2 ** -14;
        if (exp === 31) return mant ? NaN : sign * Infinity;
        return sign * (1 + mant / 1024) * 2 ** (exp - 15);
      }
      if (info === 26) { const v = dv.getFloat32(offset); offset += 4; return v; }
      if (info === 27) { const v = dv.getFloat64(offset); offset += 8; return v; }
    }
    throw new CBorDecodeError(`unsupported major type: ${major}/${info}`);
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
