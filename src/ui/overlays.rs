use macroquad::prelude::*;

use crate::data::elements::get_all_elements;
use crate::data::monsters::{get_monster_templates, get_species_display_name};
use crate::game_state::{GameState, RaidOutcome, RaidSummary};

use super::theme::*;
use super::upgrade_panel::draw_close_button;
use macroquad_toolkit::colors::with_alpha;

/// A compact post-raid summary card floating over the dungeon board: the
/// outcome, the casualties the dungeon inflicted, and the income it banked —
/// so the player can *see* whether the build worked. Returns true when
/// dismissed.
pub fn draw_raid_summary(summary: &RaidSummary, area: Rect) -> bool {
    let w = 300.0_f32.min(area.w - 24.0);
    let h = 218.0;
    let x = area.x + (area.w - w) * 0.5;
    let y = area.y + 16.0;
    let card = Rect::new(x, y, w, h);

    let (title, accent) = match summary.outcome {
        RaidOutcome::Wiped => ("Party Wiped Out", EMERALD),
        RaidOutcome::Repelled if summary.gold_gained > 0 => ("Adventurers Escaped", TREASURE),
        RaidOutcome::Repelled => ("Raid Repelled", WARNING),
    };

    draw_panel(card, Some("Raid Report"), accent);
    draw_text_fit(
        title,
        card.x + 14.0,
        card.y + 52.0,
        card.w - 28.0,
        18.0,
        accent,
    );
    draw_text_fit(
        &format!(
            "{} of {} adventurers slain · {} escaped",
            summary.slain, summary.party_size, summary.survivors
        ),
        card.x + 14.0,
        card.y + 71.0,
        card.w - 28.0,
        11.0,
        TEXT_MUTED,
    );
    draw_line(
        card.x + 14.0,
        card.y + 82.0,
        card.x + card.w - 14.0,
        card.y + 82.0,
        1.0,
        BORDER_MUTED,
    );

    // Income the dungeon banked, plus what it cost in defenders.
    let rows = [
        ("Mana earned", format!("+{}", summary.mana_gained), MANA),
        (
            "Recovery cost",
            format!("-{}", summary.mana_recovery_cost),
            if summary.mana_recovery_cost > 0 {
                DANGER
            } else {
                TEXT_MUTED
            },
        ),
        (
            "Mana net",
            format!("{:+}", summary.mana_gained - summary.mana_recovery_cost),
            if summary.mana_gained >= summary.mana_recovery_cost {
                EMERALD
            } else {
                DANGER
            },
        ),
        ("Gold banked", format!("+{}", summary.gold_gained), TREASURE),
        (
            "Souls harvested",
            format!("+{}", summary.souls_gained),
            SOUL,
        ),
        (
            "Defenders lost",
            summary.defenders_lost.to_string(),
            if summary.defenders_lost > 0 {
                DANGER
            } else {
                TEXT_MUTED
            },
        ),
        (
            "Reputation",
            format!(
                "{:+} · {}",
                summary.reputation_change, summary.reputation_after
            ),
            if summary.reputation_change >= 0 {
                EMERALD
            } else {
                DANGER
            },
        ),
    ];
    let mut ry = card.y + 96.0;
    for (label, value, color) in &rows {
        draw_text_fit(label, card.x + 16.0, ry, card.w * 0.6, 11.0, TEXT_MUTED);
        draw_text_fit_right(
            value,
            card.x + card.w - 16.0,
            ry,
            card.w * 0.36,
            12.0,
            *color,
        );
        ry += 15.0;
    }

    draw_command_button(
        Rect::new(card.x + 14.0, card.y + card.h - 30.0, card.w - 28.0, 22.0),
        "Dismiss",
        ButtonTone::Ghost,
        true,
    )
}

/// Screen-state dressing for an active siege: a pulsing red frame around the
/// whole screen plus a bold banner, so the siege reads as a distinct, alarming
/// moment rather than a single line scrolling past in the log.
pub fn draw_siege_overlay(sw: f32, sh: f32) {
    let pulse = (get_time() as f32 * 3.0).sin().abs();

    // Pulsing danger frame — an "alert" border that doesn't obscure content.
    let thickness = 7.0 + pulse * 5.0;
    draw_rectangle_lines(
        2.0,
        2.0,
        sw - 4.0,
        sh - 4.0,
        thickness,
        Color::new(0.86, 0.10, 0.13, 0.42 + pulse * 0.34),
    );

    // Centered banner just below the HUD.
    let bw = 460.0_f32.min(sw - 40.0);
    let bh = 40.0;
    let bx = (sw - bw) * 0.5;
    let by = 98.0;
    let banner = Rect::new(bx, by, bw, bh);
    draw_card(
        banner,
        Color::new(0.34, 0.02, 0.05, 0.94),
        with_alpha(DANGER, 0.7 + pulse * 0.3),
    );
    draw_centered_text(
        "THE REALM'S SIEGE — DEFEND THE CORE",
        banner,
        17.0,
        Color::new(1.0, 0.86, 0.86, 0.85 + pulse * 0.15),
    );
}

/// Full-screen bestiary/codex: the element wheel and every monster the
/// dungeon has discovered. Returns true when the player closes it.
pub fn draw_codex(state: &GameState, sw: f32, sh: f32, scroll: &mut f32) -> bool {
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.8));
    let w = (sw - 120.0).min(940.0);
    let h = (sh - 80.0).min(620.0);
    let x = (sw - w) / 2.0;
    let y = (sh - h) / 2.0;
    let panel = Rect::new(x, y, w, h);
    draw_panel(panel, Some("Codex"), ARCANE);

    let mut close = false;
    if draw_close_button(Rect::new(x + w - 40.0, y + 14.0, 30.0, 26.0)) {
        close = true;
    }
    if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::Escape) {
        close = true;
    }

    // Left column: element effectiveness wheel.
    let col_gap = 24.0;
    let left_w = (w * 0.36).clamp(240.0, 340.0);
    let left = Rect::new(x + 20.0, y + 52.0, left_w, h - 72.0);
    draw_text_fit("ELEMENTS", left.x, left.y + 14.0, left.w, 13.0, SOUL);
    draw_text_fit(
        "Attacker is strong vs:",
        left.x,
        left.y + 34.0,
        left.w,
        10.0,
        TEXT_DIM,
    );
    let mut ey = left.y + 52.0;
    for element in get_all_elements() {
        if ey + 24.0 > left.y + left.h {
            break;
        }
        let targets = if element.strong_against.is_empty() {
            "— (neutral)".to_string()
        } else {
            element.strong_against.join(", ")
        };
        draw_text_fit(
            &format!("{}  {}", element.emoji, element.id),
            left.x,
            ey + 12.0,
            left.w * 0.42,
            11.0,
            TEXT,
        );
        draw_text_fit(
            &format!("› {}", targets),
            left.x + left.w * 0.42,
            ey + 12.0,
            left.w * 0.58,
            10.0,
            EMERALD,
        );
        ey += 22.0;
    }

    // Right column: discovered monsters (unlocked only), scrollable.
    let right = Rect::new(
        left.x + left_w + col_gap,
        y + 52.0,
        w - left_w - col_gap - 40.0,
        h - 72.0,
    );
    let discovered: Vec<_> = get_monster_templates()
        .into_iter()
        .filter(|m| state.unlocked_monsters.contains(&m.name))
        .collect();
    draw_text_fit(
        &format!("BESTIARY  ({} discovered)", discovered.len()),
        right.x,
        right.y + 14.0,
        right.w,
        13.0,
        SOUL,
    );

    let list = Rect::new(right.x, right.y + 30.0, right.w, right.h - 30.0);
    let row_h = 46.0;
    let visible = (list.h / row_h) as usize;
    let max_scroll = discovered.len().saturating_sub(visible) as f32;
    if list.contains(vec2(mouse_position().0, mouse_position().1)) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.0 {
            *scroll -= wheel_y.signum();
        }
    }
    *scroll = scroll.clamp(0.0, max_scroll);
    let first = *scroll as usize;

    if discovered.is_empty() {
        draw_text_fit(
            "No monsters discovered yet. Unlock a species to begin.",
            list.x,
            list.y + 30.0,
            list.w,
            12.0,
            TEXT_DIM,
        );
    }
    for (slot, m) in discovered.iter().skip(first).take(visible).enumerate() {
        let row = Rect::new(list.x, list.y + slot as f32 * row_h, list.w, row_h - 6.0);
        draw_card(row, CARD, with_alpha(BORDER, 0.2));
        draw_text_fit(
            &format!("{}  {}", m.emoji, m.name),
            row.x + 10.0,
            row.y + 16.0,
            row.w * 0.5,
            12.0,
            TEXT,
        );
        draw_text_fit(
            &format!(
                "T{} {} · {}",
                m.tier,
                get_species_display_name(&m.species),
                m.element.as_deref().unwrap_or("Neutral")
            ),
            row.x + 10.0,
            row.y + 33.0,
            row.w * 0.5,
            9.0,
            TEXT_MUTED,
        );
        draw_text_fit_right(
            &format!("HP {}  ATK {}  DEF {}", m.hp, m.attack, m.defense),
            row.x + row.w - 10.0,
            row.y + 18.0,
            row.w * 0.46,
            10.0,
            TEXT_MUTED,
        );
        if !m.traits.is_empty() {
            let names: Vec<String> = m
                .traits
                .iter()
                .filter_map(|t| crate::data::traits::get_trait(t).map(|d| d.name))
                .collect();
            draw_text_fit_right(
                &names.join(", "),
                row.x + row.w - 10.0,
                row.y + 34.0,
                row.w * 0.46,
                9.0,
                ARCANE,
            );
        }
    }

    draw_text_fit(
        "Press C or Esc to close.",
        x + 20.0,
        y + h - 14.0,
        w - 40.0,
        10.0,
        TEXT_DIM,
    );

    close
}

/// Keyboard and pointer reference, kept in the game so a keeper never has to
/// leave a running dungeon to rediscover its controls.
pub fn draw_controls_reference(sw: f32, sh: f32) -> bool {
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.72));
    let card = Rect::new((sw - 480.0) * 0.5, (sh - 380.0) * 0.5, 480.0, 380.0);
    draw_panel(card, Some("Controls"), ARCANE);
    draw_text_fit(
        "Keyboard shortcuts",
        card.x + 22.0,
        card.y + 54.0,
        card.w - 44.0,
        14.0,
        TEXT,
    );
    let shortcuts = [
        ("Space", "Pause / resume the dungeon"),
        ("Arrow keys", "Inspect connected rooms and nearby floors"),
        ("Q", "Cast Core Smite"),
        ("C", "Open the element Codex"),
        ("K", "Open the goals track"),
        ("P", "Open Core Powers"),
        ("H / Esc", "Close this reference"),
    ];
    let mut y = card.y + 82.0;
    for (key, action) in shortcuts {
        draw_pill(Rect::new(card.x + 22.0, y - 14.0, 72.0, 22.0), key, SOUL);
        draw_text_fit(
            action,
            card.x + 108.0,
            y + 1.0,
            card.w - 130.0,
            12.0,
            TEXT_MUTED,
        );
        y += 32.0;
    }
    draw_text_fit(
        "Pointer: click rooms to inspect or place an armed defender/upgrade; click drawer tabs and cards to choose actions; scroll long lists.",
        card.x + 22.0,
        card.y + 290.0,
        card.w - 44.0,
        11.0,
        TEXT_DIM,
    );
    draw_centered_text(
        "Press H or Esc to return",
        Rect::new(card.x + 22.0, card.y + 326.0, card.w - 44.0, 26.0),
        11.0,
        TEXT_MUTED,
    );
    is_key_pressed(KeyCode::H) || is_key_pressed(KeyCode::Escape)
}

/// Full-screen "the core has fallen" overlay. Returns true when the player
/// clicks to begin a new dungeon.
pub fn draw_game_over_overlay(state: &GameState, sw: f32, sh: f32) -> bool {
    let w = 520.0_f32.min(sw - 40.0);
    let h = 300.0_f32.min(sh - 40.0);
    let x = (sw - w) / 2.0;
    let y = (sh - h) / 2.0;
    let panel = Rect::new(x, y, w, h);
    draw_panel(panel, None, DANGER);

    draw_centered_text(
        "THE CORE HAS FALLEN",
        Rect::new(x, y + 40.0, w, 30.0),
        30.0,
        DANGER,
    );
    draw_centered_text(
        "The realm's army has shattered your dungeon heart.",
        Rect::new(x, y + 96.0, w, 20.0),
        14.0,
        TEXT,
    );
    draw_centered_text(
        &format!(
            "You survived {} days and repelled {} sieges.",
            state.day, state.prestige
        ),
        Rect::new(x, y + 130.0, w, 20.0),
        13.0,
        TEXT_MUTED,
    );
    draw_centered_text(
        &format!("Adventurers slain across the run: {}", state.total_deaths),
        Rect::new(x, y + 156.0, w, 20.0),
        12.0,
        TEXT_MUTED,
    );

    let btn = Rect::new(x + w / 2.0 - 110.0, y + h - 70.0, 220.0, 44.0);
    draw_command_button(btn, "Raise a New Dungeon", ButtonTone::Primary, true)
}
