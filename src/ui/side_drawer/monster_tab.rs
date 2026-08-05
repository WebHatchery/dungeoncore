//! MONSTERS tab: the summonable-defender list with portraits and a placement hint.

use macroquad::prelude::*;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::data::monsters::{get_monster_templates, get_species_display_name, MonsterTemplate};
use crate::data::traits::get_trait;
use crate::game_state::GameState;
use crate::ui::theme::*;

use super::draw_section_title;
use macroquad_toolkit::colors::with_alpha;

pub(super) fn draw_monster_tab(state: &GameState, rect: Rect) -> Option<String> {
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
    let available_h = (rect.h - 138.0).max(0.0);
    let row_h = (available_h / templates.len().max(1) as f32).clamp(46.0, 72.0);
    let row_gap = 6.0;
    let mut y = rect.y + 72.0;
    for template in &templates {
        if y + row_h > rect.y + rect.h - 68.0 {
            break;
        }
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
            "Click rooms to place; reclick entry to stop.",
            hint.x + 10.0,
            hint.y + 39.0,
            hint.w - 20.0,
            11.0,
            SOUL,
        );
    }

    selected
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
    let portrait = Rect::new(rect.x + 9.0, rect.y + 9.0, 54.0, rect.h - 18.0);
    draw_monster_portrait(portrait, unlocked, selected);
    let title = if unlocked {
        template.name.as_str()
    } else {
        template.species.as_str()
    };
    draw_text_fit(
        title,
        rect.x + 74.0,
        rect.y + rect.h * 0.38,
        rect.w - 126.0,
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
        rect.x + 74.0,
        rect.y + rect.h * 0.70,
        rect.w - 126.0,
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
        rect.y + rect.h * 0.58,
        54.0,
        13.0,
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
            "Choose it, then click a combat room with a free defender slot."
        };
        let souls = if template.souls_cost > 0 {
            format!(" + {} souls", template.souls_cost)
        } else {
            String::new()
        };
        macroquad_toolkit::ui::draw_tooltip(
            &format!(
                "{}\nCost: {} mana{} · Tier {}\n{}\n{}",
                template.name, template.base_cost, souls, template.tier, traits, availability,
            ),
            vec2(rect.x, rect.y + rect.h),
        );
    }

    enabled && was_clicked_rect(rect)
}

fn draw_monster_portrait(rect: Rect, unlocked: bool, selected: bool) {
    let color = if unlocked { EMERALD } else { TEXT_DIM };
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.30),
        with_alpha(color, if selected { 0.55 } else { 0.20 }),
    );
    if unlocked {
        draw_circle(
            rect.x + rect.w * 0.50,
            rect.y + rect.h * 0.42,
            rect.w * 0.22,
            with_alpha(color, 0.42),
        );
        draw_triangle(
            vec2(rect.x + rect.w * 0.21, rect.y + rect.h * 0.30),
            vec2(rect.x + rect.w * 0.38, rect.y + rect.h * 0.38),
            vec2(rect.x + rect.w * 0.30, rect.y + rect.h * 0.55),
            color,
        );
        draw_triangle(
            vec2(rect.x + rect.w * 0.79, rect.y + rect.h * 0.30),
            vec2(rect.x + rect.w * 0.62, rect.y + rect.h * 0.38),
            vec2(rect.x + rect.w * 0.70, rect.y + rect.h * 0.55),
            color,
        );
        draw_circle(rect.x + rect.w * 0.42, rect.y + rect.h * 0.40, 2.0, BG_DEEP);
        draw_circle(rect.x + rect.w * 0.58, rect.y + rect.h * 0.40, 2.0, BG_DEEP);
    } else {
        draw_rectangle(
            rect.x + rect.w * 0.30,
            rect.y + rect.h * 0.38,
            rect.w * 0.40,
            rect.h * 0.34,
            Color::new(0.0, 0.0, 0.0, 0.34),
        );
        draw_rectangle_lines(
            rect.x + rect.w * 0.34,
            rect.y + rect.h * 0.45,
            rect.w * 0.32,
            rect.h * 0.22,
            2.0,
            TEXT_DIM,
        );
        draw_circle_lines(
            rect.x + rect.w * 0.50,
            rect.y + rect.h * 0.43,
            rect.w * 0.16,
            2.0,
            TEXT_DIM,
        );
    }
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
