//! Title-screen save selection and the explicit corrupt-save recovery choice.

use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;

use crate::persistence::{SlotState, SAVE_SLOTS};

use super::theme::*;
use super::{
    draw_title_background, draw_title_button, draw_title_notice, draw_title_panel, ButtonTone,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveSlotAction {
    None,
    Load(&'static str),
    New(&'static str),
    Recover(&'static str),
    Back,
}

pub fn draw_save_slots_screen(
    assets: &AssetManager,
    states: &[SlotState; 3],
    notice: Option<&str>,
) -> SaveSlotAction {
    let sw = screen_width();
    let sh = screen_height();
    draw_title_background(assets, sw, sh);
    let panel = Rect::new((sw - 500.0) * 0.5, (sh - 510.0) * 0.5, 500.0, 510.0);
    draw_title_panel(panel);
    draw_text_fit(
        "SAVE VAULT",
        panel.x + 26.0,
        panel.y + 38.0,
        panel.w - 52.0,
        22.0,
        TEXT,
    );
    draw_text_fit(
        "Choose a dungeon to load or a vessel for a new reign.",
        panel.x + 26.0,
        panel.y + 62.0,
        panel.w - 52.0,
        12.0,
        TEXT_MUTED,
    );

    for (idx, slot) in SAVE_SLOTS.iter().enumerate() {
        let y = panel.y + 88.0 + idx as f32 * 106.0;
        let state = &states[idx];
        let (summary, detail, tone) = match state {
            SlotState::Empty => ("Empty".to_string(), None, TEXT_DIM),
            SlotState::Ready {
                day,
                difficulty,
                deepest_floor,
                prestige,
                dungeon_open,
            } => (
                format!("Day {day} · {difficulty}"),
                Some(format!(
                    "Floor {deepest_floor} · Prestige {prestige} · {}",
                    if *dungeon_open { "Open" } else { "Closed" }
                )),
                EMERALD,
            ),
            SlotState::Corrupt => ("Needs recovery".to_string(), None, DANGER),
        };
        draw_card(
            Rect::new(panel.x + 22.0, y, panel.w - 44.0, 92.0),
            Color::new(0.03, 0.04, 0.06, 0.84),
            tone,
        );
        draw_text_fit(
            &format!("Slot {}", idx + 1),
            panel.x + 38.0,
            y + 27.0,
            130.0,
            16.0,
            TEXT,
        );
        draw_text_fit(&summary, panel.x + 38.0, y + 49.0, 210.0, 12.0, tone);
        if let Some(detail) = detail {
            draw_text_fit(&detail, panel.x + 38.0, y + 69.0, 250.0, 10.0, TEXT_MUTED);
        }
        let button = Rect::new(panel.x + panel.w - 176.0, y + 24.0, 138.0, 38.0);
        match state {
            SlotState::Ready { .. } => {
                if draw_title_button(button, "Load", true, ButtonTone::Primary) {
                    return SaveSlotAction::Load(slot);
                }
                if draw_title_button(
                    Rect::new(button.x - 94.0, button.y, 84.0, button.h),
                    "New",
                    true,
                    ButtonTone::Ghost,
                ) {
                    return SaveSlotAction::New(slot);
                }
            }
            SlotState::Empty => {
                if draw_title_button(button, "New", true, ButtonTone::Arcane) {
                    return SaveSlotAction::New(slot);
                }
            }
            SlotState::Corrupt => {
                if draw_title_button(button, "Recover", true, ButtonTone::Danger) {
                    return SaveSlotAction::Recover(slot);
                }
            }
        }
    }
    if draw_title_button(
        Rect::new(
            panel.x + 22.0,
            panel.y + panel.h - 52.0,
            panel.w - 44.0,
            36.0,
        ),
        "Back",
        true,
        ButtonTone::Ghost,
    ) || is_key_pressed(KeyCode::Escape)
    {
        return SaveSlotAction::Back;
    }
    if let Some(message) = notice {
        draw_title_notice(message, sw, sh);
    }
    SaveSlotAction::None
}

pub fn draw_slot_overwrite_confirmation(assets: &AssetManager, slot: &str) -> Option<bool> {
    let sw = screen_width();
    let sh = screen_height();
    draw_title_background(assets, sw, sh);
    let card = Rect::new((sw - 390.0) * 0.5, (sh - 210.0) * 0.5, 390.0, 210.0);
    draw_title_panel(card);
    draw_text_fit(
        "REPLACE THIS DUNGEON?",
        card.x + 22.0,
        card.y + 38.0,
        card.w - 44.0,
        18.0,
        DANGER,
    );
    draw_text_fit(
        &format!("Starting a new run in {slot} replaces its current dungeon."),
        card.x + 22.0,
        card.y + 72.0,
        card.w - 44.0,
        12.0,
        TEXT_MUTED,
    );
    if draw_title_button(
        Rect::new(card.x + 22.0, card.y + 150.0, 160.0, 36.0),
        "Cancel",
        true,
        ButtonTone::Ghost,
    ) {
        return Some(false);
    }
    if draw_title_button(
        Rect::new(card.x + 208.0, card.y + 150.0, 160.0, 36.0),
        "Replace",
        true,
        ButtonTone::Danger,
    ) {
        return Some(true);
    }
    None
}
