//! Cross-platform save slots, with a one-time migration from the original
//! single-key save and explicit quarantine for unreadable player data.

use crate::game_state::GameState;
use macroquad_toolkit::persistence::{
    json_key_exists, load_json_key, quarantine_slot, save_to_slot_with_version, slot_exists,
};

const LEGACY_SAVE_FILE: &str = "dungeon_core_save.json";
const GAME_NAME: &str = "dungeon_core";
const SAVE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_SLOT: &str = "slot_1";
pub const SAVE_SLOTS: [&str; 3] = ["slot_1", "slot_2", "slot_3"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotState {
    Empty,
    Ready { day: i32, difficulty: String },
    Corrupt,
}

pub fn save_game(slot: &str, state: &GameState) -> Result<(), String> {
    save_to_slot_with_version(GAME_NAME, slot, state, SAVE_VERSION)
}

pub fn load_game(slot: &str) -> Result<GameState, String> {
    let mut state: GameState = macroquad_toolkit::persistence::load_from_slot(GAME_NAME, slot)?;
    state.migrate();
    Ok(state)
}

pub fn slot_state(slot: &str) -> SlotState {
    if !slot_exists(GAME_NAME, slot) {
        return SlotState::Empty;
    }
    match load_game(slot) {
        Ok(state) => SlotState::Ready {
            day: state.day,
            difficulty: state.difficulty.profile().name.to_string(),
        },
        Err(_) => SlotState::Corrupt,
    }
}

pub fn recover_corrupt_slot(slot: &str) -> Result<(), String> {
    quarantine_slot(GAME_NAME, slot).map(|_| ())
}

/// Move a readable pre-slot save into Slot 1 once. Corrupt legacy bytes remain
/// untouched, so a failed migration never turns into silent data loss.
pub fn migrate_legacy_save() -> Result<bool, String> {
    if slot_exists(GAME_NAME, DEFAULT_SLOT) || !json_key_exists(GAME_NAME, LEGACY_SAVE_FILE) {
        return Ok(false);
    }
    let mut state: GameState = load_json_key(GAME_NAME, LEGACY_SAVE_FILE)?;
    state.migrate();
    save_game(DEFAULT_SLOT, &state)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_names_are_distinct_and_include_the_default() {
        assert!(SAVE_SLOTS.contains(&DEFAULT_SLOT));
        assert_eq!(SAVE_SLOTS.len(), 3);
        assert_ne!(SAVE_SLOTS[0], SAVE_SLOTS[1]);
        assert_ne!(SAVE_SLOTS[1], SAVE_SLOTS[2]);
    }
}
