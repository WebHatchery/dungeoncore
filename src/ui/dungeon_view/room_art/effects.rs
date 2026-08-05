//! Transient, simulation-independent visual effects anchored to room tiles.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use crate::game_state::{EffectAnchor, EffectKind, GameState, Room};
use crate::ui::theme::*;

use super::super::DungeonSprites;

pub(super) fn draw_room_effects(
    state: &GameState,
    room: &Room,
    rect: Rect,
    sprites: &DungeonSprites,
) {
    let mut stack_by_anchor: [i32; 3] = [0, 0, 0];
    for effect in state
        .effects
        .iter()
        .filter(|effect| effect.floor == room.floor_number && effect.room == room.position)
    {
        let (anchor_idx, anchor_x) = match effect.anchor {
            EffectAnchor::Defenders => (0usize, rect.x + rect.w * 0.30),
            EffectAnchor::Invaders => (1usize, rect.x + rect.w * 0.70),
            EffectAnchor::Center => (2usize, rect.x + rect.w * 0.5),
        };
        let stack = stack_by_anchor[anchor_idx];
        stack_by_anchor[anchor_idx] += 1;

        let life = effect.life_fraction();
        draw_room_effect_shape(
            effect.kind,
            rect,
            effect.anchor,
            life,
            effect.visual_unit.as_deref(),
            effect.visual_element.as_deref(),
            sprites,
        );
        let rise = (1.0 - life) * 28.0 + stack as f32 * 15.0;
        let color = effect_color(effect.kind);
        let cx = anchor_x;
        let cy = rect.y + rect.h * 0.36 - rise;
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
            with_alpha(color, life),
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
        EffectKind::MeleeDust => Color::new(0.78, 0.66, 0.47, 1.0),
        EffectKind::HitSpark => Color::new(1.0, 0.94, 0.55, 1.0),
    }
}

fn draw_room_effect_shape(
    kind: EffectKind,
    rect: Rect,
    anchor: EffectAnchor,
    life: f32,
    visual_unit: Option<&str>,
    visual_element: Option<&str>,
    sprites: &DungeonSprites,
) {
    let (cx, cy) = match anchor {
        EffectAnchor::Defenders => (rect.x + rect.w * 0.34, rect.y + rect.h * 0.48),
        EffectAnchor::Invaders => (rect.x + rect.w * 0.66, rect.y + rect.h * 0.48),
        EffectAnchor::Center => (rect.x + rect.w * 0.50, rect.y + rect.h * 0.46),
    };
    match kind {
        EffectKind::MeleeDust => {
            let radius = 13.0 + (1.0 - life) * 15.0;
            for (dx, dy, scale) in [(-0.65, 0.35, 0.62), (0.58, 0.28, 0.70), (0.0, -0.18, 1.0)] {
                draw_circle(
                    cx + dx * radius,
                    cy + dy * radius,
                    radius * scale,
                    with_alpha(Color::new(0.75, 0.63, 0.45, 1.0), life * 0.55),
                );
            }
        }
        EffectKind::HitSpark => {
            let radius = 5.0 + life * 8.0;
            let color = with_alpha(element_impact_color(visual_element), life);
            draw_line(cx - radius, cy, cx + radius, cy, 2.0, color);
            draw_line(cx, cy - radius, cx, cy + radius, 2.0, color);
            draw_line(
                cx - radius * 0.7,
                cy - radius * 0.7,
                cx + radius * 0.7,
                cy + radius * 0.7,
                1.4,
                color,
            );
            draw_element_impact(visual_element, cx, cy, radius, life);
        }
        EffectKind::MonsterDown | EffectKind::AdventurerDown => {
            if let Some(unit) = visual_unit {
                let monster = kind == EffectKind::MonsterDown;
                if sprites.draw_death(monster, unit, vec2(cx, cy), 30.0 * life, 0.0, !monster) {
                    return;
                }
            }
            let color = if kind == EffectKind::MonsterDown {
                DANGER
            } else {
                EMERALD
            };
            draw_circle(
                cx,
                cy + (1.0 - life) * 8.0,
                10.0 * life,
                with_alpha(color, life * 0.55),
            );
        }
        _ => {}
    }
}

fn element_impact_color(element: Option<&str>) -> Color {
    match element.unwrap_or_default() {
        "Fire" => Color::new(1.0, 0.38, 0.10, 1.0),
        "Water" => Color::new(0.30, 0.78, 1.0, 1.0),
        "Nature" => Color::new(0.45, 0.92, 0.32, 1.0),
        "Earth" => Color::new(0.72, 0.48, 0.25, 1.0),
        "Air" => Color::new(0.86, 0.96, 1.0, 1.0),
        "Spirit" => Color::new(1.0, 0.82, 0.35, 1.0),
        "Death" => Color::new(0.60, 0.34, 0.78, 1.0),
        "Arcane" => Color::new(0.78, 0.42, 1.0, 1.0),
        "Body" => Color::new(1.0, 0.88, 0.62, 1.0),
        _ => Color::new(1.0, 0.92, 0.42, 1.0),
    }
}

fn draw_element_impact(element: Option<&str>, cx: f32, cy: f32, radius: f32, life: f32) {
    let color = with_alpha(element_impact_color(element), life * 0.72);
    match element.unwrap_or_default() {
        "Fire" => {
            draw_circle(cx, cy - radius * 0.32, radius * 0.46, color);
            draw_circle(cx + radius * 0.36, cy + radius * 0.18, radius * 0.28, color);
        }
        "Water" | "Air" => {
            draw_circle_lines(cx, cy, radius * 0.70, 1.2, color);
            draw_circle_lines(cx, cy, radius * 0.38, 1.0, color);
        }
        "Nature" => {
            draw_circle(cx - radius * 0.42, cy + radius * 0.34, radius * 0.18, color);
            draw_circle(cx + radius * 0.38, cy - radius * 0.20, radius * 0.24, color);
            draw_circle(cx + radius * 0.08, cy + radius * 0.42, radius * 0.15, color);
        }
        "Earth" => {
            draw_rectangle(
                cx - radius * 0.58,
                cy + radius * 0.24,
                radius * 0.42,
                radius * 0.32,
                color,
            );
            draw_rectangle(
                cx + radius * 0.12,
                cy - radius * 0.08,
                radius * 0.44,
                radius * 0.44,
                color,
            );
        }
        "Death" | "Arcane" | "Spirit" => {
            draw_circle_lines(cx, cy, radius * 0.80, 1.5, color);
            draw_circle(cx, cy, radius * 0.20, color);
        }
        _ => {}
    }
}
