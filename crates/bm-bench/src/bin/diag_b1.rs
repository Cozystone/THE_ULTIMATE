//! Diagnostic: composition of live concepts (Phase B, B1/B4).
use bm_memory::*;
use bm_worlds::ground::{GroundWorld, Setup, JUNK};
use std::collections::{BTreeMap, HashMap};

fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let setup = Setup { noise_pct: 5, miss_pct: 10, junk_pct: 5, ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "lab");
    for (i, o) in w.objs.iter().enumerate() {
        println!("obj {i:2} props {:?}", o.props);
    }
    let mut g = Grounder::new(seed);
    let mut comp: HashMap<u32, BTreeMap<String, u32>> = HashMap::new();
    let mut truths: Vec<Vec<(u16, usize)>> = Vec::new();
    for _ in 0..2500 {
        let (ev, truth) = w.step(None);
        truths.push(truth.clone());
        if let Some(gr) = g.observe(ev) {
            for s in &gr.slots {
                let Some(k) = s.concept else { continue };
                let o = truth.iter().find(|x| x.0 == s.slot).map(|x| x.1).unwrap_or(JUNK);
                let name = if o == JUNK { "junk".to_string() } else { format!("o{o}") };
                *comp.entry(g.resolve(k)).or_default().entry(name).or_insert(0) += 1;
            }
        }
    }
    for k in g.live_concepts() {
        let c = &g.concepts[k as usize];
        let maj: Vec<(u16, i64)> = c.hist.keys().filter_map(|&ch| c.majority(ch).map(|m| (ch, m.0))).collect();
        println!("concept {k:3} sightings {:4} born {:?} majority {:?} truth {:?}", c.sightings, c.lineage.first(), maj, comp.get(&k));
        if c.sightings < 100 {
            println!("   hist {:?}", c.hist);
            for &(e, sl) in &c.exemplars {
                let o = truths.get(e as usize).and_then(|t| t.iter().find(|x| x.0 == sl)).map(|x| x.1);
                let raw: Vec<(u16, i64)> = g.store.get(e).pre.slot(sl);
                println!("   exemplar event {e} slot {sl} truth {:?} raw {:?}", o.map(|o| if o == JUNK { "junk".to_string() } else { format!("o{o}") }), raw);
            }
        }
    }
    println!("{:?}", g.stats);
}
