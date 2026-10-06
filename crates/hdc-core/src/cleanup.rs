//! Associative cleanup memory: restore the nearest stored item, or abstain below the noise floor.

use crate::hv::{noise_floor, Hv};
use crate::kernel;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    /// Caller-supplied id of the stored item.
    pub id: u64,
    /// Hamming distance to the query.
    pub dist: u32,
    /// Distance gap to the runner-up (`u32::MAX` if only one item). Small margin = ambiguous.
    pub margin: u32,
}

/// Contiguous slab of hypervectors with ids.
#[derive(Clone, Debug)]
pub struct CleanupMemory<const W: usize> {
    ids: Vec<u64>,
    slab: Vec<u64>,
    /// First `PREFIX` words of every row, stored contiguously for a bandwidth-cheap first pass.
    prefix: Vec<u64>,
    /// Acceptance threshold in bits (inclusive).
    pub accept: u32,
}

/// Words used by the prefix pre-filter (2,048 bits).
pub const PREFIX: usize = 32;

impl<const W: usize> Default for CleanupMemory<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize> CleanupMemory<W> {
    /// Default acceptance: z = 6 sigma below D/2 (D003).
    pub fn new() -> Self {
        Self::with_z(6)
    }

    pub fn with_z(z: u32) -> Self {
        CleanupMemory { ids: Vec::new(), slab: Vec::new(), prefix: Vec::new(), accept: noise_floor::<W>(z) }
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn insert(&mut self, id: u64, hv: &Hv<W>) {
        self.ids.push(id);
        self.slab.extend_from_slice(&hv.w);
        self.prefix.extend_from_slice(&hv.w[..Self::pw()]);
    }

    #[inline]
    const fn pw() -> usize {
        if W < PREFIX {
            W
        } else {
            PREFIX
        }
    }

    /// Replace the vector stored under `id` (first occurrence). Returns false if absent.
    pub fn update(&mut self, id: u64, hv: &Hv<W>) -> bool {
        if let Some(pos) = self.ids.iter().position(|&x| x == id) {
            self.slab[pos * W..(pos + 1) * W].copy_from_slice(&hv.w);
            let p = Self::pw();
            self.prefix[pos * p..(pos + 1) * p].copy_from_slice(&hv.w[..p]);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, id: u64) -> bool {
        if let Some(pos) = self.ids.iter().position(|&x| x == id) {
            self.ids.swap_remove(pos);
            let last = self.ids.len();
            if pos != last {
                let (head, tail) = self.slab.split_at_mut(last * W);
                head[pos * W..(pos + 1) * W].copy_from_slice(&tail[..W]);
            }
            self.slab.truncate(last * W);
            let p = Self::pw();
            if pos != last {
                let (head, tail) = self.prefix.split_at_mut(last * p);
                head[pos * p..(pos + 1) * p].copy_from_slice(&tail[..p]);
            }
            self.prefix.truncate(last * p);
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: u64) -> Option<Hv<W>> {
        self.ids.iter().position(|&x| x == id).map(|pos| {
            let mut w = [0u64; W];
            w.copy_from_slice(&self.slab[pos * W..(pos + 1) * W]);
            Hv { w }
        })
    }

    #[inline]
    fn row(&self, i: usize) -> &[u64] {
        &self.slab[i * W..(i + 1) * W]
    }

    /// Nearest item regardless of the noise floor.
    pub fn nearest_raw(&self, q: &Hv<W>) -> Option<Hit> {
        if self.ids.is_empty() {
            return None;
        }
        let mut best = (0usize, u32::MAX);
        let mut second = u32::MAX;
        for i in 0..self.ids.len() {
            let d = kernel::hamming(&q.w, self.row(i));
            if d < best.1 {
                second = best.1;
                best = (i, d);
            } else if d < second {
                second = d;
            }
        }
        Some(Hit { id: self.ids[best.0], dist: best.1, margin: second.saturating_sub(best.1) })
    }

    /// Nearest item if it is significantly closer than chance, else abstain.
    pub fn cleanup(&self, q: &Hv<W>) -> Option<Hit> {
        self.nearest_raw(q).filter(|h| h.dist <= self.accept)
    }

    /// The `k` nearest items (ascending distance).
    pub fn top_k(&self, q: &Hv<W>, k: usize) -> Vec<(u64, u32)> {
        let mut all: Vec<(u64, u32)> =
            (0..self.ids.len()).map(|i| (self.ids[i], kernel::hamming(&q.w, self.row(i)))).collect();
        all.sort_by_key(|&(id, d)| (d, id));
        all.truncate(k);
        all
    }

    /// Every item within `max_dist` bits.
    pub fn within(&self, q: &Hv<W>, max_dist: u32) -> Vec<(u64, u32)> {
        let mut v = Vec::new();
        for i in 0..self.ids.len() {
            let d = kernel::hamming(&q.w, self.row(i));
            if d <= max_dist {
                v.push((self.ids[i], d));
            }
        }
        v
    }

    pub fn bytes(&self) -> usize {
        (self.slab.len() + self.prefix.len() + self.ids.len()) * 8
    }

    /// Two-stage cleanup: stream the contiguous 2,048-bit prefix slab, keep rows whose prefix
    /// distance is below the prefix noise floor (z = 3), then verify survivors on the full width.
    /// Approximate: an item whose prefix is unluckily noisy can be missed. Recall is measured in
    /// EXPERIMENTS.md E-A1; the exact `cleanup` remains available.
    pub fn cleanup_fast(&self, q: &Hv<W>) -> Option<Hit> {
        let p = Self::pw();
        if p == W {
            return self.cleanup(q);
        }
        let pbits = (p * 64) as u64;
        let sigma = crate::hv::isqrt(pbits) as u32 / 2;
        let pre_accept = (pbits as u32) / 2 - 3 * sigma;
        let qp = &q.w[..p];
        let mut best = (usize::MAX, u32::MAX);
        let mut second = u32::MAX;
        for i in 0..self.ids.len() {
            let pd = kernel::hamming(qp, &self.prefix[i * p..(i + 1) * p]);
            if pd <= pre_accept {
                let d = kernel::hamming(&q.w, self.row(i));
                if d < best.1 {
                    second = best.1;
                    best = (i, d);
                } else if d < second {
                    second = d;
                }
            }
        }
        if best.0 == usize::MAX || best.1 > self.accept {
            return None;
        }
        Some(Hit { id: self.ids[best.0], dist: best.1, margin: second.saturating_sub(best.1) })
    }

    /// Exact nearest search split across `threads` OS threads (scoped, no allocation per row).
    pub fn cleanup_par(&self, q: &Hv<W>, threads: usize) -> Option<Hit> {
        let n = self.ids.len();
        if n == 0 {
            return None;
        }
        let threads = threads.max(1).min(n);
        let chunk = n.div_ceil(threads);
        let parts: Vec<(usize, u32, u32)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..threads)
                .map(|t| {
                    let lo = t * chunk;
                    let hi = ((t + 1) * chunk).min(n);
                    s.spawn(move || {
                        let mut best = (usize::MAX, u32::MAX);
                        let mut second = u32::MAX;
                        for i in lo..hi {
                            let d = kernel::hamming(&q.w, self.row(i));
                            if d < best.1 {
                                second = best.1;
                                best = (i, d);
                            } else if d < second {
                                second = d;
                            }
                        }
                        (best.0, best.1, second)
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().expect("worker")).collect()
        });
        let mut best = (usize::MAX, u32::MAX);
        let mut second = u32::MAX;
        for (i, d, s2) in parts {
            if i == usize::MAX {
                continue;
            }
            if d < best.1 {
                second = second.min(best.1);
                best = (i, d);
            } else {
                second = second.min(d);
            }
            second = second.min(s2);
        }
        if best.1 > self.accept {
            return None;
        }
        Some(Hit { id: self.ids[best.0], dist: best.1, margin: second.saturating_sub(best.1) })
    }
}
