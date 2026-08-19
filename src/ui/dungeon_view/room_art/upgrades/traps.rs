//! Trap props for installed room upgrades.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use crate::game_state::RoomUpgrade;
use crate::ui::theme::*;

use super::{prop_frame, UpgradeArtKey};

pub(super) fn draw_trap(rect: Rect, upgrade: &RoomUpgrade, key: UpgradeArtKey) {
    match key {
        UpgradeArtKey::SpikeTrap => draw_spike_trap(rect, upgrade),
        UpgradeArtKey::PoisonDart => draw_poison_dart(rect, upgrade),
        UpgradeArtKey::BoulderTrap => draw_boulder_trap(rect, upgrade),
        UpgradeArtKey::FlameVent => draw_flame_vent(rect, upgrade),
        UpgradeArtKey::FrostSnare => draw_frost_snare(rect, upgrade),
        UpgradeArtKey::AlarmGong => draw_alarm_gong(rect, upgrade),
        UpgradeArtKey::ManaSiphon => draw_mana_siphon(rect, upgrade),
        UpgradeArtKey::GoldSnatcher => draw_gold_snatcher(rect, upgrade),
        UpgradeArtKey::AbyssalMaw => draw_abyssal_maw(rect, upgrade),
        _ => {}
    }
}

fn trap_color(upgrade: &RoomUpgrade) -> Color {
    if upgrade.disarmed {
        TEXT_DIM
    } else {
        upgrade
            .element
            .as_deref()
            .map(element_color)
            .unwrap_or(WARNING)
    }
}

fn draw_spike_trap(rect: Rect, upgrade: &RoomUpgrade) {
    let color = trap_color(upgrade);
    prop_frame(rect, color);
    draw_rectangle(
        rect.x + 8.0,
        rect.y + rect.h - 8.0,
        rect.w - 16.0,
        4.0,
        with_alpha(TEXT_DIM, 0.75),
    );
    for x in [rect.x + 18.0, rect.x + 32.0, rect.x + 46.0, rect.x + 60.0] {
        draw_triangle(
            vec2(x - 5.0, rect.y + rect.h - 8.0),
            vec2(x, rect.y + 7.0),
            vec2(x + 5.0, rect.y + rect.h - 8.0),
            with_alpha(color, if upgrade.disarmed { 0.34 } else { 0.92 }),
        );
    }
    if upgrade.disarmed {
        draw_line(
            rect.x + 8.0,
            rect.y + 7.0,
            rect.x + rect.w - 8.0,
            rect.y + rect.h - 7.0,
            1.5,
            TEXT_DIM,
        );
    }
}

fn draw_poison_dart(rect: Rect, upgrade: &RoomUpgrade) {
    let color = trap_color(upgrade);
    prop_frame(rect, color);
    draw_rectangle(
        rect.x + 8.0,
        rect.y + 8.0,
        12.0,
        13.0,
        with_alpha(BG_DEEP, 0.90),
    );
    draw_rectangle_lines(rect.x + 8.0, rect.y + 8.0, 12.0, 13.0, 1.0, color);
    draw_line(
        rect.x + 20.0,
        rect.y + 14.0,
        rect.x + rect.w - 9.0,
        rect.y + 14.0,
        2.0,
        color,
    );
    draw_triangle(
        vec2(rect.x + rect.w - 7.0, rect.y + 14.0),
        vec2(rect.x + rect.w - 13.0, rect.y + 10.0),
        vec2(rect.x + rect.w - 13.0, rect.y + 18.0),
        color,
    );
    draw_circle(rect.x + 27.0, rect.y + 20.0, 2.0, with_alpha(EMERALD, 0.85));
    draw_circle(rect.x + 35.0, rect.y + 21.0, 1.5, with_alpha(EMERALD, 0.68));
}

fn draw_boulder_trap(rect: Rect, upgrade: &RoomUpgrade) {
    let color = trap_color(upgrade);
    prop_frame(rect, color);
    draw_circle(
        rect.x + rect.w * 0.52,
        rect.y + 14.0,
        9.0,
        with_alpha(Color::new(0.30, 0.32, 0.36, 1.0), 0.96),
    );
    draw_circle_lines(rect.x + rect.w * 0.52, rect.y + 14.0, 9.0, 1.0, color);
    draw_line(
        rect.x + 16.0,
        rect.y + 22.0,
        rect.x + 24.0,
        rect.y + 8.0,
        2.0,
        color,
    );
    draw_line(
        rect.x + rect.w - 16.0,
        rect.y + 22.0,
        rect.x + rect.w - 24.0,
        rect.y + 8.0,
        2.0,
        color,
    );
    draw_rectangle(
        rect.x + 9.0,
        rect.y + rect.h - 7.0,
        rect.w - 18.0,
        3.0,
        with_alpha(color, 0.66),
    );
}

fn draw_flame_vent(rect: Rect, upgrade: &RoomUpgrade) {
    let color = trap_color(upgrade);
    prop_frame(rect, color);
    draw_rectangle(
        rect.x + 8.0,
        rect.y + rect.h - 10.0,
        rect.w - 16.0,
        5.0,
        with_alpha(TEXT_DIM, 0.85),
    );
    for x in [rect.x + 16.0, rect.x + 28.0, rect.x + 40.0, rect.x + 52.0] {
        draw_line(x, rect.y + rect.h - 11.0, x + 2.0, rect.y + 7.0, 1.5, color);
    }
    draw_triangle(
        vec2(rect.x + 25.0, rect.y + 20.0),
        vec2(rect.x + 31.0, rect.y + 6.0),
        vec2(rect.x + 36.0, rect.y + 20.0),
        with_alpha(color, 0.92),
    );
    draw_triangle(
        vec2(rect.x + 38.0, rect.y + 20.0),
        vec2(rect.x + 44.0, rect.y + 9.0),
        vec2(rect.x + 49.0, rect.y + 20.0),
        with_alpha(DANGER, 0.82),
    );
}

fn draw_frost_snare(rect: Rect, upgrade: &RoomUpgrade) {
    let color = trap_color(upgrade);
    prop_frame(rect, color);
    let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    draw_circle_lines(center.x, center.y, 9.0, 2.0, with_alpha(color, 0.92));
    draw_circle_lines(center.x, center.y, 4.0, 1.0, with_alpha(color, 0.72));
    for angle in [0.0_f32, 1.57, 3.14, 4.71] {
        let spoke = vec2(angle.cos(), angle.sin());
        draw_line(
            center.x - spoke.x * 12.0,
            center.y - spoke.y * 12.0,
            center.x + spoke.x * 12.0,
            center.y + spoke.y * 12.0,
            1.2,
            color,
        );
    }
}

fn draw_alarm_gong(rect: Rect, upgrade: &RoomUpgrade) {
    let color = trap_color(upgrade);
    prop_frame(rect, color);
    draw_circle_lines(rect.x + 24.0, rect.y + 15.0, 9.0, 2.0, color);
    draw_line(
        rect.x + 15.0,
        rect.y + 15.0,
        rect.x + 33.0,
        rect.y + 15.0,
        1.2,
        color,
    );
    draw_line(
        rect.x + 24.0,
        rect.y + 6.0,
        rect.x + 24.0,
        rect.y + 24.0,
        1.2,
        color,
    );
    draw_line(
        rect.x + 32.0,
        rect.y + 7.0,
        rect.x + rect.w - 8.0,
        rect.y + 22.0,
        2.0,
        with_alpha(TREASURE, 0.88),
    );
    draw_circle(rect.x + rect.w - 8.0, rect.y + 22.0, 3.0, TREASURE);
}

fn draw_mana_siphon(rect: Rect, upgrade: &RoomUpgrade) {
    let color = trap_color(upgrade);
    prop_frame(rect, color);
    draw_triangle(
        vec2(rect.x + 22.0, rect.y + 5.0),
        vec2(rect.x + 11.0, rect.y + 22.0),
        vec2(rect.x + 33.0, rect.y + 22.0),
        with_alpha(color, 0.82),
    );
    draw_line(
        rect.x + 39.0,
        rect.y + 8.0,
        rect.x + 39.0,
        rect.y + 22.0,
        2.0,
        MANA,
    );
    draw_line(
        rect.x + 46.0,
        rect.y + 8.0,
        rect.x + 46.0,
        rect.y + 22.0,
        1.5,
        SOUL,
    );
    draw_circle(rect.x + 39.0, rect.y + 6.0, 2.0, MANA);
    draw_circle(rect.x + 46.0, rect.y + 6.0, 1.5, SOUL);
    draw_line(
        rect.x + 33.0,
        rect.y + 14.0,
        rect.x + 38.0,
        rect.y + 14.0,
        1.0,
        color,
    );
}

fn draw_gold_snatcher(rect: Rect, upgrade: &RoomUpgrade) {
    let color = trap_color(upgrade);
    prop_frame(rect, color);
    draw_rectangle(
        rect.x + 10.0,
        rect.y + 17.0,
        21.0,
        7.0,
        with_alpha(TREASURE, 0.78),
    );
    draw_circle(rect.x + 16.0, rect.y + 15.0, 3.0, TREASURE);
    draw_circle(rect.x + 25.0, rect.y + 15.0, 3.0, TREASURE);
    draw_line(
        rect.x + 32.0,
        rect.y + 8.0,
        rect.x + 39.0,
        rect.y + 21.0,
        2.0,
        color,
    );
    draw_line(
        rect.x + 39.0,
        rect.y + 21.0,
        rect.x + 49.0,
        rect.y + 8.0,
        2.0,
        color,
    );
    draw_line(
        rect.x + 49.0,
        rect.y + 8.0,
        rect.x + 41.0,
        rect.y + 9.0,
        1.5,
        color,
    );
    draw_line(
        rect.x + 49.0,
        rect.y + 8.0,
        rect.x + 48.0,
        rect.y + 17.0,
        1.5,
        color,
    );
}

fn draw_abyssal_maw(rect: Rect, upgrade: &RoomUpgrade) {
    let color = trap_color(upgrade);
    prop_frame(rect, color);
    let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    draw_circle(center.x, center.y, 10.0, with_alpha(BG_DEEP, 0.96));
    draw_circle_lines(center.x, center.y, 10.0, 2.0, with_alpha(color, 0.92));
    for x in [center.x - 7.0, center.x, center.x + 7.0] {
        draw_triangle(
            vec2(x - 3.0, center.y - 5.0),
            vec2(x, center.y + 2.0),
            vec2(x + 3.0, center.y - 5.0),
            with_alpha(TEXT, 0.85),
        );
        draw_triangle(
            vec2(x - 3.0, center.y + 5.0),
            vec2(x, center.y - 2.0),
            vec2(x + 3.0, center.y + 5.0),
            with_alpha(TEXT, 0.70),
        );
    }
}
