//! Board surface, subterranean rock, floor markers, and cutaway backplates.

use macroquad::prelude::*;

use crate::ui::theme::*;
use macroquad_toolkit::colors::with_alpha;

pub(super) fn draw_board_surface(rect: Rect) {
    draw_card(
        rect,
        Color::new(0.018, 0.015, 0.014, 1.0),
        with_alpha(BORDER_MUTED, 0.20),
    );
    draw_cavern_backdrop(rect);
    let mut y = rect.y + 34.0;
    while y < rect.y + rect.h {
        draw_line(
            rect.x + 8.0,
            y,
            rect.x + rect.w - 8.0,
            y + 4.0,
            1.0,
            Color::new(0.42, 0.32, 0.23, 0.045),
        );
        y += 46.0;
    }
}

fn draw_cavern_backdrop(rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.055, 0.043, 0.037, 1.0),
    );

    // Rock teeth frame the playable cutaway without competing with it.
    let rock = Color::new(0.075, 0.060, 0.052, 0.74);
    let tooth_w = (rect.w / 12.0).max(28.0);
    for i in 0..12 {
        let x = rect.x + i as f32 * tooth_w;
        let depth = 12.0 + (i % 4) as f32 * 8.0;
        draw_triangle(
            vec2(x, rect.y),
            vec2(x + tooth_w * 0.48, rect.y + depth),
            vec2(x + tooth_w, rect.y),
            rock,
        );
        draw_triangle(
            vec2(x, rect.y + rect.h),
            vec2(x + tooth_w * 0.55, rect.y + rect.h - depth * 0.65),
            vec2(x + tooth_w, rect.y + rect.h),
            with_alpha(rock, 0.62),
        );
    }
}

pub(super) fn draw_room_route_backplate(rect: Rect, selected: bool, border: Color) {
    let rock = if selected {
        Color::new(0.075, 0.060, 0.050, 0.94)
    } else {
        Color::new(0.045, 0.038, 0.035, 0.90)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, rock);
    if !selected {
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            1.0,
            with_alpha(border, 0.13),
        );
    }
    let floor_y = rect.y + rect.h - 20.0;
    draw_rectangle(
        rect.x,
        floor_y,
        rect.w,
        20.0,
        Color::new(0.020, 0.018, 0.017, 0.86),
    );
    draw_line(
        rect.x,
        floor_y,
        rect.x + rect.w,
        floor_y,
        1.0,
        with_alpha(TREASURE, if selected { 0.20 } else { 0.07 }),
    );
}

pub(super) fn draw_floor_rail(rect: Rect, floor_num: i32, room_count: usize, deepest: bool) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.025, 0.025, 0.028, 0.96),
    );
    draw_rectangle(
        rect.x,
        rect.y,
        4.0,
        rect.h,
        Color::new(0.16, 0.14, 0.12, 0.92),
    );
    draw_rectangle(
        rect.x + rect.w - 4.0,
        rect.y,
        4.0,
        rect.h,
        Color::new(0.16, 0.14, 0.12, 0.92),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        if deepest {
            with_alpha(ARCANE, 0.50)
        } else {
            BORDER_MUTED
        },
    );
    draw_text_fit(
        &format!("FLOOR {floor_num}"),
        rect.x + 6.0,
        rect.y + 18.0,
        rect.w - 12.0,
        12.0,
        TEXT,
    );
    draw_centered_text(
        &format!("{room_count}R"),
        Rect::new(rect.x, rect.y + rect.h - 26.0, rect.w, 18.0),
        10.0,
        if deepest { SOUL } else { TEXT_MUTED },
    );
    if deepest {
        draw_circle(rect.x + rect.w * 0.5, rect.y + rect.h - 37.0, 3.0, SOUL);
    }
}
