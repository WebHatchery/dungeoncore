//! The Goals overlay: the milestone/achievement track, giving the run a shaped
//! destination. Shows the current prestige rank and every milestone grouped by
//! tier with an achieved/locked state. Opened with [K].

use macroquad::prelude::*;

use crate::game_state::GameState;
use crate::simulation::milestones::{achieved_count, prestige_rank, MILESTONES};

use super::theme::*;
use super::upgrade_panel::draw_close_button;
use macroquad_toolkit::colors::with_alpha;

/// Draw the milestones overlay; returns true when it should close.
pub fn draw_milestones(state: &GameState, sw: f32, sh: f32, scroll: &mut f32) -> bool {
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.82));
    let w = (sw - 120.0).min(760.0);
    let h = (sh - 80.0).min(620.0);
    let x = (sw - w) / 2.0;
    let y = (sh - h) / 2.0;
    let panel = Rect::new(x, y, w, h);
    draw_panel(panel, Some("Goals"), ARCANE);

    let mut close = false;
    if draw_close_button(Rect::new(x + w - 48.0, y + 8.0, 40.0, 34.0))
        || is_key_pressed(KeyCode::K)
        || is_key_pressed(KeyCode::Escape)
    {
        close = true;
    }

    // Header: current rank + overall progress.
    let achieved = achieved_count(state);
    let total = MILESTONES.len();
    draw_text_fit(
        &format!(
            "Rank: {}  (Prestige {})    {} difficulty",
            prestige_rank(state.prestige),
            state.prestige,
            state.difficulty.name()
        ),
        x + 20.0,
        y + 36.0,
        w - 200.0,
        15.0,
        SOUL,
    );
    draw_text_fit(
        &format!("{achieved}/{total} milestones"),
        x + w - 160.0,
        y + 36.0,
        140.0,
        13.0,
        if achieved == total {
            EMERALD
        } else {
            TEXT_MUTED
        },
    );
    draw_bar(
        Rect::new(x + 20.0, y + 46.0, w - 40.0, 5.0),
        achieved as f32,
        total as f32,
        EMERALD,
        None,
    );

    // Scrollable list of milestones.
    let list = Rect::new(x + 16.0, y + 60.0, w - 32.0, h - 76.0);
    let row_h = 52.0;
    let content_h = MILESTONES.len() as f32 * row_h;
    let max_scroll = (content_h - list.h).max(0.0);
    let (_, wheel_y) = mouse_wheel();
    if list.contains(vec2(mouse_position().0, mouse_position().1)) && wheel_y != 0.0 {
        *scroll = (*scroll - wheel_y * 28.0).clamp(0.0, max_scroll);
    }
    *scroll = scroll.clamp(0.0, max_scroll);

    // Clip roughly by skipping rows fully outside the list rect.
    let mut ry = list.y - *scroll;
    for m in MILESTONES.iter() {
        if ry + row_h >= list.y && ry <= list.y + list.h {
            let owned = state.milestones.iter().any(|id| id == m.id);
            let accent = if owned { EMERALD } else { BORDER };
            let row = Rect::new(list.x, ry, list.w, row_h - 8.0);
            draw_card(
                row,
                with_alpha(accent, if owned { 0.12 } else { 0.04 }),
                with_alpha(accent, if owned { 0.5 } else { 0.24 }),
            );
            // Achieved marker: a filled diamond when earned, an outline when not
            // (drawn, so it never depends on font glyph coverage).
            let mc = vec2(row.x + 22.0, row.y + row.h * 0.5);
            if owned {
                draw_poly(mc.x, mc.y, 4, 8.0, 45.0, TREASURE);
            } else {
                draw_poly_lines(mc.x, mc.y, 4, 8.0, 45.0, 1.5, TEXT_DIM);
            }
            draw_text_fit(
                m.name,
                row.x + 42.0,
                row.y + 18.0,
                row.w - 130.0,
                14.0,
                if owned { TEXT } else { TEXT_DIM },
            );
            draw_text_fit(
                m.description,
                row.x + 42.0,
                row.y + 35.0,
                row.w - 130.0,
                10.5,
                if owned { TEXT_MUTED } else { TEXT_DIM },
            );
            let (badge, badge_color) = if owned {
                ("ACHIEVED", EMERALD)
            } else {
                ("LOCKED", TEXT_DIM)
            };
            draw_pill(
                Rect::new(row.x + row.w - 84.0, row.y + 12.0, 72.0, 18.0),
                badge,
                badge_color,
            );
        }
        ry += row_h;
    }

    close
}
