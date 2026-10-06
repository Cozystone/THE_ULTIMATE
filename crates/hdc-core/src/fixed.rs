//! Integer fixed-point information measures (Q16 = value * 65536). No floats.

pub const Q: i64 = 1 << 16;

/// log2(n) in Q16 for n >= 1. Integer part from leading zeros, fraction by repeated squaring.
pub fn log2_q16(n: u64) -> i64 {
    assert!(n >= 1, "log2 of zero");
    let ip = 63 - n.leading_zeros() as i64;
    // normalise mantissa m in [1,2) as Q62
    let mut m: u128 = if ip >= 62 { (n as u128) >> (ip - 62) } else { (n as u128) << (62 - ip) };
    let one: u128 = 1u128 << 62;
    let mut frac: i64 = 0;
    for bit in (0..16).rev() {
        m = (m * m) >> 62;
        if m >= 2 * one {
            m >>= 1;
            frac |= 1 << bit;
        }
    }
    ip * Q + frac
}

/// `n * log2(n)` in Q16, with 0*log 0 = 0.
pub fn nlog2n_q16(n: u64) -> i64 {
    if n == 0 {
        0
    } else {
        n as i64 * log2_q16(n)
    }
}

/// Shannon entropy (bits, Q16) of a count histogram.
pub fn entropy_q16(counts: &[u64]) -> i64 {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0;
    }
    let s: i64 = counts.iter().map(|&c| nlog2n_q16(c)).sum();
    // H = log2 N - (1/N) sum c log2 c
    log2_q16(total) - s / total as i64
}

/// Entropy of the Laplace-smoothed predictive distribution over `k` outcomes
/// (counts + 1 each). `counts.len()` must be k.
pub fn predictive_entropy_q16(counts: &[u64]) -> i64 {
    let sm: Vec<u64> = counts.iter().map(|&c| c + 1).collect();
    entropy_q16(&sm)
}

/// Expected reduction of predictive entropy after one more observation (Q16 bits),
/// Laplace-smoothed categorical over `counts.len()` outcomes. Integer arithmetic only.
pub fn expected_info_gain_q16(counts: &[u64]) -> i64 {
    let k = counts.len();
    if k == 0 {
        return 0;
    }
    let sm: Vec<u64> = counts.iter().map(|&c| c + 1).collect();
    let total: u64 = sm.iter().sum();
    let h0 = entropy_q16(&sm);
    let mut exp_h: i64 = 0; // sum p_i * H_i, p_i = sm_i / total
    let mut buf = sm.clone();
    for i in 0..k {
        buf[i] += 1;
        let hi = entropy_q16(&buf);
        buf[i] -= 1;
        exp_h += hi * sm[i] as i64;
    }
    h0 - exp_h / total as i64
}

/// Binary entropy of p = num/den in Q16 bits.
pub fn binary_entropy_q16(num: u64, den: u64) -> i64 {
    if den == 0 || num == 0 || num >= den {
        return 0;
    }
    entropy_q16(&[num, den - num])
}

/// Integer ratio in Q16.
pub fn ratio_q16(num: u64, den: u64) -> i64 {
    if den == 0 {
        0
    } else {
        ((num as i128 * Q as i128) / den as i128) as i64
    }
}
