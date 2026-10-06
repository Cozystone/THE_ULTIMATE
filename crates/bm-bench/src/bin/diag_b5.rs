//! Diagnostic: split timing and post-reveal assignments of the look-alike pair (Phase B, B5b).
use bm_memory::*;
use bm_worlds::ground::{self as gw, GroundWorld, Setup};

fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let setup = Setup { noise_pct: 5, miss_pct: 10, hidden_until: Some((gw::MARK, 800)), exclusive: Some((0, 1)), ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "lab-split");
    let twin = w.objs[0].props;
    w.objs[1].props[..4].copy_from_slice(&twin[..4]);
    for (i, o) in w.objs.iter().enumerate() {
        println!("obj {i} props {:?}", o.props);
    }
    let mut g = Grounder::new(seed);
    let mut wrong = Vec::new();
    for _ in 0..2400 {
        let (ev, truth) = w.step(None);
        let t = w.t;
        if let Some(gr) = g.observe(ev) {
            for s in &gr.slots {
                let Some(&(_, o)) = truth.iter().find(|x| x.0 == s.slot) else { continue };
                if o <= 1 && t > 1300 {
                    if let Some(k) = s.concept {
                        wrong.push((t, o, g.resolve(k)));
                    }
                }
            }
        }
    }
    for c in &g.concepts {
        if c.lineage.iter().any(|e| matches!(e, ConceptEvent::SplitInto { .. } | ConceptEvent::SplitFrom { .. })) {
            println!("concept {} status {:?} lineage {:?} hist {:?}", c.id, c.status, c.lineage, c.hist);
        }
    }
    let mut counts: std::collections::BTreeMap<(usize, u32), u32> = Default::default();
    for &(_, o, k) in &wrong {
        *counts.entry((o, k)).or_insert(0) += 1;
    }
    println!("post-reveal (obj, concept) counts: {counts:?}");
}
