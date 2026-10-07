//! Grounding and recall unit tests, including failure cases.

use bm_memory::*;
use hdc_core::Rng;

/// Tiny world: 6 objects with 4 property channels and one state channel (ch 9), 3 visible per
/// event, slots shuffled, TOGGLE flips the state of its argument.
struct Tiny {
    objs: Vec<[i64; 5]>,
    rng: Rng,
    t: u64,
}

impl Tiny {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let objs = (0..6).map(|i| [i as i64, rng.below(3) as i64, rng.below(3) as i64, 10 + i as i64, 0]).collect();
        Tiny { objs, rng, t: 0 }
    }
    fn scene(&self, vis: &[(u16, usize)]) -> Scene {
        let mut tokens = Vec::new();
        for &(s, o) in vis {
            for ch in 0..4u16 {
                tokens.push(Token { slot: s, ch, val: self.objs[o][ch as usize] });
            }
            tokens.push(Token { slot: s, ch: 9, val: self.objs[o][4] });
        }
        Scene { tokens }
    }
    fn step(&mut self) -> (Event, Vec<(u16, usize)>) {
        self.t += 1;
        let pick = self.rng.sample_distinct(6, 3);
        let mut slots: Vec<u16> = vec![0, 1, 2];
        self.rng.shuffle(&mut slots);
        let vis: Vec<(u16, usize)> = slots.iter().zip(pick.iter()).map(|(&s, &o)| (s, o)).collect();
        let pre = self.scene(&vis);
        let arg = vis[0];
        self.objs[arg.1][4] ^= 1;
        let post = self.scene(&vis);
        (Event { id: 0, t: self.t, source: 0, context: 1, pre, act: Some(Act { id: 1, args: vec![arg.0] }), post, kind: Kind::Intervention }, vis)
    }
}

fn trained() -> (Grounder, Tiny) {
    let mut w = Tiny::new(1);
    let mut g = Grounder::new(2);
    for _ in 0..400 {
        let (ev, _) = w.step();
        g.observe(ev);
    }
    (g, w)
}

#[test]
fn state_and_property_channels_are_separated_and_objects_identified() {
    let (mut g, mut w) = trained();
    assert_eq!(g.class.get(&9), Some(&ChannelClass::State));
    for ch in 0..4 {
        assert_eq!(g.class.get(&ch), Some(&ChannelClass::Property));
    }
    assert_eq!(g.live_concepts().len(), 6);
    // identity is stable across reshuffled slots
    let mut map = std::collections::HashMap::new();
    for _ in 0..100 {
        let (ev, vis) = w.step();
        let gr = g.ground(&ev);
        for (s, o) in vis {
            let k = gr.slot(s).and_then(|x| x.concept).expect("identified");
            assert_eq!(*map.entry(o).or_insert(k), k);
        }
    }
}

#[test]
fn ambiguous_or_thin_percepts_get_no_identity() {
    let (mut g, _) = trained();
    // failure case: one token cannot establish identity (D033)
    assert!(g.identify(&[(0, 3)], &Default::default()).is_none());
    // a percept matching nothing is unknown, not forced onto a concept
    assert!(g.identify(&[(0, 99), (1, 99), (2, 99), (3, 99)], &Default::default()).is_none());
}

#[test]
fn missing_properties_are_completed_only_with_confidence() {
    let (mut g, mut w) = trained();
    let (mut ev, vis) = w.step();
    let (s0, o0) = vis[1];
    ev.pre.tokens.retain(|t| !(t.slot == s0 && t.ch == 1));
    ev.post.tokens.retain(|t| !(t.slot == s0 && t.ch == 1));
    let gr = g.ground(&ev);
    let sg = gr.slot(s0).unwrap();
    assert!(sg.completed.contains(&1));
    assert_eq!(sg.props.iter().find(|p| p.0 == 1).map(|p| p.1), Some(w.objs[o0][1]));
}

#[test]
fn one_off_noise_never_becomes_a_concept() {
    let (mut g, mut w) = trained();
    let before = g.live_concepts().len();
    let mut rng = Rng::new(9);
    for _ in 0..50 {
        let (mut ev, _) = w.step();
        // add a junk slot with random values
        for ch in 0..4u16 {
            let v = 1000 + rng.below(1000) as i64;
            ev.pre.tokens.push(Token { slot: 7, ch, val: v });
            ev.post.tokens.push(Token { slot: 7, ch, val: v });
        }
        g.observe(ev);
    }
    assert_eq!(g.live_concepts().len(), before);
}

#[test]
fn event_recall_restores_and_abstains_on_garbage() {
    let (mut g, mut w) = trained();
    let mut mem = EventMemory::new(3);
    let mut evs = Vec::new();
    for _ in 0..200 {
        let (ev, _) = w.step();
        let gr = g.observe(ev.clone()).unwrap();
        mem.store(&gr);
        evs.push((gr.event, ev));
    }
    let (id, ev) = &evs[57];
    let mut q = ev.clone();
    q.pre.tokens.retain(|t| t.ch != 2);
    let gq = g.ground(&q);
    assert_eq!(mem.recall_event(&gq, &q).map(|h| h.id), Some(*id));
    // failure case: an event of unknown objects is not recalled as anything
    let junk = Event {
        id: 0,
        t: 0,
        source: 0,
        context: 1,
        pre: Scene { tokens: (0..4u16).map(|ch| Token { slot: 0, ch, val: 5000 + ch as i64 }).collect() },
        act: Some(Act { id: 7, args: vec![0] }),
        post: Scene::default(),
        kind: Kind::Intervention,
    };
    let gj = g.ground(&junk);
    assert!(mem.recall_event(&gj, &junk).is_none());
}

#[test]
#[ignore]
fn debug_concepts() {
    let (g, w) = trained();
    for o in &w.objs { println!("obj {:?}", o); }
    for c in &g.concepts { if c.sightings > 5 { println!("{} {:?} sightings {} hist {:?}", c.id, c.status, c.sightings, c.hist); } }
    println!("{:?}", g.stats);
}

/// D026b: 16 look-alike objects (same kind, 4 colours, unique mark), 4 visible per event. Before
/// D026b, ties between duplicate protos seeded a new proto on every sighting and some objects were
/// never born (1,314 concepts for 16 objects in the Phase C links world).
fn links_concepts(seed: u64) -> (usize, usize, bool) {
    let classes = [0i64, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 4, 5, 6];
    let mut rng = Rng::new(seed);
    let objs: Vec<[i64; 3]> = (0..16).map(|i| [50, rng.below(4) as i64, 200 + i as i64]).collect();
    let mut state = vec![0i64; 16];
    let mut g = Grounder::new(seed);
    let mut last_ok = false;
    for t in 0..1500u64 {
        let pick = rng.sample_distinct(16, 4);
        let mut slots: Vec<u16> = vec![0, 1, 2, 3];
        rng.shuffle(&mut slots);
        let vis: Vec<(u16, usize)> = slots.iter().zip(pick.iter()).map(|(&s, &o)| (s, o)).collect();
        let scene = |st: &Vec<i64>| Scene {
            tokens: vis
                .iter()
                .flat_map(|&(s, o)| {
                    let mut v: Vec<Token> = (0..3u16).map(|ch| Token { slot: s, ch, val: objs[o][ch as usize] }).collect();
                    v.push(Token { slot: s, ch: 3, val: st[o] });
                    v
                })
                .collect(),
        };
        let pre = scene(&state);
        if classes[vis[0].1] == classes[vis[1].1] {
            state[vis[1].1] ^= 1;
        }
        let post = scene(&state);
        let ev = Event { id: 0, t, source: 0, context: 1, pre, act: Some(Act { id: 3, args: vec![vis[0].0, vis[1].0] }), post, kind: Kind::Intervention };
        if let Some(gr) = g.observe(ev) {
            last_ok = vis.iter().all(|&(s, _)| gr.slot(s).and_then(|x| x.concept).is_some());
        }
    }
    let born = g.concepts.iter().filter(|c| c.status == ConceptStatus::Concept).count();
    (g.concepts.len(), born, last_ok)
}

#[test]
fn duplicate_protos_do_not_churn_and_every_object_is_born() {
    for seed in 1..=8 {
        let (total, born, last_ok) = links_concepts(seed);
        assert!(total <= 48, "seed {seed}: concept churn: {total} concepts for 16 objects");
        assert!(born >= 16, "seed {seed}: only {born} objects were born");
        assert!(last_ok, "seed {seed}: last scene not fully identified");
    }
}

/// D052 mechanism: under sensor noise, events stored early carry groundings made with immature
/// concepts; consolidation re-grounds them from the raw record (changed > 0) without degrading
/// recall. Failure case: a second consolidation finds nothing stale and changes nothing.
#[test]
fn consolidation_restores_recall_of_events_stored_before_concepts_existed() {
    let mut w = Tiny::new(5);
    let mut g = Grounder::new(6);
    let mut mem = EventMemory::new(7);
    let mut early = Vec::new();
    let mut noise = Rng::new(9);
    for _ in 0..800 {
        let (mut ev, _) = w.step();
        // sensor noise as in the F1 world: 5% wrong values, 10% missing tokens
        for sc in [&mut ev.pre, &mut ev.post] {
            sc.tokens.retain(|_| noise.below(100) >= 10);
            for t in sc.tokens.iter_mut() {
                if noise.below(100) < 5 {
                    t.val = noise.below(4) as i64 + 50;
                }
            }
        }
        if let Some(gr) = g.observe(ev.clone()) {
            mem.store(&gr);
            // the first stored events: grounded with the least mature concepts
            if early.len() < 60 {
                early.push((gr.event, ev));
            }
        }
    }
    let mut rng = Rng::new(8);
    let cues: Vec<(u64, Event)> = early
        .iter()
        .map(|(id, ev)| {
            let mut q = ev.clone();
            q.pre.tokens.retain(|_| rng.below(10) >= 3);
            q.post.tokens.retain(|_| rng.below(10) >= 3);
            (*id, q)
        })
        .collect();
    let score = |mem: &mut EventMemory, g: &mut Grounder| -> usize {
        cues.iter().filter(|(id, q)| { let gq = g.ground(q); mem.recall_event(&gq, q).map(|h| h.id) == Some(*id) }).count()
    };
    let before = score(&mut mem, &mut g);
    let changed = mem.consolidate(&mut g);
    let after = score(&mut mem, &mut g);
    assert!(changed > 0, "early events were grounded before concepts existed");
    // recall in this tiny world is limited by near-duplicate events; the outcome test on the F1
    // world is crates/bm-bench/tests/consolidation.rs
    assert!(after >= before, "consolidation must not degrade recall: before {before}, after {after}");
    assert_eq!(mem.consolidate(&mut g), 0, "nothing stale after consolidation");
    assert_eq!(score(&mut mem, &mut g), after);
}
