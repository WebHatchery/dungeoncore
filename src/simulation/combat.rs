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
use crate::game_state::{EffectAnchor, EffectKind, GameState, LogEntry};

use abilities::{resolve_abilities, tick_conditions};
use helpers::{
    adventurer_element, attunement_mult, has_passive, monster_attack_value,
    monster_damage_taken_mult, monster_element, passive_value, split_spawn, target_monster_idx,
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
    }

    let reinforcement_mult = state.floors[floor_idx].rooms[room_idx].reinforcement_multiplier();

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
    let adv_attacks: Vec<(u64, i32, String)> = if snared {
        Vec::new()
    } else {
        state.adventurer_parties[party_idx]
            .members
            .iter()
            .filter(|a| a.alive)
            .map(|a| {
                (
                    a.id,
                    a.scaled_stats.attack,
                    adventurer_element(&a.class_name),
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
        for (attacker_id, attack, adv_element) in &adv_attacks {
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
            let damage = ((*attack as f32 - effective_def / 2.0).max(1.0) * taken_mult * elem_mult)
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
        state.push_effect_at(
            floor_num,
            room_pos,
            format!("-{}", damage_to_monsters),
            EffectKind::Damage,
            EffectAnchor::Defenders,
        );
        state.push_effect_at(
            floor_num,
            room_pos,
            "",
            EffectKind::HitSpark,
            EffectAnchor::Defenders,
        );
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
                MonsterStrike {
                    monster_id: m.id,
                    attack: monster_attack_value(
                        m,
                        alive_count,
                        enemies_alive,
                        reinforcement_mult * attune_mult * alarm_mult,
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
    let mut lifesteal_heals: Vec<(u64, i32)> = Vec::new();
    let mut leeched_mana = 0;
    {
        let party = &mut state.adventurer_parties[party_idx];
        for strike in monster_strikes {
            let alive_idxs: Vec<usize> = party
                .members
                .iter()
                .enumerate()
                .filter(|(_, a)| a.alive)
                .map(|(i, _)| i)
                .collect();
            if alive_idxs.is_empty() {
                break;
            }
            let victim_idx = alive_idxs[state.run_rng.below(alive_idxs.len())];
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
            let damage = ((strike.attack as f32 - victim_def).max(1.0) * elem_mult * vuln)
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
        state.push_effect_at(
            floor_num,
            room_pos,
            "",
            EffectKind::HitSpark,
            EffectAnchor::Invaders,
        );
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
    let living = party.members.iter().filter(|a| a.alive);
    for member in living {
        match member.race.as_str() {
            "Halfling" => threshold -= 1,
            "Dwarf" => threshold += 1,
            _ => {}
        }
        if member.class_name == "Paladin" {
            threshold += 1;
        }
    }
    threshold.clamp(1, 5)
}

/// Flag the party as retreating after heavy losses or a full wipe.
fn check_retreat(state: &mut GameState, party_idx: usize) {
    // Dread core powers unnerve invaders, breaking them sooner (stacks). Siege
    // parties are fanatics and never break early.
    let dread = crate::simulation::endgame::core_dread_bonus(state);
    let party = &mut state.adventurer_parties[party_idx];
    if party.retreating {
        return;
    }
    let no_survivors = party.members.iter().all(|a| !a.alive);
    let nerve = if party.sieging {
        99
    } else {
        (party_nerve(party) - dread).max(1)
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
mod tests {
    use super::*;
    use crate::game_state::{Adventurer, AdventurerParty, Monster, Room, RoomType, Stats};

    fn sturdy_monster() -> Monster {
        Monster {
            id: 1,
            type_name: "Goblin".to_string(),
            hp: 500,
            max_hp: 500,
            alive: true,
            is_boss: false,
            scaled_stats: Stats {
                hp: 500,
                attack: 30,
                defense: 0,
            },
            active_traits: Vec::new(),
        }
    }

    fn lone_invader(hp: i32) -> AdventurerParty {
        AdventurerParty {
            id: 1,
            members: vec![Adventurer {
                id: 10,
                name: "Tess".to_string(),
                class_name: "Warrior".to_string(),
                race: "Human".to_string(),
                level: 3,
                hp,
                max_hp: hp,
                alive: true,
                experience: 0,
                gold: 0,
                equipment: Default::default(),
                conditions: Vec::new(),
                scaled_stats: Stats {
                    hp,
                    attack: 5,
                    defense: 0,
                },
            }],
            current_floor: 1,
            current_room: 2,
            retreating: false,
            casualties: 0,
            loot: 0,
            entry_time: 0,
            target_floor: 1,
            snared_ticks: 0,
            alarmed: false,
            sieging: false,
            prev_room: 0,
            move_anim: macroquad_toolkit::timing::Cooldown::new(
                crate::game_state::PARTY_MOVE_SECONDS,
            ),
        }
    }

    /// One combat tick against the same sturdy monster deals more damage to a
    /// held (snared) party than a free one — the spatial "snare before killbox"
    /// combo. Returns HP lost by the invader.
    fn damage_taken(snared: bool) -> i32 {
        let mut s = GameState::new();
        let mut room = Room::new(99, RoomType::Normal, 2, 1);
        room.monsters.push(sturdy_monster());
        s.floors[0].rooms.push(room);
        let room_idx = s.floors[0].rooms.len() - 1;
        s.adventurer_parties.push(lone_invader(300));
        if snared {
            s.adventurer_parties[0].snared_ticks = 3;
        }
        let before = s.adventurer_parties[0].members[0].hp;
        resolve_combat(&mut s, 0, 0, room_idx);
        before - s.adventurer_parties[0].members[0].hp
    }

    #[test]
    fn snared_party_takes_amplified_damage() {
        let free = damage_taken(false);
        let held = damage_taken(true);
        assert!(free > 0, "monster should hit the free party");
        assert!(
            held > free,
            "a held party takes more ({held}) than a free one ({free})"
        );
    }
}
