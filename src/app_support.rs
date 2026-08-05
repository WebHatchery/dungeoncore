//! Application lifecycle state and helpers shared by title and play flows.

use macroquad::prelude::get_time;
use macroquad::prelude::Conf;
use macroquad_toolkit::capture;

use crate::data::difficulty::Difficulty;
use crate::game_state::GameState;

/// Env-var prefix for the screenshot capture harness (DUNGEON_CORE_CAPTURE_*).
pub const CAPTURE_PREFIX: &str = "DUNGEON_CORE";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppScreen {
    Title,
    SaveSlots,
    ConfirmSlotOverwrite,
    NewGameSetup,
    Settings,
    Playing,
}

pub fn window_conf() -> Conf {
    capture::capture_window_conf(CAPTURE_PREFIX, "Dungeon Core", 1280, 720)
}

pub fn create_new_game(difficulty: Difficulty, default_speed: i32) -> GameState {
    let mut state = GameState::new();
    state.difficulty = difficulty;
    let mult = difficulty.profile().core_hp_mult;
    state.core_max_hp = ((state.core_max_hp as f32 * mult).round() as i32).max(1);
    state.core_hp = state.core_max_hp;
    state.speed = default_speed.clamp(1, 4);
    state
}

pub fn reset_timers(
    last_time_advance: &mut f64,
    last_adventure_tick: &mut f64,
    last_save: &mut f64,
) {
    let now = get_time();
    *last_time_advance = now;
    *last_adventure_tick = now;
    *last_save = now;
}
