# BITMIND v0.2 pre-registration

Written 2026-10-07 at `main` = `37b5c0f`, before any v0.2 learner change.
`bitmind-v0.1` (`f8ff695`) is immutable. Phase G showed that v0.1 does not generalize reliably:
G4 9/15 seeds with 25 wrong answers, F1 replication 11/15, C4b stage 2 not demonstrated. v0.2
must make "a reusable relation, not an identity lookup" mechanically hard to fake, and show it on
worlds that do not exist yet.

## 1. Hypotheses

**K1 (label arithmetic).** v0.1 computes order and offset (`Order`, `Delta`) transforms on every
channel (`features.rs`), including identity labels, marks, file names, content hashes and latent
class labels. Laws such as `r0.ext=2 & delta(r0,r1; identity)=-6` memorise particular pairs while
looking relational. *Prediction:* with order/offset transforms restricted to channels whose
measurement level is ordinal (and that are not identifiers), no law over label arithmetic can
exist. Relational predictions then become invariant under any consistent renaming of identities,
and G4 wrong answers attributable to label arithmetic disappear.

**K4 (inflated independence).** v0.1 counts gate-1 independence, transfer novelty and shared
conflict cases with whole-scene signatures. Bystander objects therefore create "new" cases,
so a handful of distinct actor/partner pairs looks like 50-74 independent supports.
*Prediction:* when evidence is counted per relevant binding (the objects the law actually
connects), repeated probes of one pair count once, bystanders add nothing, and the G4 laws behind
wrong answers (`r0.colour=0 & r1.mark=300`, ...) fail gate 1.

## 2. Mechanisms to implement (architecture level, no world-specific branches)

* **D049 Measurement levels.** Every channel is *nominal* unless the producing sensor declares
  it *ordinal*: a channel schema reported by the adapter, like a unit, not a law of the world.
  Nominal channels support absolute value, same and different only. Ordinal channels add order
  and offset. Channels the learner creates itself are always nominal: the identity channel,
  latent-class channels, and binding identities.
* **D049 identifier veto.** A channel declared ordinal whose values are injective over >= 8
  distinct bound entities is an identifier and is treated as nominal (logged). A re-encoded ID
  therefore gains no arithmetic.
* **D050 Relevant bindings.** An `Entity` carries an optional `binding` (the identity of the bound
  object, supplied by the producer: grounded concept or world object). If none is supplied, the
  binding falls back to a hash of the role's fillers. The relevant roles of a law in an episode are
  the action's argument roles, the roles its condition mentions, and the target's role when the
  producer reports it. For each supporting episode a law records:
  - a *binding key*: action plus the bindings of the relevant roles;
  - a *situation key*: action plus all fillers of the relevant roles, with bystanders excluded.
* **D050 gate 1.**
  - *General law:* support spans >= 2 distinct binding keys. Independence = distinct binding keys,
    >= `min_independent` (5, unchanged).
  - *Particular law:* support has exactly one binding key, i.e. it is about specific objects.
    Independence = distinct situation keys, >= 5. It is marked *particular*, never counted as
    relational or transfer progress, and reported separately.
* **D050 transfer.** A held-out case is one whose binding key is new to the law (general law) or
  whose situation key is new (particular law). Bystander novelty no longer counts.
* **D050 conflicts.** A shared case is independent when its binding key is new. The binding key
  here is taken over the union of both laws' relevant roles. The threshold stays 3.
* Counterexamples are counted per episode as before and are never discarded.

Unit tests (each must fail on the v0.1 learner):
1. Consistent permutation of object identities leaves relational predictions unchanged.
2. A world whose outcome is an identity offset licenses no law, and new pairs are abstained.
3. Equality-based identity continuity (same object in two roles) can still be licensed.
4. A fixed relevant pair with changing bystanders cannot license a general law.
5. Three distinct relevant bindings can settle a D041 conflict, while one pair probed many times
   cannot.

## 3. Data roles

* **Development data:** former G4 (MagnetWorld), seeds 101-115. Former G1-G3 and G5 seeds
  101-115 are regression data.
* **Regression battery (criteria unchanged):** R0 seeds 1-25; B 1-25; C 46-60 including the
  two-stage C4b; D 1-15; E 46-60; F 1-15; G1-G4 and F on 101-115.
* **Held-out:** new worlds H1-H5 (section 5), written only after the v0.2 learner is frozen,
  formal seeds 201-215.

Every regression or development run records, per seed:
* gate results;
* correct, wrong and abstain counts;
* abstention reasons;
* wall time;
* peak working set (every bench binary reports it at exit).

## 4. F1 and K2 before the freeze

* **F1** (one-shot recall fell to 89-93% on seeds 101-115). Before any mechanism change, a
  diagnostic bin attributes every F1 miss to one class:
  - retrieval/cleanup (true event outside the candidate list);
  - alignment loss (true event in the list but outscored);
  - collision (oracle cannot separate it from another stored event);
  - calibration abstention (true event best but posterior < 98%);
  - harness.
  A mechanism change is allowed only for a class shown to be a learner fault, recorded as a
  decision, followed by a regression run.
* **K2** (~10 GB per process on long real-OS runs). Peak memory is measured by phase, and the
  largest retained structures are counted: laws and their 2 KB hypervectors, bins and episode
  lists, stored episodes, per-episode feature lists, latent observations. A memory-budget design is
  written:
  - bounded episodic retention;
  - compaction into licensed laws and prototypes;
  - provenance-preserving summaries;
  - revival on demand;
  - counterexamples never deleted.

  K2 is implemented before the freeze only if it does not confound the K1/K4 comparison; the
  decision is recorded either way.
* **K3** stays deferred unless it blocks H1-H5.

## 5. Held-out validation (after the freeze)

Freeze: tag `bitmind-v0.2`, guard script comparing the learner crates to the tag, constitution
check and all tests green. Worlds written after the tag must use:
* randomized, counterbalanced identities and labels;
* nuisance bystanders;
* an automatic anti-leakage check: no hidden variable may be a function of any identity, mark or
  label value. Each world asserts this at construction.

| world | purpose |
|---|---|
| H1 | positive relational transfer to never-seen objects and values |
| H2 | latent cause structurally unlike G4 (an asymmetric relation over hidden classes) |
| H3 | identity-permutation invariance: identical structure, identities and labels re-drawn; answers must not change |
| H4 | conflict resolution where >= 3 independent relevant bindings are reachable by the agent's own probes |
| H5 | long horizon under a memory budget (>= 20,000 steps) |

Every answer is classified as one of:
* correct relational generalization;
* safe abstention;
* insufficient evidence (abstained, evidence below a gate);
* wrong confident answer;
* lookup/identity leakage (answered by a particular law on a new object, or the answer changes
  under identity permutation);
* memory-budget failure.

**Criteria (fixed now):**
* **H1, H2 and H3:** >= 90% correct on established cases, <= 2% wrong, 0 leakage.
* **H3:** identical answers under permutation in >= 99% of queries.
* **H4:**
  - 0 answers while a conflict rests on < 3 independent bindings;
  - >= 90% of saved conflicts settled and answered correctly within the step budget;
  - 0 wrong answers.
* **H5:**
  - peak working set <= 2 GB;
  - accuracy at the end >= accuracy at mid-run minus 2 percentage points;
  - 0 counterexamples lost (provenance audit).

Abstention never counts as correct.

## 6. Rules

* No learner change after the first formal held-out run starts. Failures are reported as they are,
  with a diagnosis. Fixes belong to v0.3 with new held-out worlds.
* No world-specific branches, no hard-coded answers, no natural language, no pretrained
  components, no LLM. Constitution check before each freeze.
* Development seeds and every threshold above are fixed by this document. A change requires a
  dated amendment written before the run it affects.

## Amendment 1 (2026-10-07, before any v0.2 learner change)
Running the unit tests against `bitmind-v0.1` showed that test 3 (identity continuity) passes on
v0.1. That is intended: it guards a capability that must survive, so "each must fail on v0.1"
applies to tests 1, 2, 4 and 5 only. Test 1 as first written also passed on v0.1 because its
training pairs gave identity arithmetic no consistent coincidence. It now draws half of the
training pairs as neighbours inside a colour block, so in engine A's numbering same-colour pairs
often differ by one identity unit, which is the trap v0.1 fell into in G4.

## Amendment 2 (2026-10-07, before any F regression run of v0.2)
F1 diagnosis (`experiments/results/v02_f1_diagnosis.txt`, seeds 101-115, v0.1 protocol): every
wrong recall (28) and most abstentions (147 of 184) are retrieval failures. The true event is not
among the 32 candidates because it was encoded when its objects had no mature concept. There are
no collisions and 4 alignment ties. With one consolidation after training
(`v02_f1_diagnosis_reconsolidated.txt`): 0 wrong; at 30% missing only 2 of 3000 queries missed, both
abstentions.
Change: the learner gets a sleep-time consolidation (D052) that re-grounds stored events from their
raw records. The F1 protocol becomes "experience, the system's sleep consolidation, then recall";
the v0.1 no-sleep numbers are reported alongside on every seed. Criteria unchanged.
