//! Excavated rock field, shared floor slabs, lift shafts, and viewport framing.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use crate::ui::theme::*;

pub(super) fn draw_board_surface(rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.055, 0.045, 0.040, 1.0),
    );

    let band_h = 42.0;
    let mut row = 0usize;
    let mut y = rect.y;
    while y < rect.y + rect.h {
        let shade = if row.is_multiple_of(2) {
            Color::new(0.080, 0.063, 0.052, 0.34)
        } else {
            Color::new(0.035, 0.030, 0.028, 0.30)
        };
        draw_rectangle(rect.x, y, rect.w, band_h, shade);
        draw_line(
            rect.x,
            y + band_h,
            rect.x + rect.w,
            y + band_h - 3.0,
            1.0,
            Color::new(0.24, 0.18, 0.13, 0.11),
        );
        y += band_h;
        row += 1;
    }

    // Deterministic seams keep capture tests stable while making the terrain
    // read as rock rather than unused canvas.
    let cell_w = 64.0;
    let cell_h = 48.0;
    let cols = (rect.w / cell_w).ceil() as usize + 1;
    let rows = (rect.h / cell_h).ceil() as usize + 1;
    for row in 0..rows {
        for col in 0..cols {
            let seed = (row * 17 + col * 31) as f32;
            let x = rect.x + col as f32 * cell_w + (seed * 0.73).sin() * 13.0;
            let y = rect.y + row as f32 * cell_h + (seed * 1.17).cos() * 9.0;
            let radius = 7.0 + (seed * 0.41).sin().abs() * 10.0;
            draw_circle(x, y, radius, Color::new(0.13, 0.10, 0.08, 0.12));
            draw_line(
                x - radius * 0.8,
                y + radius * 0.2,
                x + radius,
                y - radius * 0.25,
                1.0,
                Color::new(0.30, 0.22, 0.16, 0.10),
            );
        }
    }
}

pub(super) fn draw_floor_structure(rect: Rect, floor_num: i32, selected: bool) {
    let depth = ((floor_num - 1).max(0) as f32 / 19.0).clamp(0.0, 1.0);
    let stratum = crate::data::strata::stratum_for_floor(floor_num);
    let stratum_color = element_color(&stratum.element);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        with_alpha(stratum_color, 0.025),
    );
    let slab_h = 14.0_f32.min(rect.h * 0.12);
    let slab_y = rect.y + rect.h - slab_h;
    draw_rectangle(
        rect.x - 8.0,
        slab_y + 5.0,
        rect.w + 16.0,
        slab_h + 8.0,
        Color::new(0.0, 0.0, 0.0, 0.44),
    );
    draw_rectangle(
        rect.x - 5.0,
        slab_y,
        rect.w + 10.0,
        slab_h,
        Color::new(0.085 + depth * 0.03, 0.075, 0.070 + depth * 0.04, 1.0),
    );
    draw_line(
        rect.x - 4.0,
        slab_y,
        rect.x + rect.w + 4.0,
        slab_y,
        2.0,
        with_alpha(
            if selected { MANA } else { stratum_color },
            if selected { 0.48 } else { 0.18 },
        ),
    );
}

pub(super) fn draw_lift_shaft(rect: Rect, floor_num: i32, deepest: bool) {
    let stratum = crate::data::strata::stratum_for_floor(floor_num);
    let stratum_color = element_color(&stratum.element);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.025, 0.028, 0.034, 1.0),
    );
    draw_rectangle(
        rect.x,
        rect.y,
        5.0,
        rect.h,
        Color::new(0.16, 0.13, 0.11, 0.96),
    );
    draw_rectangle(
        rect.x + rect.w - 5.0,
        rect.y,
        5.0,
        rect.h,
        Color::new(0.16, 0.13, 0.11, 0.96),
    );
    let cable = with_alpha(if deepest { SOUL } else { TEXT_DIM }, 0.50);
    for x in [rect.x + rect.w * 0.36, rect.x + rect.w * 0.64] {
        draw_line(x, rect.y + 28.0, x, rect.y + rect.h - 14.0, 1.5, cable);
    }
    draw_centered_text(
        &format!("F{floor_num}"),
        Rect::new(rect.x + 5.0, rect.y + 7.0, rect.w - 10.0, 18.0),
        11.0,
        if deepest { SOUL } else { stratum_color },
    );
    draw_centered_text(
        if rect.w >= 44.0 {
            &stratum.name
        } else {
            stratum.short_label()
        },
        Rect::new(rect.x + 3.0, rect.y + 21.0, rect.w - 6.0, 12.0),
        7.0,
        with_alpha(stratum_color, 0.82),
    );
    let car = Rect::new(rect.x + 9.0, rect.y + rect.h * 0.52, rect.w - 18.0, 34.0);
    draw_rectangle(
        car.x,
        car.y,
        car.w,
        car.h,
        Color::new(0.07, 0.075, 0.085, 1.0),
    );
    draw_rectangle_lines(car.x, car.y, car.w, car.h, 1.0, with_alpha(TREASURE, 0.28));
    draw_line(
        car.x + car.w * 0.5,
        car.y + 3.0,
        car.x + car.w * 0.5,
        car.y + car.h - 3.0,
        1.0,
        BORDER_MUTED,
    );
}

pub(super) fn draw_lift_extension(rect: Rect) {
    if rect.h <= 0.0 {
        return;
    }
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.018, 0.020, 0.025, 0.96),
    );
    for x in [rect.x + 3.0, rect.x + rect.w - 6.0] {
        draw_rectangle(x, rect.y, 3.0, rect.h, Color::new(0.13, 0.11, 0.10, 0.92));
    }
    for x in [rect.x + rect.w * 0.36, rect.x + rect.w * 0.64] {
        draw_line(x, rect.y, x, rect.y + rect.h, 1.2, with_alpha(SOUL, 0.34));
    }
}

pub(super) fn draw_cutaway_frame(rect: Rect) {
    let edge = Color::new(0.025, 0.021, 0.019, 0.94);
    draw_rectangle(rect.x, rect.y, rect.w, 7.0, edge);
    draw_rectangle(rect.x, rect.y + rect.h - 7.0, rect.w, 7.0, edge);
    draw_rectangle(rect.x, rect.y, 7.0, rect.h, edge);
    draw_rectangle(rect.x + rect.w - 7.0, rect.y, 7.0, rect.h, edge);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        with_alpha(BORDER, 0.34),
    );
}

pub(super) fn begin_cutaway_clip(rect: Rect) {
    unsafe {
        macroquad::window::get_internal_gl().quad_gl.scissor(Some((
            rect.x.round() as i32,
            rect.y.round() as i32,
            rect.w.round() as i32,
            rect.h.round() as i32,
        )));
    }
}

pub(super) fn end_cutaway_clip() {
    unsafe {
        macroquad::window::get_internal_gl().quad_gl.scissor(None);
    }
}
