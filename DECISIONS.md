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
