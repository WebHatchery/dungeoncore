//! Cross-platform save slots, with a one-time migration from the original
//! single-key save and explicit quarantine for unreadable player data.

use crate::game_state::GameState;
use macroquad_toolkit::persistence::{
    json_key_exists, load_from_slot_with_migration, load_json_key, quarantine_slot,
    save_to_slot_with_version, slot_exists,
};
use serde_json::Value;

const LEGACY_SAVE_FILE: &str = "dungeon_core_save.json";
const GAME_NAME: &str = "dungeon_core";
const SAVE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_SLOT: &str = "slot_1";
pub const SAVE_SLOTS: [&str; 3] = ["slot_1", "slot_2", "slot_3"];

/// A named, idempotent save-state migration. Entries stay in chronological
/// order so adding a later schema step never turns migration history into an
/// opaque catch-all method.
struct SaveMigration {
    name: &'static str,
    apply: fn(&mut GameState),
}

const SAVE_MIGRATIONS: &[SaveMigration] = &[SaveMigration {
    name: "room upgrades and graph exits",
    apply: GameState::migrate,
}];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotState {
    Empty,
    Ready {
        day: i32,
        difficulty: String,
        deepest_floor: i32,
        prestige: i32,
        dungeon_open: bool,
    },
    Corrupt,
}

pub fn save_game(slot: &str, state: &GameState) -> Result<(), String> {
    save_to_slot_with_version(GAME_NAME, slot, state, SAVE_VERSION)
}

pub fn load_game(slot: &str) -> Result<GameState, String> {
    let mut state = load_from_slot_with_migration(GAME_NAME, slot, SAVE_VERSION, migrate_slot)?;
    apply_save_migrations(&mut state);
    Ok(state)
}

/// Decode an older slot wrapper. The shared toolkit owns wrapper validation;
/// this game owns only the version-independent migration registry for its data.
fn migrate_slot(_version: Option<String>, value: Value) -> Result<GameState, String> {
    let data = value
        .get("data")
        .cloned()
        .ok_or_else(|| "Save slot is missing its data payload.".to_string())?;
    serde_json::from_value(data).map_err(|error| format!("Save data is invalid: {error}"))
}

fn apply_save_migrations(state: &mut GameState) {
    for migration in SAVE_MIGRATIONS {
        (migration.apply)(state);
    }
}

pub fn slot_state(slot: &str) -> SlotState {
    if !slot_exists(GAME_NAME, slot) {
        return SlotState::Empty;
    }
    match load_game(slot) {
        Ok(state) => ready_slot_state(&state),
        Err(_) => SlotState::Corrupt,
    }
}

fn ready_slot_state(state: &GameState) -> SlotState {
    SlotState::Ready {
        day: state.day,
        difficulty: state.difficulty.profile().name.to_string(),
        deepest_floor: state.total_floors,
        prestige: state.prestige,
        dungeon_open: state.status == crate::game_state::DungeonStatus::Open,
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
    apply_save_migrations(&mut state);
    save_game(DEFAULT_SLOT, &state)?;
    Ok(true)
}

#[cfg(test)]
mod tests;
