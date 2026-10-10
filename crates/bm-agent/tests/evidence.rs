//! D061 invariants (docs/observation-evidence-design.md section D). The API exercised here
//! (law-pair settlement, percept standing, settlement audit, open-question registry) does not exist
//! on `bitmind-capability-h4-v0.1`; test 1 reproduces, in miniature, the traced I4 seed-605 failure
//! of group-level D041.
//!
//! Fixture: PROBE(a, b) changes b iff colour(a) = colour(b); otherwise a tagged b never changes and
//! an untagged b changes iff a's parity is 1; a crown (shape 2) never changes b. Training never shows a crown with a same-colour partner, so
//! `r0.shape=crown => 0` and `same(colour) => 1` conflict there. Objects tagged 7 all have colour 3;
//! one tagged object (id 3) is not a crown, so a law about tag 7 on the second argument
//! ("b tagged => no change") is licensed with a few genuine exceptions: exactly the pairs (3, b),
//! where `same(colour)` holds and is right. Those exceptions are shared cases of the tag law and
//! the colour law, never of the crown law.

use bm_agent::*;
use bm_relation::*;
use hdc_core::fixed::Q;
use hdc_core::Rng;

const T: u32 = 0;
const COLOUR: u16 = 0;
const SHAPE: u16 = 1;
const PARITY: u16 = 2;
const TAG: u16 = 3;
const CROWN: i64 = 2;

#[derive(Clone)]
struct Obj {
    id: i64,
    colour: i64,
    shape: i64,
    parity: i64,
    tag: i64,
}

fn objects() -> Vec<Obj> {
    let tagged = [3usize, 10, 15, 25, 35];
    (0..40usize)
        .map(|i| {
            let t = tagged.contains(&i);
            Obj {
                id: 100 + i as i64,
                colour: if t { 3 } else { (i % 3) as i64 },
                shape: if i % 5 == 0 { CROWN } else { (i % 2) as i64 },
                parity: ((i / 3) % 2) as i64,
                tag: if t { 7 } else { 0 },
            }
        })
        .collect()
}

fn truth(x: &Obj, y: &Obj) -> i64 {
    if x.shape == CROWN {
        0
    } else if x.colour == y.colour {
        1
    } else if y.tag == 7 {
        0
    } else {
        x.parity
    }
}

fn region(x: &Obj, y: &Obj) -> bool {
    x.shape == CROWN && x.colour == y.colour
}

fn ent(o: &Obj, unverified: &[u16]) -> Entity {
    let mut e = Entity::bound(o.id, &[(COLOUR, o.colour), (SHAPE, o.shape), (PARITY, o.parity), (TAG, o.tag)]);
    e.unverified = unverified.to_vec();
    e
}

fn ep(x: &Obj, y: &Obj, out: Option<i64>, unverified_a: &[u16]) -> Episode {
    Episode {
        id: 0,
        t: 0,
        context: context_of("E"),
        source: 0,
        action: 1,
        roles: vec![ent(x, unverified_a), ent(y, &[])],
        n_args: 2,
        outcomes: out.map(|o| vec![(T, o)]).unwrap_or_default(),
        kind: Kind::Intervention,
    }
}

fn trained(tol_pct: u32) -> (RelationEngine, Vec<Obj>) {
    let mut p = LicensePolicy::default();
    p.noise_tol_num = tol_pct;
    p.noise_tol_den = 100;
    let mut e = RelationEngine::with_policy(11, p);
    let objs = objects();
    let mut rng = Rng::new(11);
    let mut n = 0;
    while n < 4000 {
        let pr = rng.sample_distinct(objs.len(), 2);
        let (x, y) = (&objs[pr[0]], &objs[pr[1]]);
        if region(x, y) {
            continue;
        }
        e.observe(ep(x, y, Some(truth(x, y)), &[]));
        n += 1;
    }
    (e, objs)
}

fn law_with(e: &RelationEngine, f: impl Fn(&FeatureKind) -> bool) -> Option<usize> {
    e.laws.iter().find(|l| l.condition.len() == 1 && f(&l.condition[0])).map(|l| l.id)
}

/// 1 (INV4). Evidence about another claim never settles this claim (the seed-605 pattern): the
/// tag law fails, alongside the colour law, on pairs where the crown law does not apply. Those
/// shared cases must not decide the crown law versus the colour law at a crown-and-tag query.
#[test]
fn exceptions_of_another_law_do_not_settle_the_query() {
    let (mut e, objs) = trained(6);
    let ctx = context_of("E");
    let q = ep(&objs[10], &objs[15], None, &[]);
    let applying = e.explain(&q, T);
    let vals: std::collections::BTreeSet<i64> = applying.iter().map(|x| x.1).collect();
    assert_eq!(vals.len(), 2, "the fixture conflict forms: {applying:?}");
    let tag_law = law_with(&e, |f| matches!(f, FeatureKind::Abs { role: 1, ch: TAG, val: 7 })).expect("tag law");
    assert!(applying.iter().any(|x| x.0 == tag_law), "the tag law applies to the query and is licensed: {}", e.summary(tag_law, ctx));
    // the old group-level shared cases exist (tag law and colour law fail/succeed together)...
    let colour_law = law_with(&e, |f| matches!(f, FeatureKind::Same { ch: COLOUR, .. })).expect("colour law");
    let rec = e.pair_record(colour_law, tag_law, ctx, T, &q, true, None);
    assert!(rec.units >= 3 && rec.right_a >= rec.right_b + 2, "the colour law beats the tag law on their own shared cases: {rec:?}");
    // ...but the crown law is undefeated, so the query is not answered
    assert_eq!(e.predict(&q, T), Answer::Abstain(Abstain::Conflict));
    let crown_law = law_with(&e, |f| matches!(f, FeatureKind::Abs { role: 0, ch: SHAPE, val: CROWN })).expect("crown law");
    let audit = e.settlement_audit(&q, T);
    assert!(audit.iter().all(|i| !((i.law_a == colour_law && i.law_b == crown_law) || (i.law_a == crown_law && i.law_b == colour_law))), "no crown/colour shared case exists");
}

/// 2 (INV5 scope). Only opposing laws applying to the query must be defeated: once the crown law
/// has 3 verified region units against the colour law (and the tag law too), the query is
/// answered, although other conflicts elsewhere remain open.
#[test]
fn only_laws_applying_to_the_query_must_be_defeated() {
    let (mut e, objs) = trained(6);
    // region evidence: crowns of colour 3 acting on other tagged objects; outcome 0 (crown law)
    for (i, j) in [(25usize, 3usize), (35, 3), (25, 10), (35, 15)] {
        e.observe(ep(&objs[i], &objs[j], Some(0), &[]));
    }
    let q = ep(&objs[10], &objs[15], None, &[]);
    assert_eq!(e.predict(&q, T).value(), Some(0), "every opposing law applying to Q is defeated: {:?}", e.conflict_evidence(&q, T));
}

/// 3 (INV6). Unverified percepts never become evidence, and a query whose conflict depends on an
/// unverified percept is reported as needing re-observation.
#[test]
fn unverified_percepts_are_not_evidence() {
    let (mut e, objs) = trained(6);
    // the same region evidence, but the actor's shape (the crown law's condition) is unverified
    for (i, j) in [(25usize, 3usize), (35, 3), (25, 10), (35, 15)] {
        e.observe(ep(&objs[i], &objs[j], Some(0), &[SHAPE]));
    }
    let q = ep(&objs[10], &objs[15], None, &[]);
    assert_eq!(e.predict(&q, T), Answer::Abstain(Abstain::Conflict), "unverified region cases must not settle");
    let qu = ep(&objs[10], &objs[15], None, &[SHAPE]);
    assert_eq!(e.predict(&qu, T), Answer::Abstain(Abstain::Unverified));
}

/// 4 (INV7). Fragility: a win by one unit is not enough; it must survive one wrong observation.
#[test]
fn a_one_unit_margin_does_not_settle() {
    let (mut e, objs) = trained(6);
    // three distinct region bindings: two show 0 (crown law), one shows 1 (colour law)
    e.observe(ep(&objs[25], &objs[3], Some(0), &[]));
    e.observe(ep(&objs[35], &objs[3], Some(0), &[]));
    e.observe(ep(&objs[25], &objs[10], Some(1), &[]));
    let q = ep(&objs[10], &objs[15], None, &[]);
    assert!(e.predict(&q, T).value().is_none(), "2:1 must not settle");
    e.observe(ep(&objs[35], &objs[15], Some(0), &[]));
    e.observe(ep(&objs[35], &objs[10], Some(0), &[]));
    assert_eq!(e.predict(&q, T).value(), Some(0), "4:1 settles");
}

/// 5 (INV8). D060 reads the same ledger: a probe on which only a pair that is not an open question
/// disagrees is not informative; after the query has met the conflict, a region probe is.
#[test]
fn probes_are_informative_only_for_open_questions() {
    let (mut e, objs) = trained(6);
    let probe = ep(&objs[25], &objs[3], None, &[]);
    assert_eq!(e.conflict_probe(&probe, T).informative_pairs(), 0, "no open question yet");
    let q = ep(&objs[10], &objs[15], None, &[]);
    assert!(e.predict(&q, T).value().is_none());
    assert!(!e.live_pairs.is_empty(), "the conflict became an open question");
    let cp = e.conflict_probe(&probe, T);
    assert!(cp.informative_pairs() >= 1, "{cp:?}");
    // the probe with an unverified condition percept earns nothing
    let pu = ep(&objs[25], &objs[3], None, &[SHAPE]);
    assert_eq!(e.conflict_probe(&pu, T).informative_pairs(), 0);
    let cands = vec![Candidate { kind: OptionKind::Act { action: 1, args: vec![25, 3] }, episode: Some(pu), targets: vec![T], tier: 1, cost_q16: Q / 4, reliability_q16: Q }];
    assert!(choose_probe(&mut e, &cands).is_none());
}
