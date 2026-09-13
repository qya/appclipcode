use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HuffmanNode {
    pub freq: u32,
    pub symbol_index: Option<usize>,
    pub symbol: String,
    pub left: Option<Box<HuffmanNode>>,
    pub right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    pub fn leftmost_leaf_symbol(&self) -> &str {
        let mut curr = self;
        loop {
            if let Some(ref left) = curr.left {
                curr = left;
            } else if let Some(ref right) = curr.right {
                curr = right;
            } else {
                return &curr.symbol;
            }
        }
    }
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.freq.cmp(&self.freq) {
            Ordering::Equal => {
                let sym_self = self.leftmost_leaf_symbol();
                let sym_other = other.leftmost_leaf_symbol();
                sym_other.cmp(sym_self)
            }
            ord => ord,
        }
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct HuffmanCoder {
    pub codes: Vec<String>,
    pub num_symbols: usize,
}

impl HuffmanCoder {
    pub fn new(freqs: &[u16], symbols: &[&str]) -> Self {
        let n = freqs.len();
        let mut hc = HuffmanCoder {
            codes: vec![String::new(); n],
            num_symbols: n,
        };

        let mut heap = BinaryHeap::new();
        for (i, &f) in freqs.iter().enumerate() {
            if f > 0 {
                let sym = if i < symbols.len() { symbols[i] } else { "" };
                heap.push(HuffmanNode {
                    freq: f as u32,
                    symbol_index: Some(i),
                    symbol: sym.to_string(),
                    left: None,
                    right: None,
                });
            }
        }

        if heap.is_empty() {
            return hc;
        }

        if heap.len() == 1 {
            let single = heap.pop().unwrap();
            hc.codes[single.symbol_index.unwrap()] = "0".to_string();
            return hc;
        }

        while heap.len() > 1 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            let combined = HuffmanNode {
                freq: left.freq + right.freq,
                symbol_index: None,
                symbol: String::new(),
                left: Some(Box::new(left)),
                right: Some(Box::new(right)),
            };
            heap.push(combined);
        }

        let root = heap.pop().unwrap();
        hc.build_codes(&root, "");
        hc
    }

    fn build_codes(&mut self, node: &HuffmanNode, prefix: &str) {
        if node.left.is_none() && node.right.is_none() {
            let code = if prefix.is_empty() { "0" } else { prefix };
            if let Some(idx) = node.symbol_index {
                self.codes[idx] = code.to_string();
            }
            return;
        }

        if let Some(ref left) = node.left {
            let mut next = String::with_capacity(prefix.len() + 1);
            next.push_str(prefix);
            next.push('0');
            self.build_codes(left, &next);
        }
        if let Some(ref right) = node.right {
            let mut next = String::with_capacity(prefix.len() + 1);
            next.push_str(prefix);
            next.push('1');
            self.build_codes(right, &next);
        }
    }

    #[inline]
    pub fn encode(&self, symbol_index: usize) -> &str {
        if symbol_index < self.codes.len() {
            &self.codes[symbol_index]
        } else {
            ""
        }
    }

    #[inline]
    pub fn can_encode(&self, symbol_index: usize) -> bool {
        symbol_index < self.codes.len() && !self.codes[symbol_index].is_empty()
    }

    pub fn decode(&self, data: &[bool], pos: &mut usize) -> Result<usize, String> {
        for (i, code) in self.codes.iter().enumerate() {
            if code.is_empty() {
                continue;
            }
            let len = code.len();
            if *pos + len <= data.len() {
                let mut match_found = true;
                for (j, b) in code.bytes().enumerate() {
                    let expected = b == b'1';
                    if data[*pos + j] != expected {
                        match_found = false;
                        break;
                    }
                }
                if match_found {
                    *pos += len;
                    return Ok(i);
                }
            }
        }
        Err(format!("no matching Huffman code at bit position {}", *pos))
    }
}
