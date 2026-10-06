//! Integer evidence state: support, refutation, recency. No floats (constitution 4).

use crate::fixed::{ratio_q16, Q};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Evidence {
    pub support: u32,
    pub refute: u32,
    /// Tick of the most recent update.
    pub last: u64,
}

impl Evidence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(&mut self, ok: bool, now: u64) {
        if ok {
            self.support = self.support.saturating_add(1);
        } else {
            self.refute = self.refute.saturating_add(1);
        }
        self.last = now;
    }

    pub fn total(&self) -> u32 {
        self.support + self.refute
    }

    /// Laplace-smoothed confidence (support+1)/(total+2) in Q16.
    pub fn confidence_q16(&self) -> i64 {
        ratio_q16(self.support as u64 + 1, self.total() as u64 + 2)
    }

    /// Conservative lower bound: (support) / (total + 4) in Q16. Grows only with volume.
    pub fn lower_q16(&self) -> i64 {
        ratio_q16(self.support as u64, self.total() as u64 + 4)
    }

    /// Halve the counts once per elapsed `half_life` ticks (bit shift decay).
    pub fn decay(&mut self, now: u64, half_life: u64) {
        if half_life == 0 || now <= self.last {
            return;
        }
        let periods = ((now - self.last) / half_life).min(31) as u32;
        if periods > 0 {
            self.support >>= periods;
            self.refute >>= periods;
            self.last += periods as u64 * half_life;
        }
    }

    /// True if refutations are at most `num/den` of the total (integer comparison).
    pub fn refute_rate_at_most(&self, num: u32, den: u32) -> bool {
        (self.refute as u64) * den as u64 <= (self.total() as u64) * num as u64
    }
}

/// Q16 helpers re-exported for callers that only need confidence.
pub fn q16_percent(q: i64) -> i64 {
    q * 100 / Q
}
