//! Room tile composition: chamber art, unit icons, floating effects, labels,
//! the build-here ghost tile, and route connectors.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::game_state::{Adventurer, GameState, Room, RoomType, RoomUpgradeType};
use crate::ui::theme::*;

use super::icons::{
    draw_combat_art, draw_core_art, draw_dashed_border, draw_entrance_art, draw_room_icon,
};
use super::DungeonSprites;
use super::{adventurers_in_room, BuildPreview, PlacementState};

mod effects;
mod units;

use effects::draw_room_effects;
use units::draw_room_units;

pub(super) fn draw_room_tile(
    state: &GameState,
    room: &Room,
    rect: Rect,
    placement: PlacementState,
    sprites: &DungeonSprites,
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
    if let Some((element, multiplier)) = room.attunement() {
        draw_pill(
            Rect::new(draw_rect.x + 7.0, draw_rect.y + 29.0, 58.0, 15.0),
            &format!(
                "{} +{:.0}%",
                element_marker(element),
                (multiplier - 1.0) * 100.0
            ),
            element_color(element),
        );
    }
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
                            Rect::new(draw_rect.x + 7.0, draw_rect.y + 29.0, 76.0, 15.0),
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
            // A combat room refusing the armed thing says which kind of "no"
            // it is: out of slots, or already carrying one of these.
            if room.room_type == RoomType::Normal || room.room_type == RoomType::Boss {
                let refusal = if let Some(monster) = &state.selected_monster {
                    crate::simulation::monsters::monster_placement_refusal(room, monster)
                } else if state
                    .selected_upgrade
                    .as_deref()
                    .map(|upgrade| super::room_holds_upgrade_kind(room, upgrade))
                    .unwrap_or(false)
                {
                    Some("Has one")
                } else {
                    None
                };
                if let Some(label) = refusal {
                    draw_pill(
                        Rect::new(
                            draw_rect.x + draw_rect.w - 60.0,
                            draw_rect.y + draw_rect.h * 0.46,
                            55.0,
                            16.0,
                        ),
                        label,
                        WARNING,
                    );
                }
            }
        }
        PlacementState::Idle => {}
    }

    let label_rect = Rect::new(
        draw_rect.x + 7.0,
        draw_rect.y + 6.0,
        draw_rect.w - 14.0,
        20.0,
    );
    draw_room_label_plate(label_rect, title, room, icon_color);

    // Per-unit icons: defenders on the left, adventurers on the right.
    let strip = Rect::new(
        draw_rect.x + 8.0,
        draw_rect.y + draw_rect.h - 34.0,
        draw_rect.w - 16.0,
        27.0,
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
    draw_room_units(room, strip, &adventurers, fighting, &rival_ids, sprites);

    // Floating combat feedback (damage numbers, kills) rising over the room.
    draw_room_effects(state, room, draw_rect, sprites);

    was_clicked_rect(rect) && !state.board_dragged
}

fn draw_room_chamber_art(rect: Rect, room: &Room, fill: Color, border: Color, icon_color: Color) {
    // A heavy silhouette around a brighter interior gives every floor the
    // readable side-on cutaway shape of an inhabited underground complex.
    draw_rectangle(
        rect.x - 3.0,
        rect.y - 4.0,
        rect.w + 6.0,
        rect.h + 9.0,
        BLACK,
    );
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, with_alpha(border, 0.46));
    let wall = Rect::new(rect.x + 5.0, rect.y + 7.0, rect.w - 10.0, rect.h - 14.0);
    draw_rectangle(wall.x, wall.y, wall.w, wall.h, fill);
    draw_rectangle(
        wall.x,
        wall.y,
        wall.w,
        wall.h * 0.62,
        Color::new(0.0, 0.0, 0.0, 0.10),
    );

    // Depth is a visual material change, not a stat or simulation effect:
    // shallow halls keep their stone, while the lower dungeon gradually takes
    // on the Core's cool arcane light.
    let depth = ((room.floor_number - 1).max(0) as f32 / 19.0).clamp(0.0, 1.0);
    let depth_tint = Color::new(
        0.10 + depth * 0.17,
        0.16 + depth * 0.03,
        0.28 + depth * 0.30,
        depth * 0.26,
    );
    draw_rectangle(wall.x, wall.y, wall.w, wall.h, depth_tint);

    let brick = with_alpha(icon_color, 0.10);
    let mut by = wall.y + 18.0;
    let mut row = 0usize;
    while by < wall.y + wall.h - 13.0 {
        draw_line(wall.x + 3.0, by, wall.x + wall.w - 3.0, by, 1.0, brick);
        let offset = if row % 2 == 0 { 10.0 } else { 22.0 };
        let mut bx = wall.x + offset;
        while bx < wall.x + wall.w - 4.0 {
            draw_line(bx, by - 14.0, bx, by, 1.0, brick);
            bx += 28.0;
        }
        by += 14.0;
        row += 1;
    }

    // Stone ceiling, side supports, and a lit floor anchor the room as a
    // physical chamber instead of a floating UI card.
    draw_rectangle(wall.x, wall.y, wall.w, 5.0, with_alpha(BLACK, 0.62));
    draw_rectangle(
        wall.x,
        wall.y + wall.h - 10.0,
        wall.w,
        10.0,
        with_alpha(BLACK, 0.70),
    );
    draw_line(
        wall.x,
        wall.y + wall.h - 11.0,
        wall.x + wall.w,
        wall.y + wall.h - 11.0,
        1.5,
        with_alpha(icon_color, 0.38),
    );
    for x in [wall.x + 3.0, wall.x + wall.w - 6.0] {
        draw_rectangle(x, wall.y + 5.0, 4.0, wall.h - 15.0, with_alpha(BLACK, 0.34));
    }

    match room.room_type {
        RoomType::Entrance => draw_entrance_art(wall, icon_color),
        RoomType::Normal | RoomType::Boss => draw_combat_art(wall, icon_color),
        RoomType::Core => draw_core_art(wall, icon_color),
    }
    draw_room_upgrade_art(wall, room);
}

/// Give installed room upgrades a compact, distinct physical presence inside
/// the chamber. These cues supplement the inspector and never cover unit rows.
fn draw_room_upgrade_art(wall: Rect, room: &Room) {
    for upgrade in &room.upgrades {
        let element = upgrade.element.as_deref().unwrap_or_default();
        let elemental_color = element_color(element);
        match upgrade.upgrade_type {
            RoomUpgradeType::Trap => {
                let color = if upgrade.disarmed {
                    TEXT_DIM
                } else if element.is_empty() {
                    WARNING
                } else {
                    elemental_color
                };
                let y = wall.y + 9.0;
                for offset in [0.0, 10.0, 20.0] {
                    let x = wall.x + wall.w * 0.5 - 10.0 + offset;
                    draw_triangle(
                        vec2(x - 4.0, y + 7.0),
                        vec2(x, y),
                        vec2(x + 4.0, y + 7.0),
                        with_alpha(color, if upgrade.disarmed { 0.34 } else { 0.78 }),
                    );
                }
                if upgrade.disarmed {
                    draw_line(
                        wall.x + wall.w * 0.5 - 16.0,
                        y + 2.0,
                        wall.x + wall.w * 0.5 + 16.0,
                        y + 10.0,
                        1.5,
                        TEXT_DIM,
                    );
                }
            }
            RoomUpgradeType::Treasure => {
                let chest = Rect::new(wall.x + 10.0, wall.y + 11.0, 15.0, 11.0);
                draw_rectangle(
                    chest.x,
                    chest.y,
                    chest.w,
                    chest.h,
                    with_alpha(TREASURE, 0.68),
                );
                draw_rectangle_lines(chest.x, chest.y, chest.w, chest.h, 1.0, TREASURE);
                draw_line(
                    chest.x,
                    chest.y + 4.0,
                    chest.x + chest.w,
                    chest.y + 4.0,
                    1.0,
                    BLACK,
                );
                draw_circle(chest.x + chest.w * 0.5, chest.y + 5.0, 1.5, SOUL);
            }
            RoomUpgradeType::Reinforcement => {
                let center = vec2(wall.x + wall.w - 17.0, wall.y + 17.0);
                let color = Color::new(0.68, 0.76, 0.84, 1.0);
                draw_triangle(
                    vec2(center.x, center.y - 9.0),
                    vec2(center.x - 8.0, center.y - 4.0),
                    vec2(center.x + 8.0, center.y - 4.0),
                    with_alpha(color, 0.52),
                );
                draw_line(
                    center.x - 8.0,
                    center.y - 4.0,
                    center.x - 5.0,
                    center.y + 8.0,
                    1.5,
                    color,
                );
                draw_line(
                    center.x + 8.0,
                    center.y - 4.0,
                    center.x + 5.0,
                    center.y + 8.0,
                    1.5,
                    color,
                );
                draw_line(
                    center.x - 5.0,
                    center.y + 8.0,
                    center.x + 5.0,
                    center.y + 8.0,
                    1.5,
                    color,
                );
            }
            RoomUpgradeType::Evolution => {
                let center = vec2(wall.x + 18.0, wall.y + wall.h * 0.48);
                draw_circle_lines(center.x, center.y, 8.0, 1.2, with_alpha(SOUL, 0.72));
                draw_line(
                    center.x - 5.0,
                    center.y + 4.0,
                    center.x + 5.0,
                    center.y - 4.0,
                    1.4,
                    SOUL,
                );
                draw_circle(center.x - 5.0, center.y + 4.0, 1.5, SOUL);
                draw_circle(center.x + 5.0, center.y - 4.0, 1.5, SOUL);
            }
            RoomUpgradeType::Attunement => {
                let center = vec2(wall.x + wall.w - 18.0, wall.y + wall.h * 0.48);
                draw_circle(center.x, center.y, 9.0, with_alpha(elemental_color, 0.18));
                draw_triangle(
                    vec2(center.x, center.y - 8.0),
                    vec2(center.x - 6.0, center.y + 4.0),
                    vec2(center.x, center.y + 8.0),
                    with_alpha(elemental_color, 0.74),
                );
                draw_triangle(
                    vec2(center.x, center.y - 8.0),
                    vec2(center.x + 6.0, center.y + 4.0),
                    vec2(center.x, center.y + 8.0),
                    with_alpha(elemental_color, 0.48),
                );
            }
        }
    }
}

fn draw_room_label_plate(rect: Rect, title: &str, room: &Room, color: Color) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.0, 0.0, 0.0, 0.62),
    );
    draw_line(
        rect.x,
        rect.y + rect.h,
        rect.x + rect.w,
        rect.y + rect.h,
        1.0,
        with_alpha(color, 0.46),
    );
    draw_text_fit(
        title,
        rect.x + 22.0,
        rect.y + 14.0,
        rect.w - 28.0,
        13.0,
        TEXT,
    );
    draw_room_icon(
        &room.room_type,
        vec2(rect.x + 11.0, rect.y + rect.h * 0.50),
        8.0,
        color,
    );

    // Combat rooms carry their occupancy: slots are the scarce build resource,
    // so the board states them without needing a click.
    if room.room_type == RoomType::Normal || room.room_type == RoomType::Boss {
        let capacity = crate::data::constants::room_capacity(room);
        let used = room.monsters.len();
        draw_text_fit_right(
            &format!("{}/{}", used, capacity),
            rect.x + rect.w - 5.0,
            rect.y + 14.0,
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

    draw_rectangle(
        draw_rect.x - 3.0,
        draw_rect.y - 4.0,
        draw_rect.w + 6.0,
        draw_rect.h + 9.0,
        BLACK,
    );
    draw_card(draw_rect, fill, border);
    draw_dashed_border(draw_rect, with_alpha(border, 0.72));
    draw_centered_text(
        "+",
        Rect::new(
            draw_rect.x,
            draw_rect.y + draw_rect.h * 0.20,
            draw_rect.w,
            28.0,
        ),
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
        draw_rect.x + 7.0,
        draw_rect.y + 6.0,
        draw_rect.w - 14.0,
        20.0,
    );
    draw_card(
        label_rect,
        Color::new(0.0, 0.0, 0.0, 0.34),
        with_alpha(border, 0.34),
    );
    draw_centered_text(&label, label_rect, 11.0, TEXT_MUTED);
    draw_centered_text(
        &format!("{}M", plan.cost),
        Rect::new(
            draw_rect.x,
            draw_rect.y + draw_rect.h - 25.0,
            draw_rect.w,
            14.0,
        ),
        11.0,
        if can_afford { MANA } else { DANGER },
    );

    was_clicked_rect(rect)
}

/// Draw a party marker gliding along a corridor connector at `progress` (0..1),
/// so a party visibly crosses from one room to the next instead of teleporting.
pub(super) fn draw_party_transit(
    connector: Rect,
    progress: f32,
    members: &[Adventurer],
    sprites: &DungeonSprites,
) {
    let cx = connector.x + connector.w * progress;
    let cy = connector.y + connector.h * 0.5;
    // A short motion trail behind the marker sells the direction of travel.
    draw_circle(cx - 5.0, cy, 4.0, with_alpha(WARNING, 0.16));
    let shown = members.iter().filter(|member| member.alive).take(3).count();
    for (index, member) in members
        .iter()
        .filter(|member| member.alive)
        .take(3)
        .enumerate()
    {
        let offset = (index as f32 - (shown.saturating_sub(1) as f32) * 0.5) * 10.0;
        let center = vec2(cx + offset, cy + (index % 2) as f32 * 5.0 - 2.5);
        if !sprites.draw_adventurer(
            &member.class_name,
            center,
            16.0,
            get_time() as f32 + member.id as f32 * 0.173,
            true,
            false,
            true,
        ) {
            draw_icon_disc(center, 6.5, WARNING, "A");
        }
    }
    let alive = members.iter().filter(|member| member.alive).count();
    if alive > shown {
        draw_text_fit(
            &format!("+{}", alive - shown),
            cx + 14.0,
            cy + 4.0,
            22.0,
            10.0,
            WARNING,
        );
    }
}

pub(super) fn draw_connector(rect: Rect, ghost: bool) {
    let alpha = if ghost { 0.32 } else { 0.70 };
    draw_rectangle(
        rect.x,
        rect.y - 5.0,
        rect.w,
        rect.h + 10.0,
        with_alpha(BLACK, alpha),
    );
    let passage = Rect::new(rect.x, rect.y + 3.0, rect.w, rect.h - 6.0);
    draw_rectangle(
        passage.x,
        passage.y,
        passage.w,
        passage.h,
        Color::new(0.18, 0.16, 0.14, alpha),
    );
    draw_line(
        rect.x,
        rect.y + rect.h * 0.5,
        rect.x + rect.w,
        rect.y + rect.h * 0.5,
        1.5,
        with_alpha(TREASURE, if ghost { 0.22 } else { 0.36 }),
    );
}

fn room_tone(room: &Room) -> (Color, Color, Color, &'static str) {
    match room.room_type {
        RoomType::Entrance => (
            Color::new(0.075, 0.25, 0.17, 0.98),
            with_alpha(EMERALD, 0.78),
            Color::new(0.70, 1.00, 0.82, 1.0),
            "Entrance",
        ),
        RoomType::Normal => (
            Color::new(0.16, 0.19, 0.22, 0.98),
            Color::new(0.38, 0.42, 0.46, 0.92),
            Color::new(0.94, 0.62, 0.28, 1.0),
            "Room",
        ),
        RoomType::Boss => (
            Color::new(0.34, 0.15, 0.08, 0.98),
            with_alpha(WARNING, 0.78),
            Color::new(1.0, 0.80, 0.58, 1.0),
            "Boss",
        ),
        RoomType::Core => (
            Color::new(0.24, 0.11, 0.34, 0.98),
            with_alpha(SOUL, 0.82),
            Color::new(0.93, 0.78, 1.0, 1.0),
            "Core",
        ),
    }
}
