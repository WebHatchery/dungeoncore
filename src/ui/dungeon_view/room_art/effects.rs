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
            let color = with_alpha(Color::new(1.0, 0.92, 0.42, 1.0), life);
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
