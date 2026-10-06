//! Diagnostic: per-sighting identity trace of novel objects (Phase B, B3).
use bm_memory::*;
use bm_worlds::ground::{GObj, GroundWorld, Setup};

fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let setup = Setup { noise_pct: 5, miss_pct: 10, junk_pct: 5, ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "lab");
    let mut g = Grounder::new(seed);
    for _ in 0..2500 {
        let (ev, _) = w.step(None);
        g.observe(ev);
    }
    let mut novel = Vec::new();
    for i in 0..4 {
        let colour = w.rng().below(8) as i64;
        let size = w.rng().below(4) as i64;
        novel.push(w.add(GObj { props: [colour, 4, size, 0, 100 + i], lit: 0, pos: 0, ty: 4 }));
    }
    for &o in &novel {
        println!("novel obj {o}: props {:?}", w.objs[o].props);
    }
    let mut n: std::collections::HashMap<usize, u32> = Default::default();
    for _ in 0..900 {
        let (ev, truth) = w.step(None);
        let raw = ev.clone();
        let Some(gr) = g.observe(ev) else { continue };
        for s in &gr.slots {
            let Some(&(_, o)) = truth.iter().find(|x| x.0 == s.slot) else { continue };
            if !novel.contains(&o) {
                continue;
            }
            let c = n.entry(o).or_insert(0);
            *c += 1;
            if *c <= 40 && o == novel[1] {
                let toks: Vec<(u16, i64)> = raw.pre.slot(s.slot);
                println!("obj {o} sighting {c}: concept {:?} tokens {:?}", s.concept, toks);
            }
        }
    }
    println!("stats {:?}", g.stats);
}
