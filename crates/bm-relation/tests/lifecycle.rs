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
    try_same_law(e).expect("same(colour) candidate exists")
}

/// D055: a hypothesis exists only once its value has earned capacity.
fn try_same_law(e: &RelationEngine) -> Option<usize> {
    e.laws.iter().find(|l| l.condition.len() == 1 && matches!(l.condition[0], FeatureKind::Same { ch: 0, .. })).map(|l| l.id)
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
    // the base law sees one independent support; same(colour) carries no information against
    // it and (D055) need not exist, but if it does it has one support too
    let base = e.laws.iter().find(|l| l.condition.is_empty()).expect("base law");
    assert_eq!(base.ctx(context_of("A")).unwrap().independent(), 1);
    if let Some(id) = try_same_law(&e) {
        assert_eq!(e.laws[id].ctx(context_of("A")).unwrap().independent(), 1);
    }
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
    // no unconditional same(colour) law may survive: either it never earned capacity (D055) or
    // it was split / revoked with hidden-condition children
    if let Some(id) = try_same_law(&e) {
        let st = e.laws[id].status_in(ctx);
        assert!(matches!(st, Status::Split | Status::Revoked | Status::Candidate | Status::Contested), "{}", e.summary(id, ctx));
        assert_ne!(st, Status::Licensed);
        if matches!(st, Status::Split | Status::Revoked) {
            assert!(!e.laws[id].lineage.children.is_empty(), "hidden-condition search produced children");
        }
    }
    let prop = e.laws.iter().find(|l| l.condition == vec![FeatureKind::Abs { role: 0, ch: 0, val: 3 }]).unwrap();
    assert_eq!(prop.status_in(ctx), Status::Licensed);
}

#[test]
fn relational_features_come_from_substrate_transforms() {
    let mut cb = Codebook::new(9);
    let ord: std::collections::BTreeSet<u16> = [1u16].into_iter().collect();
    let e = ep("A", Entity::new(&[(1, 7)]), Entity::new(&[(1, 12)]), 0, Kind::Intervention);
    let f = cb.features(&e, &ord);
    // D049 failure case: on a nominal channel only same/different exists
    let nominal = cb.features(&e, &std::collections::BTreeSet::new());
    assert!(!nominal.iter().any(|x| matches!(x.kind, FeatureKind::Order { .. } | FeatureKind::Delta { .. })));
    assert!(f.iter().any(|x| x.kind == FeatureKind::Diff { r1: 0, r2: 1, ch: 1 }));
    assert!(f.iter().any(|x| x.kind == FeatureKind::Order { r1: 0, r2: 1, ch: 1, sign: 1 }));
    assert!(f.iter().any(|x| x.kind == FeatureKind::Delta { r1: 0, r2: 1, ch: 1, k: 5 }));
    // the same offset between other values yields the same feature vector (value independence)
    let e2 = ep("A", Entity::new(&[(1, 1000)]), Entity::new(&[(1, 1005)]), 0, Kind::Intervention);
    let f2 = cb.features(&e2, &ord);
    let d1 = f.iter().find(|x| matches!(x.kind, FeatureKind::Delta { .. })).unwrap();
    let d2 = f2.iter().find(|x| matches!(x.kind, FeatureKind::Delta { .. })).unwrap();
    assert_eq!(d1.hv, d2.hv);
    // equal fillers: XOR transform is exactly zero
    let e3 = ep("A", Entity::new(&[(1, 4)]), Entity::new(&[(1, 4)]), 0, Kind::Intervention);
    let s = cb.features(&e3, &ord).into_iter().find(|x| matches!(x.kind, FeatureKind::Same { .. })).unwrap();
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
    let st = e.sleep(context_of("S"), 200);
    for _ in 0..300 {
        e.observe(closed_world(&mut rng, false));
    }
    if std::env::var("DIAG_T").is_ok() {
        eprintln!("sleep {st:?}");
        for l in e.laws.iter().filter(|l| l.condition.len() <= 2) {
            eprintln!("  {}", e.summary(l.id, context_of("S")));
        }
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

const ID_CH: u16 = 9;

fn pair_ep(a: i64, b: i64, out: i64) -> Episode {
    Episode {
        id: 0,
        t: 0,
        context: context_of("L"),
        source: 0,
        action: 1,
        roles: vec![Entity::new(&[(ID_CH, a)]), Entity::new(&[(ID_CH, b)])],
        n_args: 2,
        outcomes: vec![(T, out)],
        kind: Kind::Intervention,
    }
}

fn latent_of(ind: &LatentInducer, a: i64, b: i64, ch: u16) -> (Option<i64>, Option<i64>) {
    let mut e = pair_ep(a, b, 0).without_outcomes();
    ind.augment(&mut e);
    (e.roles[0].get(ch), e.roles[1].get(ch))
}

/// Hidden classes {1,2,3}, {4}, {5}; outcome = same class. Pairs (4,5) and (1,3) never probed.
fn class_world(ind: &mut LatentInducer, skip: &[(i64, i64)]) {
    let class = |x: i64| if x <= 3 { 0 } else { x };
    for _ in 0..3 {
        for a in 1..=5 {
            for b in 1..=5 {
                if a == b || skip.contains(&(a, b)) || skip.contains(&(b, a)) {
                    continue;
                }
                ind.add(&pair_ep(a, b, (class(a) == class(b)) as i64));
            }
        }
    }
    ind.induce();
}

#[test]
fn a_latent_relation_is_exported_only_when_observed() {
    let mut ind = LatentInducer::new(1, T, ID_CH, 3000, 3001);
    class_world(&mut ind, &[(4, 5), (1, 3)]);
    let (bch, lch) = ind.channels();
    // 1 and 3 were never probed together, but their link class was established by observed
    // positive pairs (1,2), (2,3): the relation is exported
    let (x, y) = latent_of(&ind, 1, 3, lch);
    assert!(x.is_some() && x == y, "same link class exported");
    // D039f failure case: 4 and 5 are interchangeable (same block), but no pair inside that block
    // was ever observed, so "same block" is not established for their own pair
    let (x, y) = latent_of(&ind, 4, 5, bch);
    assert_eq!((x, y), (None, None), "unobserved within-block relation is not exported");
}

#[test]
fn an_older_partition_version_speaks_only_where_the_relation_is_unchanged() {
    let mut ind = LatentInducer::new(1, T, ID_CH, 3000, 3001);
    class_world(&mut ind, &[(4, 5), (1, 3)]);
    let (_, l_old) = ind.channels();
    // new evidence: 4 and 5 turn out to be linked; the link partition changes (new version)
    for _ in 0..3 {
        ind.add(&pair_ep(4, 5, 1));
        ind.add(&pair_ep(5, 4, 1));
    }
    ind.induce();
    let (_, l_new) = ind.channels();
    assert_ne!(l_old, l_new, "partition change gets a new channel");
    // (1,2): same class in both versions -> the old version still applies (D039e)
    let (x, y) = latent_of(&ind, 1, 2, l_old);
    assert!(x.is_some() && x == y);
    // (4,5): different in the old version, same in the new one -> old version is silent
    assert_eq!(latent_of(&ind, 4, 5, l_old), (None, None));
    let (x, y) = latent_of(&ind, 4, 5, l_new);
    assert!(x.is_some() && x == y);
}

fn ent3(colour: i64, shape: i64, id: i64) -> Entity {
    Entity::new(&[(0, colour), (1, shape), (3, id)])
}

/// D041a: a conflict between licensed laws is settled only by >= 3 independent shared situations;
/// repeated copies of one situation count once.
#[test]
fn conflicts_are_settled_only_by_independent_shared_situations() {
    let pol = LicensePolicy { noise_tol_num: 10, noise_tol_den: 100, ..Default::default() };
    let mut e = RelationEngine::with_policy(41, pol);
    let mut rng = Rng::new(42);
    let mut id = 0i64;
    let mut next = || {
        id += 1;
        id
    };
    // law A: same colour => 1 (shapes 0..5); different colours: random
    for _ in 0..500 {
        let ca = rng.below(6) as i64;
        let cb = if rng.below(2) == 0 { ca } else { rng.below(6) as i64 };
        let (sa, sb) = (rng.below(5) as i64, rng.below(5) as i64);
        // different colours: the outcome is a coin flip (nothing general explains it)
        let out = if ca == cb { 1 } else { rng.below(2) as i64 };
        e.observe(ep("C", ent3(ca, sa, next()), ent3(cb, sb, next()), out, Kind::Intervention));
    }
    // law B: a shape-9 first object => 0 (only seen with different colours so far)
    for _ in 0..120 {
        let ca = rng.below(6) as i64;
        let cb = (ca + 1 + rng.below(5) as i64) % 6;
        e.observe(ep("C", ent3(ca, 9, next()), ent3(cb, rng.below(5) as i64, next()), 0, Kind::Intervention));
    }
    let query = |e: &mut RelationEngine, c: i64| ep("C", ent3(c, 9, 900_000 + c), ent3(c, 2, 900_100 + c), 0, Kind::Intervention).without_outcomes();
    let q = query(&mut e, 4);
    assert!(matches!(e.predict(&q, T), Answer::Abstain(Abstain::Conflict)), "two licensed laws disagree: abstain, got {:?}", e.predict(&q, T));
    // one shared situation repeated five times is one case
    let rep = ep("C", ent3(1, 9, 777), ent3(1, 3, 778), 0, Kind::Intervention);
    for _ in 0..5 {
        e.observe(rep.clone());
    }
    // a second independent shared situation
    e.observe(ep("C", ent3(2, 9, next()), ent3(2, 1, next()), 0, Kind::Intervention));
    assert!(matches!(e.predict(&q, T), Answer::Abstain(Abstain::Conflict)), "2 independent shared situations are not enough");
    // the third independent shared situation settles it, in favour of the law that was right
    e.observe(ep("C", ent3(3, 9, next()), ent3(3, 4, next()), 0, Kind::Intervention));
    assert_eq!(e.predict(&q, T).value(), Some(0), "settled by 3 independent shared situations");
}
