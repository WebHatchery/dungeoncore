//! Physical map props for installed room upgrades.
//!
//! Upgrade effects are still resolved by the simulation. This module only
//! makes the installed mechanism visible in the chamber, using the same
//! vector language as the room art so it remains crisp at every board zoom.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use crate::game_state::{Room, RoomUpgrade, RoomUpgradeType};
use crate::ui::theme::*;

#[cfg(test)]
mod tests;
mod traps;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UpgradeArtKey {
    SpikeTrap,
    PoisonDart,
    BoulderTrap,
    FlameVent,
    FrostSnare,
    AlarmGong,
    ManaSiphon,
    GoldSnatcher,
    AbyssalMaw,
    GoldCache,
    TreasureVault,
    DragonHoard,
    StoneWalls,
    DarkAura,
    DemonicPact,
    GrowthChamber,
    EvolutionPit,
    FireShrine,
    SpringAltar,
    StandingStones,
    Ossuary,
}

/// The map has a named prop for every upgrade in `assets/upgrades.json`.
/// Unknown save data gets a safe category fallback instead of crashing.
fn upgrade_art_key(name: &str) -> Option<UpgradeArtKey> {
    Some(match name {
        "Spike Trap" => UpgradeArtKey::SpikeTrap,
        "Poison Dart" => UpgradeArtKey::PoisonDart,
        "Boulder Trap" => UpgradeArtKey::BoulderTrap,
        "Flame Vent" => UpgradeArtKey::FlameVent,
        "Frost Snare" => UpgradeArtKey::FrostSnare,
        "Alarm Gong" => UpgradeArtKey::AlarmGong,
        "Mana Siphon" => UpgradeArtKey::ManaSiphon,
        "Gold Snatcher" => UpgradeArtKey::GoldSnatcher,
        "Abyssal Maw" => UpgradeArtKey::AbyssalMaw,
        "Gold Cache" => UpgradeArtKey::GoldCache,
        "Treasure Vault" => UpgradeArtKey::TreasureVault,
        "Dragon Hoard" => UpgradeArtKey::DragonHoard,
        "Stone Walls" => UpgradeArtKey::StoneWalls,
        "Dark Aura" => UpgradeArtKey::DarkAura,
        "Demonic Pact" => UpgradeArtKey::DemonicPact,
        "Growth Chamber" => UpgradeArtKey::GrowthChamber,
        "Evolution Pit" => UpgradeArtKey::EvolutionPit,
        "Fire Shrine" => UpgradeArtKey::FireShrine,
        "Spring Altar" => UpgradeArtKey::SpringAltar,
        "Standing Stones" => UpgradeArtKey::StandingStones,
        "Ossuary" => UpgradeArtKey::Ossuary,
        _ => return None,
    })
}

pub(super) fn draw_room_upgrade_art(wall: Rect, room: &Room) {
    for upgrade in &room.upgrades {
        match upgrade_art_key(&upgrade.name) {
            Some(key) => draw_named_upgrade(wall, upgrade, key),
            None => draw_generic_upgrade(wall, upgrade),
        }
    }
}

fn draw_named_upgrade(wall: Rect, upgrade: &RoomUpgrade, key: UpgradeArtKey) {
    match key {
        UpgradeArtKey::StoneWalls => draw_stone_walls(wall),
        UpgradeArtKey::SpikeTrap
        | UpgradeArtKey::PoisonDart
        | UpgradeArtKey::BoulderTrap
        | UpgradeArtKey::FlameVent
        | UpgradeArtKey::FrostSnare
        | UpgradeArtKey::AlarmGong
        | UpgradeArtKey::ManaSiphon
        | UpgradeArtKey::GoldSnatcher
        | UpgradeArtKey::AbyssalMaw => traps::draw_trap(trap_slot(wall), upgrade, key),
        UpgradeArtKey::GoldCache => draw_gold_cache(treasure_slot(wall)),
        UpgradeArtKey::TreasureVault => draw_treasure_vault(treasure_slot(wall)),
        UpgradeArtKey::DragonHoard => draw_dragon_hoard(treasure_slot(wall)),
        UpgradeArtKey::DarkAura => draw_dark_aura(reinforcement_slot(wall)),
        UpgradeArtKey::DemonicPact => draw_demonic_pact(reinforcement_slot(wall)),
        UpgradeArtKey::GrowthChamber => draw_growth_chamber(evolution_slot(wall)),
        UpgradeArtKey::EvolutionPit => draw_evolution_pit(evolution_slot(wall)),
        UpgradeArtKey::FireShrine => draw_fire_shrine(attunement_slot(wall)),
        UpgradeArtKey::SpringAltar => draw_spring_altar(attunement_slot(wall)),
        UpgradeArtKey::StandingStones => draw_standing_stones(attunement_slot(wall)),
        UpgradeArtKey::Ossuary => draw_ossuary(attunement_slot(wall)),
    }
}

fn trap_slot(wall: Rect) -> Rect {
    Rect::new(wall.x + wall.w * 0.5 - 37.0, wall.y + 38.0, 74.0, 29.0)
}

fn treasure_slot(wall: Rect) -> Rect {
    Rect::new(wall.x + 8.0, wall.y + 39.0, 48.0, 29.0)
}

fn reinforcement_slot(wall: Rect) -> Rect {
    Rect::new(wall.x + wall.w - 54.0, wall.y + 39.0, 46.0, 29.0)
}

fn evolution_slot(wall: Rect) -> Rect {
    Rect::new(wall.x + wall.w * 0.25 - 23.0, wall.y + 70.0, 46.0, 27.0)
}

fn attunement_slot(wall: Rect) -> Rect {
    Rect::new(wall.x + wall.w - 54.0, wall.y + 70.0, 46.0, 27.0)
}

fn prop_frame(rect: Rect, accent: Color) {
    draw_rectangle(
        rect.x + 2.0,
        rect.y + 2.0,
        rect.w,
        rect.h,
        with_alpha(BG_DEEP, 0.72),
    );
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, with_alpha(PANEL_ALT, 0.88));
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.2,
        with_alpha(accent, 0.86),
    );
    draw_line(
        rect.x + 3.0,
        rect.y + 3.0,
        rect.x + rect.w - 3.0,
        rect.y + 3.0,
        1.0,
        with_alpha(accent, 0.30),
    );
}

fn draw_gold_cache(rect: Rect) {
    prop_frame(rect, TREASURE);
    draw_chest(rect, TREASURE, 14.0);
    draw_circle(
        rect.x + 34.0,
        rect.y + 21.0,
        3.0,
        with_alpha(TREASURE, 0.92),
    );
    draw_circle(
        rect.x + 41.0,
        rect.y + 19.0,
        2.5,
        with_alpha(TREASURE, 0.78),
    );
}

fn draw_treasure_vault(rect: Rect) {
    prop_frame(rect, TREASURE);
    draw_rectangle(
        rect.x + 14.0,
        rect.y + 6.0,
        20.0,
        18.0,
        with_alpha(Color::new(0.25, 0.22, 0.16, 1.0), 0.96),
    );
    draw_rectangle_lines(rect.x + 14.0, rect.y + 6.0, 20.0, 18.0, 1.5, TREASURE);
    draw_circle_lines(rect.x + 24.0, rect.y + 15.0, 5.0, 1.2, TREASURE);
    draw_line(
        rect.x + 24.0,
        rect.y + 10.0,
        rect.x + 24.0,
        rect.y + 20.0,
        1.0,
        TREASURE,
    );
    draw_line(
        rect.x + 19.0,
        rect.y + 15.0,
        rect.x + 29.0,
        rect.y + 15.0,
        1.0,
        TREASURE,
    );
    draw_circle(rect.x + 40.0, rect.y + 22.0, 3.0, TREASURE);
}

fn draw_dragon_hoard(rect: Rect) {
    prop_frame(rect, TREASURE);
    for (x, y, r) in [
        (14.0, 21.0, 4.0),
        (22.0, 17.0, 4.0),
        (30.0, 21.0, 4.0),
        (38.0, 17.0, 4.0),
    ] {
        draw_circle(rect.x + x, rect.y + y, r, with_alpha(TREASURE, 0.86));
        draw_circle_lines(rect.x + x, rect.y + y, r, 0.8, with_alpha(TEXT, 0.55));
    }
    draw_triangle(
        vec2(rect.x + 35.0, rect.y + 8.0),
        vec2(rect.x + 29.0, rect.y + 18.0),
        vec2(rect.x + 41.0, rect.y + 18.0),
        DANGER,
    );
    draw_circle(rect.x + 35.0, rect.y + 12.0, 1.5, TEXT);
}

fn draw_stone_walls(wall: Rect) {
    let stone = Color::new(0.68, 0.75, 0.82, 1.0);
    let shadow = Color::new(0.20, 0.25, 0.32, 1.0);

    // Reinforcement changes the room silhouette: broad masonry pillars and a
    // capped lintel replace the unmodified chamber's thin edge seams.
    draw_rectangle(
        wall.x + 1.0,
        wall.y + 1.0,
        8.0,
        wall.h - 2.0,
        with_alpha(shadow, 0.92),
    );
    draw_rectangle(
        wall.x + wall.w - 9.0,
        wall.y + 1.0,
        8.0,
        wall.h - 2.0,
        with_alpha(shadow, 0.92),
    );
    draw_rectangle(
        wall.x + 1.0,
        wall.y + 1.0,
        wall.w - 2.0,
        8.0,
        with_alpha(shadow, 0.94),
    );
    draw_line(
        wall.x + 4.0,
        wall.y + 2.0,
        wall.x + wall.w - 4.0,
        wall.y + 2.0,
        1.5,
        stone,
    );

    let mut y = wall.y + 16.0;
    let mut row = 0usize;
    while y < wall.y + wall.h - 17.0 {
        draw_line(
            wall.x + 2.0,
            y,
            wall.x + 9.0,
            y,
            1.2,
            with_alpha(stone, 0.72),
        );
        draw_line(
            wall.x + wall.w - 9.0,
            y,
            wall.x + wall.w - 2.0,
            y,
            1.2,
            with_alpha(stone, 0.72),
        );
        if row.is_multiple_of(2) {
            draw_line(
                wall.x + 5.0,
                y - 13.0,
                wall.x + 5.0,
                y,
                1.0,
                with_alpha(stone, 0.55),
            );
            draw_line(
                wall.x + wall.w - 5.0,
                y - 13.0,
                wall.x + wall.w - 5.0,
                y,
                1.0,
                with_alpha(stone, 0.55),
            );
        }
        y += 13.0;
        row += 1;
    }
    draw_rectangle_lines(
        wall.x + 1.0,
        wall.y + 1.0,
        wall.w - 2.0,
        wall.h - 2.0,
        1.6,
        with_alpha(stone, 0.86),
    );
    for x in [wall.x + 5.0, wall.x + wall.w - 5.0] {
        draw_circle(x, wall.y + 10.0, 2.0, TREASURE);
        draw_circle(x, wall.y + wall.h - 11.0, 2.0, TREASURE);
    }
}

fn draw_dark_aura(rect: Rect) {
    prop_frame(rect, SOUL);
    let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    draw_circle(center.x, center.y, 11.0, with_alpha(SOUL, 0.14));
    draw_circle_lines(center.x, center.y, 10.0, 1.5, with_alpha(SOUL, 0.90));
    draw_circle_lines(center.x, center.y, 5.0, 1.0, with_alpha(ARCANE, 0.80));
    for x in [rect.x + 10.0, rect.x + rect.w - 10.0] {
        draw_circle(x, rect.y + 14.0, 3.0, with_alpha(SOUL, 0.76));
    }
}

fn draw_demonic_pact(rect: Rect) {
    prop_frame(rect, DANGER);
    let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    draw_circle_lines(center.x, center.y, 10.0, 1.5, with_alpha(DANGER, 0.90));
    for (dx, dy) in [(0.0, -8.0), (8.0, 5.0), (-8.0, 5.0)] {
        draw_line(
            center.x,
            center.y,
            center.x + dx,
            center.y + dy,
            1.2,
            DANGER,
        );
    }
    draw_circle(center.x, center.y, 3.0, with_alpha(SOUL, 0.86));
    draw_line(
        rect.x + 7.0,
        rect.y + 8.0,
        rect.x + 15.0,
        rect.y + 16.0,
        1.3,
        DANGER,
    );
    draw_line(
        rect.x + rect.w - 7.0,
        rect.y + 8.0,
        rect.x + rect.w - 15.0,
        rect.y + 16.0,
        1.3,
        DANGER,
    );
}

fn draw_growth_chamber(rect: Rect) {
    prop_frame(rect, EMERALD);
    draw_rectangle_lines(rect.x + 8.0, rect.y + 7.0, 12.0, 17.0, 1.2, EMERALD);
    draw_rectangle_lines(rect.x + 26.0, rect.y + 7.0, 12.0, 17.0, 1.2, EMERALD);
    draw_circle(rect.x + 14.0, rect.y + 16.0, 4.0, with_alpha(EMERALD, 0.44));
    draw_circle(rect.x + 32.0, rect.y + 16.0, 4.0, with_alpha(SOUL, 0.44));
    draw_line(
        rect.x + 22.0,
        rect.y + 23.0,
        rect.x + 22.0,
        rect.y + 12.0,
        1.4,
        EMERALD,
    );
    draw_triangle(
        vec2(rect.x + 22.0, rect.y + 8.0),
        vec2(rect.x + 16.0, rect.y + 14.0),
        vec2(rect.x + 22.0, rect.y + 14.0),
        EMERALD,
    );
}

fn draw_evolution_pit(rect: Rect) {
    prop_frame(rect, SOUL);
    draw_circle(
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.5,
        10.0,
        with_alpha(BG_DEEP, 0.94),
    );
    draw_circle_lines(
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.5,
        10.0,
        1.6,
        SOUL,
    );
    draw_circle_lines(
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.5,
        5.0,
        1.0,
        ARCANE,
    );
    draw_triangle(
        vec2(rect.x + 13.0, rect.y + 8.0),
        vec2(rect.x + 8.0, rect.y + 14.0),
        vec2(rect.x + 16.0, rect.y + 14.0),
        SOUL,
    );
    draw_triangle(
        vec2(rect.x + rect.w - 13.0, rect.y + rect.h - 8.0),
        vec2(rect.x + rect.w - 8.0, rect.y + rect.h - 14.0),
        vec2(rect.x + rect.w - 16.0, rect.y + rect.h - 14.0),
        SOUL,
    );
}

fn draw_fire_shrine(rect: Rect) {
    prop_frame(rect, element_color("Fire"));
    draw_rectangle(
        rect.x + 16.0,
        rect.y + 19.0,
        15.0,
        5.0,
        with_alpha(TREASURE, 0.86),
    );
    draw_triangle(
        vec2(rect.x + 23.0, rect.y + 5.0),
        vec2(rect.x + 15.0, rect.y + 20.0),
        vec2(rect.x + 31.0, rect.y + 20.0),
        element_color("Fire"),
    );
    draw_triangle(
        vec2(rect.x + 23.0, rect.y + 10.0),
        vec2(rect.x + 19.0, rect.y + 20.0),
        vec2(rect.x + 27.0, rect.y + 20.0),
        TREASURE,
    );
}

fn draw_spring_altar(rect: Rect) {
    let color = element_color("Water");
    prop_frame(rect, color);
    draw_rectangle(
        rect.x + 12.0,
        rect.y + 19.0,
        24.0,
        5.0,
        with_alpha(color, 0.80),
    );
    draw_circle(rect.x + 24.0, rect.y + 18.0, 7.0, with_alpha(color, 0.34));
    draw_line(
        rect.x + 24.0,
        rect.y + 18.0,
        rect.x + 24.0,
        rect.y + 7.0,
        1.5,
        color,
    );
    draw_circle(rect.x + 24.0, rect.y + 7.0, 2.5, color);
    draw_line(
        rect.x + 17.0,
        rect.y + 15.0,
        rect.x + 13.0,
        rect.y + 11.0,
        1.0,
        color,
    );
    draw_line(
        rect.x + 31.0,
        rect.y + 15.0,
        rect.x + 35.0,
        rect.y + 11.0,
        1.0,
        color,
    );
}

fn draw_standing_stones(rect: Rect) {
    let color = element_color("Earth");
    prop_frame(rect, color);
    draw_rectangle(
        rect.x + 9.0,
        rect.y + 10.0,
        8.0,
        14.0,
        with_alpha(color, 0.82),
    );
    draw_rectangle(
        rect.x + 19.0,
        rect.y + 6.0,
        9.0,
        18.0,
        with_alpha(color, 0.96),
    );
    draw_rectangle(
        rect.x + 30.0,
        rect.y + 11.0,
        8.0,
        13.0,
        with_alpha(color, 0.70),
    );
    for x in [rect.x + 9.0, rect.x + 19.0, rect.x + 30.0] {
        draw_line(
            x,
            rect.y + 10.0,
            x + 4.0,
            rect.y + 7.0,
            1.0,
            with_alpha(TEXT, 0.42),
        );
    }
    draw_line(
        rect.x + 7.0,
        rect.y + 24.0,
        rect.x + 40.0,
        rect.y + 24.0,
        1.5,
        color,
    );
}

fn draw_ossuary(rect: Rect) {
    let color = element_color("Death");
    prop_frame(rect, color);
    let center = vec2(rect.x + 24.0, rect.y + 13.0);
    draw_circle(
        center.x,
        center.y,
        8.0,
        with_alpha(Color::new(0.72, 0.70, 0.64, 1.0), 0.94),
    );
    draw_circle(center.x - 3.0, center.y - 1.0, 2.0, BG_DEEP);
    draw_circle(center.x + 3.0, center.y - 1.0, 2.0, BG_DEEP);
    draw_rectangle(
        center.x - 5.0,
        center.y + 4.0,
        10.0,
        4.0,
        with_alpha(BG_DEEP, 0.92),
    );
    draw_line(
        rect.x + 14.0,
        rect.y + 24.0,
        rect.x + 34.0,
        rect.y + 24.0,
        2.0,
        color,
    );
    draw_line(
        rect.x + 17.0,
        rect.y + 21.0,
        rect.x + 31.0,
        rect.y + 9.0,
        1.2,
        color,
    );
    draw_line(
        rect.x + 31.0,
        rect.y + 21.0,
        rect.x + 17.0,
        rect.y + 9.0,
        1.2,
        color,
    );
}

fn draw_chest(rect: Rect, color: Color, x: f32) {
    draw_rectangle(
        rect.x + x,
        rect.y + 12.0,
        17.0,
        11.0,
        with_alpha(color, 0.76),
    );
    draw_rectangle_lines(rect.x + x, rect.y + 12.0, 17.0, 11.0, 1.0, color);
    draw_line(
        rect.x + x,
        rect.y + 17.0,
        rect.x + x + 17.0,
        rect.y + 17.0,
        1.0,
        BG_DEEP,
    );
    draw_circle(rect.x + x + 8.5, rect.y + 18.0, 1.5, SOUL);
}

fn draw_generic_upgrade(wall: Rect, upgrade: &RoomUpgrade) {
    let color = upgrade
        .element
        .as_deref()
        .map(element_color)
        .unwrap_or(TEXT_MUTED);
    let rect = match &upgrade.upgrade_type {
        RoomUpgradeType::Trap => trap_slot(wall),
        RoomUpgradeType::Treasure => treasure_slot(wall),
        RoomUpgradeType::Reinforcement => reinforcement_slot(wall),
        RoomUpgradeType::Evolution => evolution_slot(wall),
        RoomUpgradeType::Attunement => attunement_slot(wall),
    };
    prop_frame(rect, color);
    draw_circle_lines(
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.5,
        8.0,
        1.5,
        with_alpha(color, 0.90),
    );
    draw_line(
        rect.x + rect.w * 0.5 - 5.0,
        rect.y + rect.h * 0.5,
        rect.x + rect.w * 0.5 + 5.0,
        rect.y + rect.h * 0.5,
        1.2,
        color,
    );
}
