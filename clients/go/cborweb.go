// SPDX-License-Identifier: MIT
// License: MIT — Copyright (c) 2026 ExploDev / Deltopide SL
// Repository: https://github.com/ploteddie-bit/cbor-web

// Package cborweb provides a minimal CBOR-Web client SDK for Go (stdlib only).
// Endpoints: manifest (/.well-known/cbor-web), page (/.well-known/cbor-web/pages/{file}.cbor), bundle (/.well-known/cbor-web/bundle).
package cborweb

import (
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"math"
	"net/http"
	"strings"
)

// CBORWebClient fetches CBOR-Web documents from a site.
type CBORWebClient struct {
	BaseURL string
	Client  *http.Client
}

// NewClient creates a CBOR-Web client for the given base URL.
func NewClient(baseURL string) *CBORWebClient {
	return &CBORWebClient{BaseURL: strings.TrimRight(baseURL, "/"), Client: http.DefaultClient}
}

func (c *CBORWebClient) Manifest() (interface{}, error) { return c.fetch("/.well-known/cbor-web") }
func (c *CBORWebClient) Page(path string) (interface{}, error) {
	return c.fetch("/.well-known/cbor-web/pages/" + EncodePagePath(path) + ".cbor")
}
func (c *CBORWebClient) Bundle() (interface{}, error) { return c.fetch("/.well-known/cbor-web/bundle") }

func (c *CBORWebClient) fetch(path string) (interface{}, error) {
	req, _ := http.NewRequest("GET", c.BaseURL+path, nil)
	req.Header.Set("Accept", "application/cbor")
	resp, err := c.Client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != 200 {
		return nil, fmt.Errorf("fetch %s: HTTP %d", path, resp.StatusCode)
	}
	data, _ := io.ReadAll(resp.Body)
	return DecodeCBOR(data)
}

// EncodePagePath encodes a URL path to a CBOR-Web page filename (§6.1 bijective).
func EncodePagePath(path string) string {
	if path == "/" {
		return "_index"
	}
	s := strings.ReplaceAll(path, "_", "%5F")
	s = strings.TrimPrefix(s, "/")
	return strings.ReplaceAll(s, "/", "_")
}

// DecodeCBOR decodes a CBOR byte sequence into a Go value.
func DecodeCBOR(data []byte) (interface{}, error) {
	r := &cborReader{data: data}
	val, err := r.decode()
	if err != nil {
		return nil, err
	}
	if r.pos != len(data) {
		return nil, fmt.Errorf("trailing %d bytes", len(data)-r.pos)
	}
	return val, nil
}

type cborReader struct {
	data []byte
	pos  int
}

func (r *cborReader) decode() (interface{}, error) {
	if r.pos >= len(r.data) {
		return nil, errors.New("unexpected eof")
	}
	ib := r.data[r.pos]
	r.pos++
	major := ib >> 5
	info := ib & 0x1F

	arg, err := r.readArg(info)
	if err != nil {
		return nil, err
	}

	switch major {
	case 0:
		return uint64(arg), nil
	case 1:
		return int64(-1 - arg), nil
	case 2: // bstr
		val := make([]byte, arg)
		copy(val, r.data[r.pos:r.pos+arg])
		r.pos += arg
		return val, nil
	case 3: // tstr
		val := string(r.data[r.pos : r.pos+arg])
		r.pos += arg
		return val, nil
	case 4: // array
		if info == 31 {
			items := make([]interface{}, 0)
			for r.pos < len(r.data) && r.data[r.pos] != 0xFF {
				item, _ := r.decode()
				items = append(items, item)
			}
			r.pos++
			return items, nil
		}
		items := make([]interface{}, arg)
		for i := 0; i < arg; i++ {
			items[i], _ = r.decode()
		}
		return items, nil
	case 5: // map
		m := make(map[string]interface{})
		if info == 31 {
			for r.pos < len(r.data) && r.data[r.pos] != 0xFF {
				k, v := r.decodePair()
				m[k] = v
			}
			r.pos++
		} else {
			for i := 0; i < arg; i++ {
				k, v := r.decodePair()
				m[k] = v
			}
		}
		return m, nil
	case 6: // tag
		inner, _ := r.decode()
		if arg == 55799 {
			return inner, nil
		}
		return map[string]interface{}{"_tag": uint64(arg), "_value": inner}, nil
	case 7: // simple / float
		switch info {
		case 20:
			return false, nil
		case 21:
			return true, nil
		case 22:
			return nil, nil
		case 25:
			r.pos += 2
			return float64(0), nil
		case 26:
			bits := binary.BigEndian.Uint32(r.data[r.pos:])
			r.pos += 4
			return float64(math.Float32frombits(bits)), nil
		case 27:
			bits := binary.BigEndian.Uint64(r.data[r.pos:])
			r.pos += 8
			return math.Float64frombits(bits), nil
		}
	}
	return nil, fmt.Errorf("unsupported major %d/%d", major, info)
}

func (r *cborReader) decodePair() (string, interface{}) {
	k, _ := r.decode()
	v, _ := r.decode()
	return fmt.Sprintf("%v", k), v
}

func (r *cborReader) readArg(info byte) (int, error) {
	if info < 24 {
		return int(info), nil
	}
	switch info {
	case 24:
		v := int(r.data[r.pos])
		r.pos++
		return v, nil
	case 25:
		v := int(binary.BigEndian.Uint16(r.data[r.pos:]))
		r.pos += 2
		return v, nil
	case 26:
		v := int(binary.BigEndian.Uint32(r.data[r.pos:]))
		r.pos += 4
		return v, nil
	case 27:
		v := int(binary.BigEndian.Uint64(r.data[r.pos:]))
		r.pos += 8
		return v, nil
	}
	return 0, fmt.Errorf("bad arg %d", info)
}
