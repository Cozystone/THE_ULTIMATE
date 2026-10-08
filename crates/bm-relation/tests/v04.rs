//! v0.4 pre-registered tests for K5 (PREREG-v0.4 section 3 and amendment 1). The licence cost
//! is charged on every hypothesis examined, not only on those stored. Tests marked
//! "discriminating" are expected to fail on the v0.3 learner; the others guard capabilities that
//! must survive the stricter charge.

use bm_relation::*;
use hdc_core::Rng;

const T: u32 = 0;
const COLOUR: u16 = 0;
const NUIS: u16 = 1;
const MARK: u16 = 2;
const P: u16 = 4;
const X: u16 = 5;

fn ep_in(ctx: &str, roles: Vec<Entity>, out: i64) -> Episode {
    let n = roles.len() as u8;
    Episode { id: 0, t: 0, context: context_of(ctx), source: 0, action: 1, roles, n_args: n, outcomes: vec![(T, out)], kind: Kind::Intervention }
}

/// 1. Deferring 100,000 weak candidates leaves the licence cost at log2(>= 100,000), not at
/// the cost of the ~100 hypotheses that are stored.
#[test]
fn many_deferred_weak_candidates_are_charged() {
    let mut rng = Rng::new(11);
    let mut e = RelationEngine::new(11);
    for k in 0..52_000i64 {
        let (ca, cb) = (rng.below(6) as i64, rng.below(6) as i64);
        let roles = vec![
            Entity::bound(2 * k, &[(COLOUR, ca), (NUIS, 1_000_000 + 2 * k)]),
            Entity::bound(2 * k + 1, &[(COLOUR, cb), (NUIS, 1_000_001 + 2 * k)]),
        ];
        e.observe(ep_in("A", roles, rng.below(2) as i64));
    }
    let ctx = context_of("A");
    let stored = e.laws.iter().filter(|l| !l.pruned && l.target == T).count();
    let fam = e.family_size(1, T, ctx);
    assert!(fam >= 100_000, "family {fam}");
    assert!(stored < 1_000, "stored hypotheses {stored}");
    assert!(e.licensed_in(ctx).is_empty(), "a coin-flip world licensed something");
}

/// 2. A real sparse dependency amid a large nuisance space still materializes and licenses.
#[test]
fn a_sparse_real_signal_still_licenses() {
    let mut rng = Rng::new(12);
    let mut e = RelationEngine::new(12);
    for k in 0..12_000i64 {
        let v = if rng.below(32) == 0 { 7 } else { rng.below(256) as i64 };
        let out = if v == 7 { 1 } else { rng.below(2) as i64 };
        let roles = vec![
            Entity::bound(2 * k, &[(COLOUR, rng.below(6) as i64), (NUIS, v), (MARK, 5_000_000 + 2 * k)]),
            Entity::bound(2 * k + 1, &[(COLOUR, rng.below(6) as i64), (NUIS, rng.below(256) as i64), (MARK, 5_000_001 + 2 * k)]),
        ];
        e.observe(ep_in("S", roles, out));
    }
    let q = ep_in("S", vec![Entity::bound(900_000, &[(COLOUR, 1), (NUIS, 7), (MARK, 1)]), Entity::bound(900_001, &[(COLOUR, 2), (NUIS, 50), (MARK, 2)])], 0).without_outcomes();
    assert_eq!(e.predict(&q, T).value(), Some(1), "the sparse signal was lost");
    let qn = ep_in("S", vec![Entity::bound(900_002, &[(COLOUR, 1), (NUIS, 8), (MARK, 3)]), Entity::bound(900_003, &[(COLOUR, 2), (NUIS, 50), (MARK, 4)])], 0).without_outcomes();
    assert_eq!(e.predict(&qn, T).value(), None, "a coin-flip value must not be answered");
}

/// 3. (discriminating) H1-like trap. Phase A: one object (its mark is an identifier) shows a
/// constant outcome in one fixed situation while the family of examined hypotheses is still
/// small, so its mark law is materialized with modest utility. Phase B: many objects, fresh
/// nuisance values every episode (the family grows), and a general rule (colour 5 => 1) that
/// covers the object; the mark law gains independent situations and transfer trials but no
/// further utility. It must not be licensed: its evidence was a selection from a family that
/// has since grown far beyond the one it was charged for.
#[test]
fn object_specific_mark_laws_stay_unlicensed_in_a_multiplicity_trap() {
    let mut rng = Rng::new(13);
    let mut e = RelationEngine::new(13);
    let trap = 0i64;
    // phase A: the trap object with one fixed partner; other objects coin-flip
    for k in 0..400i64 {
        let roles = if k % 20 == 0 {
            vec![Entity::bound(trap, &[(COLOUR, 5), (MARK, 70_000)]), Entity::bound(1, &[(COLOUR, 2), (MARK, 70_001)])]
        } else {
            let a = 2 + rng.below(30) as i64;
            vec![Entity::bound(a, &[(COLOUR, rng.below(5) as i64), (MARK, 70_000 + a)]), Entity::bound(100 + k, &[(COLOUR, rng.below(5) as i64), (MARK, 80_000 + k)])]
        };
        let out = if k % 20 == 0 { 1 } else { rng.below(2) as i64 };
        e.observe(ep_in("H", roles, out));
    }
    // phase B: general rule colour 5 => 1, fresh nuisance values, the trap object recurs
    for k in 0..20_000i64 {
        let a = if rng.below(50) == 0 { trap } else { 2 + rng.below(300) as i64 };
        let ca = if a == trap { 5 } else { rng.below(6) as i64 };
        let roles = vec![
            Entity::bound(a, &[(COLOUR, ca), (MARK, 70_000 + a), (NUIS, 1_000_000 + 2 * k)]),
            Entity::bound(10_000_000 + k, &[(COLOUR, rng.below(6) as i64), (MARK, 90_000_000 + k), (NUIS, 1_000_001 + 2 * k)]),
        ];
        let out = if ca == 5 { 1 } else { rng.below(2) as i64 };
        e.observe(ep_in("H", roles, out));
    }
    let ctx = context_of("H");
    if std::env::var("DIAG_TRAP").is_ok() {
        for l in e.laws.iter().filter(|l| l.condition.iter().any(|f| matches!(f, FeatureKind::Abs { ch: MARK, val: 70_000, .. } | FeatureKind::Abs { ch: COLOUR, val: 5, .. }))) {
            eprintln!("TRAP {}", e.summary(l.id, ctx));
        }
    }
    let mut licensed_marks = Vec::new();
    for l in e.licensed_in(ctx) {
        if e.laws[l].condition.iter().any(|f| matches!(f, FeatureKind::Abs { ch: MARK, .. })) {
            licensed_marks.push(e.summary(l, ctx));
        }
    }
    assert!(licensed_marks.is_empty(), "object-specific laws licensed: {licensed_marks:#?}");
    // the general rule is licensed
    let q = ep_in("H", vec![Entity::bound(990_000, &[(COLOUR, 5), (MARK, 999_000)]), Entity::bound(990_001, &[(COLOUR, 1), (MARK, 999_001)])], 0).without_outcomes();
    assert_eq!(e.predict(&q, T).value(), Some(1), "the general rule was not learned");
}

/// 4. The family count survives sleep, compaction, retirement and revival: it never decreases.
/// Part A: the governing rule alternates between rounds, so laws are revoked (and compacted),
/// refined in sleep and recur (revival of a compacted record). Part B: a latent-class world whose
/// classes are reassigned, so latent versions die and their laws are retired (D054e).
#[test]
fn family_counts_survive_sleep_compaction_retirement_and_revival() {
    let mut rng = Rng::new(14);
    let mut e = RelationEngine::new(14);
    let ctx = context_of("F");
    let mut last = 0u64;
    let mut compacted = 0usize;
    for round in 0..8i64 {
        for k in 0..800i64 {
            let id = round * 10_000 + k;
            let (ca, cb) = (rng.below(4) as i64, rng.below(4) as i64);
            let out = if round % 2 == 0 { (ca == cb) as i64 } else { (ca == 0) as i64 };
            let roles = vec![
                Entity::bound(2 * id, &[(COLOUR, ca), (NUIS, rng.below(12) as i64)]),
                Entity::bound(2 * id + 1, &[(COLOUR, cb), (NUIS, rng.below(12) as i64)]),
            ];
            e.observe(ep_in("F", roles, out));
            let f = e.family_size(1, T, ctx);
            assert!(f >= last, "family decreased during wake: {last} -> {f}");
            last = f;
        }
        let before = e.family_size(1, T, ctx);
        e.sleep(ctx, 0);
        e.retire_stale(0);
        let after = e.family_size(1, T, ctx);
        assert!(after >= before, "family decreased in sleep/retirement: {before} -> {after}");
        last = after;
        compacted = compacted.max(e.laws.iter().filter(|l| l.ctx(ctx).map(|c| c.compacted).unwrap_or(false)).count());
    }
    assert!(compacted > 0, "part A did not exercise compaction");

    // part B: latent classes, reassigned twice
    const IDC: u16 = 9;
    let mut e = RelationEngine::new(15);
    e.identity_channel = Some(IDC);
    let ctx = context_of("L");
    let mut state = vec![0i64; 12];
    let mut fams: Vec<(u32, u64)> = vec![(10002, 0), (15002, 0)];
    let mut pruned = 0usize;
    for phase in 0..3u64 {
        let class: Vec<i64> = (0..12).map(|i| ((i as u64 * 7 + phase * 5) % 12 % 2) as i64).collect();
        for step in 0..1500 {
            let p = rng.sample_distinct(12, 2);
            let (a, b) = (p[0], p[1]);
            let change = (class[a] != class[b]) as i64;
            let before = state[b];
            state[b] ^= change;
            e.observe(Episode {
                id: 0,
                t: 0,
                context: ctx,
                source: 0,
                action: 3,
                roles: vec![Entity::bound(a as i64, &[(IDC, a as i64), (2, state[a])]), Entity::bound(b as i64, &[(IDC, b as i64), (2, before)])],
                n_args: 2,
                outcomes: vec![(10002, state[b]), (15002, change)],
                kind: Kind::Intervention,
            });
            if step % 250 == 249 {
                e.retire_stale(100);
            }
            for f in fams.iter_mut() {
                let now = e.family_size(3, f.0, ctx);
                assert!(now >= f.1, "latent world: family of t{} decreased {} -> {now}", f.0, f.1);
                f.1 = now;
            }
        }
        e.sleep(ctx, 100);
        pruned = pruned.max(e.laws.iter().filter(|l| l.pruned).count());
    }
    eprintln!("part A compacted {compacted}; part B pruned or retired {pruned}; families {fams:?}");
    assert!(pruned > 0, "part B did not exercise retirement");
}

/// 5. Refinement candidates are charged, and a valid low-cardinality refinement still licenses:
/// same colour produces the outcome only when a two-valued switch is on.
#[test]
fn refinement_candidates_are_charged_and_a_valid_refinement_licenses() {
    let mut rng = Rng::new(15);
    let mut e = RelationEngine::new(15);
    let ctx = context_of("R");
    for k in 0..4_000i64 {
        let (ca, cb, sw) = (rng.below(4) as i64, rng.below(4) as i64, rng.below(2) as i64);
        let roles = vec![
            Entity::bound(2 * k, &[(COLOUR, ca), (P, sw), (NUIS, rng.below(30) as i64)]),
            Entity::bound(2 * k + 1, &[(COLOUR, cb), (NUIS, rng.below(30) as i64)]),
        ];
        e.observe(ep_in("R", roles, ((ca == cb) && sw == 1) as i64));
    }
    let fam = e.family_size(1, T, ctx);
    let materialized_or_deferred = e.laws.iter().filter(|l| l.target == T).count() as u64 + e.deferred.get(&(1, T, ctx)).map(|t| t.tested + t.untracked).unwrap_or(0);
    let refined = e.laws.iter().filter(|l| matches!(l.lineage.origin, Origin::Refined { .. })).count();
    eprintln!("family {fam}; laws + deferred values {materialized_or_deferred}; refined children {refined}");
    assert!(refined > 0, "no refinement happened");
    assert!(fam > materialized_or_deferred, "refinement candidates were not charged");
    let on = ep_in("R", vec![Entity::bound(990_000, &[(COLOUR, 2), (P, 1), (NUIS, 3)]), Entity::bound(990_001, &[(COLOUR, 2), (NUIS, 4)])], 0).without_outcomes();
    let off = ep_in("R", vec![Entity::bound(990_002, &[(COLOUR, 2), (P, 0), (NUIS, 3)]), Entity::bound(990_003, &[(COLOUR, 2), (NUIS, 4)])], 0).without_outcomes();
    assert_eq!(e.predict(&on, T).value(), Some(1), "the valid refinement is not licensed");
    assert_eq!(e.predict(&off, T).value(), Some(0));
}

/// 6. Identity permutation invariance and counterevidence retention under K5.
#[test]
fn identity_permutation_invariance_and_counterevidence_retention() {
    let mut rng = Rng::new(16);
    let mut perm: Vec<i64> = (0..256).collect();
    rng.shuffle(&mut perm);
    let (mut a, mut b) = (RelationEngine::new(16), RelationEngine::new(16));
    let mut sevens = 0;
    for k in 0..12_000i64 {
        let (ca, cb) = (rng.below(6) as i64, rng.below(6) as i64);
        let (na, nb) = (rng.below(256) as usize, rng.below(256) as usize);
        let v = if rng.below(32) == 0 { 7 } else { na };
        let out = if v == 7 {
            sevens += 1;
            if sevens <= 2 { 0 } else { 1 }
        } else {
            (ca == cb) as i64
        };
        a.observe(ep_in("P", vec![Entity::bound(2 * k, &[(COLOUR, ca), (NUIS, v as i64)]), Entity::bound(2 * k + 1, &[(COLOUR, cb), (NUIS, nb as i64)])], out));
        b.observe(ep_in("P", vec![Entity::bound(7_000_000 + 3 * k, &[(COLOUR, ca), (NUIS, perm[v])]), Entity::bound(9_000_000 + 5 * k, &[(COLOUR, cb), (NUIS, perm[nb])])], out));
    }
    let mut diff = 0;
    for k in 0..300i64 {
        let (ca, cb, v) = (rng.below(6) as i64, rng.below(6) as i64, rng.below(256) as usize);
        let qa = ep_in("P", vec![Entity::bound(500_000 + 2 * k, &[(COLOUR, ca), (NUIS, v as i64)]), Entity::bound(500_001 + 2 * k, &[(COLOUR, cb), (NUIS, 3)])], 0).without_outcomes();
        let qb = ep_in("P", vec![Entity::bound(800_000 + 2 * k, &[(COLOUR, ca), (NUIS, perm[v])]), Entity::bound(800_001 + 2 * k, &[(COLOUR, cb), (NUIS, perm[3])])], 0).without_outcomes();
        diff += (a.predict(&qa, T).value() != b.predict(&qb, T).value()) as u32;
    }
    assert_eq!(diff, 0, "predictions depend on nominal labels or identities");
    let ctx = context_of("P");
    if let Some(law) = a.laws.iter().find(|l| l.condition == vec![FeatureKind::Abs { role: 0, ch: NUIS, val: 7 }]) {
        let ev = law.ctx(ctx).expect("evidence");
        assert!(ev.counters() >= 2, "early counterexamples lost: {}", ev.counters());
    }
}

/// 7. (discriminating, D057) Closed world, late materialization: in phase 1 the switch is always
/// on and the outcome always 1; in phase 2 the switch varies. The law "switch on => 1" is deferred
/// in phase 1 (it says nothing beyond the base rate) and is materialized late, after every
/// relevant situation has been seen. It must still be licensed, from prospective trials made
/// while it was deferred, and "switch off => 0" too.
#[test]
fn a_hypothesis_materialized_late_in_a_closed_world_can_still_be_licensed() {
    let mut rng = Rng::new(17);
    let mut e = RelationEngine::new(17);
    for step in 0..2_400 {
        let p = if step < 600 { 1 } else { rng.below(2) as i64 };
        let obj = rng.below(8) as i64;
        let roles = vec![Entity::bound(obj, &[(P, p), (X, rng.below(16) as i64)])];
        e.observe(ep_in("C", roles, p));
    }
    let ctx = context_of("C");
    let on = ep_in("C", vec![Entity::bound(2, &[(P, 1), (X, 1)])], 0).without_outcomes();
    let off = ep_in("C", vec![Entity::bound(2, &[(P, 0), (X, 1)])], 0).without_outcomes();
    let on_law = e.laws.iter().find(|l| l.condition == vec![FeatureKind::Abs { role: 0, ch: P, val: 1 }]).map(|l| e.summary(l.id, ctx));
    eprintln!("switch-on law: {on_law:?}");
    assert_eq!(e.predict(&on, T).value(), Some(1), "late-materialized law not licensed: {on_law:?}");
    assert_eq!(e.predict(&off, T).value(), Some(0));
}

/// 8. (D057) A deferred value with a counterexample earns no prospective trial; its counterexample
/// is kept when it is materialized.
#[test]
fn a_deferred_value_with_a_counterexample_earns_no_prospective_trial() {
    let mut rng = Rng::new(18);
    let mut e = RelationEngine::new(18);
    for step in 0..2_400 {
        let p = if step < 600 || step == 700 { 1 } else { rng.below(2) as i64 };
        // one early counterexample of "switch on => 1" (step 700), then the law holds
        let out = if step == 700 { 0 } else { p };
        let obj = rng.below(8) as i64;
        e.observe(ep_in("D", vec![Entity::bound(obj, &[(P, p), (X, rng.below(16) as i64)])], out));
    }
    let ctx = context_of("D");
    if let Some(l) = e.laws.iter().find(|l| l.condition == vec![FeatureKind::Abs { role: 0, ch: P, val: 1 }]) {
        let ev = l.ctx(ctx).expect("evidence");
        eprintln!("{}", e.summary(l.id, ctx));
        assert!(ev.counters() >= 1, "the counterexample was lost");
    }
    for t in e.deferred.values() {
        for (k, d) in &t.entries {
            if d.hist.len() > 1 || d.pooled > 0 {
                assert!(d.rsits.sat, "{k:?}: a non-deterministic value still tracks situations");
            }
        }
    }
}
