use macroquad::prelude::*;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::game_state::{DungeonStatus, GameState, LogEntry, RoomUpgradeType};

use super::theme::*;
use macroquad_toolkit::colors::with_alpha;

/// Which piece of UI a tutorial step points the player toward.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TutorialAnchor {
    Drawer,
    Board,
    Hud,
}

struct StepDef {
    title: &'static str,
    instruction: &'static str,
    anchor: TutorialAnchor,
}

const STEPS: [StepDef; 7] = [
    StepDef {
        title: "Build a room",
        instruction: "Open the BUILD tab on the left (or click the glowing room on the map) to add a combat room.",
        anchor: TutorialAnchor::Drawer,
    },
    StepDef {
        title: "Place a defender",
        instruction: "Open the MONSTERS tab, pick a unit, then click your new combat room to summon it.",
        anchor: TutorialAnchor::Drawer,
    },
    StepDef {
        title: "Learn the elements",
        instruction: "Press C to open the Codex — it shows each element and what it beats.",
        anchor: TutorialAnchor::Hud,
    },
    StepDef {
        title: "Set a trap",
        instruction: "Select a room, then apply a Trap upgrade from the right panel.",
        anchor: TutorialAnchor::Board,
    },
    StepDef {
        title: "Split the route",
        instruction: "With the dungeon safe, select a room with one exit and use Branch from here in BUILD. Loot can bait a party down one route while defenders wait on the other.",
        anchor: TutorialAnchor::Drawer,
    },
    StepDef {
        title: "Open the dungeon",
        instruction: "Press 'Open Dungeon' in the top bar to invite adventurers inside.",
        anchor: TutorialAnchor::Hud,
    },
    StepDef {
        title: "Survive a raid",
        instruction: "Watch the HP bars trade blows. Hold your core and let your defenders work!",
        anchor: TutorialAnchor::Board,
    },
];

/// True while the tutorial is running and still has steps to show.
pub fn is_active(state: &GameState) -> bool {
    state.tutorial_active && (state.tutorial_step as usize) < STEPS.len()
}

/// The UI area the current step wants to highlight.
pub fn current_anchor(state: &GameState) -> Option<TutorialAnchor> {
    STEPS
        .get(state.tutorial_step.max(0) as usize)
        .map(|step| step.anchor)
}

/// End the tutorial early at the player's request.
pub fn skip(state: &mut GameState) {
    if state.tutorial_active {
        state.tutorial_active = false;
        state.add_log(LogEntry::system(
            "Tutorial dismissed. You can shape the dungeon however you like.",
        ));
    }
}

fn step_complete(state: &GameState, step_idx: usize) -> bool {
    match step_idx {
        // Build a room: any non-entrance/core room exists.
        0 => state.total_room_count() >= 1,
        // Place a defender: any room holds a monster.
        1 => state
            .floors
            .iter()
            .flat_map(|floor| &floor.rooms)
            .any(|room| !room.monsters.is_empty()),
        // Learn the elements: the player opened the Codex element wheel.
        2 => state.tutorial_codex_seen,
        // Set a trap: any room carries a Trap upgrade.
        3 => state
            .floors
            .iter()
            .flat_map(|floor| &floor.rooms)
            .any(|room| room.has_upgrade_type(RoomUpgradeType::Trap)),
        // Branch any route: a fork is a deliberate build choice, not a new
        // combat rule, so it is taught after the basic trap lever.
        4 => state
            .floors
            .iter()
            .flat_map(|floor| &floor.rooms)
            .any(|room| room.exits.len() > 1),
        // Open the dungeon: it is open, has visitors, or a raid already ran.
        5 => {
            state.status == DungeonStatus::Open
                || !state.adventurer_parties.is_empty()
                || state.raids_completed >= 1
        }
        // Survive a raid: at least one party has come and gone.
        6 => state.raids_completed >= 1,
        _ => true,
    }
}

/// Advance the tutorial if the current step's goal has been met. Call once per
/// frame; advances at most one step so each completion is announced.
pub fn advance(state: &mut GameState) {
    if !state.tutorial_active {
        return;
    }
    let idx = state.tutorial_step.max(0) as usize;
    if idx >= STEPS.len() {
        state.tutorial_active = false;
        return;
    }
    if step_complete(state, idx) {
        state.add_log(LogEntry::building(format!(
            "Tutorial: {} \u{2713}",
            STEPS[idx].title
        )));
        state.tutorial_step += 1;
        if state.tutorial_step as usize >= STEPS.len() {
            state.tutorial_active = false;
            state.add_log(LogEntry::system(
                "Tutorial complete! Grow your dungeon and keep the threat in check.",
            ));
        }
    }
}

/// Draw the tutorial callout and target highlight. Returns true if the player
/// clicked Skip this frame.
pub fn draw(state: &GameState, board_rect: Rect, anchor_rect: Rect) -> bool {
    let idx = state.tutorial_step.max(0) as usize;
    let Some(step) = STEPS.get(idx) else {
        return false;
    };

    // Pulsing highlight around the step's target.
    let pulse = (get_time() as f32 * 4.0).sin().abs();
    let glow = with_alpha(TREASURE, 0.35 + pulse * 0.45);
    draw_rectangle_lines(
        anchor_rect.x - 3.0,
        anchor_rect.y - 3.0,
        anchor_rect.w + 6.0,
        anchor_rect.h + 6.0,
        3.0,
        glow,
    );

    // Callout card pinned to the top of the board area.
    let card_w = (board_rect.w - 40.0).clamp(300.0, 560.0);
    let card = Rect::new(board_rect.x + 14.0, board_rect.y + 12.0, card_w, 88.0);
    draw_card(
        card,
        Color::new(0.05, 0.04, 0.10, 0.94),
        with_alpha(TREASURE, 0.55),
    );

    draw_text_fit(
        &format!("TUTORIAL  {}/{}", idx + 1, STEPS.len()),
        card.x + 14.0,
        card.y + 22.0,
        card.w - 120.0,
        11.0,
        TREASURE,
    );
    draw_text_fit(
        step.title,
        card.x + 14.0,
        card.y + 44.0,
        card.w - 110.0,
        16.0,
        TEXT,
    );
    draw_text_fit(
        step.instruction,
        card.x + 14.0,
        card.y + 68.0,
        card.w - 28.0,
        12.0,
        TEXT_MUTED,
    );

    // Skip button, top-right of the card.
    let skip = Rect::new(card.x + card.w - 74.0, card.y + 12.0, 62.0, 24.0);
    let hovered = is_hovered_rect(skip);
    draw_card(
        skip,
        Color::new(0.0, 0.0, 0.0, 0.22),
        with_alpha(TEXT_MUTED, if hovered { 0.6 } else { 0.3 }),
    );
    draw_centered_text("Skip", skip, 12.0, if hovered { TEXT } else { TEXT_MUTED });

    was_clicked_rect(skip)
}
