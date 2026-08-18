use super::*;

#[test]
fn keyboard_navigation_follows_edges_and_defaults_to_entrance() {
    let mut state = GameState::new();
    assert_eq!(
        keyboard_room_selection(&state, RoomNavigation::Right),
        Some((1, 1))
    );

    state.selected_room = Some((1, 1));
    assert_eq!(
        keyboard_room_selection(&state, RoomNavigation::Left),
        Some((1, 0))
    );
}

#[test]
fn adding_a_room_expands_the_world_without_rescaling_existing_rooms() {
    let mut state = GameState::new();
    state.mana = 10_000;
    let before = floor_world_width(&state.floors[0], None, 1.0);

    crate::simulation::rooms::add_room(&mut state, Some(1)).expect("room should be affordable");

    let after = floor_world_width(&state.floors[0], None, 1.0);
    assert_eq!(after - before, BASE_ROOM_W + BASE_CONNECTOR_W);
}

#[test]
fn a_parallel_branch_claims_an_additional_vertical_room_band() {
    let mut state = GameState::new();
    state.mana = 10_000;

    crate::simulation::rooms::branch_from(&mut state, 1, 0)
        .expect("entrance should support a parallel route");

    assert_eq!(
        floor_world_height(&state.floors[0], 1.0),
        BASE_ROOM_H * 2.0 + BASE_SLAB_H
    );
}
