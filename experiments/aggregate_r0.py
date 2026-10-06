"""Aggregate experiments/results/r0.json over seeds (reporting only)."""
import json, os, statistics as st

here = os.path.dirname(os.path.abspath(__file__))
data = json.load(open(os.path.join(here, "results", "r0.json"), encoding="utf-8"))
runs = data["runs"]
rows = {}
for r in runs:
    rows.setdefault(r["condition"], []).append(r)

def m(xs):
    return f"{st.mean(xs):.3f} (min {min(xs):.3f})"

print(f"overall_pass={data['overall_pass']}  seeds={len(rows.get('Equality', []))}")
for cond in ["Equality", "Order", "NoRelation", "AbsClass"]:
    rs = rows.get(cond, [])
    if not rs:
        continue
    print(f"\n{cond}")
    for learner in ["class", "class_level", "lookup", "relational", "oracle"]:
        held = [r["heldout"][learner]["acc"] for r in rs]
        wrong = [r["heldout"][learner]["wrong"] for r in rs]
        abst = [r["heldout"][learner]["abstain"] for r in rs]
        seen = [r["seen"][learner]["acc"] for r in rs]
        print(f"  {learner:<12} seen {m(seen)}  held-out {m(held)}  wrong/seed {st.mean(wrong):.1f}  abstain/seed {st.mean(abst):.1f}")
    print(f"  licensed/seed {st.mean([r['licensed'] for r in rs]):.1f}  relational {st.mean([r['licensed_relational'] for r in rs]):.1f}  false licences total {sum(r['false_licences'] for r in rs)}")
c5 = rows.get("Confound", [])
if c5:
    print(f"\nConfound: licensed after passive {[r['licensed_passive'] for r in c5]}, after interventions {[r['licensed_after_intervention'] for r in c5]}")
    print(f"  same(colour) status after interventions: {sorted(set(r['same_status_after'] for r in c5))}")
c6 = rows.get("NegativeTransfer", [])
if c6:
    print(f"\nNegativeTransfer: lived wrong total {sum(r['lived']['wrong'] for r in c6)}, ablation wrong total {sum(r['ablation']['wrong'] for r in c6)}, cost ratio mean {st.mean(r['cost_ratio'] for r in c6):.3f} max {max(r['cost_ratio'] for r in c6):.3f}")
