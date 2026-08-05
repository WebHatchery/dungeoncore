//! Dedicated controls page for rebinding the live dungeon shortcuts.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use crate::keybindings::{BindingAction, KeyBindings};

use super::theme::*;

pub enum KeybindingsScreenAction {
    None,
    Changed,
    Back,
}

pub fn draw_keybindings_screen(
    bindings: &mut KeyBindings,
    capturing: &mut Option<BindingAction>,
) -> KeybindingsScreenAction {
    let sw = screen_width();
    let sh = screen_height();
    clear_background(BG_DEEP);
    let card = Rect::new((sw - 530.0) * 0.5, (sh - 650.0) * 0.5, 530.0, 650.0);
    draw_panel(card, Some("Keyboard bindings"), ARCANE);
    draw_text_fit(
        "Click an action, then press a supported key. Reusing a key swaps the two actions.",
        card.x + 24.0,
        card.y + 48.0,
        card.w - 48.0,
        12.0,
        TEXT_MUTED,
    );

    if let Some(action) = *capturing {
        draw_text_fit(
            &format!("Press a key for {} — Esc cancels.", action.label()),
            card.x + 24.0,
            card.y + 70.0,
            card.w - 48.0,
            13.0,
            SOUL,
        );
        if is_key_pressed(KeyCode::Escape) {
            *capturing = None;
        } else if let Some(key) = get_last_key_pressed() {
            if KeyBindings::supports(key) {
                bindings.assign(action, key);
                *capturing = None;
                return KeybindingsScreenAction::Changed;
            }
        }
    }

    let mut y = card.y + 96.0;
    for action in BindingAction::ALL {
        let row = Rect::new(card.x + 24.0, y, card.w - 48.0, 39.0);
        let capturing_this = *capturing == Some(action);
        draw_card(
            row,
            if capturing_this {
                with_alpha(SOUL, 0.16)
            } else {
                Color::new(0.02, 0.03, 0.05, 0.7)
            },
            if capturing_this { SOUL } else { BORDER_MUTED },
        );
        draw_text_fit(
            action.label(),
            row.x + 14.0,
            row.y + 25.0,
            row.w - 150.0,
            13.0,
            TEXT,
        );
        draw_pill(
            Rect::new(row.x + row.w - 106.0, row.y + 9.0, 92.0, 21.0),
            bindings.label(action),
            SOUL,
        );
        if row.contains(vec2(mouse_position().0, mouse_position().1))
            && is_mouse_button_released(MouseButton::Left)
        {
            *capturing = Some(action);
        }
        y += 43.0;
    }
    let reset = Rect::new(card.x + 24.0, card.y + card.h - 92.0, 220.0, 38.0);
    if draw_command_button(reset, "Restore defaults", ButtonTone::Ghost, true) {
        bindings.reset();
        *capturing = None;
        return KeybindingsScreenAction::Changed;
    }
    let back = Rect::new(card.x + card.w - 244.0, card.y + card.h - 92.0, 220.0, 38.0);
    if draw_command_button(back, "Back", ButtonTone::Primary, true)
        || is_key_pressed(KeyCode::Escape)
    {
        *capturing = None;
        return KeybindingsScreenAction::Back;
    }
    KeybindingsScreenAction::None
}
