//! D052 outcome test on the F1 world (seed 101 failed F1 in the v0.1 replication): one-shot
//! recall of events seen once, cue with 30% of tokens missing, without and with the system's
//! sleep-time consolidation.

use bm_memory::*;
use bm_worlds::ground::{GroundWorld, Setup};

fn recall(seed: u64, sleep: bool) -> (u32, u32, u32) {
    let setup = Setup { noise_pct: 5, miss_pct: 10, ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "f1");
    let mut g = Grounder::new(seed);
    let mut mem = EventMemory::new(seed ^ 0xF1);
    let mut ids = Vec::new();
    for _ in 0..2000 {
        let (ev, _) = w.step(None);
        let id = g.store.len() as u64;
        if let Some(gr) = g.observe(ev) {
            mem.store(&gr);
            if id >= 150 {
                ids.push(id);
            }
        }
    }
    if sleep {
        assert!(mem.consolidate(&mut g) > 0);
    }
    let nm = g.noise_model();
    let mut rng = hdc_core::Rng::new(seed ^ 0xF11);
    let (mut ok, mut wrong, mut abst) = (0, 0, 0);
    for q in 0..200 {
        let id = ids[(q * 9) % ids.len()];
        let mut cue = g.store.get(id).clone();
        for sc in [&mut cue.pre, &mut cue.post] {
            sc.tokens.retain(|_| rng.below(100) >= 30);
        }
        let gq = g.ground(&cue);
        match mem.restore_calibrated(&gq, &cue, &nm).map(|(h, _)| h.id) {
            Some(x) if x == id => ok += 1,
            Some(_) => wrong += 1,
            None => abst += 1,
        }
    }
    (ok, wrong, abst)
}

#[test]
fn consolidation_restores_one_shot_recall_on_a_failing_seed() {
    let (ok0, wrong0, abst0) = recall(101, false);
    let (ok1, wrong1, abst1) = recall(101, true);
    eprintln!("seed 101 without sleep: {ok0} ok, {wrong0} wrong, {abst0} abstain; with consolidation: {ok1} ok, {wrong1} wrong, {abst1} abstain");
    assert_eq!(wrong1, 0);
    assert!(ok1 * 100 >= 200 * 95, "with consolidation {ok1}/200");
    assert!(ok1 > ok0);
}
