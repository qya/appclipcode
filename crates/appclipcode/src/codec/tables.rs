use std::sync::OnceLock;
use super::huffman::HuffmanCoder;

pub static HUFFMAN_TLDS: [(&str, u16); 20] = [
    (".au", 0x0513),
    (".br", 0x04ad),
    (".ca", 0x0482),
    (".cn", 0x0cb7),
    (".com", 0xfffe),
    (".de", 0x1163),
    (".edu", 0x03c1),
    (".fr", 0x059d),
    (".ga", 0x0346),
    (".in", 0x03d5),
    (".info", 0x0449),
    (".it", 0x062c),
    (".jp", 0x08e2),
    (".net", 0x1766),
    (".nl", 0x0598),
    (".org", 0x26f6),
    (".pl", 0x0352),
    (".ru", 0x0fed),
    (".uk", 0x0c86),
    (".us", 0x0361),
];

pub fn tld_huffman_coder() -> &'static HuffmanCoder {
    static CODER: OnceLock<HuffmanCoder> = OnceLock::new();
    CODER.get_or_init(|| {
        let freqs: Vec<u16> = HUFFMAN_TLDS.iter().map(|(_, f)| *f).collect();
        let syms: Vec<&str> = HUFFMAN_TLDS.iter().map(|(s, _)| *s).collect();
        HuffmanCoder::new(&freqs, &syms)
    })
}

pub fn huffman_tld_index(tld: &str) -> Option<usize> {
    HUFFMAN_TLDS.iter().position(|(s, _)| *s == tld)
}

pub static FIXED_TLDS: [(&str, usize); 113] = [
    (".ae", 54), (".ai", 57), (".am", 68), (".app", 58), (".ar", 33), (".at", 7),
    (".be", 6), (".bid", 93), (".bike", 111), (".biz", 17), (".business", 110),
    (".by", 48), (".cc", 27), (".center", 98), (".cf", 13), (".ch", 2),
    (".cl", 36), (".cloud", 66), (".club", 29), (".cm", 94), (".company", 86),
    (".cz", 9), (".digital", 97), (".dk", 30), (".do", 74), (".es", 1),
    (".estate", 113), (".eu", 3), (".fi", 38), (".fun", 64), (".gl", 107),
    (".global", 85), (".gov", 10), (".gr", 12), (".gt", 90), (".help", 106),
    (".hk", 40), (".host", 87), (".hu", 24), (".id", 21), (".ie", 31),
    (".il", 42), (".int", 83), (".io", 4), (".is", 59), (".jobs", 70),
    (".kr", 14), (".kz", 47), (".life", 71), (".live", 53), (".loan", 112),
    (".ltd", 100), (".lu", 67), (".ly", 73), (".md", 76), (".me", 16),
    (".media", 79), (".mo", 95), (".mobi", 56), (".museum", 108), (".mx", 22),
    (".my", 39), (".name", 61), (".network", 65), (".news", 60), (".no", 34),
    (".nu", 45), (".nz", 25), (".online", 35), (".ph", 52), (".pk", 49),
    (".plus", 99), (".pm", 109), (".pt", 43), (".pub", 105), (".py", 91),
    (".qa", 84), (".ro", 26), (".se", 19), (".services", 101), (".sg", 46),
    (".shop", 77), (".site", 18), (".sk", 41), (".so", 102), (".space", 55),
    (".store", 78), (".stream", 89), (".su", 50), (".support", 104),
    (".tech", 62), (".tel", 96), (".th", 44), (".tk", 37), (".tn", 75),
    (".to", 51), (".top", 28), (".tr", 20), (".travel", 81), (".tt", 103),
    (".tv", 11), (".tw", 15), (".ua", 8), (".video", 92), (".vip", 63),
    (".vn", 5), (".wang", 23), (".website", 69), (".wiki", 88), (".win", 72),
    (".work", 82), (".world", 80), (".za", 32),
];

pub fn fixed_tld_index(tld: &str) -> Option<usize> {
    FIXED_TLDS.iter().find(|(s, _)| *s == tld).map(|(_, idx)| *idx)
}

pub fn fixed_tld_by_index(idx: usize) -> Option<&'static str> {
    FIXED_TLDS.iter().find(|(_, i)| *i == idx).map(|(s, _)| *s)
}

pub static KNOWN_WORDS: [(&str, usize); 156] = [
    ("about", 0), ("access", 1), ("account", 2), ("add", 3), ("app", 4),
    ("archives", 5), ("article", 6), ("attraction", 7), ("author", 8), ("bag", 9),
    ("biz", 10), ("book", 11), ("brand", 12), ("brands", 13), ("browse", 14),
    ("buy", 15), ("cancel", 16), ("cart", 17), ("cat", 18), ("catalog", 19),
    ("category", 20), ("categories", 21), ("channel", 22), ("charts", 23),
    ("checkin", 24), ("checkout", 25), ("collection", 26), ("collections", 27),
    ("company", 28), ("compare", 29), ("connect", 30), ("contact", 31),
    ("content", 32), ("contents", 33), ("cost", 34), ("coupons", 35),
    ("create", 36), ("data", 37), ("demo", 38), ("destinations", 39),
    ("detail", 40), ("discover", 41), ("download", 42), ("entry", 43),
    ("event", 44), ("events", 45), ("explore", 46), ("faq", 47),
    ("fetch", 48), ("finance", 49), ("find", 50), ("food", 51),
    ("fund", 52), ("game", 53), ("gift", 54), ("goods", 55),
    ("guide", 56), ("health", 57), ("help", 58), ("home", 59),
    ("hotel", 60), ("hotels", 61), ("id", 62), ("index", 63),
    ("info", 64), ("item", 65), ("item_id", 66), ("join", 67),
    ("lifestyle", 68), ("list", 69), ("listen", 70), ("live", 71),
    ("local", 72), ("location", 73), ("locations", 74), ("locator", 75),
    ("login", 76), ("manage", 77), ("menu", 78), ("more", 79),
    ("music", 80), ("name", 81), ("news", 82), ("note", 83),
    ("open", 84), ("order", 85), ("overview", 86), ("park", 87),
    ("part", 88), ("pay", 89), ("payment", 90), ("payments", 91),
    ("play", 92), ("post", 93), ("posts", 94), ("preview", 95),
    ("product", 96), ("product_id", 97), ("products", 98), ("profile", 99),
    ("promotion", 100), ("purchase", 101), ("rate", 102), ("recipe", 103),
    ("recipes", 104), ("reservation", 105), ("reservations", 106), ("reserve", 107),
    ("retail", 108), ("review", 109), ("rewards", 110), ("sale", 111),
    ("scan", 112), ("schedule", 113), ("search", 114), ("sell", 115),
    ("send", 116), ("service", 117), ("share", 118), ("shop", 119),
    ("show", 120), ("showtime", 121), ("site", 122), ("song", 123),
    ("special", 124), ("stations", 125), ("status", 126), ("store", 127),
    ("store-locator", 128), ("stores", 129), ("stories", 130), ("story", 131),
    ("tag", 132), ("tags", 133), ("terms", 134), ("tickets", 135),
    ("tips", 136), ("title", 137), ("today", 138), ("top", 139),
    ("topic", 140), ("tours", 141), ("track", 142), ("transaction", 143),
    ("travel", 144), ("try", 145), ("update", 146), ("upload", 147),
    ("use", 148), ("user", 149), ("vehicles", 150), ("video", 151),
    ("view", 152), ("visit", 153), ("watch", 154), ("wiki", 155),
];

pub fn known_word_index(word: &str) -> Option<usize> {
    KNOWN_WORDS.iter().find(|(w, _)| *w == word).map(|(_, idx)| *idx)
}

pub fn known_word_by_index(idx: usize) -> Option<&'static str> {
    KNOWN_WORDS.iter().find(|(_, i)| *i == idx).map(|(w, _)| *w)
}

pub static FIXED6_ALPHABET: &[u8] = b".0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz|";

pub fn encode_fixed6(value: &str) -> Result<String, String> {
    let mut bits = String::with_capacity(value.len() * 6);
    for b in value.bytes() {
        if let Some(pos) = FIXED6_ALPHABET.iter().position(|&x| x == b) {
            bits.push_str(&int_to_bits(pos, 6));
        } else {
            return Err(format!("symbol {:?} not encodable by fixed6", b as char));
        }
    }
    Ok(bits)
}

pub fn decode_fixed6(bits: &[bool], pos: &mut usize, max_symbols: usize) -> Result<String, String> {
    let mut res = String::new();
    while *pos + 6 <= bits.len() && res.len() < max_symbols {
        let mut idx = 0;
        for i in 0..6 {
            idx = (idx << 1) | (if bits[*pos + i] { 1 } else { 0 });
        }
        *pos += 6;
        if idx >= FIXED6_ALPHABET.len() {
            return Err(format!("invalid fixed6 index {}", idx));
        }
        let ch = FIXED6_ALPHABET[idx] as char;
        res.push(ch);
        if ch == '|' {
            break;
        }
    }
    Ok(res)
}

pub fn encode_uleb128(value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Err("empty numeric value".to_string());
    }
    for b in value.bytes() {
        if !b.is_ascii_digit() {
            return Err(format!("non-decimal digit in {:?}", value));
        }
    }

    if value == "0" {
        return Ok("00000000".to_string());
    }

    let mut digits: Vec<u8> = value.bytes().map(|b| b - b'0').collect();
    let mut bytes = Vec::new();

    while !digits.is_empty() {
        let mut rem = 0u32;
        let mut next_digits = Vec::with_capacity(digits.len());
        for &d in &digits {
            let cur = rem * 10 + d as u32;
            let q = cur / 128;
            rem = cur % 128;
            if !next_digits.is_empty() || q > 0 {
                next_digits.push(q as u8);
            }
        }
        digits = next_digits;
        let mut b = rem as u8;
        if !digits.is_empty() {
            b |= 0x80;
        }
        bytes.push(b);
    }

    let mut out = String::with_capacity(bytes.len() * 8);
    for b in bytes {
        out.push_str(&int_to_bits(b as usize, 8));
    }
    Ok(out)
}

pub fn decode_uleb128(bits: &[bool], pos: &mut usize) -> Result<String, String> {
    let mut bytes = Vec::new();
    loop {
        if *pos + 8 > bits.len() {
            return Err("unexpected end of bits while decoding ULEB128".to_string());
        }
        let mut b = 0u8;
        for i in 0..8 {
            b = (b << 1) | (if bits[*pos + i] { 1 } else { 0 });
        }
        *pos += 8;
        bytes.push(b & 0x7f);
        if (b & 0x80) == 0 {
            break;
        }
    }

    let mut dec_digits = vec![0u8];
    for &b in bytes.iter().rev() {
        let mut carry = b as u32;
        for d in dec_digits.iter_mut() {
            let cur = (*d as u32) * 128 + carry;
            *d = (cur % 10) as u8;
            carry = cur / 10;
        }
        while carry > 0 {
            dec_digits.push((carry % 10) as u8);
            carry /= 10;
        }
    }

    let s: String = dec_digits.iter().rev().map(|&d| (b'0' + d) as char).collect();
    Ok(s)
}

#[inline]
pub fn int_to_bits(val: usize, num_bits: usize) -> String {
    let mut s = String::with_capacity(num_bits);
    for i in (0..num_bits).rev() {
        if ((val >> i) & 1) == 1 {
            s.push('1');
        } else {
            s.push('0');
        }
    }
    s
}
