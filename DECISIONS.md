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

## D020 Observation-error tolerance is explicit and bounded by measured sensor noise (2026-10-07)
With noisy perception every law sees counterexamples. ARCHITECTURE 2.3 lists "sensor or
observation error" as the first competing explanation of a prediction error. Choice:
`LicensePolicy.noise_tol` (default 0 for deterministic worlds) is set from the grounder's measured
noise floor. Counterexamples, failed transfer trials and failed scope trials at or below that rate
are attributed to observation error; above it they restrict, revoke or split as before. Every
counterexample is still retained. The tolerance is never a free knob: it must come from a
measured sensor noise rate, and the value used is reported with every result.

## D021 Ambiguity is not novelty (2026-10-07)
Evidence: E-B first run, 36 live concepts for 16 objects and 138 merges. A percept that two
concepts explain equally well (typically because the distinguishing channel is missing) was
treated as unknown and created a new proto-concept, which then caused more ambiguity. Choice:
identity returns Hit, Ambiguous or Unknown. Ties prefer a born concept over an unborn proto, and
between born concepts only a 3x larger history breaks the tie. Ambiguous percepts and percepts with
fewer than two property channels never create proto-concepts; they are grounded without identity.

## D022 Event recall keys are identity-independent and require a margin (2026-10-07)
Evidence: E-B first run, 10/300 wrong recalls with no missing tokens. Stored event vectors keyed
objects by concept id, which later merges changed. Choice: an object key is the bundle of its
denoised property values. A recall is accepted only if it is below the noise floor and at least
256 bits (4 sigma) closer than the runner-up; otherwise the memory abstains.

## D023 Revocation requires counterevidence beyond measured noise (2026-10-07)
Evidence: E-B second run. `diff(colour) => lit 0` had 21 counterexamples in 573 (3.7%, inside the
5% measured noise) but was REVOKED for good after one noisy counterexample among its first six
cases. Choice: the counterexamples noise can explain among n observations are bounded by the
binomial mean plus three standard deviations at the measured noise rate. Only the excess counts
toward CONTESTED and the revoke rate. With noise 0 (deterministic worlds) the old behaviour is
unchanged: any counterexample contests.

## D024 Concept birth pays a selection cost (2026-10-07)
Evidence: E-B second run, one concept born from junk tokens that recurred three times by chance
among ~200 live proto-concepts. Same multiple-comparison problem as D012. Choice: birth requires
MDL saving greater than log2(number of active protos and concepts).

## D025 No frequency prior in identity ties (2026-10-07)
The 3x-sightings tie-break of D021 picked the more frequently seen object for sparse percepts and
produced wrong completions. Since ambiguous percepts no longer create protos (D021), ties between
two born concepts are now always refused. Candidate list widened from 6 to 12.

## D026 Redundant proto-concepts are absorbed on contact (2026-10-07)
Evidence: E-B third run. A novel object's first noisy sightings created two protos (one with a
wrong size, one with a wrong texture). Every clean sighting then tied between them, was refused as
ambiguous, and the object stayed unlearned for ~400 ticks until both protos decayed. Choice: when
two unborn protos explain a percept equally well and never appeared together, the younger is
absorbed into the older (lineage kept). Ties between born concepts are still refused (D025).

## D027 Identity verification is exhaustive over active concepts (2026-10-07)
Evidence: E-B v1 completion errors at high missing rates. With a top-k candidate list, a sparse
percept consistent with two objects could be verified against only one of them and accepted.
Choice: verify every active concept by unbinding. Cost is linear in active concepts (hundreds,
microseconds each); the cleanup memory still orders candidates. A later scaling step must keep
completeness (e.g. an inverted index from (channel, value) to concepts), never silently drop it.

## D028 Concept birth requires recurrence beyond coincidence (2026-10-07)
Evidence: E-B v1, 3-4 extra concepts per seed born from 3-5 coincidental recurrences of noisy or
junk percepts. MDL with a selection cost (D024) was not enough because the chance of a match
depends on how common the matched values are. Choice: under a null model where every channel
value is drawn from its observed marginal frequency, compute the probability that a percept agrees
with the proto on all channels or all but one, multiply by the percepts seen since the proto was
created, and require sightings > expected + 3 sd + 1. Integer Q32 arithmetic. This is the concept
level version of the law-level multiple-comparison guard (D012): a pattern is real only if it
recurs more than chance predicts.

## D029 Event memory uses 32,768-bit vectors (2026-10-07)
Evidence: E-B v1 recall abstentions at 20% missing. Signal grows with D and noise with sqrt(D);
doubling D improves the best-vs-runner-up separation by sqrt(2). Margin rescaled to 5 sigma at 32k.

## D028a Chance model mirrors the identity rule (2026-10-07)
Evidence: E-B v2 B6 regression, 0 file concepts. With only two property channels the
"all-but-one" term counted any percept matching one channel as a chance match (p ~ 0.39). The
identity rule never accepts one-of-two. Choice: the one-mismatch term is included only when the
remaining channels still reach the identity rule's required matches.

## D030 Event recall: HDC proposes, fact agreement verifies (2026-10-07)
Evidence: E-B v2 B2, recall abstentions at 20% missing and wrong recalls at 80% missing with a
distance-margin rule. A fixed bit margin is wrong for correlated candidates (near-duplicate
events), whose distance difference has much smaller variance than independent ones. Choice: the
32k event memory returns the 8 nearest events; each is scored by the exact number of shared facts
(object signature x channel x value x phase, action, arguments) with the grounded query. Accept the
unique best if it leads by >= 2 facts and explains >= half of the query's facts, else abstain.
Same principle as identity (D027): HDC proposes, decomposition verifies.

## D031 Birth evidence counts only sightings consistent with the pattern (2026-10-07)
Evidence: E-B v2 seed 3, a junk concept born with 3 sightings after proto absorption (D026)
pooled disagreeing junk percepts. Choice: recurrence support = second-smallest per-channel majority
count (one mismatch tolerated) instead of raw sightings; both the minimum and the chance test
(D028) use it.

## D032 Event memory reconsolidation (2026-10-07)
Evidence: E-B v2 wrong recalls with no missing tokens: events stored while their objects were
still unknown were encoded from noisy raw values; later queries ground the same objects through
mature concepts, so the stored vector and the query no longer agree. Choice: stored event vectors
can be rebuilt from the immutable raw events with the current grounding (reconsolidation). The
raw event store is never modified. Phase E sleep will run this; the B2 test runs it once after
training.

## D026a D026 withdrawn: indistinguishable is not identical (2026-10-07)
Evidence: E-B seed 5 produced chimera concepts (mark of one object, shape of another) after D026
absorbed two protos of different objects because a sparse percept fit both. Choice: a tie between
two unborn protos marks the percept as unexplained; it may seed a new proto, and the chance test
(D028/D031) plus decay discard the losers. The deadlock D026 addressed is broken because a clean
percept now seeds a clean proto that wins later sightings outright.

## D030a Event recall by event-level alignment (2026-10-07)
Evidence: oracle bound (true identities, same corruption) recalls 300/300 at 20-60% missing while
the system recalled 250/300 at 40%: per-object identity of sparse percepts failed first and the
whole event then mismatched. Choice: HDC proposes 32 candidate events from the grounded query; each
candidate is verified by aligning the raw query slots to its stored objects (greedy one-to-one on
token agreement) and counting agreeing tokens, action and arguments. Unique best with a lead of
>= 2 tokens and >= half the query explained, else abstain. The oracle bound is now printed with
every B2 result.

## D033 Identity needs at least two agreeing channels (2026-10-07)
Evidence: E-B completion errors at 80% missing: a single surviving token that was noise matched
another object's unique value and fixed a wrong identity. One token has no redundancy to detect its
own corruption. Choice: identity requires >= 2 agreeing property channels; single-token percepts
are grounded without identity.

## D034 Second null for concept birth: noisy views of known concepts (2026-10-07)
Evidence: E-B seed 5, a duplicate of object 0 differing in two noisy channels was born from
sightings that were noisy views of object 0 itself. The random-percept null (D028) ignores the
stronger alternative explanation "these are known objects seen through noise". Choice: expected
sightings under the null = random-percept chance (D028) + for every born concept, its sightings in
the proto's lifetime times the probability that noise and missing channels make one of its sightings
fit the proto better than the concept itself (computed from the measured per-channel error rate,
value ranges and missing rate). Birth requires consistent support > expected + 3 sd + 1.

## D030b Event-level completion only through unambiguous slot alignment (2026-10-07)
Evidence: most remaining completion errors at 80% missing came from query slots with one or two
tokens aligned to the wrong object inside a correctly recalled event. Choice: a slot is completed
from the recalled event only if it agrees with its aligned stored object on >= 1 token and strictly
more than with any other object of that event; otherwise that slot is not completed.

## D035 Young unused concepts retire (2026-10-07)
Evidence: E-B seed 5, a concept born from four coincidental sightings (two noisy views of object 0,
one noisy view of object 2, one junk) received one sighting in the next 2,000 events. Choice: a born
concept with fewer than 20 sightings that has been unused for 600 ticks leaves the active set
(status Decayed, lineage event Retired). Its description cost was never amortised by use. Objects
that are seen regularly are never idle that long; an object that genuinely returns after a long
absence is learned again (a revival mechanism is future work, Phase E).

## D036 Completion is gated by a calibrated posterior from a measured noise model (2026-10-07)
Evidence: E-B v3, completion accuracy 97.2-98.0% at 60-80% missing on 9/15 seeds. The fill decision
used evidence counts (>= 2 channels, lead >= 2 tokens), which do not bound the error. Choice: the
grounder exposes a per-channel observation model measured from data (per-observation error from
the pre/post change rate, value range). Object-level completion requires the identity posterior
(all born concepts plus an unknown-source hypothesis, uniform prior) >= 98%. Event-level completion
requires P(event | query) x P(slot alignment | event) >= 98%, both from the same likelihoods.
Integer log-sum-exp (exp2 in Q16). This is the first use of explicit, calibrated uncertainty for an
action; Phase E reuses it for self-model calibration.

## D028b Leave-self-out background for the chance-recurrence test (2026-10-07)
Evidence: bm-memory unit test after Phase B v4: in a noise-free world two of six objects were never
born after ~200 sightings each. Their distinguishing values occur only because of the objects
themselves, so including the proto's own sightings in the marginal frequencies made a real object
look like a coincidence (circular null). Choice: the null frequencies exclude the proto's own
sightings. Phase B is re-verified after this change (E-B v5).

## D037 The unconditional law is a hypothesis too (2026-10-07)
Evidence: Phase C development run: "TOGGLE always changes its argument" and "USE always flips the
door (phase 1)" were never licensed, because the action-only base law was excluded from licensing,
and every more specific law earned no utility against it. Choice: the base law (condition = action
only) is evaluated like any law; its utility competitor is the ignorant uniform model over the
target's open alphabet. Specific laws keep competing against it (D017), so they are licensed only
where they add information.

## D038 Latent induction is scoped to argument effects (2026-10-07)
Evidence: Phase C development run created an inducer for every unexplained target (17 minutes for
one seed, latent channels multiplied features). Latent pair causes explain effects on the action's
arguments. Choice: automatic induction only for targets of argument roles, one inducer per
(action, role, channel).

## D015a Held-out compositional case includes new combinations of known fillers (2026-10-07)
Evidence: Phase C development run. In a small world with binary states every filler value is seen
within a few episodes, after which D015 produced no transfer trials and pure laws (switch flips the
lamp, 0 counterexamples in 180 cases) could never be licensed. Compositional generalisation means
new combinations of known parts. Choice: an episode is a transfer trial for a law if it contains a
filler the law never saw OR its full role-filler combination (signature) never supported the law.
Predictions are still fixed before the outcome is read.

## D038a A latent hypothesis is based on the most structured target of its slot (2026-10-07)
Evidence: Phase C development run. For a probed object the post-value and the change flag of the
same channel are two targets; D038 attached the inducer to whichever arrived first (the post
value, ~50/50 because it mixes the prior state), and link closure merged every object into one
class. Choice: when a slot (action, role, channel) triggers induction, the target with the lowest
outcome entropy under the unconditional law is used.

## D039 Latent partitions are versioned hypotheses (2026-10-07)
Evidence: Phase C development run. `diff(latent) => no change` was revoked by counterexamples that
had been recorded under an earlier, wrong partition. A law about partition P_v is a different
hypothesis from a law about P_(v+1). Choice: whenever an inducer's partition changes it is written to
a new channel id (version), laws about the old version stop matching and keep their lineage, and
laws about the new version gather fresh evidence. Induction runs every 10 observations until the
partition is unchanged three times, then every 50.

## D039a Latent induction keeps listening and pools both directions for closure (2026-10-07)
Evidence: Phase C development run on real hard links. Re-induction was keyed to the number of
distinct pairs, so once every pair had been seen the partition froze in an early, noisy state; and
content-hash collisions (1/16) make some same-file probes look unchanged. Choice: the schedule is
driven by total observations; link closure (an undirected grouping) pools the outcome histograms of
(a, b) and (b, a), ties going to the rare value.

## D040 Measurement noise is estimated from repeatability (2026-10-07)
Evidence: Phase C development run on real hard links: `same(a,b; latent) => b changes` was revoked
by 4/34 counterexamples that were content-hash collisions (a real change measured as "unchanged",
p = 1/16). Perceptual noise estimates (D020) cannot see this kind of measurement error. Choice: in a
deterministic world the same intervention on the same pair must give the same outcome, so minority
outcomes among repeated pairs estimate the observation error of that target. When a latent inducer
has seen >= 30 repeated observations, the target's noise tolerance is raised to that measured rate
(never lowered). The agent measures its own sensor's reliability by repeating interventions.

## D041 Conflicts between licensed laws are settled by their record on shared cases (2026-10-07)
Evidence: Phase C development run on real hard links: `same(a,b; latent) => b changes` was licensed,
and so were per-file laws such as "file 7 as b does not change", whose genuine exceptions (probes by
its hard-link partner, ~9%) fell inside the measured noise tolerance. Every same-file query became a
conflict and was abstained. Noise tolerance can mask structure; the masked structure shows up as
systematic disagreement on shared cases. Choice: when licensed laws predict different values, the
episodes where both applied are compared with what actually happened; a value group that was right
strictly more often (>= 3 shared cases) against every other group answers, otherwise the engine
abstains as before.

## D042 Utility is incremental over what credible general hypotheses already predict (2026-10-07)
Evidence: Phase C development run on real hard links. `diff(a,b; latent) => b unchanged` was right
in 607/607 cases yet had utility -13 bits: on cases where the observable, more general
`diff(a,b; content) => unchanged` also applied, both were right, but the younger law paid a larger
Laplace learning cost on every shared case, cancelling the information it added in the 6% of cases
where contents collided. Choice: on a case where a credible, strictly more general competitor and the
law both predicted the actual value as their majority, the law's utility change is 0; elsewhere it
is the code-length difference as before. A hypothesis earns bits only where it adds information.
