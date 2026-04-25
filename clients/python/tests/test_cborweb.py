# SPDX-License-Identifier: MIT
# License: MIT — Copyright (c) 2026 ExploDev / Deltopide SL

"""Tests for the CBOR-Web Python client — cborweb/__init__.py"""

import struct
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from cborweb import CBORWebClient, CBORWebError


@pytest.fixture
def client():
    return CBORWebClient("example.com")


# ═══════════════════════════════════════════════════════════════════
# 1. CBOR Decoding — _decode_item / _parse_cbor
# ═══════════════════════════════════════════════════════════════════


class TestUnsignedIntegers:
    def test_zero(self, client):
        v, off = client._decode_item(b"\x00", 0)
        assert v == 0
        assert isinstance(v, int)

    def test_one(self, client):
        v, _ = client._decode_item(b"\x01", 0)
        assert v == 1

    def test_ten(self, client):
        v, _ = client._decode_item(b"\x0a", 0)
        assert v == 10

    def test_23_direct(self, client):
        v, _ = client._decode_item(b"\x17", 0)
        assert v == 23

    def test_24_one_byte_arg(self, client):
        v, _ = client._decode_item(b"\x18\x18", 0)
        assert v == 24

    def test_100_one_byte_arg(self, client):
        v, _ = client._decode_item(b"\x18\x64", 0)
        assert v == 100

    def test_300_two_byte_arg(self, client):
        v, _ = client._decode_item(b"\x19\x01\x2c", 0)
        assert v == 300

    def test_50000_two_byte_arg(self, client):
        v, _ = client._decode_item(b"\x19\xc3\x50", 0)
        assert v == 50000

    def test_1000000_four_byte_arg(self, client):
        v, _ = client._decode_item(b"\x1a\x00\x0f\x42\x40", 0)
        assert v == 1000000

    def test_large_eight_byte_arg(self, client):
        data = b"\x1b\x00\x00\x00\x02\x00\x00\x00\x00"  # 8589934592
        v, _ = client._decode_item(data, 0)
        assert v == 8589934592


class TestNegativeIntegers:
    def test_minus_one(self, client):
        v, _ = client._decode_item(b"\x20", 0)
        assert v == -1

    def test_minus_ten(self, client):
        v, _ = client._decode_item(b"\x29", 0)
        assert v == -10

    def test_minus_24_direct(self, client):
        v, _ = client._decode_item(b"\x37", 0)
        assert v == -24

    def test_minus_100_one_byte(self, client):
        v, _ = client._decode_item(b"\x38\x63", 0)
        assert v == -100

    def test_minus_400_two_byte(self, client):
        v, _ = client._decode_item(b"\x39\x01\x8f", 0)
        assert v == -400

    def test_minus_50000_two_byte(self, client):
        v, _ = client._decode_item(b"\x39\xc3\x4f", 0)
        assert v == -50000

    def test_minus_1000000_four_byte(self, client):
        data = b"\x3a" + struct.pack(">I", 999999)
        v, _ = client._decode_item(data, 0)
        assert v == -1000000


class TestTextStrings:
    def test_empty(self, client):
        v, _ = client._decode_item(b"\x60", 0)
        assert v == ""

    def test_single_char(self, client):
        v, _ = client._decode_item(b"\x61\x61", 0)
        assert v == "a"

    def test_hello(self, client):
        v, _ = client._decode_item(b"\x65hello", 0)
        assert v == "hello"

    def test_unicode_2_bytes(self, client):
        v, _ = client._decode_item(b"\x65" + "café".encode("utf-8"), 0)
        assert v == "café"

    def test_unicode_emoji(self, client):
        v, _ = client._decode_item(b"\x67" + "hi!".encode("utf-8"), 0)
        assert v == "hi!"

    def test_unicode_multibyte(self, client):
        text = "Café résumé naïve"
        data = bytes([0x60 + len(text.encode("utf-8"))]) + text.encode("utf-8")
        v, _ = client._decode_item(data, 0)
        assert v == text

    def test_string_offsets(self, client):
        data = b"\x65hello\x61A"
        v, off = client._decode_item(data, 0)
        assert v == "hello"
        assert off == 6
        v2, off2 = client._decode_item(data, off)
        assert v2 == "A"
        assert off2 == 8


class TestByteStrings:
    def test_empty(self, client):
        v, _ = client._decode_item(b"\x40", 0)
        assert v == b""

    def test_three_bytes(self, client):
        v, _ = client._decode_item(b"\x43\x01\x02\x03", 0)
        assert v == b"\x01\x02\x03"

    def test_hex_pattern(self, client):
        v, _ = client._decode_item(b"\x44\xde\xad\xbe\xef", 0)
        assert v == b"\xde\xad\xbe\xef"

    def test_with_null_bytes(self, client):
        v, _ = client._decode_item(b"\x45\x00\xff\x00\xff\x00", 0)
        assert v == b"\x00\xff\x00\xff\x00"

    def test_long_byte_string(self, client):
        payload = b"!" * 200
        data = b"\x58\xc8" + payload
        v, _ = client._decode_item(data, 0)
        assert v == payload


class TestArrays:
    def test_empty(self, client):
        v, _ = client._decode_item(b"\x80", 0)
        assert v == []

    def test_two_ints(self, client):
        v, _ = client._decode_item(b"\x82\x01\x02", 0)
        assert v == [1, 2]

    def test_nested(self, client):
        v, _ = client._decode_item(b"\x83\x01\x82\x02\x03\x82\x04\x05", 0)
        assert v == [1, [2, 3], [4, 5]]

    def test_mixed_types(self, client):
        v, _ = client._decode_item(b"\x84\x01\x62hi\xf5\xf6", 0)
        assert v == [1, "hi", True, None]

    def test_nested_empty_arrays(self, client):
        v, _ = client._decode_item(b"\x83\x80\x80\x80", 0)
        assert v == [[], [], []]


class TestMaps:
    def test_empty(self, client):
        v, _ = client._decode_item(b"\xa0", 0)
        assert v == {}

    def test_single_pair(self, client):
        v, _ = client._decode_item(b"\xa1\x62id\x18\x2a", 0)
        assert v == {"id": 42}

    def test_multiple_pairs(self, client):
        v, _ = client._decode_item(
            b"\xa2\x64name\x65eddie\x63age\x18\x23", 0
        )
        assert v == {"name": "eddie", "age": 35}

    def test_nested_map(self, client):
        v, _ = client._decode_item(
            b"\xa1\x64user\xa2\x62id\x01\x64name\x65alice", 0
        )
        assert v == {"user": {"id": 1, "name": "alice"}}

    def test_int_keys_converted_to_str(self, client):
        v, _ = client._decode_item(b"\xa1\x18\x2a\x65hello", 0)
        assert v == {"42": "hello"}
        assert "42" in v

    def test_deeply_nested_maps(self, client):
        v, _ = client._decode_item(
            b"\xa1\x63app\xa2\x64meta\xa1\x61v\x02\x64data\x80", 0
        )
        assert v == {"app": {"meta": {"v": 2}, "data": []}}


class TestBooleansAndNull:
    def test_false(self, client):
        v, _ = client._decode_item(b"\xf4", 0)
        assert v is False

    def test_true(self, client):
        v, _ = client._decode_item(b"\xf5", 0)
        assert v is True

    def test_null(self, client):
        v, _ = client._decode_item(b"\xf6", 0)
        assert v is None

    def test_bool_distinct_from_int(self, client):
        v, _ = client._decode_item(b"\xf5", 0)
        assert v is True
        assert type(v) is bool


class TestFloats:
    def test_half_precision_zero(self, client):
        v, _ = client._decode_item(b"\xf9\x00\x00", 0)
        assert v == 0.0
        assert isinstance(v, float)

    def test_half_precision_one(self, client):
        v, _ = client._decode_item(b"\xf9\x3c\x00", 0)
        assert v == 1.0

    def test_half_precision_negative_two(self, client):
        v, _ = client._decode_item(b"\xf9\xc0\x00", 0)
        assert v == -2.0

    def test_single_precision(self, client):
        value = 3.14
        data = b"\xfa" + struct.pack(">f", value)
        v, _ = client._decode_item(data, 0)
        assert v == pytest.approx(value, rel=1e-6)

    def test_single_precision_negative(self, client):
        value = -42.5
        data = b"\xfa" + struct.pack(">f", value)
        v, _ = client._decode_item(data, 0)
        assert v == pytest.approx(value, rel=1e-6)

    def test_double_precision(self, client):
        value = 2.718281828459045
        data = b"\xfb" + struct.pack(">d", value)
        v, _ = client._decode_item(data, 0)
        assert v == pytest.approx(value, rel=1e-10)


class TestTags:
    def test_tag1_timestamp(self, client):
        epoch = 1700000000
        inner = b"\x1a" + struct.pack(">I", epoch)
        data = b"\xc1" + inner
        v, _ = client._decode_item(data, 0)
        assert isinstance(v, str)
        assert "2023-11-14" in v

    def test_tag1_timestamp_small_int(self, client):
        data = b"\xc1\x18\x2a"
        v, _ = client._decode_item(data, 0)
        assert isinstance(v, str)
        assert "1970" in v

    def test_tag_unwrapped_with_tag_key(self, client):
        data = b"\xd8\x2a\x07"
        v, _ = client._decode_item(data, 0)
        assert v == {"@tag42": 7}

    def test_tag_with_string_value(self, client):
        data = b"\xd8\x64\x65hello"
        v, _ = client._decode_item(data, 0)
        assert v == {"@tag100": "hello"}

    def test_tag_with_array_value(self, client):
        data = b"\xd8\x2a\x83\x01\x02\x03"
        v, _ = client._decode_item(data, 0)
        assert v == {"@tag42": [1, 2, 3]}


class TestSelfDescribedCBOR:
    def test_unwraps_map(self, client):
        inner = b"\xa1\x61v\x18\x2a"
        data = b"\xd9\xd9\xf7" + inner
        result = client._parse_cbor(data)
        assert result == {"v": 42}

    def test_unwraps_nested_map(self, client):
        inner = b"\xa2\x64name\x65eddie\x64role\x65admin"
        data = b"\xd9\xd9\xf7" + inner
        result = client._parse_cbor(data)
        assert result == {"name": "eddie", "role": "admin"}

    def test_unwraps_array(self, client):
        inner = b"\x84\x01\x02\x03\x04"
        data = b"\xd9\xd9\xf7" + inner
        result = client._parse_cbor(data)
        assert result == [1, 2, 3, 4]

    def test_parse_cbor_without_tag_works(self, client):
        data = b"\xa1\x61a\x01"
        result = client._parse_cbor(data)
        assert result == {"a": 1}

    def test_only_tag_no_content(self, client):
        with pytest.raises(CBORWebError):
            client._parse_cbor(b"\xd9\xd9\xf7")


# ═══════════════════════════════════════════════════════════════════
# 2. Path Encoding — _path_to_filename
# ═══════════════════════════════════════════════════════════════════


class TestPathEncoding:
    def test_root(self, client):
        assert client._path_to_filename("/") == "_index.cbor"

    def test_simple(self, client):
        assert client._path_to_filename("/about") == "about.cbor"

    def test_nested(self, client):
        assert (
            client._path_to_filename("/blog/2024/post-title")
            == "blog_2024_post-title.cbor"
        )

    def test_underscore_escaping(self, client):
        assert client._path_to_filename("/my_page") == "my%5Fpage.cbor"

    def test_multiple_underscores(self, client):
        assert (
            client._path_to_filename("/file_name_test")
            == "file%5Fname%5Ftest.cbor"
        )

    def test_leading_underscore(self, client):
        assert client._path_to_filename("/_hidden") == "%5Fhidden.cbor"

    def test_trailing_underscore(self, client):
        assert client._path_to_filename("/dir_") == "dir%5F.cbor"

    def test_hyphens_and_dots(self, client):
        assert client._path_to_filename("/my-file-v2.html") == "my-file-v2.html.cbor"

    def test_numeric_segments(self, client):
        assert client._path_to_filename("/2024/12/25") == "2024_12_25.cbor"

    def test_single_level_rootless(self, client):
        assert client._path_to_filename("contact") == "contact.cbor"

    def test_deeply_nested(self, client):
        assert (
            client._path_to_filename("/a/b/c/d/e")
            == "a_b_c_d_e.cbor"
        )

    def test_mixed_underscore_and_slash(self, client):
        assert (
            client._path_to_filename("/a_b/c_d")
            == "a%5Fb_c%5Fd.cbor"
        )


# ═══════════════════════════════════════════════════════════════════
# 3. Content Block Matching — _block_matches
# ═══════════════════════════════════════════════════════════════════


class TestContentBlockMatching:

    def test_heading_block_v_matches(self, client):
        block = {"t": "h", "v": "Welcome to Our Site"}
        assert client._block_matches(block, "welcome") is True

    def test_heading_block_v_no_match(self, client):
        block = {"t": "h", "v": "Welcome to Our Site"}
        assert client._block_matches(block, "goodbye") is False

    def test_paragraph_block_text_matches(self, client):
        block = {"t": "p", "text": "This is a paragraph about CBOR."}
        assert client._block_matches(block, "cbor") is True

    def test_paragraph_block_partial_word(self, client):
        block = {"t": "p", "text": "The quick brown fox"}
        assert client._block_matches(block, "quick") is True

    def test_paragraph_block_case_insensitive(self, client):
        block = {"t": "p", "text": "THIS IS UPPERCASE"}
        assert client._block_matches(block, "uppercase") is True

    def test_list_block_v_matches_item(self, client):
        block = {"t": "l", "v": ["apple", "banana", "cherry"]}
        assert client._block_matches(block, "banana") is True

    def test_list_block_nested_v_search(self, client):
        block = {"t": "l", "v": ["alpha", "beta", "gamma"]}
        assert client._block_matches(block, "beta") is True

    def test_list_block_no_match(self, client):
        block = {"t": "l", "v": ["red", "green", "blue"]}
        assert client._block_matches(block, "yellow") is False

    def test_quote_block_text_matches(self, client):
        block = {"t": "q", "text": "To be or not to be"}
        assert client._block_matches(block, "not to be") is True

    def test_quote_block_no_match(self, client):
        block = {"t": "q", "text": "Hello world"}
        assert client._block_matches(block, "farewell") is False

    def test_code_block_v_matches(self, client):
        block = {"t": "c", "v": "def hello(): return True"}
        assert client._block_matches(block, "hello") is True

    def test_code_block_no_match(self, client):
        block = {"t": "c", "v": "x = 1 + 1"}
        assert client._block_matches(block, "function") is False

    def test_table_block_v_list_matches(self, client):
        block = {"t": "tb", "v": ["Name", "Email", "Phone"]}
        assert client._block_matches(block, "email") is True

    def test_image_block_alt_matches(self, client):
        block = {"t": "i", "alt": "A sunset over the ocean"}
        assert client._block_matches(block, "sunset") is True

    def test_image_block_src_matches(self, client):
        block = {"t": "i", "src": "/images/logo.png", "alt": ""}
        assert client._block_matches(block, "logo") is True

    def test_image_block_no_match(self, client):
        block = {"t": "i", "alt": "Photo", "src": "/img/pic.jpg"}
        assert client._block_matches(block, "diagram") is False

    def test_cta_block_href_matches(self, client):
        block = {"t": "cta", "href": "/contact-us", "v": "Get in touch"}
        assert client._block_matches(block, "contact") is True

    def test_cta_block_v_matches(self, client):
        block = {"t": "cta", "href": "/signup", "v": "Join now"}
        assert client._block_matches(block, "join") is True

    def test_cta_block_no_match(self, client):
        block = {"t": "cta", "href": "/about", "v": "Learn more"}
        assert client._block_matches(block, "pricing") is False

    def test_attr_field_matches(self, client):
        block = {"t": "div", "attr": "class=highlight data-id=xyz123"}
        assert client._block_matches(block, "xyz123") is True

    def test_no_fields_match(self, client):
        block = {"t": "p", "v": "hello", "text": "world"}
        assert client._block_matches(block, "nonexistent") is False

    def test_empty_block(self, client):
        assert client._block_matches({}, "anything") is False

    def test_block_matches_across_v_field_as_str(self, client):
        block = {"t": "h", "v": "Main Title"}
        assert client._block_matches(block, "title") is True

    def test_v_field_is_none(self, client):
        block = {"t": "p", "v": None}
        assert client._block_matches(block, "anything") is False


# ═══════════════════════════════════════════════════════════════════
# 4. Error Handling
# ═══════════════════════════════════════════════════════════════════


class TestErrorHandling:

    def test_parse_cbor_empty_data(self, client):
        with pytest.raises(CBORWebError, match="too short"):
            client._parse_cbor(b"")

    def test_parse_cbor_one_byte(self, client):
        with pytest.raises(CBORWebError, match="too short"):
            client._parse_cbor(b"\xa1")

    def test_parse_cbor_two_bytes(self, client):
        with pytest.raises(CBORWebError, match="too short"):
            client._parse_cbor(b"\xa1\x61")

    def test_decode_item_empty_data(self, client):
        with pytest.raises(CBORWebError, match="Unexpected end"):
            client._decode_item(b"", 0)

    def test_decode_item_offset_past_end(self, client):
        with pytest.raises(CBORWebError, match="Unexpected end"):
            client._decode_item(b"\x01", 10)

    def test_truncated_map_missing_pairs(self, client):
        with pytest.raises(CBORWebError, match="Unexpected end"):
            client._decode_item(b"\xa3\x61a\x01", 0)

    def test_truncated_array_missing_items(self, client):
        with pytest.raises(CBORWebError, match="Unexpected end"):
            client._decode_item(b"\x85\x01\x02", 0)

    def test_truncated_tag_no_inner_value(self, client):
        with pytest.raises(CBORWebError, match="Unexpected end"):
            client._decode_item(b"\xc1", 0)

    def test_invalid_utf8_in_text_string(self, client):
        data = b"\x64\x80\x81\x82\x83"
        with pytest.raises(UnicodeDecodeError):
            client._decode_item(data, 0)

    def test_incomplete_length_byte(self, client):
        with pytest.raises(IndexError):
            client._decode_item(b"\x78", 0)

    def test_incomplete_two_byte_length(self, client):
        data = b"\x59\x01"
        with pytest.raises((CBORWebError, IndexError, struct.error)):
            client._decode_item(data, 0)

    def test_nested_truncation_array_in_array(self, client):
        data = b"\x82\x01\x83\x02\x03"
        with pytest.raises(CBORWebError, match="Unexpected end"):
            client._decode_item(data, 0)

    def test_nested_truncation_map_in_map(self, client):
        data = b"\xa1\x62k1\xa2\x62k2\x01"
        with pytest.raises(CBORWebError, match="Unexpected end"):
            client._decode_item(data, 0)

    def test_parse_cbor_only_tag(self, client):
        with pytest.raises(CBORWebError):
            client._parse_cbor(b"\xd9\xd9\xf7")


# ═══════════════════════════════════════════════════════════════════
# 5. Client Instantiation
# ═══════════════════════════════════════════════════════════════════


class TestClientInstantiation:
    def test_default_values(self):
        c = CBORWebClient("example.com")
        assert c.domain == "example.com"
        assert c.base == "https://example.com"
        assert c.token == ""
        assert c.timeout == 10

    def test_with_token(self):
        c = CBORWebClient("mysite.com", token="abc123")
        assert c.token == "abc123"

    def test_with_custom_timeout(self):
        c = CBORWebClient("mysite.com", timeout=30)
        assert c.timeout == 30

    def test_trailing_slash_stripped(self):
        c = CBORWebClient("mysite.com/")
        assert c.domain == "mysite.com"
        assert c.base == "https://mysite.com"

    def test_well_known_constant(self):
        c = CBORWebClient("test.com")
        assert c.WELL_KNOWN == "/.well-known/cbor-web"


# ═══════════════════════════════════════════════════════════════════
# 6. CBORWebError
# ═══════════════════════════════════════════════════════════════════


class TestCBORWebError:
    def test_string_message(self):
        e = CBORWebError("Something went wrong")
        assert str(e) == "Something went wrong"
        assert e.code == 0

    def test_with_http_code(self):
        e = CBORWebError("Not found", code=404)
        assert e.code == 404

    def test_is_exception_subclass(self):
        e = CBORWebError("test")
        assert isinstance(e, Exception)
