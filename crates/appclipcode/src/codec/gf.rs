#[derive(Debug, Clone)]
pub struct GaloisField {
    pub size: usize,
    pub gen_poly: usize,
    pub fcr_base: usize,
    pub exp_tbl: Vec<usize>,
    pub log_tbl: Vec<usize>,
}

impl GaloisField {
    pub fn new(primitive: usize, size: usize, gen_base: usize) -> Self {
        let mut exp_tbl = vec![0; size * 2];
        let mut log_tbl = vec![0; size];

        let mut x = 1;
        for i in 0..size {
            exp_tbl[i] = x;
            log_tbl[x] = i;
            x <<= 1;
            if x >= size {
                x ^= primitive;
                x &= size - 1;
            }
        }
        for i in size..(size * 2) {
            exp_tbl[i] = exp_tbl[i - size + 1];
        }

        Self {
            size,
            gen_poly: primitive,
            fcr_base: gen_base,
            exp_tbl,
            log_tbl,
        }
    }

    #[inline]
    pub fn exp(&self, a: usize) -> usize {
        self.exp_tbl[a]
    }

    #[inline]
    pub fn log(&self, a: usize) -> usize {
        self.log_tbl[a]
    }

    #[inline]
    pub fn multiply(&self, a: usize, b: usize) -> usize {
        if a == 0 || b == 0 {
            0
        } else {
            self.exp_tbl[self.log_tbl[a] + self.log_tbl[b]]
        }
    }

    #[inline]
    pub fn inverse(&self, a: usize) -> usize {
        self.exp_tbl[self.size - 1 - self.log_tbl[a]]
    }

    pub fn pow(&self, mut exponent: isize) -> usize {
        let order = (self.size - 1) as isize;
        exponent %= order;
        if exponent < 0 {
            exponent += order;
        }
        self.exp(exponent as usize)
    }
}

pub fn gf16() -> &'static GaloisField {
    use std::sync::OnceLock;
    static GF: OnceLock<GaloisField> = OnceLock::new();
    GF.get_or_init(|| GaloisField::new(0x13, 16, 0))
}

pub fn gf256() -> &'static GaloisField {
    use std::sync::OnceLock;
    static GF: OnceLock<GaloisField> = OnceLock::new();
    GF.get_or_init(|| GaloisField::new(0x11D, 256, 1))
}
