//! Deterministic 20-floor board capture fixture.

use crate::game_state::{DungeonStatus, GameState};
use crate::simulation;

pub(super) fn seed(state: &mut GameState) {
    // Exercise the real build path rather than a handwritten fake layout, then
    // begin the capture at the deep end of the bounded board viewport.
    state.mana = 1_000_000;
    state.max_mana = 1_000_000;
    if let Some(species) = crate::data::monsters::get_all_species()
        .into_iter()
        .find(|species| species.starter)
        .map(|species| species.name)
    {
        let _ = simulation::unlock_species(state, &species);
    }
    state.tutorial_active = false;
    while state.total_floors < 20 {
        if simulation::add_room(state, None).is_err() {
            break;
        }
    }
    state.selected_room = Some((state.total_floors, 0));
    state.board_zoom = 0.70;
    state.board_scroll = f32::MAX;
    state.status = DungeonStatus::Closed;
}
