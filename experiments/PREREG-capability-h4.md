# BITMIND capability pre-registration: evidence-seeking conflict resolution (capability-h4)

Written 2026-10-08 on branch `capability-h4-dev`, rooted at `bitmind-v0.2` (`059b433`), after
port commit `939ee7b`. That commit adds only read-only diagnostics, H-series worlds and harness,
the corrected E3 twin, and the v0.3/v0.4 records. The learner is the v0.2 learner. No
D055-D059 behaviour is present. `main`, `v0.4-dev` and every tag are untouched.

Operating mode: capability research on the stable v0.2 learner under the Workstation Research
Tier (section 7). The v0.3/v0.4 memory-optimization line is paused as a non-blocking track.

## 1. Claim

When two licensed explanations apply to the same query and predict different outcomes, BITMIND:
1. abstains while the conflict is live;
2. given an action budget, chooses actions on which its own licensed laws predict different
   outcomes and whose relevant binding adds a new independent shared unit to that conflict;
3. answers the query only when the conflict is settled by D041 (>= 3 distinct relevant shared
   bindings, strict majority), or when it dissolves because one side lost its licence to
   counterevidence; the two routes are reported separately;
4. keeps abstaining when no informative action is available;
5. never answers wrongly while a conflict is unresolved.

Not claimed: agency, general intelligence, real-world readiness, language, perception.

## 2. Definitions

For each saved query (a query abstained with `Abstain::Conflict` in stage 1), the final outcome
is exactly one of:

| outcome | definition |
|---|---|
| **U** unresolved safe abstention | still `Conflict` (or no answer) at the end of the budget |
| **D** dissolution | answered after the conflict view for the query is empty: one side is no longer licensed (revoked, contested or split) |
| **S** settlement | answered while both sides are still licensed, by D041: every disagreeing pair has >= 3 independent shared units and the answer's group won strictly more of them |
| **W** wrong | any answer that is not the true outcome, whatever the route |

D and S are further split by evidence source:
* **active:** at least one shared unit, or the decisive counterexample, came from a probe the
  agent chose;
* **passive:** otherwise (world drift, random probes, or episodes outside the agent's choice).

Probe classes, judged by the agent's own laws at decision time:
* **diagnostic-new:** at least two applicable licensed value groups disagree on the probe, and
  the probe's relevant binding (union of both groups' relevant roles, D050) is not yet among
  that pair's shared units;
* **diagnostic-redundant:** they disagree, but the binding is already counted. This covers
  repeats and bystander-only variation;
* **non-diagnostic:** the applicable licensed laws agree, or no licensed law applies.

The truth class is reported too: whether the probe lies in the true conflict region. It comes
from the world and is used for reporting only.

**Settlement audit:** every S answer is checked for >= 3 independent shared units for each
disagreeing pair at answer time. Any violation is a failure.

## 3. Mechanism constraints

* Learner change limited to:
  * a general conflict-probe evaluation in `bm-relation` (read-mostly; the D041 rules, D050
    units and MIN_SHARED_INDEPENDENT = 3 are unchanged);
  * its use in the `bm-agent` action-selection score, plus action/evidence provenance records.
* `predict` keeps its v0.2 semantics: an answer only from one licensed value group, or from
  D041 settlement. Acting never by itself creates an answer.
* No world names, hard-coded probes, hidden state, oracle answers, test labels or privileged
  flags. The agent sees only grounded episodes and previews of candidate actions, which are
  pre-action scenes without outcomes.
* v0.2 identity and label protections (D049, D050) stay as they are.
* Constitution:
  * no runtime LLM, SLM, Transformer or pretrained embedding;
  * no float on the cognitive path;
  * HDC, fixed point and symbolic evidence only.

## 4. Development data

All development data is listed here; nothing else is development data.
* existing C4b (real OS, seeds 46-60) and H4 (seeds 201-215) worlds and harness;
* a new development conflict world **CW** (`bm_worlds::conflict_dev`), seeds 501-510, written
  and smoke-tested before any learner change (section 5).

Known prior facts, to be preserved and not relabelled:
* v0.2 abstains safely under unresolved conflict (H4 stage 1: 160/160, wrong 0).
* v0.2 H4 stage 2 had 0 D041 settlements. Its 154 later correct answers were dissolutions.
* The v0.2 H4 stage-2 probe chooser in the harness used world knowledge (the `evidence` set of
  region pairs) as a fallback. That is oracle access. Any new development run removes it, and
  no new run may contain it.
* H4 objects are noise-free. With noise tolerance 0, one counterexample makes a licensed law
  Contested (`evaluate`: counters > noise allowance = 0), so D041 settlement is structurally
  unreachable there. Settlement requires measured noise (D020, the agent's own sensor noise
  floor) that lets both laws stay licensed while shared evidence accumulates.
* C4b-2 was never exercisable (0 saved conflicts after K1/K4).

## 5. Development world CW: requirements and smoke tests (before any learner change)

CW has:
* objects with fresh random IDs and marks, randomized nominal labels and nuisance channels, and
  a random object count;
* one probe action on (a, b), plus irrelevant actions;
* sensor noise on the observed state channel (the agent measures it itself through the
  grounder, D020);
* two licensed laws that apply to the same queries and disagree in a conflict region;
* at least 3 reachable, distinct, relevant diagnostic bindings per conflict class;
* irrelevant, redundant and bystander-only probes available;
* query states where abstention is correct before evidence.

Smoke tests (each must pass before formal development seeds; failures change the world, never
the criteria, and are documented):
1. The intended conflict forms: stage-1 queries abstain with `Conflict` on most seeds.
2. Both laws have real support (independent >= 5, interventions, transfer); neither is vacuous.
3. Reachable observations can support or weaken each law.
4. Diagnostic bindings are distinct under the D041/D050 unit rules (>= 3 per conflict class).
5. Bystander changes and repeated probes do not add units (measured on the engine's own
   `conflict_evidence`).
6. A disagreement-reducing oracle-free reference policy beats random and WAIT-only. The
   reference uses only `explain` on previews; it is a harness check of the world, not the
   mechanism.
7. No shortcut by label, object order or index: predictions are unchanged under ID and label
   relabelling (permutation test as H3).

## 6. Criteria

### Development gate (seeds CW 501-510, H4 201-215 agent-driven, C4b 46-60)
The mechanism proceeds only if all hold:
* 0 wrong answers in unresolved conflicts, and 0 wrong overall on saved queries;
* agent probes resolve correctly (S or D-active) more saved conflicts than random-probe and
  WAIT-only, summed over CW seeds (agent >= random + 20 percentage points);
* settlement audit: 0 settlements with < 3 units;
* repeated or bystander probes create no settlement (unit tests and a CW ablation);
* identity leakage 0;
* every process stays below 16 GB.

Not every case must settle; U is valid where disambiguation is impossible.

Unit tests (they must fail or be inexpressible on `bitmind-v0.2` where they test new behaviour):
1. a probe where two laws disagree is selected;
2. probes predicted identically by both laws are ignored;
3. the agent refuses to act (returns no probe) when every candidate is uninformative;
4. repeated probes of one binding count once;
5. three distinct relevant bindings reach the D041 threshold;
6. no answer before the threshold while the conflict is live;
7. safe abstention is not regressed.

### Regression gate (before any freeze)
The v0.2 battery, run clean and non-overlapping under section 7: R0 1-25, B 1-25, C 46-60,
D 1-15, E 46-60, F 1-15 and 101-115, G 101-115, H1-H5 201-215. Every gate, seed and route is
compared with the v0.2 record (`v02reg3`, `v02_phase_h`).

Blockers:
* a gate that passed in the record now fails;
* any new wrong answer;
* any identity leakage;
* any lost counterexample;
* per phase, correct < record - 1 pp, or abstentions > record x 1.10 + 5.

Baseline failures that may stay failing:
* E1 seed 54;
* E3 under the corrected twin (v0.4 amendment 1a);
* H2 seeds 210-211;
* H4 (its v0.2 harness used oracle fallback; the agent-driven H4 is reported separately as
  development data);
* H5 on the 2 GB metric.

The 2 GB / 20,000-step metric is recorded as a future low-spec target, not a gate.

### Formal held-out I-series (written only after the freeze; seeds 601-615; budget 300 actions)
Each world offers about 12 candidate actions per step (probes on random pairs, irrelevant
actions, WAIT). Policies compared on identical stage-1 states:
* the agent (frozen learner + `bm-agent` choice);
* random among the offered candidates;
* WAIT-only.

Saved queries are re-judged every 10 actions.

| world | design | criteria (fixed now) |
|---|---|---|
| **I1** new relational conflict | surface mechanics unlike CW and H4; randomized IDs, labels, nuisance, object counts | correctly resolved (S + D-active) >= 80% of saved; 0 wrong while unresolved; wrong <= 2% of answers; agent >= random + 30 pp; WAIT-only resolves <= 10%; leakage 0 |
| **I2** tempting irrelevant probes | as I1, plus frequent, salient, non-diagnostic candidates (high-variance nuisance targets) | the I1 criteria, plus diagnostic-new >= 80% of agent probes taken while a diagnostic candidate was offered |
| **I3** insufficient accessible evidence | no offered candidate touches the conflict region | 0 wrong; >= 95% of saved queries still abstain at the end; the agent declares "no informative probe" in >= 90% of steps |
| **I4** three reachable independent bindings | sensor noise measured by the agent; the conflict region small against each law's support | correctly resolved >= 80%; S (D041) route >= 70% of correct resolutions; settlement audit 0 violations; dissolutions reported separately; agent >= random + 30 pp |

Any learner change after the first formal I run invalidates the run. A freeze guard runs before
and after. Every outcome class is reported, failures included.

## 7. Resource policy (Workstation Research Tier)

* Hard per-process working-set budget: 16 GB. An external monitor polls every 30 s, logs current
  and peak working set, and terminates any learner process above 16 GB (logs preserved;
  classified as a resource failure).
* At most one memory-heavy learner process at a time (C, G and H batteries, any process expected
  above 2 GB). Light batteries (R0, B, D, E, F) may run alongside.
* The 2 GB / 20,000-step H5 metric is recorded as a future low-spec target. No claim is made that
  v0.2 is efficient or laptop-ready.

## 8. Order

1. This document (committed before any learner change).
2. CW world and smoke tests; smoke findings documented.
3. Mechanism plus unit tests.
4. Development validation.
5. Regression battery.
6. Freeze (`bitmind-capability-h4-v0.1`, constitution check, freeze guard).
7. I-series worlds written and run.
8. Report.

Stop rule: at most two development rounds of the mechanism. If round 2 fails the development
gate, nothing is frozen and the trade-off is reported.
