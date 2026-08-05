//! Confirmation overlay for actions that cannot be safely undone.

use macroquad::prelude::*;

use crate::game_state::PendingConfirmation;

use super::theme::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmationChoice {
    None,
    Confirm,
    Cancel,
}

pub fn draw_confirmation_overlay(
    action: &PendingConfirmation,
    screen_w: f32,
    screen_h: f32,
) -> ConfirmationChoice {
    draw_rectangle(
        0.0,
        0.0,
        screen_w,
        screen_h,
        Color::new(0.0, 0.0, 0.0, 0.58),
    );
    let card = Rect::new(
        (screen_w - 360.0) * 0.5,
        (screen_h - 170.0) * 0.5,
        360.0,
        170.0,
    );
    draw_panel(card, Some("Confirm action"), DANGER);
    let (question, confirm_label) = match action {
        PendingConfirmation::ResetRun => (
            "Abandon this dungeon and start a fresh run? Your current save will be replaced.",
            "Reset run",
        ),
        PendingConfirmation::DismissMonster { .. } => (
            "Dismiss this defender for a partial mana refund? Its health and placement will be lost.",
            "Dismiss defender",
        ),
    };
    let mut y = card.y + 56.0;
    for line in macroquad_toolkit::ui::wrap_text(question, card.w - 28.0, 12.0) {
        draw_text_fit(&line, card.x + 14.0, y, card.w - 28.0, 12.0, TEXT_MUTED);
        y += 16.0;
    }
    if draw_command_button(
        Rect::new(card.x + 14.0, card.y + card.h - 42.0, 154.0, 28.0),
        "Cancel",
        ButtonTone::Ghost,
        true,
    ) {
        return ConfirmationChoice::Cancel;
    }
    if draw_command_button(
        Rect::new(card.x + card.w - 168.0, card.y + card.h - 42.0, 154.0, 28.0),
        confirm_label,
        ButtonTone::Danger,
        true,
    ) {
        return ConfirmationChoice::Confirm;
    }
    ConfirmationChoice::None
}
