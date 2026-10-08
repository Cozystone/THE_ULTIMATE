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

## Amendment 1: diagnosis result and mechanism (2026-10-08, before any learner behaviour change)
Diagnosis (`experiments/results/v03_k2_diagnosis.txt`, learner = v0.2 plus read-only accounting):

| run | 1k | 5k | 10k | 20k | dominant at the last checkpoint |
|---|---|---|---|---|---|
| H5 201 / 207 / 212 | 0.7 / 0.7 / 0.9 GB | 2.5 / 2.3 / 3.4 GB | 3.8 / 3.4 / 4.8 GB | 5.3 / 4.8 / 6.7 GB | conjunctions involving latent channels: 1.38M of 1.74M laws, 4.1 GB of 5.7 GB (seed 212); revoked + retired laws still hold 3.6 GB |
| real-OS seed 58 | 0.86 GB | 2.8 GB | 4.1 GB | - | laws over targets with > 64 outcome values: 2.8 GB of 3.6 GB; latent conjunctions 2.3 GB (overlapping) |

**Correction to section 1.** K2′-o is confirmed for the real-OS world. K2′-c is confirmed, but its
main generator is not sensed high-cardinality marks; it is hidden-condition refinement building
conjunction children over versioned latent features (every partition revision brings new latent
values, and each impure parent receives up to 8 children per refinement). In H5 all targets
have <= 8 outcome values, so K2′-o is absent there.

**Mechanism D055 (evidence-gated materialization), fixed now:**
1. **Single features.** For each (action, target, context), a bounded table counts outcomes per
   feature value; a law is materialized for a value only when its outcome counts carry
   LLR >= log2(M) + 4 bits against the base law's Laplace-smoothed distribution, with >= 3 cases.
   LLR is the information in bits, n x KL(empirical || baseline). M is the number of feature
   values tested for that target in that context (multiple-comparison protection, the same
   currency as D012). Base laws always exist.
2. **Refinement.** A hidden-condition child is created only if, on the parent's replayed episodes,
   the child's outcome counts carry LLR >= log2(M_r) + 4 bits against the parent's distribution.
   M_r is the number of candidate features examined for that target in that context.
3. **Materialization by replay.** A law that earns capacity is initialized from the stored episodes
   whose features contain its condition: counts, keys and counterexamples. As in D014, replay gives
   no transfer credit and no utility, so earning capacity is never counted as new evidence.
4. **Bounds.** A value table holds at most 4,096 values per feature family (further values are
   counted as untracked and cannot earn capacity) and at most 8 outcomes per value (others are
   pooled). Tables of retired latent channel versions are dropped. Nothing about a materialized
   law or its counterexamples is deleted.
5. **Explanation.** An abstention on a query whose features exist only as table entries is
   attributable as "deferred: insufficient evidence".

The constants (4 bits, >= 3 cases, 4,096, 8) are fixed here and change only by dated amendment.

## Amendment 2: baseline of the D055 gate includes a copy model (2026-10-08, before the regression of record)
Development measurement (`experiments/results/v03dev/`, D055 build):

| case | step | working set | laws |
|---|---|---|---|
| H5 seed 201 (v0.2: 5.3 GB, 1,322,844 laws) | 20,000 | 202 MB | 5,616 |
| real-OS seed 58 | 5,000 | 2.87 GB | 219,829 |

In the real-OS case, 216,678 of those laws predict targets with > 64 outcome values. A probe
leaves the partner file's content unchanged, so its post-probe content class equals its pre-probe
value. Against a 256-value base rate, any feature that isolates three episodes with the same post
value carries ~24 bits and passes the gate. This is mostly refinement conjunctions over latent
channels.

**Change (D055a).** The gate's baseline is a mixture of the best copy source and the smoothed base
(single features) or parent (refinement) distribution:
* for each (action, target, context) the engine counts how often each input filler (role, channel)
  equals the outcome;
* the best source with >= 20 cases contributes its measured rate q;
* the baseline probability of an outcome o is q x [o equals the source's value in that episode]
  + (1 - q) x p_dist(o).

Each deferred value and each refinement candidate accumulates its baseline log-likelihood case by
case. LLR = sum h log2(h / n) - sum over cases log2 p_baseline(case). Threshold, margin and case
minimum are unchanged.

Rationale: "the output repeats an input" is a generic chance-level model (persistence), no more
world-specific than a base rate; a hypothesis must beat it to claim information.

## Amendment 3: the gate baseline takes the better chance model per case (2026-10-08, before the regression of record)
Development measurement (`experiments/results/v03dev/*_a.txt`, D055a):
* H5 seed 201 at 20,000 steps rose from 202 MB / 5,616 laws (D055) to 1,036 MB / 66,931 laws.
* Real-OS seed 58 at 10,000 probes fell from 4.1 GB (v0.2) to 2.6 GB.

Cause: the copy source is chosen by aggregate hit rate. When it is a poor conditional predictor
(e.g. a filler that is usually 0 "matches" a mostly-0 change flag), the mixture assigns tiny
probabilities to the cases it gets wrong, the baseline becomes worse than the plain distribution,
and every hypothesis looks informative.

Change: per case the baseline uses the larger of p_dist(o) and p_mix(o) for the actual outcome. The
resulting LLR is a lower bound on the information against either chance model; it can only make
materialization stricter than D055 or D055a. Threshold, margin and minimum cases unchanged.

## Amendment 4: the hypothesis side of the gate is scored prequentially (2026-10-08, before the regression of record)
Development measurement (`experiments/results/v03dev/*_b.txt`, amendment 3):

| case | working set | laws |
|---|---|---|
| H5 seed 201, 20,000 steps | 185 MB | 1,790 |
| real-OS seed 58, 10,000 probes | 1.95 GB, still rising | 96,283 |

In the real-OS case, 74,087 laws (1.05 GB) predict target 3, the probed file's own post-probe
content class: a hash of a fresh random marker, unpredictable by construction.

Cause: the hypothesis side used plug-in likelihood (sum h log2(h/n)), which overfits large
alphabets. Three cases with three different values of a 256-value outcome score log2(1/3) each
against ~log2(1/356) under the baseline: ~20 bits of information that does not exist.

Change: the hypothesis side is scored prequentially (MDL). Each case is predicted from the
hypothesis's previous cases only, with Laplace smoothing over the target alphabet,
p = (count_o + 1) / (n + |A|), before being counted. LLR = sum over cases log2 p_hypothesis
- sum over cases log2 p_baseline. Single features accumulate both sums case by case; refinement
candidates are scored by a sequential pass over the parent's episodes in store order. Threshold,
margin, minimum cases and baseline (amendments 2 and 3) are unchanged.
