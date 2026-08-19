use super::*;
use crate::game_state::{Monster, Stats};

fn dead_monster(type_name: &str, max_hp: i32) -> Monster {
    Monster {
        id: 1,
        type_name: type_name.to_string(),
        hp: 0,
        max_hp,
        alive: false,
        is_boss: false,
        fusion_rank: 1,
        scaled_stats: Stats {
            hp: max_hp,
            attack: 5,
            defense: 2,
        },
        active_traits: Vec::new(),
    }
}

#[test]
fn undead_respawn_free_and_whole_at_zero_mana() {
    let mut s = GameState::new();
    s.mana = 0;
    s.floors[0].rooms[0]
        .monsters
        .push(dead_monster("Skeleton", 40));
    respawn_monsters(&mut s);
    let m = &s.floors[0].rooms[0].monsters[0];
    assert!(m.alive);
    assert_eq!(m.hp, m.max_hp, "undead rise whole");
    assert_eq!(s.mana, 0, "undead cost nothing");
}

#[test]
fn living_respawn_charges_mana_when_affordable() {
    let mut s = GameState::new();
    s.mana = 100;
    s.floors[0].rooms[0]
        .monsters
        .push(dead_monster("Goblin", 40));
    respawn_monsters(&mut s);
    let m = &s.floors[0].rooms[0].monsters[0];
    assert!(m.alive);
    assert_eq!(m.hp, m.max_hp);
    assert!(s.mana < 100, "the living cost mana to reknit");
}

/// A dungeon with one Normal room on floor 1 and Goblins unlocked.
fn dungeon_with_a_combat_room() -> GameState {
    let mut s = GameState::new();
    s.mana = 100_000;
    crate::simulation::rooms::add_room(&mut s, None).expect("room built");
    let template = crate::data::monsters::get_monster_template("Goblin").expect("goblin exists");
    s.unlocked_species.push(template.species.clone());
    s.unlocked_monsters.push(template.name.clone());
    s
}

#[test]
fn a_room_takes_its_fill_and_then_refuses_more() {
    let mut s = dungeon_with_a_combat_room();
    let capacity = crate::data::constants::room_monster_capacity(1, false);
    for _ in 0..capacity {
        place_monster(&mut s, 1, 1, "Goblin").expect("slot available");
    }

    let err = place_monster(&mut s, 1, 1, "Goblin").expect_err("room is full");
    assert!(err.contains("full"), "{err}");
    assert_eq!(s.floors[0].rooms[1].monsters.len(), capacity);
}

#[test]
fn boss_rooms_hold_one_throne_slot_for_a_boss_unique() {
    let mut s = dungeon_with_a_combat_room();
    let room = &mut s.floors[0].rooms[1];
    room.room_type = RoomType::Boss;
    // Use a scaled floor so this throne has two total slots: one guard and
    // one reserved unique. The fixture need not alter the floor graph.
    room.floor_number = 3;
    let king = crate::data::monsters::get_monster_template("Goblin King").unwrap();
    s.unlocked_monsters.push(king.name.clone());

    place_monster(&mut s, 1, 1, "Goblin").expect("one ordinary guard fits");
    let reserved = place_monster(&mut s, 1, 1, "Goblin").expect_err("throne held open");
    assert!(reserved.contains("reserved"), "{reserved}");
    place_monster(&mut s, 1, 1, "Goblin King").expect("unique claims throne");
    assert_eq!(s.floors[0].rooms[1].monsters.len(), 2);
}

#[test]
fn an_over_capacity_room_keeps_its_defenders() {
    // A save written before the limit existed can hold more than the cap.
    // Placement stops; nothing the player paid for is taken away.
    let mut s = dungeon_with_a_combat_room();
    let capacity = crate::data::constants::room_monster_capacity(1, false);
    for _ in 0..capacity + 3 {
        s.floors[0].rooms[1]
            .monsters
            .push(dead_monster("Goblin", 40));
    }

    assert!(place_monster(&mut s, 1, 1, "Goblin").is_err());
    assert_eq!(s.floors[0].rooms[1].monsters.len(), capacity + 3);
}

#[test]
fn a_dismissal_frees_the_slot_it_held() {
    let mut s = dungeon_with_a_combat_room();
    let capacity = crate::data::constants::room_monster_capacity(1, false);
    for _ in 0..capacity {
        place_monster(&mut s, 1, 1, "Goblin").expect("slot available");
    }
    let victim = s.floors[0].rooms[1].monsters[0].id;

    remove_monster(&mut s, 1, 1, victim).expect("dismissed");
    place_monster(&mut s, 1, 1, "Goblin").expect("the freed slot takes a new defender");
}

#[test]
fn a_line_unlocks_its_variant_once_the_pool_fills() {
    let mut s = dungeon_with_a_combat_room();
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    // Goblin -> Orc wants 150 pooled XP and a floor-1 posting.
    s.add_type_experience("Goblin", 150);

    process_evolution_unlocks(&mut s);
    assert!(s.unlocked_monsters.iter().any(|m| m == "Orc"));
}

#[test]
fn the_pool_belongs_to_the_line_not_the_creature() {
    // Two goblins each contributing half the threshold unlock the variant;
    // no single creature ever reached it alone.
    let mut s = dungeon_with_a_combat_room();
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    s.add_type_experience("Goblin", 75);
    s.add_type_experience("Goblin", 75);

    assert_eq!(s.type_experience("Goblin"), 150);
    process_evolution_unlocks(&mut s);
    assert!(s.unlocked_monsters.iter().any(|m| m == "Orc"));
}

#[test]
fn a_line_the_dungeon_does_not_field_learns_nothing() {
    // The pool alone is not enough — the line has to actually be posted.
    let mut s = dungeon_with_a_combat_room();
    s.add_type_experience("Goblin", 5_000);

    process_evolution_unlocks(&mut s);
    assert!(!s.unlocked_monsters.iter().any(|m| m == "Orc"));
}

#[test]
fn living_respawn_wounded_when_broke() {
    let mut s = GameState::new();
    s.mana = 0;
    s.floors[0].rooms[0]
        .monsters
        .push(dead_monster("Goblin", 40));
    respawn_monsters(&mut s);
    let m = &s.floors[0].rooms[0].monsters[0];
    assert!(m.alive, "never lost outright");
    assert!(m.hp > 0 && m.hp < m.max_hp, "crawls back wounded");
}
