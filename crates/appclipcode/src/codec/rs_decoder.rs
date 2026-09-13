use super::gf::GaloisField;

pub fn decode_rs_codeword(
    gf: &GaloisField,
    codeword: &[usize],
    num_parity: usize,
) -> Result<Vec<usize>, String> {
    if num_parity == 0 {
        return Ok(codeword.to_vec());
    }

    let syndromes = rs_syndromes(gf, codeword, num_parity);
    if rs_syndromes_zero(&syndromes) {
        return Ok(codeword.to_vec());
    }

    if let Some(corrected) = correct_single_rs_error(gf, codeword, &syndromes) {
        return Ok(corrected);
    }

    if num_parity >= 4 {
        if let Some(corrected) = correct_double_rs_error(gf, codeword, &syndromes) {
            return Ok(corrected);
        }
    }

    Err("reed-solomon decode failed".to_string())
}

fn rs_syndromes(gf: &GaloisField, codeword: &[usize], num_parity: usize) -> Vec<usize> {
    let mut syndromes = vec![0; num_parity];
    for si in 0..num_parity {
        let root = gf.pow((gf.fcr_base + si) as isize);
        let mut acc = 0;
        for &sym in codeword {
            acc = gf.multiply(acc, root) ^ sym;
        }
        syndromes[si] = acc;
    }
    syndromes
}

fn rs_syndromes_zero(syndromes: &[usize]) -> bool {
    syndromes.iter().all(|&s| s == 0)
}

fn rs_error_term(gf: &GaloisField, codeword_len: usize, pos: usize, syndrome_index: usize) -> usize {
    let exponent = (gf.fcr_base + syndrome_index) as isize * (codeword_len - 1 - pos) as isize;
    gf.pow(exponent)
}

fn correct_single_rs_error(
    gf: &GaloisField,
    codeword: &[usize],
    syndromes: &[usize],
) -> Option<Vec<usize>> {
    let n = codeword.len();
    for pos in 0..n {
        let a0 = rs_error_term(gf, n, pos, 0);
        if a0 == 0 {
            continue;
        }
        let err_mag = gf.multiply(syndromes[0], gf.inverse(a0));
        if err_mag == 0 {
            continue;
        }

        let mut ok = true;
        for (si, &syndrome) in syndromes.iter().enumerate() {
            if gf.multiply(err_mag, rs_error_term(gf, n, pos, si)) != syndrome {
                ok = false;
                break;
            }
        }
        if !ok {
            continue;
        }

        let mut out = codeword.to_vec();
        out[pos] ^= err_mag;
        if rs_syndromes_zero(&rs_syndromes(gf, &out, syndromes.len())) {
            return Some(out);
        }
    }
    None
}

fn correct_double_rs_error(
    gf: &GaloisField,
    codeword: &[usize],
    syndromes: &[usize],
) -> Option<Vec<usize>> {
    let n = codeword.len();
    if syndromes.len() < 2 {
        return None;
    }

    for p in 0..n {
        for q in (p + 1)..n {
            let a0 = rs_error_term(gf, n, p, 0);
            let b0 = rs_error_term(gf, n, q, 0);
            let a1 = rs_error_term(gf, n, p, 1);
            let b1 = rs_error_term(gf, n, q, 1);

            let det = gf.multiply(a0, b1) ^ gf.multiply(a1, b0);
            if det == 0 {
                continue;
            }

            let inv_det = gf.inverse(det);
            let term_p = gf.multiply(syndromes[0], b1) ^ gf.multiply(syndromes[1], b0);
            let err_p = gf.multiply(term_p, inv_det);

            let term_q = gf.multiply(a0, syndromes[1]) ^ gf.multiply(a1, syndromes[0]);
            let err_q = gf.multiply(term_q, inv_det);

            if err_p == 0 || err_q == 0 {
                continue;
            }

            let mut ok = true;
            for (si, &syndrome) in syndromes.iter().enumerate() {
                let reconstructed = gf.multiply(err_p, rs_error_term(gf, n, p, si))
                    ^ gf.multiply(err_q, rs_error_term(gf, n, q, si));
                if reconstructed != syndrome {
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }

            let mut out = codeword.to_vec();
            out[p] ^= err_p;
            out[q] ^= err_q;
            if rs_syndromes_zero(&rs_syndromes(gf, &out, syndromes.len())) {
                return Some(out);
            }
        }
    }

    None
}
