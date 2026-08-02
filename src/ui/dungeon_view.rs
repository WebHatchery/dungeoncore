//! The dungeon board: floor rows of room tiles from entrance to core, plus the
//! build-here ghost tile. Rendering detail lives in submodules: [`backdrop`]
//! (surface/skyline/rails), [`room_art`] (tile composition), and [`icons`]
//! (vector glyphs).

use macroquad::prelude::*;

use crate::data::constants::{get_room_cost, MAX_ROOMS_PER_FLOOR};
use crate::game_state::{DungeonStatus, GameState, Room, RoomType};

use super::theme::*;

mod backdrop;
mod icons;
mod room_art;

use backdrop::{draw_board_surface, draw_floor_rail, draw_room_route_backplate};
use macroquad_toolkit::colors::with_alpha;
use room_art::{draw_connector, draw_future_room_tile, draw_party_transit, draw_room_tile};

const BASE_ROOM_W: f32 = 156.0;
const BASE_ROOM_H: f32 = 122.0;
const BASE_CONNECTOR_W: f32 = 36.0;
const FLOOR_RAIL_W: f32 = 58.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonAction {
    None,
    RoomSelected(i32, usize),
    BuildRoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlacementState {
    Idle,
    Valid,
    Invalid,
}

#[derive(Debug, Clone)]
struct BuildPreview {
    floor: i32,
    position: usize,
    room_type: RoomType,
    cost: i32,
    new_floor: bool,
}

/// Draw the production-style dungeon board and return the selected board action.
pub fn draw_dungeon_board(state: &GameState, rect: Rect) -> DungeonAction {
    let mut action = DungeonAction::None;
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.18),
        with_alpha(BORDER, 0.18),
    );

    draw_text_fit("Dungeon", rect.x + 22.0, rect.y + 30.0, 160.0, 24.0, TEXT);
    draw_text_fit(
        &current_objective(state),
        rect.x + 22.0,
        rect.y + 54.0,
        (rect.w * 0.48).max(240.0),
        13.0,
        TEXT_MUTED,
    );

    if let Some(monster) = &state.selected_monster {
        draw_placement_badge(state, monster, rect);
    }

    let content = Rect::new(rect.x + 10.0, rect.y + 76.0, rect.w - 20.0, rect.h - 86.0);
    draw_board_surface(content);

    let sorted_floors = sorted_floors(state);
    if sorted_floors.is_empty() {
        draw_centered_text("No dungeon mapped", content, 18.0, TEXT_MUTED);
        return action;
    }

    let gap = 14.0;
    let floor_count = sorted_floors.len() as f32;
    let row_h = ((content.h - gap * (sorted_floors.len().saturating_sub(1) as f32)) / floor_count)
        .clamp(150.0, 224.0);
    let used_h = row_h * floor_count + gap * (sorted_floors.len().saturating_sub(1) as f32);
    let mut row_y = content.y + ((content.h - used_h) * 0.5).max(14.0);
    let preview = next_build_preview(state);

    for floor in sorted_floors {
        if row_y + row_h > content.y + content.h - 2.0 {
            break;
        }

        let row_rect = Rect::new(content.x + 18.0, row_y, content.w - 36.0, row_h);
        let selected_floor = state
            .selected_room
            .map(|(floor_num, _)| floor_num == floor.number)
            .unwrap_or(floor.is_deepest);
        let row_border = if selected_floor {
            with_alpha(TREASURE, 0.48)
        } else {
            BORDER_MUTED
        };
        draw_room_route_backplate(row_rect, selected_floor, row_border);

        let rail = Rect::new(
            row_rect.x + 8.0,
            row_rect.y + 8.0,
            FLOOR_RAIL_W,
            row_rect.h - 16.0,
        );
        draw_floor_rail(rail, floor.number, floor.rooms.len(), floor.is_deepest);

        let rooms_area = Rect::new(
            rail.x + rail.w + 10.0,
            row_rect.y + 8.0,
            row_rect.w - rail.w - 28.0,
            row_rect.h - 16.0,
        );
        if let Some(row_action) = draw_floor_rooms(state, floor, rooms_area, preview.as_ref()) {
            action = row_action;
        }

        row_y += row_h + gap;
    }

    action
}

fn draw_floor_rooms(
    state: &GameState,
    floor: &crate::game_state::Floor,
    area: Rect,
    preview: Option<&BuildPreview>,
) -> Option<DungeonAction> {
    let mut action = None;
    let rooms = sorted_rooms(&floor.rooms);
    let future_in_floor = preview.filter(|plan| plan.floor == floor.number);
    let has_future = future_in_floor.is_some();
    let future_before_core = future_in_floor.map(|plan| !plan.new_floor).unwrap_or(false);
    let future_after_core = future_in_floor.map(|plan| plan.new_floor).unwrap_or(false);
    let node_count = rooms.len() + usize::from(has_future);
    let total_w =
        node_count as f32 * BASE_ROOM_W + node_count.saturating_sub(1) as f32 * BASE_CONNECTOR_W;
    let scale = (area.w / total_w.max(1.0)).clamp(0.58, 1.0);
    let tile_w = BASE_ROOM_W * scale;
    let tile_h = BASE_ROOM_H * scale;
    let connector_w = BASE_CONNECTOR_W * scale;
    let label_h = 32.0 * scale;
    let tile_y = area.y + ((area.h - tile_h - label_h) * 0.5).max(0.0);
    let mut x = area.x + 6.0;
    let mut drawn_nodes = 0usize;
    let total_nodes = node_count;

    for room in rooms {
        if future_before_core && room.room_type == RoomType::Core {
            if let Some(plan) = future_in_floor {
                let future_rect = Rect::new(x, tile_y, tile_w, tile_h);
                if draw_future_room_tile(state, future_rect, plan) {
                    action = Some(DungeonAction::BuildRoom);
                }
                drawn_nodes += 1;
                x += tile_w;
                if drawn_nodes < total_nodes {
                    draw_connector(
                        Rect::new(x, tile_y + tile_h * 0.36, connector_w, tile_h * 0.28),
                        true,
                    );
                    x += connector_w;
                }
            }
        }

        let rect = Rect::new(x, tile_y, tile_w, tile_h);
        let placement = placement_state(state, room);
        if draw_room_tile(state, room, rect, placement) {
            action = Some(DungeonAction::RoomSelected(
                room.floor_number,
                room.position,
            ));
        }
        drawn_nodes += 1;
        x += tile_w;

        if drawn_nodes < total_nodes {
            let connector = Rect::new(x, tile_y + tile_h * 0.36, connector_w, tile_h * 0.28);
            draw_connector(connector, false);
            // A party crossing this corridor rides the connector between rooms.
            if let Some(progress) = party_transit_progress(state, floor.number, room.position) {
                draw_party_transit(connector, progress);
            }
            x += connector_w;
        }
    }

    if future_after_core {
        if let Some(plan) = future_in_floor {
            let future_rect = Rect::new(x, tile_y, tile_w, tile_h);
            if draw_future_room_tile(state, future_rect, plan) {
                action = Some(DungeonAction::BuildRoom);
            }
        }
    }

    action
}

/// While placing a monster, teach its matchup at the moment of choice: its
/// element (colour-coded) and what that element is strong against. The Codex
/// wheel is reference material; this puts the same knowledge in the funnel.
fn draw_placement_badge(_state: &GameState, monster: &str, rect: Rect) {
    let badge = Rect::new(rect.x + rect.w - 360.0, rect.y + 22.0, 200.0, 46.0);
    let element = crate::data::monsters::monster_element_id(monster);
    let accent = element.as_deref().map(element_color).unwrap_or(SOUL);

    draw_card(badge, with_alpha(accent, 0.12), with_alpha(accent, 0.52));
    draw_text_fit(
        &format!("PLACING {}", monster.to_uppercase()),
        badge.x + 12.0,
        badge.y + 19.0,
        badge.w - 20.0,
        13.0,
        accent,
    );
    let sub = match element.as_deref() {
        Some(elem) => {
            let strong = crate::data::elements::get_element(elem)
                .map(|def| def.strong_against.join(", "))
                .filter(|list| !list.is_empty());
            match strong {
                Some(list) => format!("{elem} · strong vs {list}"),
                None => format!("{elem} · neutral element"),
            }
        }
        None => "No element".to_string(),
    };
    draw_text_fit(
        &sub,
        badge.x + 12.0,
        badge.y + 37.0,
        badge.w - 20.0,
        10.0,
        TEXT_MUTED,
    );
}

fn placement_state(state: &GameState, room: &Room) -> PlacementState {
    if state.selected_monster.is_none() {
        return PlacementState::Idle;
    }

    let combat_room = room.room_type == RoomType::Normal || room.room_type == RoomType::Boss;
    if combat_room && !crate::data::constants::room_is_full(room) {
        PlacementState::Valid
    } else {
        PlacementState::Invalid
    }
}

fn adventurer_count_in_room(state: &GameState, room: &Room) -> usize {
    adventurers_in_room(state, room).len()
}

/// The living adventurers currently standing in a room (from any non-retreating
/// party present), so the board can draw each one with its own health bar.
/// Parties mid-corridor (`move_anim` not ready) are excluded — they're drawn
/// gliding along the connector instead, so they don't pop into the destination
/// early.
fn adventurers_in_room<'a>(
    state: &'a GameState,
    room: &Room,
) -> Vec<&'a crate::game_state::Adventurer> {
    state
        .adventurer_parties
        .iter()
        .filter(|party| {
            party.current_floor == room.floor_number
                && party.current_room == room.position
                && !party.retreating
                && party.move_anim.is_ready()
        })
        .flat_map(|party| party.members.iter().filter(|member| member.alive))
        .collect()
}

/// If a party is currently travelling the corridor leaving `from_pos` on this
/// floor, the 0..1 progress of that glide (0 = just left, 1 = arriving).
fn party_transit_progress(state: &GameState, floor_number: i32, from_pos: usize) -> Option<f32> {
    state.adventurer_parties.iter().find_map(|party| {
        if party.current_floor == floor_number
            && !party.move_anim.is_ready()
            && party.prev_room == from_pos
            && party.current_room == from_pos + 1
            && !party.retreating
        {
            Some(party.move_anim.fraction_elapsed())
        } else {
            None
        }
    })
}

fn current_objective(state: &GameState) -> String {
    if let Some(monster) = &state.selected_monster {
        return format!("Place {monster} in a combat room.");
    }

    if !state.adventurer_parties.is_empty() {
        return "Adventurers are inside. Hold the route.".to_string();
    }

    let has_defender = state
        .floors
        .iter()
        .flat_map(|floor| &floor.rooms)
        .any(|room| !room.monsters.is_empty());

    match state.status {
        DungeonStatus::Closed => {
            if !has_defender {
                "Build a room and place a defender, then open the dungeon up top.".to_string()
            } else {
                "Dungeon is closed. Open it (top bar) when you're ready for adventurers."
                    .to_string()
            }
        }
        DungeonStatus::Closing => "Closing... adventurers are finishing their run.".to_string(),
        _ => {
            if state.mana < 20 {
                "Gather mana before expanding.".to_string()
            } else {
                "Build deeper or strengthen a selected room.".to_string()
            }
        }
    }
}

fn sorted_floors(state: &GameState) -> Vec<&crate::game_state::Floor> {
    let mut floors: Vec<_> = state.floors.iter().collect();
    floors.sort_by_key(|floor| floor.number);
    floors
}

fn sorted_rooms(rooms: &[Room]) -> Vec<&Room> {
    let mut sorted: Vec<_> = rooms.iter().collect();
    sorted.sort_by_key(|room| room.position);
    sorted
}

fn next_build_preview(state: &GameState) -> Option<BuildPreview> {
    let deepest = state.deepest_floor()?;
    let non_core_count = deepest
        .rooms
        .iter()
        .filter(|room| room.room_type != RoomType::Core)
        .count();
    let total_rooms = state.total_room_count();

    if non_core_count > MAX_ROOMS_PER_FLOOR {
        let floor = state.total_floors + 1;
        return Some(BuildPreview {
            floor: deepest.number,
            position: 1,
            room_type: RoomType::Normal,
            cost: get_room_cost(total_rooms, false),
            new_floor: true,
        })
        .map(|mut preview| {
            preview.floor = floor;
            preview
        });
    }

    let position = non_core_count;
    let is_boss = position == MAX_ROOMS_PER_FLOOR;
    Some(BuildPreview {
        floor: deepest.number,
        position,
        room_type: if is_boss {
            RoomType::Boss
        } else {
            RoomType::Normal
        },
        cost: get_room_cost(total_rooms, is_boss),
        new_floor: false,
    })
}
