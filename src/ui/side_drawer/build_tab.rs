//! BUILD tab: next-room summary, the build button, and soul-bought core powers.

use macroquad::prelude::*;
use macroquad_toolkit::input::is_hovered_rect;

use crate::data::constants::{get_room_cost, MAX_ROOMS_PER_FLOOR};
use crate::game_state::{GameState, RoomType};
use crate::ui::theme::*;

use super::{draw_section_title, BuildTabAction};
use macroquad_toolkit::colors::with_alpha;

pub(super) fn draw_build_tab(state: &GameState, rect: Rect) -> BuildTabAction {
    draw_section_title(rect, "BUILD", "Shape the dungeon path.");

    let (label, detail, cost) = next_build_summary(state);
    let can_build = state.adventurer_parties.is_empty() && state.mana >= cost;

    let card = Rect::new(rect.x, rect.y + 70.0, rect.w, 126.0);
    draw_card(
        card,
        with_alpha(TREASURE, 0.075),
        with_alpha(TREASURE, 0.24),
    );
    draw_text_fit(
        &label,
        card.x + 12.0,
        card.y + 27.0,
        card.w - 24.0,
        17.0,
        TEXT,
    );
    draw_text_fit(
        &detail,
        card.x + 12.0,
        card.y + 56.0,
        card.w - 24.0,
        12.0,
        TEXT_MUTED,
    );
    draw_text_fit(
        &format!("{cost} mana"),
        card.x + 12.0,
        card.y + 86.0,
        card.w - 24.0,
        14.0,
        if state.mana >= cost { MANA } else { DANGER },
    );
    draw_text_fit(
        if can_build {
            "Click the glowing room or build here."
        } else if state.adventurer_parties.is_empty() {
            "Gather more mana."
        } else {
            "Wait until the dungeon is safe."
        },
        card.x + 12.0,
        card.y + 112.0,
        card.w - 24.0,
        12.0,
        if can_build { EMERALD } else { TEXT_DIM },
    );
    if is_hovered_rect(card) {
        macroquad_toolkit::ui::draw_tooltip(
            &format!(
                "{}\n{}\nCost: {cost} mana. Rooms can only be built between raids.",
                label, detail
            ),
            vec2(card.x, card.y + card.h),
        );
    }

    let build_rect = Rect::new(rect.x, card.y + card.h + 16.0, rect.w, 42.0);
    let build_clicked = draw_command_button(build_rect, "Build", ButtonTone::Arcane, can_build);
    if build_clicked {
        return BuildTabAction::Build;
    }
    if is_hovered_rect(build_rect) {
        macroquad_toolkit::ui::draw_tooltip(
            "Build the next room shown above. A boss chamber appears at the end of each floor; the next build then opens a deeper floor.",
            vec2(build_rect.x, build_rect.y + build_rect.h),
        );
    }

    let branch = state.selected_room.and_then(|(floor, room)| {
        crate::simulation::rooms::branch_cost(state, floor, room)
            .ok()
            .map(|cost| ((floor, room), cost))
    });
    let branch_y = card.y + card.h + 66.0;
    if let Some(((floor, room), branch_cost)) = branch {
        let branch_rect = Rect::new(rect.x, branch_y, rect.w, 34.0);
        if draw_command_button(
            branch_rect,
            &format!("Branch from room {room} · {branch_cost} mana"),
            ButtonTone::Primary,
            state.mana >= branch_cost,
        ) {
            return BuildTabAction::Branch;
        }
        if is_hovered_rect(branch_rect) {
            macroquad_toolkit::ui::draw_tooltip(
                "Create a parallel route that rejoins the selected room's successor. Adventurers can choose between the two paths; reshaping is blocked during a raid.",
                vec2(branch_rect.x, branch_rect.y + branch_rect.h),
            );
        }
        draw_text_fit(
            &format!("Parallel route on floor {floor}; both paths rejoin ahead."),
            rect.x,
            branch_y + 50.0,
            rect.w,
            10.0,
            TEXT_MUTED,
        );
    } else {
        draw_text_fit(
            "Select a room with one route ahead to branch it.",
            rect.x,
            branch_y + 18.0,
            rect.w,
            10.0,
            TEXT_DIM,
        );
    }

    // Permanent, soul-bought core powers live in their own tree overlay so the
    // branching tech tree has room to breathe. Summarise progress and offer the
    // entry point here.
    let owned = crate::simulation::endgame::CORE_POWERS
        .iter()
        .filter(|p| state.has_core_power(p.id))
        .count();
    let total = crate::simulation::endgame::CORE_POWERS.len();
    let y = card.y + card.h + 128.0;
    draw_text_fit(
        &format!("CORE POWERS · {} souls", state.souls),
        rect.x,
        y,
        rect.w,
        10.0,
        SOUL,
    );
    draw_text_fit(
        &format!("{owned}/{total} awakened"),
        rect.x,
        y + 16.0,
        rect.w,
        11.0,
        TEXT_MUTED,
    );
    let powers_rect = Rect::new(rect.x, y + 26.0, rect.w, 40.0);
    if draw_command_button(
        powers_rect,
        "Core Power Tree  [P]",
        ButtonTone::Arcane,
        true,
    ) {
        return BuildTabAction::OpenCorePowers;
    }
    if is_hovered_rect(powers_rect) {
        macroquad_toolkit::ui::draw_tooltip(
            "Spend souls earned from bosses on permanent Core Powers. Their effects survive a siege and reshape the long-term dungeon.",
            vec2(powers_rect.x, powers_rect.y + powers_rect.h),
        );
    }

    // Ongoing gold sink + mana safety-valve: channel surplus gold into mana.
    use crate::simulation::economy::{can_channel_gold, GOLD_CHANNEL_COST, GOLD_CHANNEL_MANA};
    let cy = y + 78.0;
    if cy + 66.0 <= rect.y + rect.h - 44.0 {
        draw_text_fit(
            &format!("CHANNEL THE HOARD · {} gold", state.gold),
            rect.x,
            cy,
            rect.w,
            10.0,
            TREASURE,
        );
        draw_text_fit(
            &format!("{GOLD_CHANNEL_COST} gold -> {GOLD_CHANNEL_MANA} mana"),
            rect.x,
            cy + 16.0,
            rect.w,
            11.0,
            TEXT_MUTED,
        );
        let channel_rect = Rect::new(rect.x, cy + 26.0, rect.w, 40.0);
        if draw_command_button(
            channel_rect,
            "Channel Gold -> Mana",
            ButtonTone::Primary,
            can_channel_gold(state),
        ) {
            return BuildTabAction::ChannelGold;
        }
        if is_hovered_rect(channel_rect) {
            macroquad_toolkit::ui::draw_tooltip(
                "Convert 100 gold into 20 mana. This is an inefficient emergency reserve: adventurers and room income remain the main mana source.",
                vec2(channel_rect.x, channel_rect.y + channel_rect.h),
            );
        }
    }

    let reset_rect = Rect::new(rect.x, rect.y + rect.h - 34.0, rect.w, 30.0);
    if draw_command_button(reset_rect, "Reset dungeon", ButtonTone::Danger, true) {
        return BuildTabAction::Reset;
    }
    if is_hovered_rect(reset_rect) {
        macroquad_toolkit::ui::draw_tooltip(
            "Start a new dungeon. You will be asked to confirm before anything changes.",
            vec2(reset_rect.x, reset_rect.y),
        );
    }

    BuildTabAction::None
}

fn next_build_summary(state: &GameState) -> (String, String, i32) {
    let Some(deepest) = state.deepest_floor() else {
        return (
            "Entrance".to_string(),
            "No floor mapped yet.".to_string(),
            0,
        );
    };

    let non_core_count = deepest
        .rooms
        .iter()
        .filter(|room| room.room_type != RoomType::Core)
        .count();
    let total_rooms = state.total_room_count();

    if non_core_count > MAX_ROOMS_PER_FLOOR {
        return (
            format!("Open floor {}", state.total_floors + 1),
            "Move the core deeper.".to_string(),
            get_room_cost(total_rooms, false),
        );
    }

    let is_boss = non_core_count == MAX_ROOMS_PER_FLOOR;
    (
        if is_boss {
            "Boss chamber".to_string()
        } else {
            "Combat room".to_string()
        },
        format!("Floor {}", deepest.number),
        get_room_cost(total_rooms, is_boss),
    )
}
