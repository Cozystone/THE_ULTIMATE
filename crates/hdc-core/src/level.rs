//! Encodings for integer quantities.
//!
//! * `LevelCodebook::linear`: similarity-preserving; distance grows linearly with |i-j| and reaches
//!   ~D/2 between the extremes.
//! * `LevelCodebook::circular`: same, for cyclic quantities (time of day, angle).
//! * `ordinal`: group encoding `rho^v(base)`; similarity is NOT preserved but the permutation offset
//!   between two values equals their difference, which is what relational transforms read (D006).

use crate::hv::Hv;
use crate::rng::Rng;

#[derive(Clone, Debug)]
pub struct LevelCodebook<const W: usize> {
    levels: Vec<Hv<W>>,
}

impl<const W: usize> LevelCodebook<W> {
    /// `n` levels; level 0 random, each next level flips a fresh disjoint chunk.
    pub fn linear(n: usize, seed: u64) -> Self {
        assert!(n >= 2);
        let d = W * 64;
        let mut rng = Rng::new(seed);
        let base = Hv::<W>::random(&mut rng);
        let mut perm: Vec<usize> = (0..d).collect();
        rng.shuffle(&mut perm);
        let flips_total = d / 2;
        let mut levels = Vec::with_capacity(n);
        let mut cur = base;
        let mut done = 0usize;
        levels.push(cur.clone());
        for i in 1..n {
            let target = flips_total * i / (n - 1);
            while done < target {
                cur.flip(perm[done]);
                done += 1;
            }
            levels.push(cur.clone());
        }
        LevelCodebook { levels }
    }

    /// `n` levels on a circle; opposite points are ~orthogonal, neighbours across the wrap are close.
    pub fn circular(n: usize, seed: u64) -> Self {
        assert!(n >= 4 && n % 2 == 0);
        let d = W * 64;
        let half = n / 2;
        let mut rng = Rng::new(seed);
        let base = Hv::<W>::random(&mut rng);
        let mut perm: Vec<usize> = (0..d).collect();
        rng.shuffle(&mut perm);
        let step = d / (2 * half);
        let chunks: Vec<&[usize]> = (0..half).map(|i| &perm[i * step..(i + 1) * step]).collect();
        let mut levels = Vec::with_capacity(n);
        for i in 0..n {
            let mut v = base.clone();
            let range: Vec<&[usize]> = if i <= half { chunks[..i].to_vec() } else { chunks[i - half..].to_vec() };
            for c in range {
                for &b in c {
                    v.flip(b);
                }
            }
            levels.push(v);
        }
        LevelCodebook { levels }
    }

    pub fn get(&self, i: usize) -> &Hv<W> {
        &self.levels[i.min(self.levels.len() - 1)]
    }

    pub fn len(&self) -> usize {
        self.levels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    /// Nearest level index for a (possibly noisy) vector.
    pub fn decode(&self, v: &Hv<W>) -> (usize, u32) {
        let mut best = (0usize, u32::MAX);
        for (i, l) in self.levels.iter().enumerate() {
            let d = l.distance(v);
            if d < best.1 {
                best = (i, d);
            }
        }
        best
    }
}

/// Ordinal group encoding of integer `v` relative to a channel base: `rho^v(base)`.
pub fn ordinal<const W: usize>(base: &Hv<W>, v: i64) -> Hv<W> {
    base.permute(v)
}

/// Find the permutation offset `k` in `[-max_k, max_k]` with `rho^k(a) == b` within `tol` bits.
/// Returns the offset with the smallest |k| among exact matches. `None` if no offset matches.
pub fn find_offset<const W: usize>(a: &Hv<W>, b: &Hv<W>, max_k: i64, tol: u32) -> Option<i64> {
    for m in 0..=max_k {
        let cands = [m, -m];
        let n = if m == 0 { 1 } else { 2 };
        for &k in &cands[..n] {
            if a.permute(k).distance(b) <= tol {
                return Some(k);
            }
        }
    }
    None
}
