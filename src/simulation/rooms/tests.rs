use super::*;

#[test]
fn a_branch_rejoins_the_route_without_orphaning_the_floor() {
    let mut state = GameState::new();
    state.mana = 1_000;
    add_room(&mut state, None).unwrap();
    branch_from(&mut state, 1, 0).unwrap();
    let floor = &state.floors[0];
    let entrance = floor.room_at(0).unwrap();
    assert_eq!(entrance.exits.len(), 2);
    let branch = floor.room_at(*entrance.exits.last().unwrap()).unwrap();
    assert_eq!(branch.exits, vec![1]);
    assert!(floor.validate_graph().is_ok());
}

#[test]
fn a_fork_cannot_be_forked_again_without_a_single_successor() {
    let mut state = GameState::new();
    state.mana = 1_000;
    branch_from(&mut state, 1, 0).unwrap();
    assert!(branch_from(&mut state, 1, 0).is_err());
}

#[test]
fn building_after_a_branch_keeps_both_routes_connected() {
    let mut state = GameState::new();
    state.mana = 1_000;
    branch_from(&mut state, 1, 0).unwrap();
    add_room(&mut state, None).unwrap();
    let floor = &state.floors[0];
    assert_eq!(floor.room_at(0).unwrap().exits.len(), 2);
    assert!(floor.validate_graph().is_ok());
}

#[test]
fn moving_the_core_to_a_new_floor_removes_the_old_sink_edge() {
    let mut state = GameState::new();
    state.mana = 1_000_000;
    while state.total_floors < 3 {
        add_room(&mut state, None).unwrap();
    }

    for floor in state.floors.iter().filter(|floor| !floor.is_deepest) {
        assert!(floor.rooms.iter().all(|room| {
            room.exits
                .iter()
                .all(|exit| floor.rooms.iter().any(|target| target.position == *exit))
        }));
        let last = floor.rooms.iter().max_by_key(|room| room.position).unwrap();
        assert!(last.exits.is_empty());
    }
}

#[test]
fn battle_orders_are_persistent_room_rules_with_real_tradeoffs() {
    let mut room = Room::new(1, RoomType::Normal, 1, 1);
    assert_eq!(room.battle_order, RoomBattleOrder::Balanced);

    room.battle_order = RoomBattleOrder::HoldLine;
    assert!(room.defender_attack_multiplier() < 1.0);
    assert!(room.defender_damage_taken_multiplier() < 1.0);

    room.battle_order = RoomBattleOrder::CullWounded;
    assert_eq!(room.defender_attack_multiplier(), 1.0);
    assert!(room.defender_damage_taken_multiplier() > 1.0);

    room.battle_order = RoomBattleOrder::BreakFormation;
    assert!(room.defender_attack_multiplier() > 1.0);
    assert!(room.defender_damage_taken_multiplier() > 1.0);
}

#[test]
fn old_rooms_restore_to_the_balanced_order() {
    let room = Room::new(1, RoomType::Normal, 1, 1);
    let mut value = serde_json::to_value(room).unwrap();
    value.as_object_mut().unwrap().remove("battle_order");
    let restored: Room = serde_json::from_value(value).unwrap();
    assert_eq!(restored.battle_order, RoomBattleOrder::Balanced);
}

#[test]
fn a_keeper_can_issue_orders_only_between_raids() {
    let mut state = GameState::new();
    state.mana = 1_000;
    add_room(&mut state, None).unwrap();
    set_battle_order(&mut state, 1, 1, RoomBattleOrder::HoldLine).unwrap();
    assert_eq!(
        state.floors[0].room_at(1).unwrap().battle_order,
        RoomBattleOrder::HoldLine
    );

    state
        .adventurer_parties
        .push(crate::game_state::AdventurerParty {
            id: 9,
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
            move_anim: macroquad_toolkit::timing::Cooldown::new(
                crate::game_state::PARTY_MOVE_SECONDS,
            ),
        });
    assert!(set_battle_order(&mut state, 1, 1, RoomBattleOrder::CullWounded).is_err());
    assert_eq!(
        state.floors[0].room_at(1).unwrap().battle_order,
        RoomBattleOrder::HoldLine
    );
}
