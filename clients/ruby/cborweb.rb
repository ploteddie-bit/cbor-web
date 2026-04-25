# SPDX-License-Identifier: MIT
# License: MIT — Copyright (c) 2026 ExploDev / Deltopide SL
# Repository: https://github.com/ploteddie-bit/cbor-web

# CBOR-Web Ruby Client — stdlib only
require 'net/http'
require 'json'
require 'stringio'
require 'time'

class CBORWebClient
  WELL_KNOWN = '/.well-known/cbor-web'

  def initialize(base_url)
    @base_url = base_url.chomp('/')
  end

  def manifest
    cbor_get(WELL_KNOWN)
  end

  def page(path)
    cbor_get("#{WELL_KNOWN}/pages/#{encode_page_path(path)}.cbor")
  end

  def bundle
    cbor_get("#{WELL_KNOWN}/bundle")
  end

  private

  def encode_page_path(path)
    return '_index' if path == '/'
    path.gsub('_', '%5F').sub(/\A\//, '').gsub('/', '_')
  end

  def cbor_get(path)
    uri = URI("#{@base_url}#{path}")
    req = Net::HTTP::Get.new(uri)
    req['Accept'] = 'application/cbor'
    res = Net::HTTP.start(uri.hostname, uri.port, use_ssl: uri.scheme == 'https') do |http|
      http.request(req)
    end
    raise "HTTP #{res.code}" unless res.code.to_i == 200
    decode_cbor(res.body)
  end

  def decode_cbor(data)
    io = StringIO.new(data.b)
    val = decode_item(io)
    raise 'trailing data' unless io.eof?
    val
  end

  def decode_item(io)
    ib = io.getbyte or raise 'unexpected EOF'
    major = ib >> 5
    info = ib & 0x1F
    arg = read_arg(io, info)

    case major
    when 0 then arg
    when 1 then -1 - arg
    when 2 then io.read(arg)
    when 3 then io.read(arg).force_encoding('UTF-8')
    when 4 then Array.new(arg) { decode_item(io) }
    when 5
      Hash[Array.new(arg) { [decode_item(io).to_s, decode_item(io)] }]
    when 6
      inner = decode_item(io)
      case arg
      when 55799 then inner
      when 1 then Time.at(inner.to_i).utc
      else { '@tag' => arg, '@value' => inner }
      end
    when 7
      case info
      when 20 then false
      when 21 then true
      when 22 then nil
      when 25 then decode_f16(io.read(2))
      when 26 then io.read(4).reverse.unpack1('e')
      when 27 then io.read(8).reverse.unpack1('E')
      end
    end
  end

  def read_arg(io, info)
    return info if info < 24
    len = case info
          when 24 then 1
          when 25 then 2
          when 26 then 4
          when 27 then 8
          else raise "unsupported arg info #{info}"
          end
    fmt = { 1 => 'C', 2 => 'n', 4 => 'N', 8 => 'Q>' }[len]
    io.read(len).unpack1(fmt)
  end

  def decode_f16(raw)
    bits = raw.unpack1('n')
    sign = (bits >> 15) & 1
    exp = (bits >> 10) & 0x1F
    mant = bits & 0x3FF
    val = if exp == 0
            mant * (2.0 ** -24)
          elsif exp == 31
            mant == 0 ? Float::INFINITY : Float::NAN
          else
            (1.0 + mant / 1024.0) * (2 ** (exp - 15))
          end
    sign == 1 ? -val : val
  end
end
