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
