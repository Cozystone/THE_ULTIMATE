# CW development world: smoke findings (PREREG-capability-h4 section 5)

All runs use the unchanged `bitmind-v0.2` learner. The only learner diff is `#[derive(Clone)]` on
`RelationEngine`, `Codebook` and `Grounder` (no behaviour change; it lets the harness give every
policy an identical copy of the stage-1 state). Harness: `crates/bm-bench/src/bin/phase_cw.rs`.
Policies:
* `Agent`: the v0.2 learner's own `bm_agent::choose`, before any D060 mechanism;
* `Reference`: an oracle-free harness check; it takes a candidate on which `explain` values
  disagree;
* `Random`, `Wait`;
* `Repeat`: the same region pair probed every step; uses world knowledge; smoke ablation only.

## Round 1 (`smoke_501_503.txt`): 14-22 objects, 2% sensor noise on every channel, budget 300

Findings, which led to a change of world:

1. **Noise on property channels creates misreads.** A non-region probe can look like a region
   case (a misread shape or colour). Such shared cases carry the outcome of the true
   (non-region) rule, so they give passive and possibly misleading D041 evidence:
   * stage 1 already shows 1 shared unit;
   * Random settled 11/13 with only 2-4 distinct region pairs probed;
   * on seed 503 conflicts were fragile. One stage-1 region query was answered wrongly by a
     lone crown law, and under relabelling 2 conflicts formed instead of 6 (grounding under noise
     is not isomorphic).
2. **The grounder's noise floor is the minimum over channels** (`noise_floor_pct`). The agent's
   tolerance is max(floor, 1%), as wired in Phase B (D020), so a world without noise still
   gives a 1% tolerance. That is enough for a licensed law with hundreds of cases to absorb a
   few counterexamples, so D041 settlement is reachable without sensor noise.
3. **Repeated probes do not change bindings:** 60 previews of one pair give one (binding a,
   binding b) (`GROUNDCHK`). The repeat ablation reached 2 units at most and then dissolved (the
   losing law turned Contested).
4. **The pre-mechanism agent never chose a diagnostic probe** (0 of 300 actions per seed),
   although its own laws disagreed on an offered candidate in up to 188 steps. D043 treats a
   query covered by any licensed law as known (`uncertainty().licensed`), so a conflict has no
   epistemic value. This is the capability gap.

World change: no sensor noise; 22-30 objects; 6 offered probes per step.

## Round 2 (`smoke2_501_505.txt`): noise-free, 22-30 objects, budget 300

| check | result (seeds 501-505) |
|---|---|
| 1. the conflict forms | 85 of 86 region queries abstain with `Conflict` (501: 12/13, others all) |
| 2. both laws have real support | e.g. seed 501: `r0.shape=crown => 0` and `same(r0,r1;colour) => 1`, both licensed with dozens of independent bindings |
| 3. reachable observations weaken a law | Repeat: the repeated pair's counterexamples make the losing law Contested (dissolution) |
| 4. >= 3 distinct diagnostic bindings | 12-25 evidence pairs per seed; Reference reaches 3-5 units |
| 5. repeats/bystanders add no units | Repeat: min units 1 after 20 probes of one pair; 0 settlements (all 85 dissolve) |
| 6. reference beats random and WAIT | Reference 85/85 settled; WAIT 0/85; **Random 73/85** (budget too large) |
| 7. no label/index shortcut | relabelled stage 1 identical on 5/5 seeds; licensed mark/identity laws 0, 2, 0, 0, 0 |

0 wrong answers and 0 unsettled answers in every policy. The v0.2 agent: 0 diagnostic choices,
0/85 resolved.

Finding 5, the budget: at 300 actions random sampling reaches 3 distinct region bindings
passively. The reference resolves after 3-6 probes. The CW budget is set to 60 actions, so the
world tests choice rather than exposure. The I-series budget (300) is fixed in the
pre-registration, so I-series worlds must have larger pair spaces.

## Round 3 (`smoke3_501_505.txt`): budget 60, pre-mechanism learner (build `target-cap2`)

| policy | saved 85 (seeds 501-505) |
|---|---|
| Reference (explain disagreement) | 85 settled correctly after 3-5 distinct region bindings (answers at action 10-50) |
| Random | 0 resolved (0-2 distinct region pairs) |
| WAIT-only | 0 resolved |
| Repeat (one pair, 20 probes) | 0 settled; 85 dissolved (the losing law turned Contested after repeated counterexamples on one binding; units stayed at 1) |
| v0.2 agent (`choose`, pre-mechanism) | 0 resolved, 0 diagnostic choices |

0 wrong answers and 0 unsettled answers in every policy. All seven smoke checks pass. Formal
development seeds may start after the mechanism is implemented. The smoke harness was the
`phase_cw.rs` version before the D060 provenance additions (same stage logic).
