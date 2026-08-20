//! Tick-based combat resolution between adventurer parties and room defenders.
//!
//! The per-tick orchestration lives here; supporting concerns are split into
//! submodules: [`helpers`] (stat/trait/targeting math), [`abilities`] (active
//! monster powers and afflictions), [`traps`] (room trap effects and re-arming),
//! and [`rewards`] (kill payouts).

mod abilities;
mod helpers;
mod rewards;
mod traps;

pub use traps::rearm_traps;

/// Spring a room's trap on a party passing through an *undefended* room (one
/// with no live monsters). Without this, a room's trap only ever fired during
/// combat, so a pure trap corridor — a snare or alarm room placed *before* a
/// killbox — did nothing. Making trap-only rooms matter is what turns room
/// order into a spatial decision.
pub fn spring_undefended_trap(
    state: &mut GameState,
    party_idx: usize,
    floor_idx: usize,
    room_idx: usize,
) {
    resolve_trap(state, party_idx, floor_idx, room_idx);
}

use crate::data::constants::RETREAT_THRESHOLD;
use crate::data::elements::element_multiplier;
use crate::game_state::{EffectAnchor, EffectKind, ElementSound, GameState, LogEntry, SoundEvent};

use abilities::{resolve_abilities, tick_conditions};
use helpers::{
    adventurer_attack_mult, adventurer_damage_taken_mult, adventurer_element, attunement_mult,
    has_passive, monster_attack_value, monster_damage_taken_mult, monster_element, passive_value,
    split_spawn, target_adventurer_idx, target_monster_idx,
};
use rewards::{reward_adventurer_kills, reward_monster_kills};
use traps::resolve_trap;

/// Attack bonus for all monsters fighting a party that tripped an alarm.
const ALARM_ATTACK_MULT: f32 = 1.25;
/// Extra damage a held (snared) party takes — they can't dodge or guard while
/// held fast. This is what makes a snare room placed *before* a killing room a
/// real spatial combo: the party stumbles in still snared and gets pummelled.
const SNARE_VULNERABILITY_MULT: f32 = 1.5;

/// Resolve one combat tick between a party and the monsters in a room.
///
/// Damage model: every living combatant acts once per tick.
/// Adventurers focus the front monster; monsters each strike a random
/// adventurer. Damage = max(1, attack - defense/2), further shaped by
/// room upgrades and monster traits. Deaths occur when HP reaches 0.
pub fn resolve_combat(state: &mut GameState, party_idx: usize, floor_idx: usize, room_idx: usize) {
    let has_alive_monsters = state.floors[floor_idx].rooms[room_idx]
        .monsters
        .iter()
        .any(|m| m.alive);

    if !has_alive_monsters {
        return;
    }

    let floor_num = state.floors[floor_idx].number;
    let room_pos = state.floors[floor_idx].rooms[room_idx].position;
    let stratum = crate::data::strata::stratum_for_floor(floor_num);

    // Visual-only combat punctuation. This is deliberately a transient UI
    // effect; it is neither saved nor consulted by the deterministic resolver.
    if state.adventurer_parties[party_idx]
        .members
        .iter()
        .any(|member| member.alive)
    {
        state.push_effect_at(
            floor_num,
            room_pos,
            "",
            EffectKind::MeleeDust,
            EffectAnchor::Center,
        );
        state.queue_sound(SoundEvent::Combat);
    }

    let room = &state.floors[floor_idx].rooms[room_idx];
    let reinforcement_mult = room.reinforcement_multiplier() * room.defender_attack_multiplier();
    let depth_pressure = state.depth_pressure(floor_num);
    let defender_damage_taken_mult = room.defender_damage_taken_multiplier();
    let battle_order = room.battle_order;
    let adventurer_room_attack_mult = room.adventurer_attack_multiplier();
    let adventurer_damage_to_monsters_mult = room.adventurer_damage_to_monsters_multiplier();
    let monster_regeneration_rate = room.monster_regeneration_rate();

    if monster_regeneration_rate > 0.0 {
        let room = &mut state.floors[floor_idx].rooms[room_idx];
        for monster in room.monsters.iter_mut().filter(|monster| monster.alive) {
            let healing = (monster.max_hp as f32 * monster_regeneration_rate).round() as i32;
            monster.hp = (monster.hp + healing.max(1)).min(monster.max_hp);
        }
    }

    // Phase 0: lingering afflictions (poison, burn) tick on the party
    tick_conditions(state, party_idx, floor_idx, room_idx);

    // Phase 1: the room's trap fires
    resolve_trap(state, party_idx, floor_idx, room_idx);

    // Phase 2: active abilities (e.g. Fire Breath on combat start)
    resolve_abilities(state, party_idx, floor_idx, room_idx, floor_num, room_pos);

    // Room element attunement boosts monsters of the matching element.
    let attunement: Option<(String, f32)> = state.floors[floor_idx].rooms[room_idx]
        .attunement()
        .map(|(element, mult)| (element.to_string(), mult));

    // Phase 3: adventurers strike the front monster — unless a snare trap
    // holds them fast this tick.
    let snared = state.adventurer_parties[party_idx].snared_ticks > 0;
    if snared {
        state.adventurer_parties[party_idx].snared_ticks -= 1;
        state.push_effect_at(
            floor_num,
            room_pos,
            "Snared!",
            EffectKind::Ability,
            EffectAnchor::Invaders,
        );
    }
    let doctrine =
        crate::game_state::doctrine_for_party(state, &state.adventurer_parties[party_idx]);
    let adv_attacks: Vec<(u64, f32, String, crate::game_state::HeroWard)> = if snared {
        Vec::new()
    } else {
        state.adventurer_parties[party_idx]
            .members
            .iter()
            .filter(|a| a.alive)
            .map(|a| {
                let element = adventurer_element(&a.class_name);
                let attack_mult = adventurer_attack_mult(a)
                    * adventurer_room_attack_mult
                    * doctrine.attack_multiplier()
                    * state.floors[floor_idx].rooms[room_idx]
                        .elemental_adventurer_attack_multiplier(&element);
                (
                    a.id,
                    a.scaled_stats.attack as f32 * attack_mult,
                    element,
                    a.ward.clone(),
                )
            })
            .collect()
    };

    let mut monster_kills: Vec<(String, bool)> = Vec::new();
    let mut split_spawns: Vec<String> = Vec::new();
    // (slayer, what they slew) — the journal wants the name, not just a tally.
    let mut kill_credits: Vec<(u64, String)> = Vec::new();
    let mut party_hit_strong = false;
    let mut party_hit_weak = false;
    let mut damage_to_monsters = 0;
    {
        let room = &mut state.floors[floor_idx].rooms[room_idx];
        for (attacker_id, attack, adv_element, ward) in &adv_attacks {
            // Taunting monsters soak hits before the rest of the room.
            let Some(target_idx) = target_monster_idx(&room.monsters) else {
                break;
            };
            let monster = &mut room.monsters[target_idx];
            let mon_element = monster_element(&monster.type_name);
            let attune_mult = attunement_mult(&attunement, &mon_element);
            let effective_def =
                monster.scaled_stats.defense as f32 * reinforcement_mult * attune_mult;
            let taken_mult = monster_damage_taken_mult(monster);
            let elem_mult = element_multiplier(adv_element, &mon_element);
            if elem_mult > 1.0 {
                party_hit_strong = true;
            } else if elem_mult < 1.0 {
                party_hit_weak = true;
            }
            let damage = (((*attack - effective_def / 2.0).max(1.0)
                * taken_mult
                * defender_damage_taken_mult
                * adventurer_damage_to_monsters_mult
                * stratum.guard_multiplier_for(&mon_element)
                * elem_mult)
                * ward.attack_multiplier_against(&mon_element))
            .round()
            .max(1.0) as i32;
            monster.hp -= damage;
            damage_to_monsters += damage;
            if monster.hp <= 0 {
                monster.hp = 0;
                monster.alive = false;
                monster_kills.push((monster.type_name.clone(), monster.is_boss));
                kill_credits.push((*attacker_id, monster.type_name.clone()));
                if has_passive(monster, "SplitOnDeath") {
                    if let Some(spawn) =
                        split_spawn(&mut state.run_rng, &monster.type_name, floor_num)
                    {
                        split_spawns.push(spawn.type_name.clone());
                        room.monsters.push(spawn);
                    }
                }
            }
        }
    }
    for (hero_id, monster_name) in kill_credits {
        state.record_hero_kill(hero_id, &monster_name, floor_num);
    }

    if damage_to_monsters > 0 {
        let impact_element = adv_attacks
            .first()
            .map(|(_, _, element, _)| element.as_str())
            .unwrap_or_default();
        state.push_effect_at(
            floor_num,
            room_pos,
            format!("-{}", damage_to_monsters),
            EffectKind::Damage,
            EffectAnchor::Defenders,
        );
        state.push_element_effect_at(
            floor_num,
            room_pos,
            "",
            EffectKind::HitSpark,
            EffectAnchor::Defenders,
            impact_element,
        );
        if let Some(element) = ElementSound::from_id(impact_element) {
            state.queue_sound(SoundEvent::ElementalHit(element));
        }
    }

    for spawn_name in &split_spawns {
        state.add_log(LogEntry::combat(format!(
            "The slain monster splits — a {} emerges!",
            spawn_name
        )));
        state.push_effect_at(
            floor_num,
            room_pos,
            "Split!",
            EffectKind::Ability,
            EffectAnchor::Defenders,
        );
    }

    // These describe the party's attacks landing (or not) on the defenders.
    if party_hit_strong {
        state.push_effect_at(
            floor_num,
            room_pos,
            "Strong hit!",
            EffectKind::Ability,
            EffectAnchor::Defenders,
        );
    } else if party_hit_weak {
        state.push_effect_at(
            floor_num,
            room_pos,
            "Resisted",
            EffectKind::Ability,
            EffectAnchor::Defenders,
        );
    }

    // Phase 4: surviving monsters strike back (harder if an alarm was tripped)
    let alarm_mult = if state.adventurer_parties[party_idx].alarmed {
        ALARM_ATTACK_MULT
    } else {
        1.0
    };
    let monster_strikes: Vec<MonsterStrike> = {
        let room = &state.floors[floor_idx].rooms[room_idx];
        let alive_count = room.monsters.iter().filter(|m| m.alive).count();
        let enemies_alive = state.adventurer_parties[party_idx]
            .members
            .iter()
            .filter(|a| a.alive)
            .count();
        room.monsters
            .iter()
            .filter(|m| m.alive)
            .map(|m| {
                let element = monster_element(&m.type_name);
                let attune_mult = attunement_mult(&attunement, &element);
                let stratum_mult = stratum.attack_multiplier_for(&element);
                MonsterStrike {
                    monster_id: m.id,
                    attack: monster_attack_value(
                        m,
                        alive_count,
                        enemies_alive,
                        reinforcement_mult
                            * attune_mult
                            * alarm_mult
                            * stratum_mult
                            * depth_pressure,
                    ),
                    element,
                    pierce: has_passive(m, "ArmorPierce"),
                    lifesteal: passive_value(m, "LifeStealPercent"),
                    mana_on_kill: passive_value(m, "ManaOnKill") as i32,
                }
            })
            .collect()
    };

    let mut adventurer_kills: Vec<(String, i32)> = Vec::new();
    let mut damage_to_party = 0;
    let mut monster_hit_strong = false;
    let mut hero_ward_triggered = false;
    let mut lifesteal_heals: Vec<(u64, i32)> = Vec::new();
    let mut leeched_mana = 0;
    {
        let party = &mut state.adventurer_parties[party_idx];
        for strike in &monster_strikes {
            let Some(victim_idx) = target_adventurer_idx(&mut state.run_rng, party, battle_order)
            else {
                break;
            };
            let victim = &mut party.members[victim_idx];
            let elem_mult =
                element_multiplier(&strike.element, &adventurer_element(&victim.class_name));
            if elem_mult > 1.0 {
                monster_hit_strong = true;
            }
            let victim_def = if strike.pierce {
                0.0
            } else {
                victim.scaled_stats.defense as f32 / 2.0
            };
            // A held party can't guard — snared invaders take amplified hits.
            let vuln = if snared {
                SNARE_VULNERABILITY_MULT
            } else {
                1.0
            };
            let ward_mult = victim.ward.damage_multiplier_from(&strike.element);
            hero_ward_triggered |= ward_mult < 1.0;
            let damage = ((strike.attack as f32 - victim_def).max(1.0)
                * elem_mult
                * vuln
                * adventurer_damage_taken_mult(victim)
                * ward_mult)
                .round()
                .max(1.0) as i32;
            victim.hp -= damage;
            damage_to_party += damage;
            if strike.lifesteal > 0.0 {
                let heal = (damage as f32 * strike.lifesteal).round() as i32;
                if heal > 0 {
                    lifesteal_heals.push((strike.monster_id, heal));
                }
            }
            if victim.hp <= 0 {
                victim.hp = 0;
                victim.alive = false;
                party.casualties += 1;
                adventurer_kills.push((victim.name.clone(), victim.level));
                leeched_mana += strike.mana_on_kill;
            }
        }
    }

    // Apply lifesteal heals now that the party borrow has ended.
    if !lifesteal_heals.is_empty() {
        let room = &mut state.floors[floor_idx].rooms[room_idx];
        for (monster_id, heal) in lifesteal_heals {
            if let Some(monster) = room
                .monsters
                .iter_mut()
                .find(|m| m.id == monster_id && m.alive)
            {
                // Undead identity: the dead cannot mend, even by draining life.
                if !crate::data::monsters::is_undead(&monster.type_name) {
                    monster.hp = (monster.hp + heal).min(monster.max_hp);
                }
            }
        }
    }
    if leeched_mana > 0 {
        state.mana = (state.mana + leeched_mana).min(state.max_mana);
        state.add_log(LogEntry::combat(format!(
            "Mana Leech drains +{} mana from the fallen.",
            leeched_mana
        )));
    }

    if damage_to_party > 0 && adventurer_kills.is_empty() {
        // The party is the one taking these hits — float it over their side.
        state.push_effect_at(
            floor_num,
            room_pos,
            format!(
                "-{}{}",
                damage_to_party,
                if monster_hit_strong { "!" } else { "" }
            ),
            EffectKind::Damage,
            EffectAnchor::Invaders,
        );
        let impact_element = monster_strikes
            .first()
            .map(|strike| strike.element.as_str())
            .unwrap_or_default();
        state.push_element_effect_at(
            floor_num,
            room_pos,
            "",
            EffectKind::HitSpark,
            EffectAnchor::Invaders,
            impact_element,
        );
        if let Some(element) = ElementSound::from_id(impact_element) {
            state.queue_sound(SoundEvent::ElementalHit(element));
        }
        if hero_ward_triggered {
            state.push_effect_at(
                floor_num,
                room_pos,
                "Ward!",
                EffectKind::Ability,
                EffectAnchor::Invaders,
            );
        }
    }

    reward_monster_kills(state, party_idx, floor_idx, room_idx, &monster_kills);
    reward_adventurer_kills(state, party_idx, floor_idx, room_idx, &adventurer_kills);
    check_retreat(state, party_idx);
}

/// One monster's pending attack for the strike-back phase.
struct MonsterStrike {
    monster_id: u64,
    attack: i32,
    element: String,
    pierce: bool,
    lifesteal: f32,
    mana_on_kill: i32,
}

/// Casualties a party will accept before retreating. Cautious races
/// (Halflings) bail early; brave ones (Dwarves, Paladins) hold longer.
fn party_nerve(party: &crate::game_state::AdventurerParty) -> i32 {
    let mut threshold = RETREAT_THRESHOLD;
    let living: Vec<_> = party.members.iter().filter(|a| a.alive).collect();
    for member in &living {
        match member.race.as_str() {
            "Halfling" => threshold -= 1,
            "Dwarf" => threshold += 1,
            _ => {}
        }
        if member.class_name == "Paladin" {
            threshold += 1;
        }
    }
    if living
        .iter()
        .any(|member| member.drive == crate::game_state::HeroDrive::Glory)
    {
        threshold += 1;
    }
    if living
        .iter()
        .any(|member| member.drive == crate::game_state::HeroDrive::Duty)
    {
        threshold += 1;
    }
    let average_resolve = if living.is_empty() {
        50
    } else {
        living.iter().map(|member| member.resolve).sum::<i32>() / living.len() as i32
    };
    threshold += match average_resolve {
        70.. => 1,
        ..=35 => -1,
        _ => 0,
    };
    threshold.clamp(1, 5)
}

/// Flag the party as retreating after heavy losses or a full wipe.
fn check_retreat(state: &mut GameState, party_idx: usize) {
    // Dread core powers unnerve invaders, breaking them sooner (stacks). Siege
    // parties are fanatics and never break early.
    let dread = crate::simulation::endgame::core_dread_bonus(state);
    let doctrine =
        crate::game_state::doctrine_for_party(state, &state.adventurer_parties[party_idx]);
    let party = &mut state.adventurer_parties[party_idx];
    if party.retreating {
        return;
    }
    let no_survivors = party.members.iter().all(|a| !a.alive);
    let nerve = if party.sieging {
        99
    } else {
        let doctrine_nerve = matches!(
            doctrine,
            crate::game_state::ExpeditionDoctrine::Vengeance
                | crate::game_state::ExpeditionDoctrine::RelicHunt
        ) as i32;
        (party_nerve(party) + doctrine_nerve - dread).max(1)
    };
    if no_survivors {
        party.retreating = true;
        state.add_log(LogEntry::adventure("The entire party has been wiped out!"));
    } else if party.casualties >= nerve {
        party.retreating = true;
        state.add_log(LogEntry::adventure(
            "Party is retreating due to heavy casualties!",
        ));
    }
}

#[cfg(test)]
mod tests;
