//! Party path-selection at forks (dungeon graph, Phase B).
//!
//! At a fork a party picks one exit. Two modes, per the founder's design call:
//! - **Greedy** (default): adventurers are drawn to loot and shy of obvious
//!   danger — so the player baits them down a treasure branch that is *actually*
//!   a killbox, or under-defends a branch and watches it slip past.
//! - **Beeline** (desperation): when the realm is losing adventurers too fast
//!   (high threat) or a siege/event forces it, they stop looting and storm the
//!   shortest path to the Core to end the dungeon.
//!
//! Floors are still linear today (one exit), so this is dormant until the fork
//! build op (Phase C) — but it is fully unit-tested against hand-built forks.

use crate::game_state::{AdventurerParty, Floor, GameState};

/// Threat tier at which adventurers turn desperate and beeline for the Core.
const BEELINE_THREAT_TIER: i32 = 3;

/// Distance, in rooms, from `pos` to the Core sink following `exits`
/// (breadth-first). `None` if the Core is unreachable from `pos`.
pub fn distance_to_core(floor: &Floor, pos: usize) -> Option<u32> {
    use std::collections::{HashSet, VecDeque};
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((pos, 0u32));
    while let Some((p, dist)) = queue.pop_front() {
        if !seen.insert(p) {
            continue;
        }
        let Some(room) = floor.room_at(p) else {
            continue;
        };
        if room.room_type == crate::game_state::RoomType::Core {
            return Some(dist);
        }
        for &next in &room.exits {
            queue.push_back((next, dist + 1));
        }
    }
    None
}

/// Is this party in beeline (rush-the-Core) mode? True during a siege, at high
/// realm threat, or whenever a future event/quest sets it.
pub fn is_beelining(state: &GameState, party: &AdventurerParty) -> bool {
    party.sieging || state.threat_tier() >= BEELINE_THREAT_TIER
}

/// Player-facing reason for the party's next fork choice. This stays derived
/// from simulation state so it can never disagree with `choose_exit`.
pub fn choice_reason(state: &GameState, party: &AdventurerParty, exits: &[usize]) -> &'static str {
    match exits {
        [] | [_] => "following the only route",
        _ if is_beelining(state, party) => "beelining for the Core",
        _ => "following the lure of loot",
    }
}

/// How appealing a candidate room is to a *greedy* party: loot pulls them in,
/// visible defenders push them away, and nearness to the Core gently breaks ties.
fn appeal(floor: &Floor, pos: usize) -> f32 {
    let Some(room) = floor.room_at(pos) else {
        return f32::MIN;
    };
    let loot = room.loot as f32 + (room.treasure_multiplier() - 1.0) * 100.0;
    let threat: i32 = room
        .monsters
        .iter()
        .filter(|m| m.alive)
        .map(|m| m.scaled_stats.attack)
        .sum();
    let core_bias = distance_to_core(floor, pos)
        .map(|d| -(d as f32) * 0.5)
        .unwrap_or(0.0);
    loot - threat as f32 + core_bias
}

/// Choose which exit a party takes. `exits` must be non-empty (the caller treats
/// an exit-less room as the Core sink). A single exit is taken as-is (today's
/// linear behavior); a fork is resolved by mode.
pub fn choose_exit(
    state: &GameState,
    floor: &Floor,
    party: &AdventurerParty,
    exits: &[usize],
) -> usize {
    match exits {
        [] => 0, // defensive: caller guarantees non-empty
        [only] => *only,
        _ => {
            if is_beelining(state, party) {
                // Straight for the heart: the branch nearest the Core.
                *exits
                    .iter()
                    .min_by_key(|&&p| distance_to_core(floor, p).unwrap_or(u32::MAX))
                    .unwrap_or(&exits[0])
            } else {
                // Drawn to plunder, wary of a defended path.
                *exits
                    .iter()
                    .max_by(|&&a, &&b| {
                        appeal(floor, a)
                            .partial_cmp(&appeal(floor, b))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap_or(&exits[0])
            }
        }
    }
}

#[cfg(test)]
mod tests;
