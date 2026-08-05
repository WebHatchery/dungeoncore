//! Loot, mana, soul, and monster-XP payouts for kills resolved during a tick.

use crate::data::adventurers::get_victory_quotes;
use crate::game_state::{EffectAnchor, EffectKind, GameState, LogEntry, SoundEvent};

/// Grant loot/souls for monsters slain this tick and narrate the kills.
pub(super) fn reward_monster_kills(
    state: &mut GameState,
    party_idx: usize,
    floor_idx: usize,
    room_idx: usize,
    kills: &[(String, bool)],
) {
    if kills.is_empty() {
        return;
    }

    let floor_num = state.floors[floor_idx].number;
    let room_pos = state.floors[floor_idx].rooms[room_idx].position;
    let treasure_mult = state.floors[floor_idx].rooms[room_idx].treasure_multiplier();

    for (monster_name, is_boss) in kills {
        let base_gold = if *is_boss { 50 } else { 20 };
        let gold_reward = (base_gold as f32 * treasure_mult) as i32;
        let soul_reward = if *is_boss { 1 } else { 0 };

        state.adventurer_parties[party_idx].loot += gold_reward;
        if soul_reward > 0 {
            state.souls += soul_reward;
            state.raid_tally().souls_gained += soul_reward;
        }
        // This monster was one of the dungeon's own defenders, now fallen.
        state.raid_tally().defenders_lost += 1;

        state.add_log(LogEntry::combat(format!(
            "{} defeated on floor {}, room {}! +{} gold{}",
            monster_name,
            floor_num,
            room_pos,
            gold_reward,
            if soul_reward > 0 {
                format!(", +{} soul", soul_reward)
            } else {
                String::new()
            }
        )));
        state.push_unit_effect_at(
            floor_num,
            room_pos,
            format!("{} down", monster_name),
            EffectKind::MonsterDown,
            EffectAnchor::Defenders,
            monster_name,
        );
        state.queue_sound(SoundEvent::Death);
    }

    // Victory quote
    let victory_quotes = get_victory_quotes();
    if state.run_rng.chance(0.2) && !victory_quotes.is_empty() {
        let quote = &victory_quotes[state.run_rng.below(victory_quotes.len())];
        if let Some(adv) = state.adventurer_parties[party_idx]
            .members
            .iter()
            .find(|a| a.alive)
        {
            state.add_log(LogEntry::adventure(format!(
                "{} says: \"{}\"",
                adv.name, quote
            )));
        }
    }
}

/// Grant mana/XP for adventurers slain this tick and narrate the deaths.
pub(super) fn reward_adventurer_kills(
    state: &mut GameState,
    party_idx: usize,
    floor_idx: usize,
    room_idx: usize,
    kills: &[(String, i32)],
) {
    if kills.is_empty() {
        return;
    }

    let floor_num = state.floors[floor_idx].number;

    let income_mult = state.income_mult();
    for (victim_name, victim_level) in kills {
        let mana_gain = ((victim_level * 10) as f32 * income_mult).round() as i32;
        state.mana = (state.mana + mana_gain).min(state.max_mana);
        state.total_deaths += 1;
        state.raid_tally().mana_gained += mana_gain;

        // Credit the shared pool of every type that survived the fight. The
        // creature that struck the blow gains nothing itself — its whole line
        // learns from the kill, and that is what unlocks the next variant.
        let room = &state.floors[floor_idx].rooms[room_idx];
        let room_pos = room.position;
        let evolution_mult = room.evolution_multiplier();
        let base_xp = victim_level * 5;
        let xp_gain = (base_xp as f32 * evolution_mult) as i32;

        let earners: Vec<String> = room
            .monsters
            .iter()
            .filter(|m| m.alive)
            .map(|m| m.type_name.clone())
            .collect();
        for type_name in earners {
            state.add_type_experience(&type_name, xp_gain);
        }

        state.add_log(LogEntry::combat(format!(
            "{} has fallen on floor {}! +{} mana, +{} XP to the lines that fought",
            victim_name, floor_num, mana_gain, xp_gain
        )));
        let class_name = state.adventurer_parties[party_idx]
            .members
            .iter()
            .find(|member| member.name == *victim_name)
            .map(|member| member.class_name.clone())
            .unwrap_or_else(|| "Warrior".to_string());
        state.push_unit_effect_at(
            floor_num,
            room_pos,
            "Slain!",
            EffectKind::AdventurerDown,
            EffectAnchor::Invaders,
            class_name,
        );
        state.queue_sound(SoundEvent::Death);
    }
}
