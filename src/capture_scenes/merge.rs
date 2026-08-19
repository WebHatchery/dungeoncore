//! Defender-fusion capture fixture.

use crate::game_state::GameState;
use crate::simulation;

pub(super) fn seed(state: &mut GameState) {
    if let Some(species) = super::first_starter_species() {
        let _ = simulation::unlock_species(state, &species);
    }
    state.tutorial_active = false;
    state.mana = 400;
    let _ = simulation::add_room(state, None);
    if let Some((floor, pos)) = super::find_combat_room(state) {
        let _ = simulation::place_monster(state, floor, pos, "Goblin");
        let _ = simulation::place_monster(state, floor, pos, "Goblin");
        state.selected_room = Some((floor, pos));
    }
}
