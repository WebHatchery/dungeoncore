//! Persistent-hero journal and live-rival capture fixtures.

use macroquad_toolkit::timing::Cooldown;

use crate::game_state::{
    Adventurer, AdventurerParty, DungeonStatus, Equipment, GameState, HeroDrive, HeroInsight,
    HeroRecord, HeroStatus, HeroWard, Stats, PARTY_MOVE_SECONDS,
};
use crate::simulation;

pub(super) fn seed_journal(state: &mut GameState) {
    if let Some(species) = super::first_starter_species() {
        let _ = simulation::unlock_species(state, &species);
    }
    state.tutorial_active = false;
    state.day = 14;
    let mut sable = HeroRecord {
        id: 500,
        name: "Sable the Bold".to_string(),
        class_name: "Rogue".to_string(),
        race: "Halfling".to_string(),
        drive: HeroDrive::Glory,
        resolve: 78,
        level: 5,
        experience: 30,
        delves: 5,
        kills: 12,
        gold_stolen: 240,
        escapes: 4,
        deepest_floor: 8,
        insights: vec![
            HeroInsight {
                stratum_id: "rootways".to_string(),
                mastery: 1,
            },
            HeroInsight {
                stratum_id: "ember_faults".to_string(),
                mastery: 3,
            },
        ],
        status: HeroStatus::Alive,
        death_floor: 0,
        death_day: 0,
        journal: Vec::new(),
    };
    for (day, text) in [
        (2, "First delve into the dungeon"),
        (2, "Slew a Goblin on floor 1"),
        (3, "Escaped with 40 gold"),
        (6, "Returned for delve 2"),
        (6, "Slew a Goblin Archer on floor 2"),
        (7, "Escaped with 85 gold"),
        (9, "Reached level 4"),
        (11, "Returned for delve 4"),
        (12, "Slew an Orc on floor 2"),
        (13, "Escaped with 115 gold"),
        (13, "Prepared Fire III ward from the Ember Faults"),
        (14, "Returned for delve 5"),
    ] {
        sable.remember(day, text);
    }
    state.known_adventurers = vec![sable];
    state.selected_hero = Some(500);
}

pub(super) fn seed_rival(state: &mut GameState) {
    if let Some(species) = super::first_starter_species() {
        let _ = simulation::unlock_species(state, &species);
    }
    state.tutorial_active = false;
    let _ = simulation::add_room(state, None);
    let monster = state.unlocked_monsters.first().cloned();
    if let (Some(monster), Some((floor, pos))) = (monster, super::find_combat_room(state)) {
        let _ = simulation::place_monster(state, floor, pos, &monster);
    }
    state.status = DungeonStatus::Open;
    state.total_deaths = 20;
    state.known_adventurers = vec![sable_record(), pip_record()];
    if let Some((floor, pos)) = super::find_combat_room(state) {
        state.adventurer_parties.push(AdventurerParty {
            id: 1,
            members: vec![
                live_hero(500, "Sable the Bold", "Rogue", 38),
                live_hero(501, "Pip", "Warrior", 44),
            ],
            current_floor: floor,
            current_room: pos,
            retreating: false,
            casualties: 0,
            loot: 60,
            entry_time: 8,
            target_floor: 1,
            snared_ticks: 0,
            alarmed: false,
            sieging: false,
            prev_room: 0,
            move_anim: Cooldown::new(PARTY_MOVE_SECONDS),
        });
        state.selected_room = Some((floor, pos));
    }
}

fn sable_record() -> HeroRecord {
    HeroRecord {
        id: 500,
        name: "Sable the Bold".to_string(),
        class_name: "Rogue".to_string(),
        race: "Halfling".to_string(),
        drive: HeroDrive::Glory,
        resolve: 78,
        level: 5,
        experience: 0,
        delves: 5,
        kills: 12,
        gold_stolen: 240,
        escapes: 4,
        deepest_floor: 8,
        insights: vec![HeroInsight {
            stratum_id: "ember_faults".to_string(),
            mastery: 3,
        }],
        status: HeroStatus::Inside,
        death_floor: 0,
        death_day: 0,
        journal: Vec::new(),
    }
}

fn pip_record() -> HeroRecord {
    HeroRecord {
        id: 501,
        name: "Pip".to_string(),
        class_name: "Warrior".to_string(),
        race: "Human".to_string(),
        drive: HeroDrive::Duty,
        resolve: 50,
        level: 2,
        experience: 0,
        delves: 1,
        kills: 0,
        gold_stolen: 0,
        escapes: 0,
        deepest_floor: 1,
        insights: Vec::new(),
        status: HeroStatus::Inside,
        death_floor: 0,
        death_day: 0,
        journal: Vec::new(),
    }
}

fn live_hero(id: u64, name: &str, class: &str, hp: i32) -> Adventurer {
    Adventurer {
        id,
        name: name.to_string(),
        class_name: class.to_string(),
        race: "Human".to_string(),
        drive: if id == 500 {
            HeroDrive::Glory
        } else {
            HeroDrive::Duty
        },
        resolve: if id == 500 { 78 } else { 50 },
        ward: if id == 500 {
            HeroWard {
                stratum_name: "Ember Faults".to_string(),
                element: "Fire".to_string(),
                mastery: 3,
            }
        } else {
            Default::default()
        },
        level: 4,
        hp,
        max_hp: 50,
        alive: true,
        experience: 0,
        gold: 0,
        equipment: Equipment::default(),
        conditions: Vec::new(),
        scaled_stats: Stats {
            hp: 50,
            attack: 10,
            defense: 4,
        },
    }
}
