use macroquad_toolkit::timing::Cooldown;

use crate::game_state::{
    Adventurer, AdventurerParty, EffectAnchor, EffectKind, GameState, HeroRecord, HeroStatus,
    LogEntry, RoomType, SoundEvent, Stats, PARTY_MOVE_SECONDS,
};

/// What awakening a core power does. Stat effects (`CoreHp`, `MaxMana`) are
/// baked in the moment the power is bought; the rest are *passive* and summed
/// from all owned powers wherever the sim reads them (regen, retreat, smite).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CorePowerEffect {
    /// Immediate, permanent core-HP increase.
    CoreHp(i32),
    /// Immediate, permanent max-mana increase.
    MaxMana(i32),
    /// Permanent bonus to hourly mana regeneration.
    ManaRegen(f32),
    /// Invaders break this many casualties sooner (stacks).
    Dread(i32),
    /// Bonus flat damage added to every Core Smite.
    SmiteDamage(i32),
    /// Seconds shaved off the Core Smite cooldown.
    SmiteCooldown(f32),
}

/// A soul-bought permanent core power, a node in the core-power tree.
pub struct CorePower {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i32,
    /// Depth in the tree (0 = root); drives row layout in the UI.
    pub tier: u8,
    /// Powers that must already be owned before this one can be awakened.
    pub requires: &'static [&'static str],
    pub effect: CorePowerEffect,
}

use CorePowerEffect::*;

/// The core-power tree. Three roots (kept by id for save compatibility) branch
/// into an economy line (regen/mana), a bulwark line (core HP), and an
/// offense line that empowers the Core Smite lever — so repeated prestiges can
/// specialise in different directions.
pub const CORE_POWERS: [CorePower; 15] = [
    // --- Roots (tier 0) ---------------------------------------------------
    CorePower {
        id: "deep_roots",
        name: "Deep Roots",
        description: "The core drinks deeper: +1 mana regen forever.",
        cost: 3,
        tier: 0,
        requires: &[],
        effect: ManaRegen(1.0),
    },
    CorePower {
        id: "bulwark_core",
        name: "Bulwark Core",
        description: "Reinforce the heart: +250 core HP.",
        cost: 4,
        tier: 0,
        requires: &[],
        effect: CoreHp(250),
    },
    CorePower {
        id: "dread_aura",
        name: "Dread Aura",
        description: "Invaders lose their nerve one casualty sooner.",
        cost: 3,
        tier: 0,
        requires: &[],
        effect: Dread(1),
    },
    // --- Tier 1 -----------------------------------------------------------
    CorePower {
        id: "wellspring",
        name: "Wellspring",
        description: "The roots run deeper still: +1 more mana regen.",
        cost: 5,
        tier: 1,
        requires: &["deep_roots"],
        effect: ManaRegen(1.0),
    },
    CorePower {
        id: "mana_font",
        name: "Mana Font",
        description: "Widen the reservoir: +150 max mana.",
        cost: 4,
        tier: 1,
        requires: &["deep_roots"],
        effect: MaxMana(150),
    },
    CorePower {
        id: "iron_heart",
        name: "Iron Heart",
        description: "Plate the heart: +400 core HP.",
        cost: 6,
        tier: 1,
        requires: &["bulwark_core"],
        effect: CoreHp(400),
    },
    CorePower {
        id: "searing_smite",
        name: "Searing Smite",
        description: "Core Smite sears hotter: +20 smite damage.",
        cost: 4,
        tier: 1,
        requires: &["dread_aura"],
        effect: SmiteDamage(20),
    },
    CorePower {
        id: "quickening",
        name: "Quickening",
        description: "The heart recovers faster: -4s Core Smite cooldown.",
        cost: 5,
        tier: 1,
        requires: &["dread_aura"],
        effect: SmiteCooldown(4.0),
    },
    // --- Tier 2 -----------------------------------------------------------
    CorePower {
        id: "aquifer",
        name: "Aquifer",
        description: "Tap the deep aquifer: +2 mana regen.",
        cost: 8,
        tier: 2,
        requires: &["wellspring"],
        effect: ManaRegen(2.0),
    },
    CorePower {
        id: "grand_reservoir",
        name: "Grand Reservoir",
        description: "A vast store of power: +300 max mana.",
        cost: 7,
        tier: 2,
        requires: &["mana_font"],
        effect: MaxMana(300),
    },
    CorePower {
        id: "adamant_heart",
        name: "Adamant Heart",
        description: "All but unbreakable: +750 core HP.",
        cost: 9,
        tier: 2,
        requires: &["iron_heart"],
        effect: CoreHp(750),
    },
    CorePower {
        id: "cataclysm",
        name: "Cataclysm",
        description: "Ruinous force: +45 Core Smite damage.",
        cost: 8,
        tier: 2,
        requires: &["searing_smite"],
        effect: SmiteDamage(45),
    },
    CorePower {
        id: "terror_incarnate",
        name: "Terror Incarnate",
        description: "Naked fear: invaders break one more casualty sooner.",
        cost: 7,
        tier: 2,
        requires: &["dread_aura"],
        effect: Dread(1),
    },
    // --- Tier 3 (capstones) ----------------------------------------------
    CorePower {
        id: "eternal_wellspring",
        name: "Eternal Wellspring",
        description: "An unending flood of power: +3 mana regen.",
        cost: 12,
        tier: 3,
        requires: &["aquifer"],
        effect: ManaRegen(3.0),
    },
    CorePower {
        id: "worldbreaker",
        name: "Worldbreaker",
        description: "The heart's judgement is swift: -4s Core Smite cooldown.",
        cost: 12,
        tier: 3,
        requires: &["cataclysm", "quickening"],
        effect: SmiteCooldown(4.0),
    },
];

/// Look up a power by id.
pub fn core_power(id: &str) -> Option<&'static CorePower> {
    CORE_POWERS.iter().find(|p| p.id == id)
}

/// Powers currently owned by the dungeon.
fn owned_powers(state: &GameState) -> impl Iterator<Item = &'static CorePower> + '_ {
    CORE_POWERS
        .iter()
        .filter(move |p| state.has_core_power(p.id))
}

/// Are all of a power's prerequisites already owned?
pub fn prereqs_met(state: &GameState, power: &CorePower) -> bool {
    power.requires.iter().all(|req| state.has_core_power(req))
}

/// Summed permanent mana-regen bonus from owned core powers.
pub fn core_mana_regen_bonus(state: &GameState) -> f32 {
    owned_powers(state)
        .filter_map(|p| match p.effect {
            ManaRegen(v) => Some(v),
            _ => None,
        })
        .sum()
}

/// Summed "invaders break sooner" bonus from owned core powers.
pub fn core_dread_bonus(state: &GameState) -> i32 {
    owned_powers(state)
        .filter_map(|p| match p.effect {
            Dread(v) => Some(v),
            _ => None,
        })
        .sum()
}

/// Summed flat Core Smite damage bonus from owned core powers.
pub fn core_smite_damage_bonus(state: &GameState) -> i32 {
    owned_powers(state)
        .filter_map(|p| match p.effect {
            SmiteDamage(v) => Some(v),
            _ => None,
        })
        .sum()
}

/// Summed Core Smite cooldown reduction (seconds) from owned core powers.
pub fn core_smite_cooldown_reduction(state: &GameState) -> f32 {
    owned_powers(state)
        .filter_map(|p| match p.effect {
            SmiteCooldown(v) => Some(v),
            _ => None,
        })
        .sum()
}

/// Purchase a permanent core power with souls. Fails if already owned, if any
/// prerequisite is missing, or if souls are short.
pub fn buy_core_power(state: &mut GameState, id: &str) -> Result<(), String> {
    if state.has_core_power(id) {
        return Err("That core power is already awakened.".into());
    }
    let power = core_power(id).ok_or("Unknown core power")?;

    for req in power.requires {
        if !state.has_core_power(req) {
            let req_name = core_power(req).map(|p| p.name).unwrap_or(req);
            return Err(format!("Awaken {} first.", req_name));
        }
    }

    if state.souls < power.cost {
        return Err(format!("Not enough souls! Need {}.", power.cost));
    }
    state.souls -= power.cost;
    state.core_powers.push(id.to_string());

    // Bake in immediate stat effects; passive effects are read at runtime.
    match power.effect {
        CoreHp(n) => {
            state.core_max_hp += n;
            state.core_hp += n;
        }
        MaxMana(n) => state.max_mana += n,
        ManaRegen(_) | Dread(_) | SmiteDamage(_) | SmiteCooldown(_) => {}
    }

    state.add_log(LogEntry::system(format!(
        "Core power awakened: {}!",
        power.name
    )));
    Ok(())
}

/// If the realm's fury has peaked (threat tier 4) and no siege is under way,
/// muster the army: one overwhelming elite party that marches on the core.
pub fn maybe_launch_siege(state: &mut GameState) {
    if state.siege_active || state.threat_tier() < 4 {
        return;
    }
    // Wait for a lull between ordinary raids so the siege is a clean event.
    if !state.adventurer_parties.is_empty() {
        return;
    }

    let deepest = state
        .floors
        .iter()
        .find(|f| f.is_deepest)
        .map(|f| f.number)
        .unwrap_or(1);

    let names = crate::data::adventurers::get_adventurer_names();
    let classes = crate::data::adventurers::get_adventurer_classes();
    let races = crate::data::adventurers::get_race_names();

    // Elites scale with how deep the dungeon runs and how many sieges repelled,
    // and with the run's difficulty.
    let level = 5 + state.total_floors + state.prestige * 2;
    let stat_mult = state.difficulty.profile().invader_stat_mult;
    let size = 5usize;
    let mut members = Vec::with_capacity(size);
    for _ in 0..size {
        let class = classes[state.run_rng.below(classes.len())].clone();
        let name = names[state.run_rng.below(names.len())].clone();
        let race = races
            .get(state.run_rng.below(races.len()))
            .cloned()
            .unwrap_or_else(|| "Human".to_string());
        let id = state.run_rng.next_u64();

        // Register siege champions in the ledger like anyone else.
        state.known_adventurers.push(HeroRecord {
            id,
            name: name.clone(),
            class_name: class.name.clone(),
            race: race.clone(),
            drive: crate::game_state::HeroDrive::Duty,
            resolve: 80,
            level,
            experience: 0,
            delves: 1,
            kills: 0,
            gold_stolen: 0,
            escapes: 0,
            deepest_floor: 0,
            insights: Vec::new(),
            status: HeroStatus::Inside,
            death_floor: 0,
            death_day: 0,
            journal: Vec::new(),
        });

        let hp = (((class.hp + (level - 1) * 12) as f32 * stat_mult).round() as i32).max(1);
        let attack = (((class.attack + (level - 1) * 3) as f32 * stat_mult).round() as i32).max(1);
        let defense =
            (((class.defense + (level - 1) * 2) as f32 * stat_mult).round() as i32).max(0);
        members.push(Adventurer {
            id,
            name,
            class_name: class.name.clone(),
            race,
            drive: crate::game_state::HeroDrive::Duty,
            resolve: 80,
            ward: Default::default(),
            level,
            hp,
            max_hp: hp,
            alive: true,
            experience: 0,
            gold: 0,
            equipment: crate::data::equipment::recommended_loadout(&class.name, level),
            conditions: Vec::new(),
            scaled_stats: Stats {
                hp,
                attack,
                defense,
            },
        });
    }

    state.adventurer_parties.push(AdventurerParty {
        id: state.run_rng.next_u64(),
        members,
        current_floor: 1,
        current_room: 0,
        retreating: false,
        casualties: 0,
        loot: 0,
        entry_time: state.hour,
        target_floor: deepest,
        snared_ticks: 0,
        alarmed: false,
        sieging: true,
        prev_room: 0,
        move_anim: Cooldown::new(PARTY_MOVE_SECONDS),
    });
    state.siege_active = true;

    state.status = crate::game_state::DungeonStatus::Open;
    state.add_log(LogEntry::system(
        "THE SIEGE BEGINS! The realm's army storms the dungeon, bound for your core.",
    ));
    if let Some((floor, room)) = core_location(state) {
        state.push_effect_at(
            floor,
            room,
            "SIEGE!",
            EffectKind::SiegeArrival,
            EffectAnchor::Center,
        );
    }
    state.queue_sound(SoundEvent::Siege);
}

/// A siege party has reached the core. Trade blows: the party batters the
/// heart while the core's wards cut them down. Returns true if the party is
/// spent (repelled).
pub fn assault_core(state: &mut GameState, party_idx: usize) -> bool {
    let party_attack: i32 = state.adventurer_parties[party_idx]
        .members
        .iter()
        .filter(|a| a.alive)
        .map(|a| a.scaled_stats.attack)
        .sum();

    // The core bites back, harder the deeper and more prestigious it is.
    let core_attack = 18 + state.total_floors * 8 + state.prestige * 12;

    state.core_hp -= party_attack;
    if state.core_hp <= 0 {
        state.core_hp = 0;
        state.game_over = true;
        state.add_log(LogEntry::system(
            "THE CORE HAS FALLEN. Your dungeon is undone.",
        ));
        return true;
    }

    state.add_log(LogEntry::combat(format!(
        "The core takes {} damage! ({}/{})",
        party_attack, state.core_hp, state.core_max_hp
    )));
    state.queue_sound(SoundEvent::CoreDamage);

    // Core wards strike the assailants.
    let mut deaths: Vec<(u64, String)> = Vec::new();
    {
        let party = &mut state.adventurer_parties[party_idx];
        let alive_idxs: Vec<usize> = party
            .members
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive)
            .map(|(i, _)| i)
            .collect();
        // Spread the core's wrath across up to two defenders' worth of hits.
        for &idx in alive_idxs.iter().take(2) {
            let member = &mut party.members[idx];
            member.hp -= core_attack;
            if member.hp <= 0 {
                member.hp = 0;
                member.alive = false;
                party.casualties += 1;
                deaths.push((member.id, member.name.clone()));
            }
        }
    }

    let floor = state.adventurer_parties[party_idx].current_floor;
    for (id, name) in deaths {
        state.record_hero_death(id, floor);
        state.total_deaths += 1;
        state.add_log(LogEntry::combat(format!(
            "The core's wards strike down {}!",
            name
        )));
    }

    let repelled = state.adventurer_parties[party_idx]
        .members
        .iter()
        .all(|a| !a.alive);
    if repelled {
        state.adventurer_parties[party_idx].retreating = true;
    }
    repelled
}

/// The siege has been broken. Grant prestige, a permanent boon, and reset the
/// realm's fury so the long game can continue.
pub fn repel_siege(state: &mut GameState) {
    state.siege_active = false;
    state.prestige += 1;

    // Permanent boon: a sturdier, mana-richer heart.
    state.core_max_hp += 150;
    state.core_hp = state.core_max_hp;
    state.max_mana += 100;

    // The realm licks its wounds; the threat meter resets.
    state.total_deaths = 0;
    state.threat_warned = 0;

    let rank = crate::simulation::milestones::prestige_rank(state.prestige);
    state.add_log(LogEntry::system(format!(
        "THE SIEGE IS BROKEN! Prestige {} attained — your dungeon is now a {}. The core swells (+150 HP, +100 max mana) and the realm retreats to lick its wounds.",
        state.prestige, rank
    )));
    state.queue_sound(SoundEvent::Prestige);
    if let Some((floor, room)) = core_location(state) {
        state.push_effect_at(
            floor,
            room,
            format!("Prestige {}", state.prestige),
            EffectKind::Prestige,
            EffectAnchor::Center,
        );
    }

    // A repelled siege can cross prestige milestones (Ascendant, Eternal Core).
    crate::simulation::milestones::check_milestones(state);
}

fn core_location(state: &GameState) -> Option<(i32, usize)> {
    state.floors.iter().find_map(|floor| {
        floor
            .rooms
            .iter()
            .find(|room| room.room_type == RoomType::Core)
            .map(|room| (floor.number, room.position))
    })
}

#[cfg(test)]
mod tests;
