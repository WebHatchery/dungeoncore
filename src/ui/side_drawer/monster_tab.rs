//! MONSTERS tab: the summonable-defender list with portraits and a placement hint.

use macroquad::prelude::*;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::data::monsters::{get_monster_templates, get_species_display_name, MonsterTemplate};
use crate::data::traits::get_trait;
use crate::game_state::GameState;
use crate::ui::theme::*;

use super::draw_section_title;
use macroquad_toolkit::colors::with_alpha;

pub(super) fn draw_monster_tab(state: &GameState, rect: Rect, scroll: &mut f32) -> Option<String> {
    let mut selected = None;
    draw_section_title(rect, "MONSTERS", "Choose a defender.");

    // Only summonable (unlocked) monsters appear; new forms join the list
    // as species are bought and evolutions are earned.
    let templates: Vec<MonsterTemplate> = get_monster_templates()
        .into_iter()
        .filter(|t| {
            state.unlocked_species.contains(&t.species) && state.unlocked_monsters.contains(&t.name)
        })
        .collect();
    let has_scroll_controls = templates.len() > 5;
    let list_top = rect.y + if has_scroll_controls { 106.0 } else { 72.0 };
    let list_bottom = rect.y + rect.h - 68.0;
    let row_gap = 6.0;
    let row_h = 68.0;
    let visible_rows =
        (((list_bottom - list_top + row_gap) / (row_h + row_gap)).floor() as usize).max(1);
    let max_first = templates.len().saturating_sub(visible_rows);
    *scroll = scroll.clamp(0.0, max_first as f32);

    if has_scroll_controls {
        draw_scroll_controls(rect, scroll, max_first, visible_rows);
    }

    let list_rect = Rect::new(rect.x, list_top, rect.w, (list_bottom - list_top).max(0.0));
    if is_hovered_rect(list_rect) {
        let wheel = mouse_wheel().1;
        if wheel.abs() > f32::EPSILON {
            *scroll = (*scroll - wheel.signum()).clamp(0.0, max_first as f32);
        }
    }

    let mut y = list_top;
    for template in templates.iter().skip(*scroll as usize).take(visible_rows) {
        let row = Rect::new(rect.x, y, rect.w, row_h);
        if draw_monster_option(state, template, row) {
            selected = Some(template.name.clone());
        }
        y += row_h + row_gap;
    }

    if let Some(monster) = &state.selected_monster {
        let hint = Rect::new(rect.x, rect.y + rect.h - 62.0, rect.w, 52.0);
        draw_card(hint, with_alpha(SOUL, 0.10), with_alpha(SOUL, 0.30));
        draw_text_fit(
            monster,
            hint.x + 10.0,
            hint.y + 20.0,
            hint.w - 20.0,
            13.0,
            TEXT,
        );
        draw_text_fit(
            "Tap lit rooms to place. Tap again to cancel.",
            hint.x + 10.0,
            hint.y + 39.0,
            hint.w - 20.0,
            11.0,
            SOUL,
        );
    }

    selected
}

fn draw_scroll_controls(rect: Rect, scroll: &mut f32, max_first: usize, page_size: usize) {
    let controls = Rect::new(rect.x, rect.y + 68.0, rect.w, 30.0);
    let button_w = 50.0;
    let up = Rect::new(controls.x, controls.y, button_w, controls.h);
    let down = Rect::new(
        controls.x + controls.w - button_w,
        controls.y,
        button_w,
        controls.h,
    );
    if draw_command_button(up, "UP", ButtonTone::Ghost, *scroll > 0.0) {
        *scroll = (*scroll - page_size as f32).max(0.0);
    }
    if draw_command_button(down, "DOWN", ButtonTone::Ghost, *scroll < max_first as f32) {
        *scroll = (*scroll + page_size as f32).min(max_first as f32);
    }

    let first = *scroll as usize + 1;
    let last = (*scroll as usize + page_size).min(max_first + page_size);
    draw_centered_text(
        &format!("{}–{}", first, last),
        Rect::new(
            up.x + up.w + 4.0,
            controls.y,
            controls.w - button_w * 2.0 - 8.0,
            controls.h,
        ),
        12.0,
        TEXT_MUTED,
    );
}

fn draw_monster_option(state: &GameState, template: &MonsterTemplate, rect: Rect) -> bool {
    let unlocked = state.unlocked_species.contains(&template.species)
        && state.unlocked_monsters.contains(&template.name);
    let can_afford = state.mana >= template.base_cost && state.souls >= template.souls_cost;
    let enabled = unlocked && can_afford;
    let selected = state.selected_monster.as_ref() == Some(&template.name);
    let hovered = enabled && is_hovered_rect(rect);
    let fill = if selected {
        with_alpha(TREASURE, 0.13)
    } else if hovered {
        with_alpha(SOUL, 0.10)
    } else {
        CARD
    };
    let border = if selected {
        TREASURE
    } else if unlocked {
        with_alpha(SOUL, 0.24)
    } else {
        BORDER_MUTED
    };

    draw_card(rect, fill, border);
    let title = if unlocked {
        template.name.as_str()
    } else {
        template.species.as_str()
    };
    draw_text_fit(
        title,
        rect.x + 10.0,
        rect.y + 22.0,
        rect.w - 20.0,
        14.0,
        if unlocked { TEXT } else { TEXT_DIM },
    );
    let traits = trait_summary(&template.traits);
    let detail = if unlocked {
        format!(
            "T{} {} {}{}  {}",
            template.tier,
            get_species_display_name(&template.species),
            template.element.as_deref().unwrap_or("Neutral"),
            if template.boss_only {
                " • Boss room"
            } else {
                ""
            },
            traits
        )
    } else {
        "Locked".to_string()
    };
    draw_text_fit(
        &detail,
        rect.x + 10.0,
        rect.y + rect.h - 10.0,
        rect.w - 20.0,
        11.0,
        TEXT_MUTED,
    );
    let cost_label = if template.souls_cost > 0 {
        format!("{}M+{}S", template.base_cost, template.souls_cost)
    } else {
        format!("{}M", template.base_cost)
    };
    draw_text_fit_right(
        &cost_label,
        rect.x + rect.w - 10.0,
        rect.y + 41.0,
        54.0,
        12.0,
        if can_afford { MANA } else { DANGER },
    );

    if is_hovered_rect(rect) {
        let availability = if !unlocked {
            "Unlock this species and variant first."
        } else if !can_afford {
            "The dungeon cannot afford this summon yet."
        } else if template.boss_only {
            "Place this defender only in a boss room."
        } else {
            "Choose it, then tap a combat room with a free defender slot."
        };
        let souls = if template.souls_cost > 0 {
            format!(" + {} souls", template.souls_cost)
        } else {
            String::new()
        };
        crate::ui::draw_tooltip(
            &format!(
                "{}\nCost: {} mana{} · Tier {}\n{}\n{}",
                template.name, template.base_cost, souls, template.tier, traits, availability,
            ),
            vec2(rect.x, rect.y + rect.h),
        );
    }

    enabled && was_clicked_rect(rect)
}

fn trait_summary(trait_ids: &[String]) -> String {
    if trait_ids.is_empty() {
        return "No traits".to_string();
    }

    trait_ids
        .iter()
        .take(2)
        .map(|trait_id| {
            get_trait(trait_id)
                .map(|trait_def| trait_def.name)
                .unwrap_or_else(|| trait_id.clone())
        })
        .collect::<Vec<_>>()
        .join(", ")
}
