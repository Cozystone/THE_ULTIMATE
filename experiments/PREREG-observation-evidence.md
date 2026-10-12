# Pre-registration: observation-evidence held-out N-series

Written 2026-10-12, after the freeze of `bitmind-oe-v0.1` (commit `1afcb9f`) and **before any
N-series world exists**. Learner:
* `bitmind-capability-h4-v0.1`;
* plus D061 (percept standing, claim-relative settlement with a fragility margin, settlement
  audit);
* plus D061a (open questions weighted by blocked queries).

Design: `docs/observation-evidence-design.md`. Development data, never used for hold-out
claims: CW, C4b, H4, I1-I4.

## 1. Claim under test

> In limited synthetic worlds, BITMIND keeps imperfect observations apart from facts, counts
> only claim-relevant, verified, independent evidence when two licensed explanations conflict,
> chooses actions that can supply such evidence, and abstains when the evidence is lacking.

Not claimed: general intelligence, consciousness, real-world readiness, autonomous agency.

## 2. Families (worlds written after this document; seeds 701-715 each; budget 300 actions; about 12 offered candidates per step; saved queries re-judged every 10 actions)

Each world:
* uses new channel ids, labels and surface mechanics (not I-series, CW or H4);
* draws IDs, marks and object counts at random per seed;
* relabels every nominal value;
* draws the winning rule of the conflict region per seed;
* never offers a saved query pair as an action.

| family | what it tests | design (mechanism level) |
|---|---|---|
| **N0** noise-free generalization | no regression of I1-level competence | a new relational conflict world with no sensor noise |
| **N1** a new noise structure | percepts that diverge from facts through identity, not bit flips | **look-alike objects**: groups of objects share most properties, so the grounder can confuse identities. Plus low per-reading noise on channels unlike I4's |
| **N2** systematic sensor error | a sensor wrong in a fixed direction, so repetition does not help | for a fixed subset of objects, one property channel always reads the same wrong value. The bias channel is not the one the conflict laws use, but it can create spurious licensed laws |
| **N3** safe abstention | no reliable discrimination available | the conflict region is never offered. Noise as in N1, plus tempting non-diagnostic actions |
| **N4** probe cost and temptation | cheap but unreliable observations versus costlier reliable ones | two actions with the same mechanics: a reliable one (cost 1/4) and a cheap one (cost 1/8) whose outcome is read through a noisy sensor (25% flipped). Conflicts are asked about the reliable action |

## 3. Policies

* **agent:** the frozen learner's own action score (D060/D061 on the evidence ledger).
* **random:** uniform over the offered candidates.
* **wait:** WAIT only.
* **region oracle:** harness-only and never a learner policy. It picks an offered pair truly in
  the conflict region and measures how many conflicts are reachable under the same rules.

The agent and the baselines start from identical stage-1 states.

## 4. Measures, reported per family and seed

For each saved conflict, exactly one outcome:
* settled correctly;
* dissolved correctly (active or passive);
* **wrong settlement**;
* other wrong answer;
* unresolved (`Conflict`);
* unresolved (`Unverified`, needs re-observation).

Also reported:
* pre-action wrong answers on region queries (with their path: settlement or single-law);
* probe classes: diagnostic-new, redundant, non-diagnostic, and the share of diagnostic choices
  truly in the region;
* evidence quality of every answer: audited items, verified items, items truly in the region;
* identity leakage;
* per-process peak working set, wall time, resource failures.

## 5. Criteria (fixed now)

**Over all N families together:**
* **0 wrong settlements.** A single wrong settlement means no claim of safe settlement is made.
* **0 wrong answers on saved conflicts** during stage 2.
* identity leakage 0;
* settlement audit (every answer: each opposing law applying to the query defeated) 0
  violations;
* freeze guard passes before and after;
* memory over 16 GB is a resource failure, reported separately, never as a performance result.

**Per family:**

| family | criteria |
|---|---|
| N0 | resolved correctly >= 80% of saved; agent >= random + 30 pp; WAIT <= 10% |
| N1, N2, N4 | agent resolves correctly >= 50% of what the region oracle resolves (the reachable bound) and >= random + 20 pp; WAIT <= 10% |
| N3 | >= 95% of saved conflicts still abstain; "no informative candidate" in >= 90% of steps |
| N4, additionally | the agent's diagnostic choices use the reliable action in >= 90% of cases where both were offered for a live conflict |

**Reported but not gated:**
* pre-action single-law extrapolation errors, a known open issue;
* the agent/oracle ratio.

## 6. Rules

* The worlds are written after this document. No learner change after the freeze; the freeze
  guard runs before and after the formal runs.
* Before any formal seed, smoke checks may run on seeds 798-799 (outside the formal seeds)
  only for two things:
  * that the harness executes;
  * that each world is valid: the conflict forms (stage-1 `Conflict` abstentions), both
    sides have licensed laws, and the family's mechanism is present (look-alikes confused,
    bias present, cheap action noisy).

  A world that fails validity may be fixed before the formal seeds, documented in
  `results/nseries/SMOKE.md`. Agent-versus-baseline performance on the smoke seeds is not used
  to change worlds or criteria and is not reported as a result.
* If a world turns out not to test its claim (for example no conflict forms on most seeds),
  that is reported as a world-validity failure for that family. The world is not changed
  after formal results are seen.
* Each formal process runs one seed, sequentially; one memory-heavy process at a time.
