// CBOR-Web TypeScript Client SDK — v2.1
// Zero dependencies. Uses fetch() and a minimal CBOR decoder.
// Protocol: GET {base}/.well-known/cbor-web (manifest),
//           GET {base}/.well-known/cbor-web/pages/{file}.cbor (page),
//           GET {base}/.well-known/cbor-web/bundle (bundle)

type CBORValue = number | bigint | string | boolean | null | Uint8Array | CBORValue[] | { [key: string]: CBORValue } | { [key: number]: CBORValue };

class CBorDecodeError extends Error {
  constructor(msg: string) { super(msg); this.name = "CBorDecodeError"; }
}

// Limite de taille du profil déterministe CBOR-Web (spec §6.2) : une réponse réseau
// au-delà est refusée d'emblée (défense anti-DoS sur entrée potentiellement hostile).
const MAX_CBOR_INPUT = 5 * 1024 * 1024;
// Profondeur d'imbrication max : borne la récursion de decode() pour qu'un payload très
// profond (ex. 0x81 « array de 1 » ou 0xC0 « tag » répété) lève une CBorDecodeError au lieu
// de déborder la pile (RangeError non typé, non rattrapé par les appelants). Anti-DoS.
const MAX_NESTING_DEPTH = 128;

function decodeCBOR(data: Uint8Array): CBORValue {
  if (data.length > MAX_CBOR_INPUT) {
    throw new CBorDecodeError(`input exceeds ${MAX_CBOR_INPUT}-byte limit`);
  }
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const utf8 = new TextDecoder("utf-8", { fatal: true }); // rejette l'UTF-8 invalide
  let offset = 0;

  // Garde de bornes : toute lecture vérifie d'abord qu'il reste assez d'octets.
  function need(n: number): void {
    if (n < 0 || offset + n > data.length) {
      throw new CBorDecodeError("unexpected end of input");
    }
  }

  // readArg renvoie un `number` borné : utilisé pour les LONGUEURS (majors 2/3/4/5)
  // et les TAGS (major 6), qui servent d'index/compteur dans l'arithmétique de bornes.
  // Un argument 64 bits > 2^53 y est forcément hostile (aucun buffer de cette taille),
  // ce que les gardes need()/len-vs-remaining rejettent déjà ; on en garde donc un number.
  function readArg(ib: number): number {
    const info = ib & 0x1F;
    if (info < 24) return info;
    if (info === 24) { need(1); return data[offset++]; }
    if (info === 25) { need(2); const v = dv.getUint16(offset); offset += 2; return v; }
    if (info === 26) { need(4); const v = dv.getUint32(offset); offset += 4; return v; }
    if (info === 27) { need(8); const v = Number(dv.getBigUint64(offset)); offset += 8; return v; }
    // info 28-30 réservés, 31 = indefinite-length : interdits par le profil déterministe.
    throw new CBorDecodeError(`invalid argument encoding: ${info}`);
  }

  // readArgBig préserve la fidélité 64 bits des VALEURS entières (majors 0/1 : hashes,
  // IDs, timestamps ns). Au-delà de Number.MAX_SAFE_INTEGER on promeut en bigint plutôt
  // que de tronquer silencieusement (ancien Number(getBigUint64)). En deçà, on garde un
  // number pour ne pas casser les usages existants. I9.
  function readArgBig(ib: number): number | bigint {
    const info = ib & 0x1F;
    if (info < 27) return readArg(ib);            // <= 2^32-1 : toujours un number sûr
    if (info === 27) {                            // 64 bits : peut dépasser 2^53
      need(8);
      const v = dv.getBigUint64(offset); offset += 8;
      return v > BigInt(Number.MAX_SAFE_INTEGER) ? v : Number(v);
    }
    throw new CBorDecodeError(`invalid argument encoding: ${info}`);
  }

  function decode(depth: number): CBORValue {
    if (depth > MAX_NESTING_DEPTH) throw new CBorDecodeError("nesting too deep");
    need(1);
    const ib = data[offset++];
    const major = (ib >> 5) & 0x07;
    const info = ib & 0x1F;

    if (major === 0) { return readArgBig(ib); }                 // uint
    if (major === 1) {                                          // nint
      const n = readArgBig(ib);
      // -1 - n, en restant en bigint si la valeur dépasse le domaine sûr.
      return typeof n === "bigint" ? -1n - n : -1 - n;
    }
    if (major === 2) {                                          // bstr
      const len = readArg(ib);
      need(len);
      const val = data.slice(offset, offset + len);
      offset += len;
      return val;
    }
    if (major === 3) {                                          // tstr
      const len = readArg(ib);
      need(len);
      let val: string;
      try {
        val = utf8.decode(data.slice(offset, offset + len));
      } catch {
        throw new CBorDecodeError("invalid UTF-8 in text string");
      }
      offset += len;
      return val;
    }
    if (major === 4) {                                          // array
      if (info === 31) throw new CBorDecodeError("indefinite-length items are forbidden");
      const len = readArg(ib);
      // Un élément fait au moins 1 octet : une longueur > octets restants est forcément hostile.
      if (len > data.length - offset) throw new CBorDecodeError("array length exceeds remaining input");
      const items: CBORValue[] = [];
      for (let i = 0; i < len; i++) items.push(decode(depth + 1));
      return items;
    }
    if (major === 5) {                                          // map
      if (info === 31) throw new CBorDecodeError("indefinite-length items are forbidden");
      const len = readArg(ib);
      if (len > data.length - offset) throw new CBorDecodeError("map length exceeds remaining input");
      const map: Record<number | string, CBORValue> = {};
      for (let i = 0; i < len; i++) {
        const k = decode(depth + 1);
        const v = decode(depth + 1);
        // String(k) (et non Number(k)) : ne pas re-tronquer une clé entière 64 bits promue
        // en bigint (I9) — les clés d'objet JS sont de toute façon des chaînes.
        const key = typeof k === "string" ? k : String(k);
        // Profil déterministe : clés uniques (RFC 8949 §4.2.2). Évite le « key smuggling ».
        if (Object.prototype.hasOwnProperty.call(map, key)) {
          throw new CBorDecodeError(`duplicate map key: ${key}`);
        }
        map[key] = v;
      }
      return map;
    }
    if (major === 6) {                                          // tag
      const tag = readArg(ib);
      const inner = decode(depth + 1);
      if (tag === 55799) return inner;  // self-described CBOR-Web: unwrap
      return { _tag: tag, _value: inner };
    }
    if (major === 7) {
      if (info === 20) return false;
      if (info === 21) return true;
      if (info === 22) return null;
      // major 7 EST géré : ce sont certaines simple-values qui ne le sont pas.
      // On lève un message exact plutôt que de tomber sur le message générique trompeur.
      if (info === 23) throw new CBorDecodeError("undefined not supported");
      if (info < 20) throw new CBorDecodeError(`unsupported simple value: ${info}`);
      if (info === 24) {
        need(1);
        const v = data[offset++];
        throw new CBorDecodeError(`unsupported simple value: ${v}`);
      }
      if (info === 25) {
        need(2);
        const bits = dv.getUint16(offset); offset += 2;
        const sign = (bits & 0x8000) ? -1 : 1;
        const exp = (bits >> 10) & 0x1F;
        const mant = bits & 0x3FF;
        if (exp === 0) return sign * mant / 1024 * 2 ** -14;
        if (exp === 31) return mant ? NaN : sign * Infinity;
        return sign * (1 + mant / 1024) * 2 ** (exp - 15);
      }
      if (info === 26) { need(4); const v = dv.getFloat32(offset); offset += 4; return v; }
      if (info === 27) { need(8); const v = dv.getFloat64(offset); offset += 8; return v; }
    }
    throw new CBorDecodeError(`unsupported major type: ${major}/${info}`);
  }

  const result = decode(0);
  if (offset !== data.length) throw new CBorDecodeError(`trailing bytes: ${data.length - offset}`);
  return result;
}

function encodePagePath(path: string): string {
  if (path === "/") return "_index";
  // Échapper '%' AVANT '_' (ordre crucial) : sinon un '%5F' déjà présent dans le path et un
  // '_' littéral produiraient la même sortie → collision de fichiers (non-bijection).
  let s = path.replace(/%/g, "%25").replace(/_/g, "%5F");
  s = s.replace(/^\//, "");
  s = s.replace(/\//g, "_");
  return s;
}

function decodePagePath(filename: string): string {
  if (filename === "_index") return "/";
  // Inverse exact de encodePagePath : '_' (séparateur de chemin) -> '/', puis
  // '%5F'/'%5f' (échappement d'un '_' littéral) -> '_'. On N'utilise PAS
  // decodeURIComponent, qui sur-décode des séquences non produites par l'encodeur
  // (ex. '%20') et lève URIError sur un '%' isolé (ex. '/100%bad'). I7.
  let s = filename.replace(/_/g, "/");
  s = s.replace(/%5[fF]/g, "_");
  s = s.replace(/%25/g, "%");   // dé-échappe le '%' littéral en DERNIER (inverse de l'encode)
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
