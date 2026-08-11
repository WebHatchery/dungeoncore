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
