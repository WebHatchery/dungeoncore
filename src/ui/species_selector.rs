use crate::data::monsters::{
    get_all_species, get_species_display_name, get_species_unlock_cost,
    get_starter_monsters_for_species,
};
use crate::game_state::GameState;
use macroquad::prelude::*;

use super::theme::*;
use macroquad_toolkit::colors::with_alpha;

pub fn draw_species_selector(
    state: &mut GameState,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    scroll: &mut f32,
) -> Option<String> {
    let panel = Rect::new(x, y, w, h);
    draw_panel(panel, Some("Choose Your Starter Race"), SOUL);
    draw_text_fit(
        "Pick the species that awakens as your first defenders.",
        x + 16.0,
        y + 48.0,
        w - 32.0,
        12.0,
        TEXT_MUTED,
    );

    let choosing_starter = state.unlocked_species.is_empty();

    // Order the list so selectable races surface first.
    let mut species_list = get_all_species();
    species_list.sort_by(|a, b| {
        let a_key = (!a.starter, a.unlock_cost);
        let b_key = (!b.starter, b.unlock_cost);
        a_key.cmp(&b_key)
    });

    let mut selected = None;
    let columns = if w >= 700.0 { 2 } else { 1 };
    let card_h = 112.0;
    let gap = 12.0;
    let side_pad = 16.0;
    let card_w = (w - side_pad * 2.0 - gap * (columns - 1) as f32) / columns as f32;
    let list_top = y + 66.0;
    let list_bottom = y + h - 16.0;

    // Mouse-wheel scrolling: the list outgrew the modal at 8 species.
    let rows = species_list.len().div_ceil(columns);
    let total_h = rows as f32 * (card_h + gap) - gap;
    let max_scroll = (total_h - (list_bottom - list_top)).max(0.0);
    let mouse = vec2(mouse_position().0, mouse_position().1);
    if panel.contains(mouse) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.0 {
            *scroll = (*scroll - wheel_y * (card_h + gap)).clamp(0.0, max_scroll);
        }
    }
    *scroll = scroll.clamp(0.0, max_scroll);

    let mut row_y = list_top - *scroll;

    for (index, species) in species_list.into_iter().enumerate() {
        let column = index % columns;
        if column == 0 && index > 0 {
            row_y += card_h + gap;
        }
        // Skip cards fully or partially outside the visible list area.
        if row_y < list_top - 1.0 || row_y + card_h > list_bottom {
            continue;
        }

        let cost = get_species_unlock_cost(&species.name).unwrap_or(0);
        let selectable = if choosing_starter {
            species.starter
        } else {
            state.gold >= cost
        };
        let display_name = get_species_display_name(&species.name);
        let roster = get_starter_monsters_for_species(&species.name)
            .into_iter()
            .map(|template| template.name)
            .collect::<Vec<_>>();
        let roster_text = if roster.is_empty() {
            "Roster unlocks later".to_string()
        } else {
            roster.join(", ")
        };

        let card = Rect::new(
            x + side_pad + column as f32 * (card_w + gap),
            row_y,
            card_w,
            card_h,
        );
        let accent = if selectable { SOUL } else { BORDER_MUTED };
        draw_card(
            card,
            with_alpha(accent, if selectable { 0.06 } else { 0.02 }),
            with_alpha(accent, if selectable { 0.44 } else { 0.16 }),
        );
        draw_rectangle(
            card.x + 1.0,
            card.y + 1.0,
            4.0,
            card.h - 2.0,
            with_alpha(accent, if selectable { 0.62 } else { 0.18 }),
        );

        draw_text_fit(
            &display_name,
            card.x + 14.0,
            card.y + 26.0,
            card.w - 150.0,
            18.0,
            if selectable { TEXT } else { TEXT_DIM },
        );

        // Status pill in the top-right corner.
        let (pill_text, pill_color) = if species.starter {
            ("STARTER", EMERALD)
        } else if selectable {
            ("READY", TREASURE)
        } else {
            ("LOCKED", TEXT_DIM)
        };
        draw_pill(
            Rect::new(card.x + card.w - 92.0, card.y + 12.0, 78.0, 18.0),
            pill_text,
            pill_color,
        );

        draw_text_fit(
            &species.description,
            card.x + 14.0,
            card.y + 53.0,
            card.w - 28.0,
            12.0,
            TEXT_MUTED,
        );
        draw_text_fit(
            &format!("Units: {}", roster_text),
            card.x + 14.0,
            card.y + 81.0,
            card.w - 130.0,
            11.0,
            if selectable { EMERALD } else { TEXT_DIM },
        );

        let label = if choosing_starter {
            if species.starter {
                "Choose".to_string()
            } else {
                "Locked".to_string()
            }
        } else if cost == 0 {
            "Choose".to_string()
        } else {
            format!("Unlock {}g", cost)
        };
        let tone = if species.starter {
            ButtonTone::Primary
        } else {
            ButtonTone::Arcane
        };
        let btn = Rect::new(card.x + card.w - 126.0, card.y + card.h - 40.0, 112.0, 30.0);
        if draw_command_button(btn, &label, tone, selectable) {
            selected = Some(species.name.clone());
        }
    }

    selected
}
