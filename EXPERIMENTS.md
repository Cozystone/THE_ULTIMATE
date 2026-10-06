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
* C6 (added before the first run): negative-transfer guard. Licences earned in room-A
  (equality) are confronted with room-B (inverted law, different observable context). Pass: the
  lived agent with scoped licences gives zero wrong answers in room-B, the scope-ignoring ablation
  does give wrong answers, and the lived agent's learning cost is at most 1.2x a naive agent's.

### E-R0 v1 result (2026-10-07, seed 1, raw: `experiments/results/r0_v1_seed1.json`)

| condition | relational held-out | best baseline held-out | verdict | why |
|---|---|---|---|---|
| C1 equality | 1.000 | lookup 0.667 | **FAIL** | baseline bound 0.65 exceeded by memorized lookup |
| C2 order | 1.000 | lookup 0.402 (chance 0.33) | PASS | |
| C3 no relation | abstain 300/300, 0 licensed | lookup 0.513 | PASS | |
| C4 absolute class | 1.000 | lookup 1.000 | **FAIL** | 7 relational-form laws licensed |
| C5 confound | 0 licensed after passive | | **FAIL** | 1 law licensed after interventions |
| C6 negative transfer | lived 0 wrong / ablation 400 wrong | cost ratio 1.18 | PASS | |

Analysis, without changing v1's verdicts:
* C1: the holistic HDC record of an equal-colour pair contains a value-independent trace:
  `(role0 ^ x) ^ (role1 ^ x) = role0 ^ role1`. Nearest-neighbour lookup therefore picks up a weak
  equality signal and reaches 0.667 on unseen colours. Memorized lookup in HDC is not pure lookup.
  The relational learner still separates by 33 points with zero wrong answers.
* C4: the licensed relational-form laws are conditionally TRUE under the ground truth, e.g.
  `r1.colour=3 & diff(colour) => 0` and `r1.colour=3 & same(colour) => 1`. The unconditional
  `same(colour)` was correctly SPLIT. The v1 criterion conflated relational form with falsity.
  Still a real defect: `r1.colour=3 & same(colour) => 1` is redundant with the licensed property law
  `r0.colour=3 => 1` and should not have earned utility (no Occam subsumption against licensed
  knowledge).
* C5: **a genuine false licence.** `r0.id=12 & r1.colour=2 => 1` (entity 12 has colour 2, so the
  conjunction is the confound in disguise): 28 passive supports, 4 lucky interventions (p = 1/16),
  intervention utility +2 bits. Gate 2 only required intervention utility > 0, without the
  multiple-comparison correction already applied to gate 5.

### E-R0 v2 pre-registration (written before any v2 run)
Changes to the learner: D016 (gate 2 must pay the selection cost on intervention data alone),
D017 (Occam subsumption: utility is measured against more general hypotheses that already explain
the episode). Changes to the protocol:
* Seeds fixed in advance: 1, 2, 3, 4, 5. Every criterion must hold on every seed.
* New global criterion, all conditions: **false-licence audit = 0**. Every licensed law is checked
  against the oracle on 2,000 fresh intervention episodes drawn from both pools; a law is false if
  it is wrong on any matching episode. In C5 every licensed law is false by construction
  (intervened outcomes are random).
* C1, C2, C3, C6: criteria unchanged from v1 (including the chance+15pp baseline bound).
* C4 replaced: (a) `r0.colour=3 => 1` licensed, (b) unconditional `same(colour)` not licensed,
  (c) false-licence audit = 0. Rationale: v1 analysis above; relational form is not falsity.
* C5 unchanged.

### E-R0 v2 result (2026-10-07, seeds 1-5, raw: `experiments/results/r0_v2_report.txt`)

| condition | pass | notes |
|---|---|---|
| C1 equality | 3/5 | relational 1.000 held-out, 0 wrong, 0 false licences on all 5 seeds. Fails on seeds 1, 3 only because memorized lookup scores exactly 0.667 (> 0.65 bound) |
| C2 order | 5/5 | |
| C3 no relation | 5/5 | 0 licensed, full abstention |
| C4 absolute class | 5/5 | property law licensed, unconditional same(colour) never licensed, 0 false licences |
| C5 confound | 5/5 | 0 licensed after passive and after interventions; same(colour) REVOKED |
| C6 negative transfer | 4/5 | lived agent never wrong; seed 3 learning-cost ratio 1.44 > 1.2 |

Verdict v2: **FAIL** (C1 baseline bound, C6 seed 3).

Mechanism of the C1 deviation: lookup predicts every equal-colour pair correctly. The holistic
record of an equal pair contains `(role0 ^ x) ^ (role1 ^ x) = role0 ^ role1`, a value-independent
equality trace, so nearest-neighbour retrieval generalises equality partially (150/150 equal pairs,
50/150 unequal pairs). Finding recorded: **in HDC, holistic memorized lookup is weakly relational
by construction.** It still makes 100 wrong answers out of 300 where the licensed relation makes 0.

Mechanism of the C6 deviation: the multiple-comparison threshold used the number of candidates
for the target across all contexts, so a lived agent paid for hypotheses it had searched in room-A
when licensing in room-B (D018).

### E-R0 v3 pre-registration (written before any v3 run)
Learner change: D018 (selection cost counts candidates tested in the current context).
Criteria:
* Seeds 1-10 (1-5 already seen, 6-10 new). Every criterion on every seed.
* C1, C2 baseline bound changed from "<= chance + 15pp" to "every baseline makes >= 20% wrong
  answers on the held-out set". Rationale: the intent was "the held-out set is not solved by
  class learning or lookup"; the v2 mechanism shows an HDC lookup can reach 0.667 legitimately
  without solving it. Relational requirements unchanged and tightened: held-out >= 0.95 **and zero
  wrong answers**, zero false licences, a licensed relational law with >= 3 pre-registered transfer
  successes.
* C3, C4 (v2 form), C5, C6: unchanged.

### E-R0 v3 result (2026-10-07, seeds 1-10, raw: `experiments/results/r0_v3_report.txt`)
C1 10/10, C2 10/10, C3 10/10, C4 10/10, C5 10/10, C6 9/10 (seed 7 cost ratio 1.39). Verdict **FAIL**.
Ratios over the ten seeds: 1.18, 1.00, 1.00, 1.00, 1.14, 1.06, 1.39, 1.20, 1.00, 1.11.
Mechanism: refined conjunctions inherited from room-A were counted as tested hypotheses in room-B
(D019).

### E-R0 v4 pre-registration (written before any v4 run)
Learner change: D019. Criteria identical to v3. Seeds 1-20 (11-20 new, never run).

### E-R0 v4 result (2026-10-07, seeds 1-20, raw: `experiments/results/r0.json`, `r0_v4_report.txt`)
**PASS on every condition and every seed (120/120).** Aggregates (`python experiments/aggregate_r0.py`):

| condition | class (random) | class (level) | memorized lookup | relational | oracle |
|---|---|---|---|---|---|
| C1 equality, unseen colours | 0.526 | 0.498 | 0.637 | **1.000, 0 wrong** | 1.000 |
| C2 order, unseen weights | 0.396 | 0.327 | 0.411 | **1.000, 0 wrong** | 1.000 |
| C3 no relation, new pairs | 0.499 | 0.501 | 0.510 (seen pairs 1.000) | abstain 300/300 | 1.000 |
| C4 property, new partners | 0.955 | 0.713 | 1.000 | 0.861, 0 wrong, 41.6 abstain | 1.000 |

* False-licence audit: **0 false licences** across all 80 audited runs (2,000 fresh episodes each).
* C5 confound: 0 licensed after 800 passive observations and 0 after 800 interventions in all 20
  seeds; unconditional `same(colour)` REVOKED in all 20.
* C6 negative transfer: lived agent with scoped licences 0 wrong answers in room-B (20 x 400
  queries); scope-ignoring ablation 8,000 wrong (every query); learning-cost ratio lived/naive
  mean 0.991, max 1.000.

The four outcomes are separated:
1. Class learning works where the structure is a property (C4) and fails on unseen relational
   values (C1, C2).
2. Memorized lookup is perfect on seen cases (all conditions) and fails on unseen ones; in HDC it is
   weakly relational for equality (0.637).
3. Relational learning licenses role-structured transforms (`same`, `diff`, `order`) only after
   independence, intervention, pre-registered transfer and utility gates, and never licenses in the
   no-relation, confound or passive conditions.
4. Transfer: licensed relations answer unseen values with zero errors; licences do not leak across
   contexts (C6).

Gate before Phase B: **PASS**.

---

## E-B Phase B: event memory and symbol grounding (pre-registered before any run)
Input: anonymous slot tokens (slots reshuffled every event), 5% value noise, 10% missing tokens.
Ground world: 16 objects (4 hidden types), channels colour, shape, size, texture, mark
(properties) and lit, pos (states); actions IDLE, TOGGLE(a), MOVE(a), COMBINE(a,b) where
COMBINE sets both lit to `colour(a) == colour(b)`.
Reminder (ARCHITECTURE 2.3): concept formation here is class learning and does not count as
progress by itself. B7 is the bridge to outcomes 3 and 4.

| gate | criterion |
|---|---|
| B1 structure | channel classes 7/7 correct; action-effect map correct (TOGGLE->lit, MOVE->pos, COMBINE->lit, IDLE->none); instance purity >= 97% and completeness >= 97%; live concepts <= 18 |
| B2 restoration | event recall >= 99% at 20% missing tokens (+5% flips); wrong recall <= 1% at 20/40/60/80% missing; property completion accuracy >= 98% |
| B3 novelty | 4 novel objects: >= 95% of their sightings never assigned to an old concept; each gets a born concept within 30 sightings; old completeness stays >= 97% |
| B4 noise discipline | junk slots in 5% of scenes: zero concepts whose majority sightings are junk |
| B5a merge | object first seen partially (2 channels) then fully: duplicate merged; its later sightings map to one live concept >= 95% |
| B5b split | two exclusive look-alikes differing only in a channel hidden until t=800: one concept before, split after; post-reveal purity >= 95% |
| B6 OS | real filesystem sandbox: ext, name -> property; size, content -> state; file concept purity >= 97% |
| B7 bridge | from raw percepts: a relational law on colour sameness is LICENSED for COMBINE; on 4 novel objects with unseen colours, predicted lit after COMBINE >= 90% correct vs world truth and <= 5% wrong |

### E-B v1 result (2026-10-07, seeds 1-5, raw: `experiments/results/phase_b_report.txt` of that run)
Development on seed 1 went through D021-D026 (each recorded with its evidence). Final v1 run on
seeds 1-5: **FAIL**.

| gate | pass | failure mechanism |
|---|---|---|
| B1 structure | 2/5 | 19-20 live concepts: 3-4 extra concepts with 3-5 sightings born from coincidental recurrences of noisy or junk percepts |
| B2 restoration | 0/5 | recall at 20% missing 289-299/300 (< 297) from margin abstentions; completion 95-98% at 60-80% missing from identity on sparse percepts with an incomplete candidate list |
| B3 novelty | 5/5 | |
| B4 junk | 4/5 | seed 3: two junk-born concepts (same mechanism as B1) |
| B5a merge | 4/5 | seed 4: the partial-view duplicate never formed, so nothing to merge (v1 criterion requires a merge) |
| B5b split | 4/5 | seed 2: a third accidental look-alike of the same type made the pair ambiguous before the reveal (test design) |
| B6 OS | 5/5 | |
| B7 bridge | 5/5 | relational COMBINE law licensed from raw percepts on every seed; unseen-colour predictions 200/200 correct |

### E-B v2 pre-registration (written before any v2 run)
Learner changes: D027 (exhaustive identity verification), D028 (chance-recurrence test for concept
birth), D029 (event memory at 32,768 bits, margin 640).
Test-design change: B5b world guarantees the pair is the only look-alike group
(`make_lookalike_pair`). Criterion change: B5a requires object 0's late sightings on one live
concept >= 95% **and no surviving partial-view duplicate**; a merge is reported but not required,
because a duplicate that never forms needs no merge (the v1 verdict is printed alongside).
All other criteria unchanged. Seeds 1-10 (6-10 new).

### E-B v2 result (2026-10-07, seeds 1-10, raw: `experiments/results/phase_b_v2_report.txt`)
**FAIL.** B2 failed on 10/10 seeds (recall at 20% missing 290-297, completion < 98% at high missing),
B6 OS failed on 10/10 (regression: D028 chance model counted one-of-two channel matches, so no file
concept could be born), B4 failed on 3, B1 on 2, B5b on 1. B3, B5a (v2 criterion), B7 passed 10/10.

Development after v2 (each change recorded with its evidence): D028a, D030, D030a, D031, D032,
D026a (withdraws D026), D033, D034, D030b, D035. An oracle bound for recall (true identities,
same corruption) is now printed with every B2 result: it recalls 300/300 at 0-60% missing and
289-297/300 at 80%.

### E-B v3 pre-registration (written before any v3 run)
* B5b criterion: pre-reveal sharing >= 90% and post-reveal purity >= 95%; the mechanism (split or
  birth) is reported but not required. Reason: v2 seed 5 individuated the pair by a birth with
  100% post-reveal purity; the gate is about the outcome "one concept turns out to be two".
* All other criteria unchanged from v2 (B2 recall >= 99% at 20% missing, wrong <= 1% at every
  level, completion >= 98% at every level).
* Seeds 1-15 (11-15 new).

### E-B v3 result (2026-10-07, seeds 1-15, raw: `experiments/results/phase_b_v3_report.txt`)
**FAIL**, only on B2 (9/15 seeds) and only on completion accuracy at 60-80% missing (97.2-98.0%).
B1, B3, B4, B5a, B5b, B6, B7: 15/15. Recall criteria held on every seed. Error sources (seed 1):
about half from object-level identity on two-token percepts with one noisy token, half from
event-level fills of one-token slots. Root cause: fill decisions used evidence counts that do not
bound the error rate (D036).

### E-B v4 pre-registration (written before any v4 run)
Learner change: D036 (completion gated by a calibrated posterior >= 98% from the measured noise
model). Criteria identical to v3. Seeds 1-20 (16-20 new).

### E-B v4 result (2026-10-07, seeds 1-20, raw: `experiments/results/phase_b_v4_report.txt`, `phase_b.json`)
**PASS: every gate on every seed (160/160).**

| gate | result (20 seeds) |
|---|---|
| B1 structure | channel classes 7/7 and action-effect map correct on all seeds; purity and completeness >= 99.8% |
| B2 restoration | recall 300/300 at 0-20% missing on most seeds (>= 297 on all), wrong <= 1% at every level; at 80% missing the system recalls ~125/300 and abstains on the rest (oracle with true identities ~292/300); completion >= 98% at every level (99%+ at 80% missing, by abstaining below 98% posterior) |
| B3 novelty | 0 novel sightings assigned to old concepts; novel objects born within 1-6 sightings |
| B4 junk | 0 junk-majority concepts |
| B5a merge | partial-view object ends on one live concept, no surviving duplicate |
| B5b split | look-alike pair shares one concept before reveal, purity >= 95% after |
| B6 real OS | ext/name property, size/content state; file purity 100% |
| B7 bridge | from raw noisy percepts, `same(colour)` and `diff(colour)` COMBINE laws LICENSED; unseen-colour predictions 196-200/200 correct |

Honest notes: B2 recall at 60-80% missing remains well below the oracle bound (the gap is
identity of sparse percepts); the system abstains there instead of guessing. B5a/B5b pass under the
mechanism-agnostic v2/v3 criteria; under the original v1/v2 mechanism criteria several seeds would
fail because no duplicate formed or the pair was individuated by a birth rather than a split.

Gate to Phase C: **PASS**.

### E-B v5 pre-registration (re-verification after D028b)
A bm-memory unit test written after v4 exposed a circular null in the chance-recurrence test
(D028b). The learner changed, so Phase B is re-run. Criteria identical to v4. Seeds 1-25 (21-25 new).

---

## Regression after Phase C development (pre-registered before running)
Learner changes since the last formal runs: D028b (Phase B), D037, D038/D038a, D039/D039a, D040,
D041, D042, D015a (relation engine). R0 and Phase B are re-run with criteria unchanged:
* E-R0 v5: seeds 1-20, criteria of v3/v4.
* E-B v5: seeds 1-25, criteria of v4.

## E-C Phase C: causal world model (pre-registered before any formal run)
Worlds: device worlds (confound: switch/indicator/lamp with a hidden timer; door: key/door/power
with a hidden condition), links microworld (16 tokens, 7 hidden classes), real filesystem with
hard links (12 files, 7 inode groups). Perception through the grounder (no labels), scene episodes
(all objects as roles), change-flag targets. Seeds 1-10.

| gate | criterion |
|---|---|
| C1 untrained combinations | (state, action) combinations never seen in training, every device's change predicted: >= 95% correct, <= 2% wrong |
| C2 intervention vs observation | do(TOGGLE indicator) lamp change: causal model >= 95% correct, <= 2% wrong; association model (lamp = indicator, from passive data) <= 60%; zero laws licensed from passive WAIT |
| C3 hidden condition | the phase-1 unconditional door law is licensed, loses its licence within 300 events after power starts varying, a power-conditioned door law is licensed, USE with random power >= 95% correct, <= 2% wrong |
| C4a latent cause (micro) | never-probed pairs: >= 90% correct on oracle-feasible pairs, <= 5% wrong overall, a licensed law over a latent channel; ablation without latent induction reported |
| C4b latent cause (real OS) | same criteria with real hard links (2,000 training probes) |
| C5 counterfactual | "had I toggled the switch instead": >= 95% correct, <= 2% wrong |

Oracle-feasible: a held-out same-group pair is inferable only if its members are connected through
same-group pairs probed in training; different-group pairs are always counted.
