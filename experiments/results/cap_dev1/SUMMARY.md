# capability-h4 development round 1 (learner `83bb7a1`, D060; harness `83bb7a1` + H4_AGENT mode)

## CW, seeds 501-510, budget 60 actions, identical stage-1 state per policy (`cw.txt`, `cw.err`)

| policy | saved | settled correct (D041) | dissolved correct | wrong | unsettled answers | unresolved | chosen actions: diagnostic-new / redundant / non-diagnostic |
|---|---|---|---|---|---|---|---|
| agent (D060) | 189 | **189** | 0 | 0 | 0 | 0 | 30 / 0 / rest; exactly 3 per seed, all in the conflict region |
| random | 189 | 0 | 0 | 0 | 0 | 189 | n/a (10 region probes over 10 seeds, <= 2 distinct per seed) |
| WAIT-only | 189 | 0 | 0 | 0 | 0 | 189 | n/a |

The probe log (`PROBE` lines in `cw.err`) shows the agent's units rising 0 -> 1 -> 2 -> 3 on
distinct relevant bindings. Answers came only at the first re-judge after the third binding
(actions 10-50). On steps with no informative candidate (7-47 per seed) the agent took ordinary
exploratory actions; no candidate was chosen as "diagnostic" there.

## H4 agent-driven (`H4_AGENT=1`), seeds 201-215 (`h4_agent.txt`)

The agent chooses among 8 offered TOUCH pairs plus WAIT with its own action score; there is no
harness search and no oracle fallback. H keeps its v0.2 agent assembly (noise tolerance 0).

| outcome | count |
|---|---|
| saved conflicts | 135 |
| D041 settlement | 0 |
| dissolution, active (correct) | 133; the losing law was Contested after 1-2 agent-chosen region probes |
| unresolved (safe abstention) | 2 (seeds 210, 215, after 4,000 actions) |
| wrong / answers while unsettled | 0 / 0 |

H4 stays FAIL by its pre-registered definition (only D041 settlements count): with noise
tolerance 0, one clean counterexample refutes a deterministic law, so settlement is unreachable
there (PREREG-capability-h4 section 4). These answers are reported as **active dissolution**,
not settlement. The v0.2 record harness (oracle fallback) needed 3-14 region pairs for the same
dissolutions.

## Development gate (PREREG section 6)

| criterion | result |
|---|---|
| 0 wrong in unresolved conflicts | 0 (CW, H4) |
| agent >= random + 20 pp (CW, S + D-active) | 100% vs 0% |
| settlement audit (no settlement under 3 units) | 0 violations |
| repeats / bystanders create no settlement | smoke Repeat ablation: 0 settlements; unit test 4 |
| identity leakage | 0 (no answer on a new object; CW region queries answered by groups containing the general laws) |
| < 16 GB per process | CW peak ~3.7 GB (smoke), H4 2.4 GB |

The gate passes in round 1. C4b is covered by the regression battery: v0.2 forms no C4b
conflicts, so C4b-2 is not exercisable.
