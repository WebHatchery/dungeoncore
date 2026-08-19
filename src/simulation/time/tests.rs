use super::*;
use crate::data::difficulty::Difficulty;
use crate::game_state::{Adventurer, AdventurerParty, Equipment, Stats, PARTY_MOVE_SECONDS};
use macroquad_toolkit::timing::Cooldown;

fn adventurer(id: u64, level: i32, alive: bool) -> Adventurer {
    Adventurer {
        id,
        name: format!("Delver {id}"),
        class_name: "Warrior".to_string(),
        race: "Human".to_string(),
        drive: crate::game_state::HeroDrive::Duty,
        resolve: 50,
        level,
        hp: if alive { 30 } else { 0 },
        max_hp: 30,
        alive,
        experience: 0,
        gold: 0,
        equipment: Equipment::default(),
        conditions: Vec::new(),
        scaled_stats: Stats {
            hp: 30,
            attack: 8,
            defense: 3,
        },
    }
}

fn party_of(members: Vec<Adventurer>) -> AdventurerParty {
    AdventurerParty {
        id: 1,
        members,
        current_floor: 1,
        current_room: 1,
        retreating: false,
        casualties: 0,
        loot: 0,
        entry_time: 0,
        target_floor: 1,
        snared_ticks: 0,
        alarmed: false,
        sieging: false,
        prev_room: 0,
        move_anim: Cooldown::new(PARTY_MOVE_SECONDS),
    }
}

#[test]
fn an_empty_dungeon_draws_nothing_from_intruders() {
    let s = GameState::new();
    assert_eq!(adventurer_presence_regen(&s), 0.0);
}

#[test]
fn a_higher_level_delver_feeds_the_core_more() {
    let mut low = GameState::new();
    low.adventurer_parties
        .push(party_of(vec![adventurer(1, 1, true)]));
    let mut high = GameState::new();
    high.adventurer_parties
        .push(party_of(vec![adventurer(1, 5, true)]));

    assert!(adventurer_presence_regen(&high) > adventurer_presence_regen(&low));
}

#[test]
fn the_dead_feed_nothing() {
    let mut s = GameState::new();
    s.adventurer_parties.push(party_of(vec![
        adventurer(1, 3, true),
        adventurer(2, 3, false),
    ]));
    let with_corpse = adventurer_presence_regen(&s);

    let mut alone = GameState::new();
    alone
        .adventurer_parties
        .push(party_of(vec![adventurer(1, 3, true)]));

    assert_eq!(with_corpse, adventurer_presence_regen(&alone));
}

#[test]
fn presence_income_respects_difficulty() {
    let members = vec![adventurer(1, 4, true), adventurer(2, 2, true)];
    let mut lean = GameState::new();
    lean.difficulty = Difficulty::Overlord;
    lean.adventurer_parties.push(party_of(members.clone()));
    let mut rich = GameState::new();
    rich.difficulty = Difficulty::Apprentice;
    rich.adventurer_parties.push(party_of(members));

    assert!(adventurer_presence_regen(&rich) > adventurer_presence_regen(&lean));
}

#[test]
fn a_party_inside_beats_the_idle_trickle() {
    // The point of the change: a raid in progress must out-earn an empty
    // dungeon by enough to cover respawning defenders and re-arming traps.
    let mut s = GameState::new();
    s.adventurer_parties.push(party_of(vec![
        adventurer(1, 2, true),
        adventurer(2, 3, true),
        adventurer(3, 3, true),
    ]));
    assert!(adventurer_presence_regen(&s) >= 5.0);
}

#[test]
fn presence_income_and_new_threat_tier_request_feedback() {
    let mut s = GameState::new();
    s.mana = 0;
    s.total_deaths = 10;
    s.adventurer_parties
        .push(party_of(vec![adventurer(1, 2, true)]));

    advance_time(&mut s);
    let sounds = s.take_sound_events();
    assert!(sounds.contains(&SoundEvent::Income));
    assert!(sounds.contains(&SoundEvent::Threat));
}
