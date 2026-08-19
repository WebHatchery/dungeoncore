//! HEROES tab: the scrollable ledger of every adventurer who has delved.

use macroquad::prelude::*;

use crate::game_state::{GameState, HeroRecord, HeroStatus};
use crate::ui::theme::*;
use macroquad_toolkit::input::was_clicked_rect;

use super::draw_section_title;
use macroquad_toolkit::colors::with_alpha;

/// What the HEROES tab wants to do this frame.
pub(super) enum HeroesTabAction {
    None,
    Open(u64),
    Close,
}

fn hero_page_offset(current: usize, total: usize, visible: usize, delta: isize) -> usize {
    current
        .saturating_add_signed(delta)
        .min(total.saturating_sub(visible))
}

pub(super) fn draw_heroes_tab(state: &GameState, rect: Rect, scroll: &mut f32) -> HeroesTabAction {
    // A hero's own page takes over the tab while one is open.
    if let Some(hero) = state
        .selected_hero
        .and_then(|id| state.known_adventurers.iter().find(|h| h.id == id))
    {
        return draw_hero_journal(hero, rect);
    }

    draw_section_title(rect, "HEROES", "Everyone who has delved.");

    let quality = state.visitor_quality();
    let band = state.reputation_band();
    let next = band
        .next_threshold()
        .map(|score| format!("next at {score}"))
        .unwrap_or_else(|| "highest band".to_string());
    draw_card(
        Rect::new(rect.x, rect.y + 54.0, rect.w, 32.0),
        with_alpha(WARNING, 0.08),
        with_alpha(WARNING, 0.28),
    );
    draw_text_fit(
        &format!(
            "REPUTATION: {} ({}) · {}",
            band.name(),
            state.reputation,
            next
        ),
        rect.x + 8.0,
        rect.y + 67.0,
        rect.w - 16.0,
        10.0,
        WARNING,
    );
    draw_text_fit(
        &format!(
            "Visitors: L{:+}, {:.0}% frequency · threat still measures siege danger",
            quality.level_bonus,
            quality.spawn_chance_mult * 100.0
        ),
        rect.x + 8.0,
        rect.y + 80.0,
        rect.w - 16.0,
        9.0,
        TEXT_MUTED,
    );

    if state.known_adventurers.is_empty() {
        draw_text_fit(
            "No adventurers have entered yet. Open the dungeon to draw them in.",
            rect.x,
            rect.y + 100.0,
            rect.w,
            12.0,
            TEXT_DIM,
        );
        return HeroesTabAction::None;
    }

    // Summary line: living / inside / fallen.
    let inside = state
        .known_adventurers
        .iter()
        .filter(|h| h.status == HeroStatus::Inside)
        .count();
    let alive = state
        .known_adventurers
        .iter()
        .filter(|h| h.status == HeroStatus::Alive)
        .count();
    let dead = state
        .known_adventurers
        .iter()
        .filter(|h| h.status == HeroStatus::Dead)
        .count();
    draw_text_fit(
        &format!("Inside {}  Free {}  Fallen {}", inside, alive, dead),
        rect.x,
        rect.y + 104.0,
        rect.w,
        11.0,
        TEXT_MUTED,
    );

    // Sort: active raiders first, then veterans by delves, graves last.
    let mut order: Vec<usize> = (0..state.known_adventurers.len()).collect();
    let rank = |s: HeroStatus| match s {
        HeroStatus::Inside => 0,
        HeroStatus::Alive => 1,
        HeroStatus::Dead => 2,
    };
    order.sort_by(|&a, &b| {
        let ha = &state.known_adventurers[a];
        let hb = &state.known_adventurers[b];
        rank(ha.status)
            .cmp(&rank(hb.status))
            .then(hb.delves.cmp(&ha.delves))
    });

    let list_top = rect.y + 116.0;
    let list_h = (rect.y + rect.h - list_top).max(0.0);
    let row_h = 44.0;
    let raw_visible = (list_h / row_h) as usize;
    let needs_pager = order.len() > raw_visible;
    let pager_h = if needs_pager { 32.0 } else { 0.0 };
    let visible = ((list_h - pager_h).max(row_h) / row_h) as usize;
    let max_scroll = order.len().saturating_sub(visible) as f32;
    if rect.contains(vec2(mouse_position().0, mouse_position().1)) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.0 {
            *scroll -= wheel_y.signum();
        }
    }
    *scroll = scroll.clamp(0.0, max_scroll);
    let first = *scroll as usize;

    let mut action = HeroesTabAction::None;
    for (slot, &record_idx) in order.iter().skip(first).take(visible).enumerate() {
        let hero = &state.known_adventurers[record_idx];
        let row = Rect::new(rect.x, list_top + slot as f32 * row_h, rect.w, row_h - 6.0);
        if was_clicked_rect(row) {
            action = HeroesTabAction::Open(hero.id);
        }
        let (tag, tag_color) = match hero.status {
            HeroStatus::Inside => ("IN", DANGER),
            HeroStatus::Alive => ("FREE", EMERALD),
            HeroStatus::Dead => ("DEAD", TEXT_DIM),
        };
        draw_card(
            row,
            with_alpha(tag_color, 0.06),
            with_alpha(tag_color, 0.22),
        );
        draw_text_fit(
            &format!("{}  L{}", hero.name, hero.level),
            row.x + 10.0,
            row.y + 15.0,
            row.w - 60.0,
            12.0,
            if hero.status == HeroStatus::Dead {
                TEXT_DIM
            } else {
                TEXT
            },
        );
        draw_pill(
            Rect::new(row.x + row.w - 50.0, row.y + 7.0, 42.0, 15.0),
            tag,
            tag_color,
        );
        // Rivals — recurring survivors or prolific slayers — carry a gold badge.
        if hero.is_rival() {
            draw_pill(
                Rect::new(row.x + row.w - 98.0, row.y + 7.0, 44.0, 15.0),
                "RIVAL",
                TREASURE,
            );
        }
        let detail = match hero.status {
            HeroStatus::Dead => format!(
                "{} {} · died F{} D{}",
                hero.race, hero.class_name, hero.death_floor, hero.death_day
            ),
            _ => format!(
                "{} {} · {} · {} escapes · F{}",
                hero.race,
                hero.class_name,
                hero.drive.label(),
                hero.escapes,
                hero.deepest_floor
            ),
        };
        draw_text_fit(
            &detail,
            row.x + 10.0,
            row.y + 31.0,
            row.w - 20.0,
            9.0,
            TEXT_MUTED,
        );
    }

    if order.len() > visible {
        let y = rect.y + rect.h - 27.0;
        let button_w = 66.0;
        if draw_command_button(
            Rect::new(rect.x, y, button_w, 25.0),
            "Prev",
            ButtonTone::Ghost,
            first > 0,
        ) {
            *scroll = hero_page_offset(first, order.len(), visible, -1) as f32;
        }
        if draw_command_button(
            Rect::new(rect.x + rect.w - button_w, y, button_w, 25.0),
            "Next",
            ButtonTone::Ghost,
            first + visible < order.len(),
        ) {
            *scroll = hero_page_offset(first, order.len(), visible, 1) as f32;
        }
        draw_text_fit_right(
            &format!(
                "{}-{} of {}",
                first + 1,
                (first + visible).min(order.len()),
                order.len()
            ),
            rect.x + rect.w - button_w - 8.0,
            y + 17.0,
            rect.w - button_w * 2.0 - 16.0,
            9.0,
            TEXT_DIM,
        );
    }

    action
}

#[cfg(test)]
mod tests;

/// One hero's page: who they are, what they have taken from the dungeon, and
/// their own history in it. The ledger, not a live tracker — a hero inside the
/// dungeon is watched on the board, not here.
fn draw_hero_journal(hero: &HeroRecord, rect: Rect) -> HeroesTabAction {
    let mut action = HeroesTabAction::None;

    let (tag, tone) = match hero.status {
        HeroStatus::Inside => ("IN THE DUNGEON", DANGER),
        HeroStatus::Alive => ("AT LARGE", EMERALD),
        HeroStatus::Dead => ("FALLEN", TEXT_DIM),
    };

    if draw_command_button(
        Rect::new(rect.x, rect.y + 26.0, 74.0, 24.0),
        "< Heroes",
        ButtonTone::Ghost,
        true,
    ) {
        action = HeroesTabAction::Close;
    }

    draw_text_fit(
        &hero.name,
        rect.x,
        rect.y + 76.0,
        rect.w - 8.0,
        20.0,
        if hero.status == HeroStatus::Dead {
            TEXT_DIM
        } else {
            TEXT
        },
    );
    draw_text_fit(
        &format!(
            "Level {} {} {} · {}",
            hero.level,
            hero.race,
            hero.class_name,
            hero.drive.label()
        ),
        rect.x,
        rect.y + 96.0,
        rect.w - 8.0,
        12.0,
        TEXT_MUTED,
    );
    draw_pill(Rect::new(rect.x, rect.y + 106.0, 118.0, 16.0), tag, tone);
    draw_text_fit(
        hero.drive.description(),
        rect.x,
        rect.y + 138.0,
        rect.w,
        10.0,
        TEXT_MUTED,
    );

    // A rival is a grudge with a price on it — say both plainly.
    let mut y = rect.y + 158.0;
    if hero.is_rival() {
        let (souls, gold) = hero.bounty();
        draw_card(
            Rect::new(rect.x, y - 16.0, rect.w, 40.0),
            with_alpha(TREASURE, 0.10),
            with_alpha(TREASURE, 0.34),
        );
        draw_text_fit(
            "RIVAL — bounty when slain",
            rect.x + 10.0,
            y,
            rect.w - 20.0,
            11.0,
            TREASURE,
        );
        draw_text_fit(
            &format!("{} souls, {} gold", souls, gold),
            rect.x + 10.0,
            y + 14.0,
            rect.w - 20.0,
            11.0,
            SOUL,
        );
        y += 44.0;
    }

    let stats = [
        ("Delves", hero.delves.to_string(), TEXT),
        ("Escapes", hero.escapes.to_string(), EMERALD),
        ("Deepest floor", hero.deepest_floor.to_string(), SOUL),
        ("Resolve", format!("{} / 100", hero.resolve), WARNING),
        (
            "Prepared ward",
            hero.prepared_ward().label(),
            element_color(&hero.prepared_ward().element),
        ),
        ("Defenders slain", hero.kills.to_string(), DANGER),
        ("Gold carried off", hero.gold_stolen.to_string(), TREASURE),
    ];
    for (label, value, color) in stats {
        draw_text_fit(label, rect.x, y, rect.w * 0.62, 11.0, TEXT_MUTED);
        draw_text_fit_right(&value, rect.x + rect.w, y, rect.w * 0.34, 12.0, color);
        y += 18.0;
    }

    if !hero.insights.is_empty() {
        draw_text_fit(
            &format!("Known strata: {}", hero.insight_summary()),
            rect.x,
            y + 2.0,
            rect.w,
            10.0,
            SOUL,
        );
        draw_text_fit(
            "Ward rank grants +4% attack and -8% damage per rank vs its element.",
            rect.x,
            y + 16.0,
            rect.w,
            9.0,
            TEXT_MUTED,
        );
        y += 30.0;
    }
    if hero.status == HeroStatus::Dead {
        draw_text_fit(
            &format!("Died on floor {}, day {}", hero.death_floor, hero.death_day),
            rect.x,
            y + 4.0,
            rect.w,
            11.0,
            TEXT_DIM,
        );
        y += 22.0;
    }

    y += 14.0;
    draw_text_fit("HISTORY", rect.x, y, rect.w, 12.0, SOUL);
    y += 18.0;
    if hero.journal.is_empty() {
        draw_text_fit("Nothing recorded yet.", rect.x, y, rect.w, 11.0, TEXT_DIM);
        return action;
    }
    // Newest first — the last thing they did is the thing worth knowing.
    for event in hero.journal.iter().rev() {
        if y > rect.y + rect.h - 12.0 {
            break;
        }
        draw_text_fit(
            &format!("Day {}", event.day),
            rect.x,
            y,
            rect.w * 0.24,
            10.0,
            TEXT_DIM,
        );
        draw_text_fit(
            &event.text,
            rect.x + rect.w * 0.26,
            y,
            rect.w * 0.74,
            10.0,
            TEXT_MUTED,
        );
        y += 16.0;
    }

    action
}
