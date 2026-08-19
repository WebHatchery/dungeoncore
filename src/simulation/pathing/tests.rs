use super::*;
use crate::game_state::{Adventurer, HeroDrive, Monster, Room, RoomType, Stats};

/// Entrance(0) → {1, 2} → Core(3). Room 1 is a treasure lure (no guards);
/// room 2 is a guarded killbox. Both reconverge at the Core.
fn forked_floor() -> Floor {
    let mut floor = Floor::new(1, 1, true);
    let mut entrance = Room::new(0, RoomType::Entrance, 0, 1);
    entrance.exits = vec![1, 2];
    let mut lure = Room::new(1, RoomType::Normal, 1, 1);
    lure.exits = vec![3];
    lure.loot = 80;
    let mut killbox = Room::new(2, RoomType::Normal, 2, 1);
    killbox.exits = vec![3];
    killbox.monsters.push(Monster {
        id: 1,
        type_name: "Goblin".to_string(),
        hp: 40,
        max_hp: 40,
        alive: true,
        is_boss: false,
        scaled_stats: Stats {
            hp: 40,
            attack: 40,
            defense: 2,
        },
        active_traits: Vec::new(),
    });
    let core = Room::new(3, RoomType::Core, 3, 1);
    floor.rooms = vec![entrance, lure, killbox, core];
    floor
}

fn party() -> AdventurerParty {
    AdventurerParty {
        id: 1,
        members: Vec::new(),
        current_floor: 1,
        current_room: 0,
        retreating: false,
        casualties: 0,
        loot: 0,
        entry_time: 0,
        target_floor: 1,
        snared_ticks: 0,
        alarmed: false,
        sieging: false,
        prev_room: 0,
        move_anim: macroquad_toolkit::timing::Cooldown::new(crate::game_state::PARTY_MOVE_SECONDS),
    }
}

fn party_with_drive(drive: HeroDrive) -> AdventurerParty {
    let mut party = party();
    party.members.push(Adventurer {
        id: 7,
        name: "Mara".to_string(),
        class_name: "Warrior".to_string(),
        race: "Human".to_string(),
        drive,
        resolve: 50,
        level: 2,
        hp: 100,
        max_hp: 100,
        alive: true,
        experience: 0,
        gold: 0,
        equipment: Default::default(),
        conditions: Vec::new(),
        scaled_stats: Stats {
            hp: 100,
            attack: 20,
            defense: 10,
        },
    });
    party
}

#[test]
fn greedy_party_takes_the_loot_lure_over_the_killbox() {
    let s = GameState::new(); // threat 0 → greedy
    let floor = forked_floor();
    assert_eq!(choose_exit(&s, &floor, &party(), &[1, 2]), 1);
}

#[test]
fn desperate_party_beelines_the_shortest_path_to_core() {
    // Make the guarded branch (2) the *shorter* route to the core, so a
    // greedy party would avoid it but a beelining one takes it anyway.
    let mut floor = forked_floor();
    // Reroute the lure (branch 1) through an extra room: 0 → {1,2};
    // 1 → 4 → Core(3); 2 → Core(3). Branch 2 is the shorter route now.
    let mut detour = Room::new(4, RoomType::Normal, 4, 1);
    detour.exits = vec![3];
    floor
        .rooms
        .iter_mut()
        .find(|r| r.position == 1)
        .unwrap()
        .exits = vec![4];
    floor.rooms.push(detour);

    let mut s = GameState::new();
    s.total_deaths = 100; // threat tier 4 → beeline
    assert!(is_beelining(&s, &party()));
    assert_eq!(
        choose_exit(&s, &floor, &party(), &[1, 2]),
        2,
        "beeline takes the shorter path to the core"
    );
}

#[test]
fn distance_to_core_counts_rooms() {
    let floor = forked_floor();
    assert_eq!(distance_to_core(&floor, 3), Some(0));
    assert_eq!(distance_to_core(&floor, 1), Some(1));
    assert_eq!(distance_to_core(&floor, 0), Some(2));
}

#[test]
fn glory_seekers_choose_a_fight_that_other_heroes_avoid() {
    let s = GameState::new();
    let mut floor = forked_floor();
    floor
        .rooms
        .iter_mut()
        .find(|room| room.position == 1)
        .unwrap()
        .loot = 5;

    assert_eq!(
        choose_exit(&s, &floor, &party_with_drive(HeroDrive::Duty), &[1, 2]),
        1
    );
    assert_eq!(
        choose_exit(&s, &floor, &party_with_drive(HeroDrive::Glory), &[1, 2]),
        2
    );
}
