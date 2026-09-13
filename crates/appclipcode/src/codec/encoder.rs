use super::tables::{
    encode_fixed6, encode_uleb128, fixed_tld_index, huffman_tld_index, int_to_bits,
    known_word_index, tld_huffman_coder,
};
use super::trie::{cpq_coder, host_coder, spq_coder};

pub fn compress_url(raw_url: &str) -> Result<Vec<u8>, String> {
    let u = parse_compression_url(raw_url)?;

    let mut host = u.host.as_str();
    let subdomain_type = if host.starts_with("appclip.") {
        host = &host["appclip.".len()..];
        1
    } else {
        0
    };

    let has_path_or_query = !u.path.is_empty() || !u.query.is_empty() || !u.fragment.is_empty();

    let (pq_bits, template_type) = if has_path_or_query {
        choose_path_query_encoding(&u.path, &u.query, &u.fragment)?
    } else {
        (String::new(), 0)
    };

    let mut bits = String::new();
    bits.push('1'); // begin marker
    bits.push(if template_type == 1 { '1' } else { '0' });
    bits.push(if subdomain_type == 1 { '1' } else { '0' });

    let (host_bits, host_fmt) = encode_host(host, has_path_or_query)?;
    match host_fmt {
        0 => bits.push('0'),
        1 => bits.push_str("10"),
        2 => bits.push_str("11"),
        _ => unreachable!(),
    }

    bits.push_str(&host_bits);
    bits.push_str(&pq_bits);

    raw_bits_to_bytes(&bits)
}

fn raw_bits_to_bytes(bits: &str) -> Result<Vec<u8>, String> {
    if bits.len() > 128 {
        return Err(format!(
            "compressed URL too large: {} bits (max 128)",
            bits.len()
        ));
    }

    let mut padded = String::with_capacity(128);
    for _ in 0..(128 - bits.len()) {
        padded.push('0');
    }
    padded.push_str(bits);

    let mut result = vec![0u8; 16];
    let bytes = padded.as_bytes();
    for i in 0..16 {
        let mut b = 0u8;
        for j in 0..8 {
            if bytes[i * 8 + j] == b'1' {
                b |= 1 << (7 - j);
            }
        }
        result[i] = b;
    }
    Ok(result)
}

#[derive(Debug, Clone)]
struct CompressionURL {
    host: String,
    path: String,
    query: String,
    fragment: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum URLComponentKind {
    Path,
    Query,
    Fragment,
}

fn parse_compression_url(raw_url: &str) -> Result<CompressionURL, String> {
    const SCHEME: &str = "https://";
    if raw_url.len() < SCHEME.len() || !raw_url[..SCHEME.len()].eq_ignore_ascii_case(SCHEME) {
        return Err("URL scheme must be https".to_string());
    }

    let rest = &raw_url[SCHEME.len()..];
    let authority_end = rest.find(|c| c == '/' || c == '?' || c == '#');
    let (authority, mut suffix) = match authority_end {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, ""),
    };

    if authority.is_empty() {
        return Err("URL must have a host".to_string());
    }
    if authority.contains('@') {
        return Err("URL must not have user info".to_string());
    }
    if authority.contains(':') {
        return Err("URL must not have a port".to_string());
    }

    let host = canonicalize_host(authority)?;
    let mut out = CompressionURL {
        host,
        path: String::new(),
        query: String::new(),
        fragment: String::new(),
    };

    if suffix.starts_with('/') {
        let path_end = suffix.find(|c| c == '?' || c == '#').unwrap_or(suffix.len());
        out.path = canonicalize_url_component(&suffix[..path_end], URLComponentKind::Path)?;
        suffix = &suffix[path_end..];
    }

    if suffix.starts_with('?') {
        suffix = &suffix[1..];
        let query_end = suffix.find('#').unwrap_or(suffix.len());
        out.query = canonicalize_url_component(&suffix[..query_end], URLComponentKind::Query)?;
        suffix = &suffix[query_end..];
    }

    if suffix.starts_with('#') {
        out.fragment = canonicalize_url_component(&suffix[1..], URLComponentKind::Fragment)?;
    }

    Ok(out)
}

fn canonicalize_host(authority: &str) -> Result<String, String> {
    let lower = authority.to_ascii_lowercase();
    for &b in lower.as_bytes() {
        if b.is_ascii_alphanumeric() || b == b'.' || b == b'-' {
            continue;
        }
        return Err("URL contains unsupported host characters".to_string());
    }

    for label in lower.split('.') {
        if label.starts_with("xn--") {
            return Err("URL contains unsupported host characters".to_string());
        }
    }
    Ok(lower)
}

fn canonicalize_url_component(s: &str, kind: URLComponentKind) -> Result<String, String> {
    let mut b = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'%' {
            if i + 2 < bytes.len() && is_hex_digit(bytes[i + 1]) && is_hex_digit(bytes[i + 2]) {
                b.push('%');
                b.push(bytes[i + 1] as char);
                b.push(bytes[i + 2] as char);
                i += 3;
                continue;
            }
            return Err("URL contains invalid percent escape".to_string());
        }
        if c < 0x20 || c == 0x7f || c >= 0x80 {
            return Err("URL contains unsupported characters".to_string());
        }
        if rejects_raw_url_component_byte(c, kind) {
            return Err("URL contains unsupported characters".to_string());
        }
        if is_allowed_url_component_byte(c, kind) {
            b.push(c as char);
        } else {
            write_percent_encoded_byte(&mut b, c);
        }
        i += 1;
    }
    Ok(b)
}

fn rejects_raw_url_component_byte(c: u8, kind: URLComponentKind) -> bool {
    match c {
        b' ' | b'"' | b'%' | b'<' | b'>' | b'\\' | b'^' | b'`' | b'{' | b'|' | b'}' => true,
        b'#' => kind == URLComponentKind::Fragment,
        _ => false,
    }
}

fn is_allowed_url_component_byte(c: u8, kind: URLComponentKind) -> bool {
    if c.is_ascii_alphanumeric() {
        return true;
    }
    match c {
        b'-' | b'.' | b'_' | b'~' | b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+'
        | b',' | b';' | b'=' | b':' | b'@' | b'/' => true,
        b'?' => kind != URLComponentKind::Path,
        b'#' => false,
        _ => false,
    }
}

fn is_hex_digit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

fn write_percent_encoded_byte(b: &mut String, c: u8) {
    const HEX: &[u8] = b"0123456789ABCDEF";
    b.push('%');
    b.push(HEX[(c >> 4) as usize] as char);
    b.push(HEX[(c & 0x0f) as usize] as char);
}

fn encode_host(host: &str, has_path_or_query: bool) -> Result<(String, usize), String> {
    let last_dot = host.rfind('.').ok_or_else(|| format!("host has no TLD: {:?}", host))?;
    let tld = &host[last_dot..];
    let mut domain = host[..last_dot].to_string();

    if has_path_or_query {
        domain.push('|');
    }

    let domain_chars: Vec<&str> = domain.split_inclusive(|_| true).collect();

    // Try format 0: Huffman TLD
    if let Some(tld_idx) = huffman_tld_index(tld) {
        let tld_hc = tld_huffman_coder();
        if tld_hc.can_encode(tld_idx) {
            if let Ok(domain_bits) = host_coder().encode(&domain_chars) {
                let tld_bits = tld_hc.encode(tld_idx);
                return Ok((format!("{}{}", tld_bits, domain_bits), 0));
            }
        }
    }

    // Try format 1: Fixed TLD
    if let Some(tld_idx) = fixed_tld_index(tld) {
        if let Ok(domain_bits) = host_coder().encode(&domain_chars) {
            let f_bits = int_to_bits(tld_idx, 8);
            return Ok((format!("{}{}", f_bits, domain_bits), 1));
        }
    }

    // Format 2: Full host with Huffman
    let mut full_host = host.to_string();
    if has_path_or_query {
        full_host.push('|');
    }
    let host_chars: Vec<&str> = full_host.split_inclusive(|_| true).collect();
    let all_bits = host_coder().encode(&host_chars)
        .map_err(|e| format!("encode full host {:?}: {}", host, e))?;
    Ok((all_bits, 2))
}

fn choose_path_query_encoding(
    path: &str,
    query: &str,
    fragment: &str,
) -> Result<(String, usize), String> {
    struct Candidate {
        bits: String,
        template_type: usize,
    }

    let mut candidates = Vec::new();
    if let Ok(bits) = encode_template_path_query(path, query, fragment) {
        candidates.push(Candidate {
            bits,
            template_type: 1,
        });
    }
    if let Ok(bits) = encode_non_template_path_query(path, query, fragment) {
        candidates.push(Candidate {
            bits,
            template_type: 0,
        });
    }

    if candidates.is_empty() {
        return Err("cannot encode path/query".to_string());
    }

    let mut best = &candidates[0];
    for cand in &candidates[1..] {
        if cand.bits.len() < best.bits.len() {
            best = cand;
        } else if cand.bits.len() == best.bits.len() && best.template_type == 1 && cand.template_type == 0 {
            best = cand;
        }
    }

    Ok((best.bits.clone(), best.template_type))
}

fn encode_template_path_query(path: &str, query: &str, fragment: &str) -> Result<String, String> {
    if !fragment.is_empty() {
        return Err("template mode does not support fragments".to_string());
    }

    let (path_word, params) = match_auto_query_template(path, query)
        .ok_or_else(|| "path/query do not match template auto-query format".to_string())?;

    let mut bits = String::new();
    if let Some(word) = path_word {
        let idx = known_word_index(word).ok_or_else(|| format!("unknown template word {:?}", word))?;
        if idx > 0xff {
            return Err(format!("template path word {:?} exceeds 8-bit auto-query range", word));
        }
        bits.push('0');
        bits.push_str(&int_to_bits(idx, 8));
    }

    if !params.is_empty() {
        bits.push('1');
        for (i, param) in params.iter().enumerate() {
            let component_bits = encode_auto_query_template_query_component(param, i + 1 < params.len())?;
            bits.push_str(&component_bits);
        }
    }

    if bits.is_empty() {
        return Err("template mode requires a path word or auto-query parameters".to_string());
    }
    Ok(bits)
}

fn match_auto_query_template<'a>(path: &'a str, query: &'a str) -> Option<(Option<&'a str>, Vec<&'a str>)> {
    if path.len() >= 2 && path.ends_with('/') {
        return None;
    }
    if query.ends_with('&') {
        return None;
    }

    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if path_parts.len() > 1 {
        return None;
    }

    let path_word = if path_parts.len() == 1 {
        let word = path_parts[0];
        let idx = known_word_index(word)?;
        if idx > 0xff {
            return None;
        }
        Some(word)
    } else {
        None
    };

    let params: Vec<&str> = query.split('&').filter(|s| !s.is_empty()).collect();
    if params.is_empty() {
        if path_word.is_some() || path == "/" {
            return Some((path_word, Vec::new()));
        } else {
            return None;
        }
    }

    for (i, param) in params.iter().enumerate() {
        let (key, _) = param.split_once('=')?;
        let want_key = if i == 0 {
            "p".to_string()
        } else {
            format!("p{}", i)
        };
        if key != want_key {
            return None;
        }
    }

    Some((path_word, params))
}

fn encode_auto_query_template_query_component(param: &str, has_more: bool) -> Result<String, String> {
    let (_, value) = param.split_once('=')
        .ok_or_else(|| format!("template query parameter {:?} missing '='", param))?;

    let mut best_bits = String::new();
    if let Ok(bits) = encode_spq_value("=", value, has_more) {
        best_bits = format!("00{}", bits);
    }
    if let Ok(bits) = encode_uleb128(value) {
        let cand = format!("01{}", bits);
        best_bits = shorter_bits(&best_bits, &cand);
    }
    if let Ok(bits) = encode_fixed6_value(value, has_more) {
        let cand = format!("10{}", bits);
        best_bits = shorter_bits(&best_bits, &cand);
    }

    if best_bits.is_empty() {
        return Err(format!("cannot encode template query value from {:?}", param));
    }
    Ok(best_bits)
}

fn encode_non_template_path_query(path: &str, query: &str, fragment: &str) -> Result<String, String> {
    let combined_res = encode_combined_path_query(path, query, fragment);
    let segmented_res = encode_segmented_path_query(path, query, fragment);

    match (combined_res, segmented_res) {
        (Ok(c), Ok(s)) => {
            if c.len() <= s.len() {
                Ok(format!("0{}", c))
            } else {
                Ok(format!("1{}", s))
            }
        }
        (Ok(c), Err(_)) => Ok(format!("0{}", c)),
        (Err(_), Ok(s)) => Ok(format!("1{}", s)),
        (Err(ce), Err(se)) => Err(format!("cannot encode path/query: combined: {}, segmented: {}", ce, se)),
    }
}

fn encode_combined_path_query(path: &str, query: &str, fragment: &str) -> Result<String, String> {
    let mut combined = String::new();
    combined.push_str(path);
    if !query.is_empty() {
        combined.push('?');
        combined.push_str(query);
    }
    if !fragment.is_empty() {
        combined.push('#');
        combined.push_str(fragment);
    }

    if combined.starts_with('/') && (combined.len() == 1 || combined.as_bytes()[1] != b'#') {
        combined.remove(0);
    }
    if combined.is_empty() {
        return Err("combined path/query is empty".to_string());
    }

    let chars: Vec<&str> = combined.split_inclusive(|_| true).collect();
    cpq_coder().encode(&chars)
}

fn encode_segmented_path_query(path: &str, query: &str, fragment: &str) -> Result<String, String> {
    if !fragment.is_empty() {
        return Err("segmented mode does not support fragments".to_string());
    }

    let items = build_segmented_path_items(path);
    let mut bits = String::new();

    for (i, item) in items.iter().enumerate() {
        if *item == "/" {
            bits.push_str("10");
            continue;
        }

        let has_more_path_items = i + 1 < items.len();
        let component_bits = encode_segmented_path_component(item, has_more_path_items || !query.is_empty())?;
        bits.push('0');
        bits.push_str(&component_bits);
    }

    if !query.is_empty() {
        let params: Vec<&str> = query.split('&').collect();
        if params.is_empty() {
            return Err("invalid segmented query".to_string());
        }
        bits.push_str("11");
        for (i, param) in params.iter().enumerate() {
            let component_bits = encode_segmented_query_component(param, i + 1 < params.len())?;
            bits.push_str(&component_bits);
        }
    }

    if bits.is_empty() {
        return Err("segmented path/query is empty".to_string());
    }
    Ok(bits)
}

fn build_segmented_path_items(path: &str) -> Vec<&str> {
    if path.is_empty() {
        return Vec::new();
    }

    let trimmed = path.trim_start_matches('/');
    let mut items: Vec<&str> = trimmed.split('/').filter(|s| !s.is_empty()).collect();
    if items.is_empty() || path.ends_with('/') {
        items.push("/");
    }
    items
}

fn encode_segmented_path_component(component: &str, needs_terminator: bool) -> Result<String, String> {
    if component.is_empty() {
        return Err("cannot encode empty path component".to_string());
    }

    let mut best_bits = String::new();
    if let Ok(bits) = encode_spq_value("", component, needs_terminator) {
        best_bits = format!("00{}", bits);
    }
    if let Ok(bits) = encode_uleb128(component) {
        let cand = format!("01{}", bits);
        best_bits = shorter_bits(&best_bits, &cand);
    }
    if let Ok(bits) = encode_fixed6_value(component, needs_terminator) {
        let cand = format!("10{}", bits);
        best_bits = shorter_bits(&best_bits, &cand);
    }
    if let Some(idx) = known_word_index(component) {
        if idx <= 0xff {
            let cand = format!("11{}", int_to_bits(idx, 8));
            best_bits = shorter_bits(&best_bits, &cand);
        }
    }

    if best_bits.is_empty() {
        return Err(format!("cannot encode segmented path component {:?}", component));
    }
    Ok(best_bits)
}

fn encode_segmented_query_component(param: &str, has_more: bool) -> Result<String, String> {
    let (key, value) = param.split_once('=')
        .ok_or_else(|| format!("cannot encode segmented query parameter {:?}", param))?;

    let key_with_terminator = encode_spq_value("?", key, true)
        .map_err(|e| format!("encode query key {:?}: {}", key, e))?;
    let key_no_terminator = encode_spq_value("?", key, has_more)
        .map_err(|e| format!("encode query key {:?}: {}", key, e))?;

    let mut best_bits = String::new();
    if let Ok(bits) = encode_spq_value("=", value, has_more) {
        best_bits = format!("00{}{}", key_with_terminator, bits);
    }
    if let Ok(bits) = encode_uleb128(value) {
        let cand = format!("01{}{}", bits, key_no_terminator);
        best_bits = shorter_bits(&best_bits, &cand);
    }
    if let Ok(bits) = encode_fixed6_value(value, has_more) {
        let cand = format!("10{}{}", key_with_terminator, bits);
        best_bits = shorter_bits(&best_bits, &cand);
    }

    if best_bits.is_empty() {
        return Err(format!("cannot encode segmented query parameter {:?}", param));
    }
    Ok(best_bits)
}

fn encode_spq_value(start_context: &str, value: &str, needs_terminator: bool) -> Result<String, String> {
    let mut s = value.to_string();
    if needs_terminator {
        s.push('|');
    }
    let chars: Vec<&str> = s.split_inclusive(|_| true).collect();
    spq_coder().encode_with_start_context(&chars, start_context)
}

fn encode_fixed6_value(value: &str, needs_terminator: bool) -> Result<String, String> {
    let mut s = value.to_string();
    if needs_terminator {
        s.push('|');
    }
    encode_fixed6(&s)
}

fn shorter_bits(current: &str, candidate: &str) -> String {
    if current.is_empty() || candidate.len() < current.len() {
        candidate.to_string()
    } else {
        current.to_string()
    }
}
