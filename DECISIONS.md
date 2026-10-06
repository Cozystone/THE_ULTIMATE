# DECISIONS

Every design choice that affects relation licensing, counterevidence, transfer or the constitution
is recorded here with options, choice and reason.

---

## D001 Main engine language: Rust (2026-10-06)
Options: Rust, C++, Zig. Choice: Rust 1.95. Reason: memory safety for a long-lived daemon,
first-class `std::arch` AVX2/AVX-512 intrinsics (AVX-512 stabilised in 1.89), portable code,
no GC pauses. C FFI remains available.

## D002 Default hypervector width: 16,384 bits, 32,768 supported (2026-10-06)
`Hv<W>` is const-generic over u64 words. `Hv16k = Hv<256>`, `Hv32k = Hv<512>`. Width is a type
parameter so capacity experiments can compare widths without code changes.

## D003 Similarity convention (2026-10-06)
`similarity(a,b) = D - popcount(a ^ b)`. Random pairs sit at D/2 with sigma = sqrt(D)/2
(64 bits at 16k). A cleanup hit is accepted only if its distance is below D/2 - z*sigma
(default z = 6). Below the noise floor the system abstains. Reason: abstention is the guard
against ATANOR's confident wrongness.

## D004 Bundling is integer-only (2026-10-06)
Two bundlers: a bit-plane (vertical carry-save) counter for unweighted majority, and an i32
per-bit counter for weighted/signed accumulation. Ties broken by a deterministic seeded
hypervector. No floats.

## D005 Determinism (2026-10-06)
All atomic hypervectors come from a seeded xoshiro256** stream keyed by FNV-1a of the symbol.
Same seed + same symbol = same vector across runs and machines. No dependency on `rand`.

## D006 Relation = group transform between role fillers (2026-10-06)
Options considered:
(a) one hand-added primitive per relation type (ATANOR's RELEQ path);
(b) learn arbitrary pair tables (memorized lookup);
(c) relation as the substrate's own group element mapping filler a to filler b.
Choice: (c). XOR difference gives equality for any value including unseen ones; permutation
offset gives order and difference magnitude for ordinally encoded channels. One mechanism,
any channel, any pair of roles, including latent channels. Whether a relational feature matters is
decided only by licensing. Risk recorded: relations that are neither equality nor offset
(e.g. arbitrary pair tables) are not expressible as a single transform. They must appear as
latent-cause hypotheses or remain unlicensed.

## D007 Licensing lifecycle and gates are fixed before any relational code (2026-10-06)
States CANDIDATE, CONTESTED, PROVISIONAL, LICENSED, RESTRICTED, REVOKED, SPLIT and the five gates
(independence, intervention, held-out transfer, counterevidence kept, utility) are the contract in
ARCHITECTURE.md section 2.4. Thresholds live in one `LicensePolicy` struct and every change to
them is logged here. Candidates may steer exploration but may never answer.

## D008 Smallest falsification test before event store and OS integration (2026-10-06)
R0 must separate class learning, memorized lookup, relational learning and transfer, and must
include a no-relation control and an observation-only confound control. Specified in
EXPERIMENTS.md E-R0. No large event store or OS adapter is built until R0 passes.

## D009 Scope-bound licences (negative-transfer guard) (2026-10-06)
A licence carries the set of context signatures in which it was verified. In an unseen context
the relation is treated as CANDIDATE there until re-verified. Reason: ATANOR DS1 showed old
relations answering confidently wrong in new worlds.

## D010 Cleanup search uses a prefix cascade; per-query thread fan-out rejected (2026-10-06)
Measured (E-A1): exact scan of 10^5 x 16k vectors is DRAM-bandwidth bound at 5.2 ms. A contiguous
2,048-bit prefix slab with a z = 3 pre-filter, then full-width verification, takes 0.38 ms with no
recall loss at up to 40% noise. 16 scoped threads per query took 3.1 ms (spawn overhead). Choice:
`cleanup_fast` is the default for large memories; exact `cleanup` stays for small memories and
tests. Risk: an item with an unluckily noisy prefix can be missed; the cascade is approximate and
its recall is re-measured whenever the prefix width changes.

## D011 Usable bundle capacity is a hard structural budget (2026-10-06)
Measured (E-A2): ~300 items at 99% recall at 16k, ~600 at 32k. Every bundled structure (event
record, concept prototype, scope set, interaction profile) must stay within budget or be split
hierarchically with cleanup per level. Episode memory therefore stores one vector per episode in a
cleanup memory, never one giant bundle of all episodes.

## D012 Coincidence vs law: utility must pay for the search (multiple-comparison guard) (2026-10-06)
Problem (ATANOR's open question): with hundreds of candidate conditions, some will look pure on a
handful of episodes by chance (P(5 identical binary outcomes) = 1/16). Purity + support alone
licenses coincidences.
Choice: gate 5 utility is prequential description length. For every episode, before updating,
each matching law pays `log2(1/p_law(actual))` bits with Laplace-smoothed counts; its competitor
pays the same. Utility = bits saved versus the **best competing hypothesis**:
* single-feature law: competitor = the base-rate law of the same action and target;
* conjunction law: competitor = the better of its two single-feature parents.
A law is licensable only if `utility >= log2(#candidate laws for this target) + margin` (margin
4 bits). The hypothesis must save more bits than it costs to have picked it out of the space
searched. Consequences: (a) a conjunction that adds nothing beyond a more general law earns no
utility and is never licensed (subsumption); (b) chance-pure conjunctions with few supports cannot
cover the selection cost.

## D013 Licences are per context; scope extension is cheaper than first licensing (2026-10-06)
Evidence, status and transfer records are kept per context signature. A law can be LICENSED in
context X and REVOKED in Y (aggregate status RESTRICTED). In an unseen context the law starts as
CANDIDATE: it may steer exploration but not answer. Scope extension to a new context requires
`min_scope_support` consistent interventions there and zero counterexamples, not the full gate
set again. This keeps transfer fast while forbidding confident wrongness (ATANOR DS1).

## D014 Hidden-condition search is triggered by counterevidence, not by novelty (2026-10-06)
Conjunction candidates are not enumerated blindly. When a single-feature law becomes impure, the
engine replays its episodes from the immutable store and ranks every co-present feature by how
much it reduces outcome entropy among those episodes (competing explanations of the
counterexamples). The top 8 become child candidates with lineage `Refined{parent}`. Their counts
are initialised by replay, but utility and transfer accrue only from live episodes after
creation (no self-confirmation from data that produced the hypothesis). When a child is licensed
and the parent's counterexample rate is above tolerance, the parent becomes SPLIT.

## D015 Compositional novelty for transfer tests (2026-10-06)
An episode is a held-out compositional case for a law if it contains a role-filler
(role, channel, value) never present in that law's supporting episodes in that context. The
prediction is recorded before the outcome is read. Only eligible laws (no counterexamples,
>= 3 independent supports) are tested.

## D016 Gate 2 pays the selection cost on intervention data alone (2026-10-07)
Evidence: E-R0 v1 C5 licensed `r0.id=12 & r1.colour=2 => 1` from 28 passive observations plus
4 lucky interventions (+2 bits). Choice: a law needs `utility_int >= log2(#candidates) + margin`,
the same threshold as gate 5 but computed only on the agent's own interventions. Passive data may
generate and rank hypotheses; it can never pay for a licence.

## D017 Occam subsumption against existing knowledge (2026-10-07)
Evidence: E-R0 v1 C4 licensed `r1.colour=3 & same(colour) => 1`, which adds nothing beyond the
licensed `r0.colour=3 => 1`. Choice: on each episode, a law's prequential competitor is the
lowest-loss hypothesis among (a) its base/parent competitors (D012) and (b) every other matching
law in the same context that is at least PROVISIONAL, has zero counterexamples and strictly
greater coverage (ties: lower id). A hypothesis earns utility only where it predicts better than
the most general credible hypothesis that already covers the case. Specific laws subsumed by
general ones stop accruing utility; equally specific peers do not cancel each other because the
comparison is only against strictly more general (larger coverage) laws.

## D018 Selection cost is counted per context (2026-10-07)
Evidence: E-R0 v2 C6 seed 3, lived/naive learning-cost ratio 1.44. The licence threshold
`log2(#candidates)` counted every candidate for the target in every context, so experience in
room-A made licensing in room-B more expensive. Choice: the look-elsewhere count is the number of
laws for this target that have evidence in the current context, i.e. the hypotheses actually tested
here. Hypotheses searched elsewhere are not part of this context's selection.

## D019 Conjunction hypotheses are activated per context by that context's own search (2026-10-07)
Evidence: E-R0 v3 C6 seed 7, cost ratio 1.39. Refined conjunctions created to explain
counterexamples in room-A were matched and counted in room-B, inflating room-B's hypothesis
space and its selection cost. Choice: a refined hypothesis takes part in a context only if (a) that
context's own hidden-condition search selected it, or (b) it is licensed somewhere (then it is
tested in the new context through borrowed scope trials). When a context's search selects an
existing hypothesis, its evidence there is initialised by replay of that context's episodes.
Directly generated single-feature hypotheses remain active everywhere.
