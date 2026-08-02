//! Room tile composition: chamber art, unit icons, floating effects, labels,
//! the build-here ghost tile, and route connectors.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::game_state::{Adventurer, EffectAnchor, EffectKind, GameState, Monster, Room, RoomType};
use crate::ui::theme::*;

use super::icons::{
    draw_combat_art, draw_core_art, draw_dashed_border, draw_entrance_art, draw_room_icon,
};
use super::{adventurers_in_room, BuildPreview, PlacementState};

pub(super) fn draw_room_tile(
    state: &GameState,
    room: &Room,
    rect: Rect,
    placement: PlacementState,
) -> bool {
    let hovered = is_hovered_rect(rect);
    let selected = state.selected_room == Some((room.floor_number, room.position));
    let adventurers = adventurers_in_room(state, room);
    let alive = room.monsters.iter().filter(|monster| monster.alive).count();
    let fighting = !adventurers.is_empty() && alive > 0;
    let (fill, border, icon_color, title) = room_tone(room);
    let mut draw_rect = rect;
    if hovered {
        draw_rect.y -= 2.0;
    }

    draw_room_chamber_art(draw_rect, room, fill, border, icon_color);
    if fighting {
        let pulse = (get_time() as f32 * 5.5).sin().abs();
        draw_rectangle_lines(
            draw_rect.x - 2.0,
            draw_rect.y - 2.0,
            draw_rect.w + 4.0,
            draw_rect.h + 4.0,
            2.0,
            with_alpha(WARNING, 0.45 + pulse * 0.35),
        );
    }
    if selected {
        draw_rectangle_lines(
            draw_rect.x - 2.0,
            draw_rect.y - 2.0,
            draw_rect.w + 4.0,
            draw_rect.h + 4.0,
            3.0,
            MANA,
        );
    }

    match placement {
        PlacementState::Valid => {
            draw_rectangle_lines(
                draw_rect.x + 2.0,
                draw_rect.y + 2.0,
                draw_rect.w - 4.0,
                draw_rect.h - 4.0,
                2.0,
                EMERALD,
            );
            draw_pill(
                Rect::new(
                    draw_rect.x + draw_rect.w - 44.0,
                    draw_rect.y + draw_rect.h * 0.46,
                    39.0,
                    16.0,
                ),
                "Place",
                EMERALD,
            );
            // Synergy hint: this room's attunement matches the selected
            // monster's element, so placing here grants the attunement bonus.
            if let Some(selected) = &state.selected_monster {
                if let (Some(elem), Some((room_elem, _))) = (
                    crate::data::monsters::monster_element_id(selected),
                    room.attunement(),
                ) {
                    if elem.as_str() == room_elem {
                        draw_pill(
                            Rect::new(draw_rect.x + 6.0, draw_rect.y + 6.0, 76.0, 16.0),
                            "Attuned",
                            element_color(&elem),
                        );
                    }
                }
            }
        }
        PlacementState::Invalid => {
            draw_rectangle(
                draw_rect.x,
                draw_rect.y,
                draw_rect.w,
                draw_rect.h,
                Color::new(0.0, 0.0, 0.0, 0.36),
            );
            // A combat room that is merely out of slots reads as "no room
            // left", not as the wrong kind of room.
            if (room.room_type == RoomType::Normal || room.room_type == RoomType::Boss)
                && crate::data::constants::room_is_full(room)
            {
                draw_pill(
                    Rect::new(
                        draw_rect.x + draw_rect.w - 44.0,
                        draw_rect.y + draw_rect.h * 0.46,
                        39.0,
                        16.0,
                    ),
                    "Full",
                    WARNING,
                );
            }
        }
        PlacementState::Idle => {}
    }

    let label_rect = Rect::new(
        draw_rect.x + draw_rect.w * 0.12,
        draw_rect.y + draw_rect.h + 9.0,
        draw_rect.w * 0.76,
        27.0,
    );
    draw_room_label_plate(label_rect, title, room, icon_color);

    // Per-unit icons: defenders on the left, adventurers on the right.
    let strip = Rect::new(
        draw_rect.x + 8.0,
        draw_rect.y + draw_rect.h - 26.0,
        draw_rect.w - 16.0,
        22.0,
    );
    // Recurring survivors / prolific slayers are "rivals": name them on the
    // board so heroes read as recognisable actors, not anonymous tokens.
    let rival_ids: Vec<u64> = adventurers
        .iter()
        .filter(|a| {
            state
                .known_adventurers
                .iter()
                .any(|h| h.id == a.id && h.is_rival())
        })
        .map(|a| a.id)
        .collect();
    draw_room_units(room, strip, &adventurers, fighting, &rival_ids);

    // Floating combat feedback (damage numbers, kills) rising over the room.
    draw_room_effects(state, room, draw_rect);

    was_clicked_rect(rect)
}

/// Draw one icon per unit in the room: monster defenders (with their initial)
/// on the left, adventurers on the right. Living units carry a health bar so a
/// raid reads as a fight — defenders and invaders trading HP — not a count.
/// Overflow collapses into a "+N" tag.
fn draw_room_units(
    room: &Room,
    strip: Rect,
    adventurers: &[&Adventurer],
    fighting: bool,
    rival_ids: &[u64],
) {
    let radius = 7.0;
    let step = radius * 2.0 + 3.0;
    // Leave headroom below each disc for its health bar.
    let cy = strip.y + radius + 1.0;

    if room.room_type == RoomType::Normal || room.room_type == RoomType::Boss {
        // Alive defenders first so they read clearly, then fallen ones.
        let mut ordered: Vec<&Monster> = room.monsters.iter().filter(|m| m.alive).collect();
        ordered.extend(room.monsters.iter().filter(|m| !m.alive));

        let zone_w = strip.w * 0.60;
        let max_icons = ((zone_w / step).floor() as usize).max(1);
        let mut x = strip.x + radius + 1.0;
        let mut drawn = 0;
        for monster in &ordered {
            if drawn >= max_icons {
                break;
            }
            // Colour a living defender by its element so different monster
            // types read apart at a glance; the initial keeps them legible
            // without relying on hue alone.
            let color = if monster.alive {
                match crate::data::monsters::monster_element_id(&monster.type_name) {
                    Some(element) => element_color(&element),
                    None => EMERALD,
                }
            } else {
                TEXT_DIM
            };
            let initial = monster
                .type_name
                .chars()
                .next()
                .map(|c| c.to_ascii_uppercase().to_string())
                .unwrap_or_else(|| "?".to_string());
            draw_icon_disc(vec2(x, cy), radius, color, &initial);
            if monster.alive && (fighting || monster.hp < monster.max_hp) {
                draw_unit_hp_bar(vec2(x, cy), radius, monster.hp, monster.max_hp);
            }
            x += step;
            drawn += 1;
        }
        if ordered.len() > drawn {
            draw_text_fit(
                &format!("+{}", ordered.len() - drawn),
                x - 2.0,
                cy + 4.0,
                28.0,
                11.0,
                TEXT_MUTED,
            );
        }
    }

    if !adventurers.is_empty() {
        let zone_w = strip.w * 0.36;
        let max_icons = ((zone_w / step).floor() as usize).max(1);
        let shown = adventurers.len().min(max_icons);
        let mut x = strip.x + strip.w - radius - 1.0;
        for adventurer in adventurers.iter().take(shown) {
            // Label each invader with its class initial so a Warrior, Rogue,
            // and Mage read apart rather than as identical "A" tokens.
            let initial = adventurer
                .class_name
                .chars()
                .next()
                .map(|c| c.to_ascii_uppercase().to_string())
                .unwrap_or_else(|| "A".to_string());
            let is_rival = rival_ids.contains(&adventurer.id);
            draw_icon_disc(vec2(x, cy), radius, WARNING, &initial);
            if is_rival {
                // A gold ring and a name plate mark the dungeon's rival.
                draw_circle_lines(x, cy, radius + 2.5, 1.6, TREASURE);
                let first = adventurer
                    .name
                    .split_whitespace()
                    .next()
                    .unwrap_or(&adventurer.name);
                draw_centered_text(
                    first,
                    Rect::new(x - 30.0, cy - radius - 15.0, 60.0, 12.0),
                    10.0,
                    TREASURE,
                );
            }
            if fighting || adventurer.hp < adventurer.max_hp {
                draw_unit_hp_bar(vec2(x, cy), radius, adventurer.hp, adventurer.max_hp);
            }
            x -= step;
        }
        if adventurers.len() > shown {
            draw_text_fit_right(
                &format!("+{}", adventurers.len() - shown),
                x + radius,
                cy + 4.0,
                28.0,
                11.0,
                WARNING,
            );
        }
    }
}

/// A compact health bar tucked under a unit disc. Colour shifts green → amber →
/// red as the unit loses HP, so the state of a fight is readable at a glance.
fn draw_unit_hp_bar(center: Vec2, radius: f32, hp: i32, max_hp: i32) {
    let ratio = if max_hp > 0 {
        (hp as f32 / max_hp as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let w = radius * 2.2;
    let h = 3.0;
    let x = center.x - w * 0.5;
    let y = center.y + radius + 1.0;
    draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.66));
    draw_rectangle(x, y, w * ratio, h, hp_bar_color(ratio));
    draw_rectangle_lines(x, y, w, h, 1.0, Color::new(0.0, 0.0, 0.0, 0.5));
}

/// Health-fraction colour: healthy green, wounded amber, critical red.
fn hp_bar_color(ratio: f32) -> Color {
    if ratio > 0.6 {
        EMERALD
    } else if ratio > 0.3 {
        WARNING
    } else {
        DANGER
    }
}

/// Render active floating effects anchored to this room, rising and fading out.
/// Effects stack and rise within their anchor side (defenders left, invaders
/// right, neutral centre) so a fight reads as two sides trading blows.
fn draw_room_effects(state: &GameState, room: &Room, rect: Rect) {
    let mut stack_by_anchor: [i32; 3] = [0, 0, 0];
    for effect in state
        .effects
        .iter()
        .filter(|e| e.floor == room.floor_number && e.room == room.position)
    {
        let (anchor_idx, anchor_x) = match effect.anchor {
            EffectAnchor::Defenders => (0usize, rect.x + rect.w * 0.30),
            EffectAnchor::Invaders => (1usize, rect.x + rect.w * 0.70),
            EffectAnchor::Center => (2usize, rect.x + rect.w * 0.5),
        };
        let stack = stack_by_anchor[anchor_idx];
        stack_by_anchor[anchor_idx] += 1;

        let life = effect.life_fraction();
        let rise = (1.0 - life) * 28.0 + stack as f32 * 15.0;
        let color = effect_color(effect.kind);
        let faded = with_alpha(color, life);
        let cx = anchor_x;
        let cy = rect.y + rect.h * 0.36 - rise;
        // Shadow for legibility over busy art.
        draw_centered_text(
            &effect.text,
            Rect::new(cx - 69.0, cy - 7.0, 140.0, 16.0),
            13.0,
            with_alpha(BLACK, life * 0.7),
        );
        draw_centered_text(
            &effect.text,
            Rect::new(cx - 70.0, cy - 8.0, 140.0, 16.0),
            13.0,
            faded,
        );
    }
}

fn effect_color(kind: EffectKind) -> Color {
    match kind {
        EffectKind::Damage => WARNING,
        EffectKind::Ability => SOUL,
        EffectKind::MonsterDown => DANGER,
        EffectKind::AdventurerDown => EMERALD,
        EffectKind::Loot => TREASURE,
    }
}

fn draw_room_chamber_art(rect: Rect, room: &Room, fill: Color, border: Color, icon_color: Color) {
    draw_card(rect, fill, border);

    let wall = Rect::new(rect.x + 8.0, rect.y + 8.0, rect.w - 16.0, rect.h - 18.0);
    draw_rectangle(
        wall.x,
        wall.y,
        wall.w,
        wall.h,
        Color::new(0.0, 0.0, 0.0, 0.18),
    );

    let brick = Color::new(0.20, 0.22, 0.25, 0.13);
    let mut by = wall.y + 8.0;
    while by < wall.y + wall.h - 8.0 {
        draw_line(wall.x + 6.0, by, wall.x + wall.w - 6.0, by, 1.0, brick);
        by += 16.0;
    }
    let mut bx = wall.x + 12.0;
    while bx < wall.x + wall.w - 10.0 {
        draw_line(
            bx,
            wall.y + 10.0,
            bx,
            wall.y + wall.h - 10.0,
            1.0,
            Color::new(0.18, 0.20, 0.24, 0.08),
        );
        bx += 24.0;
    }

    match room.room_type {
        RoomType::Entrance => draw_entrance_art(wall, icon_color),
        RoomType::Normal | RoomType::Boss => draw_combat_art(wall, icon_color),
        RoomType::Core => draw_core_art(wall, icon_color),
    }
}

fn draw_room_label_plate(rect: Rect, title: &str, room: &Room, color: Color) {
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.34),
        with_alpha(color, 0.28),
    );
    draw_text_fit(
        title,
        rect.x + 28.0,
        rect.y + 18.0,
        rect.w - 34.0,
        12.0,
        color,
    );
    draw_room_icon(
        &room.room_type,
        vec2(rect.x + 15.0, rect.y + rect.h * 0.50),
        7.0,
        color,
    );

    // Combat rooms carry their occupancy: slots are the scarce build resource,
    // so the board states them without needing a click.
    if room.room_type == RoomType::Normal || room.room_type == RoomType::Boss {
        let capacity = crate::data::constants::room_capacity(room);
        let used = room.monsters.len();
        draw_text_fit_right(
            &format!("{}/{}", used, capacity),
            rect.x + rect.w - 6.0,
            rect.y + 18.0,
            30.0,
            11.0,
            if used >= capacity {
                WARNING
            } else {
                TEXT_MUTED
            },
        );
    }
}

pub(super) fn draw_future_room_tile(state: &GameState, rect: Rect, plan: &BuildPreview) -> bool {
    let can_afford = state.mana >= plan.cost;
    let can_build = can_afford && state.adventurer_parties.is_empty();
    let hovered = is_hovered_rect(rect);
    let fill = if can_build {
        with_alpha(TREASURE, 0.10)
    } else {
        Color::new(0.045, 0.052, 0.075, 0.72)
    };
    let border = if can_build { TREASURE } else { BORDER_MUTED };
    let mut draw_rect = rect;
    if hovered && can_build {
        draw_rect.y -= 2.0;
    }

    draw_card(draw_rect, fill, border);
    draw_dashed_border(draw_rect, border);
    draw_centered_text(
        "+",
        Rect::new(draw_rect.x, draw_rect.y + 8.0, draw_rect.w, 26.0),
        30.0,
        border,
    );
    let label = if plan.new_floor {
        format!("Floor {}", plan.floor)
    } else if plan.room_type == RoomType::Boss {
        "Boss".to_string()
    } else {
        "Room".to_string()
    };
    draw_centered_text(
        "Build Room",
        Rect::new(
            draw_rect.x,
            draw_rect.y + draw_rect.h * 0.57,
            draw_rect.w,
            22.0,
        ),
        13.0,
        border,
    );
    let label_rect = Rect::new(
        draw_rect.x + draw_rect.w * 0.18,
        draw_rect.y + draw_rect.h + 9.0,
        draw_rect.w * 0.64,
        27.0,
    );
    draw_card(
        label_rect,
        Color::new(0.0, 0.0, 0.0, 0.34),
        with_alpha(border, 0.24),
    );
    draw_centered_text(&label, label_rect, 11.0, TEXT_MUTED);
    draw_centered_text(
        &format!("{}M", plan.cost),
        Rect::new(label_rect.x, label_rect.y + 24.0, label_rect.w, 14.0),
        11.0,
        if can_afford { MANA } else { DANGER },
    );

    was_clicked_rect(rect)
}

/// Draw a party marker gliding along a corridor connector at `progress` (0..1),
/// so a party visibly crosses from one room to the next instead of teleporting.
pub(super) fn draw_party_transit(connector: Rect, progress: f32) {
    let cx = connector.x + connector.w * progress;
    let cy = connector.y + connector.h * 0.5;
    // A short motion trail behind the marker sells the direction of travel.
    draw_circle(cx - 5.0, cy, 4.0, with_alpha(WARNING, 0.16));
    draw_icon_disc(vec2(cx, cy), 6.5, WARNING, "A");
}

pub(super) fn draw_connector(rect: Rect, ghost: bool) {
    let alpha = if ghost { 0.32 } else { 0.70 };
    let fill = Color::new(0.110, 0.145, 0.195, alpha);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::new(0.35, 0.40, 0.48, alpha),
    );
    draw_line(
        rect.x + 4.0,
        rect.y + rect.h * 0.5,
        rect.x + rect.w - 4.0,
        rect.y + rect.h * 0.5,
        2.0,
        with_alpha(TREASURE, if ghost { 0.22 } else { 0.36 }),
    );
}

fn room_tone(room: &Room) -> (Color, Color, Color, &'static str) {
    match room.room_type {
        RoomType::Entrance => (
            Color::new(0.05, 0.34, 0.22, 0.94),
            with_alpha(EMERALD, 0.78),
            Color::new(0.70, 1.00, 0.82, 1.0),
            "Entrance",
        ),
        RoomType::Normal => (
            Color::new(0.07, 0.19, 0.29, 0.94),
            with_alpha(MANA, 0.58),
            Color::new(0.72, 0.91, 1.0, 1.0),
            "Room",
        ),
        RoomType::Boss => (
            Color::new(0.36, 0.12, 0.07, 0.94),
            with_alpha(WARNING, 0.78),
            Color::new(1.0, 0.80, 0.58, 1.0),
            "Boss",
        ),
        RoomType::Core => (
            Color::new(0.26, 0.10, 0.42, 0.96),
            with_alpha(SOUL, 0.82),
            Color::new(0.93, 0.78, 1.0, 1.0),
            "Core",
        ),
    }
}
