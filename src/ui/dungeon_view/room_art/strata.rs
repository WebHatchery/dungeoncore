//! Procedural environmental art for the five authored dungeon strata. These
//! details live inside the chamber render so they scale with board zoom and do
//! not require a raster asset for every possible floor layout.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use crate::ui::theme::element_color;

pub(super) fn draw_stratum_art(wall: Rect, floor: i32, time: f32) {
    let stratum = crate::data::strata::stratum_for_floor(floor);
    let accent = element_color(&stratum.element);
    draw_rectangle(wall.x, wall.y, wall.w, wall.h, with_alpha(accent, 0.035));

    match stratum.id.as_str() {
        "rootways" => draw_rootways(wall, accent),
        "ember_faults" => draw_ember_faults(wall, accent, time),
        "drowned_hollows" => draw_drowned_hollows(wall, accent, time),
        "crystal_veins" => draw_crystal_veins(wall, accent),
        "grave_below" => draw_grave_below(wall, accent, time),
        _ => {}
    }
}

fn draw_rootways(wall: Rect, accent: Color) {
    for vine in 0..4 {
        let base_x = wall.x + wall.w * (0.12 + vine as f32 * 0.24);
        let mut previous = vec2(base_x, wall.y);
        for segment in 1..=8 {
            let y = wall.y + wall.h * segment as f32 / 8.0;
            let x = base_x + (segment as f32 * 1.7 + vine as f32).sin() * 8.0;
            let point = vec2(x, y);
            draw_line(
                previous.x,
                previous.y,
                point.x,
                point.y,
                2.0,
                with_alpha(accent, 0.24),
            );
            if segment % 3 == 0 {
                let side = if (segment + vine) % 2 == 0 { -1.0 } else { 1.0 };
                draw_ellipse(
                    point.x + side * 5.0,
                    point.y,
                    5.0,
                    2.5,
                    0.4 * side,
                    with_alpha(accent, 0.28),
                );
            }
            previous = point;
        }
    }
}

fn draw_ember_faults(wall: Rect, accent: Color, time: f32) {
    draw_rectangle(
        wall.x,
        wall.y + wall.h * 0.76,
        wall.w,
        wall.h * 0.24,
        with_alpha(accent, 0.055),
    );
    for crack in 0..6 {
        let x = wall.x + wall.w * (0.08 + crack as f32 * 0.17);
        let bottom = wall.y + wall.h - 2.0;
        let mid = vec2(x + ((crack % 2) as f32 - 0.5) * 13.0, bottom - 19.0);
        draw_line(x, bottom, mid.x, mid.y, 2.0, with_alpha(accent, 0.44));
        draw_line(
            mid.x,
            mid.y,
            x + 8.0,
            bottom - 31.0,
            1.0,
            with_alpha(accent, 0.30),
        );
    }
    for ember in 0..8 {
        let phase = time * (0.35 + ember as f32 * 0.025) + ember as f32 * 0.73;
        let x = wall.x + wall.w * (0.08 + ember as f32 * 0.115);
        let rise = (phase.fract() + 1.0).fract();
        let y = wall.y + wall.h * (0.84 - rise * 0.45);
        draw_circle(
            x,
            y,
            1.5 + (ember % 3) as f32 * 0.5,
            with_alpha(accent, 0.34),
        );
    }
}

fn draw_drowned_hollows(wall: Rect, accent: Color, time: f32) {
    let water_y = wall.y + wall.h * 0.67;
    draw_rectangle(
        wall.x,
        water_y,
        wall.w,
        wall.y + wall.h - water_y,
        with_alpha(accent, 0.075),
    );
    for wave in 0..3 {
        let y = water_y + wave as f32 * 10.0;
        let mut x = wall.x;
        while x < wall.x + wall.w {
            let next = (x + 18.0).min(wall.x + wall.w);
            let wobble = (x * 0.06 + time * 0.6 + wave as f32).sin() * 2.0;
            draw_line(
                x,
                y + wobble,
                next,
                y - wobble,
                1.0,
                with_alpha(accent, 0.26),
            );
            x = next;
        }
    }
    for bubble in 0..6 {
        let phase = (time * 0.08 + bubble as f32 * 0.19).fract();
        let x = wall.x + wall.w * (0.13 + bubble as f32 * 0.15);
        let y = wall.y + wall.h * (0.92 - phase * 0.52);
        draw_circle_lines(
            x,
            y,
            2.0 + (bubble % 2) as f32,
            1.0,
            with_alpha(accent, 0.30),
        );
    }
}

fn draw_crystal_veins(wall: Rect, accent: Color) {
    for crystal in 0..7 {
        let x = wall.x + wall.w * (0.08 + crystal as f32 * 0.14);
        let height = 12.0 + (crystal % 3) as f32 * 7.0;
        let base = wall.y + wall.h - 3.0;
        let half_w = 5.0 + (crystal % 2) as f32 * 2.0;
        draw_triangle(
            vec2(x - half_w, base),
            vec2(x + half_w, base),
            vec2(x, base - height),
            with_alpha(accent, 0.22),
        );
        draw_line(
            x,
            base - height,
            x,
            base - 2.0,
            1.0,
            with_alpha(WHITE, 0.18),
        );
    }
    draw_line(
        wall.x + wall.w * 0.12,
        wall.y + wall.h * 0.28,
        wall.x + wall.w * 0.88,
        wall.y + wall.h * 0.44,
        1.5,
        with_alpha(accent, 0.20),
    );
}

fn draw_grave_below(wall: Rect, accent: Color, time: f32) {
    for grave in 0..5 {
        let x = wall.x + wall.w * (0.12 + grave as f32 * 0.19);
        let y = wall.y + wall.h - 10.0;
        draw_line(x - 7.0, y, x + 7.0, y, 2.0, with_alpha(accent, 0.22));
        draw_line(
            x - 4.0,
            y - 5.0,
            x + 5.0,
            y + 2.0,
            2.0,
            with_alpha(accent, 0.20),
        );
        draw_circle(x - 5.0, y - 6.0, 2.5, with_alpha(accent, 0.24));
    }
    for wisp in 0..4 {
        let sway = (time * 0.45 + wisp as f32 * 1.4).sin();
        let x = wall.x + wall.w * (0.2 + wisp as f32 * 0.21) + sway * 7.0;
        let y = wall.y + wall.h * (0.28 + (wisp % 2) as f32 * 0.18);
        draw_circle(x, y, 7.0, with_alpha(accent, 0.045));
        draw_circle(x, y, 2.0, with_alpha(accent, 0.22));
    }
}
