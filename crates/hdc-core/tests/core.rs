//! Phase A gate tests: inverse operations, noise recovery, capacity limits, determinism,
//! kernel equivalence, and explicit failure cases.

use hdc_core::fixed::*;
use hdc_core::kernel::{available_kernels, hamming_portable};
use hdc_core::*;
use hdc_core::Rng;
use proptest::prelude::*;

type H = Hv16k;
const D: u32 = H::BITS;

fn hv_from(words: &[u64]) -> H {
    let mut w = [0u64; 256];
    for (i, x) in words.iter().enumerate().take(256) {
        w[i] = *x;
    }
    Hv { w }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn kernels_match_portable(a in prop::collection::vec(any::<u64>(), 256), b in prop::collection::vec(any::<u64>(), 256)) {
        let r = hamming_portable(&a, &b);
        for (kind, f) in available_kernels() {
            prop_assert_eq!(f(&a, &b), r, "kernel {:?}", kind);
        }
        // odd lengths exercise the scalar tails of SIMD kernels
        for len in [1usize, 3, 7, 9, 13, 255] {
            let r = hamming_portable(&a[..len], &b[..len]);
            for (kind, f) in available_kernels() {
                prop_assert_eq!(f(&a[..len], &b[..len]), r, "kernel {:?} len {}", kind, len);
            }
        }
    }

    #[test]
    fn bind_is_self_inverse_and_commutative(a in prop::collection::vec(any::<u64>(), 256), b in prop::collection::vec(any::<u64>(), 256), c in prop::collection::vec(any::<u64>(), 256)) {
        let (a, b, c) = (hv_from(&a), hv_from(&b), hv_from(&c));
        prop_assert_eq!(a.bind(&b).unbind(&b), a.clone());
        prop_assert_eq!(a.bind(&b), b.bind(&a));
        prop_assert_eq!(a.bind(&b).bind(&c), a.bind(&b.bind(&c)));
        // binding preserves distance: d(a^c, b^c) = d(a,b)
        prop_assert_eq!(a.bind(&c).distance(&b.bind(&c)), a.distance(&b));
    }

    #[test]
    fn permute_is_invertible_and_composes(a in prop::collection::vec(any::<u64>(), 256), k1 in -40000i64..40000, k2 in -40000i64..40000) {
        let a = hv_from(&a);
        prop_assert_eq!(a.permute(k1).unpermute(k1), a.clone());
        prop_assert_eq!(a.permute(k1).popcount(), a.popcount());
        prop_assert_eq!(a.permute(k1).permute(k2), a.permute(k1 + k2));
        prop_assert_eq!(a.permute(D as i64), a.clone());
        // permutation distributes over binding
        let b = a.permute(17);
        prop_assert_eq!(a.bind(&b).permute(k1), a.permute(k1).bind(&b.permute(k1)));
    }

    #[test]
    fn distance_is_a_metric(a in prop::collection::vec(any::<u64>(), 256), b in prop::collection::vec(any::<u64>(), 256), c in prop::collection::vec(any::<u64>(), 256)) {
        let (a, b, c) = (hv_from(&a), hv_from(&b), hv_from(&c));
        prop_assert_eq!(a.distance(&b), b.distance(&a));
        prop_assert!(a.distance(&c) <= a.distance(&b) + b.distance(&c));
        prop_assert_eq!(a.distance(&a), 0);
        prop_assert_eq!(a.similarity(&b), D - a.distance(&b));
    }

    #[test]
    fn bitplane_majority_equals_counter_majority(seed in any::<u64>(), n in 1usize..40) {
        let mut rng = Rng::new(seed);
        let items: Vec<H> = (0..n).map(|_| H::random(&mut rng)).collect();
        let tb = H::random(&mut rng);
        let mut bp = BitPlaneBundler::<256>::new();
        let mut cb = CounterBundler::<256>::new();
        for it in &items { bp.add(it); cb.add(it); }
        prop_assert_eq!(bp.majority(&tb), cb.finish(&tb));
        // exact per-dimension counts
        for i in [0usize, 1, 63, 64, 1000, 16383] {
            let c = items.iter().filter(|h| h.get(i)).count() as u32;
            prop_assert_eq!(bp.count(i), c);
        }
    }
}

#[test]
fn random_pairs_concentrate_at_half_dimension() {
    let mut rng = Rng::new(1);
    let sigma = (isqrt(D as u64) / 2) as u32;
    let (lo, hi) = (D / 2 - 6 * sigma, D / 2 + 6 * sigma);
    for _ in 0..2000 {
        let d = H::random(&mut rng).distance(&H::random(&mut rng));
        assert!(d > lo && d < hi, "distance {d} outside 6 sigma");
    }
}

#[test]
fn item_memory_is_deterministic_and_order_independent() {
    let mut m1 = ItemMemory::<256>::new(42);
    let mut m2 = ItemMemory::<256>::new(42);
    let a1 = m1.get("apple");
    let _ = m2.get("zebra");
    let a2 = m2.get("apple");
    assert_eq!(a1, a2);
    assert_eq!(a1, ItemMemory::<256>::atom(42, "apple"));
    let other_seed = ItemMemory::<256>::atom(43, "apple");
    assert!(a1.distance(&other_seed) > noise_floor::<256>(6));
    assert!(a1.distance(&m1.get("pear")) > noise_floor::<256>(6));
}

#[test]
fn role_filler_record_is_decomposable() {
    let mut im = ItemMemory::<256>::new(7);
    let mut fillers = CleanupMemory::<256>::new();
    for i in 0..1000u64 {
        fillers.insert(i, &im.get(&format!("f{i}")));
    }
    let tb = im.get("__tiebreak");
    let pairs: Vec<(String, u64)> = (0..9).map(|r| (format!("role{r}"), 100 + r as u64 * 7)).collect();
    let bound: Vec<H> = pairs.iter().map(|(r, f)| im.get(r).bind(&im.get(&format!("f{f}")))).collect();
    let record = bundle(&bound, &tb);
    for (r, f) in &pairs {
        let q = record.unbind(&im.get(r));
        let hit = fillers.cleanup(&q).expect("filler recovered");
        assert_eq!(hit.id, *f);
    }
    // failure case: unbinding with a key that was never bound gives noise -> abstain
    let q = record.unbind(&im.get("role_never_used"));
    assert!(fillers.cleanup(&q).is_none(), "wrong key must not produce a confident answer");
}

#[test]
fn cleanup_recovers_noisy_items_and_abstains_on_noise() {
    let mut rng = Rng::new(3);
    let n = 10_000u64;
    let mut mem = CleanupMemory::<256>::new();
    let items: Vec<H> = (0..n).map(|_| H::random(&mut rng)).collect();
    for (i, h) in items.iter().enumerate() {
        mem.insert(i as u64, h);
    }
    // 30% of bits flipped: still recovered
    for t in 0..50 {
        let id = (t * 197) % n as usize;
        let mut q = items[id].clone();
        q.flip_random((D as usize) * 30 / 100, &mut rng);
        assert_eq!(mem.cleanup(&q).map(|h| h.id), Some(id as u64));
    }
    // failure region: 48% flipped is indistinguishable from chance -> abstain
    let mut abstained = 0;
    for t in 0..50 {
        let mut q = items[t].clone();
        q.flip_random((D as usize) * 48 / 100, &mut rng);
        if mem.cleanup(&q).is_none() {
            abstained += 1;
        }
    }
    assert!(abstained >= 45, "abstained only {abstained}/50 at 48% noise");
    // pure random query: abstain
    for _ in 0..50 {
        assert!(mem.cleanup(&H::random(&mut rng)).is_none());
    }
}

fn bundle_recall(k: usize, seed: u64) -> (usize, usize) {
    let mut rng = Rng::new(seed);
    let n = 10_000usize;
    let items: Vec<H> = (0..n).map(|_| H::random(&mut rng)).collect();
    let idx = rng.sample_distinct(n, k);
    let mut b = BitPlaneBundler::<256>::new();
    for &i in &idx {
        b.add(&items[i]);
    }
    let s = b.majority(&H::random(&mut rng));
    let mut d: Vec<(u32, usize)> = items.iter().enumerate().map(|(i, h)| (h.distance(&s), i)).collect();
    d.sort();
    let top: std::collections::HashSet<usize> = d[..k].iter().map(|x| x.1).collect();
    (idx.iter().filter(|i| top.contains(i)).count(), k)
}

#[test]
fn bundle_capacity_holds_then_fails() {
    let (hit, k) = bundle_recall(200, 11);
    assert!(hit * 100 >= k * 99, "k=200 recall {hit}/{k}");
    // documented failure region: far beyond capacity, recall collapses
    let (hit, k) = bundle_recall(3000, 12);
    assert!(hit * 100 < k * 85, "k=3000 should exceed capacity, recall {hit}/{k}");
}

#[test]
fn level_codes_preserve_order_of_similarity() {
    let lc = LevelCodebook::<256>::linear(21, 5);
    let d01 = lc.get(0).distance(lc.get(1));
    let d05 = lc.get(0).distance(lc.get(5));
    let d0_20 = lc.get(0).distance(lc.get(20));
    assert!(d01 < d05 && d05 < d0_20);
    assert_eq!(d0_20, D / 2);
    let cc = LevelCodebook::<256>::circular(24, 9);
    assert!(cc.get(0).distance(cc.get(23)) < cc.get(0).distance(cc.get(12)));
    let mut rng = Rng::new(2);
    let mut noisy = lc.get(7).clone();
    noisy.flip_random(1500, &mut rng);
    assert_eq!(lc.decode(&noisy).0, 7);
}

#[test]
fn ordinal_offset_is_value_independent() {
    let base = Hv16k::from_seed(99);
    for (a, b) in [(3i64, 8i64), (40, 45), (1000, 1005), (12, 2)] {
        let k = find_offset(&ordinal(&base, a), &ordinal(&base, b), 64, 0);
        assert_eq!(k, Some(b - a));
    }
    // unrelated vectors have no offset
    assert_eq!(find_offset(&Hv16k::from_seed(1), &Hv16k::from_seed(2), 64, noise_floor::<256>(6)), None);
}

#[test]
fn sequence_positions_are_recoverable() {
    let mut im = ItemMemory::<256>::new(1);
    let words = ["the", "red", "ball", "rolls", "left"];
    let hv: Vec<H> = words.iter().map(|w| im.get(w)).collect();
    let mut mem = CleanupMemory::<256>::new();
    for (i, h) in hv.iter().enumerate() {
        mem.insert(i as u64, h);
    }
    let s = seq::encode_sequence(&hv, &im.get("__tb"));
    for i in 0..words.len() {
        assert_eq!(mem.cleanup(&seq::probe_position(&s, i)).map(|h| h.id), Some(i as u64));
    }
    // n-grams are order sensitive
    let ab = seq::ngram(&[hv[0].clone(), hv[1].clone()]);
    let ba = seq::ngram(&[hv[1].clone(), hv[0].clone()]);
    assert!(ab.distance(&ba) > noise_floor::<256>(6));
}

#[test]
fn fixed_point_information_measures() {
    assert_eq!(log2_q16(1), 0);
    assert_eq!(log2_q16(8), 3 * Q);
    // log2(3) = 1.5849625 -> 103872.4 in Q16
    assert!((log2_q16(3) - 103_872).abs() <= 2);
    assert_eq!(entropy_q16(&[5, 5]), Q);
    assert_eq!(entropy_q16(&[1, 1, 1, 1]), 2 * Q);
    assert_eq!(entropy_q16(&[7, 0]), 0);
    let ig_fresh = expected_info_gain_q16(&[0, 0]);
    let ig_known = expected_info_gain_q16(&[50, 0]);
    assert!(ig_fresh > ig_known && ig_known >= 0);
}

#[test]
fn evidence_is_integer_and_decays_by_shift() {
    let mut e = Evidence::new();
    for i in 0..8 {
        e.observe(true, i);
    }
    e.observe(false, 8);
    assert_eq!((e.support, e.refute), (8, 1));
    assert!(e.confidence_q16() > e.lower_q16());
    assert!(e.refute_rate_at_most(1, 8));
    assert!(!e.refute_rate_at_most(1, 10));
    e.decay(8 + 200, 100);
    assert_eq!((e.support, e.refute), (2, 0));
}

#[test]
fn fast_majority3_matches_bundler() {
    let mut rng = Rng::new(8);
    let (a, b, c) = (H::random(&mut rng), H::random(&mut rng), H::random(&mut rng));
    assert_eq!(Hv::majority3(&a, &b, &c), bundle(&[a.clone(), b.clone(), c.clone()], &H::zero()));
}

#[test]
fn fast_and_parallel_cleanup_agree_with_exact() {
    let mut rng = Rng::new(21);
    let mut mem = CleanupMemory::<256>::new();
    let items: Vec<H> = (0..5000).map(|_| H::random(&mut rng)).collect();
    for (i, h) in items.iter().enumerate() {
        mem.insert(i as u64, h);
    }
    for t in 0..40usize {
        let mut q = items[t * 101 % 5000].clone();
        q.flip_random(D as usize * 35 / 100, &mut rng);
        let exact = mem.cleanup(&q).map(|h| h.id);
        assert_eq!(mem.cleanup_par(&q, 7).map(|h| h.id), exact);
        assert_eq!(mem.cleanup_fast(&q).map(|h| h.id), exact);
    }
    // random queries: all three abstain
    for _ in 0..20 {
        let q = H::random(&mut rng);
        assert!(mem.cleanup_fast(&q).is_none() && mem.cleanup_par(&q, 4).is_none());
    }
    // remove keeps prefix slab consistent
    assert!(mem.remove(17));
    let q = items[4999].clone();
    assert_eq!(mem.cleanup_fast(&q).map(|h| h.id), Some(4999));
}

#[test]
fn exp2_and_posterior_are_integer_and_accurate() {
    assert_eq!(exp2_neg_q16(0), Q);
    assert_eq!(exp2_neg_q16(Q), Q / 2);
    assert_eq!(exp2_neg_q16(3 * Q), Q / 8);
    // 2^-0.5 = 0.70710678 -> 46341 in Q16
    assert!((exp2_neg_q16(Q / 2) - 46341).abs() <= 2);
    // two equal hypotheses -> 0.5
    let (_, p) = best_posterior_q16(&[-10 * Q, -10 * Q]).unwrap();
    assert!((p - Q / 2).abs() <= 2);
    // 10 bits better -> 1024/1025
    let (i, p) = best_posterior_q16(&[-20 * Q, -10 * Q]).unwrap();
    assert_eq!(i, 1);
    assert!((p - Q * 1024 / 1025).abs() <= 4);
}
