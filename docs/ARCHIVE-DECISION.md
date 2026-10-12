# Archive decision: the evidence-settlement research line is closed (2026-10-12)

Branch `observation-evidence-dev`, closing tag `bitmind-evidence-research-archive`.

The line is closed by direction change: the research moves to a persistent-agent prototype in a
separate repository. Nothing here is optimised further. Forbidden from now on: D062,
probe-weight tuning, I/N-series-specific fixes, threshold changes, new exception rules.

## What D060-D061a actually secured

* **D060 (action selection for conflicts).** On noise-free worlds the agent chose actions on
  which its licensed laws disagreed and settled conflicts only after 3 distinct relevant
  bindings:
  * held-out I1/I2: 145/145 correct, 0 wrong, against random 22% and 8%;
  * I3: 100% safe abstention when no evidence was reachable.
* **D061 (claim-relative evidence).** A read-only trace showed that the I4 wrong settlements
  (9/9) and the traced pre-action wrong answers (23/37) came from group-level aggregation:
  * exceptions of other laws with the same predicted value were counted against the law that
    created the conflict;
  * no sensor misread was involved.

  D061 counts only shared, verified cases of each pair of laws, requires every opposing law
  that applies to the query to be beaten with a one-error margin, and keeps percept standing.
  Results:
  * I4 wrong settlements went 9 → 0, pre-action wrong 37 → 4;
  * I1-I3 unchanged;
  * the v0.2 regression stayed identical.
* **Reusable tools:** read-only provenance traces, a harness-only region-oracle bound, and the
  16 GB watchdog.

## What it did not solve

* I4 resolution under D061 + D061a: 49.3%, against random 6.3% and WAIT 0%. The harness-only
  region oracle (world truth, same offers and budget) reaches 71.8%. The old 80% criterion is
  unreachable under the claim-relative rule, and the agent gets about 69% of the reachable
  bound.
* 4 pre-action wrong answers remain: single-law extrapolation into unobserved regions, not
  settlement.
* Not addressed: the tolerance allowance at small n, and the value of probes whose preview is
  unverified.
* The N-series was stopped before its criteria were evaluated (`experiments/results/nseries/STOPPED.md`).
  Its partial results are not judged.
* The evaluation design measured "how much is solved within a fixed budget in a resettable
  world". It never measured whether the system predicts and recovers better as experience
  accumulates.

## Why this direction is not optimised further

Every further gain would come from tuning evidence thresholds, probe weights and world-specific
offers against a fixed benchmark. That answers neither the new research question (continuous
adaptation over a lifetime) nor the old one more than it already has. The structural lesson is
recorded. More tuning would fit benchmarks.

## Assets the new project may consider (candidates only; each must be re-justified and re-tested in isolation)

* **Evidence keeps its subject and standing:**
  * an observation is not a fact;
  * evidence names the claim it tests;
  * repeats of one source are not independent.
* **HDC binding and item memory**, only if it serves the new hypothesis.
* **Read-only provenance tracing**, the harness-only oracle bound, the watchdog, and the
  pre-registration discipline.

## Assumptions not carried over

* Resettable episodes, fixed action budgets, "queries" as the unit of evaluation.
* "0 wrong answers" as a global objective; high-confidence claims are audited separately instead.
* The licence/law machinery (D001-D061a) as the agent's core. No code is copied into the new
  repository.
