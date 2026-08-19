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
        move_anim: macroquad_toolkit::timing::Cooldown::new(crate::game_state::PARTY_MOVE_SECONDS),
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

fn damage_to_room_monster(upgrade_name: Option<&str>) -> i32 {
    let mut state = GameState::new();
    let mut room = Room::new(99, RoomType::Normal, 2, 1);
    if let Some(name) = upgrade_name {
        room.upgrades.push(
            crate::data::upgrades::get_upgrade_template(name)
                .unwrap()
                .to_room_upgrade(),
        );
    }
    room.monsters.push(sturdy_monster());
    state.floors[0].rooms.push(room);
    let room_idx = state.floors[0].rooms.len() - 1;
    state.adventurer_parties.push(lone_invader(300));
    let before = state.floors[0].rooms[room_idx].monsters[0].hp;
    resolve_combat(&mut state, 0, 0, room_idx);
    before - state.floors[0].rooms[room_idx].monsters[0].hp
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

#[test]
fn stone_walls_reduce_damage_to_defenders() {
    let open = damage_to_room_monster(None);
    let walled = damage_to_room_monster(Some("Stone Walls"));
    assert!(open > 0);
    assert!(
        walled < open,
        "Stone Walls should protect defenders: {walled} < {open}"
    );
}

#[test]
fn treasure_and_evolution_rooms_expose_defenders_to_adventurers() {
    let open = damage_to_room_monster(None);
    let treasure = damage_to_room_monster(Some("Gold Cache"));
    let evolution = damage_to_room_monster(Some("Evolution Pit"));
    assert!(
        treasure > open,
        "treasure lure should empower invader attacks"
    );
    assert!(
        evolution > open,
        "unstable growth should expose weak points"
    );
}

#[test]
fn growth_chamber_regenerates_a_wounded_defender() {
    let mut state = GameState::new();
    let mut room = Room::new(99, RoomType::Normal, 2, 1);
    let mut monster = sturdy_monster();
    monster.hp = 200;
    room.monsters.push(monster);
    room.upgrades.push(
        crate::data::upgrades::get_upgrade_template("Growth Chamber")
            .unwrap()
            .to_room_upgrade(),
    );
    state.floors[0].rooms.push(room);
    let room_idx = state.floors[0].rooms.len() - 1;
    state.adventurer_parties.push(lone_invader(300));

    resolve_combat(&mut state, 0, 0, room_idx);

    assert!(
        state.floors[0].rooms[room_idx].monsters[0].hp > 200,
        "Growth Chamber should heal before the room trades attacks"
    );
}

#[test]
fn spike_trap_applies_its_bleed_secondary_effect() {
    let mut state = GameState::new();
    let mut room = Room::new(99, RoomType::Normal, 2, 1);
    room.upgrades.push(
        crate::data::upgrades::get_upgrade_template("Spike Trap")
            .unwrap()
            .to_room_upgrade(),
    );
    state.floors[0].rooms.push(room);
    let room_idx = state.floors[0].rooms.len() - 1;
    state.adventurer_parties.push(lone_invader(10_000));

    for _ in 0..100 {
        super::traps::resolve_trap(&mut state, 0, 0, room_idx);
    }

    assert!(state.adventurer_parties[0].members[0]
        .conditions
        .iter()
        .any(|condition| condition.kind == "Bleeding"));
}
