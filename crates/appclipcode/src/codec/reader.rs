use super::tables::{
    decode_fixed6, decode_uleb128, fixed_tld_by_index, known_word_by_index, tld_huffman_coder,
    HUFFMAN_TLDS,
};
use super::trie::{cpq_coder, host_coder, spq_coder, CPQ_SYMBOLS, HOST_SYMBOLS, SPQ_SYMBOLS};

pub fn decompress_url(payload: &[u8]) -> Result<String, String> {
    if payload.len() != 16 {
        return Err(format!("expected 16-byte payload, got {}", payload.len()));
    }
    let mut bits = Vec::with_capacity(128);
    for i in 0..16 {
        let b = payload[i];
        for j in (0..8).rev() {
            bits.push(((b >> j) & 1) == 1);
        }
    }

    let mut start_bit = None;
    for (i, &b) in bits.iter().enumerate() {
        if b {
            start_bit = Some(i + 1);
            break;
        }
    }

    let start_bit = start_bit.ok_or_else(|| "no begin marker found".to_string())?;
    let data = &bits[start_bit..];
    let mut pos = 0;

    let template_bit = read_bit(data, &mut pos)?;
    let template_type = if template_bit { 1 } else { 0 };

    let has_app_clip = read_bit(data, &mut pos)?;

    let fmt_bit = read_bit(data, &mut pos)?;
    let host_format = if fmt_bit {
        let fmt_bit2 = read_bit(data, &mut pos)?;
        if fmt_bit2 { 2 } else { 1 }
    } else {
        0
    };

    let host;
    let mut has_path = false;

    match host_format {
        0 => {
            let tld_hc = tld_huffman_coder();
            let tld_idx = tld_hc.decode(data, &mut pos)
                .map_err(|e| format!("decode TLD: {}", e))?;
            if tld_idx >= HUFFMAN_TLDS.len() {
                return Err(format!("invalid TLD index {}", tld_idx));
            }
            let tld = HUFFMAN_TLDS[tld_idx].0;

            let mut domain = decode_host_chars(data, &mut pos)?;
            if domain.ends_with('|') {
                domain.pop();
                has_path = true;
            }
            host = format!("{}{}", domain, tld);
        }
        1 => {
            let tld_idx = read_bits(data, &mut pos, 8)?;
            let tld = fixed_tld_by_index(tld_idx)
                .ok_or_else(|| format!("unknown fixed TLD index {}", tld_idx))?;

            let mut domain = decode_host_chars(data, &mut pos)?;
            if domain.ends_with('|') {
                domain.pop();
                has_path = true;
            }
            host = format!("{}{}", domain, tld);
        }
        2 => {
            let mut full_host = decode_host_chars(data, &mut pos)?;
            if full_host.ends_with('|') {
                full_host.pop();
                has_path = true;
            }
            host = full_host;
        }
        _ => unreachable!(),
    }

    let mut url = String::from("https://");
    if has_app_clip {
        url.push_str("appclip.");
    }
    url.push_str(&host);

    if has_path {
        if template_type == 1 {
            let path_query = decode_auto_query_template_rest(data, &mut pos)?;
            url.push_str(&path_query);
        } else {
            let type_bit = read_bit(data, &mut pos)?;
            if !type_bit {
                let mut path = decode_cpq_chars(data, &mut pos)?;
                if !path.is_empty() && !path.starts_with('/') && !path.starts_with('#') {
                    path.insert(0, '/');
                }
                url.push_str(&path);
            } else {
                let path_query = decode_segmented_path_query(data, &mut pos)?;
                url.push_str(&path_query);
            }
        }
    }

    Ok(url)
}

fn read_bit(data: &[bool], pos: &mut usize) -> Result<bool, String> {
    if *pos >= data.len() {
        return Err(format!("unexpected end of data at bit {}", *pos));
    }
    let b = data[*pos];
    *pos += 1;
    Ok(b)
}

fn read_bits(data: &[bool], pos: &mut usize, n: usize) -> Result<usize, String> {
    let mut val = 0;
    for _ in 0..n {
        let b = read_bit(data, pos)?;
        val = (val << 1) | (if b { 1 } else { 0 });
    }
    Ok(val)
}

fn decode_chars_with_start_context(
    coder: &super::trie::MultiContextHuffmanCoder,
    symbols: &[&str],
    data: &[bool],
    pos: &mut usize,
    stop_sym: &str,
    start_ctx: &str,
) -> Result<String, String> {
    let mut result = String::new();
    let mut node_offset = 0;
    let mut depth = 0;

    for c in start_ctx.chars() {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        let idx = coder.symbol_index(s)
            .ok_or_else(|| format!("unknown start context symbol: {:?}", s))?;
        let (n, d) = coder.advance_context(node_offset, depth, idx);
        node_offset = n;
        depth = d;
    }

    while *pos < data.len() {
        let hc = coder.coder_for_node(node_offset);
        let idx = match hc.decode(data, pos) {
            Ok(i) => i,
            Err(_) => break,
        };
        if idx >= symbols.len() {
            break;
        }
        let sym = symbols[idx];
        result.push_str(sym);

        if !stop_sym.is_empty() && sym == stop_sym {
            break;
        }

        let (n, d) = coder.advance_context(node_offset, depth, idx);
        node_offset = n;
        depth = d;
    }

    Ok(result)
}

fn decode_host_chars(data: &[bool], pos: &mut usize) -> Result<String, String> {
    decode_chars_with_start_context(host_coder(), &HOST_SYMBOLS, data, pos, "|", "")
}

fn decode_cpq_chars(data: &[bool], pos: &mut usize) -> Result<String, String> {
    decode_chars_with_start_context(cpq_coder(), &CPQ_SYMBOLS, data, pos, "", "")
}

fn decode_spq_value_until_terminator(data: &[bool], pos: &mut usize, start_ctx: &str) -> Result<String, String> {
    let s = decode_chars_with_start_context(spq_coder(), &SPQ_SYMBOLS, data, pos, "|", start_ctx)?;
    Ok(s.trim_end_matches('|').to_string())
}

fn decode_auto_query_template_rest(data: &[bool], pos: &mut usize) -> Result<String, String> {
    if *pos >= data.len() {
        return Ok(String::new());
    }

    let first = read_bit(data, pos)?;
    let mut sb = String::new();

    if !first {
        let word_idx = read_bits(data, pos, 8)?;
        let word = known_word_by_index(word_idx)
            .ok_or_else(|| format!("unknown template path word index {}", word_idx))?;
        sb.push('/');
        sb.push_str(word);

        if *pos >= data.len() {
            return Ok(sb);
        }

        let query_indicator = read_bit(data, pos)?;
        if !query_indicator {
            return Err("encountered path indicator while decoding template query".to_string());
        }
    }

    if *pos >= data.len() {
        return Ok(sb);
    }

    sb.push('?');
    let mut i = 0;
    while *pos < data.len() {
        if i > 0 {
            sb.push('&');
        }
        sb.push('p');
        if i > 0 {
            sb.push_str(&i.to_string());
        }
        sb.push('=');

        let component_type = read_bits(data, pos, 2)?;
        let value = match component_type {
            0 => decode_spq_value_until_terminator(data, pos, "=")?,
            1 => decode_uleb128(data, pos)?,
            2 => {
                let s = decode_fixed6(data, pos, 1024)?;
                s.trim_end_matches('|').to_string()
            }
            _ => return Err(format!("invalid template query component type {}", component_type)),
        };
        sb.push_str(&value);
        i += 1;
    }

    Ok(sb)
}

fn decode_segmented_path_query(data: &[bool], pos: &mut usize) -> Result<String, String> {
    let mut path_parts = Vec::new();
    let mut root_only = false;
    let mut trailing_slash = false;

    while *pos < data.len() {
        let first = read_bit(data, pos)?;
        if first {
            let second = read_bit(data, pos)?;
            if !second {
                if path_parts.is_empty() {
                    root_only = true;
                }
                trailing_slash = true;
                continue;
            }

            let query = decode_segmented_query_string(data, pos)?;
            let mut path = build_segmented_path(&path_parts, root_only, trailing_slash);
            if path.is_empty() {
                path.push('/');
            }
            return Ok(format!("{}{}", path, query));
        }

        let component = decode_segmented_path_component(data, pos)?;
        path_parts.push(component);
        root_only = false;
        trailing_slash = false;
    }

    Ok(build_segmented_path(&path_parts, root_only, trailing_slash))
}

fn build_segmented_path(path_parts: &[String], root_only: bool, trailing_slash: bool) -> String {
    if path_parts.is_empty() {
        if root_only {
            return "/".to_string();
        }
        return String::new();
    }

    let mut path = format!("/{}", path_parts.join("/"));
    if trailing_slash {
        path.push('/');
    }
    path
}

fn decode_segmented_path_component(data: &[bool], pos: &mut usize) -> Result<String, String> {
    let component_type = read_bits(data, pos, 2)?;
    match component_type {
        0 => decode_spq_value_until_terminator(data, pos, ""),
        1 => decode_uleb128(data, pos),
        2 => {
            let s = decode_fixed6(data, pos, 1024)?;
            Ok(s.trim_end_matches('|').to_string())
        }
        3 => {
            let word_idx = read_bits(data, pos, 8)?;
            let word = known_word_by_index(word_idx)
                .ok_or_else(|| format!("unknown segmented path word index {}", word_idx))?;
            Ok(word.to_string())
        }
        _ => Err(format!("invalid segmented path component type {}", component_type)),
    }
}

fn decode_segmented_query_string(data: &[bool], pos: &mut usize) -> Result<String, String> {
    let mut sb = String::from("?");
    let mut first_component = true;

    while *pos < data.len() {
        let (key, value) = decode_segmented_query_component(data, pos)?;
        if !first_component {
            sb.push('&');
        }
        first_component = false;
        sb.push_str(&key);
        sb.push('=');
        sb.push_str(&value);
    }

    Ok(sb)
}

fn decode_segmented_query_component(data: &[bool], pos: &mut usize) -> Result<(String, String), String> {
    let component_type = read_bits(data, pos, 2)?;
    match component_type {
        0 => {
            let key = decode_spq_value_until_terminator(data, pos, "?")?;
            let value = decode_spq_value_until_terminator(data, pos, "=")?;
            Ok((key, value))
        }
        1 => {
            let value = decode_uleb128(data, pos)?;
            let key = decode_spq_value_until_terminator(data, pos, "?")?;
            Ok((key, value))
        }
        2 => {
            let key = decode_spq_value_until_terminator(data, pos, "?")?;
            let value_raw = decode_fixed6(data, pos, 1024)?;
            let value = value_raw.trim_end_matches('|').to_string();
            Ok((key, value))
        }
        _ => Err(format!("invalid segmented query component type {}", component_type)),
    }
}
