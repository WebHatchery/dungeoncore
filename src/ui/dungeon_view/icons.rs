//! Low-level vector art: room-type glyphs, chamber decorations, borders.

use macroquad::prelude::*;

use crate::game_state::RoomType;
use crate::ui::theme::*;
use macroquad_toolkit::colors::with_alpha;

pub(super) fn draw_entrance_art(rect: Rect, color: Color) {
    let arch = Rect::new(
        rect.x + rect.w * 0.30,
        rect.y + rect.h * 0.22,
        rect.w * 0.40,
        rect.h * 0.64,
    );
    draw_rectangle(
        arch.x,
        arch.y + arch.h * 0.36,
        arch.w,
        arch.h * 0.64,
        Color::new(0.0, 0.0, 0.0, 0.38),
    );
    draw_triangle(
        vec2(arch.x, arch.y + arch.h * 0.38),
        vec2(arch.x + arch.w * 0.5, arch.y),
        vec2(arch.x + arch.w, arch.y + arch.h * 0.38),
        Color::new(0.0, 0.0, 0.0, 0.38),
    );
    let line = with_alpha(color, 0.52);
    draw_rectangle_lines(
        arch.x,
        arch.y + arch.h * 0.36,
        arch.w,
        arch.h * 0.64,
        2.0,
        line,
    );
    draw_line(
        arch.x,
        arch.y + arch.h * 0.38,
        arch.x + arch.w * 0.5,
        arch.y,
        2.0,
        line,
    );
    draw_line(
        arch.x + arch.w,
        arch.y + arch.h * 0.38,
        arch.x + arch.w * 0.5,
        arch.y,
        2.0,
        line,
    );
    draw_circle(
        arch.x + arch.w * 0.28,
        arch.y + arch.h * 0.70,
        4.0,
        with_alpha(EMERALD, 0.58),
    );
}

pub(super) fn draw_combat_art(rect: Rect, color: Color) {
    // A furnished guard chamber reads more like a place creatures inhabit than
    // a large abstract room icon. The small banner retains the room identity.
    let banner = Rect::new(
        rect.x + rect.w * 0.41,
        rect.y + rect.h * 0.28,
        rect.w * 0.18,
        rect.h * 0.24,
    );
    draw_rectangle(
        banner.x,
        banner.y,
        banner.w,
        banner.h,
        Color::new(0.08, 0.035, 0.025, 0.30),
    );
    draw_triangle(
        vec2(banner.x, banner.y + banner.h),
        vec2(banner.x + banner.w * 0.50, banner.y + banner.h * 0.72),
        vec2(banner.x + banner.w, banner.y + banner.h),
        Color::new(0.08, 0.035, 0.025, 0.30),
    );
    let c = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.41);
    draw_room_icon(&RoomType::Normal, c, rect.h * 0.09, with_alpha(color, 0.76));
    draw_torch(vec2(rect.x + rect.w * 0.17, rect.y + rect.h * 0.53));
    draw_torch(vec2(rect.x + rect.w * 0.83, rect.y + rect.h * 0.53));

    // Low weapon racks and supply crates leave the foreground open for units.
    let rack_y = rect.y + rect.h * 0.67;
    draw_line(
        rect.x + rect.w * 0.27,
        rack_y - 10.0,
        rect.x + rect.w * 0.27,
        rack_y + 9.0,
        2.0,
        with_alpha(color, 0.46),
    );
    draw_line(
        rect.x + rect.w * 0.34,
        rack_y - 10.0,
        rect.x + rect.w * 0.34,
        rack_y + 9.0,
        2.0,
        with_alpha(color, 0.46),
    );
    draw_line(
        rect.x + rect.w * 0.24,
        rack_y - 2.0,
        rect.x + rect.w * 0.37,
        rack_y - 2.0,
        1.5,
        with_alpha(color, 0.34),
    );
    let crate_rect = Rect::new(rect.x + rect.w * 0.68, rack_y - 7.0, 13.0, 14.0);
    draw_rectangle(
        crate_rect.x,
        crate_rect.y,
        crate_rect.w,
        crate_rect.h,
        Color::new(0.30, 0.19, 0.10, 0.78),
    );
    draw_rectangle_lines(
        crate_rect.x,
        crate_rect.y,
        crate_rect.w,
        crate_rect.h,
        1.0,
        with_alpha(TREASURE, 0.42),
    );
    draw_line(
        crate_rect.x,
        crate_rect.y,
        crate_rect.x + crate_rect.w,
        crate_rect.y + crate_rect.h,
        1.0,
        with_alpha(TREASURE, 0.28),
    );
}

pub(super) fn draw_core_art(rect: Rect, color: Color) {
    let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.50);
    draw_circle(center.x, center.y, rect.h * 0.36, with_alpha(SOUL, 0.13));
    draw_room_icon(&RoomType::Core, center, rect.h * 0.28, color);
    draw_line(
        center.x,
        rect.y + 10.0,
        center.x,
        rect.y + rect.h - 8.0,
        1.5,
        with_alpha(color, 0.24),
    );
}

fn draw_torch(pos: Vec2) {
    draw_circle(pos.x, pos.y - 9.0, 13.0, with_alpha(TREASURE, 0.09));
    draw_rectangle(
        pos.x - 2.0,
        pos.y - 8.0,
        4.0,
        22.0,
        Color::new(0.28, 0.18, 0.10, 0.90),
    );
    draw_circle(pos.x, pos.y - 9.0, 6.0, with_alpha(TREASURE, 0.30));
    draw_triangle(
        vec2(pos.x, pos.y - 18.0),
        vec2(pos.x - 5.0, pos.y - 5.0),
        vec2(pos.x + 5.0, pos.y - 5.0),
        TREASURE,
    );
}

pub(super) fn draw_room_icon(room_type: &RoomType, center: Vec2, size: f32, color: Color) {
    match room_type {
        RoomType::Entrance => {
            draw_rectangle_lines(
                center.x - size * 0.42,
                center.y - size * 0.48,
                size * 0.84,
                size * 0.96,
                3.0,
                color,
            );
            draw_line(
                center.x - size * 0.18,
                center.y + size * 0.40,
                center.x + size * 0.18,
                center.y + size * 0.40,
                3.0,
                color,
            );
            draw_circle(center.x + size * 0.20, center.y, size * 0.06, color);
        }
        RoomType::Normal => {
            draw_line(
                center.x - size * 0.50,
                center.y - size * 0.48,
                center.x + size * 0.48,
                center.y + size * 0.50,
                4.0,
                color,
            );
            draw_line(
                center.x + size * 0.50,
                center.y - size * 0.48,
                center.x - size * 0.48,
                center.y + size * 0.50,
                4.0,
                color,
            );
            draw_line(
                center.x - size * 0.64,
                center.y + size * 0.32,
                center.x - size * 0.30,
                center.y + size * 0.66,
                3.0,
                color,
            );
            draw_line(
                center.x + size * 0.64,
                center.y + size * 0.32,
                center.x + size * 0.30,
                center.y + size * 0.66,
                3.0,
                color,
            );
        }
        RoomType::Boss => {
            draw_circle_lines(center.x, center.y - size * 0.05, size * 0.44, 3.0, color);
            draw_circle(
                center.x - size * 0.17,
                center.y - size * 0.10,
                size * 0.07,
                color,
            );
            draw_circle(
                center.x + size * 0.17,
                center.y - size * 0.10,
                size * 0.07,
                color,
            );
            draw_line(
                center.x - size * 0.18,
                center.y + size * 0.20,
                center.x + size * 0.18,
                center.y + size * 0.20,
                3.0,
                color,
            );
            draw_line(
                center.x - size * 0.32,
                center.y - size * 0.48,
                center.x - size * 0.48,
                center.y - size * 0.70,
                3.0,
                color,
            );
            draw_line(
                center.x + size * 0.32,
                center.y - size * 0.48,
                center.x + size * 0.48,
                center.y - size * 0.70,
                3.0,
                color,
            );
        }
        RoomType::Core => {
            draw_triangle(
                vec2(center.x, center.y - size * 0.62),
                vec2(center.x - size * 0.48, center.y),
                vec2(center.x, center.y + size * 0.62),
                with_alpha(color, 0.28),
            );
            draw_triangle(
                vec2(center.x, center.y - size * 0.62),
                vec2(center.x + size * 0.48, center.y),
                vec2(center.x, center.y + size * 0.62),
                with_alpha(color, 0.28),
            );
            draw_line(
                center.x,
                center.y - size * 0.62,
                center.x - size * 0.48,
                center.y,
                3.0,
                color,
            );
            draw_line(
                center.x - size * 0.48,
                center.y,
                center.x,
                center.y + size * 0.62,
                3.0,
                color,
            );
            draw_line(
                center.x,
                center.y + size * 0.62,
                center.x + size * 0.48,
                center.y,
                3.0,
                color,
            );
            draw_line(
                center.x + size * 0.48,
                center.y,
                center.x,
                center.y - size * 0.62,
                3.0,
                color,
            );
            draw_line(
                center.x,
                center.y - size * 0.62,
                center.x,
                center.y + size * 0.62,
                2.0,
                color,
            );
        }
    }
}

pub(super) fn draw_dashed_border(rect: Rect, color: Color) {
    let dash = 7.0;
    let gap = 5.0;
    let mut x = rect.x + 3.0;
    while x < rect.x + rect.w - 3.0 {
        draw_line(
            x,
            rect.y + 3.0,
            (x + dash).min(rect.x + rect.w - 3.0),
            rect.y + 3.0,
            1.5,
            color,
        );
        draw_line(
            x,
            rect.y + rect.h - 3.0,
            (x + dash).min(rect.x + rect.w - 3.0),
            rect.y + rect.h - 3.0,
            1.5,
            color,
        );
        x += dash + gap;
    }
    let mut y = rect.y + 3.0;
    while y < rect.y + rect.h - 3.0 {
        draw_line(
            rect.x + 3.0,
            y,
            rect.x + 3.0,
            (y + dash).min(rect.y + rect.h - 3.0),
            1.5,
            color,
        );
        draw_line(
            rect.x + rect.w - 3.0,
            y,
            rect.x + rect.w - 3.0,
            (y + dash).min(rect.y + rect.h - 3.0),
            1.5,
            color,
        );
        y += dash + gap;
    }
}
