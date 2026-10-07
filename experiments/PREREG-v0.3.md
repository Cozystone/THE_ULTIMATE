# BITMIND v0.3 pre-registration: K2′ hypothesis budgeting

Written 2026-10-08 at `main` = `c800627`, before any v0.3 learner change.
`bitmind-v0.1` (`f8ff695`) and `bitmind-v0.2` (`059b433`) are immutable.

**Mission.** Stop hypothesis growth that no evidence asked for, especially over high-cardinality
nominal values. Preserve:
* relational transfer;
* counterevidence;
* safe abstention;
* every v0.2 protection (D049 measurement levels, D050 relevant bindings).

**Out of scope:** H4/D041 settlement, active exploration, language, perception, agency claims. If
v0.3 accidentally changes H4 behaviour it is reported, not tuned.

## 1. K2′ hypothesis

**Believed cause.** Candidate generation is unconditional. For every episode, `observe` creates
(or touches) a law for every single feature × every target, and refinement and latent versions add
conjunctions on top. Nothing asks whether a feature has ever reduced uncertainty about that target.
Two paths make the law population grow with the number of distinct nominal values seen rather than
with what has been learned:

* **K2′-c (condition side).** Absolute features on high-cardinality nominal channels (object marks,
  concept identities, content hashes, latent-class labels, versioned latent channels) create one
  law per value per target.
* **K2′-o (outcome side).** A law predicting a high-cardinality nominal outcome keeps one outcome
  bin, with its key sets, per observed value.

v0.2 measured the outcome side in real-OS runs. But the H5 world has only a binary state outcome
and still reached 677,412 laws after 2,000 steps, so the condition side is probably at least as
large. Section 2 decides this before any mechanism is designed.

**Prediction if the cause is confirmed and removed:**
* law and candidate counts grow sublinearly with steps under repeated irrelevant novelty;
* peak working set stays <= 2 GB at 20,000 steps;
* no correct answer is lost, no wrong answer or identity leakage is added.

## 2. Diagnosis before design (read-only accounting)

Measured at 1k, 5k, 10k and 20k steps where feasible, on:
* H5 development seeds 201, 207, 212 (smallest and largest v0.2 peaks plus one with wrong answers);
* the real-OS case C4b seed 58.

Attribution:
* laws by lifecycle state (candidate, contested, provisional, licensed, revoked/compacted,
  pruned husks);
* laws by condition type (absolute vs relational; channel; channel cardinality class: <= 8, 9-64,
  > 64 distinct values; latent vs sensed);
* laws by outcome cardinality of their target;
* outcome bins and key sets (logical items and allocated capacity);
* index, condition lists and lineage;
* episode store, per-episode feature lists, latent inducer observations;
* process working set (current and peak).

Artifact: `experiments/results/v03_k2_diagnosis.txt`. The mechanism targets the confirmed source;
if the diagnosis contradicts section 1, the hypothesis is amended (dated) before implementation.

## 3. Mechanism constraints (design fixed after diagnosis, principles fixed now)

* A feature/target pair earns materialized hypotheses only when that feature measurably reduces
  uncertainty about the target beyond an explicit chance model. The gate has multiple-comparison
  protection over the number of feature families tested for that target.
* Until then, evidence is held in a bounded family-level summary (integer counts), not in one law
  per value. Episodes stay in the store, so a hypothesis that earns capacity later is initialized
  by replay; replayed cases are counts, not transfer evidence (D014 rule).
* Counterevidence is never deleted. Every deferral or retirement keeps enough to explain an
  abstention, and a revived hypothesis carries its prior counterexamples.
* Nominal identity keeps equality and object continuity (D049); no lookup family per identity value
  unless it earns capacity like any other feature.
* No channel names, no world names, no blacklist ("ignore hashes" is forbidden). Thresholds are
  set before the first development run and changed only by dated amendment.

**Unit tests** (each must fail on `bitmind-v0.2` where possible):
1. a 256-value nominal nuisance channel produces no per-value law family;
2. a high-cardinality channel with a real sparse dependency (one value predicts the outcome)
   stays learnable;
3. identity permutation invariance;
4. counterevidence survives deferral and retirement;
5. candidate and law counts stay bounded under repeated irrelevant novelty;
6. equality-based object continuity still works.

## 4. Data roles

* **Development data:** H5 (Phase H seeds 201-215) and the real-OS memory cases (C4b seed 58, G1/G2
  real-OS runs). No longer formal evidence.
* **Non-negotiable regressions:**
  - all unit tests;
  - R0 1-25, B 1-25, C 46-60, D 1-15, E 46-60, F 1-15 and 101-115, G1-G4 101-115;
  - H1-H4 201-215.

  Criteria per phase:
  - identity leakage 0;
  - wrong answers not above the v0.2 regression of record;
  - correct answers on every gate not lower than v0.2 by more than 1 percentage point, with every
    abstention change reported;
  - gate verdicts as in v0.2. An H4 dissolution is never reported as D041 settlement.
* **Formal held-out:** J1-J4 (section 5), written only after the v0.3 freeze, seeds 301-315.

## 5. Formal held-out J-series (after the freeze)

| world | content | criteria (fixed now) |
|---|---|---|
| J1 long-horizon nominal nuisance | 20,000 steps; changing high-cardinality labels/hashes that are causally irrelevant; one low-cardinality genuine relation | peak working set <= 2048 MB; established relation queries >= 90% correct, <= 2% wrong; identity leakage 0; end accuracy >= mid accuracy - 2 pp |
| J2 sparse high-cardinality signal | a 256-valued nominal channel where one or a few values cause an effect, embedded among high-cardinality noise | queries on the signal values >= 80% correct, <= 2% wrong; nuisance values never answered as causes (wrong <= 2%) |
| J3 identity/label permutation stress | J1/J2 structure under fresh random relabelling and object permutation | identical answers >= 99%; identity leakage 0 |
| J4 memory and provenance audit | long run with deferral and retirement active | 0 counterexamples lost; 0 evidence counts above the number of supporting episodes; every hypothesis that answers after revival keeps its prior counterexamples; every abstention attributable (no licensed law / conflict / deferred family) |

**Metrics, every run, every seed:**
* peak and current working set at checkpoints;
* law count and candidate count, family summaries;
* retained evidence items;
* correct, safe abstention, insufficient evidence, wrong, identity leakage;
* wall time.

**Memory budget:** 2 GB at 20,000 steps, unchanged. A revision needs a documented measurement that
the benchmark itself, not the learner, makes it infeasible, and must be dated before any formal run.

## 6. Rules

* Learner crates frozen at the v0.3 tag; a freeze guard runs before and after every formal J run.
  No learner change after formal J runs begin; failures are reported and fixes belong to v0.4.
* Stop rule: if the 2 GB H5-development target cannot be met without harming correctness, the
  tradeoff is reported and work stops; no indefinite layering of retention patches.
* Constitution check and all tests green before the freeze.
