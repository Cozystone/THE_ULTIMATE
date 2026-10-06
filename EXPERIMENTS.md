# EXPERIMENTS

Every experiment: hypothesis, setup, pass criterion fixed before running, result, verdict.
Hardware for all runs unless stated: Ryzen 9 9950X3D (Zen 5, AVX-512 VPOPCNTDQ), 64 GB DDR5,
Windows 11, Rust 1.95, `--release`.

---

## E-A1 Kernel correctness and baseline performance (Phase A)
Hypothesis: XOR/popcount kernels at 16,384 and 32,768 bits are exact across scalar, AVX2 and
AVX-512 paths, and fast enough that a cleanup search over 10^5 items fits a 1 ms cognitive tick.
Pass: all kernels bit-identical to scalar on property tests; numbers recorded.

Result (2026-10-06, `cargo run --release -p bm-bench --bin hdc_bench`, raw: `experiments/results/hdc_bench.json`):

| op | 16,384 bits | 32,768 bits |
|---|---|---|
| Hamming, portable SWAR | 106.1 ns | 214.9 ns |
| Hamming, x86 POPCNT | 48.3 ns | 98.7 ns |
| Hamming, AVX2 vpshufb | 31.0 ns | 61.7 ns |
| Hamming, AVX-512 VPOPCNTDQ (dispatched) | **7.4 ns** | **13.5 ns** |
| bind (XOR, by value) | 43.1 ns | 87.5 ns |
| permute, arbitrary offset | 105.2 ns | 225.4 ns |
| bundle add, bit-plane counter | 90.8 ns | 174.3 ns |
| bundle add, i32 counter | 2,375 ns | 4,780 ns |

Cleanup search, one query, 16,384 bits:

| N items | memory | exact scan | prefix cascade (2,048-bit pre-filter) | recall of cascade at 40% noise |
|---|---|---|---|---|
| 1,000 | 2.3 MB | 15.3 us | 3.0 us | 100% |
| 10,000 | 23 MB | 180 us | 30.5 us | 100% |
| 100,000 | 231 MB | 5,181 us | **376 us** | 100% |
| 1,000,000 | 2.3 GB | 50,376 us | 7,642 us | 100% |

Verdict: kernel equivalence PASS (17 tests, property tested at odd lengths). The pre-registered
latency criterion (10^5 items within a 1 ms tick) **FAILED with the exact scan** (5.2 ms): the scan
is DRAM-bandwidth bound (231 MB per query, ~45 GB/s), not compute bound. **PASSES with the prefix
cascade** (0.38 ms, no recall loss up to 40% noise). Parallel scan with 16 scoped threads per query
was slower (3.1 ms at 10^5) because spawn cost dominates; rejected (D010).

## E-A2 Capacity and noise recovery (Phase A)
Hypothesis: at 16k bits a bundle holds about 2x the 10k-bit capacity measured in the Python probe
(about 200 items at 99% recall at 10k bits), and cleanup recovers items with up to ~35% flipped bits.
Pass: measured curves recorded; failure region demonstrated by a test.

Result (10,000-item codebook, recall of bundled members among top-k):

| items bundled | 16,384 bits | 32,768 bits |
|---|---|---|
| 200 | 1.000 | 1.000 |
| 300 | 0.989 | |
| 400 | 0.978 | 0.999 |
| 600 | 0.934 | 0.993 |
| 800 | 0.897 | 0.983 |
| 1,600 | 0.789 | 0.922 |
| 3,200 | | 0.855 |

Noise recovery (10,000 items, z = 6 floor): 16k recovers 100/100 up to 46% flipped bits and abstains
100/100 at 48%. 32k recovers 100/100 at 48%. **Zero wrong answers in every noise condition**: below
the floor the memory abstains instead of guessing.

Verdict: PASS. Usable bundle capacity at 99% recall is ~300 items at 16k and ~600 at 32k. This is
the hard budget for any single bundled structure (records, prototypes, scope sets). Larger
structures must be hierarchical with cleanup at each level (see Python probe: 2-4 levels max
without cleanup).

## E-R0 Smallest relation-learning falsification test (gate before Phase B)
Purpose: separate the four outcomes (class learning, memorized lookup, relational learning,
transfer) on the smallest possible world, before any large event store exists.

World: entities with a few channels. Pair action `act(a, b)` returns an outcome.
Four learners on identical training experience:
* `ClassLearner`: prototype per outcome from whole-episode hypervectors, nearest prototype.
* `LookupLearner`: exact memorized table of seen episodes plus nearest stored episode.
* `RelationalLearner`: role-structured transform features with the licensing lifecycle.
* `Oracle`: knows the true rule (ceiling).

Conditions (each a separate world, fixed before running):
1. **Equality relation, held-out values**: outcome = 1 iff colour(a) == colour(b). Training uses
   colours 0..5, test uses only colours 6..9, never seen.
2. **Order relation, held-out values**: outcome = sign(weight(a) - weight(b)). Train weights 1..10,
   test 11..20.
3. **No-relation control**: outcome is a fixed random function of the pair identity. Nothing
   reusable exists. A licensed relation here is a false licence.
4. **Absolute-class control**: outcome = 1 iff colour(a) == 3. The real structure is a property,
   not a relation. Licensing a relational law here is a false relation.
5. **Observation-only confound**: training data is passive. A hidden cause sets both equality of
   colours and the outcome. Without intervention nothing may be LICENSED.

Pass criteria (all must hold):
* C1, C2: RelationalLearner accuracy on held-out values >= 95% while Class and Lookup <= chance+15pp.
  The licensed relation must have a transfer record with pre-registered successes.
* C3: zero LICENSED relations; RelationalLearner abstains on >= 90% of test queries.
* C4: no LICENSED relational law; the licensed law (if any) is the absolute-property law.
* C5: zero LICENSED relations after passive training; after allowing interventions the correct
  status is reached or the confound is exposed (relation REVOKED).
Result: pending.
