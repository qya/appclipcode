use super::gf::GaloisField;

#[derive(Debug, Clone)]
pub struct RSEncoder<'a> {
    gf: &'a GaloisField,
    gen_poly: Vec<usize>,
    num_parity: usize,
}

impl<'a> RSEncoder<'a> {
    pub fn new(gf: &'a GaloisField, num_parity: usize) -> Self {
        let mut gen = vec![1];
        for i in 0..num_parity {
            let root = gf.exp(gf.fcr_base + i);
            let mut new_gen = vec![0; gen.len() + 1];
            new_gen[..gen.len()].copy_from_slice(&gen);
            for j in 0..gen.len() {
                new_gen[j + 1] ^= gf.multiply(gen[j], root);
            }
            gen = new_gen;
        }

        Self {
            gf,
            gen_poly: gen,
            num_parity,
        }
    }

    pub fn encode(&self, data: &[usize]) -> Vec<usize> {
        let n = data.len() + self.num_parity;
        let mut result = vec![0; n];
        result[..data.len()].copy_from_slice(data);

        for i in 0..data.len() {
            let coef = result[i];
            if coef != 0 {
                for j in 1..=self.num_parity {
                    result[i + j] ^= self.gf.multiply(self.gen_poly[j], coef);
                }
            }
        }

        result[..data.len()].copy_from_slice(data);
        result
    }
}
