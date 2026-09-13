use super::gf::{gf16, gf256};
use super::reader::decompress_url;
use super::rs::RSEncoder;
use super::rs_decoder::decode_rs_codeword;

pub static K_GAPS_BITS_ORDER_LUT: [usize; 128] = [
    16, 0, 1, 2, 4, 5, 6, 7, 30, 31, 32, 33, 34, 36, 37, 38,
    127, 95, 94, 66, 65, 17, 18, 19, 101, 102, 103, 71, 72, 46, 23, 24,
    114, 115, 116, 117, 83, 84, 56, 57, 118, 119, 120, 85, 86, 87, 58, 59,
    96, 97, 98, 67, 68, 41, 42, 20, 104, 105, 73, 74, 75, 47, 48, 25,
    8, 9, 10, 11, 12, 13, 14, 15, 121, 122, 123, 88, 89, 60, 61, 62,
    124, 125, 126, 91, 92, 93, 64, 39, 100, 69, 70, 43, 44, 21, 22, 3,
    111, 112, 113, 80, 81, 82, 54, 53, 106, 107, 108, 76, 77, 49, 50, 26,
    109, 110, 78, 79, 51, 52, 28, 29, 27, 35, 40, 45, 55, 63, 90, 99,
];

#[derive(Debug, Clone, Copy)]
pub struct FormatParams {
    pub gaps_data_count: usize,
    pub gaps_parity_count: usize,
    pub arcs_data_count: usize,
    pub arcs_parity_count: usize,
}

pub const FORMAT_V0: FormatParams = FormatParams {
    gaps_data_count: 9,
    gaps_parity_count: 4,
    arcs_data_count: 5,
    arcs_parity_count: 2,
};

pub const FORMAT_V1: FormatParams = FormatParams {
    gaps_data_count: 11,
    gaps_parity_count: 2,
    arcs_data_count: 5,
    arcs_parity_count: 2,
};

pub fn encode_payload(payload: &[u8]) -> Result<Vec<bool>, String> {
    // Step 1: FitOptimalVersion - strip leading zeros, pick version
    let mut trimmed = payload;
    while !trimmed.is_empty() && trimmed[0] == 0 {
        trimmed = &trimmed[1..];
    }
    let ver = if trimmed.len() > 14 { 1 } else { 0 };
    let fp = if ver == 0 { FORMAT_V0 } else { FORMAT_V1 };
    let total_data = fp.gaps_data_count + fp.arcs_data_count;

    // Pad back to total_data bytes with leading zeros
    let mut padded = vec![0u8; total_data];
    if trimmed.len() <= total_data {
        let offset = total_data - trimmed.len();
        padded[offset..].copy_from_slice(trimmed);
    } else {
        let offset = trimmed.len() - total_data;
        padded.copy_from_slice(&trimmed[offset..]);
    }

    // Step 2: Scramble - reverse and XOR with 0xa5
    let mut scrambled = vec![0u8; total_data];
    for i in 0..total_data {
        scrambled[i] = padded[total_data - 1 - i] ^ 0xa5;
    }

    // Step 3: Split into gaps and arcs data
    let gaps_data = &scrambled[..fp.gaps_data_count];
    let arcs_data = &scrambled[total_data - fp.arcs_data_count..];

    // Step 4: RS encode gaps (GF(256), fcr=1)
    let gaps_rs = RSEncoder::new(gf256(), fp.gaps_parity_count);
    let gaps_symbols: Vec<usize> = gaps_data.iter().map(|&b| b as usize).collect();
    let gaps_encoded = gaps_rs.encode(&gaps_symbols);
    let mut gaps_bits = blocks_to_bits(&gaps_encoded, 8); // 104 bits

    // Step 5: Gap inversion - invert when zero_count <= 51
    let gap_zeros = gaps_bits.iter().filter(|&&b| !b).count();
    let inverted = if gap_zeros <= 51 {
        for b in gaps_bits.iter_mut() {
            *b = !*b;
        }
        true
    } else {
        false
    };

    // Step 6: Metadata RS encode (GF(16), fcr=0)
    let meta_rs = RSEncoder::new(gf16(), 2);
    let meta_data = vec![
        ver >> 3,
        (if inverted { 1 } else { 0 }) | ((ver & 7) << 1),
    ];
    let meta_encoded = meta_rs.encode(&meta_data);
    let meta_bits = blocks_to_bits(&meta_encoded, 4); // 16 bits

    // Step 7: Arcs RS encode (GF(256), fcr=1)
    let arcs_rs = RSEncoder::new(gf256(), fp.arcs_parity_count);
    let arcs_symbols: Vec<usize> = arcs_data.iter().map(|&b| b as usize).collect();
    let arcs_encoded = arcs_rs.encode(&arcs_symbols);
    let arcs_bits = blocks_to_bits(&arcs_encoded, 8); // 56 bits

    // Step 8: Assemble 128 pre-permutation bits: [meta 16][gaps 104][template 8]
    // 0x2a LSB-first: false, true, false, true, false, true, false, false
    let template_bits = [false, true, false, true, false, true, false, false];
    let mut pre_perm = vec![false; 128];
    pre_perm[0..16].copy_from_slice(&meta_bits);
    pre_perm[16..120].copy_from_slice(&gaps_bits);
    pre_perm[120..128].copy_from_slice(&template_bits);

    let zero_count_128 = pre_perm.iter().filter(|&&b| !b).count();

    // Step 9: LUT permutation
    let total_len = 129 + zero_count_128;
    let mut output = vec![false; total_len];
    for i in 0..128 {
        output[K_GAPS_BITS_ORDER_LUT[i]] = pre_perm[i];
    }

    // Step 10: Append separator (false), arcs, and extra gap bits
    let mut pos = 128;
    output[pos] = false; // separator bit
    pos += 1;

    output[pos..pos + arcs_bits.len()].copy_from_slice(&arcs_bits);
    pos += arcs_bits.len();

    let extra_count = zero_count_128.saturating_sub(arcs_bits.len());
    if extra_count > 0 && extra_count <= gaps_bits.len() {
        output[pos..pos + extra_count].copy_from_slice(&gaps_bits[..extra_count]);
    }

    Ok(output)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedBarcode {
    pub version: usize,
    pub inverted: bool,
    pub payload: Vec<u8>,
    pub url: Option<String>,
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

pub fn decode_barcode(bits: &[bool]) -> Result<DecodedBarcode, String> {
    if bits.len() < 128 {
        return Err(format!("need at least 128 bits, got {}", bits.len()));
    }

    let ring_bits = &bits[..128];

    // Step 1: Reverse LUT permutation
    let mut pre_perm = vec![false; 128];
    for i in 0..128 {
        pre_perm[i] = ring_bits[K_GAPS_BITS_ORDER_LUT[i]];
    }

    // Step 2: Extract metadata (bits 0-15, 4 symbols of 4 bits)
    let meta_codeword = bits_to_symbols(&pre_perm[0..16], 4);
    let meta_syms = decode_rs_codeword(gf16(), &meta_codeword, 2)
        .map_err(|e| format!("decode metadata RS: {}", e))?;

    let ver = (meta_syms[0] << 3) | (meta_syms[1] >> 1);
    let inverted = (meta_syms[1] & 1) == 1;

    if ver == 8 {
        let mut gap_bits = pre_perm[16..120].to_vec();
        if inverted {
            for b in gap_bits.iter_mut() {
                *b = !*b;
            }
        }
        let mut payload = vec![0u8; 13];
        for i in 0..13 {
            let mut v = 0u8;
            for j in 0..8 {
                if gap_bits[i * 8 + j] {
                    v |= 1 << (7 - j);
                }
            }
            payload[i] = v;
        }

        if bits.len() > 128 {
            let color_stream = &bits[128..];
            if color_stream.len() >= 57 {
                let mut arc_bytes = vec![0u8; 7];
                for i in 0..7 {
                    let mut v = 0u8;
                    for j in 0..8 {
                        if color_stream[1 + i * 8 + j] {
                            v |= 1 << (7 - j);
                        }
                    }
                    arc_bytes[i] = v;
                }
                payload.extend_from_slice(&arc_bytes);
            }
        }

        return Ok(DecodedBarcode {
            version: ver,
            inverted,
            payload,
            url: None,
        });
    }

    if ver > 1 {
        return Err(format!("invalid version: {} (expected 0, 1, or 8)", ver));
    }
    let fp = if ver == 0 { FORMAT_V0 } else { FORMAT_V1 };

    // Step 3: Extract gap symbols (bits 16-120, 13 symbols of 8 bits)
    let mut gap_bits = pre_perm[16..120].to_vec();
    if inverted {
        for b in gap_bits.iter_mut() {
            *b = !*b;
        }
    }

    let gap_codeword = bits_to_symbols(&gap_bits, 8);
    let gap_syms = decode_rs_codeword(gf256(), &gap_codeword, fp.gaps_parity_count)
        .map_err(|e| format!("decode gap RS: {}", e))?;
    let data_syms = &gap_syms[..fp.gaps_data_count];

    // Step 4: Arcs recovery if color stream present
    let mut arcs_data = Vec::new();
    if bits.len() > 128 {
        let color_stream = &bits[128..];
        if color_stream.len() >= 57 {
            if color_stream[0] {
                return Err("invalid separator bit: got 1, want 0".to_string());
            }
            let arcs_codeword = bits_to_symbols(&color_stream[1..57], 8);
            let arcs_syms = decode_rs_codeword(gf256(), &arcs_codeword, fp.arcs_parity_count)
                .map_err(|e| format!("decode arcs RS: {}", e))?;
            arcs_data = arcs_syms[..fp.arcs_data_count].iter().map(|&s| s as u8).collect();
        }
    }

    // Step 5: Unscramble
    let total_data = fp.gaps_data_count + fp.arcs_data_count;
    let mut scrambled = vec![0u8; total_data];
    for i in 0..fp.gaps_data_count {
        scrambled[i] = data_syms[i] as u8;
    }
    if arcs_data.len() == fp.arcs_data_count {
        scrambled[fp.gaps_data_count..total_data].copy_from_slice(&arcs_data);
    }

    let mut padded = vec![0u8; total_data];
    for i in 0..total_data {
        padded[total_data - 1 - i] = scrambled[i] ^ 0xa5;
    }

    // Step 6: Build 16-byte payload (right aligned)
    let mut payload = vec![0u8; 16];
    let offset = 16 - total_data;
    payload[offset..].copy_from_slice(&padded);

    let url = decompress_url(&payload).ok();

    Ok(DecodedBarcode {
        version: ver,
        inverted,
        payload,
        url,
    })
}

pub fn decode_payload(bits: &[bool]) -> Result<Vec<u8>, String> {
    decode_barcode(bits).map(|b| b.payload)
}

pub fn blocks_to_bits(symbols: &[usize], bits_per_symbol: usize) -> Vec<bool> {
    let mut bits = Vec::with_capacity(symbols.len() * bits_per_symbol);
    for &sym in symbols {
        for j in (0..bits_per_symbol).rev() {
            bits.push(((sym >> j) & 1) == 1);
        }
    }
    bits
}

pub fn bits_to_symbols(bits: &[bool], bits_per_symbol: usize) -> Vec<usize> {
    let count = bits.len() / bits_per_symbol;
    let mut syms = Vec::with_capacity(count);
    for i in 0..count {
        let mut sym = 0;
        for j in 0..bits_per_symbol {
            sym = (sym << 1) | (if bits[i * bits_per_symbol + j] { 1 } else { 0 });
        }
        syms.push(sym);
    }
    syms
}
