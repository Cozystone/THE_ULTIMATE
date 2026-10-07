//! E-B: Phase B gates (event memory, symbol grounding, bridge to relational laws).
//! Writes experiments/results/phase_b.json. Floats only for reporting.

use bm_memory::*;
use bm_relation::{FeatureKind, LicensePolicy, RelationEngine};
use bm_worlds::ground::{self as gw, GObj, GroundWorld, Setup, JUNK};
use bm_worlds::os::{self as osw, FsWorld};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

struct Out {
    report: String,
    json: Vec<String>,
    all: bool,
}

impl Out {
    fn gate(&mut self, name: &str, pass: bool, detail: String) {
        self.all &= pass;
        let _ = writeln!(self.report, "  {name:<22} {}  {detail}", if pass { "PASS" } else { "FAIL" });
        self.json.push(format!("{{\"gate\":\"{name}\",\"pass\":{pass},\"detail\":{:?}}}", detail));
    }
}

fn pct(a: u64, b: u64) -> f64 {
    100.0 * a as f64 / b.max(1) as f64
}

/// (truth object, concept) pairs from a grounding, skipping junk and unassigned slots.
fn pairs(g: &Grounded, truth: &[(u16, usize)], grounder: &Grounder) -> Vec<(usize, u32)> {
    let mut v = Vec::new();
    for s in &g.slots {
        let Some(k) = s.concept else { continue };
        let Some(&(_, o)) = truth.iter().find(|x| x.0 == s.slot) else { continue };
        v.push((o, grounder.resolve(k)));
    }
    v
}

fn quality(p: &[(usize, u32)]) -> (f64, f64) {
    let q: Vec<(u32, u32)> = p.iter().filter(|x| x.0 != JUNK).map(|&(o, k)| (o as u32, k)).collect();
    let mut by_c: HashMap<u32, HashMap<u32, u32>> = HashMap::new();
    let mut by_t: HashMap<u32, HashMap<u32, u32>> = HashMap::new();
    for &(t, c) in &q {
        *by_c.entry(c).or_default().entry(t).or_insert(0) += 1;
        *by_t.entry(t).or_default().entry(c).or_insert(0) += 1;
    }
    let n = q.len().max(1) as f64;
    let pure: u32 = by_c.values().map(|h| *h.values().max().unwrap_or(&0)).sum();
    let comp: u32 = by_t.values().map(|h| *h.values().max().unwrap_or(&0)).sum();
    (100.0 * pure as f64 / n, 100.0 * comp as f64 / n)
}

fn corrupt(e: &Event, miss_pct: u64, flip_pct: u64, rng: &mut hdc_core::Rng) -> Event {
    let mut c = e.clone();
    for sc in [&mut c.pre, &mut c.post] {
        sc.tokens.retain(|_| rng.below(100) >= miss_pct);
        for t in sc.tokens.iter_mut() {
            if rng.below(100) < flip_pct {
                t.val = rng.below(gw::range(t.ch) as u64) as i64;
            }
        }
    }
    c
}

/// Facts of a raw event under TRUE object identities (oracle view).
fn true_facts(e: &Event, truth: &[(u16, usize)]) -> Vec<u64> {
    use hdc_core::rng::mix64;
    let obj = |slot: u16| truth.iter().find(|x| x.0 == slot).map(|x| x.1 as u64).unwrap_or(u64::MAX);
    let mut v: Vec<u64> = Vec::new();
    for (ph, sc) in [(1u64, &e.pre), (2u64, &e.post)] {
        for t in &sc.tokens {
            v.push(mix64(mix64(obj(t.slot), ph), mix64(t.ch as u64, t.val as u64)));
        }
    }
    if let Some(a) = &e.act {
        v.push(mix64(0xAC7, a.id as u64));
        for (i, s) in a.args.iter().enumerate() {
            v.push(mix64(0xA6 + i as u64, obj(*s)));
        }
    }
    v.sort_unstable();
    v.dedup();
    v
}

fn overlap(a: &[u64], b: &[u64]) -> u32 {
    let (mut i, mut j, mut n) = (0, 0, 0u32);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                n += 1;
                i += 1;
                j += 1;
            }
        }
    }
    n
}

fn main_world(seed: u64, out: &mut Out) -> (Grounder, GroundWorld) {
    let setup = Setup { noise_pct: 5, miss_pct: 10, junk_pct: 5, ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "lab");
    let mut g = Grounder::new(seed);
    let mut mem = EventMemory::new(seed ^ 9);
    let mut log: Vec<(u64, Vec<(u16, usize)>)> = Vec::new();
    let mut steady: Vec<(usize, u32)> = Vec::new();
    let mut junk_by_concept: HashMap<u32, (u32, u32)> = HashMap::new();
    let n = 2500;
    for i in 0..n {
        let (ev, truth) = w.step(None);
        let id = g.store.len() as u64;
        log.push((id, truth.clone()));
        if let Some(gr) = g.observe(ev) {
            mem.store(&gr);
            let p = pairs(&gr, &truth, &g);
            for &(o, k) in &p {
                let e = junk_by_concept.entry(k).or_insert((0, 0));
                if o == JUNK {
                    e.0 += 1;
                } else {
                    e.1 += 1;
                }
            }
            if i >= 1000 {
                steady.extend(p);
            }
        }
    }
    // ---- B1
    let classes = g.channel_report();
    let class_ok = classes
        .iter()
        .filter(|c| c.0 < gw::N_CH)
        .all(|c| (c.0 >= gw::LIT) == (c.1 == ChannelClass::State) && c.1 != ChannelClass::Unknown);
    let effects = g.action_effect_map();
    let want: BTreeMap<u16, Vec<u16>> =
        [(gw::IDLE, vec![]), (gw::TOGGLE, vec![gw::LIT]), (gw::MOVE, vec![gw::POS]), (gw::COMBINE, vec![gw::LIT])].into_iter().collect();
    let eff_ok = want.iter().all(|(a, chans)| effects.get(a).map(|v| v == chans).unwrap_or(chans.is_empty()));
    let (pur, comp) = quality(&steady);
    let live = g.live_concepts().len();
    let _ = writeln!(out.report, "\n[B1-B4 main world] seed {seed}: {n} events, noise 5%, missing 10%, junk 5%");
    let _ = writeln!(out.report, "    channel classes (ch, class, %changed on args, %changed elsewhere): {:?}", classes);
    let _ = writeln!(out.report, "    noise floor {}%, action-effect map {:?}", g.noise_floor_pct(), effects);
    let _ = writeln!(out.report, "    concepts: live {live}, born {}, decayed protos {}, merges {}, splits {}", g.stats.born, g.stats.decayed, g.stats.merges, g.stats.splits);
    out.gate(
        "B1 structure",
        class_ok && eff_ok && pur >= 97.0 && comp >= 97.0 && live <= 18,
        format!("classes ok {class_ok}, effects ok {eff_ok}, purity {pur:.2}%, completeness {comp:.2}%, live concepts {live}/16"),
    );
    // ---- B2 restoration (after reconsolidation of stored events with the mature grounding)
    let regrounded: Vec<Grounded> = log.iter().filter(|x| x.0 >= 100).map(|x| {
        let e = g.store.get(x.0).clone();
        g.ground(&e)
    }).collect();
    mem.reconsolidate(regrounded.iter());
    let nm = g.noise_model();
    let mut rng = hdc_core::Rng::new(seed ^ 0xB2);
    let candidates: Vec<&(u64, Vec<(u16, usize)>)> = log.iter().filter(|x| x.0 >= 150).collect();
    let mut b2 = String::new();
    let mut b2_ok = true;
    let stored_true: Vec<(u64, Vec<u64>)> = candidates.iter().map(|(id, tr)| (*id, true_facts(g.store.get(*id), tr))).collect();
    for miss in [0u64, 20, 40, 60, 80] {
        let (mut ok, mut wrong, mut abst) = (0u32, 0u32, 0u32);
        let (mut comp_ok, mut comp_n) = (0u32, 0u32);
        let (mut o_ok, mut o_wrong) = (0u32, 0u32);
        for k in 0..300 {
            let (id, truth) = candidates[(k * 7) % candidates.len()];
            let orig = g.store.get(*id).clone();
            let q = corrupt(&orig, miss, 5, &mut rng);
            // oracle: true identities, exact overlap, unique best
            let qf = true_facts(&q, truth);
            let mut best = (0u32, u64::MAX, 0u32);
            for (sid, sf) in &stored_true {
                let o = overlap(&qf, sf);
                if o > best.0 {
                    best = (o, *sid, best.0);
                } else if o > best.2 {
                    best.2 = o;
                }
            }
            if best.0 > best.2 {
                if best.1 == *id {
                    o_ok += 1;
                } else {
                    o_wrong += 1;
                }
            }
            let gq = g.ground(&q);
            let restored = mem.restore_calibrated(&gq, &q, &nm);
            match &restored {
                Some((h, _)) if h.id == *id => ok += 1,
                Some(_) => wrong += 1,
                None => abst += 1,
            }
            // completion of property tokens missing from the query: event-level when the event
            // was recalled, otherwise object-level from the grounding
            for s in &gq.slots {
                let Some(&(_, o)) = truth.iter().find(|x| x.0 == s.slot) else { continue };
                if o == JUNK {
                    continue;
                }
                let present: Vec<u16> = q.pre.slot(s.slot).iter().chain(q.post.slot(s.slot).iter()).map(|x| x.0).collect();
                let from_event = restored.as_ref().and_then(|(_, m)| m.iter().find(|x| x.0 == s.slot)).is_some();
                let filled: Vec<(u16, i64)> = match restored.as_ref().and_then(|(_, m)| m.iter().find(|x| x.0 == s.slot)) {
                    Some((_, sg)) => sg.props.iter().copied().filter(|p| !present.contains(&p.0)).collect(),
                    None => s.props.iter().copied().filter(|p| s.completed.contains(&p.0)).collect(),
                };
                for (c, v) in filled {
                    if c >= gw::LIT {
                        continue;
                    }
                    comp_n += 1;
                    if v == w.objs[o].get(c) {
                        comp_ok += 1;
                    } else if std::env::var("DIAG_B2").is_ok() {
                        let right_event = restored.as_ref().map(|r| r.0.id == *id).unwrap_or(false);
                        eprintln!("DIAG miss {miss} src {} right_event {right_event} ch {c} filled {v} truth {} obj {o} query_slot_tokens {:?}", if from_event { "event" } else { "object" }, w.objs[o].get(c), q.pre.slot(s.slot));
                    }
                }
            }
        }
        let _ = write!(b2, "miss {miss}%: recall {ok}/300 wrong {wrong} abstain {abst} [oracle {o_ok} wrong {o_wrong}] completion {comp_ok}/{comp_n}; ");
        if miss == 20 && ok * 100 < 300 * 99 {
            b2_ok = false;
        }
        if wrong * 100 > 300 {
            b2_ok = false;
        }
        if comp_n > 0 && (comp_ok as u64) * 100 < comp_n as u64 * 98 {
            b2_ok = false;
        }
    }
    out.gate("B2 restoration", b2_ok, b2);
    // ---- B4 junk
    let junk_concepts = junk_by_concept.iter().filter(|(k, v)| v.0 > v.1 && g.concepts[**k as usize].status == ConceptStatus::Concept).count();
    out.gate("B4 noise discipline", junk_concepts == 0, format!("junk-majority concepts {junk_concepts}, decayed protos {}", g.stats.decayed));
    (g, w)
}

fn novelty(seed: u64, g: &mut Grounder, w: &mut GroundWorld, out: &mut Out) {
    let old: std::collections::HashSet<u32> = g.live_concepts().into_iter().collect();
    let mut novel = Vec::new();
    for i in 0..4 {
        let colour = w.rng().below(8) as i64;
        let size = w.rng().below(4) as i64;
        novel.push(w.add(GObj { props: [colour, 4, size, 0, 100 + i], lit: 0, pos: 0, ty: 4 }));
    }
    let mut assigned_old = 0u32;
    let mut novel_seen = 0u32;
    let mut first_new: HashMap<usize, u32> = HashMap::new();
    let mut count: HashMap<usize, u32> = HashMap::new();
    let mut old_pairs = Vec::new();
    let mut novel_pairs = Vec::new();
    for _ in 0..900 {
        let (ev, truth) = w.step(None);
        let Some(gr) = g.observe(ev) else { continue };
        for (o, k) in pairs(&gr, &truth, g) {
            if novel.contains(&o) {
                novel_seen += 1;
                let c = count.entry(o).or_insert(0);
                *c += 1;
                if old.contains(&k) {
                    assigned_old += 1;
                } else {
                    first_new.entry(o).or_insert(*c);
                    novel_pairs.push((o, k));
                }
            } else if o != JUNK {
                old_pairs.push((o, k));
            }
        }
        // sightings not yet assigned to a born concept still count toward the 30-sighting bound
        for s in &gr.slots {
            if s.concept.is_none() {
                if let Some(&(_, o)) = truth.iter().find(|x| x.0 == s.slot) {
                    if novel.contains(&o) {
                        *count.entry(o).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    let birth_ok = novel.iter().all(|o| first_new.get(o).map(|&c| c <= 30).unwrap_or(false));
    let no_false_merge = (assigned_old as u64) * 100 <= novel_seen.max(1) as u64 * 5;
    let (_, old_comp) = quality(&old_pairs);
    let (nov_pur, _) = quality(&novel_pairs);
    out.gate(
        "B3 novelty",
        birth_ok && no_false_merge && old_comp >= 97.0,
        format!(
            "seed {seed}: novel sightings assigned to old concepts {assigned_old}/{novel_seen}; first born-concept sighting per novel object {:?}; novel purity {nov_pur:.1}%; old completeness {old_comp:.2}%",
            novel.iter().map(|o| first_new.get(o).copied()).collect::<Vec<_>>()
        ),
    );
}

fn merge_case(seed: u64, out: &mut Out) {
    let setup = Setup { noise_pct: 5, miss_pct: 10, partial: Some((0, vec![gw::COLOUR, gw::SHAPE], 600)), ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "lab-merge");
    let mut g = Grounder::new(seed);
    let mut late = Vec::new();
    for _ in 0..1800 {
        let (ev, truth) = w.step(None);
        if let Some(gr) = g.observe(ev) {
            if w.t > 1000 {
                late.extend(pairs(&gr, &truth, &g).into_iter().filter(|p| p.0 == 0));
            }
        }
    }
    let mut hist: HashMap<u32, u32> = HashMap::new();
    for &(_, k) in &late {
        *hist.entry(k).or_insert(0) += 1;
    }
    let top = hist.values().max().copied().unwrap_or(0);
    let share = pct(top as u64, late.len() as u64);
    let top_k = hist.iter().max_by_key(|x| x.1).map(|x| *x.0);
    // v2: no surviving duplicate = no other live concept whose sightings are mostly object 0
    let partial_concepts: Vec<u32> = g
        .live_concepts()
        .into_iter()
        .filter(|&k| Some(k) != top_k)
        .filter(|&k| {
            let c = &g.concepts[k as usize];
            c.majority(gw::COLOUR).map(|m| m.0) == Some(w.objs[0].props[0])
                && c.majority(gw::SHAPE).map(|m| m.0) == Some(w.objs[0].props[1])
                && c.hist.len() <= 2
        })
        .collect();
    let v1 = g.stats.merges >= 1 && share >= 95.0;
    out.gate(
        "B5a merge",
        share >= 95.0 && partial_concepts.is_empty(),
        format!(
            "seed {seed}: merges {} (proto absorptions {}), object-0 late sightings on one concept {share:.1}% ({} sightings), surviving partial duplicates {}; v1 criterion would give {}",
            g.stats.merges,
            g.stats.proto_merges,
            late.len(),
            partial_concepts.len(),
            if v1 { "PASS" } else { "FAIL" }
        ),
    );
}

fn split_case(seed: u64, out: &mut Out) {
    let setup = Setup { noise_pct: 5, miss_pct: 10, hidden_until: Some((gw::MARK, 800)), exclusive: Some((0, 1)), ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "lab-split");
    w.make_lookalike_pair(0, 1);
    let mut g = Grounder::new(seed);
    let (mut before, mut after) = (Vec::new(), Vec::new());
    for _ in 0..2400 {
        let (ev, truth) = w.step(None);
        let t = w.t;
        if let Some(gr) = g.observe(ev) {
            let p: Vec<(usize, u32)> = pairs(&gr, &truth, &g).into_iter().filter(|p| p.0 <= 1).collect();
            if (300..800).contains(&t) {
                before.extend(p);
            } else if t > 1300 {
                after.extend(p);
            }
        }
    }
    // before reveal: the two look-alikes share one concept
    let mut hb: HashMap<u32, u32> = HashMap::new();
    for &(_, k) in &before {
        *hb.entry(k).or_insert(0) += 1;
    }
    let shared = pct(hb.values().max().copied().unwrap_or(0) as u64, before.len() as u64);
    let (pur, _) = quality(&after);
    let v2 = g.stats.splits >= 1 && shared >= 90.0 && pur >= 95.0;
    out.gate(
        "B5b split",
        shared >= 90.0 && pur >= 95.0,
        format!(
            "seed {seed}: splits {}, births {}, pre-reveal sightings on one concept {shared:.1}%, post-reveal purity {pur:.1}%; v2 criterion (split required) would give {}",
            g.stats.splits,
            g.stats.born,
            if v2 { "PASS" } else { "FAIL" }
        ),
    );
}

fn os_case(seed: u64, out: &mut Out) {
    let mut w = match FsWorld::new(seed, 12, 8, "phaseb") {
        Ok(w) => w,
        Err(e) => {
            out.gate("B6 OS", false, format!("sandbox error {e}"));
            return;
        }
    };
    let mut g = Grounder::new(seed);
    let mut steady = Vec::new();
    for i in 0..700 {
        let (ev, truth) = match w.step(None) {
            Ok(x) => x,
            Err(e) => {
                out.gate("B6 OS", false, format!("io error {e}"));
                return;
            }
        };
        if let Some(gr) = g.observe(ev) {
            if i > 300 {
                steady.extend(pairs(&gr, &truth, &g));
            }
        }
    }
    let classes = g.channel_report();
    let class_ok = classes.iter().all(|c| (c.0 >= osw::SIZE) == (c.1 == ChannelClass::State) && c.1 != ChannelClass::Unknown);
    let (pur, comp) = quality(&steady);
    let _ = writeln!(out.report, "    OS sandbox {}: channel classes {:?}; refused actions {}", w.root.display(), classes, w.refused.len());
    out.gate(
        "B6 OS",
        class_ok && pur >= 97.0,
        format!("seed {seed}: real files, classes ok {class_ok}, file purity {pur:.2}%, completeness {comp:.2}%, live concepts {} for 12 files", g.live_concepts().len()),
    );
}

fn bridge_case(seed: u64, out: &mut Out) {
    let setup = Setup { noise_pct: 5, miss_pct: 10, ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "lab-bridge");
    let mut g = Grounder::new(seed);
    let mut policy = LicensePolicy::default();
    let mut rel = RelationEngine::with_policy(seed ^ 0xB7, policy.clone());
    bm_bench::declare(&mut rel, bm_worlds::ground::ORDINAL);
    let lit0 = target_id(0, gw::LIT);
    let mut fed = 0;
    for _ in 0..3000 {
        let (ev, _) = w.step(None);
        if let Some(gr) = g.observe(ev) {
            if rel.policy.noise_tol_num == 0 {
                // D020: tolerance from the measured sensor noise floor (pre/post change rate)
                let floor = g.noise_floor_pct().max(1);
                policy.noise_tol_num = floor;
                policy.noise_tol_den = 100;
                rel.policy = policy.clone();
            }
            if let Some(ep) = to_episode(&gr) {
                rel.observe(ep);
                fed += 1;
            }
        }
    }
    let ctx = w.context;
    let lic: Vec<usize> = rel.licensed_in(ctx);
    let colour_rel: Vec<usize> = lic
        .iter()
        .copied()
        .filter(|&l| {
            let law = &rel.laws[l];
            law.target == lit0
                && law.action == gw::COMBINE
                && law.condition.iter().any(|f| matches!(f, FeatureKind::Same { ch: gw::COLOUR, .. } | FeatureKind::Diff { ch: gw::COLOUR, .. }))
        })
        .collect();
    for &l in colour_rel.iter().take(4) {
        let _ = writeln!(out.report, "    licensed: {}", rel.summary(l, ctx));
    }
    for law in rel.laws.iter().filter(|l| {
        l.target == lit0 && l.action == gw::COMBINE && l.condition.len() == 1
            && matches!(l.condition[0], FeatureKind::Same { ch: gw::COLOUR, .. } | FeatureKind::Diff { ch: gw::COLOUR, .. })
    }) {
        let _ = writeln!(out.report, "    colour law: {}", rel.summary(law.id, ctx));
    }
    // novel objects with unseen colours, familiarised without COMBINE
    let mut novel = Vec::new();
    for (i, c) in [8i64, 8, 9, 9, 10, 11].into_iter().enumerate() {
        novel.push(w.add(GObj { props: [c, (i % 4) as i64, 1, ((i % 4) * 2 + 1) as i64, 200 + i as i64], lit: 0, pos: 0, ty: 9 }));
    }
    for k in 0..300 {
        let o = novel[k % novel.len()];
        let act = if k % 2 == 0 { gw::TOGGLE } else { gw::MOVE };
        let (ev, _) = w.step(Some((act, vec![o])));
        if let Some(gr) = g.observe(ev) {
            if let Some(ep) = to_episode(&gr) {
                rel.observe(ep);
            }
        }
    }
    let (mut ok, mut wrong, mut abst) = (0u32, 0u32, 0u32);
    for k in 0..200 {
        let i = k % novel.len();
        let j = (i + 1 + (k / novel.len()) % (novel.len() - 1)) % novel.len();
        let (a, b) = (novel[i], novel[j]);
        let truth = (w.objs[a].props[0] == w.objs[b].props[0]) as i64;
        let (ev, _) = w.step(Some((gw::COMBINE, vec![a, b])));
        let gr = g.ground(&ev);
        let Some(ep) = to_episode(&gr) else { continue };
        match rel.predict(&ep.without_outcomes(), lit0).value() {
            Some(v) if v == truth => ok += 1,
            Some(_) => wrong += 1,
            None => abst += 1,
        }
    }
    let n = (ok + wrong + abst).max(1);
    out.gate(
        "B7 bridge",
        !colour_rel.is_empty() && ok * 100 >= n * 90 && wrong * 100 <= n * 5,
        format!(
            "seed {seed}: {fed} grounded episodes, noise tolerance {}/{}, licensed colour-relational COMBINE laws {}, unseen-colour COMBINE predictions correct {ok} wrong {wrong} abstain {abst}",
            policy.noise_tol_num,
            policy.noise_tol_den,
            colour_rel.len()
        ),
    );
}

fn main() {
    let started = std::time::Instant::now();
    let seeds: Vec<u64> = std::env::args().nth(1).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![1, 2, 3]);
    let mut out = Out { report: String::new(), json: Vec::new(), all: true };
    for &seed in &seeds {
        let _ = writeln!(out.report, "\n===== seed {seed}");
        let (mut g, mut w) = main_world(seed, &mut out);
        novelty(seed, &mut g, &mut w, &mut out);
        merge_case(seed, &mut out);
        split_case(seed, &mut out);
        os_case(seed, &mut out);
        bridge_case(seed, &mut out);
    }
    println!("{}", out.report);
    println!("E-B OVERALL: {}", if out.all { "PASS" } else { "FAIL" });
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experiments/results/phase_b.json");
    std::fs::write(&path, format!("{{\"overall_pass\":{},\"gates\":[\n{}\n]}}", out.all, out.json.join(",\n"))).expect("write");
    println!("{}", bm_bench::resources_line(started));
}
