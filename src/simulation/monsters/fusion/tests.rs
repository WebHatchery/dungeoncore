use super::*;

fn fusion_dungeon() -> GameState {
    let mut state = GameState::new();
    state.mana = 100_000;
    crate::simulation::rooms::add_room(&mut state, None).unwrap();
    state.floors[0].room_at_mut(1).unwrap().floor_number = 3;
    let goblin = get_monster_template("Goblin").unwrap();
    state.unlocked_species.push(goblin.species);
    state.unlocked_monsters.push(goblin.name);
    state
}

fn place_pair(state: &mut GameState) {
    super::super::place_monster(state, 1, 1, "Goblin").unwrap();
    super::super::place_monster(state, 1, 1, "Goblin").unwrap();
}

#[test]
fn same_element_defenders_can_awaken_a_resonance_instead_of_a_rank() {
    let mut state = fusion_dungeon();
    state.unlocked_species.push("Construct".to_string());
    state.unlocked_monsters.push("Clay Golem".to_string());
    super::super::place_monster(&mut state, 1, 1, "Goblin").unwrap();
    super::super::place_monster(&mut state, 1, 1, "Clay Golem").unwrap();
    let primary = state.floors[0].room_at(1).unwrap().monsters[0].id;
    assert!(matches!(
        fusion_plan(state.floors[0].room_at(1).unwrap(), primary),
        Some(FusionPlan::Resonance(_))
    ));

    merge_monsters(&mut state, 1, 1, primary).unwrap();
    let monster = &state.floors[0].room_at(1).unwrap().monsters[0];
    assert_eq!(monster.fusion_rank, 1);
    assert!(monster
        .active_traits
        .iter()
        .any(|trait_data| trait_data.id == "resonance_strike"));
    assert!(state
        .log
        .iter()
        .any(|entry| entry.message.contains("Resonance")));
}

#[test]
fn identical_rank_one_defenders_fuse_and_free_a_slot() {
    let mut state = fusion_dungeon();
    place_pair(&mut state);
    let id = state.floors[0].room_at(1).unwrap().monsters[0].id;
    let base_attack = state.floors[0].room_at(1).unwrap().monsters[0]
        .scaled_stats
        .attack;

    merge_monsters(&mut state, 1, 1, id).unwrap();

    let room = state.floors[0].room_at(1).unwrap();
    assert_eq!(room.monsters.len(), 1);
    assert_eq!(room.monsters[0].id, id, "primary identity survives");
    assert_eq!(room.monsters[0].fusion_rank, 2);
    assert!(room.monsters[0].scaled_stats.attack > base_attack);
    assert!(room.monsters[0].scaled_stats.attack < base_attack * 2);
}

#[test]
fn four_basic_defenders_can_compress_into_one_rank_three_veteran() {
    let mut state = fusion_dungeon();
    // The floor-1 room normally has only two slots, so build two rank-II
    // veterans in sequence, then seat them together for the final fusion.
    place_pair(&mut state);
    let first = state.floors[0].room_at(1).unwrap().monsters[0].id;
    merge_monsters(&mut state, 1, 1, first).unwrap();
    place_pair(&mut state);
    let rank_one = state.floors[0]
        .room_at(1)
        .unwrap()
        .monsters
        .iter()
        .find(|monster| monster.fusion_rank == 1)
        .unwrap()
        .id;
    merge_monsters(&mut state, 1, 1, rank_one).unwrap();
    let rank_two_ids: Vec<u64> = state.floors[0]
        .room_at(1)
        .unwrap()
        .monsters
        .iter()
        .map(|monster| monster.id)
        .collect();
    merge_monsters(&mut state, 1, 1, rank_two_ids[0]).unwrap();

    let room = state.floors[0].room_at(1).unwrap();
    assert_eq!(room.monsters.len(), 1);
    assert_eq!(room.monsters[0].fusion_rank, 3);
    assert_eq!(fusion_target_rank(room, room.monsters[0].id), None);
}

#[test]
fn unequal_ranks_cannot_fuse() {
    let mut state = fusion_dungeon();
    place_pair(&mut state);
    state.floors[0].room_at_mut(1).unwrap().monsters[0].fusion_rank = 2;
    let id = state.floors[0].room_at(1).unwrap().monsters[0].id;
    assert!(merge_monsters(&mut state, 1, 1, id).is_err());
}

#[test]
fn fusion_is_locked_during_a_raid() {
    let mut state = fusion_dungeon();
    place_pair(&mut state);
    let id = state.floors[0].room_at(1).unwrap().monsters[0].id;
    state
        .adventurer_parties
        .push(crate::game_state::AdventurerParty {
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
            move_anim: macroquad_toolkit::timing::Cooldown::new(
                crate::game_state::PARTY_MOVE_SECONDS,
            ),
        });

    let error = merge_monsters(&mut state, 1, 1, id).unwrap_err();
    assert!(error.contains("adventurers"));
    assert_eq!(state.floors[0].room_at(1).unwrap().monsters.len(), 2);
}

#[test]
fn legacy_monsters_default_to_rank_one() {
    let mut state = fusion_dungeon();
    place_pair(&mut state);
    let monster = &state.floors[0].room_at(1).unwrap().monsters[0];
    let mut value = serde_json::to_value(monster).unwrap();
    value.as_object_mut().unwrap().remove("fusion_rank");

    let restored: crate::game_state::Monster = serde_json::from_value(value).unwrap();
    assert_eq!(restored.fusion_rank, 1);
}
