//! DEPTH tab: the authored chapter map, permanent relics, and the live party
//! doctrine that tells the keeper what the current visitors are actually after.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use crate::game_state::{DepthLayer, GameState};
use crate::ui::theme::*;

use super::draw_section_title;

pub(super) fn draw_depth_tab(state: &GameState, rect: Rect) {
    draw_section_title(rect, "DEPTH", "The dungeon remembers what it survives.");

    let floor = state.total_floors.max(1);
    let layer = DepthLayer::for_floor(floor);
    let stratum = crate::data::strata::stratum_for_floor(floor);
    let accent = element_color(&stratum.element);
    let card = Rect::new(rect.x, rect.y + 70.0, rect.w, 142.0);
    draw_card(card, with_alpha(accent, 0.08), with_alpha(accent, 0.34));
    draw_text_fit(
        &format!("FLOOR {} · {} {}", floor, stratum.name, layer.label()),
        card.x + 12.0,
        card.y + 28.0,
        card.w - 24.0,
        15.0,
        accent,
    );
    draw_text_fit(
        &format!(
            "Depth mark {} · pressure x{:.2}",
            layer.art_mark(),
            state.depth_pressure(floor)
        ),
        card.x + 12.0,
        card.y + 52.0,
        card.w - 24.0,
        11.0,
        TEXT,
    );
    draw_text_fit(
        layer.description(),
        card.x + 12.0,
        card.y + 78.0,
        card.w - 24.0,
        11.0,
        TEXT_MUTED,
    );
    draw_text_fit(
        &format!(
            "Apex loot x{:.2} · {}",
            layer.loot_multiplier(),
            stratum.description
        ),
        card.x + 12.0,
        card.y + 106.0,
        card.w - 24.0,
        10.0,
        TEXT_DIM,
    );

    let relic_y = card.y + card.h + 16.0;
    draw_text_fit(
        &format!("APEX RELICS · {}/5 recovered", state.depth_relics.len()),
        rect.x,
        relic_y + 14.0,
        rect.w,
        11.0,
        SOUL,
    );
    let relics = [
        crate::game_state::relic_for_floor(1),
        crate::game_state::relic_for_floor(5),
        crate::game_state::relic_for_floor(9),
        crate::game_state::relic_for_floor(13),
        crate::game_state::relic_for_floor(17),
    ];
    for (index, relic) in relics.iter().enumerate() {
        let row = Rect::new(rect.x, relic_y + 24.0 + index as f32 * 37.0, rect.w, 31.0);
        let claimed = state.has_depth_relic(relic.id);
        let tone = if claimed { SOUL } else { TEXT_DIM };
        draw_card(row, with_alpha(tone, 0.06), with_alpha(tone, 0.20));
        draw_icon_disc(
            vec2(row.x + 18.0, row.y + 15.0),
            9.0,
            tone,
            if claimed { "◆" } else { "?" },
        );
        draw_text_fit(
            relic.name,
            row.x + 34.0,
            row.y + 14.0,
            row.w - 44.0,
            10.0,
            if claimed { TEXT } else { TEXT_DIM },
        );
        draw_text_fit(
            if claimed {
                relic.boon
            } else {
                "Defeat this stratum's apex boss"
            },
            row.x + 34.0,
            row.y + 26.0,
            row.w - 44.0,
            8.0,
            if claimed { SOUL } else { TEXT_DIM },
        );
    }

    if let Some(party) = state.adventurer_parties.first() {
        let doctrine = crate::game_state::doctrine_for_party(state, party);
        let y = (relic_y + 24.0 + relics.len() as f32 * 37.0).min(rect.y + rect.h - 48.0);
        draw_card(
            Rect::new(rect.x, y, rect.w, 42.0),
            with_alpha(WARNING, 0.08),
            with_alpha(WARNING, 0.26),
        );
        draw_text_fit(
            &format!("LIVE DOCTRINE · {}", doctrine.label()),
            rect.x + 10.0,
            y + 17.0,
            rect.w - 20.0,
            11.0,
            WARNING,
        );
        draw_text_fit(
            doctrine.description(),
            rect.x + 10.0,
            y + 32.0,
            rect.w - 20.0,
            9.0,
            TEXT_MUTED,
        );
    }
}
