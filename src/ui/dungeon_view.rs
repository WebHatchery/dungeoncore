//! The dungeon board: floor rows of room tiles from entrance to core, plus the
//! build-here ghost tile. Rendering detail lives in submodules: [`backdrop`]
//! (surface/skyline/rails), [`room_art`] (tile composition), and [`icons`]
//! (vector glyphs).

use macroquad::prelude::*;

use crate::data::constants::{get_room_cost, MAX_ROOMS_PER_FLOOR};
use crate::game_state::{DungeonStatus, GameState, Room, RoomType};

use super::theme::*;

mod backdrop;
mod camera;
mod icons;
mod layout;
mod room_art;
mod sprites;

use backdrop::{
    begin_cutaway_clip, draw_board_surface, draw_cutaway_frame, draw_floor_structure,
    draw_lift_extension, draw_lift_shaft, end_cutaway_clip,
};
use camera::{draw_zoom_controls, update as update_camera};
use layout::layout_floor;
use macroquad_toolkit::colors::with_alpha;
use room_art::{
    draw_connector, draw_future_room_tile, draw_party_transit, draw_room_tile, draw_route_tunnel,
};
pub use sprites::{
    DungeonSprites, ANIMATED_ADVENTURER_SHEET_KEY, ANIMATED_ADVENTURER_SHEET_PATH,
    ANIMATED_FULL_MONSTER_SHEET_KEY, ANIMATED_FULL_MONSTER_SHEET_PATH, ANIMATED_MONSTER_SHEET_KEY,
    ANIMATED_MONSTER_SHEET_PATH, ANIMATED_UNIT_SHEET_KEY, ANIMATED_UNIT_SHEET_PATH, UNIT_SHEET_KEY,
    UNIT_SHEET_PATH,
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
pub fn draw_dungeon_board(
    state: &mut GameState,
    rect: Rect,
    sprites: &DungeonSprites,
) -> DungeonAction {
    let mut action = DungeonAction::None;
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, BG_DEEP);
    draw_rectangle(rect.x, rect.y, rect.w, BOARD_HEADER_H, PANEL);
    draw_line(
        rect.x,
        rect.y + BOARD_HEADER_H,
        rect.x + rect.w,
        rect.y + BOARD_HEADER_H,
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

    if let Some(monster) = &state.selected_monster {
        draw_placement_badge(state, monster, rect);
    }
    draw_route_choice(state, rect);
    draw_zoom_controls(state, rect);

    let content = Rect::new(
        rect.x + 2.0,
        rect.y + BOARD_HEADER_H + 1.0,
        rect.w - 4.0,
        rect.h - BOARD_HEADER_H - 3.0,
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

fn floor_world_width(
    floor: &crate::game_state::Floor,
    preview: Option<&BuildPreview>,
    zoom: f32,
) -> f32 {
    let branching = floor
        .rooms
        .iter()
        .any(|room| room.room_type != RoomType::Core && room.exits.len() != 1);
    if branching {
        let layout = layout_floor(floor);
        let columns = layout.iter().map(|node| node.depth).max().unwrap_or(0) + 1;
        let room_w = BASE_ROOM_W * zoom;
        let step_x = room_w + (BASE_CONNECTOR_W + 18.0) * zoom;
        let rooms_w = columns as f32 * step_x - (step_x - room_w);
        return BASE_SHAFT_W * zoom + BASE_CONNECTOR_W * zoom + rooms_w;
    }
    let has_preview = preview.is_some_and(|plan| plan.floor == floor.number);
    let nodes = floor.rooms.len() + usize::from(has_preview);
    let rooms_w = nodes as f32 * BASE_ROOM_W * zoom;
    let seams_w = nodes.saturating_sub(1) as f32 * BASE_CONNECTOR_W * zoom;
    BASE_SHAFT_W * zoom + BASE_CONNECTOR_W * zoom + rooms_w + seams_w
}

fn floor_world_height(floor: &crate::game_state::Floor, zoom: f32) -> f32 {
    let branching = floor
        .rooms
        .iter()
        .any(|room| room.room_type != RoomType::Core && room.exits.len() != 1);
    let lanes = if branching {
        layout_floor(floor)
            .iter()
            .map(|node| node.lane)
            .max()
            .unwrap_or(0) as usize
            + 1
    } else {
        1
    };
    lanes as f32 * BASE_ROOM_H * zoom + BASE_SLAB_H * zoom
}

/// Make the active fork's decision readable without opening the log. The
/// wording comes from the same pure pathing helper that selects its edge.
fn draw_route_choice(state: &GameState, rect: Rect) {
    let Some(party) = state
        .adventurer_parties
        .iter()
        .find(|party| !party.retreating)
    else {
        return;
    };
    let Some(floor) = state
        .floors
        .iter()
        .find(|floor| floor.number == party.current_floor)
    else {
        return;
    };
    let Some(room) = floor.room_at(party.current_room) else {
        return;
    };
    if room.exits.len() < 2 {
        return;
    }
    let reason = crate::simulation::pathing::choice_reason(state, party, &room.exits);
    let badge = Rect::new(rect.x + rect.w - 236.0, rect.y + 36.0, 214.0, 20.0);
    draw_card(badge, with_alpha(WARNING, 0.12), with_alpha(WARNING, 0.42));
    draw_centered_text(&format!("ROUTE: {reason}"), badge, 10.0, WARNING);
}

fn draw_floor_rooms(
    state: &GameState,
    floor: &crate::game_state::Floor,
    area: Rect,
    viewport: Rect,
    preview: Option<&BuildPreview>,
    sprites: &DungeonSprites,
) -> Option<DungeonAction> {
    if floor
        .rooms
        .iter()
        .any(|room| room.room_type != RoomType::Core && room.exits.len() != 1)
    {
        return draw_graph_rooms(state, floor, area, viewport, sprites);
    }
    let mut action = None;
    let rooms = sorted_rooms(&floor.rooms);
    let future_in_floor = preview.filter(|plan| plan.floor == floor.number);
    let has_future = future_in_floor.is_some();
    let future_before_core = future_in_floor.map(|plan| !plan.new_floor).unwrap_or(false);
    let future_after_core = future_in_floor.map(|plan| plan.new_floor).unwrap_or(false);
    let node_count = rooms.len() + usize::from(has_future);
    let tile_w = BASE_ROOM_W * state.board_zoom;
    let tile_h = BASE_ROOM_H * state.board_zoom;
    let connector_w = BASE_CONNECTOR_W * state.board_zoom;
    let tile_y = area.y;
    let mut x = area.x;
    let mut drawn_nodes = 0usize;
    let total_nodes = node_count;

    for room in rooms {
        if future_before_core && room.room_type == RoomType::Core {
            if let Some(plan) = future_in_floor {
                let future_rect = Rect::new(x, tile_y, tile_w, tile_h);
                if room_rect_visible(future_rect, viewport)
                    && draw_future_room_tile(state, future_rect, viewport, plan)
                {
                    action = Some(DungeonAction::BuildRoom);
                }
                drawn_nodes += 1;
                x += tile_w;
                if drawn_nodes < total_nodes {
                    let connector = Rect::new(x, tile_y, connector_w, tile_h);
                    if room_rect_visible(connector, viewport) {
                        draw_connector(connector, true);
                    }
                    x += connector_w;
                }
            }
        }

        let rect = Rect::new(x, tile_y, tile_w, tile_h);
        let placement = placement_state(state, room);
        if room_rect_visible(rect, viewport)
            && draw_room_tile(state, room, rect, viewport, placement, sprites)
        {
            action = Some(DungeonAction::RoomSelected(
                room.floor_number,
                room.position,
            ));
        }
        drawn_nodes += 1;
        x += tile_w;

        if drawn_nodes < total_nodes {
            let connector = Rect::new(x, tile_y, connector_w, tile_h);
            if room_rect_visible(connector, viewport) {
                draw_connector(connector, false);
            }
            // A party crossing this corridor rides the connector between rooms.
            if room_rect_visible(connector, viewport) {
                if let Some(party) = party_in_transit(
                    state,
                    floor.number,
                    room.position,
                    room.exits.first().copied().unwrap_or(room.position + 1),
                ) {
                    draw_party_transit(
                        connector,
                        party.move_anim.fraction_elapsed(),
                        &party.members,
                        sprites,
                    );
                }
            }
            x += connector_w;
        }
    }

    if future_after_core {
        if let Some(plan) = future_in_floor {
            let future_rect = Rect::new(x, tile_y, tile_w, tile_h);
            if room_rect_visible(future_rect, viewport)
                && draw_future_room_tile(state, future_rect, viewport, plan)
            {
                action = Some(DungeonAction::BuildRoom);
            }
        }
    }

    action
}

/// Draw a branching floor as columns of graph depth and rows of fork lanes.
/// Connectors are drawn first so rooms and units retain a crisp foreground.
fn draw_graph_rooms(
    state: &GameState,
    floor: &crate::game_state::Floor,
    area: Rect,
    viewport: Rect,
    sprites: &DungeonSprites,
) -> Option<DungeonAction> {
    let layout = layout_floor(floor);
    let max_lane = layout.iter().map(|node| node.lane).max().unwrap_or(0) as usize;
    let rows = max_lane + 1;
    let tile_w = BASE_ROOM_W * state.board_zoom;
    let tile_h = BASE_ROOM_H * state.board_zoom;
    let connector_w = BASE_CONNECTOR_W * state.board_zoom;
    let step_x = tile_w + connector_w + 18.0 * state.board_zoom;
    let step_y = area.h / rows.max(1) as f32;
    let origin_x = area.x;
    let center = |depth: usize, lane: i32| {
        vec2(
            origin_x + tile_w * 0.5 + step_x * depth as f32,
            area.y + step_y * (lane as f32 + 0.5),
        )
    };
    let node_at = |position: usize| layout.iter().find(|node| node.position == position);

    for room in &floor.rooms {
        let Some(from) = node_at(room.position) else {
            continue;
        };
        let from_center = center(from.depth, from.lane);
        for &exit in &room.exits {
            let Some(to) = node_at(exit) else { continue };
            let to_center = center(to.depth, to.lane);
            draw_route_tunnel(from_center, to_center, tile_w, tile_h, state.board_zoom);
            if let Some(party) = party_in_transit(state, floor.number, room.position, exit) {
                let progress = party.move_anim.fraction_elapsed();
                let point = from_center.lerp(to_center, progress);
                draw_party_transit(
                    Rect::new(
                        point.x - tile_w * 0.22,
                        point.y - tile_h * 0.12,
                        tile_w * 0.44,
                        tile_h * 0.24,
                    ),
                    progress,
                    &party.members,
                    sprites,
                );
            }
        }
    }
    let mut action = None;
    for node in layout {
        let room = floor.room_at(node.position)?;
        let point = center(node.depth, node.lane);
        let rect = Rect::new(
            point.x - tile_w * 0.5,
            point.y - tile_h * 0.5,
            tile_w,
            tile_h,
        );
        let placement = placement_state(state, room);
        if room_rect_visible(rect, viewport)
            && draw_room_tile(state, room, rect, viewport, placement, sprites)
        {
            action = Some(DungeonAction::RoomSelected(
                room.floor_number,
                room.position,
            ));
        }
    }
    action
}

/// While placing a monster, teach its matchup at the moment of choice: its
/// element (colour-coded) and what that element is strong against. The Codex
/// wheel is reference material; this puts the same knowledge in the funnel.
fn draw_placement_badge(_state: &GameState, monster: &str, rect: Rect) {
    let badge = Rect::new(rect.x + rect.w - 360.0, rect.y + 7.0, 200.0, 46.0);
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
    let combat_room = room.room_type == RoomType::Normal || room.room_type == RoomType::Boss;

    // A monster wants an empty slot; an upgrade wants a room that does not
    // already hold one of its kind. Both light up the same way.
    if let Some(monster) = &state.selected_monster {
        return if combat_room
            && crate::simulation::monsters::monster_placement_refusal(room, monster).is_none()
        {
            PlacementState::Valid
        } else {
            PlacementState::Invalid
        };
    }

    if let Some(upgrade) = &state.selected_upgrade {
        return if combat_room && !room_holds_upgrade_kind(room, upgrade) {
            PlacementState::Valid
        } else {
            PlacementState::Invalid
        };
    }

    PlacementState::Idle
}

/// Whether the room already holds an upgrade of the armed upgrade's type — a
/// room can carry a trap *and* a treasure, but never two traps.
pub(super) fn room_holds_upgrade_kind(room: &Room, upgrade_name: &str) -> bool {
    crate::data::upgrades::get_upgrade_template(upgrade_name)
        .map(|template| {
            room.has_upgrade_type(crate::data::upgrades::parse_upgrade_type(
                &template.upgrade_type,
            ))
        })
        .unwrap_or(false)
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

fn room_rect_visible(rect: Rect, viewport: Rect) -> bool {
    rect.x < viewport.x + viewport.w
        && rect.x + rect.w > viewport.x
        && rect.y < viewport.y + viewport.h
        && rect.y + rect.h > viewport.y
}

/// If a party is currently travelling the corridor leaving `from_pos` on this
/// floor, the 0..1 progress of that glide (0 = just left, 1 = arriving).
fn party_in_transit<'a>(
    state: &'a GameState,
    floor_number: i32,
    from_pos: usize,
    to_pos: usize,
) -> Option<&'a crate::game_state::AdventurerParty> {
    state.adventurer_parties.iter().find_map(|party| {
        if party.current_floor == floor_number
            && !party.move_anim.is_ready()
            && party.prev_room == from_pos
            && party.current_room == to_pos
            && !party.retreating
        {
            Some(party)
        } else {
            None
        }
    })
}

fn current_objective(state: &GameState) -> String {
    if let Some(monster) = &state.selected_monster {
        return format!("Place {monster} in a combat room.");
    }

    if let Some(upgrade) = &state.selected_upgrade {
        return format!("Install {upgrade} in a combat room.");
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

#[cfg(test)]
mod tests;
