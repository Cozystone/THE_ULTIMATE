# BITMIND v0.4 pre-registration: K5, the licence cost over every hypothesis examined

Written 2026-10-08 on branch `v0.4-dev` at `f44c37f`. That commit is `bitmind-v0.2` (`059b433`)
plus ported records, H-series worlds and harnesses, and one read-only accounting method; it holds
no v0.3 learner behaviour. `bitmind-v0.1`, `bitmind-v0.2` and `main` (`01fbffb`, the failed v0.3
record) are not rewritten.

**Mission.** Test whether the v0.3 memory gain (evidence-gated materialization, D055) can coexist
with v0.2's statistical scepticism and correctness. The system may store few hypotheses, but it
must remember how many it considered before licensing any one of them.

**Out of scope:** D041 settlement, G2 history-dependent state, language, perception, agency, active
exploration, new interfaces, unrelated memory features.

## 1. K5

**Definition.** D012 licenses a law only if its utility reaches log2(n_cand) + margin bits, where
`n_cand` is the number of laws holding evidence for that target in that context. Every licence is
therefore a selection from n_cand candidates, and the charge is a Bonferroni-style
multiple-comparison cost.

* In v0.2, n_cand was large because every examined feature value became a law. It also shrinks
  when laws are pruned or retired.
* In v0.3, D055 kept examined values in deferred tables and refinement candidates in transient
  tallies; neither counted. n_cand fell from ~10^5-10^6 to ~10^3. The charge fell by ~8 bits, and
  weak object-specific laws were licensed: H1 leakage 153, e.g. `r1.mark=6574 => unchanged` with
  15 bits of utility.

**Candidate lifecycle**, per (action, target, context):
* **examined:** a feature value or refinement candidate whose outcome evidence for the target has
  been scored at least once;
* **deferred:** examined, held only in a bounded value table (no law), not yet materialized;
* **materialized:** given a law (counts, keys, counterexamples) because it passed the gate;
* **refined:** a refinement candidate that became a child law;
* **retired / revoked:** a materialized law that left the active set (pruned, dead latent version)
  or failed its evidence (revoked, split). Its examination still counts.

**Effective hypothesis family** F(action, target, context) = every candidate examined for that
target in that context since the engine started:
* every distinct single-feature value examined, whether deferred, materialized, untracked beyond
  the table bound, or later retired;
* every refinement candidate examined in any hidden-condition search for that target and context,
  online or in sleep;
* every law created for the target and context by any other path.

The licence cost of any law about that target in that context is log2(|F|) + margin.

Rationale: a licence certifies that a law survived a search over F, so the search, not the
storage, sets the multiple-comparison charge. Retirement does not un-examine a hypothesis, so |F|
never decreases. Note: v0.2's pruning decrement of n_cand is therefore removed as well.

**Bounded memory.** |F| is one u64 counter per (action, target, context), plus the per-family value
counts the D055 table already keeps (<= 4,096 values per family; values beyond the bound are counted
as examined and untracked). No rejected candidate is stored. The counter survives sleep, compaction,
retirement, revival and latent version retirement.

## 2. Diagnose C3 and E3 before any repair

Read-only provenance diagnostics compare the v0.3 candidate (`main`, learner `28fac94`) with v0.2
on the C3 door world (seeds 46-60) and the E3 verification-action measure (seeds 46-60). For each
failing seed, the failure is assigned to exactly one class:
1. parent law never examined;
2. parent examined but deferred;
3. parent materialized but never licensed;
4. refinement child never generated;
5. child generated but gated out;
6. replay/transfer evidence accounting;
7. sleep compaction or revocation interaction;
8. other, named concretely.

Artifact: `experiments/results/v04_c3_e3_diagnosis.txt`, one paragraph per failure class with seeds
and the decisive law summaries. A repair is designed only for an evidenced class; K5 is not
presumed to explain C3 or E3.

## 3. Mechanism (implementation after section 2)

* **D055 (re-introduced deliberately, not inherited).** The gate, prequential hypothesis scoring,
  better-chance-model baseline with copy model only for > 64 outcomes, replay without transfer
  credit, gate information carried as initial utility, and SET_CAP 1,024 are ported from the
  v0.3 record as decided there, with the same constants.
* **D056 (new).** The K5 family counter replaces `per_target_ctx` in the D012 licence cost and is
  incremented for every examined value (deferred or materialized), every refinement candidate
  examined, and every law created. It is never decremented.
* **C3/E3 repairs.** Only for diagnosed classes, each a dated amendment before the regression of
  record.

**Unit tests** (discriminating ones must fail on the v0.3 learner where applicable):
1. deferring 100,000 weak candidates leaves the licence cost at log2(>= 100,000), not that of
   100 candidates;
2. a real sparse dependency still materializes and licenses;
3. object-specific laws stay unlicensed in an H1-like trap (a tempting mark law with modest
   utility);
4. family counts survive sleep, compaction, retirement and revival;
5. refinement candidates are charged, while a valid low-cardinality refinement still licenses;
6. identity permutation invariance and counterevidence retention.

## 4. Development validation (all existing worlds are development data)

Battery:
* R0 1-25, B 1-25, C 46-60, D 1-15, E 46-60;
* F 1-15 and 101-115;
* G1-G4 101-115;
* H1-H5 201-215.

Run from a clean build with no overlapping processes of other builds. Reported by phase and seed:
* correct, abstain, wrong, identity leakage;
* family count;
* deferred, materialized, licensed and revoked counts;
* current and peak working set;
* wall time.

**Proceed only if all hold:**
* identity leakage 0 everywhere;
* every gate that passed in the v0.2 regression of record (`v02reg3`, `v02_phase_h`) passes;
  E1 seed 54 and H2 seeds 210-211 were v0.2 failures and may stay so;
* no wrong answer beyond v0.2's count on any gate;
* per phase, correct >= v0.2 correct - 1 percentage point, and abstentions <= v0.2 abstentions x
  1.10 + 5 (no pass bought with abstention);
* no counterexample count ever decreases (H5 audit);
* H5 peak working set <= 2,048 MB at 20,000 steps on all 15 seeds;
* F1 after consolidation >= 99% at 30% missing, 0 wrong.

**Stop rule.** At most two development regression rounds. If round 2 fails any condition, the
trade-off is reported, nothing is frozen, and no K-series world is created.

## 5. Freeze and formal K-series (only after a clean development result)

Commit, all tests, constitution check, tag `bitmind-v0.4`, freeze guard. Then write fresh worlds
(seeds 401-415), never seen during development:

| world | criteria (fixed now) |
|---|---|
| K1 deferred-candidate multiplicity trap: many irrelevant high-cardinality values plus a tempting object-specific rule with modest apparent utility | the object-specific rule is never licensed; identity leakage 0; wrong <= 2% |
| K2 sparse real signal amid a large nuisance space | signal queries >= 80% correct, <= 2% wrong; peak <= 2,048 MB |
| K3 long horizon, 20,000 steps, independently randomized nuisance | peak <= 2,048 MB; established relation >= 90% correct, <= 2% wrong; leakage 0; end accuracy >= mid accuracy - 2 pp |
| K4 refinement transfer, structurally unlike C3/E3 | the hidden condition is found and transfers: >= 90% correct on held-out condition cases, <= 2% wrong |
| K5 permutation and provenance audit | identical answers under fresh relabelling >= 99%; 0 counterexamples lost; every licence and every abstention explainable (licensed law, conflict, or deferred / insufficient) |

No learner-crate change after the first formal K run; a freeze guard runs before and after. Every
outcome class is reported.

## Amendment 1 (2026-10-08, after the section-2 diagnosis, before any learner change)

Evidence: `experiments/results/v04_c3_e3_diagnosis.txt`.

**1a. E3 measurement.**
* The E3 twin is corrected: it replays the same store, runs the same sleep, and tests its own
  sleep-created laws. Previously it tested law indices of another engine and had never slept.
* Under the corrected measurement the v0.2 learner fails E3 on 15/15 seeds (active 30, twin 30):
  every option, WAIT included, matches some of ~1,500 unlicensed sleep hypotheses.
* E3 is therefore moved to the list of v0.2 baseline failures (with E1 seed 54 and H2 seeds
  210-211). It is still run and reported for every seed, but it is not a must-pass gate.
* This is a change to a criterion fixed in section 4. It is made before any v0.4 learner change
  and on a re-measured baseline, and it is stated in the final report.
* E3 will not be "repaired" by restoring hypothesis over-generation.

**1b. D057, prospective transfer for deferred hypotheses (the C3 repair, class 6).**
A deferred value keeps the pre-registered transfer record that its law would keep:
* a bounded relevant-situation key set (`SET_CAP`, saturating conservatively);
* up to 3 distinct relevant-binding keys;
* ok / fail counters;
* the failed trials (episode ids, bounded).

A case of the value is a prospective trial only if all of these hold:
* the value has >= `GATE_MIN_CASES` cases;
* no counterexample so far (all its cases share one outcome; stricter than a law's
  within-noise test);
* >= 2 distinct relevant bindings (general-law semantics; particular values earn no trial);
* the case's relevant situation (action arguments plus the feature's roles, as D050a) is
  provably unseen for the value.

The prediction (the value's single outcome) is fixed before the outcome is counted. On
materialization the law's transfer record receives these tallies and failed trials. Replay
still gives no transfer credit (D014). Nothing is credited retroactively: every trial was a
prediction of a hypothesis that existed, deferred, at that time.

**1c. Order of implementation.**
1. D055 as recorded in v0.3 (`28fac94`).
2. D056 (K5 family counter).
3. D057.
4. The six K5 unit tests plus a D057 unit test (a deferred law in a closed world that is
   materialized late can still be licensed; a value with a counterexample earns no prospective
   trial).
