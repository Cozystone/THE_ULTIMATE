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

## D043 Epistemic value is the mutual information with the unknown outcome distribution (2026-10-07)
Evidence: Phase D development run: the agent waited almost always; the one-step reduction of
predictive entropy of an untested action is tiny (0.085 bits for 3 outcomes), so any cost exceeded
it. Active inference's epistemic value is the mutual information between the next outcome and the
unknown parameters. For a Dirichlet(counts+1)-categorical it is exact with harmonic numbers:
EIG = H[predictive] - (H_A - sum a_i/A H_{a_i}). Implemented in Q16 integer arithmetic
(0.383 bits for an untested 3-outcome target, -> 0 with data).

## D044 Rule of three: the evidence must bound the exception rate (2026-10-07)
Evidence: E-R0 v5 C6 seeds 9 and 20: the unconditional law "always 1" (true exception rate 17%) was
licensed after 11 consecutive confirmations (probability 0.13 by chance) and then answered wrong.
D037 made such base-rate laws licensable. Choice: a law is licensable only if (excess
counterexamples + 3) / independent supports <= 10%: with zero counterexamples at least 30
independent supports, the ~95% upper bound on its exception rate.

## D045 Trials without recorded change are evidence (2026-10-07)
Evidence: Phase D development run: an agent that only waited never saw any channel change, so no
state channel and no outcome target existed, every option looked equally unknown, the cheapest
(waiting) always won, and the agent never learned anything (a perceptual dark room). Choice: the
engine counts how often each action was taken in each context. For a target with no hypothesis
data, those trials count as observations of the status quo, so repeated waiting loses epistemic
value while an untried action keeps it.

## D043a The licence-progress bonus applies only to options that can supply the missing evidence
Evidence: Phase D development run: hypotheses about passive waiting can never be licensed (no
interventions), so they stayed "near licence" forever and gave waiting a permanent bonus of up to
1 bit per target. Choice: the bonus counts only for intervention options.

## D043b The licence-progress bonus is removed
Evidence: Phase D development run: in the "wait" scenario an expensive action won because up to four
near-licence hypotheses per target each added 0.25 bits, many of them subsumed laws that can never
be licensed. The bonus had no derivation. Choice: epistemic value is the Dirichlet mutual
information alone; hypotheses with little data already carry high mutual information.

## D043c Curiosity targets the agent's own licensing question (2026-10-07)
Evidence: Phase D development run: the epistemic agent acted 58 times in 300 decisions and licensed
nothing; the one-step Dirichlet information gain fades after a handful of trials, long before the
rule of three (D044) can be met. The agent's curiosity and its standard of knowledge disagreed.
Choice: for an intervention on a hypothesis confirmed k times without exception, the epistemic value
also includes the expected information about the binary question "is its exception rate <= 10%?"
(posterior 1 - 0.9^(k+1)); a counterexample settles the question. The larger of the two values is
used. Observation options get none of it (they cannot license, D043a).

## D043d The licensing question counts independent supports and new combinations
Evidence: Phase D development run: the epistemic agent toggled the same device 58 times; its
uncertainty was read from the most confident general hypothesis, so untested devices looked known,
and repeated combinations gave only 4 independent supports (no licence possible). Choice: the
licence-question value uses each consistent unlicensed matching hypothesis's independent supports,
and only if the query is a combination that hypothesis has never been supported by; the maximum
over such hypotheses is used.

## D044a The rule of three counts intervention trials, not distinct combinations (2026-10-07)
Evidence: E-C v1 C3 failed on 10/10 seeds: in the door world (5 binary devices, at most 32
situations) "USE flips the door" had 182 consistent interventions but only 16 distinct situations,
so D044 (30 independent supports) could never be met. Repeated trials of the same situation are
valid samples of an exception rate; breadth of generalisation is gate 1's job (>= 5 independent).
Choice: (excess counterexamples + 3) / intervention trials <= 10%.

## D039b A latent difference is exported only when it was observed (2026-10-07)
Evidence: E-C v1 C4a seed 7, 4 wrong answers on held-out same-class pairs whose classes had never
been connected by evidence: the link partition kept them apart, `diff(latent)` applied, and absence
of evidence acted as evidence of absence. Choice: for the action's argument pair, if their classes
differ and no pair between those classes was ever observed, the argument roles get no latent filler
for that partition, so no latent relation applies and the engine abstains.

## D017a Only licensed knowledge subsumes (2026-10-07)
Evidence: E-C v2 development on seed 2: a coincidental conjunction (colour difference and kind
order, equivalent to "not the switch" under that seed's fixed properties) stayed PROVISIONAL, yet as
a credible general competitor it held the specific law "toggling the indicator leaves the lamp
unchanged" at zero utility; neither was licensed and every query abstained. Choice: the subsumption
competitor of D017/D042 must be LICENSED with zero counterexamples. Redundant consistent laws may now
both be licensed; sleep compression reports them as redundant.

## D046 A channel is a state if actions change it significantly more than nothing does (2026-10-07)
Evidence: E-C v2 development, links world seed 4: the ON channel changes only on the probed object
when the hidden classes match (~6% of acted-on observations, 0% elsewhere). The fixed rule
"change rate > 2 x floor + 4%" put it on the boundary, so it flipped between property and state,
and 80% of the episodes had no outcome. Choice: in addition to the passive rule, a channel is a
state when its change rate on acted-on objects is higher than on other objects by a two-proportion
z-test with z > 3 (integer arithmetic).

## D040a One noise model per target everywhere (2026-10-07)
Evidence: E-C v2 development, real OS seed 2: `same(a,b; latent) => b changes` had 8/115
counterexamples, inside the measured repeatability noise of its target (D040), but transfer
eligibility used the global policy (noise 0), so it never received a transfer trial and could not be
licensed. Choice: transfer eligibility and own-prediction checks use the same per-target policy as
licensing.

## D042b Utility is measured against the system's actual knowledge (2026-10-07)
Evidence: E-C v2 development, real OS seed 2: `diff(a,b; latent) => unchanged` had 1,276 trials,
0 counterexamples and 1,273/1,273 transfer successes but only 7 bits of utility, because on the
cases where the licensed content cue did not apply (hash collisions) it was compared with the
unlicensed base rate. The system never answers from an unlicensed law; on those cases its actual
state was ignorance. Choice: competitors (base law, parents, general laws) count only if LICENSED;
otherwise the reference is the uniform code over the open alphabet. Same principle as D017a: only
knowledge is a competitor. Multiple-comparison cost (D012), rule of three (D044a), transfer and
independence gates are unchanged.

## D047 Conjunctions bind features with positional permutations (2026-10-07)
Evidence: R0 regression after D042b: `diff(c0) & diff(c1) => 1` answered a pair whose colours AND
weights were equal. Relational features are `tag ^ role ^ role' ^ channel`; XOR-binding two features
with the same tag cancels the tag, so same(c0)&same(c1) and diff(c0)&diff(c1) had identical
condition vectors. The defect was latent since R0 and only became visible once such conjunctions
could earn licences. Choice: conjunction vectors bind features in canonical order with the k-th
permuted by k (`act ^ rho1(f_lo) ^ rho2(f_hi)`); shared factors no longer cancel. A unit test
reproduces the collision. Every earlier gate (R0, B, C) is re-run.

## D042b withdrawn (2026-10-07)
Evidence: R0 regression with D042b: in room-B, where 83% of outcomes are 1, coincidental feature
laws ("shape order < => 1", 39 consecutive ones, p = 0.0007 each, many candidates) earned 1 bit per
case against "ignorance" and were licensed (false licences in C4, wrong answers in C6). The system
does know base rates statistically even when no deterministic law is licensed; a law that predicts
the majority outcome must beat the base rate to carry information (D012). The real-OS latent law's
limited utility is therefore a genuine information limit: beyond the licensed content cue it is
informative only on hash collisions (~6% of cases, ~0.18 bits each).

## D046a The acted-vs-other state test needs an effect size (2026-10-07)
Evidence: E-B v7 seed 22, B5b FAIL (post-reveal purity 84.1%; v6 99.8%). The revealed MARK
channel changed on 12% of acted-on observations and 8% elsewhere (noise on a wide-range channel).
Re-tested every classification, the z > 3 test of D046 eventually fired at t=1900, MARK became a
state, the look-alike pair lost its only distinguishing property, was merged at t=2000 and split
again at t=2300. Disabling D046 restores purity on this seed (diag_b5). Choice: in addition to
z > 3, the acted-on change rate must be at least twice the rate elsewhere (most acted-on changes
are attributable to the action). The motivating case (6% vs 0%) still qualifies.

## D048 Sleep tests its own hypotheses on held-out replay (2026-10-07)
Evidence: E-E development, door world with USE on any device (the effect needs "argument is the
key AND power is on"). After sleep the correct conjunction existed with 134 cases, 0 counterexamples,
16 independent situations, but 0 transfer trials: the world has only 8 situations for that
condition and replay had already shown all of them to the child, and replay earns no transfer credit
(D014). In a small closed world a sleep-born hypothesis could therefore never be licensed.
Choice: in sleep, the parent's episodes are split by a hash of the situation signature. Candidate
conditions are ranked and the child is initialised on the selection half only. On the held-out
half the child pre-registers its majority prediction before each outcome is counted; situations it
has never counted are transfer trials, and prequential utility is measured against the parent's
final counts (which include the scored episode: biased towards the parent, conservative). A lookup
table fitted on the selection half could not answer held-out situations, so this is held-out
transfer in the ATANOR sense. Online refinement (wake) is unchanged. Tests: a hidden conjunction is
licensed after sleep; a coin-flip outcome in the same cells never is.

## D026b Duplicate protos are one hypothesis (2026-10-07)
Evidence: E-C v3 C4a FAIL on seeds 3 and 12 (20 and 18 abstentions, 0 wrong) and the same seeds in
E-E v1 E1. The links world had 1,314 concepts for 16 objects: whenever an object had two identical
unborn protos, every later sighting tied them, D026a declared the percept unexplained and seeded a
third identical proto, so four objects were never born, had no identity, and got no latent class.
Choice: a tie between two unborn protos that agree on every non-state channel both define (at
least two shared, no contradiction) is resolved in favour of the older proto; the duplicates decay
unused. A tie between protos that contradict each other on some channel stays unexplained (D026a,
chimera guard). Result on the development seeds: 16-20 concepts, C4a seeds 3 and 12 answer every
never-probed pair, E-B seeds 1-5 and 22 PASS. Test: 16 look-alike objects are all born without churn.
Rejected on the way: D039c (replaying the store into an inducer enabled while awake) did not change
the abstentions and introduced 2 wrong answers on seed 2; withdrawn before any formal run.

## D036a A stored observation without a concept is one noisy sample (2026-10-07)
Evidence: E-B v9 seed 15, B2 completion at 20% missing 469/479 = 97.9% (< 98%). Every wrong fill
came from the correctly recalled event: the stored slot had no concept at storage time, so its
property values were single observations under 5% sensor noise, while the 98% posterior (D036)
covered only event and slot identity. Choice: for such slots a property is filled only if
P(event) x P(slot) x P(match on that channel, from the measured noise model) >= 98%. Slots stored
with a concept keep their concept-corrected values. Result seed 15: 360/360 at 20% missing (fewer
fills, none wrong). Covered by the B2 gate (completion accuracy) on 25 seeds.

## D039e An older partition version speaks where its relation is unchanged (2026-10-07)
Evidence: E-C v4 seed 19, C4a 0 correct, 0 wrong, 50 abstain. At observation 1050 a new positive
probe merged entity 12 into its true class (8 -> 7 link classes, now correct); the version bump
(D039) made every licensed latent law stop matching and too few positives remained to re-license.
Choice: each generator keeps its last two earlier partitions. For the argument pair, an earlier
version's filler is exported when its same/different relation equals the current version's (and
was observed, D039f); laws licensed on it keep answering there. Pairs whose relation changed see
only the new version and must wait for new licences. Test included (failure case: the changed
pair gets no old-version filler).

## D039f Sameness inside a latent class also needs an observed pair (2026-10-07)
Evidence: with D039e, seed 2 gave 2 wrong answers on (14,15): two singleton classes that never
produced the rarer outcome are interchangeable, so BLOCK put them in one block, and "same block"
was exported for their own pair although no pair inside that block had ever been observed.
D039b required evidence only for differences. Choice: a class relation (same or different) is
exported for the argument pair only if some pair between those classes was observed. Link classes
are built from observed positive pairs, so their sameness stays exported. Test included (fails
without D039f).

## C4b conflicts from content-hash collisions are left unresolved (2026-10-07)
Evidence: E-C v4 seed 30, C4b 26/30, 0 wrong, 4 abstain (Conflict): different files with
colliding content class; the licensed content law (6 collision counterexamples) and the latent
law disagree, and fewer than 3 shared cases exist to settle it (D041). Re-scoring past episodes
under the current latent partition would be circular (the partition was induced from those
outcomes). No change: abstention is the honest answer with this evidence.

## D041a Conflicts are settled by independent shared situations (2026-10-07)
Requested by the user for the two-stage C4b test. D041 counted shared episodes, so one situation
probed five times counted five times. Choice: shared cases are counted by distinct situation
signatures; a conflict is settled only by >= 3 independent shared situations
(`MIN_SHARED_INDEPENDENT`), in favour of the value that happened in strictly more of them. A
read-only `conflict_evidence` view reports, per pair of disagreeing groups, the independent shared
count and each side's record. Test: two licensed laws in conflict stay unanswered through five
copies of one shared situation plus a second one, and are settled by the third.

## D041b Shared cases are found without the episode-list cap (2026-10-07)
Evidence: two-stage C4b development, seed 31: the content law had 7,944 cases but each outcome bin
keeps at most 4,096 episode ids (EP_CAP), so every later shared case, including all of stage 2,
was invisible to conflict resolution. Choice: when a law's list is truncated, shared cases are found
by testing each law's condition against the stored feature set of every episode in the context.

## Known issue K1 (not fixed in v0.1): order/delta transforms on nominal labels
Evidence: two-stage C4b development, seeds 30 and 31. The saved conflicts are the perfect
content law `diff(content) => unchanged` (7,944 cases, 0 counterexamples) against laws such as
`r0.ext=2 & delta(r0,r1; identity)=-6 => changed`. The identity channel and latent class channels
carry nominal labels; differences of labels are arithmetic on names. Such a law memorises one
linked pair while looking relational, earns transfer credit from changing bystanders, and
conflicts with the general law on any new pair that happens to have the same label difference.
Shared evidence with the content law almost never occurs, so these conflicts stay open and the
engine abstains (no wrong answer). This corrects the earlier explanation that the C4b abstentions
were hash collisions: some are, most observed in development are K1. The freeze instruction holds:
v0.1 keeps K1; the fix (no order/delta transforms on identity and latent-label channels, or a
learned ordinal/nominal channel test) belongs to v0.2 with its own fresh held-out test.

## Known issue K2 (not fixed in v0.1): hypothesis growth on long real-OS runs
Two-stage C4b runs (9,000 real-OS probes) reach ~10-11 GB working set per process, against 569 MB
for the whole Phase F battery. Hypothesis count grows with refinement and latent versions; nothing
bounds it except D018 pruning. Target for v0.2: a memory budget with MDL-ranked eviction.
