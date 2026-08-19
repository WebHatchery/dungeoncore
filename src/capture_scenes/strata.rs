//! One representative floor from every authored stratum, composed for a
//! single deterministic visual-verification capture.

use crate::game_state::{DungeonStatus, Floor, GameState, Room, RoomType};

pub(super) fn seed(state: &mut GameState) {
    if let Some(species) = crate::data::monsters::get_all_species()
        .into_iter()
        .find(|species| species.starter)
    {
        let _ = crate::simulation::unlock_species(state, &species.name);
    }
    let representative_floors = [1, 5, 9, 13, 17];
    state.floors = representative_floors
        .into_iter()
        .enumerate()
        .map(|(index, number)| {
            let mut floor = Floor::new(10_000 + index as u64, number, false);
            floor.rooms.push(Room::new(
                20_000 + index as u64 * 10,
                RoomType::Entrance,
                0,
                number,
            ));
            floor.rooms.push(Room::new(
                20_001 + index as u64 * 10,
                RoomType::Normal,
                1,
                number,
            ));
            floor.rooms.push(Room::new(
                20_002 + index as u64 * 10,
                RoomType::Core,
                2,
                number,
            ));
            floor.rebuild_linear_exits();
            floor
        })
        .collect();
    state.total_floors = 17;
    state.deep_core_bonus = 1.7;
    state.board_zoom = 0.38;
    state.board_scroll = 0.0;
    state.board_pan_x = 0.0;
    state.selected_room = None;
    state.tutorial_active = false;
    state.status = DungeonStatus::Closed;
    state.mana = 500;
    state.max_mana = 1_000;
}
