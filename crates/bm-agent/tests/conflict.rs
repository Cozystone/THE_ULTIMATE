//! capability-h4 unit tests for D060 (PREREG-capability-h4 section 6). The API they exercise does
//! not exist on `bitmind-v0.2` (inexpressible there); test 7 guards v0.2 behaviour.
//!
//! Fixture: PROBE(a, b) changes b iff colour(a) = colour(b); otherwise iff a's nominal parity
//! is 1; a crown (shape 2) never changes b (the parity clause makes "crown => 0" informative beyond
//! the colour rule, as in H4 amendment 6). Training never shows a crown with a same-colour partner, so two licensed laws,
//! `same(colour) => 1` and `r0.shape=2 => 0`, conflict there. Noise tolerance 1% (the agent's
//! floor, D020) lets each law absorb a few counterexamples, so D041 settlement is reachable.

use bm_agent::*;
use bm_relation::*;
use hdc_core::fixed::Q;
use hdc_core::Rng;

const T: u32 = 0;
const COLOUR: u16 = 0;
const SHAPE: u16 = 1;
const CROWN: i64 = 2;
const PARITY: u16 = 2;

#[allow(clippy::too_many_arguments)]
fn ep(a: i64, b: i64, ca: i64, sa: i64, pa: i64, cb: i64, sb: i64, pb: i64, out: Option<i64>) -> Episode {
    Episode {
        id: 0,
        t: 0,
        context: context_of("K"),
        source: 0,
        action: 1,
        roles: vec![Entity::bound(a, &[(COLOUR, ca), (SHAPE, sa), (PARITY, pa)]), Entity::bound(b, &[(COLOUR, cb), (SHAPE, sb), (PARITY, pb)])],
        n_args: 2,
        outcomes: out.map(|o| vec![(T, o)]).unwrap_or_default(),
        kind: Kind::Intervention,
    }
}

struct Obj {
    id: i64,
    colour: i64,
    shape: i64,
    parity: i64,
}

fn objects() -> Vec<Obj> {
    let mut v = Vec::new();
    for i in 0..40i64 {
        v.push(Obj { id: 100 + i, colour: i % 4, shape: if i % 5 == 0 { CROWN } else { i % 2 }, parity: (i / 3) % 2 });
    }
    v
}

fn truth(x: &Obj, y: &Obj) -> i64 {
    if x.shape == CROWN {
        0
    } else if x.colour == y.colour {
        1
    } else {
        x.parity
    }
}

fn region(x: &Obj, y: &Obj) -> bool {
    x.shape == CROWN && x.colour == y.colour
}

/// A trained engine with the conflict, and the objects.
fn trained() -> (RelationEngine, Vec<Obj>) {
    let mut p = LicensePolicy::default();
    p.noise_tol_num = 1;
    p.noise_tol_den = 100;
    let mut e = RelationEngine::with_policy(7, p);
    let objs = objects();
    let mut rng = Rng::new(7);
    let mut n = 0;
    while n < 3000 {
        let pr = rng.sample_distinct(objs.len(), 2);
        let (x, y) = (&objs[pr[0]], &objs[pr[1]]);
        if region(x, y) {
            continue;
        }
        e.observe(ep(x.id, y.id, x.colour, x.shape, x.parity, y.colour, y.shape, y.parity, Some(truth(x, y))));
        n += 1;
    }
    (e, objs)
}

fn region_pairs(objs: &[Obj]) -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    for i in 0..objs.len() {
        for j in 0..objs.len() {
            if i != j && region(&objs[i], &objs[j]) {
                v.push((i, j));
            }
        }
    }
    v
}

fn q(objs: &[Obj], i: usize, j: usize) -> Episode {
    let (x, y) = (&objs[i], &objs[j]);
    ep(x.id, y.id, x.colour, x.shape, x.parity, y.colour, y.shape, y.parity, None)
}

fn cand(objs: &[Obj], i: usize, j: usize) -> Candidate {
    Candidate { kind: OptionKind::Act { action: 1, args: vec![i, j] }, episode: Some(q(objs, i, j)), targets: vec![T], tier: 1, cost_q16: Q / 4, reliability_q16: Q }
}

/// Region outcome observed for pair (i, j): here the crown law holds (0).
fn observe_region(e: &mut RelationEngine, objs: &[Obj], i: usize, j: usize) {
    let (x, y) = (&objs[i], &objs[j]);
    e.observe(ep(x.id, y.id, x.colour, x.shape, x.parity, y.colour, y.shape, y.parity, Some(0)));
}

#[test]
fn the_fixture_conflict_forms_and_is_abstained() {
    let (mut e, objs) = trained();
    let r = region_pairs(&objs);
    let query = q(&objs, r[0].0, r[0].1);
    assert_eq!(e.predict(&query, T), Answer::Abstain(Abstain::Conflict), "laws: {:?}", e.explain(&query, T));
}

/// 1. A probe on which two licensed laws disagree is selected over non-diagnostic ones.
#[test]
fn a_probe_where_two_laws_disagree_is_selected() {
    let (mut e, objs) = trained();
    let r = region_pairs(&objs);
    let (i, j) = r[1];
    let cp = e.conflict_probe(&q(&objs, i, j), T);
    assert!(cp.disagrees() && cp.informative_pairs() >= 1, "{cp:?}");
    // candidates: two non-region pairs, WAIT, and the region pair (not first in the list)
    let mut cands = vec![cand(&objs, 1, 2), cand(&objs, 3, 6)];
    cands.push(Candidate { kind: OptionKind::Wait, episode: None, targets: vec![], tier: 0, cost_q16: Q / 4, reliability_q16: Q });
    cands.push(cand(&objs, i, j));
    let rec = choose_probe(&mut e, &cands).expect("an informative probe");
    assert_eq!(rec.option, 3);
    let prefs = Preferences::default();
    assert_eq!(choose(&mut e, &cands, &prefs).map(|x| x.0), Some(3), "the general action score prefers the diagnostic probe");
}

/// 2. Probes predicted identically by every applicable licensed law are ignored.
#[test]
fn probes_with_identical_predictions_are_ignored() {
    let (mut e, objs) = trained();
    for (i, j) in [(1usize, 2usize), (3, 6), (0, 1)] {
        if region(&objs[i], &objs[j]) {
            continue;
        }
        let cp = e.conflict_probe(&q(&objs, i, j), T);
        assert_eq!(cp.informative_pairs(), 0, "pair ({i},{j}): {cp:?}");
    }
}

/// 3. No action is proposed when every candidate is uninformative.
#[test]
fn no_probe_is_proposed_when_all_candidates_are_uninformative() {
    let (mut e, objs) = trained();
    let cands = vec![cand(&objs, 1, 2), cand(&objs, 3, 6), cand(&objs, 2, 9)];
    assert!(choose_probe(&mut e, &cands).is_none());
}

/// 4. Repeated probes of the same binding count once and do not settle the conflict.
#[test]
fn repeated_probes_of_one_binding_count_once() {
    let (mut e, objs) = trained();
    let r = region_pairs(&objs);
    let (qi, qj) = r[0];
    let (i, j) = r.iter().copied().find(|&(a, b)| a != qi && b != qj).expect("another pair");
    for _ in 0..5 {
        observe_region(&mut e, &objs, i, j);
    }
    let cp = e.conflict_probe(&q(&objs, i, j), T);
    assert!(cp.pairs.iter().all(|p| !p.new_unit), "the repeated binding is not new: {cp:?}");
    assert!(cp.pairs.iter().all(|p| p.units_before <= 1), "{cp:?}");
    assert_eq!(e.predict(&q(&objs, qi, qj), T), Answer::Abstain(Abstain::Conflict));
}

/// 5 and 6. No answer before the threshold; three distinct relevant bindings settle it (D041).
#[test]
fn three_distinct_bindings_settle_and_fewer_do_not() {
    let (mut e, objs) = trained();
    let r = region_pairs(&objs);
    let (qi, qj) = r[0];
    let query = q(&objs, qi, qj);
    let others: Vec<(usize, usize)> = r.iter().copied().filter(|&p| p != (qi, qj)).collect();
    let mut used = Vec::new();
    for &(i, j) in &others {
        if used.len() == 3 {
            break;
        }
        // distinct relevant bindings: a new (a, b) pair each time
        observe_region(&mut e, &objs, i, j);
        used.push((i, j));
        let a = e.predict(&query, T);
        if used.len() < 3 {
            assert_eq!(a, Answer::Abstain(Abstain::Conflict), "answered after {} bindings", used.len());
        }
    }
    let ce = e.conflict_evidence(&query, T);
    assert!(ce.iter().all(|c| c.2 >= 3), "{ce:?}");
    assert_eq!(e.predict(&query, T).value(), Some(0), "settled by three distinct bindings");
}

/// 7. Safe abstention is not regressed: without region evidence, no conflicted region query is
/// answered.
#[test]
fn safe_abstention_is_not_regressed() {
    let (mut e, objs) = trained();
    for (i, j) in region_pairs(&objs) {
        let a = e.predict(&q(&objs, i, j), T);
        assert!(a.value().is_none(), "region pair ({i},{j}) answered: {a:?}");
    }
}
