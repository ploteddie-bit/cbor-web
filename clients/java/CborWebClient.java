// SPDX-License-Identifier: MIT
// License: MIT — Copyright (c) 2026 ExploDev / Deltopide SL
// Repository: https://github.com/ploteddie-bit/cbor-web

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.io.ByteArrayInputStream;
import java.io.DataInputStream;
import java.io.IOException;
import java.util.*;

public class CborWebClient {
    private final String baseUrl;
    private final HttpClient http;

    public CborWebClient(String baseUrl) {
        this.baseUrl = baseUrl.replaceAll("/$", "");
        this.http = HttpClient.newHttpClient();
    }

    public Map manifest() throws IOException, InterruptedException {
        return fetch("/_manifest.cbor");
    }

    public Map page(String path) throws IOException, InterruptedException {
        return fetch(encodePath(path));
    }

    public Map bundle() throws IOException, InterruptedException {
        return fetch("/_bundle.cbor");
    }

    private Map fetch(String urlPath) throws IOException, InterruptedException {
        HttpRequest req = HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + urlPath))
                .GET().build();
        HttpResponse<byte[]> resp = http.send(req, HttpResponse.BodyHandlers.ofByteArray());
        if (resp.statusCode() != 200)
            throw new IOException("HTTP " + resp.statusCode());
        return (Map) decode(new DataInputStream(new ByteArrayInputStream(resp.body())));
    }

    private String encodePath(String path) {
        if (path == null || path.isEmpty() || "/".equals(path))
            return "/_index.cbor";
        String p = path.startsWith("/") ? path.substring(1) : path;
        return "/" + p.replace("_", "%5F") + ".cbor";
    }

    /* ---- minimal CBOR decoder ---- */

    private Object decode(DataInputStream in) throws IOException {
        int head = in.readUnsignedByte();
        int major = (head >> 5) & 0x07;
        int info = head & 0x1F;
        long arg = readArg(in, info);

        switch (major) {
            case 0: return arg;                         // unsigned integer
            case 1: return -1 - arg;                    // negative integer
            case 2: {                                   // byte string
                byte[] b = new byte[(int) arg];
                in.readFully(b);
                return b;
            }
            case 3: {                                   // text string
                byte[] b = new byte[(int) arg];
                in.readFully(b);
                return new String(b, "UTF-8");
            }
            case 4: {                                   // array
                List<Object> list = new ArrayList<>();
                for (long i = 0; i < arg; i++)
                    list.add(decode(in));
                return list;
            }
            case 5: {                                   // map
                Map<String, Object> map = new LinkedHashMap<>();
                for (long i = 0; i < arg; i++)
                    map.put((String) decode(in), decode(in));
                return map;
            }
            case 6:                                     // tag
                if (arg == 55799) return decode(in);   // unwrap self-described CBOR
                throw new IOException("Unsupported tag: " + arg);
            case 7: return decodeSimple(info, arg, in); // float / simple
            default:
                throw new IOException("Unknown major type: " + major);
        }
    }

    private long readArg(DataInputStream in, int info) throws IOException {
        if (info < 24) return info;
        switch (info) {
            case 24: return in.readUnsignedByte();
            case 25: return in.readUnsignedShort();
            case 26: return Integer.toUnsignedLong(in.readInt());
            case 27: return in.readLong();
            default:
                throw new IOException("Invalid additional info: " + info);
        }
    }

    private Object decodeSimple(int info, long val, DataInputStream in) throws IOException {
        switch (info) {
            case 20: return false;
            case 21: return true;
            case 22: return null;
            case 25: return readF16(in);
            case 26: return in.readFloat();             // f32
            case 27: return in.readDouble();            // f64
            default:
                if (info < 20) return val;              // simple value
                return val;
        }
    }

    private float readF16(DataInputStream in) throws IOException {
        int h = in.readUnsignedShort();
        int sign = (h >> 15) & 1;
        int exp  = (h >> 10) & 0x1F;
        int mant = h & 0x3FF;

        if (exp == 0)
            return (float) ((sign == 0 ? 1 : -1)
                    * Math.pow(2, -14) * mant / 1024.0);
        if (exp == 31)
            return mant == 0
                    ? (sign == 0 ? Float.POSITIVE_INFINITY : Float.NEGATIVE_INFINITY)
                    : Float.NaN;
        return (float) ((sign == 0 ? 1 : -1)
                * Math.pow(2, exp - 15) * (1.0 + mant / 1024.0));
    }
}
