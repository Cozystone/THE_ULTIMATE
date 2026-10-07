//! Bridge from grounded events to the relational substrate (ATANOR path:
//! experience -> episode -> proto-concept -> concept -> relation/law).
//!
//! Roles are the action's arguments in order. Each role filler carries the object's property
//! values (denoised by its concept when known), its concept identity and its pre-action state.
//! Targets are the post-action state channels of each role.

use crate::ground::Grounded;
use bm_relation::{Entity, Episode, Filler};

/// Channel id used for the grounded object identity inside a role filler.
pub const INST_CH: u16 = 1000;

/// Target id for state channel `ch` of role `r`.
pub fn target_id(role: usize, ch: u16) -> u32 {
    role as u32 * 10_000 + ch as u32
}

pub fn decode_target(t: u32) -> (usize, u16) {
    ((t / 10_000) as usize, (t % 10_000) as u16)
}

/// Build a relational episode from a grounded event. None if the event has no action.
pub fn to_episode(g: &Grounded) -> Option<Episode> {
    let act = g.act.as_ref()?;
    let mut roles = Vec::new();
    let mut outcomes = Vec::new();
    for (r, &slot) in act.args.iter().enumerate() {
        let s = g.slot(slot)?;
        let mut fillers: Vec<Filler> = s.props.iter().map(|&(ch, val)| Filler { ch, val }).collect();
        if let Some(k) = s.concept {
            fillers.push(Filler { ch: INST_CH, val: k as i64 });
        }
        for &(ch, val) in &s.pre_states {
            fillers.push(Filler { ch, val });
        }
        roles.push(Entity { fillers, binding: s.concept.map(|k| k as i64) });
        for &(ch, val) in &s.post_states {
            outcomes.push((target_id(r, ch), val));
        }
    }
    Some(Episode { id: 0, t: g.t, context: g.context, source: 1, action: act.id, roles, n_args: 0, outcomes, kind: g.kind })
}

/// Offset of change-flag targets: target_id(r, ch) + CHANGE is 1 if the state changed.
pub const CHANGE: u32 = 5_000;

/// Phase C scene episode: roles are the action arguments followed by every other identified
/// object in concept-id order, so effects on non-argument objects are targets too. Targets are the
/// post-action state value and a change flag for every state channel of every role.
/// Slot order used for scene roles: arguments, then other identified objects by concept id.
pub fn scene_role_order(g: &Grounded) -> Option<Vec<u16>> {
    let act = g.act.as_ref()?;
    let mut order: Vec<u16> = act.args.clone();
    let mut others: Vec<(u32, u16)> = g
        .slots
        .iter()
        .filter(|s| !act.args.contains(&s.slot))
        .filter_map(|s| s.concept.map(|k| (k, s.slot)))
        .collect();
    others.sort();
    order.extend(others.into_iter().map(|x| x.1));
    Some(order)
}

pub fn to_episode_scene(g: &Grounded) -> Option<Episode> {
    let act = g.act.as_ref()?;
    let order = scene_role_order(g)?;
    let mut roles = Vec::new();
    let mut outcomes = Vec::new();
    for (r, &slot) in order.iter().enumerate() {
        let s = g.slot(slot)?;
        let mut fillers: Vec<Filler> = s.props.iter().map(|&(ch, val)| Filler { ch, val }).collect();
        if let Some(k) = s.concept {
            fillers.push(Filler { ch: INST_CH, val: k as i64 });
        }
        for &(ch, val) in &s.pre_states {
            fillers.push(Filler { ch, val });
        }
        roles.push(Entity { fillers, binding: s.concept.map(|k| k as i64) });
        for &(ch, val) in &s.post_states {
            outcomes.push((target_id(r, ch), val));
            if let Some(&(_, pre)) = s.pre_states.iter().find(|x| x.0 == ch) {
                outcomes.push((target_id(r, ch) + CHANGE, (val != pre) as i64));
            }
        }
    }
    let n_args = act.args.len().max(1) as u8;
    Some(Episode { id: 0, t: g.t, context: g.context, source: 2, action: act.id, roles, n_args, outcomes, kind: g.kind })
}
