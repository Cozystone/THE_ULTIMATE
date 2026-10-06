//! Integer-only bundling (D004).

use crate::hv::Hv;

/// Vertical (bit-plane) counter: plane `i` holds bit `i` of each dimension's count.
/// Adding a hypervector is a ripple of carry-save XOR/AND across planes.
#[derive(Clone, Debug)]
pub struct BitPlaneBundler<const W: usize> {
    planes: Vec<Hv<W>>,
    n: u32,
}

impl<const W: usize> Default for BitPlaneBundler<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize> BitPlaneBundler<W> {
    pub fn new() -> Self {
        BitPlaneBundler { planes: Vec::new(), n: 0 }
    }

    pub fn len(&self) -> u32 {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn add(&mut self, hv: &Hv<W>) {
        let mut carry = hv.clone();
        for p in self.planes.iter_mut() {
            let mut any = 0u64;
            for i in 0..W {
                let s = p.w[i] ^ carry.w[i];
                let c = p.w[i] & carry.w[i];
                p.w[i] = s;
                carry.w[i] = c;
                any |= c;
            }
            if any == 0 {
                self.n += 1;
                return;
            }
        }
        if !carry.is_zero() {
            self.planes.push(carry);
        }
        self.n += 1;
    }

    /// Bits whose count is strictly greater than `t`, plus the mask of counts exactly equal to `t`.
    pub fn compare(&self, t: u32) -> (Hv<W>, Hv<W>) {
        let l = self.planes.len();
        if l < 32 && (t as u64) >> l != 0 {
            // t exceeds any representable count
            return (Hv::zero(), Hv::zero());
        }
        let mut gt = Hv::<W>::zero();
        let mut eq = Hv::<W>::ones();
        for i in (0..l).rev() {
            let p = &self.planes[i];
            if (t >> i) & 1 == 0 {
                for k in 0..W {
                    gt.w[k] |= eq.w[k] & p.w[k];
                    eq.w[k] &= !p.w[k];
                }
            } else {
                for k in 0..W {
                    eq.w[k] &= p.w[k];
                }
            }
        }
        (gt, eq)
    }

    /// Bits set in at least `k` of the added vectors.
    pub fn at_least(&self, k: u32) -> Hv<W> {
        if k == 0 {
            return Hv::ones();
        }
        self.compare(k - 1).0
    }

    /// Majority vote. Ties (possible when n is even) take the bit from `tiebreak`.
    pub fn majority(&self, tiebreak: &Hv<W>) -> Hv<W> {
        let (mut gt, eq) = self.compare(self.n / 2);
        if self.n % 2 == 0 {
            for k in 0..W {
                gt.w[k] |= eq.w[k] & tiebreak.w[k];
            }
        }
        gt
    }

    /// Exact count of dimension `i` (for tests/inspection).
    pub fn count(&self, i: usize) -> u32 {
        let mut c = 0u32;
        for (b, p) in self.planes.iter().enumerate() {
            if p.get(i) {
                c |= 1 << b;
            }
        }
        c
    }

    pub fn planes(&self) -> usize {
        self.planes.len()
    }
}

/// Signed integer per-dimension counter for weighted and subtractive bundling
/// (bipolar view: bit 1 = +1, bit 0 = -1).
#[derive(Clone, Debug)]
pub struct CounterBundler<const W: usize> {
    c: Vec<i32>,
    weight: i64,
}

impl<const W: usize> Default for CounterBundler<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize> CounterBundler<W> {
    pub fn new() -> Self {
        CounterBundler { c: vec![0; W * 64], weight: 0 }
    }

    pub fn total_weight(&self) -> i64 {
        self.weight
    }

    pub fn add(&mut self, hv: &Hv<W>) {
        self.add_weighted(hv, 1);
    }

    pub fn sub(&mut self, hv: &Hv<W>) {
        self.add_weighted(hv, -1);
    }

    pub fn add_weighted(&mut self, hv: &Hv<W>, wgt: i32) {
        for k in 0..W {
            let word = hv.w[k];
            let base = k * 64;
            for b in 0..64 {
                let s = (((word >> b) & 1) as i32) * 2 - 1; // +1 / -1
                self.c[base + b] += s * wgt;
            }
        }
        self.weight += wgt as i64;
    }

    /// Sign threshold; zero counters take the tiebreak bit.
    pub fn finish(&self, tiebreak: &Hv<W>) -> Hv<W> {
        let mut out = Hv::<W>::zero();
        for i in 0..W * 64 {
            let v = self.c[i];
            if v > 0 || (v == 0 && tiebreak.get(i)) {
                out.set(i, true);
            }
        }
        out
    }

    pub fn counter(&self, i: usize) -> i32 {
        self.c[i]
    }

    /// Integer agreement of a vector with the accumulated counters (sum of signed counters on
    /// the vector's bipolar values). Larger = more similar to the bundle.
    pub fn agreement(&self, hv: &Hv<W>) -> i64 {
        let mut s = 0i64;
        for k in 0..W {
            let word = hv.w[k];
            for b in 0..64 {
                let v = self.c[k * 64 + b] as i64;
                if (word >> b) & 1 == 1 {
                    s += v;
                } else {
                    s -= v;
                }
            }
        }
        s
    }

    pub fn clear(&mut self) {
        self.c.iter_mut().for_each(|x| *x = 0);
        self.weight = 0;
    }
}

/// Convenience: majority bundle of a slice with a deterministic tiebreak.
pub fn bundle<const W: usize>(items: &[Hv<W>], tiebreak: &Hv<W>) -> Hv<W> {
    let mut b = BitPlaneBundler::<W>::new();
    for it in items {
        b.add(it);
    }
    b.majority(tiebreak)
}
