use macroquad::prelude::*;
use macroquad_toolkit::{colors::dark, ui::*};

use crate::game_state::GameState;
use macroquad_toolkit::ui::draw_ui_text;

#[derive(Debug, Clone, PartialEq)]
pub struct ResourcePanelData {
    pub mana_label: String,
    pub mana_fraction: f32,
    pub regen_label: String,
    pub gold_label: String,
    pub souls_label: String,
}

/// Compute the panel's player-facing values without drawing. Keeping the cap
/// clamp here prevents a malformed or migrated state from overflowing a bar.
pub fn resource_panel_data(state: &GameState) -> ResourcePanelData {
    let mana_fraction = if state.max_mana > 0 {
        (state.mana as f32 / state.max_mana as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ResourcePanelData {
        mana_label: format!("{}/{}", state.mana, state.max_mana),
        mana_fraction,
        regen_label: format!("(+{:.1}/tick)", state.mana_regen),
        gold_label: format!("💰 Gold: {}", state.gold),
        souls_label: format!("👻 Souls: {}", state.souls),
    }
}

/// Draw the resource panel showing mana, gold, souls
pub fn draw_resource_panel(state: &GameState, x: f32, y: f32, w: f32) {
    let data = resource_panel_data(state);
    let h = 130.0;
    panel(x, y, w, h, Some("Resources"));

    let inner_x = x + 10.0;
    let inner_w = w - 20.0;
    let text_size = 16.0;

    // Mana bar
    draw_ui_text("Mana", inner_x, y + 38.0, text_size, dark::TEXT);
    progress_bar(
        inner_x,
        y + 45.0,
        inner_w,
        20.0,
        data.mana_fraction,
        1.0,
        Color::from_hex(0x2E86AB),
    );
    draw_ui_text(
        &data.mana_label,
        inner_x + inner_w - 60.0,
        y + 60.0,
        14.0,
        dark::TEXT_BRIGHT,
    );
    draw_ui_text(&data.regen_label, inner_x, y + 72.0, 12.0, dark::TEXT_DIM);

    // Gold
    draw_ui_text(
        &data.gold_label,
        inner_x,
        y + 95.0,
        18.0,
        Color::from_hex(0xF4D03F),
    );

    // Souls
    draw_ui_text(
        &data.souls_label,
        inner_x,
        y + 118.0,
        18.0,
        Color::from_hex(0x9B59B6),
    );
}

/// Draw time display
pub fn draw_time_display(state: &GameState, x: f32, y: f32) {
    draw_ui_text(
        &format!("Day {} - {:02}:00", state.day, state.hour),
        x,
        y,
        24.0,
        dark::TEXT_BRIGHT,
    );

    // Speed indicator
    let speed_text = format!("{}x", state.speed);
    let speed_color = match state.speed {
        1 => Color::from_hex(0x27AE60),
        2 => Color::from_hex(0xF39C12),
        _ => Color::from_hex(0xE74C3C),
    };
    draw_ui_text(&speed_text, x + 180.0, y, 24.0, speed_color);
}

#[cfg(test)]
mod tests;
