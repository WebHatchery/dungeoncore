use macroquad::prelude::*;
use macroquad_toolkit::{colors::dark, ui::*};

use crate::game_state::{DungeonStatus, GameState};
use macroquad_toolkit::ui::draw_ui_text;

/// Control action returned from the controls panel
#[derive(Debug, Clone, PartialEq)]
pub enum ControlAction {
    None,
    ToggleSpeed,
    ToggleDungeon,
    RespawnMonsters,
    AddRoom,
    ResetGame,
}

/// Draw the controls panel
pub fn draw_controls(state: &GameState, x: f32, y: f32, w: f32) -> ControlAction {
    let mut action = ControlAction::None;
    let h = 140.0;

    panel(x, y, w, h, Some("Controls"));

    let btn_w = (w - 25.0) / 2.0;
    let btn_h = 28.0;
    let row1_y = y + 35.0;
    let row2_y = y + 70.0;
    let row3_y = y + 105.0;

    // Speed button
    let speed_color = match state.speed {
        1 => Color::from_hex(0x27AE60),
        2 => Color::from_hex(0xF39C12),
        _ => Color::from_hex(0xE74C3C),
    };
    let speed_style = ButtonStyle {
        normal: speed_color,
        ..ButtonStyle::default_dark()
    };
    if button_styled(
        x + 5.0,
        row1_y,
        btn_w,
        btn_h,
        &format!("{}x Speed", state.speed),
        &speed_style,
    ) {
        action = ControlAction::ToggleSpeed;
    }

    // Dungeon status button
    let (status_label, status_color) = match state.status {
        DungeonStatus::Open => ("Close Dungeon", Color::from_hex(0xE74C3C)),
        DungeonStatus::Closed => ("Open Dungeon", Color::from_hex(0x27AE60)),
        DungeonStatus::Closing => ("Closing...", Color::from_hex(0xF39C12)),
    };
    let status_style = ButtonStyle {
        normal: status_color,
        ..ButtonStyle::default_dark()
    };
    if button_styled(
        x + 10.0 + btn_w,
        row1_y,
        btn_w,
        btn_h,
        status_label,
        &status_style,
    ) {
        action = ControlAction::ToggleDungeon;
    }

    // Respawn button
    let can_respawn = state.adventurer_parties.is_empty();
    if can_respawn {
        if button(x + 5.0, row2_y, btn_w, btn_h, "Respawn All") {
            action = ControlAction::RespawnMonsters;
        }
    } else {
        let surface = SurfaceStyle::new(dark::PANEL);
        draw_surface(Rect::new(x + 5.0, row2_y, btn_w, btn_h), &surface);
        draw_ui_text("Respawn All", x + 15.0, row2_y + 18.0, 13.0, dark::TEXT_DIM);
    }

    // Reset game button
    let reset_style = ButtonStyle {
        normal: Color::from_hex(0x8B0000),
        ..ButtonStyle::default_dark()
    };
    if button_styled(x + 5.0, row3_y, w - 10.0, btn_h, "Reset Game", &reset_style) {
        action = ControlAction::ResetGame;
    }

    // Active parties indicator
    draw_ui_text(
        &format!("Parties: {}", state.adventurer_parties.len()),
        x + w - 80.0,
        y + 20.0,
        12.0,
        if state.adventurer_parties.is_empty() {
            dark::TEXT_DIM
        } else {
            Color::from_hex(0x3498DB)
        },
    );

    action
}
