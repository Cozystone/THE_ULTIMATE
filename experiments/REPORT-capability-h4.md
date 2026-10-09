# BITMIND capability report: evidence-seeking conflict resolution (capability-h4)

2026-10-10, branch `capability-h4-dev`.

Learner frozen as `bitmind-capability-h4-v0.1` (`b002dac`). That is `bitmind-v0.2` plus:
* D060 (conflict-probe evaluation in `bm-relation`, an evidence-seeking term and `choose_probe`
  in `bm-agent`);
* `#[derive(Clone)]` on the engine, codebook and grounder.

The freeze guard passed before and after the formal runs.

Records:
* `PREREG-capability-h4.md`;
* `DECISIONS.md` D060;
* `results/cw_smoke/SMOKE_NOTES.md`;
* `results/cap_dev1/SUMMARY.md`;
* `results/cap_reg1/COMPARISON.txt`;
* `results/iseries/SUMMARY.txt` and the per-seed files;
* aggregator `experiments/aggregate_iseries.py`.

These are results in synthetic worlds only. They do not show agency, general intelligence or
real-world readiness.

## What was demonstrated

In noise-free worlds, when two licensed laws conflict on a query, the frozen learner:
* first abstains;
* then chooses actions on which its own licensed laws predict different outcomes and whose
  relevant binding is new to the conflict;
* answers only once D041 is met (>= 3 distinct relevant bindings) or one side has lost its
  licence to counterevidence it gathered itself;
* keeps abstaining when no informative action exists.

It does this with no oracle, no world-specific code, and no change to how answers are formed.

## Formal held-out I-series (seeds 601-615, 300 actions, identical stage-1 state per policy)

| world | agent: resolved correctly | route | wrong | still abstaining | random | WAIT-only | verdict |
|---|---|---|---|---|---|---|---|
| I1 new relational conflict | 145/145 (100%) | 122 D041 settlements, 23 active dissolutions | 0 | 0 | 22.1% (0 wrong) | 0% | **PASS** |
| I2 tempting irrelevant probes | 145/145 (100%) | 122 settlements, 23 active dissolutions | 0 | 0 | 8.3% | 0% | **PASS** |
| I3 no accessible evidence | 0/145 answered (correct behaviour) | n/a | 0 | 145 (100%) | 0 | 0 | **PASS** |
| I4 1% sensor noise on every channel | 55/76 (72.4%, criterion 80%) | 55 settlements, 0 dissolutions | **9** (14.1% of answers) | 12 | 35.5% (18 wrong) | 0% | **FAIL** |

Other measures:
* Settlement audit (no settlement under 3 units): 0 violations in every world.
* Identity leakage: 0 everywhere.
* Answers while a conflict was unresolved: 0.

Probe choice:
* Whenever an informative candidate was offered, the agent chose a diagnostic-new action:
  100% of those steps in I1, I2 and I4.
* In I2 the I2 gate (>= 80%) passes despite 4 salient SHAKE options per step.
* Of the diagnostic-new choices, these fell in the true conflict region:

  | world | in the region | of all diagnostic-new choices |
  |---|---|---|
  | I1 | 41 | 48 |
  | I2 | 42 | 45 |
  | I3 | 0 | 11 (no region pair is ever offered) |
  | I4 | 37 | 95 |

* In I3 the agent declared "no informative candidate" on 99.8% of steps.
* In I1-I3 a few diagnostic choices fell outside the region (7, 3, 11). Those are pairs where
  other licensed laws disagree. They led to no answer and no error.

### I4 failure, diagnosed (no learner change was made)

1. **Misperception makes non-region pairs look diagnostic.** In the agent's I4 probe log, 58 of
   95 diagnostic-new choices were outside the true conflict region. A misread coating or material
   in the preview makes an ordinary pair look like a conflict case. The executed episode is read
   again, and when the misread recurs, the episode counts as a D041 shared case carrying the
   *non-region* outcome. Three such distinct bindings are enough for a settlement in the wrong
   direction:
   * seed 605: 7 wrong;
   * seed 602: 2 wrong.

   The independence rule counts bindings correctly. It cannot tell a perceived from a true
   condition. This is the hazard first seen in CW smoke round 1; for development it was
   avoided by a noise-free world, not repaired.
2. **Stage 1, before any action:** under noise the v0.2 learner already answered 37 of 137
   region queries wrongly (I1-I3: 0 wrong). Only 76 conflicts formed in the first place. This is
   a pre-existing v0.2 weakness, not caused by D060, but it is a wrong-answer result and is
   reported.
3. **Resource failure:** I4 seed 606 aborted with "memory allocation of 1,081,344 bytes failed"
   before its stage 1 finished. It is counted as a resource failure and is not part of the I4
   totals. Other I4 processes peaked at 3.4-15.0 GB, close to the 16 GB cap. Noise multiplies
   the hypothesis population.

## Development and regression (before the freeze)

* **CW development world** (`results/cw_smoke/SMOKE_NOTES.md`, `results/cap_dev1`):
  * three smoke rounds were needed: the world was made noise-free, enlarged, and given a budget
    of 60;
  * agent 189/189 settled, random 0, WAIT 0;
  * the pre-mechanism v0.2 agent chose 0 diagnostic actions.
* **H4, agent-driven** (development): 133/135 answered correctly by **active dissolution** (one
  agent-chosen region probe refutes a deterministic law at tolerance 0), 2 unresolved, 0 wrong,
  0 settlements. The H4 gate (settlement only) remains FAIL by definition. The old H4 harness
  used an oracle fallback, now documented.
* **Regression of the D060 build:** 0 violations. Every phase total is identical to the v0.2
  record. Peak 5.8 GB.

## Memory and runtime (Workstation Research Tier, 16 GB cap)

| run | peak working set | wall time |
|---|---|---|
| Regression battery | 5.8 GB | about 14 h, heavy batteries sequential |
| I-series I1-I3 | 5.2-8.9 GB per seed process | |
| I-series I4 | up to 15.0 GB per seed process; 1 allocation failure (seed 606) | |
| I-series total | | about 14.7 h, one process at a time |

The watchdog terminated nothing. The seed-606 failure was an allocation abort inside the
process. v0.2 is not efficient and not laptop-ready.

## Remaining issues

* **Noise.** Conflict evidence trusts perceived conditions. Under sensor noise, misread scenes
  supply wrong shared evidence and wrong settlements (I4). Stage-1 wrong answers under noise
  are a v0.2 weakness. A fix belongs to a new version with new held-out worlds.
* **Settlement needs noise tolerance.** With tolerance 0 one counterexample refutes a law, so
  conflicts resolve by dissolution (H4). The CW and I worlds rely on the agent's 1% minimum
  tolerance (D020 assembly).
* **Development choices.** The CW world was designed through smoke rounds: noise-free, budget 60.
  The held-out results for I1-I3 come from the noise-free regime.
* **Memory.** The low-spec 2 GB target is still unmet with the v0.2 learner (the v0.3/v0.4 line
  is paused). I4 approached the 16 GB cap and one seed failed allocation.
* **Other open items:**
  * the corrected-E3 verification policy (active 0 vs fixed rotation);
  * G2 history-dependent state;
  * H4 settlement criterion.
