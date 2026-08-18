//! Defender and adventurer composition inside a single dungeon room tile.

use macroquad::prelude::*;

use crate::game_state::{Adventurer, Monster, Room, RoomType};
use crate::ui::theme::*;

use super::super::DungeonSprites;

// Board units need to remain readable when rooms are scaled down to fit a
// long floor. The sprite is deliberately larger than its layout footprint;
// the atlas frames have transparent margins, so nearby units still separate
// cleanly while their actual creature art remains prominent.
const UNIT_RADIUS: f32 = 13.0;
const UNIT_STEP: f32 = 30.0;
const UNIT_SPRITE_SIZE: f32 = 40.0;

/// Draw one icon per unit in the room. Overflow collapses into a compact tag.
pub(super) fn draw_room_units(
    room: &Room,
    strip: Rect,
    adventurers: &[&Adventurer],
    fighting: bool,
    rival_ids: &[u64],
    sprites: &DungeonSprites,
) {
    let radius = UNIT_RADIUS;
    let step = UNIT_STEP;
    let cy = strip.y + strip.h * 0.5;

    if room.room_type == RoomType::Normal || room.room_type == RoomType::Boss {
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
            let color = if monster.alive {
                match crate::data::monsters::monster_element_id(&monster.type_name) {
                    Some(element) => element_color(&element),
                    None => EMERALD,
                }
            } else {
                TEXT_DIM
            };
            let element = crate::data::monsters::monster_element_id(&monster.type_name);
            let initial = monster
                .type_name
                .chars()
                .next()
                .map(|c| c.to_ascii_uppercase().to_string())
                .unwrap_or_else(|| "?".to_string());
            let phase = get_time() as f32 + monster.id as f32 * 0.173;
            let bob = phase.sin() * if monster.alive { 1.5 } else { 0.0 };
            let drawn_sprite = monster.alive
                && sprites.draw_monster(
                    &monster.type_name,
                    vec2(x, cy + bob),
                    UNIT_SPRITE_SIZE,
                    phase,
                    false,
                    fighting,
                );
            if !drawn_sprite {
                draw_icon_disc(vec2(x, cy), radius, color, &initial);
            }
            if monster.alive {
                if let Some(element) = element {
                    draw_element_badge(vec2(x + radius - 2.0, cy - radius + 2.0), &element);
                }
            }
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
            let initial = adventurer
                .class_name
                .chars()
                .next()
                .map(|c| c.to_ascii_uppercase().to_string())
                .unwrap_or_else(|| "A".to_string());
            let is_rival = rival_ids.contains(&adventurer.id);
            let phase = get_time() as f32 + adventurer.id as f32 * 0.173;
            let bob = phase.sin() * 1.5;
            let drawn_sprite = sprites.draw_adventurer(
                &adventurer.class_name,
                vec2(x, cy + bob),
                UNIT_SPRITE_SIZE,
                phase,
                true,
                false,
                fighting,
            );
            if !drawn_sprite {
                draw_icon_disc(vec2(x, cy), radius, WARNING, &initial);
            }
            if is_rival {
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

fn hp_bar_color(ratio: f32) -> Color {
    if ratio > 0.6 {
        EMERALD
    } else if ratio > 0.3 {
        WARNING
    } else {
        DANGER
    }
}
