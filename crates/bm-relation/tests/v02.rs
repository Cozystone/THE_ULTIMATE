//! v0.2 pre-registered tests for K1 (no arithmetic on labels) and K4 (independence per relevant
//! binding). Every test here was required to fail on the frozen v0.1 learner.

use bm_relation::*;
use hdc_core::Rng;

const T: u32 = 0;
const COLOUR: u16 = 0;
const ID: u16 = 3;

fn ent(colour: i64, id: i64) -> Entity {
    Entity::new(&[(COLOUR, colour), (ID, id)])
}

fn ep(ctx: &str, roles: Vec<Entity>, n_args: u8, out: i64) -> Episode {
    Episode { id: 0, t: 0, context: context_of(ctx), source: 0, action: 1, roles, n_args, outcomes: vec![(T, out)], kind: Kind::Intervention }
}

fn label_arithmetic(e: &RelationEngine) -> Vec<String> {
    e.laws
        .iter()
        .filter(|l| l.ctx.iter().any(|c| c.status == Status::Licensed))
        .filter(|l| l.condition.iter().any(|f| matches!(f, FeatureKind::Order { ch, .. } | FeatureKind::Delta { ch, .. } if *ch == ID)))
        .map(|l| l.describe())
        .collect()
}

/// K1-1: a consistent renaming of identities leaves predictions on new objects unchanged. Engine A
/// sees identities numbered in colour order (same-colour objects get nearby numbers, the kind of
/// accident identity arithmetic exploits); engine B sees the same episodes under a random
/// renaming.
#[test]
fn identity_renaming_leaves_relational_predictions_invariant() {
    let mut rng = Rng::new(7);
    let n = 40;
    let colours: Vec<i64> = {
        let mut c: Vec<i64> = (0..n).map(|i| (i % 6) as i64).collect();
        c.sort();
        c
    };
    let ids_a: Vec<i64> = (0..n as i64).collect();
    let mut ids_b = ids_a.clone();
    rng.shuffle(&mut ids_b);
    let mut ea = RelationEngine::new(1);
    let mut eb = RelationEngine::new(1);
    for _ in 0..700 {
        // half of the pairs are neighbours inside a colour block: same colour, and in A's numbering
        // also identities that differ by one (a coincidence identity arithmetic can latch onto)
        let p = if rng.below(2) == 0 {
            let mut j = rng.below(n as u64 - 1) as usize;
            while colours[j] != colours[j + 1] {
                j = rng.below(n as u64 - 1) as usize;
            }
            vec![j, j + 1]
        } else {
            rng.sample_distinct(n, 2)
        };
        let out = (colours[p[0]] == colours[p[1]]) as i64;
        ea.observe(ep("A", vec![ent(colours[p[0]], ids_a[p[0]]), ent(colours[p[1]], ids_a[p[1]])], 2, out));
        eb.observe(ep("A", vec![ent(colours[p[0]], ids_b[p[0]]), ent(colours[p[1]], ids_b[p[1]])], 2, out));
    }
    let mut diff = 0;
    for k in 0..200i64 {
        let (ca, cb) = (rng.below(6) as i64, rng.below(6) as i64);
        // new objects: identities never seen, renamed consistently between the two engines
        let qa = ep("A", vec![ent(ca, 1000 + 2 * k), ent(cb, 1001 + 2 * k)], 2, 0).without_outcomes();
        let qb = ep("A", vec![ent(ca, 5000 + 7 * k), ent(cb, 4000 + 3 * k)], 2, 0).without_outcomes();
        if ea.predict(&qa, T).value() != eb.predict(&qb, T).value() {
            diff += 1;
        }
    }
    assert_eq!(diff, 0, "predictions depend on identity numbering; label-arithmetic laws: {:?}", label_arithmetic(&ea));
}

/// K1-2: a world whose outcome is an identity offset (b = a + 3) offers no reusable relation over
/// any meaningful channel: nothing over identity arithmetic may be licensed, and new pairs are not
/// answered by one.
#[test]
fn identity_offset_rules_cannot_be_licensed() {
    let mut rng = Rng::new(9);
    let mut e = RelationEngine::new(2);
    for _ in 0..800 {
        let a = rng.below(60) as i64;
        let b = if rng.below(2) == 0 { a + 3 } else { rng.below(63) as i64 };
        e.observe(ep("O", vec![ent(rng.below(6) as i64, a), ent(rng.below(6) as i64, b)], 2, (b - a == 3) as i64));
    }
    assert!(label_arithmetic(&e).is_empty(), "licensed label arithmetic: {:?}", label_arithmetic(&e));
    // new identities with the same offset
    let q = ep("O", vec![ent(1, 500), ent(4, 503)], 2, 0).without_outcomes();
    assert_eq!(e.predict(&q, T).value(), None, "an identity-offset rule answered");
}

/// K1-3: identity continuity stays expressible: "both roles are the same object" is equality on
/// the identity channel, not arithmetic.
#[test]
fn equality_based_identity_continuity_is_still_licensed() {
    let mut rng = Rng::new(11);
    let mut e = RelationEngine::new(3);
    for _ in 0..600 {
        let a = rng.below(80) as i64;
        let b = if rng.below(2) == 0 { a } else { rng.below(80) as i64 };
        e.observe(ep("S", vec![ent(rng.below(6) as i64, a), ent(rng.below(6) as i64, b)], 2, (a == b) as i64));
    }
    let q_same = ep("S", vec![ent(2, 900), ent(5, 900)], 2, 0).without_outcomes();
    let q_diff = ep("S", vec![ent(2, 900), ent(5, 901)], 2, 0).without_outcomes();
    assert_eq!(e.predict(&q_same, T).value(), Some(1));
    assert_eq!(e.predict(&q_diff, T).value(), Some(0));
}

/// K4-1: one fixed relevant pair probed many times while the bystanders change is one binding.
/// It cannot license a general law, so a new pair is not answered from it.
#[test]
fn bystander_variation_does_not_create_independent_support() {
    let mut rng = Rng::new(13);
    let mut e = RelationEngine::new(4);
    let (a, b) = (ent(1, 10), ent(2, 20));
    for _ in 0..400 {
        let by1 = ent(rng.below(6) as i64, 100 + rng.below(50) as i64);
        let by2 = ent(rng.below(6) as i64, 100 + rng.below(50) as i64);
        e.observe(ep("B", vec![a.clone(), b.clone(), by1, by2], 2, 1));
    }
    let general: Vec<String> = e
        .laws
        .iter()
        .filter(|l| l.ctx.iter().any(|c| c.status == Status::Licensed))
        .filter(|l| !l.condition.iter().any(|f| matches!(f, FeatureKind::Abs { ch, .. } if *ch == ID)))
        .map(|l| l.describe())
        .collect();
    let q = ep("B", vec![ent(3, 30), ent(4, 40), ent(0, 120), ent(5, 121)], 2, 0).without_outcomes();
    assert_eq!(e.predict(&q, T).value(), None, "a new pair was answered from one pair's repeats; licensed: {general:?}");
}

/// K4-2: a D041 conflict is settled by three distinct relevant bindings, never by one pair probed
/// repeatedly under changing bystanders.
#[test]
fn conflicts_need_three_distinct_relevant_bindings() {
    let pol = LicensePolicy { noise_tol_num: 10, noise_tol_den: 100, ..Default::default() };
    let mut e = RelationEngine::with_policy(41, pol);
    let mut rng = Rng::new(42);
    let mut id = 0i64;
    let mut next = || {
        id += 1;
        id
    };
    let shape_ent = |c: i64, s: i64, i: i64| Entity::new(&[(COLOUR, c), (1, s), (ID, i)]);
    let by = |rng: &mut Rng| Entity::new(&[(COLOUR, rng.below(6) as i64), (1, rng.below(5) as i64), (ID, 50_000 + rng.below(1000) as i64)]);
    for _ in 0..500 {
        let ca = rng.below(6) as i64;
        let cb = if rng.below(2) == 0 { ca } else { rng.below(6) as i64 };
        let out = if ca == cb { 1 } else { rng.below(2) as i64 };
        let roles = vec![shape_ent(ca, rng.below(5) as i64, next()), shape_ent(cb, rng.below(5) as i64, next()), by(&mut rng)];
        e.observe(ep("C", roles, 2, out));
    }
    for _ in 0..120 {
        let ca = rng.below(6) as i64;
        let cb = (ca + 1 + rng.below(5) as i64) % 6;
        let roles = vec![shape_ent(ca, 9, next()), shape_ent(cb, rng.below(5) as i64, next()), by(&mut rng)];
        e.observe(ep("C", roles, 2, 0));
    }
    let q = ep("C", vec![shape_ent(4, 9, 900_000), shape_ent(4, 2, 900_001), shape_ent(0, 0, 900_002)], 2, 0).without_outcomes();
    assert!(matches!(e.predict(&q, T), Answer::Abstain(Abstain::Conflict)), "conflict expected, got {:?}", e.predict(&q, T));
    // one relevant pair, probed six times under different bystanders: one binding
    let (pa, pb) = (shape_ent(1, 9, 777), shape_ent(1, 3, 778));
    for _ in 0..6 {
        e.observe(ep("C", vec![pa.clone(), pb.clone(), by(&mut rng)], 2, 0));
    }
    e.observe(ep("C", vec![shape_ent(2, 9, next()), shape_ent(2, 1, next()), by(&mut rng)], 2, 0));
    assert!(matches!(e.predict(&q, T), Answer::Abstain(Abstain::Conflict)), "settled by two relevant bindings");
    e.observe(ep("C", vec![shape_ent(3, 9, next()), shape_ent(3, 4, next()), by(&mut rng)], 2, 0));
    assert_eq!(e.predict(&q, T).value(), Some(0), "three distinct relevant bindings settle it");
}

/// K1-4 (D049 identifier veto): an adapter that declares an identity channel ordinal does not hand
/// the learner identity arithmetic: a channel whose values are injective over >= 8 bound objects is
/// treated as nominal. A genuine magnitude shared by many objects stays ordinal.
#[test]
fn an_identity_channel_declared_ordinal_is_vetoed() {
    let mut rng = Rng::new(17);
    let mut e = RelationEngine::new(5);
    e.declare_ordinal(ID);
    e.declare_ordinal(1); // a real magnitude: weight 1..=6 shared by many objects
    for _ in 0..800 {
        let a = rng.below(60) as i64;
        let b = if rng.below(2) == 0 { a + 3 } else { rng.below(63) as i64 };
        let wa = 1 + rng.below(6) as i64;
        let wb = 1 + rng.below(6) as i64;
        let ea = Entity::bound(a, &[(COLOUR, rng.below(6) as i64), (1, wa), (ID, a)]);
        let eb = Entity::bound(b, &[(COLOUR, rng.below(6) as i64), (1, wb), (ID, b)]);
        e.observe(ep("V", vec![ea, eb], 2, (b - a == 3) as i64));
    }
    assert!(e.vetoed.contains(&ID), "identity channel not vetoed");
    assert!(!e.vetoed.contains(&1), "a shared magnitude was vetoed");
    assert!(label_arithmetic(&e).is_empty(), "licensed label arithmetic: {:?}", label_arithmetic(&e));
}
