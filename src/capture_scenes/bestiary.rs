//! Deep-room bestiary and touch-pagination capture fixture.

use crate::game_state::{DungeonStatus, GameState, RoomType};
use crate::simulation;

pub(super) fn seed(state: &mut GameState) {
    state.mana = 9_999;
    state.max_mana = 9_999;
    state.gold = 9_999;
    state.tutorial_active = false;

    for species in ["Draconic", "Slime", "Undead"] {
        let _ = simulation::unlock_species(state, species);
    }
    for name in ["Storm Drake", "Pyre Ooze", "Death Tide", "Astral Gel"] {
        if !state
            .unlocked_monsters
            .iter()
            .any(|unlocked| unlocked == name)
        {
            state.unlocked_monsters.push(name.to_string());
        }
    }

    while state.total_floors < 4 {
        if simulation::add_room(state, None).is_err() {
            break;
        }
    }
    let target = state
        .floors
        .iter()
        .find(|floor| floor.number == 4)
        .and_then(|floor| {
            floor
                .rooms
                .iter()
                .find(|room| room.room_type == RoomType::Normal)
                .map(|room| (floor.number, room.position))
        });
    if let Some((floor, position)) = target {
        for name in ["Storm Drake", "Pyre Ooze", "Death Tide", "Astral Gel"] {
            let _ = simulation::place_monster(state, floor, position, name);
        }
        state.selected_room = Some((floor, position));
    }
    state.board_zoom = 0.82;
    state.board_scroll = f32::MAX;
    state.status = DungeonStatus::Closed;
}
