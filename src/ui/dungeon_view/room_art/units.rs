//! Defender and adventurer composition inside a single dungeon room tile.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use crate::game_state::{Adventurer, Monster, Room, RoomType};
use crate::ui::theme::*;

use super::super::DungeonSprites;

// The cutaway is read through its occupants first. Sprites therefore carry
// real visual weight against furniture while overflow still collapses safely.
const UNIT_RADIUS: f32 = 16.0;
const UNIT_STEP: f32 = 38.0;
const UNIT_SPRITE_SIZE: f32 = 52.0;

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
        let mut x = strip.x + strip.w - radius - 1.0;
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
            let initial = monster
                .type_name
                .chars()
                .next()
                .map(|c| c.to_ascii_uppercase().to_string())
                .unwrap_or_else(|| "?".to_string());
            let phase = crate::ui::visual_time() + monster.id as f32 * 0.173;
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
            if monster.fusion_rank >= 2 {
                draw_circle_lines(x, cy, radius + 2.5, 1.5, with_alpha(color, 0.78));
            }
            if monster.fusion_rank >= 3 {
                draw_circle_lines(x, cy, radius + 5.0, 1.2, with_alpha(TREASURE, 0.82));
            }
            if monster.alive && (fighting || monster.hp < monster.max_hp) {
                draw_unit_hp_bar(vec2(x, cy), radius, monster.hp, monster.max_hp);
            }
            x -= step;
            drawn += 1;
        }
        if ordered.len() > drawn {
            draw_text_fit_right(
                &format!("+{}", ordered.len() - drawn),
                x + radius,
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
        let mut x = strip.x + radius + 1.0;
        for adventurer in adventurers.iter().take(shown) {
            let initial = adventurer
                .class_name
                .chars()
                .next()
                .map(|c| c.to_ascii_uppercase().to_string())
                .unwrap_or_else(|| "A".to_string());
            let is_rival = rival_ids.contains(&adventurer.id);
            let phase = crate::ui::visual_time() + adventurer.id as f32 * 0.173;
            let bob = phase.sin() * 1.5;
            let drawn_sprite = sprites.draw_adventurer(
                &adventurer.class_name,
                vec2(x, cy + bob),
                UNIT_SPRITE_SIZE,
                phase,
                false,
                false,
                fighting,
            );
            if !drawn_sprite {
                draw_icon_disc(vec2(x, cy), radius, WARNING, &initial);
            }
            if adventurer.ward.mastery > 0 {
                let ward_color = element_color(&adventurer.ward.element);
                draw_circle_lines(x, cy, radius + 2.5, 1.4, with_alpha(ward_color, 0.84));
                if adventurer.ward.mastery >= 2 {
                    draw_circle_lines(x, cy, radius + 5.0, 1.0, with_alpha(ward_color, 0.50));
                }
            }
            if is_rival {
                let rival_radius = if adventurer.ward.mastery > 0 {
                    radius + 7.5
                } else {
                    radius + 2.5
                };
                draw_circle_lines(x, cy, rival_radius, 1.6, TREASURE);
            }
            if fighting || adventurer.hp < adventurer.max_hp {
                draw_unit_hp_bar(vec2(x, cy), radius, adventurer.hp, adventurer.max_hp);
            }
            x += step;
        }
        if adventurers.len() > shown {
            draw_text_fit(
                &format!("+{}", adventurers.len() - shown),
                x - 2.0,
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
