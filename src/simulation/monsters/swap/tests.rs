use super::*;
use crate::simulation::place_monster;

/// A dungeon with one Normal room on floor 1, Goblins and Orcs unlocked, and
/// mana to spare.
fn dungeon() -> GameState {
    let mut s = GameState::new();
    s.mana = 100_000;
    s.gold = 100_000;
    crate::simulation::rooms::add_room(&mut s, None).expect("room built");
    for name in ["Goblin", "Orc"] {
        let template = get_monster_template(name).expect("template exists");
        if !s.unlocked_species.contains(&template.species) {
            s.unlocked_species.push(template.species.clone());
        }
        s.unlocked_monsters.push(template.name.clone());
    }
    s
}

fn only_monster(s: &GameState) -> &crate::game_state::Monster {
    &s.floors[0].rooms[1].monsters[0]
}

/// A party in the dungeon — only its presence matters to the swap gate.
fn raiding_party() -> crate::game_state::AdventurerParty {
    crate::game_state::AdventurerParty {
        id: 1,
        members: Vec::new(),
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
        move_anim: macroquad_toolkit::timing::Cooldown::new(crate::game_state::PARTY_MOVE_SECONDS),
    }
}

#[test]
fn a_variant_of_the_same_line_is_an_upgrade() {
    let s = dungeon();
    let mut s = s;
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    let id = only_monster(&s).id;

    let plan = plan_swap(&s, 1, 1, id, "Orc").expect("plan");
    assert_eq!(plan.kind, SwapKind::Upgrade);
}

#[test]
fn an_unrelated_monster_is_a_replacement() {
    let mut s = dungeon();
    // Slimes are not on the goblin line.
    let slime = get_monster_template("Green Slime").expect("template");
    s.unlocked_species.push(slime.species.clone());
    s.unlocked_monsters.push(slime.name.clone());
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    let id = only_monster(&s).id;

    let plan = plan_swap(&s, 1, 1, id, "Green Slime").expect("plan");
    assert_eq!(plan.kind, SwapKind::Replace);
}

#[test]
fn growing_a_line_undercuts_evicting_it() {
    // The whole point of a monster line: the upgrade path must be cheaper in
    // mana than retiring the occupant and summoning the same creature fresh.
    let mut s = dungeon();
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    let id = only_monster(&s).id;

    let upgrade = plan_swap(&s, 1, 1, id, "Orc").expect("plan");
    let orc_cost = get_monster_mana_cost(
        get_monster_template("Orc").expect("template").base_cost,
        1,
        false,
    );
    let refund = retirement_refund(&s.floors[0].rooms[1], "Goblin", 1);
    let replace_equivalent = orc_cost - refund;

    assert_eq!(upgrade.kind, SwapKind::Upgrade);
    assert!(
        upgrade.mana < replace_equivalent,
        "upgrade {} should undercut replace {}",
        upgrade.mana,
        replace_equivalent
    );
}

#[test]
fn an_upgrade_keeps_the_slot_and_takes_the_new_form() {
    let mut s = dungeon();
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    let id = only_monster(&s).id;
    let held_before = s.floors[0].rooms[1].monsters.len();

    swap_monster(&mut s, 1, 1, id, "Orc").expect("upgraded");

    let monster = only_monster(&s);
    assert_eq!(monster.id, id, "same creature, grown up");
    assert_eq!(monster.type_name, "Orc");
    assert_eq!(monster.hp, monster.max_hp, "the new form arrives whole");
    assert_eq!(s.floors[0].rooms[1].monsters.len(), held_before);
}

#[test]
fn a_fused_veteran_keeps_its_rank_when_it_evolves() {
    let mut s = dungeon();
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    let id = only_monster(&s).id;
    s.floors[0].rooms[1].monsters[0].fusion_rank = 2;

    swap_monster(&mut s, 1, 1, id, "Orc").expect("upgraded");

    let monster = only_monster(&s);
    let base_orc_hp = get_scaled_stats(
        Stats {
            hp: get_monster_template("Orc").unwrap().hp,
            attack: get_monster_template("Orc").unwrap().attack,
            defense: get_monster_template("Orc").unwrap().defense,
        },
        1,
        false,
    )
    .hp;
    assert_eq!(monster.fusion_rank, 2);
    assert!(monster.max_hp > base_orc_hp);
}

#[test]
fn a_replacement_evicts_the_occupant() {
    let mut s = dungeon();
    let slime = get_monster_template("Green Slime").expect("template");
    s.unlocked_species.push(slime.species.clone());
    s.unlocked_monsters.push(slime.name.clone());
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    let id = only_monster(&s).id;

    swap_monster(&mut s, 1, 1, id, "Green Slime").expect("replaced");

    let monster = only_monster(&s);
    assert_ne!(monster.id, id, "a different creature stands there now");
    assert_eq!(monster.type_name, "Green Slime");
    assert_eq!(
        s.floors[0].rooms[1].monsters.len(),
        1,
        "still one slot used"
    );
}

#[test]
fn a_swap_the_dungeon_cannot_afford_costs_it_nothing() {
    // The occupant must survive a failed swap — it is paid for.
    let mut s = dungeon();
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    let id = only_monster(&s).id;
    s.mana = 0;
    s.gold = 0;

    assert!(swap_monster(&mut s, 1, 1, id, "Orc").is_err());
    assert_eq!(only_monster(&s).type_name, "Goblin");
    assert_eq!(only_monster(&s).id, id);
}

#[test]
fn replacing_a_seated_boss_cannot_break_the_throne_reserve() {
    let mut s = dungeon();
    let king = get_monster_template("Goblin King").expect("king template");
    s.unlocked_monsters.push(king.name.clone());
    let room = &mut s.floors[0].rooms[1];
    room.room_type = RoomType::Boss;
    room.floor_number = 3;
    place_monster(&mut s, 1, 1, "Goblin").expect("guard fits");
    place_monster(&mut s, 1, 1, "Goblin King").expect("king fits");
    let king_id = s.floors[0].rooms[1].monsters[1].id;

    assert!(plan_swap(&s, 1, 1, king_id, "Goblin").is_none());
    assert!(swap_monster(&mut s, 1, 1, king_id, "Goblin").is_err());
    assert_eq!(s.floors[0].rooms[1].monsters[1].type_name, "Goblin King");
}

#[test]
fn defenders_cannot_be_restructured_mid_raid() {
    let mut s = dungeon();
    place_monster(&mut s, 1, 1, "Goblin").expect("placed");
    let id = only_monster(&s).id;
    // Upgrading a wounded defender mid-fight would heal it to full for free.
    s.floors[0].rooms[1].monsters[0].hp = 1;
    s.adventurer_parties.push(raiding_party());

    assert!(swap_monster(&mut s, 1, 1, id, "Orc").is_err());
    assert_eq!(only_monster(&s).hp, 1, "still wounded");
}
