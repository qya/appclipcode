use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use super::huffman::HuffmanCoder;

pub static H_TRIE_ZLIB: &[u8] = include_bytes!("../../data/h.trie.zlib");
pub static CPQ_TRIE_ZLIB: &[u8] = include_bytes!("../../data/cpq.trie.zlib");
pub static SPQ_TRIE_ZLIB: &[u8] = include_bytes!("../../data/spq.trie.zlib");

pub static HOST_SYMBOLS: [&str; 39] = [
    "-", ".", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l",
    "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x",
    "y", "z", "|",
];

pub static CPQ_SYMBOLS: [&str; 75] = [
    "#", "%", "&", "+", ",", "-", ".", "/",
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    ":", ";", "=", "?",
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
    "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X",
    "Y", "Z",
    "_",
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l",
    "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x",
    "y", "z",
];

pub static SPQ_SYMBOLS: [&str; 71] = [
    "&", "+", "-", ".", "/",
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    "=", "?",
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
    "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X",
    "Y", "Z",
    "_",
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l",
    "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x",
    "y", "z", "|",
];

pub struct PackedHuffmanTrie {
    data: Vec<u8>,
    pub symbols: &'static [&'static str],
    pub num_symbols: usize,
    pub max_depth: usize,
    pub symbol_index_bits: usize,
    pub shape_bits_per_node: usize,
    pub leaf_bits_per_node: usize,
    pub leaf_bit_offset: usize,
}

impl PackedHuffmanTrie {
    pub fn new(compressed_data: &[u8], symbols: &'static [&'static str]) -> Self {
        let decompressed = miniz_oxide::inflate::decompress_to_vec_zlib(compressed_data)
            .expect("failed to decompress trie data");
        let num_symbols = symbols.len();
        let symbol_index_bits = (num_symbols as f64).log2().ceil() as usize;
        let shape_bits_per_node = num_symbols * 2 - 1;
        let expected_nodes = 1 + num_symbols + num_symbols * num_symbols;
        let leaf_bits_per_node = num_symbols * symbol_index_bits;
        let shape_bytes = (expected_nodes * shape_bits_per_node + 7) / 8;
        let leaf_bit_offset = shape_bytes * 8;
        let expected_size = shape_bytes + (expected_nodes * leaf_bits_per_node + 7) / 8;
        assert_eq!(
            decompressed.len(),
            expected_size,
            "trie unpacked length mismatch: expected {}, got {}",
            expected_size,
            decompressed.len()
        );

        Self {
            data: decompressed,
            symbols,
            num_symbols,
            max_depth: 2,
            symbol_index_bits,
            shape_bits_per_node,
            leaf_bits_per_node,
            leaf_bit_offset,
        }
    }

    #[inline]
    fn read_bit(&self, bit_offset: usize) -> bool {
        let byte = self.data[bit_offset >> 3];
        ((byte >> (7 - (bit_offset & 7))) & 1) == 1
    }

    #[inline]
    fn read_bits(&self, bit_offset: usize, count: usize) -> usize {
        let mut value = 0;
        for i in 0..count {
            let b = self.read_bit(bit_offset + i);
            value = (value << 1) | (b as usize);
        }
        value
    }

    pub fn build_coder(&self, node_offset: usize) -> HuffmanCoder {
        let mut codes = vec![String::new(); self.num_symbols];
        let mut shape_bit_offset = node_offset * self.shape_bits_per_node;
        let mut leaf_bit_offset = self.leaf_bit_offset + node_offset * self.leaf_bits_per_node;

        fn walk(
            trie: &PackedHuffmanTrie,
            shape_off: &mut usize,
            leaf_off: &mut usize,
            prefix: &str,
            codes: &mut [String],
        ) {
            let is_leaf = trie.read_bit(*shape_off);
            *shape_off += 1;

            if is_leaf {
                let symbol_index = trie.read_bits(*leaf_off, trie.symbol_index_bits);
                *leaf_off += trie.symbol_index_bits;
                codes[symbol_index] = if prefix.is_empty() { "0".to_string() } else { prefix.to_string() };
                return;
            }

            let mut left_prefix = String::with_capacity(prefix.len() + 1);
            left_prefix.push_str(prefix);
            left_prefix.push('0');
            walk(trie, shape_off, leaf_off, &left_prefix, codes);

            let mut right_prefix = String::with_capacity(prefix.len() + 1);
            right_prefix.push_str(prefix);
            right_prefix.push('1');
            walk(trie, shape_off, leaf_off, &right_prefix, codes);
        }

        walk(self, &mut shape_bit_offset, &mut leaf_bit_offset, "", &mut codes);

        HuffmanCoder {
            codes,
            num_symbols: self.num_symbols,
        }
    }

    #[inline]
    pub fn child_offset(&self, parent_offset: usize, symbol_index: usize) -> usize {
        self.num_symbols * parent_offset + 1 + symbol_index
    }
}

pub struct MultiContextHuffmanCoder {
    pub trie: PackedHuffmanTrie,
    cache: RwLock<HashMap<usize, HuffmanCoder>>,
}

impl MultiContextHuffmanCoder {
    pub fn new(trie: PackedHuffmanTrie) -> Self {
        Self {
            trie,
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn coder_for_node(&self, node_offset: usize) -> HuffmanCoder {
        {
            let read_guard = self.cache.read().unwrap();
            if let Some(coder) = read_guard.get(&node_offset) {
                return coder.clone();
            }
        }

        let coder = self.trie.build_coder(node_offset);
        let mut write_guard = self.cache.write().unwrap();
        write_guard.insert(node_offset, coder.clone());
        coder
    }

    #[inline]
    pub fn symbol_index(&self, sym: &str) -> Option<usize> {
        self.trie.symbols.iter().position(|&s| s == sym)
    }

    pub fn encode(&self, syms: &[&str]) -> Result<String, String> {
        self.encode_with_start_context(syms, "")
    }

    pub fn encode_with_start_context(&self, syms: &[&str], start_ctx: &str) -> Result<String, String> {
        let mut node_offset = 0;
        let mut depth = 0;

        for c in start_ctx.chars() {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            let idx = self.symbol_index(s)
                .ok_or_else(|| format!("unknown start context symbol: {:?}", s))?;
            let (n, d) = self.advance_context(node_offset, depth, idx);
            node_offset = n;
            depth = d;
        }

        let mut bits = String::new();
        for &sym in syms {
            let idx = self.symbol_index(sym)
                .ok_or_else(|| format!("unknown symbol: {:?}", sym))?;
            let hc = self.coder_for_node(node_offset);
            if !hc.can_encode(idx) {
                return Err(format!("cannot encode symbol {:?} at context node {}", sym, node_offset));
            }
            bits.push_str(hc.encode(idx));
            let (n, d) = self.advance_context(node_offset, depth, idx);
            node_offset = n;
            depth = d;
        }

        Ok(bits)
    }

    #[inline]
    pub fn advance_context(&self, node_offset: usize, depth: usize, symbol_index: usize) -> (usize, usize) {
        if depth < self.trie.max_depth {
            (self.trie.child_offset(node_offset, symbol_index), depth + 1)
        } else {
            let prev_sym_idx = (node_offset - 1) % self.trie.num_symbols;
            (self.trie.child_offset(1 + prev_sym_idx, symbol_index), depth)
        }
    }
}

pub fn host_coder() -> &'static MultiContextHuffmanCoder {
    static CODER: OnceLock<MultiContextHuffmanCoder> = OnceLock::new();
    CODER.get_or_init(|| {
        let trie = PackedHuffmanTrie::new(H_TRIE_ZLIB, &HOST_SYMBOLS);
        MultiContextHuffmanCoder::new(trie)
    })
}

pub fn cpq_coder() -> &'static MultiContextHuffmanCoder {
    static CODER: OnceLock<MultiContextHuffmanCoder> = OnceLock::new();
    CODER.get_or_init(|| {
        let trie = PackedHuffmanTrie::new(CPQ_TRIE_ZLIB, &CPQ_SYMBOLS);
        MultiContextHuffmanCoder::new(trie)
    })
}

pub fn spq_coder() -> &'static MultiContextHuffmanCoder {
    static CODER: OnceLock<MultiContextHuffmanCoder> = OnceLock::new();
    CODER.get_or_init(|| {
        let trie = PackedHuffmanTrie::new(SPQ_TRIE_ZLIB, &SPQ_SYMBOLS);
        MultiContextHuffmanCoder::new(trie)
    })
}
