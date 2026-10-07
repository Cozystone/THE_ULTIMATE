//! v0.3 pre-registered tests for K2′ (PREREG-v0.3 section 3). Tests 1 and 5 must fail on
//! `bitmind-v0.2`; tests 2, 3, 4 and 6 guard capabilities that must survive the budget.

use bm_relation::*;
use hdc_core::Rng;

const T: u32 = 0;
const COLOUR: u16 = 0;
const NUIS: u16 = 1;
const ID: u16 = 3;

fn ent(colour: i64, nuis: i64, id: i64) -> Entity {
    Entity::bound(id, &[(COLOUR, colour), (NUIS, nuis), (ID, id)])
}

fn ep(roles: Vec<Entity>, out: i64) -> Episode {
    Episode { id: 0, t: 0, context: context_of("N"), source: 0, action: 1, roles, n_args: 2, outcomes: vec![(T, out)], kind: Kind::Intervention }
}

fn laws_on(e: &RelationEngine, ch: u16) -> usize {
    e.laws.iter().filter(|l| !l.pruned && l.condition.iter().any(|f| matches!(f, FeatureKind::Abs { ch: c, .. } if *c == ch))).count()
}

/// 1. A 256-value nominal channel that is causally irrelevant does not produce one law family per
/// value (the relation is same colour).
#[test]
fn a_256_value_nuisance_channel_produces_no_per_value_law_family() {
    let mut rng = Rng::new(1);
    let mut e = RelationEngine::new(1);
    for k in 0..3000i64 {
        let (ca, cb) = (rng.below(6) as i64, rng.below(6) as i64);
        let roles = vec![ent(ca, rng.below(256) as i64, 2 * k), ent(cb, rng.below(256) as i64, 2 * k + 1)];
        e.observe(ep(roles, (ca == cb) as i64));
    }
    let n = laws_on(&e, NUIS);
    assert!(n <= 16, "{n} laws conditioned on single values of a 256-value nuisance channel");
    // the genuine relation is still learned
    let q = ep(vec![ent(4, 999, 900_000), ent(4, 998, 900_001)], 0).without_outcomes();
    assert_eq!(e.predict(&q, T).value(), Some(1));
}

/// 2. A high-cardinality channel with a real sparse dependency stays learnable: value 7 of a
/// 256-value channel always produces the outcome; every other value is a coin flip.
#[test]
fn a_sparse_dependency_in_a_high_cardinality_channel_stays_learnable() {
    let mut rng = Rng::new(2);
    let mut e = RelationEngine::new(2);
    for k in 0..12000i64 {
        let v = if rng.below(32) == 0 { 7 } else { rng.below(256) as i64 };
        let out = if v == 7 { 1 } else { rng.below(2) as i64 };
        let roles = vec![ent(rng.below(6) as i64, v, 2 * k), ent(rng.below(6) as i64, rng.below(256) as i64, 2 * k + 1)];
        e.observe(ep(roles, out));
    }
    let q = ep(vec![ent(1, 7, 900_000), ent(2, 50, 900_001)], 0).without_outcomes();
    assert_eq!(e.predict(&q, T).value(), Some(1), "the sparse signal was lost");
    let qn = ep(vec![ent(1, 8, 900_002), ent(2, 50, 900_003)], 0).without_outcomes();
    assert_eq!(e.predict(&qn, T).value(), None, "a coin-flip value must not be answered");
}

/// 3. Relabelling the nominal values (identities and the nuisance channel) consistently leaves
/// every prediction unchanged.
#[test]
fn relabelling_nominal_values_leaves_predictions_invariant() {
    let mut rng = Rng::new(3);
    let mut perm: Vec<i64> = (0..256).collect();
    rng.shuffle(&mut perm);
    let (mut a, mut b) = (RelationEngine::new(3), RelationEngine::new(3));
    for k in 0..3000i64 {
        let (ca, cb) = (rng.below(6) as i64, rng.below(6) as i64);
        let (na, nb) = (rng.below(256) as usize, rng.below(256) as usize);
        let v = if rng.below(32) == 0 { 7 } else { na as i64 };
        let out = if v == 7 { 1 } else { (ca == cb) as i64 };
        a.observe(ep(vec![ent(ca, v, 2 * k), ent(cb, nb as i64, 2 * k + 1)], out));
        b.observe(ep(vec![ent(ca, perm[v as usize], 7_000_000 + 3 * k), ent(cb, perm[nb], 9_000_000 + 5 * k)], out));
    }
    let mut diff = 0;
    for k in 0..300i64 {
        let (ca, cb, v) = (rng.below(6) as i64, rng.below(6) as i64, rng.below(256) as usize);
        let qa = ep(vec![ent(ca, v as i64, 500_000 + 2 * k), ent(cb, 3, 500_001 + 2 * k)], 0).without_outcomes();
        let qb = ep(vec![ent(ca, perm[v], 800_000 + 2 * k), ent(cb, perm[3], 800_001 + 2 * k)], 0).without_outcomes();
        diff += (a.predict(&qa, T).value() != b.predict(&qb, T).value()) as u32;
    }
    assert_eq!(diff, 0, "predictions depend on nominal labels");
}

/// 4. Counterevidence survives deferral: value 7 shows two counterexamples before it has earned a
/// law; once it is materialized, its record contains them.
#[test]
fn counterevidence_survives_deferral() {
    let mut rng = Rng::new(4);
    let mut e = RelationEngine::new(4);
    let mut sevens = 0;
    for k in 0..12000i64 {
        let v = if rng.below(32) == 0 { 7 } else { rng.below(256) as i64 };
        let out = if v == 7 {
            sevens += 1;
            if sevens <= 2 { 0 } else { 1 }
        } else {
            rng.below(2) as i64
        };
        e.observe(ep(vec![ent(rng.below(6) as i64, v, 2 * k), ent(rng.below(6) as i64, rng.below(256) as i64, 2 * k + 1)], out));
    }
    let ctx = context_of("N");
    let law = e
        .laws
        .iter()
        .find(|l| l.condition == vec![FeatureKind::Abs { role: 0, ch: NUIS, val: 7 }])
        .expect("value 7 earned a law");
    let ev = law.ctx(ctx).expect("evidence");
    assert!(ev.counters() >= 2, "early counterexamples lost: {}", ev.counters());
}

/// 5. Under repeated irrelevant novelty (a fresh nominal value every episode) the number of laws
/// stays bounded instead of growing with every new value.
#[test]
fn law_count_stays_bounded_under_irrelevant_novelty() {
    let mut rng = Rng::new(5);
    let mut e = RelationEngine::new(5);
    let mut at_1000 = 0;
    for k in 0..4000i64 {
        let (ca, cb) = (rng.below(6) as i64, rng.below(6) as i64);
        e.observe(ep(vec![ent(ca, 10_000 + 2 * k, 2 * k), ent(cb, 10_001 + 2 * k, 2 * k + 1)], (ca == cb) as i64));
        if k == 999 {
            at_1000 = e.laws.len();
        }
    }
    let at_4000 = e.laws.len();
    assert!(at_4000 <= at_1000 + 200, "law count grows with irrelevant novelty: {at_1000} -> {at_4000}");
}

/// 6. Equality-based object continuity still works next to a high-cardinality nuisance channel.
#[test]
fn equality_continuity_survives_the_budget() {
    let mut rng = Rng::new(6);
    let mut e = RelationEngine::new(6);
    for _ in 0..1500 {
        let a = rng.below(80) as i64;
        let b = if rng.below(2) == 0 { a } else { rng.below(80) as i64 };
        let roles = vec![
            Entity::new(&[(COLOUR, rng.below(6) as i64), (NUIS, rng.below(256) as i64), (ID, a)]),
            Entity::new(&[(COLOUR, rng.below(6) as i64), (NUIS, rng.below(256) as i64), (ID, b)]),
        ];
        e.observe(ep(roles, (a == b) as i64));
    }
    let same = ep(vec![Entity::new(&[(COLOUR, 2), (NUIS, 17), (ID, 900)]), Entity::new(&[(COLOUR, 5), (NUIS, 200), (ID, 900)])], 0).without_outcomes();
    let diff = ep(vec![Entity::new(&[(COLOUR, 2), (NUIS, 17), (ID, 900)]), Entity::new(&[(COLOUR, 5), (NUIS, 200), (ID, 901)])], 0).without_outcomes();
    assert_eq!(e.predict(&same, T).value(), Some(1));
    assert_eq!(e.predict(&diff, T).value(), Some(0));
}
