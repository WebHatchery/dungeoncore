//! VARIANTS tab: what each monster line has pooled, the variant it is
//! about to unlock, and the next species available to buy.

use macroquad::prelude::*;

use crate::data::evolutions::get_evolution_for_monster;
use crate::data::monsters::{get_all_species, get_species_display_name};
use crate::game_state::GameState;
use crate::ui::theme::*;
use std::collections::BTreeMap;

use super::{draw_section_title, DrawerAction};
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::input::is_hovered_rect;

pub(super) fn draw_evolution_tab(state: &GameState, rect: Rect) -> DrawerAction {
    draw_section_title(rect, "VARIANTS", "What your lines are learning.");
    let mut action = DrawerAction::None;

    let rows = collect_variant_rows(state);
    let unlocked_count = rows.iter().filter(|row| row.unlocked).count();
    let waiting_count = rows
        .iter()
        .filter(|row| !row.unlocked && row.has_path)
        .count();
    let final_count = rows.iter().filter(|row| !row.has_path).count();

    let card = Rect::new(rect.x, rect.y + 70.0, rect.w, 126.0);
    draw_card(card, CARD, BORDER_MUTED);
    draw_text_fit(
        &format!("Unlocked: {}  Learning: {}", unlocked_count, waiting_count),
        card.x + 12.0,
        card.y + 28.0,
        card.w - 24.0,
        15.0,
        TEXT,
    );
    draw_text_fit(
        &format!("Final forms: {}", final_count),
        card.x + 12.0,
        card.y + 55.0,
        card.w - 24.0,
        13.0,
        TEXT_MUTED,
    );
    let xp_rect = Rect::new(rect.x + 6.0, rect.y + 19.0, rect.w - 92.0, 18.0);
    if is_hovered_rect(xp_rect) {
        crate::ui::draw_tooltip(
            "XP is pooled across every defender of this line. Field the listed floor to unlock its next variant.",
            vec2(xp_rect.x, xp_rect.y + xp_rect.h + 4.0),
        );
    }
    draw_text_fit(
        &format!(
            "Species: {}  Souls: {}",
            state.unlocked_species.len(),
            state.souls
        ),
        card.x + 12.0,
        card.y + 82.0,
        card.w - 24.0,
        13.0,
        SOUL,
    );
    draw_text_fit(
        "Experience pools per line, not per creature.",
        card.x + 12.0,
        card.y + 108.0,
        card.w - 24.0,
        11.0,
        TEXT_DIM,
    );

    let mut row_y = card.y + card.h + 12.0;
    let row_h = 46.0;
    for row in rows
        .iter()
        .take(((rect.y + rect.h - row_y - 106.0) / row_h).max(0.0) as usize)
    {
        draw_variant_row(row, Rect::new(rect.x, row_y, rect.w, row_h - 6.0));
        row_y += row_h;
    }

    if let Some(species) = next_locked_species(state) {
        let unlock_cost = species.unlock_cost;
        let can_afford = state.gold >= unlock_cost;
        let species_name = species.name.clone();
        let unlock_rect = Rect::new(rect.x, rect.y + rect.h - 94.0, rect.w, 40.0);
        draw_card(
            unlock_rect,
            with_alpha(TREASURE, 0.075),
            with_alpha(TREASURE, 0.24),
        );
        draw_text_fit(
            &format!("Next race: {}", get_species_display_name(&species_name)),
            unlock_rect.x + 10.0,
            unlock_rect.y + 16.0,
            unlock_rect.w - 96.0,
            11.0,
            TEXT,
        );
        draw_text_fit(
            &format!("{} gold", unlock_cost),
            unlock_rect.x + 10.0,
            unlock_rect.y + 32.0,
            unlock_rect.w - 96.0,
            10.0,
            if can_afford { TREASURE } else { TEXT_DIM },
        );
        if draw_command_button(
            Rect::new(
                unlock_rect.x + unlock_rect.w - 78.0,
                unlock_rect.y + 7.0,
                68.0,
                26.0,
            ),
            "Unlock",
            ButtonTone::Ghost,
            can_afford,
        ) {
            action = DrawerAction::UnlockSpecies(species_name);
        }
    }

    action
}

#[derive(Debug)]
struct VariantUiRow {
    /// The line that is learning — a monster type, not one creature.
    line: String,
    /// Where that line is fielded, and how deep.
    fielded: String,
    xp_label: String,
    status: String,
    color: Color,
    unlocked: bool,
    has_path: bool,
}

/// One row per monster *type* the dungeon fields, showing the pooled experience
/// of that whole line and what it is about to unlock. Several goblins in
/// several rooms are one row, because they share one pool.
fn collect_variant_rows(state: &GameState) -> Vec<VariantUiRow> {
    // Where each line stands: how many creatures, and the deepest floor —
    // the depth gate on a variant is checked against where the line is fielded.
    let mut fielded: BTreeMap<&str, (usize, i32)> = BTreeMap::new();
    for floor in &state.floors {
        for room in &floor.rooms {
            for monster in &room.monsters {
                let entry = fielded.entry(&monster.type_name).or_insert((0, 0));
                entry.0 += 1;
                entry.1 = entry.1.max(room.floor_number);
            }
        }
    }

    let mut rows = Vec::new();
    for (line, (count, deepest)) in fielded {
        let pooled = state.type_experience(line);
        let fielded_label = format!("{} placed · F{}", count, deepest);

        let Some(path) = get_evolution_for_monster(line) else {
            rows.push(VariantUiRow {
                line: line.to_string(),
                fielded: fielded_label,
                xp_label: format!("{} XP", pooled),
                status: "Final form".to_string(),
                color: TEXT_DIM,
                unlocked: false,
                has_path: false,
            });
            continue;
        };

        let unlocked = state.unlocked_monsters.contains(&path.to_monster);
        let enough_xp = pooled >= path.experience_required;
        let deep_enough = deepest >= path.conditions.min_floor;
        let (status, color) = if unlocked {
            (format!("{} unlocked", path.to_monster), EMERALD)
        } else if !enough_xp {
            (
                format!(
                    "{} XP to {}",
                    path.experience_required - pooled,
                    path.to_monster
                ),
                MANA,
            )
        } else if !deep_enough {
            (
                format!("Field it on floor {}", path.conditions.min_floor),
                WARNING,
            )
        } else {
            ("Unlocking...".to_string(), EMERALD)
        };

        rows.push(VariantUiRow {
            line: line.to_string(),
            fielded: fielded_label,
            xp_label: format!("{}/{} XP", pooled, path.experience_required),
            status,
            color,
            unlocked,
            has_path: true,
        });
    }

    // Lines still learning first — those are the ones the player can act on.
    rows.sort_by(|a, b| {
        a.unlocked
            .cmp(&b.unlocked)
            .then_with(|| b.has_path.cmp(&a.has_path))
            .then_with(|| a.line.cmp(&b.line))
    });
    rows
}

fn draw_variant_row(row: &VariantUiRow, rect: Rect) {
    draw_card(
        rect,
        with_alpha(row.color, 0.075),
        with_alpha(row.color, 0.26),
    );
    draw_text_fit(
        &row.line,
        rect.x + 9.0,
        rect.y + 16.0,
        rect.w - 82.0,
        12.0,
        TEXT,
    );
    draw_text_fit(
        &format!("{} · {}", row.fielded, row.xp_label),
        rect.x + 9.0,
        rect.y + 32.0,
        rect.w - 82.0,
        10.0,
        TEXT_MUTED,
    );
    draw_text_fit_right(
        &row.status,
        rect.x + rect.w - 8.0,
        rect.y + 25.0,
        92.0,
        10.0,
        row.color,
    );
}

fn next_locked_species(state: &GameState) -> Option<crate::data::monsters::SpeciesData> {
    let mut locked = get_all_species()
        .into_iter()
        .filter(|species| !state.unlocked_species.contains(&species.name))
        .collect::<Vec<_>>();
    locked.sort_by_key(|species| species.unlock_cost);
    locked.into_iter().next()
}
