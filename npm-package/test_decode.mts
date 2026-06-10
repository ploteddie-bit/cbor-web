// Test de durcissement du décodeur (C1) + correctifs I7/I9. Lancer : npx -y tsx test_decode.mts
import { decodeCBOR, encodePagePath, decodePagePath } from "./cborweb.ts";

let fails = 0;
function ok(cond: boolean, msg: string) {
  console.log((cond ? "  OK   " : "  FAIL ") + msg);
  if (!cond) fails++;
}
function mustThrow(name: string, bytes: number[] | Uint8Array) {
  try {
    decodeCBOR(bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes));
    ok(false, name + " — aurait dû lever une erreur");
  } catch (e) {
    const n = e instanceof Error ? e.name : typeof e;
    ok(n === "CBorDecodeError", name + " -> " + n);
  }
}

console.log("— CBOR valides (doivent décoder) —");
ok(decodeCBOR(new Uint8Array([0x01])) === 1, "uint 1");
{
  const r = decodeCBOR(new Uint8Array([0x83, 0x01, 0x02, 0x03])); // [1,2,3]
  ok(Array.isArray(r) && (r as number[]).length === 3 && (r as number[])[2] === 3, "array [1,2,3]");
}
ok(decodeCBOR(new Uint8Array([0x62, 0x68, 0x69])) === "hi", 'tstr "hi"');

console.log("— Payloads hostiles (doivent lever, SANS boucler) —");
mustThrow("array indefinite 0x9F (ex-boucle infinie)", [0x9f]);
mustThrow("map indefinite 0xBF (ex-boucle infinie)", [0xbf]);
mustThrow("array longueur enorme (0x9b ff*8)", [0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
mustThrow("array tronque 0x82 0x01", [0x82, 0x01]);
mustThrow("tstr longueur > buffer (0x65 'hi')", [0x65, 0x68, 0x69]);
mustThrow("buffer vide (EOF)", []);
mustThrow("UTF-8 invalide (0x61 0xff)", [0x61, 0xff]);
mustThrow("octets residuels (0x01 0x02)", [0x01, 0x02]);
mustThrow("input > 5 Mo", new Uint8Array(5 * 1024 * 1024 + 1));

console.log("— major 7 : simple-values non gérées doivent lever CBorDecodeError —");
function mustThrowMsg(name: string, bytes: number[], expectedMsg: string) {
  try {
    decodeCBOR(new Uint8Array(bytes));
    ok(false, name + " — aurait dû lever une erreur");
  } catch (e) {
    const isCbor = e instanceof Error && e.name === "CBorDecodeError";
    const msg = e instanceof Error ? e.message : String(e);
    ok(isCbor && msg === expectedMsg, `${name} -> ${isCbor ? msg : (e instanceof Error ? e.name : typeof e)}`);
  }
}
// 0xF7 = major 7 / info 23 = 'undefined'
mustThrowMsg("undefined (0xF7)", [0xf7], "undefined not supported");
// 0xF0 = major 7 / info 16 = simple value 16 (info < 20)
mustThrowMsg("simple value 16 (0xF0)", [0xf0], "unsupported simple value: 16");
// 0xF8 0xFF = major 7 / info 24 (simple value sur 1 octet suivant) = 255
mustThrowMsg("simple value 255 (0xF8 0xFF)", [0xf8, 0xff], "unsupported simple value: 255");

console.log("— I7 : decodePagePath inverse exact de encodePagePath (pas de sur-décodage) —");
{
  // Bijection : encodePagePath(p) puis decodePagePath(...) doit redonner p.
  const roundtrip = ["/", "/about", "/a_b/c", "/x_y_z", "/under_score/path", "/déjà"];
  for (const p of roundtrip) {
    const back = decodePagePath(encodePagePath(p));
    ok(back === p, `round-trip ${JSON.stringify(p)} -> ${JSON.stringify(encodePagePath(p))} -> ${JSON.stringify(back)}`);
  }
  // '%' isolé : decodeURIComponent levait URIError ; le correctif doit le laisser passer tel quel.
  ok(decodePagePath("100%bad") === "/100%bad", "'%' isolé ne crashe pas (/100%bad)");
  // '%20' ne doit PAS être sur-décodé en espace (l'encodeur ne le produit jamais).
  ok(decodePagePath("foo%20bar") === "/foo%20bar", "'%20' n'est pas sur-décodé en espace");
  // Seul %5F / %5f (échappement de '_') est ré-interprété en '_'.
  ok(decodePagePath("a%5Fb") === "/a_b", "%5F -> '_'");
  ok(decodePagePath("a%5fb") === "/a_b", "%5f (minuscule) -> '_'");
}

console.log("— I9 : fidélité des entiers 64 bits > 2^53 (bigint, pas de troncature) —");
{
  // uint 0x1B 00 20 00 00 00 00 00 01 = 2^53 + 1 = 9007199254740993 (> MAX_SAFE_INTEGER).
  const big = decodeCBOR(new Uint8Array([0x1b, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]));
  ok(typeof big === "bigint" && big === 9007199254740993n, "uint 2^53+1 -> bigint exact (pas 9007199254740992)");

  // uint u64::MAX = 0x1B FF*8 = 18446744073709551615 : doit rester exact en bigint.
  const max = decodeCBOR(new Uint8Array([0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]));
  ok(typeof max === "bigint" && max === 18446744073709551615n, "uint u64::MAX -> bigint exact");

  // nint 0x3B FF*8 = -1 - (2^64-1) = -18446744073709551616 : bigint négatif exact.
  const nbig = decodeCBOR(new Uint8Array([0x3b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]));
  ok(typeof nbig === "bigint" && nbig === -18446744073709551616n, "nint i64-min-1 -> bigint exact");

  // Un uint 64 bits qui TIENT dans le domaine sûr reste un number (pas de régression de type).
  const small = decodeCBOR(new Uint8Array([0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2a]));
  ok(typeof small === "number" && small === 42, "uint 42 sur 8 octets reste un number");

  // Les petites valeurs courantes restent des number (uint 1 déjà testé plus haut).
  ok(typeof decodeCBOR(new Uint8Array([0x18, 0xff])) === "number", "uint 255 (1 octet) reste un number");
}

console.log("— Relecture adversariale : récursion bornée, bijection des chemins, clés —");
// Récursion non bornée -> doit lever 'nesting too deep' (CBorDecodeError), PAS un RangeError.
mustThrowMsg("array profond (0x81 x1000)", Array(1000).fill(0x81), "nesting too deep");
mustThrowMsg("tag profond (0xC0 x1000)", Array(1000).fill(0xc0), "nesting too deep");
// Bijection encode/decodePagePath même si le path contient déjà '%', '%5F', '%25'.
for (const p of ["/a%5Fb", "/100%bad", "/%25", "/x_y", "/a%5Fb_c/d"]) {
  ok(decodePagePath(encodePagePath(p)) === p, `bijection path ${JSON.stringify(p)}`);
}
// Clé de map dupliquée -> rejet (profil déterministe, RFC 8949 §4.2.2).
mustThrowMsg("clé de map dupliquée (a2 01 01 01 02)", [0xa2, 0x01, 0x01, 0x01, 0x02], "duplicate map key: 1");
// Clé de map entière 64 bits > 2^53 : préservée en String (pas tronquée par Number).
{
  const m = decodeCBOR(new Uint8Array([0xa1, 0x1b, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01])) as Record<string, unknown>;
  ok(Object.keys(m)[0] === "9007199254740993", "clé de map 2^53+1 -> String exacte (pas tronquée)");
}

console.log(fails === 0 ? "\nTOUS LES TESTS PASSENT (C1 + I7 + I9 + relecture)" : `\n${fails} ECHEC(S)`);
process.exit(fails === 0 ? 0 : 1);
