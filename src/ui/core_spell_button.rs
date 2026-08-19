//! The floating "Core Smite" action button — the player's one mid-raid lever.
//!
//! It only appears while there are invaders in the dungeon, so it reads as a
//! contextual raid action rather than a permanent control. It shows readiness,
//! its mana cost, and a recharge bar so the cooldown is legible at a glance.

use macroquad::prelude::*;
use macroquad_toolkit::input::is_hovered_rect;

use crate::game_state::GameState;
use crate::simulation::core_spell::{is_ready, smite_target, CORE_SMITE_MANA_COST};

use super::theme::*;
use macroquad_toolkit::colors::with_alpha;

/// Width/height of the Core Smite button.
pub const CORE_SPELL_BTN_W: f32 = 176.0;
pub const CORE_SPELL_BTN_H: f32 = 46.0;

/// Should the Core Smite lever be shown right now? Only when a smiteable party
/// is in the dungeon.
pub fn core_spell_visible(state: &GameState) -> bool {
    smite_target(state).is_some()
}

/// Draw the Core Smite button at `rect`. Returns true when the player clicks it
/// (regardless of readiness — the cast attempt surfaces its own feedback so a
/// premature click still teaches the cost/cooldown).
pub fn draw_core_spell_button(state: &GameState, rect: Rect) -> bool {
    let ready = is_ready(state);
    let affordable = state.mana >= CORE_SMITE_MANA_COST;
    let hovered = is_hovered_rect(rect);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
    let clicked = hovered && is_mouse_button_released(MouseButton::Left);

    // Purple arcane fill when live; dim slate when recharging or unaffordable.
    let live = ready && affordable;
    let base = if live {
        Color::new(0.34, 0.14, 0.56, 1.0)
    } else {
        Color::new(0.085, 0.075, 0.115, 1.0)
    };
    let fill = if pressed {
        Color::new(base.r * 0.72, base.g * 0.72, base.b * 0.72, base.a)
    } else if hovered && live {
        Color::new(
            (base.r * 1.22).min(1.0),
            (base.g * 1.22).min(1.0),
            (base.b * 1.22).min(1.0),
            base.a,
        )
    } else {
        base
    };
    let border = if live { SOUL } else { with_alpha(SOUL, 0.34) };
    draw_card(rect, fill, border);

    let text_color = if live {
        Color::new(0.96, 0.91, 1.0, 1.0)
    } else {
        TEXT_DIM
    };
    // A small drawn "spark" diamond stands in for a bolt glyph the font lacks.
    let spark = vec2(rect.x + 20.0, rect.y + 16.0);
    draw_poly(spark.x, spark.y, 4, 6.0, 45.0, text_color);
    draw_text_fit(
        "Core Smite",
        rect.x + 34.0,
        rect.y + 20.0,
        rect.w - 42.0,
        16.0,
        text_color,
    );

    // Second line: hotkey + mana cost when ready, or the recharge countdown.
    let subline = if !ready {
        format!(
            "Recharging  {:.0}s",
            state.core_smite_cooldown.remaining().ceil()
        )
    } else if !affordable {
        format!("Need {} mana", CORE_SMITE_MANA_COST)
    } else {
        format!("Cast · {} mana", CORE_SMITE_MANA_COST)
    };
    let sub_color = if !ready {
        with_alpha(SOUL, 0.9)
    } else if !affordable {
        DANGER
    } else {
        Color::new(0.80, 0.72, 0.95, 1.0)
    };
    draw_text_fit(
        &subline,
        rect.x + 12.0,
        rect.y + 37.0,
        rect.w - 20.0,
        11.5,
        sub_color,
    );

    // Recharge bar along the bottom edge while cooling down.
    if !ready {
        let filled = state.core_smite_cooldown.fraction_elapsed();
        draw_rectangle(
            rect.x + 2.0,
            rect.y + rect.h - 4.0,
            (rect.w - 4.0) * filled,
            2.5,
            SOUL,
        );
    }

    if hovered {
        crate::ui::draw_tooltip(
            "Core Smite spends mana to strike the deepest invading party. It can only fire while recharged.",
            vec2(rect.x, rect.y + rect.h),
        );
    }

    clicked
}
