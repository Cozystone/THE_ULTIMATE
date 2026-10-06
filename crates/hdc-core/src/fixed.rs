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

/// log2 of a Q16 fraction num/den (both > 0), in Q16.
pub fn log2_ratio_q16(num: u64, den: u64) -> i64 {
    log2_q16(num.max(1)) - log2_q16(den.max(1))
}

/// 2^(-x) for x >= 0 given in Q16, result in Q16 (0 when x >= 48 bits).
pub fn exp2_neg_q16(x: i64) -> i64 {
    if x <= 0 {
        return Q;
    }
    let ip = x >> 16;
    if ip >= 48 {
        return 0;
    }
    let frac = (x & 0xFFFF) as u64; // fraction in Q16
    // 2^(-f) for f in [0,1): 2^(-f) = 1 / 2^f ; compute 2^f by binary expansion of f
    // using sqrt(2) powers: 2^(1/2), 2^(1/4), ... in Q30
    const ROOTS: [u64; 16] = [
        1518500250, 1276901417, 1170923762, 1121280436, 1097253708, 1085434106, 1079572006, 1076653045,
        1075196444, 1074468868, 1074105258, 1073923493, 1073832616, 1073787178, 1073764459, 1073753100,
    ];
    let mut p: u128 = 1 << 30; // 2^f in Q30
    for (i, r) in ROOTS.iter().enumerate() {
        if frac & (1 << (15 - i)) != 0 {
            p = (p * *r as u128) >> 30;
        }
    }
    // 2^(-f) in Q16 = 2^46 / p(Q30) ; then shift by integer part
    let inv = ((1u128 << 46) / p) as i64;
    inv >> ip
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

/// Posterior of the best of several hypotheses given their log2-likelihoods (Q16), uniform prior.
/// Returns (index of best, posterior in Q16). Integer log-sum-exp.
pub fn best_posterior_q16(loglik: &[i64]) -> Option<(usize, i64)> {
    let (bi, &bmax) = loglik.iter().enumerate().max_by_key(|x| (*x.1, std::cmp::Reverse(x.0)))?;
    let mut z: i64 = 0;
    for &l in loglik {
        z += exp2_neg_q16(bmax - l);
    }
    Some((bi, (Q * Q) / z.max(1)))
}

/// ln(2) in Q16.
pub const LN2_Q16: i64 = 45426;

/// Harmonic number H_n = 1 + 1/2 + ... + 1/n in Q16 nats (exact sum up to 4096, asymptotic beyond).
pub fn harmonic_q16(n: u64) -> i64 {
    if n == 0 {
        return 0;
    }
    if n <= 4096 {
        let mut s: i64 = 0;
        for k in 1..=n {
            s += (Q << 16) / k as i64;
        }
        return s >> 16;
    }
    // H_n ~ ln n + gamma + 1/(2n); gamma = 0.5772156649 -> 37829 in Q16
    let ln_n = log2_q16(n) * LN2_Q16 / Q;
    ln_n + 37829 + Q / (2 * n as i64)
}

/// Expected information gain (mutual information between the next outcome and the unknown
/// outcome distribution) for a categorical with a Dirichlet(counts + 1) posterior, in Q16 bits.
/// EIG = H[predictive] - E_theta[H(outcome | theta)], with
/// E[H] = H_A - sum_i (a_i / A) H_{a_i} (nats), a_i = counts_i + 1, A = sum a_i.
pub fn dirichlet_eig_q16(counts: &[u64]) -> i64 {
    if counts.len() < 2 {
        return 0;
    }
    let a: Vec<u64> = counts.iter().map(|&c| c + 1).collect();
    let big_a: u64 = a.iter().sum();
    let h_pred_bits = entropy_q16(&a);
    let mut e_h_nats = harmonic_q16(big_a);
    for &ai in &a {
        e_h_nats -= (ai as i128 * harmonic_q16(ai) as i128 / big_a as i128) as i64;
    }
    let e_h_bits = e_h_nats * Q / LN2_Q16;
    (h_pred_bits - e_h_bits).max(0)
}
