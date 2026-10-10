# Observation, evidence and settlement: design review (observation-evidence-dev)

2026-10-10.

Branch `observation-evidence-dev`, from the frozen tag `bitmind-capability-h4-v0.1` (`b002dac`).
The capability-h4 I-series results and report are kept as a failure record. From here on, I1-I4
are **development data**; new held-out worlds come after a new freeze.

Evidence for section A:
* `experiments/results/i4_diag/`: read-only traces of I4 seeds 605, 602, 604, 610;
* `I4_605_laws.*`: a law-level re-run.

The traces reproduce the formal results exactly (605: 7 wrong; 602: 2 wrong).

## A. The causal path of the I4 failures (traced, not assumed)

### A1. What the traces show

| | traced | path |
|---|---|---|
| wrong settlements in stage 2 | **9 of 9** (605: 7, 602: 2) | each rests on exactly 3 "shared" episodes. At most 1 of them lies in the true conflict region; **0 of them contain a sensor misread**. The others are ordinary pairs, perceived correctly, with their true outcome |
| wrong answers before any action (stage 1) | **23 of 37** (605: 1, 602: 6, 604: 8, 610: 8) | **23 of 23** were D041 settlements: both values had applicable licensed laws, and the conflict was "settled" from stage-0 episodes. Only 3 of 23 queries had a misread in their own argument readings |

So the earlier report's explanation, "misread coating or material disguises non-region cases
as region evidence", is **not** what happened in the traced cases. It is corrected here. The
defect is real but sits one layer higher.

### A2. Seed 605, the first wrong settlement, step by step

| step | what the system had | status it should have had | what it was treated as |
|---|---|---|---|
| 1. raw readings | tokens of PRESS(15, 11), PRESS(15, 39), PRESS(43, 25); no misread | observations | (fine) |
| 2. percepts | concept-denoised properties (`bridge.rs`: props replaced by the concept prototype once mature); states (LIT) single raw readings | perceptual hypotheses with a source and a status | **facts**: no status survives into the episode |
| 3. laws applying | 3540 `same(material) => 1`, and on the 0 side bystander-relative laws such as 703577 `same(r0,r3;coating) & diff(r1,r3;coating) => 0` (8 counterexamples in 121 cases), 265693, 23186, and a particular law 66681 `r0.mark=117673 => 0` | each law's application is evidence **about that law** | (fine) |
| 4. the query's conflict | at query Q the 0 side is a *value group* of 15+ licensed laws. The one that makes Q a conflict is the coating law 4673 `r1.coating=558 => 0`; the group also holds the bystander and particular laws above | a conflict between specific laws | **one undifferentiated group** |
| 5. evidence ledger | D041 `shared_record`: an episode is shared if *any* law of group 0 and *any* law of group 1 applied there | an episode tests the claim at Q only if the laws carrying Q's conflict applied there | **promotion error 1:** "some member of group 0 failed in episode e" became "group 0's claim at Q was tested and failed in e" |
| 6. independence | D050 units = bindings over the *union* of every group law's relevant roles (bystander roles included, through the bystander laws) | units over the roles of the laws actually compared | **promotion error 2:** units inherit roles from laws that are not part of the comparison |
| 7. settlement | 3 distinct units, value 1 happened in 2 or 3 → answer 1 (truth 0) | abstain: the coating law was never tested against 3540 on a single shared case | wrong settlement |

### A3. Why noise matters, though not through the shared episodes

* The bystander-relative and particular laws are licensed despite real exceptions. Law 703577
  has 8 counterexamples in 121 cases (6.6%). At the agent's 1% floor the allowance would be
  about 4.5, so the target's tolerance was evidently raised: D040 raises it to the repeat noise
  measured by latent inducers. That is an inference from the licensed status, to be confirmed
  with a read-only print before implementation.
* Under noise, laws with genuine exceptions survive as licensed. Value groups fill with sloppy
  members whose own failures then masquerade as evidence about the query's claim. In I1-I3
  (noise-free) the same group-level rule exists, but those members are revoked or never
  licensed, so the defect stayed latent.
* D060 inherits the defect: its `conflict_probe` also works on value groups. In I4, 58 of 95
  "diagnostic" probes were outside the true conflict region: places where only sloppy members
  disagree.
* The perceptual layer still has a genuine gap. Percept status (corrected, completed,
  unrecognised; single raw state reading) is discarded when an episode is built (`bridge.rs`),
  so nothing downstream can tell a verified condition from an assumed one. In seed 610 one
  settlement used 14 shared episodes, 7 of them with misreads (it happened to be correct). The
  traced wrong settlements did not need this path; it is the next one.

### A4. Summary of the defect

Evidence loses its **subject**: which claim it tests. It also loses its **standing**: whether
the conditions it rests on were observed, corrected, inferred or unverified. Settlement then
counts evidence about other claims, of unknown standing, as if it tested this claim.

## B. Designs compared

| criterion | **1. claim-relative evidence ledger with percept provenance** (chosen) | 2. corroboration before promotion (k independent re-observations or channels) | 3. model-consistency vetting (licensed laws judge observation reliability) |
|---|---|---|---|
| separates perceived from actual | yes: every condition carries percept status; unverified conditions do not enter settlement | partially: a fact after k agreeing reads, but systematic or repeated-source errors pass | weakly: reliability is inferred from the laws being judged |
| transfers to other sensor errors without I4 special cases | yes: statuses come from the grounder's own concept and identity machinery; no channel names | depends on re-observation actions existing; repeats of one biased sensor are not independent | risky: circular whenever the laws are wrong |
| fits D060 | naturally: a probe's value = expected new verified claim-relative units for an undefeated opposing law, per cost | adds a separate "re-observe" path that D060 would have to bypass or wrap | needs a second scoring path |
| abstention safety | strong: answer only when every opposing law at Q is individually defeated on verified units, with a one-error margin | fixes misreads, **not** the traced claim-relevance defect | can over-trust a consistent but wrong law set |
| cost (16 GB) | small: a status byte per filler; ledgers computed on demand from existing episode lists | k x observations, more episodes and laws | extra passes over laws |

Choice: **design 1**. Design 2 is rejected as the primary mechanism: it would not have
prevented any of the 9 traced wrong settlements, which had no misreads, and it treats repetition
as independence. Its useful part is kept: re-observation can raise a percept's status when it
is consistent. Design 3 is rejected for settlement (circular) but may later serve to flag
percepts for re-verification.

Why this is an interface and not a patch: every future sensor, action or world feeds the same
four objects (observation → percept with status → claim-relative evidence item → settlement or
abstention), and action selection reads the same ledger. Nothing in it names a channel, a noise
rate or a world.

## C. Data structures and evidence flow

1. **Observation** (exists): `Event` pre/post tokens with source, time and slot. Immutable and
   kept in the grounder's store.
2. **Percept** (new status on each filler): `PerceptStatus` per (role, channel), one of
   * `Direct`: the reading equals the mature concept's prototype;
   * `Corrected`: the reading differed and was replaced by the prototype;
   * `Completed`: missing, filled by inference (D036);
   * `Unrecognised`: no mature concept, raw reading;
   * `StateRead`: a single raw state reading.

   It also carries the identity posterior (Q16) of the role's binding. It is carried from
   `Grounded` into `Episode` (it is computed today and then discarded).
3. **Evidence item** (computed on demand, never a free-floating counter): for a pair of laws
   (a, b) and an episode e, `EvidenceItem { episode, law_a, law_b, unit, outcome, standing }`.
   * `unit` = binding over the relevant roles of a and b only.
   * `standing` = `Verified` when every filler their conditions read is `Direct` or `Corrected`
     with an identity posterior >= the grounder's existing D036 confidence; otherwise
     `Unverified(reason)`.

   An audit API returns the items behind any answer.
4. **Settlement** for query Q (replaces D041's group rule). Let A(Q), B(Q) be the licensed laws
   applying to Q with values v_a ≠ v_b. Value v is answered only if:
   * for **every** law o applying to Q with value ≠ v, there is a law w applying to Q with
     value v such that the pair (w, o) has >= 3 independent **verified** units on which w was
     right strictly more often (D041a/D050 on the pair, not the group); and
   * the margin survives removing any single unit: right_w - right_o >= 2 (robust to one
     observation error).

   Otherwise: abstain. The abstention reports which opposing laws are undefeated and why
   (too few units; units unverified; not robust).
5. **Action selection (D060 on the ledger)**:
   * a candidate's value = expected number of new verified units for undefeated (w, o) pairs
     at live saved queries, per unit cost;
   * it is "expected" because a candidate whose relevant percepts are unverified in its preview
     earns its value only in proportion to their standing;
   * no separate path: `conflict_probe` becomes a view of the same ledger;
   * when nothing has value: no probe, and the agent keeps abstaining.

## D. Invariants (each enforced by a test)

1. **Observations are not facts.** An `Episode` keeps a percept status for every filler. No
   settlement or probe code reads a filler without its status.
2. **Evidence keeps its subject and standing.** Every unit counted in a settlement is a
   claim-relative `EvidenceItem` (law pair, unit, episode, standing) and can be listed through
   the audit API.
3. **Repeats and one systematic source do not multiply.** Units are bindings over the compared
   laws' relevant roles: repeats, bystander changes and the same object pair seen many times
   collapse to one unit. Unverified items do not count.
4. **Evidence about other claims never settles this claim.** If only laws other than a pair
   (w, o) applied in an episode, that episode adds nothing to (w, o). This is a reproduction
   test of the 605 pattern.
5. **Every opposing law must be individually defeated** before an answer; otherwise abstain.
6. **Unstable or contradictory percepts route to re-verification, not evidence.** An item with
   an `Unverified` standing is reported as "needs re-verification" and is excluded.
7. **Fragility check before every settlement:** the decision must survive removing any single
   unit.
8. **D060 has no bypass.** A probe is informative only if it can create a verified unit for an
   undefeated pair at a live query; probes where only unrelated laws disagree are not
   informative.
9. **No reliable evidence, no answer.** Conflicts without defeated opponents stay abstained,
   however many actions were taken.

### Thresholds and their meaning

| threshold | value | meaning and source |
|---|---|---|
| MIN_SHARED_INDEPENDENT | 3 | unchanged; D041a |
| fragility margin | 1 unit | "one observation may be wrong" |
| identity confidence | the grounder's existing D036 `FILL_POSTERIOR_Q16` | target accuracy for acting on inferred identity |

None of these is chosen from I4.

Sensitivity plan (development worlds only, reported, never tuned on hold-out): margin 0/1/2;
posterior threshold at the D036 value and one step either side.

## E. Memory

I4 peaks of 15 GB come from two sources:
1. the harness keeping three clones of a large stage-1 state;
2. law proliferation under raised tolerance.

Defences:
* the harness runs each policy in a separate process from a replayed state, removing the 3x
  clone;
* percept status costs 1 byte per filler; evidence items are computed from existing episode
  lists (no new resident ledger);
* the 16 GB watchdog stays in place;
* candidate completeness and determinism get regression tests (identical answers with and
  without the audit API; no candidate dropped).

## F. Kept and changed relative to v0.2 and D060

**Kept:**
* grounding, law generation, licensing, D050 bindings, MIN_SHARED_INDEPENDENT;
* D060's principle: choose actions that can separate live competing explanations;
* answers from a single licensed value group are unchanged.

**Changed, deliberately:**
* D041 becomes law-pair, verified, all-opponents-defeated and fragility-checked. Some
  previously settled answers will become abstentions. The regression battery must show whether
  any v0.2 gate relied on group-level settlement; that is a reportable trade-off, not something
  to hide.
* D060's informativeness reads the new ledger.
* Episodes carry percept status (an API addition).

## G. Verification plan

1. **Unit tests**, one per invariant, including:
   * a reproduction of the 605 pattern (sloppy bystander and particular laws failing elsewhere
     must not settle Q);
   * an unverified-percept exclusion test;
   * a fragility test;
   * a D060 no-bypass test.
2. **Development (I1-I4 and CW are development data now):**
   * re-run I4 seeds 601-615: target 0 wrong settlements, and report the resolution rate
     honestly (abstaining on everything is not success);
   * I1-I3 must keep correct resolution with 0 wrong;
   * new small development worlds for misidentification and state-reading noise.
3. **Regression:** the v0.2 battery, run clean under the 16 GB tier.
4. **Freeze**, then pre-register and build four new held-out families (written after the
   freeze):
   * **N1:** a new noise structure (misidentified objects; channels and surface unlike I4);
   * **N2:** systematic or correlated sensor error (one sensor biased in a fixed direction, so
     repetition does not help);
   * **N3:** safe abstention (no reliable discrimination available);
   * **N4:** probe cost and temptation (cheap but unreliable observation actions next to
     costlier reliable ones).

   Each is compared with random and WAIT-only.

   Non-negotiable criteria:
   * 0 wrong settlements over all held-out worlds;
   * a resolution floor on discriminable conflicts;
   * no regression on noise-free generalisation;
   * freeze guard before and after;
   * memory failures reported separately.

## H. Scope

**What this version is meant to secure:** in limited synthetic worlds, BITMIND keeps
observations apart from facts, counts only claim-relevant, verified, independent evidence,
chooses actions for reliable evidence, and abstains when that evidence is lacking.

**What remains missing after it:**
* perception of real sensors;
* re-observation strategies beyond what the world offers;
* history-dependent state (G2);
* the low-spec memory target;
* language;
* long-term memory;
* any general agency.

**Reusable interfaces:**
* `PerceptStatus` on fillers;
* `EvidenceItem` with an audit API;
* law-pair settlement with a fragility check;
* ledger-based action value.

Any future sensor or action plugs into these.
