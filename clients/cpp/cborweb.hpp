// cborweb.hpp — Minimal C++17 CBOR-Web SDK (header-only, zero deps, POSIX sockets)
#ifndef CBORWEB_HPP
#define CBORWEB_HPP

#include <cstdint>
#include <string>
#include <vector>
#include <map>
#include <variant>
#include <cstring>
#include <stdexcept>
#include <algorithm>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <unistd.h>

namespace cborweb {

struct Timestamp { int64_t sec; int32_t nsec; };

struct Value;

namespace detail {
using ValueBase = std::variant<
    int64_t, double, bool, std::string,
    std::vector<uint8_t>, std::vector<Value>,
    std::map<std::string, Value>, std::nullptr_t, Timestamp>;
}

struct Value : detail::ValueBase {
    using detail::ValueBase::ValueBase;
    using detail::ValueBase::operator=;
};

std::string path_encode(const std::string& path);
Value decode(const std::vector<uint8_t>& data);

class Client {
public:
    Client(const std::string& base_url) : base_(base_url) {
        while (!base_.empty() && base_.back() == '/') base_.pop_back();
    }
    Value manifest();
    Value page(const std::string& path);
    Value bundle();
private:
    std::string base_;
    std::vector<uint8_t> http_get(const std::string& path);
};

} // namespace cborweb

#ifdef CBORWEB_IMPLEMENTATION

namespace cborweb {

// --- Path encoding (RFC 3986 percent-encoding for path segments) ------------

inline std::string path_encode(const std::string& path) {
    static const char H[] = "0123456789ABCDEF";
    std::string o;
    for (unsigned char c : path) {
        if ((c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') ||
            (c >= '0' && c <= '9') || c == '-' || c == '_' ||
            c == '.' || c == '~' || c == '/')
            o += (char)c;
        else { o += '%'; o += H[c >> 4]; o += H[c & 15]; }
    }
    return o;
}

// --- CBOR decoder -----------------------------------------------------------

static void _need(const uint8_t* p, const uint8_t* e, size_t n) {
    if ((size_t)(e - p) < n) throw std::runtime_error("CBOR: unexpected EOF");
}

static uint64_t _uint(const uint8_t*& p, const uint8_t* e, uint8_t ai) {
    if (ai < 24) return ai;
    _need(p, e, ai == 24 ? 1 : ai == 25 ? 2 : ai == 26 ? 4 : 8);
    if (ai == 24) return *p++;
    if (ai == 25) { uint64_t v = ((uint64_t)p[0] << 8) | p[1]; p += 2; return v; }
    if (ai == 26) { uint64_t v = ((uint64_t)p[0] << 24) | ((uint64_t)p[1] << 16) | ((uint64_t)p[2] << 8) | p[3]; p += 4; return v; }
    // ai == 27
    uint64_t v = ((uint64_t)p[0] << 56) | ((uint64_t)p[1] << 48) | ((uint64_t)p[2] << 40) |
                 ((uint64_t)p[3] << 32) | ((uint64_t)p[4] << 24) | ((uint64_t)p[5] << 16) |
                 ((uint64_t)p[6] << 8) | p[7];
    p += 8; return v;
}

static double _half(uint16_t h) {
    int sign = (h >> 15) & 1, exp = (h >> 10) & 31, mant = h & 1023;
    uint64_t bits;
    if (exp == 0)      bits = ((uint64_t)sign << 63);                          // zero / subnormal→zero
    else if (exp == 31) bits = ((uint64_t)sign << 63) | 0x7FF0000000000000ULL | ((uint64_t)(mant != 0) << 51);
    else               bits = ((uint64_t)sign << 63) | ((uint64_t)(exp - 15 + 1023) << 52) | ((uint64_t)mant << 42);
    double d; std::memcpy(&d, &bits, 8); return d;
}

static Value _decode(const uint8_t*& p, const uint8_t* e) {
    if (p >= e) throw std::runtime_error("CBOR: unexpected EOF");
    uint8_t ib = *p++, mt = ib >> 5, ai = ib & 0x1f;

    if (mt == 0) { uint64_t v = _uint(p, e, ai);
        if (v > (uint64_t)INT64_MAX) throw std::runtime_error("CBOR: uint overflow");
        return (int64_t)v; }
    if (mt == 1) { uint64_t v = _uint(p, e, ai);
        if (v > (uint64_t)INT64_MAX) throw std::runtime_error("CBOR: nint overflow");
        return (int64_t)(-1LL - (int64_t)v); }
    if (mt == 2) { auto len = _uint(p, e, ai); _need(p, e, (size_t)len);
        std::vector<uint8_t> v(p, p + len); p += len; return v; }
    if (mt == 3) { auto len = _uint(p, e, ai); _need(p, e, (size_t)len);
        std::string s((const char*)p, (size_t)len); p += len; return s; }
    if (mt == 4) {
        if (ai == 31) { std::vector<Value> arr;
            while (!(p < e && *p == 0xff)) arr.push_back(_decode(p, e));
            if (p < e) p++;
            return arr; }
        auto len = _uint(p, e, ai); std::vector<Value> arr; arr.reserve((size_t)len);
        for (uint64_t i = 0; i < len; i++) arr.push_back(_decode(p, e));
        return arr;
    }
    if (mt == 5) {
        auto decode_map_entry = [&]() -> std::pair<std::string, Value> {
            Value k = _decode(p, e);
            if (!std::holds_alternative<std::string>(k))
                throw std::runtime_error("CBOR: map key must be string");
            return {std::get<std::string>(k), _decode(p, e)};
        };
        if (ai == 31) { std::map<std::string, Value> m;
            while (!(p < e && *p == 0xff)) m.insert(decode_map_entry());
            if (p < e) p++;
            return m; }
        auto len = _uint(p, e, ai); std::map<std::string, Value> m;
        for (uint64_t i = 0; i < len; i++) m.insert(decode_map_entry());
        return m;
    }
    if (mt == 6) { auto tag = _uint(p, e, ai); Value v = _decode(p, e);
        if (tag == 1) {
            if (std::holds_alternative<int64_t>(v))
                return Timestamp{std::get<int64_t>(v), 0};
            if (std::holds_alternative<double>(v)) {
                double d = std::get<double>(v);
                return Timestamp{(int64_t)d, (int32_t)((d - (int64_t)d) * 1e9)};
            }
        }
        /* tag 55799 / unknown: return inner value */ return v; }
    if (mt == 7) {
        if (ai < 20) return (int64_t)ai;
        if (ai == 20) return false;
        if (ai == 21) return true;
        if (ai == 22) return nullptr;
        if (ai == 23) return nullptr;
        if (ai == 25) { _need(p, e, 2); auto v = _half((uint16_t)((p[0] << 8) | p[1])); p += 2; return v; }
        if (ai == 26) { _need(p, e, 4);
            uint32_t u = ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | p[3]; p += 4;
            float f; std::memcpy(&f, &u, 4); return (double)f; }
        if (ai == 27) { _need(p, e, 8);
            uint64_t u = ((uint64_t)p[0] << 56) | ((uint64_t)p[1] << 48) | ((uint64_t)p[2] << 40) |
                         ((uint64_t)p[3] << 32) | ((uint64_t)p[4] << 24) | ((uint64_t)p[5] << 16) |
                         ((uint64_t)p[6] << 8) | p[7]; p += 8;
            double d; std::memcpy(&d, &u, 8); return d; }
        throw std::runtime_error("CBOR: unsupported simple " + std::to_string(ai));
    }
    throw std::runtime_error("CBOR: bad major type " + std::to_string(mt));
}

inline Value decode(const std::vector<uint8_t>& data) {
    const uint8_t* p = data.data();
    return _decode(p, data.data() + data.size());
}

// --- POSIX-socket HTTP GET --------------------------------------------------

inline std::vector<uint8_t> Client::http_get(const std::string& path) {
    std::string host = base_;
    int port = 80;

    // strip scheme
    if (host.compare(0, 7, "http://") == 0) host = host.substr(7);
    else if (host.compare(0, 8, "https://") == 0) throw std::runtime_error("HTTPS not supported");

    // split host:port from /path
    auto slash = host.find('/');
    if (slash != std::string::npos) host = host.substr(0, slash);
    auto colon = host.find(':');
    if (colon != std::string::npos) {
        port = std::stoi(host.substr(colon + 1));
        host = host.substr(0, colon);
    }

    struct hostent* he = gethostbyname(host.c_str());
    if (!he) throw std::runtime_error("DNS lookup failed: " + host);
    struct sockaddr_in addr = {};
    addr.sin_family = AF_INET;
    addr.sin_port = htons((uint16_t)port);
    std::memcpy(&addr.sin_addr, he->h_addr_list[0], (size_t)he->h_length);

    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0) throw std::runtime_error("socket() failed");
    if (connect(sock, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        close(sock); throw std::runtime_error("connect() failed");
    }

    std::string req = "GET " + path + " HTTP/1.0\r\nHost: " + host +
                      "\r\nAccept: application/cbor\r\nConnection: close\r\n\r\n";
    if (send(sock, req.c_str(), req.size(), 0) < 0) {
        close(sock); throw std::runtime_error("send() failed");
    }

    std::vector<uint8_t> resp;
    char buf[4096];
    for (;;) {
        ssize_t n = recv(sock, buf, sizeof(buf), 0);
        if (n <= 0) break;
        resp.insert(resp.end(), buf, buf + n);
    }
    close(sock);

    // extract body after \r\n\r\n
    auto needle = "\r\n\r\n";
    auto it = std::search(resp.begin(), resp.end(), needle, needle + 4);
    if (it == resp.end()) throw std::runtime_error("HTTP: no header terminator");

    // basic status check
    auto sp1 = std::find(resp.begin(), resp.end(), ' ');
    if (sp1 != resp.end()) {
        auto sp2 = std::find(sp1 + 1, resp.end(), ' ');
        if (std::string(sp1 + 1, sp2) != "200")
            throw std::runtime_error("HTTP " + std::string(sp1 + 1, sp2));
    }

    return std::vector<uint8_t>(it + 4, resp.end());
}

inline Value Client::manifest() { return decode(http_get("/.well-known/cbor-web")); }
inline Value Client::page(const std::string& path) { return decode(http_get("/" + path_encode(path))); }
inline Value Client::bundle() { return decode(http_get("/")); }

} // namespace cborweb

#endif // CBORWEB_IMPLEMENTATION
#endif // CBORWEB_HPP
