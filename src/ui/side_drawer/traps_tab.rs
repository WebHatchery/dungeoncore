//! TRAPS & LOOT tab: section chips plus the placeable upgrade list.

use macroquad::prelude::*;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::game_state::GameState;
use crate::ui::theme::*;

use super::{draw_section_title, UpgradeSection};
use macroquad_toolkit::colors::with_alpha;

pub(super) fn draw_traps_tab(
    state: &GameState,
    rect: Rect,
    section: &mut UpgradeSection,
) -> Option<String> {
    let mut selected = None;
    draw_section_title(rect, "TRAPS & LOOT", "Outfit a room.");

    // Section chips: each upgrade family gets its own list.
    let chip_h = 26.0;
    let chip_gap = 6.0;
    let chip_w = (rect.w - chip_gap) / 2.0;
    for (idx, option) in [
        UpgradeSection::Traps,
        UpgradeSection::Loot,
        UpgradeSection::Buffs,
        UpgradeSection::Shrines,
    ]
    .into_iter()
    .enumerate()
    {
        let col = (idx % 2) as f32;
        let row = (idx / 2) as f32;
        let chip = Rect::new(
            rect.x + col * (chip_w + chip_gap),
            rect.y + 56.0 + row * (chip_h + chip_gap),
            chip_w,
            chip_h,
        );
        let tone = if *section == option {
            ButtonTone::Primary
        } else {
            ButtonTone::Ghost
        };
        if draw_command_button(chip, option.label(), tone, true) {
            *section = option;
        }
    }

    let upgrades: Vec<_> = crate::data::upgrades::get_all_upgrades()
        .into_iter()
        .filter(|t| section.matches(&t.upgrade_type))
        .collect();
    let list_top = rect.y + 56.0 + (chip_h + chip_gap) * 2.0 + 8.0;
    let available_h = (rect.y + rect.h - 68.0 - list_top).max(0.0);
    let row_h = (available_h / upgrades.len().max(1) as f32 - 6.0).clamp(46.0, 64.0);
    let row_gap = 6.0;
    let mut y = list_top;
    for template in &upgrades {
        if y + row_h > rect.y + rect.h - 68.0 {
            break;
        }
        let row = Rect::new(rect.x, y, rect.w, row_h);
        if draw_upgrade_option(state, template, row) {
            selected = Some(template.name.clone());
        }
        y += row_h + row_gap;
    }

    if let Some(upgrade) = &state.selected_upgrade {
        let hint = Rect::new(rect.x, rect.y + rect.h - 62.0, rect.w, 52.0);
        draw_card(hint, with_alpha(DANGER, 0.10), with_alpha(DANGER, 0.30));
        draw_text_fit(
            upgrade,
            hint.x + 10.0,
            hint.y + 20.0,
            hint.w - 20.0,
            13.0,
            TEXT,
        );
        draw_text_fit(
            "Tap lit rooms to install. Tap again to cancel.",
            hint.x + 10.0,
            hint.y + 39.0,
            hint.w - 20.0,
            11.0,
            DANGER,
        );
    }

    selected
}

fn draw_upgrade_option(
    state: &GameState,
    template: &crate::data::upgrades::UpgradeTemplate,
    rect: Rect,
) -> bool {
    // Rooms cannot be outfitted mid-raid (`apply_upgrade` refuses), so the
    // list says so up front rather than letting the click fail.
    let peacetime = state.adventurer_parties.is_empty();
    let can_afford =
        peacetime && state.mana >= template.mana_cost && state.souls >= template.souls_cost;
    let selected = state.selected_upgrade.as_ref() == Some(&template.name);
    let hovered = can_afford && is_hovered_rect(rect);
    let fill = if selected {
        with_alpha(TREASURE, 0.13)
    } else if hovered {
        with_alpha(DANGER, 0.10)
    } else {
        CARD
    };
    let border = if selected {
        TREASURE
    } else {
        with_alpha(DANGER, 0.24)
    };
    draw_card(rect, fill, border);

    draw_text_fit(
        &template.name,
        rect.x + 12.0,
        rect.y + rect.h * 0.38,
        rect.w - 76.0,
        13.0,
        if can_afford { TEXT } else { TEXT_DIM },
    );
    // What it actually does, in words — this used to live in the inspector's
    // catalog, which the drawer has now replaced as the one build path.
    draw_text_fit(
        &crate::ui::upgrade_panel::previews::upgrade_preview(template),
        rect.x + 12.0,
        rect.y + rect.h * 0.72,
        rect.w - 76.0,
        10.0,
        TEXT_MUTED,
    );
    let cost_label = if template.souls_cost > 0 {
        format!("{}M+{}S", template.mana_cost, template.souls_cost)
    } else {
        format!("{}M", template.mana_cost)
    };
    draw_text_fit_right(
        &cost_label,
        rect.x + rect.w - 10.0,
        rect.y + rect.h * 0.58,
        60.0,
        12.0,
        if can_afford { MANA } else { DANGER },
    );

    if is_hovered_rect(rect) {
        let availability = if !peacetime {
            "Upgrades can only be installed between raids."
        } else if !can_afford {
            "The dungeon cannot afford this upgrade yet."
        } else {
            "Choose it, then tap a highlighted combat room. A room can hold one of each upgrade type."
        };
        let souls = if template.souls_cost > 0 {
            format!(" + {} souls", template.souls_cost)
        } else {
            String::new()
        };
        macroquad_toolkit::ui::draw_tooltip(
            &format!(
                "{}\nCost: {} mana{}\n{}\n{}",
                template.name,
                template.mana_cost,
                souls,
                crate::ui::upgrade_panel::previews::upgrade_preview(template),
                availability,
            ),
            vec2(rect.x, rect.y + rect.h),
        );
    }

    can_afford && was_clicked_rect(rect)
}
