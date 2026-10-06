//! Licensing lifecycle mechanics, including failure cases.

use bm_relation::*;
use hdc_core::Rng;

const T: u32 = 0;

fn ent(colour: i64, id: i64) -> Entity {
    Entity::new(&[(0, colour), (3, id)])
}

fn ep(ctx: &str, a: Entity, b: Entity, out: i64, kind: Kind) -> Episode {
    Episode { id: 0, t: 0, context: context_of(ctx), source: 0, action: 1, roles: vec![a, b], n_args: 0, outcomes: vec![(T, out)], kind }
}

/// Random equality-world episode over colours 0..6 and 40 ids.
fn eq_episode(rng: &mut Rng, ctx: &str, kind: Kind) -> Episode {
    let (ia, ib) = (rng.below(40) as i64, rng.below(40) as i64);
    let ca = rng.below(6) as i64;
    let cb = if rng.below(3) == 0 { ca } else { rng.below(6) as i64 };
    ep(ctx, ent(ca, ia), ent(cb, ib), (ca == cb) as i64, kind)
}

fn same_law(e: &RelationEngine) -> usize {
    e.laws
        .iter()
        .find(|l| l.condition.len() == 1 && matches!(l.condition[0], FeatureKind::Same { ch: 0, .. }))
        .map(|l| l.id)
        .expect("same(colour) candidate exists")
}

#[test]
fn equality_relation_is_licensed_and_transfers_to_unseen_values() {
    let mut e = RelationEngine::new(1);
    let mut rng = Rng::new(2);
    for _ in 0..500 {
        e.observe(eq_episode(&mut rng, "A", Kind::Intervention));
    }
    let id = same_law(&e);
    let ctx = context_of("A");
    assert_eq!(e.laws[id].status_in(ctx), Status::Licensed, "{}", e.summary(id, ctx));
    let ev = e.laws[id].ctx(ctx).unwrap();
    assert!(ev.transfer.ok >= 3 && ev.transfer.fail == 0);
    // colours 50 and 77 never occurred
    let q = ep("A", ent(50, 900), ent(50, 901), 0, Kind::Intervention).without_outcomes();
    assert_eq!(e.predict(&q, T).value(), Some(1));
    let q = ep("A", ent(50, 900), ent(77, 901), 0, Kind::Intervention).without_outcomes();
    assert_eq!(e.predict(&q, T).value(), Some(0));
}

#[test]
fn repeated_copies_of_one_pattern_are_never_licensed() {
    // failure case for gate 1: 300 identical episodes give one independent support
    let mut e = RelationEngine::new(1);
    for _ in 0..300 {
        e.observe(ep("A", ent(2, 1), ent(2, 2), 1, Kind::Intervention));
    }
    assert!(e.licensed_in(context_of("A")).is_empty());
    let id = same_law(&e);
    assert_eq!(e.laws[id].ctx(context_of("A")).unwrap().independent(), 1);
}

#[test]
fn passive_observation_alone_never_licenses() {
    // failure case for gate 2
    let mut e = RelationEngine::new(1);
    let mut rng = Rng::new(3);
    for _ in 0..800 {
        e.observe(eq_episode(&mut rng, "A", Kind::Observation));
    }
    assert!(e.licensed_in(context_of("A")).is_empty());
    let id = same_law(&e);
    assert_eq!(e.laws[id].status_in(context_of("A")), Status::Candidate);
    let q = ep("A", ent(4, 1), ent(4, 2), 0, Kind::Intervention).without_outcomes();
    assert_eq!(e.predict(&q, T), Answer::Abstain(Abstain::NoLicensedLaw));
}

#[test]
fn counterexamples_are_retained_and_withdraw_the_licence() {
    let mut e = RelationEngine::new(1);
    let mut rng = Rng::new(4);
    for _ in 0..500 {
        e.observe(eq_episode(&mut rng, "A", Kind::Intervention));
    }
    let ctx = context_of("A");
    let id = same_law(&e);
    assert_eq!(e.laws[id].status_in(ctx), Status::Licensed);
    // the world changes: equal colours now fail
    let mut bad_ids = Vec::new();
    for k in 0..6 {
        let r = e.observe(ep("A", ent(k % 6, 200 + k), ent(k % 6, 300 + k), 0, Kind::Intervention));
        let _ = r;
        bad_ids.push(e.store.len() as u64 - 1);
    }
    let ev = e.laws[id].ctx(ctx).unwrap();
    assert_ne!(ev.status, Status::Licensed, "{}", e.summary(id, ctx));
    let kept = ev.counterexample_ids();
    for b in &bad_ids {
        assert!(kept.contains(b), "counterexample {b} must be retained");
    }
    assert!(ev.history.iter().any(|h| h.1 == Status::Licensed), "lineage keeps the licensed period");
}

#[test]
fn licence_is_bound_to_its_context() {
    let mut e = RelationEngine::new(1);
    let mut rng = Rng::new(5);
    for _ in 0..500 {
        e.observe(eq_episode(&mut rng, "A", Kind::Intervention));
    }
    let q = ep("B", ent(3, 1), ent(3, 2), 0, Kind::Intervention).without_outcomes();
    assert_eq!(e.predict(&q, T), Answer::Abstain(Abstain::OutOfScope));
    // the ablation that ignores scope answers (and would be wrong in an inverted room)
    assert_eq!(e.predict_with(&q, T, true).value(), Some(1));
    // scope extends after consistent interventions in B
    for _ in 0..200 {
        e.observe(eq_episode(&mut rng, "B", Kind::Intervention));
    }
    assert_eq!(e.predict(&q, T).value(), Some(1));
}

#[test]
fn property_world_splits_the_unconditional_relation() {
    // outcome = colour(a) == 3: unconditional same(colour) must not survive
    let mut e = RelationEngine::new(1);
    let mut rng = Rng::new(6);
    for _ in 0..800 {
        let (ia, ib) = (rng.below(40) as i64, rng.below(40) as i64);
        let ca = rng.below(6) as i64;
        let cb = if rng.below(3) == 0 { ca } else { rng.below(6) as i64 };
        e.observe(ep("A", ent(ca, ia), ent(cb, ib), (ca == 3) as i64, Kind::Intervention));
    }
    let ctx = context_of("A");
    let id = same_law(&e);
    let st = e.laws[id].status_in(ctx);
    assert!(matches!(st, Status::Split | Status::Revoked), "{}", e.summary(id, ctx));
    assert!(!e.laws[id].lineage.children.is_empty(), "hidden-condition search produced children");
    let prop = e.laws.iter().find(|l| l.condition == vec![FeatureKind::Abs { role: 0, ch: 0, val: 3 }]).unwrap();
    assert_eq!(prop.status_in(ctx), Status::Licensed);
}

#[test]
fn relational_features_come_from_substrate_transforms() {
    let mut cb = Codebook::new(9);
    let e = ep("A", Entity::new(&[(1, 7)]), Entity::new(&[(1, 12)]), 0, Kind::Intervention);
    let f = cb.features(&e);
    assert!(f.iter().any(|x| x.kind == FeatureKind::Diff { r1: 0, r2: 1, ch: 1 }));
    assert!(f.iter().any(|x| x.kind == FeatureKind::Order { r1: 0, r2: 1, ch: 1, sign: 1 }));
    assert!(f.iter().any(|x| x.kind == FeatureKind::Delta { r1: 0, r2: 1, ch: 1, k: 5 }));
    // the same offset between other values yields the same feature vector (value independence)
    let e2 = ep("A", Entity::new(&[(1, 1000)]), Entity::new(&[(1, 1005)]), 0, Kind::Intervention);
    let f2 = cb.features(&e2);
    let d1 = f.iter().find(|x| matches!(x.kind, FeatureKind::Delta { .. })).unwrap();
    let d2 = f2.iter().find(|x| matches!(x.kind, FeatureKind::Delta { .. })).unwrap();
    assert_eq!(d1.hv, d2.hv);
    // equal fillers: XOR transform is exactly zero
    let e3 = ep("A", Entity::new(&[(1, 4)]), Entity::new(&[(1, 4)]), 0, Kind::Intervention);
    let s = cb.features(&e3).into_iter().find(|x| matches!(x.kind, FeatureKind::Same { .. })).unwrap();
    assert!(s.transform.unwrap().is_zero());
}

#[test]
fn conjunctions_of_same_tagged_features_do_not_collide() {
    // failure case found in E-C v2 development: XOR cancelled the shared `same` / `diff` tag, so
    // same(c0)&same(c1) and diff(c0)&diff(c1) had the same condition vector
    let mut e = RelationEngine::new(3);
    let mk = |a: [i64; 2], b: [i64; 2], out: i64| Episode {
        id: 0,
        t: 0,
        context: context_of("A"),
        source: 0,
        action: 1,
        roles: vec![Entity::new(&[(0, a[0]), (1, a[1])]), Entity::new(&[(0, b[0]), (1, b[1])])],
        n_args: 0,
        outcomes: vec![(T, out)],
        kind: Kind::Intervention,
    };
    let mut rng = Rng::new(4);
    for _ in 0..600 {
        let a = [rng.below(4) as i64, rng.below(4) as i64];
        let b = [rng.below(4) as i64, rng.below(4) as i64];
        // outcome 1 iff both channels differ
        e.observe(mk(a, b, (a[0] != b[0] && a[1] != b[1]) as i64));
    }
    // a query where both channels are equal must never be answered by the diff&diff law
    let q = mk([2, 3], [2, 3], 0).without_outcomes();
    assert_ne!(e.predict(&q, T).value(), Some(1));
}

/// Closed world: 2 x 2 x 8 situations, all seen long before sleep. The outcome is
/// `key AND power`; the wake learner (no online refinement) cannot express it.
fn closed_world(rng: &mut Rng, noisy: bool) -> Episode {
    let key = rng.below(2) as i64;
    let power = rng.below(2) as i64;
    let d = rng.below(8) as i64;
    let out = if noisy && key == 1 && power == 1 { rng.below(2) as i64 } else { key & power };
    let a = Entity::new(&[(0, 10 + key)]);
    let b = Entity::new(&[(0, 30), (5, power)]);
    let c = Entity::new(&[(0, 40), (5, d & 1), (6, (d >> 1) & 1), (7, d >> 2)]);
    Episode { id: 0, t: 0, context: context_of("S"), source: 0, action: 1, roles: vec![a, b, c], n_args: 1, outcomes: vec![(T, out)], kind: Kind::Intervention }
}

#[test]
fn sleep_licenses_a_hidden_conjunction_by_held_out_replay() {
    let pol = LicensePolicy { online_refine: false, ..Default::default() };
    let mut e = RelationEngine::with_policy(9, pol);
    let mut rng = Rng::new(10);
    for _ in 0..1500 {
        e.observe(closed_world(&mut rng, false));
    }
    let mut q = closed_world(&mut rng, false);
    while !(q.roles[0].get(0) == Some(11) && q.roles[1].get(5) == Some(1)) {
        q = closed_world(&mut rng, false);
    }
    let q = q.without_outcomes();
    assert_eq!(e.predict(&q, T).value(), None, "wake alone cannot express the conjunction");
    e.sleep(context_of("S"), 200);
    for _ in 0..300 {
        e.observe(closed_world(&mut rng, false));
    }
    assert_eq!(e.predict(&q, T).value(), Some(1), "sleep-generated conjunction licensed");
}

#[test]
fn sleep_never_licenses_a_conjunction_for_a_random_outcome() {
    let pol = LicensePolicy { online_refine: false, ..Default::default() };
    let mut e = RelationEngine::with_policy(11, pol);
    let mut rng = Rng::new(12);
    for _ in 0..1500 {
        e.observe(closed_world(&mut rng, true));
    }
    e.sleep(context_of("S"), 200);
    for _ in 0..300 {
        e.observe(closed_world(&mut rng, true));
    }
    let mut asked = 0;
    for _ in 0..200 {
        let q = closed_world(&mut rng, true);
        if q.roles[0].get(0) == Some(11) && q.roles[1].get(5) == Some(1) {
            asked += 1;
            assert_eq!(e.predict(&q.without_outcomes(), T).value(), None, "a coin flip is never a law");
        }
    }
    assert!(asked > 10);
}
