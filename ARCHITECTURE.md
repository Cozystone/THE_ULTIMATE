# BITMIND Architecture

PROJECT: **BITMIND**, the cognitive core of THE ULTIMATE.
Goal: a local general cognitive core that works without large neural networks.

Central hypothesis under test:

> Intelligence is not a necessary consequence of giant weights, next-token prediction and massive
> pretraining. It can emerge from discrete representations, a causal world model, active inference,
> a self-model and continual compression of experience.

This is a research and engineering project to build a cognitive substrate independent of the LLM
scaling paradigm. It is not a small assistant, a RAG system, a rule engine or an LLM wrapper.

---

## 1. Constitution (immutable)

1. No LLM, SLM, Transformer, pretrained language model or API model at runtime.
2. No pretrained embeddings, CLIP, word2vec, BERT, speech or object recognition models, or external vector DB.
3. No large-scale backprop and no dense floating-point matrix multiplication on the core cognitive path.
4. Core representation and inference are HDC/VSA discrete operations.
   * default dimension 16,384 bits or more
   * ops: XOR bind/unbind, bundling, permutation, popcount, Hamming distance
   * confidence, frequency and recency are integer counters, bit-planes, thresholds and shifts, never floats
5. No hand-inserted answer rules for any benchmark.
6. Keyword matching, rule engines and if-else sets are never presented as intelligence.
7. Fluent natural language is not a goal. Language is the I/O interface of the world model.
8. Never claim "consciousness has emerged". Build functionally verifiable self-model, uncertainty
   representation, temporal continuity and active information gathering instead.
9. Difficulty never justifies turning the project into a small LLM agent or OS automation.

Floats are allowed only in evaluation/reporting code that sits outside the cognitive path
(e.g. computing a percentage for EXPERIMENTS.md). Enforced by crate boundaries: cognitive crates
contain no `f32`/`f64` (checked by `scripts/check_constitution.ps1`).

---

## 2. ATANOR_INHERITANCE

ATANOR was the predecessor: a No-LLM developmental neuro-symbolic project aiming at one persistent
Self with one continuing world state, one cognitive/action authority, experience-based concepts,
relations, causal laws, planning, memory and an eventual causal self-model, with language as I/O.

Developmental path inherited unchanged:

```
experience -> episode -> proto-concept -> concept -> relation/law
```

Every knowledge item keeps why it is believed:
`provenance, support, contradiction, scope, confidence, applicability`.

### 2.1 What ATANOR proved and where it stopped

* Storing concrete concepts and examples is comparatively easy.
* Reusable rules emerged only in constrained domains.
* Each new relational ontology needed a new hand-added primitive (e.g. `RELEQ` for equivalence,
  GATE B 2026-08-26). The substrate itself was not relational.
* **Negative transfer (DS1, 2026-08-13):** relations learned in old worlds confidently spoke wrong
  answers in new worlds. The more it had lived, the more expensive a new problem became
  (4.4x at position 8). Experience made it mistaken, not hesitant.

The unresolved problem:

> How does an agent distinguish a real reusable relation or causal law from a weak coincidence,
> a memorized lookup or a superficially plausible pattern?

More memory, graph complexity or retrieval quality did not solve it. BITMIND must not be ATANOR's
knowledge graph with faster bit operations.

### 2.2 What does NOT count as evidence of intelligence

storing more episodes; nearest-neighbour retrieval; similarity clustering; keyword or token
co-occurrence; class/category classification; replaying a previously seen answer; a hand-authored
rule that happens to pass a benchmark.

### 2.3 The four outcomes every end-to-end proof must separate

| # | outcome | statement | counts as progress |
|---|---|---|---|
| 1 | Class learning | "these are similar examples" | no |
| 2 | Memorized lookup | "this resembles something already stored" | no |
| 3 | Relational learning | "this role-structured transformation holds under stated conditions" | **yes** |
| 4 | Transfer | "the learned relation works in a previously unseen combination" | **yes** |

### 2.4 Relation / Law licensing (mandatory from the first relational code)

Lifecycle:

```
CANDIDATE -> CONTESTED -> PROVISIONAL -> LICENSED -> RESTRICTED | REVOKED | SPLIT
```

* CANDIDATE: generated from experience; may guide exploration; never used to answer.
* CONTESTED: has counterexamples or live competing hypotheses of comparable support.
* PROVISIONAL: independent support and intervention evidence, awaiting held-out transfer and
  utility proof. May be used with explicit low confidence, inside verified scope only.
* LICENSED: passed all five gates below. May be applied to new cases inside its scope.
* RESTRICTED: counterexamples narrowed the scope; licence holds only inside the narrowed scope.
* REVOKED: counterexamples or failed transfer destroyed it. Kept for lineage, never applied.
* SPLIT: replaced by two or more more-specific relations (hidden condition found). Kept for lineage.

```
RelationLaw:
- condition_hv          when it applies (bound conjunction of role-structured features)
- transformation_hv     what changes (role-structured transform code)
- predicted_effect_hv   expected result
- supporting_episodes   independent supporting experiences (episode ids + independence keys)
- counterexamples       failures and exceptions (episode ids, retained forever)
- scope                 objects, contexts and time ranges where it was verified
- confidence            integer evidence state
- applicability_license whether it may be applied to a new case
- transfer_record       pre-registered predictions on unseen combinations and their results
- lineage               parent relations, generating mechanism, split/merge history
- competing_hypotheses  alternative explanations with their own evidence
```

Licensing gates. A relation becomes LICENSED only if all hold:

1. **Independence**: supported by multiple independent experiences (distinct role-filler
   combinations), not repeated copies of one pattern.
2. **Intervention**: improves prediction under the agent's own intervention or a counterfactual
   comparison, not only passive observation.
3. **Held-out transfer**: succeeded on at least one held-out compositional case, predicted and
   recorded before the outcome was seen.
4. **Counterevidence kept**: every counterexample is retained and restricts scope, revokes the
   relation or triggers a split. Nothing is silently averaged away.
5. **Utility**: using the relation reduces later prediction error or learning cost compared with
   its best competing hypothesis (measured, integer bits).

Core principle: **a relation acquires meaning through downstream utility under later independent
experience, never through surface appearance, token similarity, co-occurrence or
self-confirmation.**

Negative-transfer guard (from DS1): a licence is bound to its scope. In an unverified scope a
licensed relation re-enters as CANDIDATE for that scope; it may steer exploration but may not
answer. Silence is preferred to confident wrongness.

### 2.5 Why HDC is the relational substrate here, not a storage trick

ATANOR needed a new primitive per relation type. In BITMIND a relation between two role fillers is
the **group element that maps one filler onto the other**, computed by the substrate's own two
operations:

* XOR group: `T = hv(a) ^ hv(b)`; `T == 0` exactly when the fillers are equal, for any value, seen
  or unseen.
* permutation group: the offset `k` with `rho^k(hv(a)) == hv(b)` for ordinally encoded channels;
  sign and magnitude of `k` are value-independent.

Relational features are therefore produced by one general mechanism over any channel, any pair of
roles, observed or latent. Whether a relational feature matters is never assumed. It must earn a
licence through the five gates.

---

## 3. Layers

```
Layer 0  sensing and immediate action      (world adapters, efference copy of own actions)
Layer 1  external world state, objects, events, causality
Layer 2  self state: memory conflicts, uncertainty, resources, competence
Layer 3  policy selection, long-term preferences, constraints, action priority
Layer 4  sleep consolidation: replay, compression, new hypothesis generation
```

## 4. Crates and module boundaries

| crate | layer | responsibility | may depend on |
|---|---|---|---|
| `hdc-core` | substrate | packed hypervectors, bind/permute/bundle, popcount kernels with CPU dispatch, item memory, level codes, cleanup memory, integer evidence, fixed-point log2 | std only |
| `bm-relation` | 1 | relation transforms, candidate generation, RelationLaw, licensing lifecycle, competing hypotheses, transfer records | hdc-core |
| `bm-worlds` | 0 | microworlds and OS sandbox adapter; ground-truth oracles for evaluation only | none of the cognitive crates |
| `bm-memory` | 1 | immutable episode store with lineage, proto-concept/concept formation by MDL | hdc-core, bm-relation |
| `bm-agent` | 2-4 | belief state, active inference policy scoring, self-model, sleep consolidation | all above |
| `bm-bench` | eval | benchmark harness, R0 falsification test, generalization battery; floats allowed here only | all |

Information flow: world -> Layer 0 tokens -> episodes (immutable, with lineage) -> relational
feature extraction -> candidate relations -> licensing via evidence, intervention, counterexample,
scope and transfer -> licensed relations feed prediction, planning and self-model.

## 5. Performance principles

1. u64 packed hypervectors 2. XOR + popcount kernels 3. AVX2 4. AVX-512 VPOPCNTDQ
5. ARM NEON 6. runtime CPU feature dispatch 7. NPU/FPGA/ASIC later.
NPU/AMX are matrix engines and are not assumed.

## 6. Language

Byte/character/word sequence encoding with position by permutation, phrase representations,
grounding of language patterns in episode/object/action structure, a compositional symbolic
grammar and a minimal template realizer that externalises internal structure. Fluency is not a goal.
