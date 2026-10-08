# BITMIND v0.4 report: K5 (the licence cost over every hypothesis examined)

2026-10-08, branch `v0.4-dev` (rooted at `bitmind-v0.2`). Records: `PREREG-v0.4.md` (with
amendments 1 and 2), `DECISIONS.md` (D055 re-introduction, D056, D057, D057b, D058, D059),
`EXPERIMENTS.md` ("v0.4 result"), `results/v04_c3_e3_diagnosis.txt`, `results/v04dev1/`,
`results/v04dev2/`.

## Outcome

**v0.4 is not frozen.** Development round 2, the last allowed, met every pre-registered
condition but one:
* B7 bridge seed 1 failed (26 correct, 0 wrong, 174 abstain; v0.2: 200 / 0 / 0).
* Because of it, phase B fell below "correct >= v0.2 - 1 pp" and above the abstention bound.

By the stop rule:
* no `bitmind-v0.4` tag and no freeze guard;
* no K-series world was written or run;
* `bitmind-v0.2` remains the latest valid frozen learner;
* `main` and the earlier tags were not touched.

These results come from synthetic worlds and a sandboxed file system only. They say nothing
about consciousness, agency, general intelligence, world-model completion or real-world
readiness.

## What was shown

1. **K5 is real, and charging it removes the v0.3 identity leakage.** With the D012 cost
   computed over the effective family F (D056), H1 identity leakage is 0 on 15/15 seeds in both
   rounds (v0.3: 153 leaked answers). A unit test reproduces the mechanism (test 3):
   * a mark law materialized while the family was small;
   * it keeps 13 bits of utility while the family grows;
   * v0.3 licenses it, v0.4 does not, and both license the general rule.

   F is one monotone u64 per (action, target, context). It survives sleep, compaction,
   retirement and revival (test 4 exercises all four).
2. **C3 and E3 had different causes, and neither was K5**
   (`results/v04_c3_e3_diagnosis.txt`, read-only provenance, 15/15 seeds each).
   * C3 is replay/transfer accounting. The decisive law was materialized after every relevant
     situation had been seen, so it could never get a transfer trial (transfer 0/0, utility
     66-95 bits, 0 counterexamples).
   * E3's v0.2 pass was produced by a defective measurement. The twin never slept and was
     compared by law index. With the twin corrected, v0.2 itself fails 15/15 (active 30, twin
     30): any action, WAIT included, matched some of ~1,500 unlicensed sleep hypotheses. E3 was
     therefore moved to the baseline-failure list (amendment 1a).

     This changes a pre-registered criterion. It was done before any learner change, on a
     re-measured baseline, and should be judged by the reader.
3. **Repairs, each tied to a diagnosed class.**
   * D057 / D057b (prospective transfer while deferred, general and particular semantics):
     restores C1, C2, C3 and F3.
   * D058 (hidden-condition search on impure deferred values): restores H2. Its answers in v0.2
     come from conjunctions of single latent values that never earn a law on their own.
   * D059 (refinement proposal bar): restores E1. The bar is LLR >= log2(candidates in this
     search): at most one expected false proposal per search by Ville's inequality. The
     cumulative multiple-comparison charge is paid only at licensing.
   * Round 2 versus v0.2:
     * C, F and G are identical or better;
     * E1 is 15/15 (v0.2 14/15);
     * H2 is 14/15 with 0 wrong (v0.2 13/15);
     * H5 is 15/15;
     * no new wrong answer anywhere;
     * no counterexample was lost.
4. **Memory.** Peak working set falls 3-10x against v0.2:

   | world | v0.4 round 2 | v0.2 |
   |---|---|---|
   | H5 | 870-1,443 MB, 15/15 under budget | 3.5-6.9 GB, 0/15 |
   | real-OS C | 279-717 MB | 2.1-4.6 GB |
   | real-OS G | 395-425 MB | 3.7-3.9 GB |

   This is far less than v0.3's 191-347 MB, because D058 and D059 restore part of the
   hypothesis search that v0.3 had cut.

## The trade-off that stopped v0.4

The failing law is `diff(r0,r1;c0) => 0` in the noisiest B world (seed 1):
* 572 cases, 21 counterexamples, transfer 381/400;
* only 19 bits of incremental utility, because the base rate already predicts most of its
  cases.

Its target's family:

| learner | family | licence cost |
|---|---|---|
| round 1 | 670 | 13.4 bits |
| round 2 | 60,067 | 19.9 bits |

The searches over deferred parents that fixed H2 and E1 examine every feature of the parent's
episodes, and every examination is counted. The same candidate examined in repeated searches
is counted again (the conservative reading of "every refinement candidate examined").

So in this design, searching more broadly for hidden conditions raises the price of every
licence for that target. A genuine law that adds little beyond the base rate can then be
priced out. Round 1 had a small family and passed B7, but failed C3, E1 and H2. Round 2 has a
large family and passes them, but loses B7 seed 1.

An untested hypothesis for a next version: charge the number of *distinct* candidates examined
(for example with a bounded integer cardinality sketch), not the number of examinations. The
charge stays monotone and survives retirement, but no longer grows with re-searching. This has
not been implemented or measured, and nothing here shows it would pass B7 while keeping H1
leakage at 0.

## Things a reviewer should scrutinize

* **Criterion change.** E3 was removed from the must-pass set (amendment 1a), after showing that
  the recorded v0.2 pass came from a measurement defect.
* **Spot runs on development seeds** between rounds guided D057b, D058 and D059. In particular,
  the D059 bar was chosen after two alternatives were tried on development seeds:
  * a flat 4-bit bar: E1 passes, H5 seed 201 peaks at 1,301 MB;
  * log2(examined) + 4: E1 seed 46 fails.

  The chosen bar has a principled justification (Ville), but the choice was data-informed.
* **D057 is stricter than its amendment text:** 3 bindings, matching the law's
  `independent() >= 3`, where the amendment said ">= 2".
* **Round 1 C4b, seeds 48-49,** failed with a sandbox collision caused by my own concurrent
  diagnostic run. The markers were restored, and round 2 ran with nothing else overlapping.
* **After the round-2 build,** one diagnostic print in `bm-relation` was changed from `f64` to
  Q16 integers to pass the constitution check (`a702c08`, display only).
* **Harness instrumentation was added** (read-only, env-gated): BM_STATS lifecycle lines,
  DIAG_C3 / DIAG_E3 / E3STEP, DIAG_H2 / DIAG_WRONG / DIAG_ABST / DIAG_ANS, DIAG_B7. The learner
  gained `diag_target`, `family_size` and `stats_line` (read-only) and a `Drop` print under
  BM_STATS.

## Weaknesses observed, not repaired

* The verification policy prefers WAIT when there are few unlicensed hypotheses. Under the
  corrected E3 twin, v0.4 scores active 0 against a fixed rotation of 18-21 sleep-hypothesis
  tests.
* H4 (reachable conflict) still fails on 15/15 seeds, as in v0.2.
* H5 seeds 203, 207 and 213 answer slightly less than v0.2 (more abstention, no extra wrong).
* Wall time on the H and G batteries is comparable to v0.2 (G 4,000-4,200 s per 5 seeds; H
  4,800-7,500 s per 3 seeds), far above v0.3's.

## Tests

`crates/bm-relation/tests/v04.rs`, 9 tests, all passing on `v0.4-dev`:
1. 100,000 deferred weak candidates are charged as such (|F| >= 100,000; nothing licensed in a
   coin-flip world).
2. A sparse real signal amid nuisance values still licenses.
3. Object-specific mark laws stay unlicensed in a multiplicity trap. **Fails on v0.3.**
4. Family counts survive sleep, compaction, retirement and revival.
5. Refinement candidates are charged, and a valid low-cardinality refinement licenses.
6. Identity permutation invariance and counterevidence retention.
7. A law materialized late in a closed world can still be licensed. **Fails on v0.3.**
8. A deferred value with a counterexample earns no prospective trial and keeps the
   counterexample.
9. A conjunction of two impure deferred values is found and licensed. **Fails on v0.3.**

Tests 1, 4, 5 and 8 read the new counters and do not compile on v0.3.
