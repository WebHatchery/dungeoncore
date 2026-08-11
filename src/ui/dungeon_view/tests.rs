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
