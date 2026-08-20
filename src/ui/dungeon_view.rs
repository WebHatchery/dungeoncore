//! The dungeon board: floor rows of room tiles from entrance to core, plus the
//! build-here ghost tile. Rendering detail lives in submodules: [`backdrop`]
//! (surface/skyline/rails), [`room_art`] (tile composition), and [`rooms`]
//! (layout, route choice, and placement affordances).

use macroquad::prelude::*;

use crate::game_state::{GameState, RoomType};

use super::theme::*;

mod backdrop;
mod camera;
mod icons;
mod layout;
mod room_art;
mod rooms;
mod sprites;

use backdrop::{
    begin_cutaway_clip, draw_board_surface, draw_cutaway_frame, draw_floor_structure,
    draw_lift_extension, draw_lift_shaft, end_cutaway_clip,
};
use camera::{draw_zoom_controls, update as update_camera};
use macroquad_toolkit::colors::with_alpha;
use rooms::{
    adventurers_in_room, current_objective, draw_floor_rooms, draw_placement_badge,
    draw_route_choice, floor_world_height, floor_world_width, next_build_preview,
    room_holds_upgrade_kind, sorted_floors,
};
pub use sprites::{
    DungeonSprites, ANIMATED_ADVENTURER_SHEET_KEY, ANIMATED_ADVENTURER_SHEET_PATH,
    ANIMATED_FULL_MONSTER_SHEET_KEY, ANIMATED_FULL_MONSTER_SHEET_PATH, ANIMATED_MONSTER_SHEET_KEY,
    ANIMATED_MONSTER_SHEET_PATH, ANIMATED_UNIT_SHEET_KEY, ANIMATED_UNIT_SHEET_PATH,
    GIANT_RAT_SPRITE_KEY, GIANT_RAT_SPRITE_PATH, UNIT_SHEET_KEY, UNIT_SHEET_PATH,
};

// Rooms are world-sized pieces of one physical base. The camera moves across
// them as the dungeon grows; these values must never be fitted down to the
// current viewport width.
const BASE_ROOM_W: f32 = 300.0;
const BASE_ROOM_H: f32 = 230.0;
const BASE_CONNECTOR_W: f32 = 8.0;
const BASE_SLAB_H: f32 = 14.0;
const BASE_SHAFT_W: f32 = 54.0;
const WORLD_MARGIN: f32 = 18.0;
const BOARD_HEADER_H: f32 = 60.0;
const COMPACT_BOARD_HEADER_H: f32 = 100.0;

fn board_header_height(rect: Rect) -> f32 {
    if rect.w < 760.0 {
        COMPACT_BOARD_HEADER_H
    } else {
        BOARD_HEADER_H
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonAction {
    None,
    RoomSelected(i32, usize),
    BuildRoom,
}

/// Directional keyboard movement through the dungeon graph and its floor rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomNavigation {
    Left,
    Right,
    Up,
    Down,
}

/// Pick the next inspectable room for a directional keyboard command. Horizontal
/// movement follows graph edges; vertical movement preserves the nearest room
/// position across adjacent floors, so it stays predictable on forked layouts.
pub fn keyboard_room_selection(
    state: &GameState,
    direction: RoomNavigation,
) -> Option<(i32, usize)> {
    let floors = sorted_floors(state);
    let current = state.selected_room.or_else(|| {
        floors.first().and_then(|floor| {
            floor
                .rooms
                .iter()
                .find(|room| room.room_type == RoomType::Entrance)
                .map(|room| (floor.number, room.position))
        })
    })?;
    let floor_index = floors.iter().position(|floor| floor.number == current.0)?;
    let floor = floors[floor_index];
    let room = floor.room_at(current.1)?;

    match direction {
        RoomNavigation::Right => room.exits.first().map(|&position| (floor.number, position)),
        RoomNavigation::Left => floor
            .rooms
            .iter()
            .filter(|candidate| candidate.exits.contains(&room.position))
            .min_by_key(|candidate| candidate.position)
            .map(|candidate| (floor.number, candidate.position)),
        RoomNavigation::Up | RoomNavigation::Down => {
            let next_floor_index = match direction {
                RoomNavigation::Up => floor_index.checked_sub(1),
                RoomNavigation::Down => (floor_index + 1 < floors.len()).then_some(floor_index + 1),
                _ => None,
            }?;
            floors[next_floor_index]
                .rooms
                .iter()
                .min_by_key(|candidate| candidate.position.abs_diff(room.position))
                .map(|candidate| (floors[next_floor_index].number, candidate.position))
        }
    }
}

/// Draw the production-style dungeon board and return the selected board action.
pub fn draw_dungeon_board(
    state: &mut GameState,
    rect: Rect,
    sprites: &DungeonSprites,
) -> DungeonAction {
    let mut action = DungeonAction::None;
    let header_h = board_header_height(rect);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, BG_DEEP);
    draw_rectangle(rect.x, rect.y, rect.w, header_h, PANEL);
    draw_line(
        rect.x,
        rect.y + header_h,
        rect.x + rect.w,
        rect.y + header_h,
        1.0,
        with_alpha(TREASURE, 0.22),
    );
    draw_text_fit("Dungeon", rect.x + 22.0, rect.y + 27.0, 160.0, 23.0, TEXT);
    draw_text_fit(
        &current_objective(state),
        rect.x + 22.0,
        rect.y + 48.0,
        (rect.w * 0.48).max(240.0),
        12.0,
        TEXT_MUTED,
    );

    if rect.w >= 760.0 {
        if let Some(monster) = &state.selected_monster {
            draw_placement_badge(state, monster, rect);
        }
    }
    draw_route_choice(state, rect);
    draw_zoom_controls(state, rect);

    let content = Rect::new(
        rect.x + 2.0,
        rect.y + header_h + 1.0,
        rect.w - 4.0,
        rect.h - header_h - 3.0,
    );
    draw_board_surface(content);

    if state.floors.is_empty() {
        draw_centered_text("No dungeon mapped", content, 18.0, TEXT_MUTED);
        draw_cutaway_frame(content);
        return action;
    }

    let preview = next_build_preview(state);
    let zoom = state.board_zoom;
    let world_h = state
        .floors
        .iter()
        .map(|floor| floor_world_height(floor, zoom))
        .sum::<f32>();
    let widest_world = state
        .floors
        .iter()
        .map(|floor| floor_world_width(floor, preview.as_ref(), zoom))
        .fold(0.0, f32::max);
    let max_pan = (widest_world + WORLD_MARGIN * 2.0 - content.w).max(0.0);
    let max_scroll = (world_h + WORLD_MARGIN * 2.0 - content.h).max(0.0);
    update_camera(state, content, max_pan, max_scroll);

    let world_x = if widest_world + WORLD_MARGIN * 2.0 <= content.w {
        content.x + (content.w - widest_world) * 0.5
    } else {
        content.x + WORLD_MARGIN - state.board_pan_x
    };
    let mut floor_y = if world_h + WORLD_MARGIN * 2.0 <= content.h {
        content.y + (content.h - world_h) * 0.5
    } else {
        content.y + WORLD_MARGIN - state.board_scroll
    };

    if max_scroll > 0.0 {
        draw_text_fit(
            &format!("Depth {:.0}%", state.board_scroll / max_scroll * 100.0),
            rect.x + rect.w - 150.0,
            rect.y + 48.0,
            132.0,
            10.0,
            TEXT_DIM,
        );
    }

    begin_cutaway_clip(content);
    let shaft_w = BASE_SHAFT_W * zoom;
    if floor_y > content.y + 7.0 {
        draw_lift_extension(Rect::new(
            world_x,
            content.y + 7.0,
            shaft_w,
            floor_y - content.y - 7.0,
        ));
    }
    let floors = sorted_floors(state);
    for floor in floors {
        let floor_h = floor_world_height(floor, zoom);
        if floor_y + floor_h < content.y || floor_y > content.y + content.h {
            floor_y += floor_h;
            continue;
        }
        let floor_world_w = floor_world_width(floor, preview.as_ref(), zoom);
        let floor_rect = Rect::new(world_x, floor_y, floor_world_w, floor_h);
        let selected_floor = state
            .selected_room
            .map(|(floor_num, _)| floor_num == floor.number)
            .unwrap_or(false);
        draw_floor_structure(floor_rect, floor.number, selected_floor);
        let shaft = Rect::new(
            world_x,
            floor_y,
            BASE_SHAFT_W * zoom,
            floor_h - BASE_SLAB_H * zoom,
        );
        draw_lift_shaft(shaft, floor.number, floor.is_deepest);
        let rooms_area = Rect::new(
            shaft.x + shaft.w + BASE_CONNECTOR_W * zoom,
            floor_y,
            floor_world_w - shaft.w - BASE_CONNECTOR_W * zoom,
            floor_h - BASE_SLAB_H * zoom,
        );
        if let Some(row_action) =
            draw_floor_rooms(state, floor, rooms_area, content, preview.as_ref(), sprites)
        {
            action = row_action;
        }
        floor_y += floor_h;
    }
    if floor_y < content.y + content.h - 7.0 {
        draw_lift_extension(Rect::new(
            world_x,
            floor_y,
            shaft_w,
            content.y + content.h - floor_y - 7.0,
        ));
    }
    end_cutaway_clip();
    draw_cutaway_frame(content);

    action
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
    room_type: RoomType,
    cost: i32,
    new_floor: bool,
}

#[cfg(test)]
mod tests;
