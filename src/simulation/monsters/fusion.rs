//! Fusion compresses two identical, equal-rank defenders into one stronger
//! veteran. It deliberately yields less raw power than two bodies while
//! freeing a scarce room slot, creating a capacity-versus-force decision.

use crate::data::constants::get_scaled_stats;
use crate::data::monsters::get_monster_template;
use crate::game_state::{GameState, LogEntry, Stats};

pub const FUSION_RANK_MAX: u8 = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FusionPlan {
    RankUp(u8),
    Resonance(String),
}

pub(super) fn fusion_multiplier(rank: u8) -> f32 {
    match rank.clamp(1, FUSION_RANK_MAX) {
        1 => 1.0,
        2 => 1.35,
        _ => 1.75,
    }
}

pub(super) fn ranked_stats(stats: Stats, rank: u8) -> Stats {
    let multiplier = fusion_multiplier(rank);
    Stats {
        hp: (stats.hp as f32 * multiplier).round().max(1.0) as i32,
        attack: (stats.attack as f32 * multiplier).round().max(1.0) as i32,
        defense: (stats.defense as f32 * multiplier).round().max(0.0) as i32,
    }
}

fn fusion_partner(room: &crate::game_state::Room, monster_id: u64) -> Option<(u64, FusionPlan)> {
    let primary = room
        .monsters
        .iter()
        .find(|monster| monster.id == monster_id)?;
    if primary.fusion_rank < FUSION_RANK_MAX {
        if let Some(candidate) = room.monsters.iter().find(|candidate| {
            candidate.id != monster_id
                && candidate.type_name == primary.type_name
                && candidate.fusion_rank == primary.fusion_rank
        }) {
            return Some((candidate.id, FusionPlan::RankUp(primary.fusion_rank + 1)));
        }
    }
    let primary_element = get_monster_template(&primary.type_name)?.element?;
    room.monsters
        .iter()
        .find(|candidate| {
            candidate.id != monster_id
                && candidate.alive
                && candidate.type_name != primary.type_name
                && candidate.fusion_rank == primary.fusion_rank
                && get_monster_template(&candidate.type_name)
                    .and_then(|template| template.element)
                    .is_some_and(|element| element == primary_element)
        })
        .map(|candidate| {
            (
                candidate.id,
                FusionPlan::Resonance(candidate.type_name.clone()),
            )
        })
}

/// Rank a row's Fuse button would create, or None when there is no compatible
/// partner in the same room.
pub fn fusion_target_rank(room: &crate::game_state::Room, monster_id: u64) -> Option<u8> {
    match fusion_partner(room, monster_id)?.1 {
        FusionPlan::RankUp(rank) => Some(rank),
        FusionPlan::Resonance(_) => None,
    }
}

pub fn fusion_plan(room: &crate::game_state::Room, monster_id: u64) -> Option<FusionPlan> {
    fusion_partner(room, monster_id).map(|(_, plan)| plan)
}

pub fn merge_monsters(
    state: &mut GameState,
    floor_num: i32,
    room_pos: usize,
    primary_id: u64,
) -> Result<(), String> {
    if !state.adventurer_parties.is_empty() {
        return Err("Cannot fuse defenders while adventurers are in the dungeon!".into());
    }

    let (partner_id, plan, type_name, is_boss, old_rank) = {
        let room = state
            .floors
            .iter()
            .find(|floor| floor.number == floor_num)
            .and_then(|floor| floor.room_at(room_pos))
            .ok_or("Room not found")?;
        let primary = room
            .monsters
            .iter()
            .find(|monster| monster.id == primary_id)
            .ok_or("Defender not found")?;
        let (partner, plan) = fusion_partner(room, primary_id).ok_or(
            "This defender needs an identical equal-rank partner or a same-element resonance partner.",
        )?;
        (
            partner,
            plan,
            primary.type_name.clone(),
            primary.is_boss,
            primary.fusion_rank,
        )
    };

    let new_rank = old_rank + 1;
    let template = get_monster_template(&type_name).ok_or("Monster data unavailable")?;
    let base = get_scaled_stats(
        Stats {
            hp: template.hp,
            attack: template.attack,
            defense: template.defense,
        },
        floor_num,
        is_boss,
    );
    let ranked = ranked_stats(base, new_rank);

    let room = state
        .floors
        .iter_mut()
        .find(|floor| floor.number == floor_num)
        .and_then(|floor| floor.room_at_mut(room_pos))
        .ok_or("Room not found")?;
    let partner_index = room
        .monsters
        .iter()
        .position(|monster| monster.id == partner_id)
        .ok_or("Fusion partner disappeared")?;
    room.monsters.remove(partner_index);
    let primary = room
        .monsters
        .iter_mut()
        .find(|monster| monster.id == primary_id)
        .ok_or("Defender disappeared")?;
    match &plan {
        FusionPlan::RankUp(_) => {
            primary.fusion_rank = new_rank;
            primary.scaled_stats = ranked;
            primary.max_hp = ranked.hp;
            primary.hp = ranked.hp;
            primary.alive = true;
        }
        FusionPlan::Resonance(partner_type) => {
            let already_resonant = primary
                .active_traits
                .iter()
                .any(|trait_data| trait_data.id == "resonance_strike");
            if !already_resonant {
                primary.active_traits.push(crate::game_state::ActiveTrait {
                    id: "resonance_strike".to_string(),
                    name: format!("Resonance: {partner_type}"),
                    cooldown_timer: 0,
                });
                primary.active_traits.push(crate::game_state::ActiveTrait {
                    id: "resonance_guard".to_string(),
                    name: "Resonant Shell".to_string(),
                    cooldown_timer: 0,
                });
            }
            primary.hp = primary.max_hp;
            primary.alive = true;
        }
    }

    match plan {
        FusionPlan::RankUp(_) => state.add_log(LogEntry::building(format!(
            "Two rank {old_rank} {type_name}s fused into rank {new_rank} on floor {floor_num}, room {room_pos}."
        ))),
        FusionPlan::Resonance(partner_type) => state.add_log(LogEntry::building(format!(
            "{type_name} absorbed a same-element {partner_type} on floor {floor_num}: Resonance traits awakened."
        ))),
    }
    Ok(())
}

#[cfg(test)]
mod tests;
