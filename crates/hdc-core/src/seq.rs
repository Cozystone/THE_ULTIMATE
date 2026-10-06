//! Sequence encodings with position carried by permutation (language section 6).

use crate::bundle::BitPlaneBundler;
use crate::hv::Hv;

/// Position-preserving bag: majority of `rho^i(item_i)`.
pub fn encode_sequence<const W: usize>(items: &[Hv<W>], tiebreak: &Hv<W>) -> Hv<W> {
    let mut b = BitPlaneBundler::<W>::new();
    for (i, it) in items.iter().enumerate() {
        b.add(&it.permute(i as i64));
    }
    b.majority(tiebreak)
}

/// n-gram: `rho^(n-1)(x0) ^ rho^(n-2)(x1) ^ ... ^ x_{n-1}`. Order-sensitive, exact.
pub fn ngram<const W: usize>(items: &[Hv<W>]) -> Hv<W> {
    let n = items.len();
    let mut acc = Hv::<W>::zero();
    for (i, it) in items.iter().enumerate() {
        acc.bind_assign(&it.permute((n - 1 - i) as i64));
    }
    acc
}

/// Bundle of all n-grams of a sequence (text/byte profile).
pub fn ngram_profile<const W: usize>(items: &[Hv<W>], n: usize, tiebreak: &Hv<W>) -> Hv<W> {
    let mut b = BitPlaneBundler::<W>::new();
    if items.len() < n {
        b.add(&ngram(items));
    } else {
        for win in items.windows(n) {
            b.add(&ngram(win));
        }
    }
    b.majority(tiebreak)
}

/// Query the item at position `i` of an `encode_sequence` vector.
pub fn probe_position<const W: usize>(seq: &Hv<W>, i: usize) -> Hv<W> {
    seq.permute(-(i as i64))
}
