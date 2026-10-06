//! Packed binary hypervector.

use crate::kernel;
use crate::rng::Rng;
use std::fmt;

/// Binary hypervector of `64 * W` bits packed into u64 words.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(C, align(64))]
pub struct Hv<const W: usize> {
    pub w: [u64; W],
}

/// 16,384-bit hypervector (default, D002).
pub type Hv16k = Hv<256>;
/// 32,768-bit hypervector.
pub type Hv32k = Hv<512>;

impl<const W: usize> fmt::Debug for Hv<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hv<{}>(pop={}, {:016x}..)", W * 64, self.popcount(), self.w[0])
    }
}

impl<const W: usize> Default for Hv<W> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<const W: usize> Hv<W> {
    /// Dimension in bits.
    pub const BITS: u32 = (W * 64) as u32;

    pub const fn zero() -> Self {
        Hv { w: [0u64; W] }
    }

    pub fn ones() -> Self {
        Hv { w: [u64::MAX; W] }
    }

    pub fn random(rng: &mut Rng) -> Self {
        let mut w = [0u64; W];
        for x in w.iter_mut() {
            *x = rng.next_u64();
        }
        Hv { w }
    }

    pub fn from_seed(seed: u64) -> Self {
        Self::random(&mut Rng::new(seed))
    }

    // ------------------------------------------------------------ algebra

    /// Binding = XOR. Self-inverse: `a.bind(b).bind(b) == a`.
    #[inline]
    pub fn bind(&self, o: &Self) -> Self {
        let mut w = [0u64; W];
        for i in 0..W {
            w[i] = self.w[i] ^ o.w[i];
        }
        Hv { w }
    }

    /// Unbinding. Identical to binding for XOR, named separately for readability.
    #[inline]
    pub fn unbind(&self, key: &Self) -> Self {
        self.bind(key)
    }

    #[inline]
    pub fn bind_assign(&mut self, o: &Self) {
        for i in 0..W {
            self.w[i] ^= o.w[i];
        }
    }

    /// Bind many vectors together (conjunction). Order-independent.
    pub fn bind_all<'a, I: IntoIterator<Item = &'a Self>>(items: I) -> Self
    where
        Self: 'a,
    {
        let mut acc = Self::zero();
        for it in items {
            acc.bind_assign(it);
        }
        acc
    }

    /// Cyclic rotation of the whole bit string by `k` positions (positive = towards higher bit
    /// index). Deterministic permutation; `permute(k).permute(-k) == self`.
    pub fn permute(&self, k: i64) -> Self {
        let d = (W * 64) as i64;
        let k = k.rem_euclid(d) as usize;
        if k == 0 {
            return self.clone();
        }
        let ws = k / 64;
        let bs = (k % 64) as u32;
        let mut w = [0u64; W];
        for i in 0..W {
            let src = (i + W - ws) % W;
            if bs == 0 {
                w[i] = self.w[src];
            } else {
                let prev = (src + W - 1) % W;
                w[i] = (self.w[src] << bs) | (self.w[prev] >> (64 - bs));
            }
        }
        Hv { w }
    }

    /// Inverse permutation.
    pub fn unpermute(&self, k: i64) -> Self {
        self.permute(-k)
    }

    // ------------------------------------------------------------ measures

    #[inline]
    pub fn popcount(&self) -> u32 {
        kernel::popcount(&self.w)
    }

    /// Hamming distance.
    #[inline]
    pub fn distance(&self, o: &Self) -> u32 {
        kernel::hamming(&self.w, &o.w)
    }

    /// `D - popcount(a ^ b)` (D003). Random pairs sit near D/2.
    #[inline]
    pub fn similarity(&self, o: &Self) -> u32 {
        Self::BITS - self.distance(o)
    }

    pub fn is_zero(&self) -> bool {
        self.w.iter().all(|&x| x == 0)
    }

    // ------------------------------------------------------------ bits

    #[inline]
    pub fn get(&self, i: usize) -> bool {
        (self.w[i / 64] >> (i % 64)) & 1 == 1
    }

    #[inline]
    pub fn set(&mut self, i: usize, v: bool) {
        let m = 1u64 << (i % 64);
        if v {
            self.w[i / 64] |= m;
        } else {
            self.w[i / 64] &= !m;
        }
    }

    #[inline]
    pub fn flip(&mut self, i: usize) {
        self.w[i / 64] ^= 1u64 << (i % 64);
    }

    /// Flip exactly `n` distinct random bits (noise injection for tests).
    pub fn flip_random(&mut self, n: usize, rng: &mut Rng) {
        for i in rng.sample_distinct(W * 64, n) {
            self.flip(i);
        }
    }

    /// Bitwise majority of three (fast exact bundle of 3).
    pub fn majority3(a: &Self, b: &Self, c: &Self) -> Self {
        let mut w = [0u64; W];
        for i in 0..W {
            w[i] = (a.w[i] & b.w[i]) | (a.w[i] & c.w[i]) | (b.w[i] & c.w[i]);
        }
        Hv { w }
    }

    /// Fingerprint of `self ^ other` without materialising the bound vector.
    pub fn fingerprint_xor(&self, other: &Self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for i in 0..W {
            h ^= self.w[i] ^ other.w[i];
            h = h.wrapping_mul(0x0000_0100_0000_01b3).rotate_left(5);
        }
        h
    }

    /// Short stable fingerprint for hashing/keys (first two words mixed).
    pub fn fingerprint(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for &x in self.w.iter() {
            h ^= x;
            h = h.wrapping_mul(0x0000_0100_0000_01b3).rotate_left(5);
        }
        h
    }
}

/// Noise floor (D003): distance below which a match is considered non-random.
/// Random distances are ~N(D/2, sqrt(D)/2). Accept if dist <= D/2 - z*sqrt(D)/2.
pub fn noise_floor<const W: usize>(z: u32) -> u32 {
    let d = Hv::<W>::BITS;
    let sigma = isqrt(d as u64) as u32 / 2;
    d / 2 - z * sigma
}

/// Integer square root.
pub fn isqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    // integer Newton iteration
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
