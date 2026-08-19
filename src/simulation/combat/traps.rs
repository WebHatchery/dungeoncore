//! Room trap resolution (damage, DoT, control, economy) and between-raid re-arming.

use crate::data::elements::element_multiplier;
use crate::game_state::{
    Condition, EffectKind, ElementSound, GameState, LogEntry, RoomUpgradeType, SoundEvent,
};

use super::helpers::{adventurer_element, random_alive_idx};

/// Chance per combat tick that a room's trap upgrade fires.
const TRAP_TRIGGER_CHANCE: f32 = 0.2;
/// Chance a triggered trap is safely sprung when a Rogue is in the party.
const ROGUE_DISARM_CHANCE: f32 = 0.3;
/// Combat ticks a poison/burn condition lasts.
const CONDITION_TICKS: i32 = 4;

/// Fire the room's trap: chance to trigger, Rogue counterplay, then an
/// effect determined by the trap's kind. Elemental traps hit matchups and
/// are empowered by a matching room attunement.
pub(super) fn resolve_trap(
    state: &mut GameState,
    party_idx: usize,
    floor_idx: usize,
    room_idx: usize,
) {
    let Some(trap) = state.floors[floor_idx].rooms[room_idx]
        .upgrade_of(RoomUpgradeType::Trap)
        .cloned()
    else {
        return;
    };
    if trap.disarmed || !state.run_rng.chance(TRAP_TRIGGER_CHANCE) {
        return;
    }

    let floor_num = state.floors[floor_idx].number;
    let room_pos = state.floors[floor_idx].rooms[room_idx].position;
    state.queue_sound(SoundEvent::Trap);
    if let Some(element) = trap.element.as_deref().and_then(ElementSound::from_id) {
        state.queue_sound(SoundEvent::ElementalHit(element));
    }

    // Rogue counterplay: the trap is sprung safely and stays down this raid.
    let has_rogue = state.adventurer_parties[party_idx]
        .members
        .iter()
        .any(|a| a.alive && a.class_name == "Rogue");
    if has_rogue && state.run_rng.chance(ROGUE_DISARM_CHANCE) {
        if let Some(installed) = state.floors[floor_idx].rooms[room_idx]
            .upgrades
            .iter_mut()
            .find(|u| u.upgrade_type == RoomUpgradeType::Trap)
        {
            installed.disarmed = true;
        }
        state.add_log(LogEntry::combat(format!(
            "A Rogue disarms the {}! It stays down until re-armed.",
            trap.name
        )));
        state.push_effect(floor_num, room_pos, "Disarmed!", EffectKind::Ability);
        return;
    }

    // Matching room attunement empowers an elemental trap.
    let attune_boost = match state.floors[floor_idx].rooms[room_idx].attunement() {
        Some((element, mult)) if Some(element) == trap.element.as_deref() => mult,
        _ => 1.0,
    };
    let trap_element = trap.element.clone().unwrap_or_default();

    match trap.effect_kind.as_str() {
        "Poison" | "Burn" => {
            let power = (trap.multiplier * attune_boost).round().max(1.0) as i32;
            let victim_name = {
                let party = &mut state.adventurer_parties[party_idx];
                random_alive_idx(&mut state.run_rng, party).map(|idx| {
                    let victim = &mut party.members[idx];
                    victim.conditions.push(Condition {
                        kind: trap.effect_kind.clone(),
                        ticks: CONDITION_TICKS,
                        power,
                        multiplier: 1.0,
                    });
                    victim.name.clone()
                })
            };
            if let Some(name) = victim_name {
                state.add_log(LogEntry::combat(format!(
                    "{} afflicts {} ({} {}/tick)!",
                    trap.name, name, trap.effect_kind, power
                )));
                state.push_effect_at(
                    floor_num,
                    room_pos,
                    format!("{}!", trap.effect_kind),
                    if trap.effect_kind == "Poison" {
                        EffectKind::PoisonCloud
                    } else {
                        EffectKind::Ability
                    },
                    crate::game_state::EffectAnchor::Invaders,
                );
            }
        }
        "Snare" => {
            let ticks = trap.multiplier.round().max(1.0) as i32;
            let party = &mut state.adventurer_parties[party_idx];
            party.snared_ticks = party.snared_ticks.max(ticks);
            state.add_log(LogEntry::combat(format!(
                "{} holds the party fast for {} ticks!",
                trap.name, ticks
            )));
            state.push_effect(floor_num, room_pos, "Held!", EffectKind::Ability);
        }
        "Alarm" => {
            let party = &mut state.adventurer_parties[party_idx];
            if !party.alarmed {
                party.alarmed = true;
                state.add_log(LogEntry::combat(format!(
                    "{} sounds! Every defender fights harder against this party.",
                    trap.name
                )));
                state.push_effect(floor_num, room_pos, "Alarm!", EffectKind::Ability);
            }
        }
        "ManaSiphon" => {
            let gain = (trap.multiplier * attune_boost).round() as i32;
            state.mana = (state.mana + gain).min(state.max_mana);
            state.add_log(LogEntry::combat(format!(
                "{} drinks the party's magic: +{} mana.",
                trap.name, gain
            )));
            state.push_effect(
                floor_num,
                room_pos,
                format!("+{} mana", gain),
                EffectKind::Loot,
            );
        }
        "GoldSteal" => {
            let party = &mut state.adventurer_parties[party_idx];
            let steal = party.loot.min(trap.multiplier.round() as i32);
            if steal > 0 {
                party.loot -= steal;
                state.gold += steal;
                state.add_log(LogEntry::combat(format!(
                    "{} pockets {} gold from the party's haul.",
                    trap.name, steal
                )));
                state.push_effect(
                    floor_num,
                    room_pos,
                    format!("+{}g", steal),
                    EffectKind::Loot,
                );
            }
        }
        // "Damage" plus legacy traps from old saves (empty effect_kind,
        // multiplier-style values around 1.2–1.5).
        _ => {
            let base = if trap.effect_kind == "Damage" {
                trap.multiplier
            } else {
                10.0 * trap.multiplier
            };
            let hit = {
                let party = &mut state.adventurer_parties[party_idx];
                random_alive_idx(&mut state.run_rng, party).map(|idx| {
                    let victim = &mut party.members[idx];
                    let elem_mult =
                        element_multiplier(&trap_element, &adventurer_element(&victim.class_name));
                    let damage = (base * attune_boost * elem_mult).round().max(1.0) as i32;
                    victim.hp -= damage;
                    if victim.hp <= 0 {
                        victim.hp = 0;
                        victim.alive = false;
                        party.casualties += 1;
                        (victim.name.clone(), victim.level, damage, true)
                    } else {
                        (victim.name.clone(), 0, damage, false)
                    }
                })
            };
            if let Some((victim_name, level, damage, killed)) = hit {
                if killed {
                    let mana_gain = level * 10;
                    state.mana = (state.mana + mana_gain).min(state.max_mana);
                    state.total_deaths += 1;
                    state.add_log(LogEntry::combat(format!(
                        "{} killed {}! +{} mana",
                        trap.name, victim_name, mana_gain
                    )));
                    state.push_effect(floor_num, room_pos, "Trapped!", EffectKind::AdventurerDown);
                } else {
                    state.add_log(LogEntry::combat(format!(
                        "{} dealt {} damage to {}",
                        trap.name, damage, victim_name
                    )));
                    state.push_element_effect_at(
                        floor_num,
                        room_pos,
                        format!("-{}", damage),
                        EffectKind::Damage,
                        crate::game_state::EffectAnchor::Invaders,
                        &trap_element,
                    );
                    state.push_element_effect_at(
                        floor_num,
                        room_pos,
                        "",
                        EffectKind::HitSpark,
                        crate::game_state::EffectAnchor::Invaders,
                        &trap_element,
                    );
                }
            }
        }
    }

    apply_secondary_effect(state, party_idx, floor_num, room_pos, &trap);
}

/// Apply the second rule carried by a trap template. These effects deliberately
/// live on the party's normal condition list so they survive a room transition
/// for a few ticks and are visible to the same combat math as poison and burn.
fn apply_secondary_effect(
    state: &mut GameState,
    party_idx: usize,
    floor_num: i32,
    room_pos: usize,
    trap: &crate::game_state::RoomUpgrade,
) {
    let kind = trap.secondary_kind.as_str();
    if kind.is_empty() {
        return;
    }

    let condition = |condition_kind: &str, multiplier: f32| Condition {
        kind: condition_kind.to_string(),
        ticks: CONDITION_TICKS,
        power: if condition_kind == "Bleeding" {
            trap.secondary_value.round().max(1.0) as i32
        } else {
            0
        },
        multiplier,
    };

    match kind {
        "Bleed" => apply_to_random_adventurer(
            state,
            party_idx,
            condition("Bleeding", 1.0),
            &trap.secondary_effect,
        ),
        "Weaken" => apply_to_random_adventurer(
            state,
            party_idx,
            condition("Weakened", trap.secondary_value),
            &trap.secondary_effect,
        ),
        "ArcaneWeaken" => apply_to_random_adventurer(
            state,
            party_idx,
            condition("ArcaneWeakened", trap.secondary_value),
            &trap.secondary_effect,
        ),
        "Brittle" => apply_to_random_adventurer(
            state,
            party_idx,
            condition("Brittle", trap.secondary_value),
            &trap.secondary_effect,
        ),
        "Snare" => {
            let ticks = trap.secondary_value.round().max(1.0) as i32;
            state.adventurer_parties[party_idx].snared_ticks =
                state.adventurer_parties[party_idx].snared_ticks.max(ticks);
            state.add_log(LogEntry::combat(format!(
                "{}'s secondary impact holds the party for {} more tick{}.",
                trap.name,
                ticks,
                if ticks == 1 { "" } else { "s" }
            )));
        }
        "PartyWeaken" => apply_to_party(
            state,
            party_idx,
            condition("Weakened", trap.secondary_value),
            &trap.secondary_effect,
        ),
        "PartyRally" => apply_to_party(
            state,
            party_idx,
            condition("Rallied", trap.secondary_value),
            &trap.secondary_effect,
        ),
        _ => return,
    }

    state.push_effect(
        floor_num,
        room_pos,
        "Secondary effect!",
        EffectKind::Ability,
    );
}

fn apply_to_random_adventurer(
    state: &mut GameState,
    party_idx: usize,
    condition: Condition,
    description: &str,
) {
    let target_name = {
        let party = &mut state.adventurer_parties[party_idx];
        random_alive_idx(&mut state.run_rng, party).map(|idx| {
            let target = &mut party.members[idx];
            target.conditions.push(condition);
            target.name.clone()
        })
    };
    if let Some(name) = target_name {
        state.add_log(LogEntry::combat(format!(
            "{} suffers an extra room effect.",
            name
        )));
    }
    if !description.is_empty() {
        state.add_log(LogEntry::combat(description.to_string()));
    }
}

fn apply_to_party(
    state: &mut GameState,
    party_idx: usize,
    condition: Condition,
    description: &str,
) {
    for member in state.adventurer_parties[party_idx]
        .members
        .iter_mut()
        .filter(|member| member.alive)
    {
        member.conditions.push(condition.clone());
    }
    if !description.is_empty() {
        state.add_log(LogEntry::combat(description.to_string()));
    }
}

/// Re-arm disarmed traps between raids; each costs a quarter of its
/// original mana price. Unaffordable traps stay down until next time.
pub fn rearm_traps(state: &mut GameState) -> i32 {
    let mut rearmed: Vec<(String, i32)> = Vec::new();
    for floor_idx in 0..state.floors.len() {
        for room_idx in 0..state.floors[floor_idx].rooms.len() {
            let Some(upgrade_idx) = state.floors[floor_idx].rooms[room_idx]
                .upgrades
                .iter()
                .position(|u| u.upgrade_type == RoomUpgradeType::Trap && u.disarmed)
            else {
                continue;
            };
            let name = state.floors[floor_idx].rooms[room_idx].upgrades[upgrade_idx]
                .name
                .clone();
            let cost = crate::data::upgrades::get_upgrade_template(&name)
                .map(|t| t.mana_cost / 4)
                .unwrap_or(0);
            if state.mana >= cost {
                state.mana -= cost;
                state.floors[floor_idx].rooms[room_idx].upgrades[upgrade_idx].disarmed = false;
                rearmed.push((name, cost));
            }
        }
    }
    let spent = rearmed.iter().map(|(_, cost)| *cost).sum();
    for (name, cost) in rearmed {
        state.add_log(LogEntry::building(format!(
            "Re-armed {} for {} mana.",
            name, cost
        )));
    }
    spent
}
