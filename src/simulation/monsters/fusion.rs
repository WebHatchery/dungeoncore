//! Fusion compresses two identical, equal-rank defenders into one stronger
//! veteran. It deliberately yields less raw power than two bodies while
//! freeing a scarce room slot, creating a capacity-versus-force decision.

use crate::data::constants::get_scaled_stats;
use crate::data::monsters::get_monster_template;
use crate::game_state::{GameState, LogEntry, Stats};

pub const FUSION_RANK_MAX: u8 = 3;

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

fn fusion_partner_id(room: &crate::game_state::Room, monster_id: u64) -> Option<u64> {
    let primary = room
        .monsters
        .iter()
        .find(|monster| monster.id == monster_id)?;
    if primary.fusion_rank >= FUSION_RANK_MAX {
        return None;
    }
    room.monsters
        .iter()
        .find(|candidate| {
            candidate.id != monster_id
                && candidate.type_name == primary.type_name
                && candidate.fusion_rank == primary.fusion_rank
        })
        .map(|candidate| candidate.id)
}

/// Rank a row's Fuse button would create, or None when there is no compatible
/// partner in the same room.
pub fn fusion_target_rank(room: &crate::game_state::Room, monster_id: u64) -> Option<u8> {
    let primary = room
        .monsters
        .iter()
        .find(|monster| monster.id == monster_id)?;
    fusion_partner_id(room, monster_id).map(|_| primary.fusion_rank + 1)
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

    let (partner_id, type_name, is_boss, old_rank) = {
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
        let partner = fusion_partner_id(room, primary_id)
            .ok_or("This defender has no identical equal-rank fusion partner.")?;
        (
            partner,
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
    primary.fusion_rank = new_rank;
    primary.scaled_stats = ranked;
    primary.max_hp = ranked.hp;
    primary.hp = ranked.hp;
    primary.alive = true;

    state.add_log(LogEntry::building(format!(
        "Two rank {old_rank} {type_name}s fused into rank {new_rank} on floor {floor_num}, room {room_pos}."
    )));
    Ok(())
}

#[cfg(test)]
mod tests;
