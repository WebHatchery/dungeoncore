use crate::data::constants::get_monster_mana_cost;
use crate::data::monsters::get_monster_template;
use crate::game_state::{GameState, LogEntry, Monster, RoomType, Stats};
use std::collections::BTreeMap;

mod swap;
pub use swap::{plan_swap, swap_monster, SwapKind};

/// Why an armed monster cannot take a free room slot. Boss rooms reserve their
/// final ordinary slot for one boss-only defender until that throne is filled.
pub fn monster_placement_refusal(
    room: &crate::game_state::Room,
    monster_name: &str,
) -> Option<&'static str> {
    let template = get_monster_template(monster_name)?;
    let is_boss_room = room.room_type == RoomType::Boss;
    if template.boss_only && !is_boss_room {
        return Some("Boss only");
    }
    let capacity = crate::data::constants::room_capacity(room);
    let has_boss = room.monsters.iter().any(|monster| {
        get_monster_template(&monster.type_name)
            .map(|occupant| occupant.boss_only)
            .unwrap_or(false)
    });
    if template.boss_only && has_boss {
        return Some("Boss set");
    }
    if is_boss_room
        && !template.boss_only
        && !has_boss
        && room.monsters.len() >= capacity.saturating_sub(1)
    {
        return Some("Reserved");
    }
    (room.monsters.len() >= capacity).then_some("Full")
}

/// Place a monster in a room
pub fn place_monster(
    state: &mut GameState,
    floor_num: i32,
    room_pos: usize,
    monster_name: &str,
) -> Result<(), String> {
    // Find monster template
    let template = get_monster_template(monster_name)
        .ok_or_else(|| format!("Unknown monster: {}", monster_name))?;

    // Check if species is unlocked
    if !state.unlocked_species.contains(&template.species) {
        return Err(format!("Species '{}' is not unlocked!", template.species));
    }

    // Check if this specific monster type is unlocked
    if !state.unlocked_monsters.contains(&template.name) {
        return Err(format!(
            "Monster '{}' is not unlocked! Evolve to unlock higher tiers.",
            template.name
        ));
    }

    // Find floor and room
    let floor = state
        .floors
        .iter_mut()
        .find(|f| f.number == floor_num)
        .ok_or("Floor not found")?;

    let room = floor
        .rooms
        .iter_mut()
        .find(|r| r.position == room_pos)
        .ok_or("Room not found")?;

    // Cannot place in entrance or core
    if room.room_type == RoomType::Entrance || room.room_type == RoomType::Core {
        return Err("Cannot place monsters in entrance or core rooms!".into());
    }

    let is_boss = room.room_type == RoomType::Boss;
    if template.boss_only && !is_boss {
        return Err(format!(
            "{} can only be summoned in a Boss room!",
            template.name
        ));
    }

    // Slots are scarce and a boss room keeps its final ordinary slot vacant
    // until a boss-only defender claims the throne.
    if let Some(refusal) = monster_placement_refusal(room, monster_name) {
        return Err(format!(
            "This room cannot take {} ({}; {} of {} slots).",
            monster_name,
            refusal.to_lowercase(),
            room.monsters.len(),
            crate::data::constants::room_capacity(room)
        ));
    }

    // Boss uniques already price in their throne room — no 2x boss surcharge.
    let boss_surcharge = is_boss && !template.boss_only;
    let cost = get_monster_mana_cost(template.base_cost, floor_num, boss_surcharge);

    if state.mana < cost {
        return Err(format!("Not enough mana! Need {} mana.", cost));
    }
    if state.souls < template.souls_cost {
        return Err(format!(
            "Not enough souls! Need {} souls.",
            template.souls_cost
        ));
    }

    state.mana -= cost;
    state.souls -= template.souls_cost;

    // Scale stats based on floor and boss status
    let base_stats = Stats {
        hp: template.hp,
        attack: template.attack,
        defense: template.defense,
    };
    let scaled = crate::data::get_scaled_stats(base_stats, floor_num, is_boss);

    // Initialize traits
    let active_traits = template
        .traits
        .iter()
        .map(|trait_id| {
            // Look up trait name (optional, but good for display without full lookup)
            // For now we just store ID and initial cooldown 0
            crate::game_state::ActiveTrait {
                id: trait_id.clone(),
                name: crate::data::traits::get_trait(trait_id)
                    .map(|t| t.name)
                    .unwrap_or_else(|| trait_id.clone()),
                cooldown_timer: 0,
            }
        })
        .collect();

    let monster = Monster {
        id: state.run_rng.next_u64(),
        type_name: monster_name.into(),
        hp: scaled.hp,
        max_hp: scaled.hp,
        alive: true,
        is_boss,
        scaled_stats: scaled,
        active_traits,
    };

    room.monsters.push(monster);

    let boss_suffix = if is_boss { " (Boss)" } else { "" };
    state.add_log(LogEntry::building(format!(
        "Spawned {}{} on floor {}, room {} for {} mana.",
        monster_name, boss_suffix, floor_num, room_pos, cost
    )));

    Ok(())
}

/// Dismiss a placed monster, refunding half its summon mana.
pub fn remove_monster(
    state: &mut GameState,
    floor_num: i32,
    room_pos: usize,
    monster_id: u64,
) -> Result<(), String> {
    if !state.adventurer_parties.is_empty() {
        return Err("Cannot dismiss monsters while adventurers are in the dungeon!".into());
    }

    let floor = state
        .floors
        .iter_mut()
        .find(|f| f.number == floor_num)
        .ok_or("Floor not found")?;
    let room = floor
        .rooms
        .iter_mut()
        .find(|r| r.position == room_pos)
        .ok_or("Room not found")?;

    let idx = room
        .monsters
        .iter()
        .position(|m| m.id == monster_id)
        .ok_or("Monster not found in this room")?;
    let monster = room.monsters.remove(idx);

    // Refund half of what the summon cost at this floor/room (souls are spent
    // essence and stay spent).
    let refund = get_monster_template(&monster.type_name)
        .map(|template| {
            let boss_surcharge = room.room_type == RoomType::Boss && !template.boss_only;
            get_monster_mana_cost(template.base_cost, floor_num, boss_surcharge) / 2
        })
        .unwrap_or(0);
    state.mana = (state.mana + refund).min(state.max_mana);

    state.add_log(LogEntry::building(format!(
        "Dismissed {} from floor {}, room {}. Refunded {} mana.",
        monster.type_name, floor_num, room_pos, refund
    )));

    Ok(())
}

/// Respawn all dead monsters (only when no adventurers present)
pub fn respawn_monsters(state: &mut GameState) -> i32 {
    if !state.adventurer_parties.is_empty() {
        return 0;
    }

    // Undead identity: the undead rise again for free and whole; the living must
    // be reknit with mana (half their summon cost). If mana runs short a living
    // defender still crawls back, but wounded (half HP) — so a poor dungeon
    // degrades gracefully rather than losing its garrison outright.
    let mut mana = state.mana;
    let mut free = 0;
    let mut paid = 0;
    let mut wounded = 0;
    for floor in &mut state.floors {
        for room in &mut floor.rooms {
            for monster in &mut room.monsters {
                if monster.alive {
                    continue;
                }
                if crate::data::monsters::is_undead(&monster.type_name) {
                    monster.hp = monster.max_hp;
                    monster.alive = true;
                    free += 1;
                    continue;
                }
                let cost = crate::data::monsters::respawn_mana_cost(&monster.type_name);
                if mana >= cost {
                    mana -= cost;
                    monster.hp = monster.max_hp;
                    monster.alive = true;
                    paid += 1;
                } else {
                    monster.hp = (monster.max_hp / 2).max(1);
                    monster.alive = true;
                    wounded += 1;
                }
            }
        }
    }
    let spent = state.mana - mana;
    state.mana = mana;

    let total = free + paid + wounded;
    if total > 0 {
        let mut msg = format!("Defenders respawn ({total}):");
        if free > 0 {
            msg.push_str(&format!(" {free} undead rose free;"));
        }
        if paid > 0 {
            msg.push_str(&format!(" {paid} reknit with mana;"));
        }
        if wounded > 0 {
            msg.push_str(&format!(" {wounded} crawled back wounded (low mana);"));
        }
        msg.pop(); // trailing ';'
        msg.push('.');
        state.add_log(LogEntry::system(msg));
    }
    spent
}

/// Unlock a monster species
pub fn unlock_species(state: &mut GameState, species_name: &str) -> Result<(), String> {
    if state.unlocked_species.contains(&species_name.to_string()) {
        return Err(format!("Species '{}' is already unlocked!", species_name));
    }

    // Get unlock cost from JSON data. Starter races are free only for the first pick.
    let species_data = crate::data::monsters::get_species(species_name)
        .ok_or_else(|| format!("Unknown species: {}", species_name))?;
    let is_first_species = state.unlocked_species.is_empty();
    let unlock_cost = if is_first_species && species_data.starter {
        0
    } else {
        species_data.unlock_cost
    };

    if unlock_cost == 0 {
        // Free unlock - still unlock the starting monster
    } else {
        if state.gold < unlock_cost {
            return Err(format!("Not enough gold! Need {} gold.", unlock_cost));
        }
        state.gold -= unlock_cost;
    }

    state.unlocked_species.push(species_name.to_string());

    let mut unlocked_now = Vec::new();
    for template in crate::data::monsters::get_starter_monsters_for_species(species_name) {
        if !state.unlocked_monsters.contains(&template.name) {
            state.unlocked_monsters.push(template.name.clone());
            unlocked_now.push(template.name);
        }
    }

    if unlocked_now.is_empty() {
        if let Some(starting_monster) =
            crate::data::evolutions::get_starting_monsters().get(species_name)
        {
            if !state.unlocked_monsters.contains(starting_monster) {
                state.unlocked_monsters.push(starting_monster.clone());
                unlocked_now.push(starting_monster.clone());
            }
        }
    }

    state.add_log(LogEntry::system(format!(
        "Unlocked {} for {} gold. Available summons: {}.",
        crate::data::monsters::get_species_display_name(species_name),
        unlock_cost,
        if unlocked_now.is_empty() {
            "none".to_string()
        } else {
            unlocked_now.join(", ")
        }
    )));

    Ok(())
}

/// Process hourly trait effects (e.g. regeneration)
// now data driven!
pub fn process_hourly_traits(state: &mut GameState) {
    for floor in &mut state.floors {
        for room in &mut floor.rooms {
            for monster in &mut room.monsters {
                if !monster.alive {
                    continue;
                }
                // Undead identity: the dead do not mend. Skip all regeneration.
                if crate::data::monsters::is_undead(&monster.type_name) {
                    continue;
                }

                for trait_data in &monster.active_traits {
                    // Look up definition
                    if let Some(def) = crate::data::traits::get_trait(&trait_data.id) {
                        if def.applies_to == "Hourly"
                            && def.trait_type == "Passive"
                            && def.effect_type == "HealPercent"
                        {
                            let heal_amount = (monster.max_hp as f32 * def.value).ceil() as i32;
                            if monster.hp < monster.max_hp {
                                monster.hp = (monster.hp + heal_amount).min(monster.max_hp);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Unlock variant forms once a monster *line* has pooled enough experience,
/// WITHOUT transforming any placed creature. The new form simply becomes
/// summonable, and can be placed onto an existing creature of its line to
/// upgrade it. Runs hourly; each form is only announced once.
pub fn process_evolution_unlocks(state: &mut GameState) {
    // Deepest floor each type currently stands on. A line learns its deeper
    // variants only while it is actually being fielded that deep — the pool
    // alone is not enough. BTreeMap so unlock order never depends on hashing.
    let mut deepest: BTreeMap<String, i32> = BTreeMap::new();
    for floor in &state.floors {
        for room in &floor.rooms {
            for monster in &room.monsters {
                let reached = deepest.entry(monster.type_name.clone()).or_insert(0);
                *reached = (*reached).max(room.floor_number);
            }
        }
    }

    // A line with branching paths unlocks every branch it qualifies for — the
    // player chooses which to field.
    let mut candidates: Vec<String> = Vec::new();
    for (type_name, floor_reached) in &deepest {
        for path in crate::data::evolutions::get_evolutions_for_monster(type_name) {
            let earned = state.type_experience(type_name) >= path.experience_required
                && *floor_reached >= path.conditions.min_floor;
            if earned
                && !state.unlocked_monsters.contains(&path.to_monster)
                && !candidates.contains(&path.to_monster)
            {
                candidates.push(path.to_monster);
            }
        }
    }

    for new_monster in candidates {
        state.unlocked_monsters.push(new_monster.clone());
        state.add_log(LogEntry::system(format!(
            "New variant unlocked: {}! Summon it, or place it onto one of its own kind to upgrade that defender.",
            new_monster
        )));
    }
}

#[cfg(test)]
mod tests;
