//! Deterministic integer randomness. No external crates (D005).

/// FNV-1a 64-bit. Stable across Rust versions, unlike `DefaultHasher`.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Mix two 64-bit values into one (for deriving sub-seeds).
pub fn mix64(a: u64, b: u64) -> u64 {
    let mut s = SplitMix64(a ^ b.rotate_left(32) ^ 0x9e37_79b9_7f4a_7c15);
    s.next_u64()
}

/// SplitMix64, used to seed xoshiro.
#[derive(Clone, Debug)]
pub struct SplitMix64(pub u64);

impl SplitMix64 {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

/// xoshiro256** generator.
#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut sm = SplitMix64(seed);
        let s = [sm.next_u64(), sm.next_u64(), sm.next_u64(), sm.next_u64()];
        Rng { s }
    }

    /// Seed from a seed and a symbol name, deterministic.
    pub fn for_symbol(seed: u64, symbol: &str) -> Self {
        Rng::new(mix64(seed, fnv1a64(symbol.as_bytes())))
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform integer in `0..n` (n > 0), Lemire's multiply-shift with rejection.
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0);
        loop {
            let x = self.next_u64();
            let m = (x as u128) * (n as u128);
            let lo = m as u64;
            if lo >= n.wrapping_neg() % n {
                return (m >> 64) as u64;
            }
        }
    }

    /// Fisher-Yates shuffle.
    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            v.swap(i, j);
        }
    }

    /// `k` distinct indices from `0..n`.
    pub fn sample_distinct(&mut self, n: usize, k: usize) -> Vec<usize> {
        assert!(k <= n);
        let mut idx: Vec<usize> = (0..n).collect();
        for i in 0..k {
            let j = i + self.below((n - i) as u64) as usize;
            idx.swap(i, j);
        }
        idx.truncate(k);
        idx
    }

    /// Bernoulli with probability num/den (integer).
    pub fn chance(&mut self, num: u64, den: u64) -> bool {
        self.below(den) < num
    }
}
