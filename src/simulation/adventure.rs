use macroquad_toolkit::timing::Cooldown;

use crate::data::adventurers::{
    get_adventurer_class, get_adventurer_classes, get_adventurer_names, get_entry_quotes,
    get_exit_quotes,
};
use crate::data::constants::{ADVENTURER_SPAWN_CHANCE, MAX_PARTY_SIZE, MIN_PARTY_SIZE};
use crate::game_state::{
    Adventurer, AdventurerParty, DungeonStatus, GameState, HeroDrive, HeroRecord, HeroStatus,
    LogEntry, Stats, PARTY_MOVE_SECONDS,
};

/// Build a combat-ready adventurer from a class, level, and identity. `stat_mult`
/// scales the final HP/attack/defense by the run's difficulty.
fn build_adventurer(
    id: u64,
    name: String,
    class_name: &str,
    race: &str,
    drive: HeroDrive,
    resolve: i32,
    level: i32,
    stat_mult: f32,
) -> Adventurer {
    let class = get_adventurer_class(class_name)
        .unwrap_or_else(|| get_adventurer_classes().into_iter().next().unwrap());
    let race_mod = crate::data::adventurers::get_race(race).unwrap_or_default();

    let base_hp = class.hp + (level - 1) * 10 + race_mod.hp;
    let equipment = crate::data::equipment::recommended_loadout(&class.name, level);
    let equipment_bonus = crate::data::equipment::equipment_stat_bonus(&equipment, &class.name);
    let raw_hp = (base_hp + equipment_bonus.hp).max(1);
    let raw_attack =
        (class.attack + (level - 1) * 2 + equipment_bonus.attack + race_mod.attack).max(1);
    let raw_defense =
        (class.defense + (level - 1) + equipment_bonus.defense + race_mod.defense).max(0);

    let hp = ((raw_hp as f32 * stat_mult).round() as i32).max(1);
    let attack = ((raw_attack as f32 * stat_mult).round() as i32).max(1);
    let defense = ((raw_defense as f32 * stat_mult).round() as i32).max(0);

    Adventurer {
        id,
        name,
        class_name: class.name.clone(),
        race: race.to_string(),
        drive,
        resolve,
        level,
        hp,
        max_hp: hp,
        alive: true,
        experience: 0,
        gold: 0,
        equipment,
        conditions: Vec::new(),
        scaled_stats: Stats {
            hp,
            attack,
            defense,
        },
    }
}

/// Party size and adventurer level range for the current threat/floor state.
/// Low threat sends larger bands of weaker heroes; high threat sends smaller,
/// far more dangerous elites.
fn threat_party_shape(state: &mut GameState) -> (usize, i32, i32) {
    let tier = state.threat_tier();
    let deepest = state.total_floors.max(1);
    let level_min = 1 + tier;
    let level_max = (3 + tier + deepest / 2).max(level_min);
    let size = match tier {
        0 => state
            .run_rng
            .range_i32(MIN_PARTY_SIZE as i32, (MAX_PARTY_SIZE + 1) as i32) as usize,
        1 | 2 => state
            .run_rng
            .range_i32(MIN_PARTY_SIZE as i32, MAX_PARTY_SIZE as i32) as usize,
        _ => MIN_PARTY_SIZE,
    };
    (size.max(1), level_min, level_max)
}

/// How far this expedition intends to delve. Strong heroes, returning
/// veterans, and a frightened realm all push deeper; a fresh low-level party
/// still tests only the upper dungeon. This replaces the old hard floor-2 cap,
/// allowing ordinary raids to interact with the whole built dungeon.
fn expedition_target_floor(state: &GameState, members: &[Adventurer]) -> i32 {
    let available = state.total_floors.max(1);
    let strongest = members.iter().map(|hero| hero.level).max().unwrap_or(1);
    let veteran_delves = members
        .iter()
        .filter_map(|member| {
            state
                .known_adventurers
                .iter()
                .find(|record| record.id == member.id)
                .map(|record| record.delves)
        })
        .max()
        .unwrap_or(1);
    let strength_depth = (strongest - 1).max(0) / 2;
    let veteran_depth = (veteran_delves - 1).max(0) / 2;
    let realm_pressure = state.threat_tier();
    let prestige_pressure = state.prestige.min(2);
    let discovery_depth = members
        .iter()
        .any(|member| member.drive == HeroDrive::Discovery) as i32;

    (2 + strength_depth + veteran_depth + realm_pressure + prestige_pressure + discovery_depth)
        .clamp(1, available)
}

/// Try to spawn a new adventurer party
pub fn spawn_party(state: &mut GameState) {
    // Only spawn when open and no parties present
    if state.status != DungeonStatus::Open {
        return;
    }
    if !state.adventurer_parties.is_empty() {
        return;
    }
    // Compare in absolute hours — hour-of-day comparisons broke at the day
    // wrap (a party spawned at hour 23 set next_party_spawn to 24, which
    // hour-of-day never reaches, so spawns stopped after day 1).
    let now_abs = state.day * 24 + state.hour;
    if now_abs < state.next_party_spawn {
        return;
    }

    let visitor_quality = state.visitor_quality();
    let spawn_chance = ADVENTURER_SPAWN_CHANCE
        * state.difficulty.profile().spawn_chance_mult
        * visitor_quality.spawn_chance_mult;
    if !state.run_rng.chance(spawn_chance) {
        return;
    }

    let stat_mult = state.difficulty.profile().invader_stat_mult;

    let names = get_adventurer_names();
    let entry_quotes = get_entry_quotes();
    let races = crate::data::adventurers::get_race_names();

    // Higher threat means fewer but stronger parties (see threat_party_shape).
    let (party_size, level_min, level_max) = threat_party_shape(state);
    let level_min = (level_min + visitor_quality.level_bonus).max(1);
    let level_max = (level_max + visitor_quality.level_bonus).max(level_min);
    let mut members = Vec::with_capacity(party_size);

    // Some slots are filled by veterans returning for another delve.
    let mut returning: Vec<u64> = state
        .known_adventurers
        .iter()
        .filter(|h| h.status == HeroStatus::Alive)
        .map(|h| h.id)
        .collect();
    state.run_rng.shuffle(&mut returning);

    for slot in 0..party_size {
        // Reputation controls how strongly a party prefers known veterans.
        let use_veteran =
            slot % visitor_quality.returning_slot_stride == 0 && !returning.is_empty();
        if use_veteran {
            let hero_id = returning.pop().unwrap();
            let day = state.day;
            if let Some(record) = state.hero_mut(hero_id) {
                record.status = HeroStatus::Inside;
                record.delves += 1;
                let delve = record.delves;
                record.remember(day, format!("Returned for delve {delve}"));
                let (name, class, race, drive, resolve, level) = (
                    record.name.clone(),
                    record.class_name.clone(),
                    record.race.clone(),
                    record.drive,
                    record.resolve,
                    record.level,
                );
                members.push(build_adventurer(
                    hero_id, name, &class, &race, drive, resolve, level, stat_mult,
                ));
                continue;
            }
        }

        // Fresh recruit: roll identity and register a new ledger entry.
        let classes = get_adventurer_classes();
        let class = classes[state.run_rng.below(classes.len())].clone();
        let name = names[state.run_rng.below(names.len())].clone();
        let race = races
            .get(state.run_rng.below(races.len()))
            .cloned()
            .unwrap_or_else(|| "Human".to_string());
        let level = state.run_rng.range_i32(level_min, level_max + 1);
        let id = state.run_rng.next_u64();
        let drive = HeroDrive::ALL[state.run_rng.below(HeroDrive::ALL.len())];
        let resolve = 50;
        state.known_adventurers.push(HeroRecord {
            id,
            name: name.clone(),
            class_name: class.name.clone(),
            race: race.clone(),
            drive,
            resolve,
            level,
            experience: 0,
            delves: 1,
            kills: 0,
            gold_stolen: 0,
            escapes: 0,
            deepest_floor: 0,
            status: HeroStatus::Inside,
            death_floor: 0,
            death_day: 0,
            journal: Vec::new(),
        });
        let day = state.day;
        if let Some(record) = state.hero_mut(id) {
            record.remember(
                day,
                format!("First delve, driven by {}", drive.label().to_lowercase()),
            );
        }
        members.push(build_adventurer(
            id,
            name,
            &class.name,
            &race,
            drive,
            resolve,
            level,
            stat_mult,
        ));
    }

    let target_floor = expedition_target_floor(state, &members);

    let party = AdventurerParty {
        id: state.run_rng.next_u64(),
        members,
        current_floor: 1,
        current_room: 0,
        retreating: false,
        casualties: 0,
        loot: 0,
        entry_time: state.hour,
        target_floor,
        snared_ticks: 0,
        alarmed: false,
        sieging: false,
        prev_room: 0,
        move_anim: Cooldown::new(PARTY_MOVE_SECONDS),
    };

    // Fresh raid: clear the prior summary card and start a new income tally.
    state.last_raid_summary = None;
    state.current_raid = Some(Default::default());

    state.add_log(LogEntry::adventure(format!(
        "{} visitors enter: {} members, levels {}–{}, expedition target floor {}.",
        state.reputation_band().name(),
        party.members.len(),
        level_min,
        level_max,
        party.target_floor
    )));

    // Random entry quote
    if state.run_rng.chance(0.3) && !entry_quotes.is_empty() {
        let quote = &entry_quotes[state.run_rng.below(entry_quotes.len())];
        let name = &party.members[0].name;
        state.add_log(LogEntry::adventure(format!("{} says: \"{}\"", name, quote)));
    }

    state.adventurer_parties.push(party);
    state.next_party_spawn = now_abs + 1;
}

/// Process all adventurer parties
pub fn process_parties(state: &mut GameState) {
    if state.adventurer_parties.is_empty() {
        return;
    }

    // Collect party IDs to process
    let party_ids: Vec<u64> = state.adventurer_parties.iter().map(|p| p.id).collect();

    for party_id in party_ids {
        process_single_party(state, party_id);
    }

    // Handle retreating parties
    handle_retreating_parties(state);
}

fn process_single_party(state: &mut GameState, party_id: u64) {
    let party_idx = match state
        .adventurer_parties
        .iter()
        .position(|p| p.id == party_id)
    {
        Some(idx) => idx,
        None => return,
    };

    // Skip retreating parties
    if state.adventurer_parties[party_idx].retreating {
        return;
    }

    let current_floor = state.adventurer_parties[party_idx].current_floor;
    let current_room = state.adventurer_parties[party_idx].current_room;

    // Find floor and room
    let floor_idx = match state.floors.iter().position(|f| f.number == current_floor) {
        Some(idx) => idx,
        None => return,
    };

    let room_idx = match state.floors[floor_idx]
        .rooms
        .iter()
        .position(|r| r.position == current_room)
    {
        Some(idx) => idx,
        None => return,
    };

    mark_room_explored(state, floor_idx, room_idx);

    // Check for combat
    let has_alive_monsters = state.floors[floor_idx].rooms[room_idx]
        .monsters
        .iter()
        .any(|m| m.alive);

    if has_alive_monsters {
        // Combat happens in combat module
        super::combat::resolve_combat(state, party_idx, floor_idx, room_idx);
    } else {
        // Undefended room: its trap still springs on the passing party (so a
        // snare/alarm room placed before a killbox carries into it), then the
        // party moves on.
        super::combat::spring_undefended_trap(state, party_idx, floor_idx, room_idx);
        advance_party(state, party_idx);
    }
}

fn mark_room_explored(state: &mut GameState, floor_idx: usize, room_idx: usize) {
    if let Some(room) = state
        .floors
        .get_mut(floor_idx)
        .and_then(|floor| floor.rooms.get_mut(room_idx))
    {
        room.explored = true;
    }
}

fn advance_party(state: &mut GameState, party_idx: usize) {
    let party = &state.adventurer_parties[party_idx];
    let current_floor = party.current_floor;
    let current_room = party.current_room;
    let target_floor = party.target_floor;

    // Find current floor
    let floor = match state.floors.iter().find(|f| f.number == current_floor) {
        Some(f) => f,
        None => return,
    };

    // Follow the room's graph edges. No exits ⇒ this is the Core sink ⇒ end of
    // floor (descend / retreat / siege-assault). At a fork, `choose_exit` picks
    // the branch (greedy for loot / shy of threat, or beelining the Core when
    // the realm is desperate); linear floors have a single exit.
    let exits: Vec<usize> = floor
        .room_at(current_room)
        .map(|room| room.exits.clone())
        .unwrap_or_default();

    if exits.is_empty() {
        // No exits ⇒ the Core sink ⇒ end of floor.

        // A siege party at the bottom assaults the core itself.
        if state.adventurer_parties[party_idx].sieging && current_floor >= target_floor {
            let party_spent = super::endgame::assault_core(state, party_idx);
            // Repel only if the core survived; if it fell, the run is over.
            if party_spent && !state.game_over {
                super::endgame::repel_siege(state);
            }
            return;
        }

        if current_floor < target_floor && current_floor < state.floors.len() as i32 {
            // Descend to next floor
            state.adventurer_parties[party_idx].current_floor += 1;
            state.adventurer_parties[party_idx].current_room = 0;
            state.add_log(LogEntry::adventure(format!(
                "Party descends to floor {}",
                current_floor + 1
            )));
        } else {
            // Completed exploration, retreat with loot
            let loot = state.adventurer_parties[party_idx].loot;
            state.gold += loot;
            state.raid_tally().gold_gained += loot;
            state.adventurer_parties[party_idx].retreating = true;
            state.add_log(LogEntry::adventure(format!(
                "Party completed exploration! +{} gold",
                loot
            )));

            // Exit quote
            let exit_quotes = get_exit_quotes();
            if state.run_rng.chance(0.4) && !exit_quotes.is_empty() {
                let quote = &exit_quotes[state.run_rng.below(exit_quotes.len())];
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
        return;
    }

    // Advance along the chosen edge, kicking off the corridor-travel animation
    // so the party visibly walks from its old room to the new one.
    let party = &state.adventurer_parties[party_idx];
    let next = super::pathing::choose_exit(state, floor, party, &exits);
    let reason = super::pathing::choice_reason(state, party, &exits);
    state.adventurer_parties[party_idx].prev_room = current_room;
    state.adventurer_parties[party_idx].move_anim.trigger();
    state.adventurer_parties[party_idx].current_room = next;
    state.add_log(LogEntry::adventure(format!(
        "Party chooses room {} on floor {} — {}.",
        next, current_floor, reason
    )));
}

fn handle_retreating_parties(state: &mut GameState) {
    // Settle the ledger for every departing party before it is removed:
    // survivors bank XP and gold and level up; the fallen are entombed.
    let departing: Vec<usize> = state
        .adventurer_parties
        .iter()
        .enumerate()
        .filter(|(_, p)| p.retreating)
        .map(|(i, _)| i)
        .collect();
    for idx in departing {
        settle_departing_party(state, idx);
    }

    let before = state.adventurer_parties.len();
    state.adventurer_parties.retain(|party| !party.retreating);
    let departed = before - state.adventurer_parties.len();
    if departed > 0 {
        // Every party that leaves (looted out or wiped/retreated) is a raid the
        // dungeon has weathered.
        state.raids_completed += departed as i32;
    }

    // Respawn monsters and re-arm sprung traps once the dungeon is clear
    if state.adventurer_parties.is_empty() {
        let recovery_cost =
            super::monsters::respawn_monsters(state) + super::combat::rearm_traps(state);
        if let Some(summary) = &mut state.last_raid_summary {
            summary.mana_recovery_cost = recovery_cost;
        }
    }
}

/// Update the hero ledger for a party that is leaving the dungeon.
fn settle_departing_party(state: &mut GameState, party_idx: usize) {
    let party_floor = state.adventurer_parties[party_idx].current_floor;
    let survivors: Vec<u64> = state.adventurer_parties[party_idx]
        .members
        .iter()
        .filter(|m| m.alive)
        .map(|m| m.id)
        .collect();
    let survivor_count = survivors.len().max(1) as i32;
    let loot_share = state.adventurer_parties[party_idx].loot / survivor_count;

    let member_ids: Vec<(u64, bool, i32, i32)> = state.adventurer_parties[party_idx]
        .members
        .iter()
        .map(|m| (m.id, m.alive, m.hp, m.max_hp))
        .collect();

    // Snapshot a summary card of the raid the dungeon just weathered.
    let party_size = member_ids.len() as i32;
    let survivors = member_ids.iter().filter(|(_, alive, _, _)| *alive).count() as i32;
    let slain = party_size - survivors;
    let returning_survivors = member_ids
        .iter()
        .filter(|(id, alive, _, _)| {
            *alive
                && state
                    .known_adventurers
                    .iter()
                    .find(|hero| hero.id == *id)
                    .is_some_and(|hero| hero.delves > 1)
        })
        .count() as i32;
    let raid_loot = state.adventurer_parties[party_idx].loot;
    let reputation_change =
        state.apply_raid_reputation(party_floor, survivors, raid_loot, returning_survivors);
    let reputation_after = state.reputation;
    let tally = state.current_raid.take().unwrap_or_default();
    state.last_raid_summary = Some(crate::game_state::RaidSummary {
        outcome: if survivors == 0 {
            crate::game_state::RaidOutcome::Wiped
        } else {
            crate::game_state::RaidOutcome::Repelled
        },
        party_size,
        slain,
        survivors,
        mana_gained: tally.mana_gained,
        mana_recovery_cost: 0,
        souls_gained: tally.souls_gained,
        gold_gained: tally.gold_gained,
        defenders_lost: tally.defenders_lost,
        reputation_change,
        reputation_after,
    });
    state.add_log(LogEntry::adventure(format!(
        "Reputation {}{} → {} ({}) — {}.",
        if reputation_change >= 0 { "+" } else { "" },
        reputation_change,
        reputation_after,
        state.reputation_band().name(),
        if survivors == 0 {
            "a wipe makes the dungeon look like a shallow deathtrap"
        } else {
            "survivors carry its tale beyond the depths"
        }
    )));

    for (id, alive, hp, max_hp) in member_ids {
        if alive {
            // Escaped: bank XP, gold, and possibly a level.
            let day = state.day;
            if let Some(record) = state.hero_mut(id) {
                record.status = HeroStatus::Alive;
                record.escapes += 1;
                record.deepest_floor = record.deepest_floor.max(party_floor);
                // Deep expeditions are the main route to veteran growth. A
                // hero who repeatedly skims floor 1 improves much more slowly
                // than one who survives the lower strata.
                let xp_gain = 15 + party_floor * 8 + record.delves * 3;
                let xp_gain = if record.drive == HeroDrive::Discovery {
                    (xp_gain as f32 * 1.25).round() as i32
                } else {
                    xp_gain
                };
                record.experience += xp_gain;
                record.gold_stolen += loot_share;
                let level_before = record.level;
                while record.level < 10
                    && record.experience >= GameState::xp_for_level(record.level)
                {
                    record.experience -= GameState::xp_for_level(record.level);
                    record.level += 1;
                }
                let escaped = if loot_share > 0 {
                    format!("Escaped floor {party_floor} with {loot_share} gold (+{xp_gain} XP)")
                } else {
                    format!("Escaped floor {party_floor} empty-handed (+{xp_gain} XP)")
                };
                record.remember(day, escaped);
                if hp * 4 <= max_hp.max(1) {
                    record.resolve = (record.resolve - 8).max(20);
                    record.remember(day, "Survived with grievous wounds");
                } else {
                    record.resolve = (record.resolve + 5).min(100);
                }
                if record.level > level_before {
                    let level = record.level;
                    record.remember(day, format!("Reached level {level}"));
                }
            }
        } else {
            state.record_hero_death(id, party_floor);
        }
    }
}

#[cfg(test)]
mod tests;
