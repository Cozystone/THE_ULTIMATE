//! bm-agent unit tests, including failure cases.

use bm_agent::*;
use bm_relation::{context_of, Entity, Episode, Kind, RelationEngine};
use hdc_core::fixed::Q;
use hdc_core::Rng;

fn ep(a: i64, b: i64, out: i64) -> Episode {
    Episode {
        id: 0,
        t: 0,
        context: context_of("A"),
        source: 0,
        action: 1,
        roles: vec![Entity::new(&[(0, a)]), Entity::new(&[(0, b)])],
        n_args: 0,
        outcomes: vec![(0, out)],
        kind: Kind::Intervention,
    }
}

#[test]
fn forbidden_option_is_never_chosen_even_if_most_informative() {
    let mut rel = RelationEngine::new(1);
    let q = ep(1, 2, 0).without_outcomes();
    let c = vec![
        Candidate { kind: OptionKind::Act { action: 1, args: vec![0, 1] }, episode: Some(q.clone()), targets: vec![0], tier: 3, cost_q16: 0, reliability_q16: Q },
        Candidate { kind: OptionKind::Wait, episode: None, targets: vec![], tier: 0, cost_q16: Q, reliability_q16: Q },
    ];
    let (i, _) = choose(&mut rel, &c, &Preferences::default()).unwrap();
    assert_eq!(c[i].kind, OptionKind::Wait);
    // with the boundary lifted the informative act wins (the score itself was not the reason)
    let open = Preferences { max_tier: 3, ..Default::default() };
    let (i, _) = choose(&mut rel, &c, &open).unwrap();
    assert!(matches!(c[i].kind, OptionKind::Act { .. }));
}

#[test]
fn known_outcomes_carry_no_information_gain() {
    let mut rel = RelationEngine::new(2);
    let mut rng = Rng::new(3);
    for _ in 0..500 {
        let a = rng.below(6) as i64;
        let b = if rng.below(3) == 0 { a } else { rng.below(6) as i64 };
        rel.observe(ep(a, b, (a == b) as i64));
    }
    let c = Candidate { kind: OptionKind::Act { action: 1, args: vec![] }, episode: Some(ep(40, 40, 0).without_outcomes()), targets: vec![0], tier: 0, cost_q16: 0, reliability_q16: Q };
    let s = score(&mut rel, &c, &Preferences::default());
    assert_eq!(s.info_gain, 0, "licensed relation answers unseen values: nothing to learn");
    // a fresh engine is maximally uncertain
    let mut fresh = RelationEngine::new(2);
    fresh.observe(ep(1, 1, 1));
    let s2 = score(&mut fresh, &c, &Preferences::default());
    assert!(s2.info_gain > 0);
}

#[test]
fn calibration_maps_overconfidence_to_empirical_accuracy() {
    let mut cal = Calibration::default();
    // stated 95% but right only 70% of the time
    for i in 0..1000 {
        cal.record(Q * 95 / 100, i % 10 < 7);
    }
    let c = cal.calibrated(Q * 95 / 100);
    assert!((c - Q * 70 / 100).abs() < Q / 50, "calibrated {}", c * 100 / Q);
    let raw: Vec<(i64, bool)> = (0..1000).map(|i| (Q * 95 / 100, i % 10 < 7)).collect();
    let fixed: Vec<(i64, bool)> = raw.iter().map(|&(c0, ok)| (cal.calibrated(c0), ok)).collect();
    assert!(Calibration::ece_q16(&raw) > Q / 5);
    assert!(Calibration::ece_q16(&fixed) < Q / 50);
}

#[test]
fn words_are_grounded_only_with_consistent_independent_evidence() {
    let mut lx = Lexicon::new(4);
    let mut rng = Rng::new(5);
    let colours = ["red", "green", "blue"];
    let shapes = ["ball", "cube"];
    for s in 0..60u64 {
        let c = rng.below(3) as usize;
        let sh = rng.below(2) as usize;
        let utter = format!("the {} {}", colours[c], shapes[sh]);
        lx.hear(&utter, &[(0, c as i64), (1, sh as i64)], s);
    }
    assert_eq!(lx.meaning("red"), Some((0, 0)));
    assert_eq!(lx.meaning("cube"), Some((1, 1)));
    // failure case: a function word co-occurs with everything and grounds nothing
    assert_eq!(lx.meaning("the"), None);
    // misspelling resolves through the character profile
    assert_eq!(lx.resolve("bleu").as_deref(), Some("blue"));
    // compositional understanding and description of a combination
    assert_eq!(lx.understand("green ball"), vec![(0, 1), (1, 0)]);
    assert_eq!(lx.describe(&[(1, 1), (0, 2)]), "blue cube");
    // failure case: too few scenes -> no licence
    let mut lx2 = Lexicon::new(6);
    lx2.hear("zork", &[(0, 9)], 1);
    lx2.hear("zork", &[(0, 9)], 2);
    assert_eq!(lx2.meaning("zork"), None);
}
