<?php
// CBOR-Web PHP Client SDK — v2.1
// Zero dependencies. Uses curl and a minimal CBOR decoder.
// Protocol: GET {base}/.well-known/cbor-web (manifest),
//           GET {base}/.well-known/cbor-web/pages/{file}.cbor (page),
//           GET {base}/.well-known/cbor-web/bundle (bundle)

class CBorDecodeError extends \RuntimeException {
    public function __construct(string $msg) { parent::__construct("CBorDecodeError: $msg"); }
}

function cbor_decode(string $data, int &$offset = 0): mixed {
    $len = strlen($data);

    function read_arg(string $data, int $info, int &$offset): int {
        if ($info < 24) return $info;
        if ($info === 24) { $offset += 1; return ord($data[$offset - 1]); }
        if ($info === 25) { $v = unpack('n', substr($data, $offset, 2))[1]; $offset += 2; return $v; }
        if ($info === 26) { $v = unpack('N', substr($data, $offset, 4))[1]; $offset += 4; return $v; }
        if ($info === 27) { $v = unpack('J', substr($data, $offset, 8))[1]; $offset += 8; return $v; }
        throw new CBorDecodeError("unsupported argument encoding: $info");
    }

    $ib = ord($data[$offset++]);
    $major = ($ib >> 5) & 0x07;
    $info = $ib & 0x1F;

    if ($major === 0) { return read_arg($data, $info, $offset); }        // uint
    if ($major === 1) { return -1 - read_arg($data, $info, $offset); }   // nint
    if ($major === 2) {                                                   // bstr
        $n = read_arg($data, $info, $offset);
        $val = substr($data, $offset, $n);
        $offset += $n;
        return $val;
    }
    if ($major === 3) {                                                   // tstr
        $n = read_arg($data, $info, $offset);
        $val = substr($data, $offset, $n);
        $offset += $n;
        return $val;
    }
    if ($major === 4) {                                                   // array
        $items = [];
        if ($info === 31) {
            while ($offset < $len && ord($data[$offset]) !== 0xFF) {
                $items[] = cbor_decode($data, $offset);
            }
            $offset++;
        } else {
            $n = read_arg($data, $info, $offset);
            for ($i = 0; $i < $n; $i++) { $items[] = cbor_decode($data, $offset); }
        }
        return $items;
    }
    if ($major === 5) {                                                   // map
        $map = [];
        if ($info === 31) {
            while ($offset < $len && ord($data[$offset]) !== 0xFF) {
                $k = cbor_decode($data, $offset);
                $v = cbor_decode($data, $offset);
                $map[is_int($k) ? $k : (string)$k] = $v;
            }
            $offset++;
        } else {
            $n = read_arg($data, $info, $offset);
            for ($i = 0; $i < $n; $i++) {
                $k = cbor_decode($data, $offset);
                $v = cbor_decode($data, $offset);
                $map[is_int($k) ? $k : (string)$k] = $v;
            }
        }
        return $map;
    }
    if ($major === 6) {                                                   // tag
        $tag = read_arg($data, $info, $offset);
        $inner = cbor_decode($data, $offset);
        if ($tag === 55799) return $inner;  // self-described CBOR-Web: unwrap
        return ['_tag' => $tag, '_value' => $inner];
    }
    if ($major === 7) {
        if ($info === 20) return false;
        if ($info === 21) return true;
        if ($info === 22) return null;
        if ($info === 25) { $bits = unpack('n', substr($data, $offset, 2))[1]; $offset += 2;
            $sign = ($bits & 0x8000) ? -1 : 1;
            $exp = ($bits >> 10) & 0x1F;
            $mant = $bits & 0x3FF;
            if ($exp === 0) return $sign * $mant / 1024 * pow(2, -14);
            if ($exp === 31) return ($mant ? NAN : INF) * $sign;
            return $sign * (1 + $mant / 1024) * pow(2, $exp - 15); }
        if ($info === 26) { $b = substr($data, $offset, 4); $offset += 4;
            $n = unpack('N', $b)[1];
            $s = ($n >> 31) ? -1 : 1; $e = ($n >> 23) & 0xFF; $m = $n & 0x7FFFFF;
            if ($e === 0) return $s * $m / 8388608 * pow(2, -126);
            if ($e === 255) return $m ? NAN : INF * $s;
            return $s * (1 + $m / 8388608) * pow(2, $e - 127); }
        if ($info === 27) { $v = unpack('E', substr($data, $offset, 8))[1]; $offset += 8; return $v; }
    }
    throw new CBorDecodeError("unsupported major type: $major/$info");
}

function cbor_encode_page_path(string $path): string {
    if ($path === '/') return '_index';
    $s = str_replace('_', '%5F', $path);
    $s = ltrim($s, '/');
    $s = str_replace('/', '_', $s);
    return $s;
}

function cbor_decode_page_path(string $filename): string {
    if ($filename === '_index') return '/';
    $s = str_replace('_', '/', $filename);
    $s = rawurldecode($s);
    return '/' . $s;
}

class CborWebClient {
    private string $baseUrl;

    public function __construct(string $baseUrl) {
        $this->baseUrl = rtrim($baseUrl, '/');
    }

    public function manifest(): mixed {
        return $this->fetch('/.well-known/cbor-web');
    }

    public function page(string $path): mixed {
        $filename = cbor_encode_page_path($path);
        return $this->fetch("/.well-known/cbor-web/pages/$filename.cbor");
    }

    public function bundle(): mixed {
        return $this->fetch('/.well-known/cbor-web/bundle');
    }

    private function fetch(string $path): mixed {
        $ch = curl_init($this->baseUrl . $path);
        curl_setopt_array($ch, [
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_HTTPHEADER => ['Accept: application/cbor'],
            CURLOPT_FOLLOWLOCATION => true,
            CURLOPT_TIMEOUT => 30,
        ]);
        $body = curl_exec($ch);
        $status = curl_getinfo($ch, CURLINFO_HTTP_CODE);
        curl_close($ch);
        if ($status !== 200) {
            throw new \RuntimeException("fetch failed ($path): HTTP $status");
        }
        return cbor_decode($body);
    }
}
