//! The defender list inside the room inspector: one card per creature standing
//! in the room, and — while a monster is armed in the drawer — the drop target
//! that either grows that creature along its own line or evicts it.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::game_state::{GameState, Monster, Room};
use crate::simulation::SwapKind;
use crate::ui::theme::*;

use super::previews::{monster_variant_status, template_trait_summary};
use super::UpgradeAction;

pub(super) const DEFENDER_ROW_H: f32 = 46.0;
pub(super) const MAX_DEFENDER_ROWS: usize = 4;

/// Vertical list of every defender in the room — one card each carrying the
/// creature's condition (health, whether it has fallen), what it hits for, its
/// element and traits, its line's variant progress, and a dismiss control.
/// Wheel-scrolls past MAX_DEFENDER_ROWS.
pub(super) fn draw_monster_progress_rows(
    state: &GameState,
    room: &Room,
    rect: Rect,
    defender_scroll: &mut f32,
) -> Option<UpgradeAction> {
    if room.monsters.is_empty() {
        draw_text_fit(
            "No defenders placed.",
            rect.x,
            rect.y + 14.0,
            rect.w,
            11.0,
            TEXT_DIM,
        );
        return None;
    }

    let total = room.monsters.len();
    let visible = total.min(MAX_DEFENDER_ROWS);
    let max_scroll = (total - visible) as f32;
    if total > visible && rect.contains(vec2(mouse_position().0, mouse_position().1)) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.0 {
            *defender_scroll -= wheel_y.signum();
        }
    }
    *defender_scroll = defender_scroll.clamp(0.0, max_scroll);
    let first = *defender_scroll as usize;

    let mut chosen = None;
    let can_dismiss = state.adventurer_parties.is_empty();
    for (slot, monster) in room.monsters.iter().skip(first).take(visible).enumerate() {
        let row = Rect::new(
            rect.x,
            rect.y + slot as f32 * DEFENDER_ROW_H,
            rect.w,
            DEFENDER_ROW_H - 4.0,
        );
        if let Some(action) = draw_defender_row(state, room, monster, row, can_dismiss) {
            chosen = Some(action);
        }
    }

    if total > visible {
        draw_text_fit_right(
            &format!("{}-{} of {} (scroll)", first + 1, first + visible, total),
            rect.x + rect.w,
            rect.y + rect.h + 10.0,
            rect.w,
            9.0,
            TEXT_DIM,
        );
    }

    chosen
}

/// One defender's card. While a monster is armed in the drawer the whole card
/// is a drop target, and states which of the two things a click would do.
fn draw_defender_row(
    state: &GameState,
    room: &Room,
    monster: &Monster,
    row: Rect,
    can_dismiss: bool,
) -> Option<UpgradeAction> {
    let element = crate::data::monsters::monster_element_id(&monster.type_name);
    let accent = match &element {
        Some(id) => element_color(id),
        None => EMERALD,
    };
    // With a monster armed in the drawer, every occupied slot is a drop target
    // and the card has to say which of the two things a click would do.
    let plan = state.selected_monster.as_ref().and_then(|armed| {
        crate::simulation::plan_swap(state, room.floor_number, room.position, monster.id, armed)
    });
    let hovered = is_hovered_rect(row);

    // A fallen defender is a state the player must be able to act on, not a
    // greyer shade of the same row — it gets the danger tone and says so.
    let tone = match &plan {
        Some(plan) if plan.kind == SwapKind::Upgrade => SOUL,
        Some(_) => WARNING,
        None if monster.alive => accent,
        None => DANGER,
    };
    let lit = plan.is_some() && hovered;
    draw_card(
        row,
        with_alpha(tone, if lit { 0.16 } else { 0.06 }),
        with_alpha(tone, if lit { 0.60 } else { 0.22 }),
    );

    let name_color = if monster.alive { TEXT } else { TEXT_DIM };
    draw_text_fit(
        &monster.type_name,
        row.x + 8.0,
        row.y + 15.0,
        row.w * 0.50,
        12.0,
        name_color,
    );

    // Offense, right-aligned and clear of the dismiss control.
    draw_text_fit_right(
        &format!(
            "ATK {}  DEF {}",
            monster.scaled_stats.attack, monster.scaled_stats.defense
        ),
        row.x + row.w - 22.0,
        row.y + 15.0,
        row.w * 0.42,
        10.0,
        if monster.alive { TEXT_MUTED } else { TEXT_DIM },
    );
    let stats_rect = Rect::new(row.x + row.w * 0.56, row.y + 3.0, row.w * 0.42, 16.0);
    if is_hovered_rect(stats_rect) {
        macroquad_toolkit::ui::draw_tooltip(
            "ATK is this defender's base hit. DEF reduces incoming damage.",
            vec2(stats_rect.x, stats_rect.y + stats_rect.h + 4.0),
        );
    }

    // Condition: a bar the width of the name column, so a defender crawling
    // back at half health after a respawn is visible at a glance.
    let bar = Rect::new(row.x + 8.0, row.y + 21.0, row.w * 0.50, 5.0);
    draw_rectangle(bar.x, bar.y, bar.w, bar.h, Color::new(0.0, 0.0, 0.0, 0.45));
    if monster.alive && monster.max_hp > 0 {
        let fraction = (monster.hp as f32 / monster.max_hp as f32).clamp(0.0, 1.0);
        let health_tone = if fraction > 0.6 {
            EMERALD
        } else if fraction > 0.3 {
            WARNING
        } else {
            DANGER
        };
        draw_rectangle(bar.x, bar.y, bar.w * fraction, bar.h, health_tone);
    }
    draw_text_fit(
        &if monster.alive {
            format!("{}/{} HP", monster.hp, monster.max_hp)
        } else {
            "Fallen".to_string()
        },
        row.x + 8.0,
        row.y + 38.0,
        row.w * 0.50,
        9.0,
        if monster.alive { TEXT_MUTED } else { DANGER },
    );

    // Element and traits on the right — but while a monster is armed the swap
    // verdict and its price take that line, because that is the live decision.
    let affordable = plan.as_ref().is_none_or(|plan| {
        state.mana >= plan.mana && state.gold >= plan.gold && state.souls >= plan.souls
    });
    match &plan {
        Some(plan) => {
            draw_text_fit_right(
                &plan.label(),
                row.x + row.w - 8.0,
                row.y + 28.0,
                row.w * 0.48,
                10.0,
                if affordable { tone } else { DANGER },
            );
        }
        None => {
            let traits = template_trait_summary(
                &monster
                    .active_traits
                    .iter()
                    .map(|t| t.id.clone())
                    .collect::<Vec<_>>(),
            );
            draw_text_fit_right(
                &format!("{} · {}", element.as_deref().unwrap_or("Neutral"), traits),
                row.x + row.w - 8.0,
                row.y + 28.0,
                row.w * 0.48,
                9.0,
                if monster.alive { accent } else { TEXT_DIM },
            );
        }
    }
    let (status, status_color) = monster_variant_status(state, room, monster);
    draw_text_fit_right(
        &status,
        row.x + row.w - 8.0,
        row.y + 40.0,
        row.w * 0.48,
        9.0,
        status_color,
    );

    // Dismiss control: refunds half the summon cost.
    let x_rect = Rect::new(row.x + row.w - 18.0, row.y + 4.0, 16.0, 16.0);
    let hovered = can_dismiss && x_rect.contains(vec2(mouse_position().0, mouse_position().1));
    draw_centered_text(
        "x",
        x_rect,
        12.0,
        if hovered {
            DANGER
        } else if can_dismiss {
            TEXT_MUTED
        } else {
            TEXT_DIM
        },
    );

    if can_dismiss && was_clicked_rect(x_rect) {
        return Some(UpgradeAction::DismissMonster(monster.id));
    }
    if plan.is_some() && affordable && was_clicked_rect(row) {
        return Some(UpgradeAction::SwapMonster(monster.id));
    }
    None
}
