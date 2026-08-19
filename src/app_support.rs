//! Application lifecycle state and helpers shared by title and play flows.

use macroquad::prelude::get_time;
use macroquad::prelude::Conf;
use macroquad_toolkit::capture;

use crate::data::difficulty::Difficulty;
use crate::game_state::GameState;
use crate::ui::{DRAWER_COLLAPSED_WIDTH, DRAWER_OPEN_WIDTH, SIDE_PANEL_WIDTH};

#[cfg(test)]
mod tests;

/// Env-var prefix for the screenshot capture harness (DUNGEON_CORE_CAPTURE_*).
pub const CAPTURE_PREFIX: &str = "DUNGEON_CORE";

/// A browser tab or native window can stop delivering frames while it is
/// hidden. When it resumes, treating the whole gap as live simulation would
/// turn a focus change into an unearned raid or even a lost core.
pub const SUSPENSION_FRAME_GAP: f32 = 0.5;

/// Whether a frame gap is large enough to be treated as a focus/visibility
/// suspension. Kept pure so the browser-safety rule is covered without a
/// window in tests.
pub fn should_pause_after_frame_gap(frame_seconds: f32) -> bool {
    frame_seconds.is_finite() && frame_seconds > SUSPENSION_FRAME_GAP
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppScreen {
    Title,
    SaveSlots,
    ConfirmSlotOverwrite,
    NewGameSetup,
    Settings,
    Keybindings,
    Playing,
}

pub fn window_conf() -> Conf {
    capture::capture_window_conf(CAPTURE_PREFIX, "Dungeon Core", 1280, 720)
}

pub fn create_new_game(difficulty: Difficulty, default_speed: i32) -> GameState {
    create_new_game_with_seed(
        difficulty,
        default_speed,
        macroquad_toolkit::rng::random_u64(),
    )
}

pub fn create_new_game_with_seed(
    difficulty: Difficulty,
    default_speed: i32,
    run_seed: u64,
) -> GameState {
    let mut state = GameState::new();
    state.run_seed = run_seed;
    state.run_rng = macroquad_toolkit::rng::SeededRng::new(run_seed);
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

/// Preserve board space on a narrower desktop while a room inspector is open.
pub fn responsive_drawer_width(has_inspector: bool, drawer_open: bool, screen_width: f32) -> f32 {
    if drawer_open && !(has_inspector && screen_width < 860.0) {
        SIDE_PANEL_WIDTH.min((screen_width * 0.27).clamp(296.0, DRAWER_OPEN_WIDTH))
    } else {
        DRAWER_COLLAPSED_WIDTH
    }
}
