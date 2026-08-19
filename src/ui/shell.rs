use macroquad::prelude::*;

use crate::game_state::{DungeonStatus, GameState, RoomType};

use super::theme::*;
use macroquad_toolkit::colors::with_alpha;

pub const HUD_HEIGHT: f32 = 76.0;
pub const LOG_BAR_COLLAPSED_HEIGHT: f32 = 42.0;
pub const LOG_BAR_EXPANDED_HEIGHT: f32 = 132.0;
pub const OUTER_MARGIN: f32 = 10.0;
pub const PANEL_GAP: f32 = 10.0;
pub const SIDE_PANEL_WIDTH: f32 = 326.0;

/// The two top-shell controls that mutate simulation state. The older controls
/// panel had extra actions but is no longer rendered anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlAction {
    None,
    TogglePause,
    SetSpeed(i32),
    ToggleDungeon,
    OpenHelp,
    OpenCodex,
    OpenGoals,
}

/// The simulation may advance only for an interactive, unpaused frame.
/// Capture scenes deliberately render with `simulate` false, as do paused runs.
pub fn simulation_active(simulate: bool, paused: bool) -> bool {
    simulate && !paused
}

/// Draw the top HUD (resources, time, threat) plus the primary controls
/// (speed and dungeon open/close). Returns any control action triggered.
pub fn draw_top_hud(state: &GameState, rect: Rect) -> ControlAction {
    let mut action = ControlAction::None;
    draw_card(
        rect,
        Color::new(0.012, 0.017, 0.027, 0.98),
        with_alpha(TREASURE, 0.24),
    );
    draw_rectangle(
        rect.x + 1.0,
        rect.y + rect.h - 4.0,
        rect.w - 2.0,
        3.0,
        with_alpha(TREASURE, 0.16),
    );
    draw_line(
        rect.x,
        rect.y + rect.h - 1.0,
        rect.x + rect.w,
        rect.y + rect.h - 1.0,
        1.0,
        with_alpha(TREASURE, 0.22),
    );

    let title_w = (rect.w * 0.155).clamp(176.0, 208.0);
    let title_rect = Rect::new(rect.x + 14.0, rect.y + 9.0, title_w, rect.h - 18.0);
    draw_brand_mark(
        vec2(title_rect.x + 20.0, title_rect.y + title_rect.h * 0.5),
        19.0,
    );
    draw_text_fit(
        "DUNGEON CORE",
        title_rect.x + 44.0,
        title_rect.y + title_rect.h * 0.5 + 2.0,
        title_rect.w - 48.0,
        19.0,
        TEXT,
    );
    let utility_y = title_rect.y + title_rect.h - 18.0;
    let utility_x = title_rect.x + 44.0;
    for (index, (label, next_action)) in [
        ("HELP", ControlAction::OpenHelp),
        ("CODEX", ControlAction::OpenCodex),
        ("GOALS", ControlAction::OpenGoals),
    ]
    .iter()
    .enumerate()
    {
        let link = Rect::new(utility_x + index as f32 * 48.0, utility_y, 44.0, 17.0);
        if draw_header_link(link, label) {
            action = *next_action;
        }
    }

    // Right-hand control cluster: speed selector + dungeon toggle.
    let dungeon_w = 156.0_f32.min(rect.w * 0.14).max(128.0);
    let speed_w = 138.0_f32.min(rect.w * 0.13).max(116.0);
    let cluster_gap = 10.0;
    let control_h = 44.0;
    let control_y = rect.y + (rect.h - control_h) * 0.5;
    let dungeon_x = rect.x + rect.w - 14.0 - dungeon_w;
    let speed_x = dungeon_x - cluster_gap - speed_w;
    if let Some(time_action) = draw_speed_segments(
        Rect::new(speed_x, control_y, speed_w, control_h),
        state.speed,
        state.paused,
    ) {
        action = time_action;
    }

    let (status_text, status_tone, enabled) = match state.status {
        DungeonStatus::Open => ("Close Dungeon", ButtonTone::Danger, true),
        DungeonStatus::Closed => ("Open Dungeon", ButtonTone::Primary, true),
        DungeonStatus::Closing => ("Closing...", ButtonTone::Ghost, false),
    };
    if draw_command_button(
        Rect::new(dungeon_x, control_y, dungeon_w, control_h),
        status_text,
        status_tone,
        enabled,
    ) {
        action = ControlAction::ToggleDungeon;
    }

    // Resource + status stats fill the space between the title and controls.
    let stats_x = title_rect.x + title_rect.w + 12.0;
    let stats_w = speed_x - stats_x - 12.0;
    let stat_w = (stats_w / 5.0).clamp(90.0, 156.0);
    let y = rect.y + 10.0;
    let stat_h = rect.h - 20.0;

    draw_top_stat(
        Rect::new(stats_x, y, stat_w, stat_h),
        "Mana",
        &format!("{}/{}", state.mana, state.max_mana),
        MANA,
        StatIcon::Mana,
        Some((state.mana as f32, state.max_mana as f32)),
    );
    draw_top_stat(
        Rect::new(stats_x + stat_w, y, stat_w, stat_h),
        "Gold",
        &state.gold.to_string(),
        TREASURE,
        StatIcon::Gold,
        None,
    );
    draw_top_stat(
        Rect::new(stats_x + stat_w * 2.0, y, stat_w, stat_h),
        "Souls",
        &state.souls.to_string(),
        SOUL,
        StatIcon::Soul,
        None,
    );
    // During a siege the threat slot becomes a live core-HP readout.
    if state.siege_active {
        draw_top_stat(
            Rect::new(stats_x + stat_w * 3.0, y, stat_w, stat_h),
            "CORE UNDER SIEGE",
            &format!("{}/{}", state.core_hp, state.core_max_hp),
            DANGER,
            StatIcon::Threat,
            Some((state.core_hp as f32, state.core_max_hp as f32)),
        );
    } else {
        let (threat_label, threat_color) = threat_display(state);
        // A rising "dread" meter toward the siege makes threat feel like mounting
        // pressure instead of a silent number.
        draw_top_stat(
            Rect::new(stats_x + stat_w * 3.0, y, stat_w, stat_h),
            &format!("Threat ({})", state.total_deaths),
            &threat_label,
            threat_color,
            StatIcon::Threat,
            Some((state.total_deaths as f32, state.siege_threshold() as f32)),
        );
    }
    // Prestige reads as a named rank to climb, not a bare counter. The number
    // leads so it survives truncation in the narrow stat cell; the rank name
    // follows as flavour.
    let compact_stats = stat_w < 118.0;
    let time_label = if compact_stats {
        if state.prestige > 0 {
            format!("Prestige {}", state.prestige)
        } else {
            "Core Rank".to_string()
        }
    } else if state.prestige > 0 {
        format!(
            "P{} {}",
            state.prestige,
            crate::simulation::milestones::prestige_rank(state.prestige)
        )
    } else {
        crate::simulation::milestones::prestige_rank(0).to_string()
    };
    let time_value = if compact_stats {
        format!("D{} {:02}:00", state.day, state.hour)
    } else {
        format!("Day {} {:02}:00", state.day, state.hour)
    };
    draw_top_stat(
        Rect::new(stats_x + stat_w * 4.0, y, stat_w, stat_h),
        &time_label,
        &time_value,
        TEXT,
        StatIcon::Time,
        None,
    );

    action
}

/// A visible frozen-state marker. The board remains inspectable beneath it;
/// only the authoritative simulation has stopped.
pub fn draw_pause_overlay(rect: Rect) -> bool {
    let top = if rect.w < 760.0 { 110.0 } else { 72.0 };
    let card = Rect::new(rect.x + (rect.w - 300.0) * 0.5, rect.y + top, 300.0, 112.0);
    draw_card(card, with_alpha(BG_DEEP, 0.96), with_alpha(DANGER, 0.72));
    draw_centered_text(
        "PAUSED",
        Rect::new(card.x, card.y + 12.0, card.w, 22.0),
        18.0,
        TEXT,
    );
    draw_centered_text(
        "The dungeon is frozen. Inspect or plan freely.",
        Rect::new(card.x + 12.0, card.y + 40.0, card.w - 24.0, 18.0),
        11.0,
        TEXT_MUTED,
    );
    draw_command_button(
        Rect::new(card.x + 42.0, card.y + 68.0, card.w - 84.0, 34.0),
        "Resume Dungeon",
        ButtonTone::Primary,
        true,
    )
}

/// Threat readout derived from accumulated adventurer deaths.
fn threat_display(state: &GameState) -> (String, Color) {
    match state.threat_tier() {
        0 => ("Calm".to_string(), EMERALD),
        1 => ("Wary".to_string(), TREASURE),
        2 => ("Alarmed".to_string(), WARNING),
        3 => ("Hunted".to_string(), DANGER),
        _ => ("Besieged".to_string(), DANGER),
    }
}

pub fn draw_adventurer_status_chip(state: &GameState, rect: Rect) {
    let (label, color, icon) = adventurer_status(state);
    draw_card(rect, with_alpha(color, 0.10), with_alpha(color, 0.42));
    draw_text_fit(
        icon,
        rect.x + 12.0,
        rect.y + rect.h * 0.62,
        24.0,
        18.0,
        color,
    );
    draw_centered_text(
        label,
        Rect::new(rect.x + 28.0, rect.y, rect.w - 34.0, rect.h),
        13.0,
        color,
    );
}

#[derive(Clone, Copy)]
enum StatIcon {
    Mana,
    Gold,
    Soul,
    Time,
    Threat,
}

fn draw_top_stat(
    rect: Rect,
    label: &str,
    value: &str,
    color: Color,
    icon: StatIcon,
    bar: Option<(f32, f32)>,
) {
    let compact = rect.w < 118.0;
    let icon_x = if compact {
        rect.x + 16.0
    } else {
        rect.x + 28.0
    };
    let text_x = if compact {
        rect.x + 31.0
    } else {
        rect.x + 50.0
    };
    let text_w = (rect.x + rect.w - text_x - 5.0).max(22.0);
    draw_line(
        rect.x,
        rect.y,
        rect.x,
        rect.y + rect.h,
        1.0,
        with_alpha(BORDER, 0.20),
    );
    draw_stat_icon(
        vec2(icon_x, rect.y + rect.h * 0.54),
        if compact { 10.0 } else { 13.0 },
        icon,
        color,
    );
    if !label.is_empty() {
        draw_text_fit(
            label,
            text_x,
            rect.y + 16.0,
            text_w,
            if compact { 9.0 } else { 11.0 },
            TEXT_MUTED,
        );
    }
    draw_text_fit(
        value,
        text_x,
        if label.is_empty() {
            rect.y + 29.0
        } else {
            rect.y + 38.0
        },
        text_w,
        if compact {
            14.0
        } else if label.is_empty() {
            17.0
        } else {
            18.0
        },
        color,
    );
    if let Some((current, max)) = bar {
        draw_bar(
            Rect::new(text_x, rect.y + rect.h - 4.0, (text_w - 8.0).max(12.0), 3.0),
            current,
            max,
            color,
            None,
        );
    }
    if rect.contains(vec2(mouse_position().0, mouse_position().1)) {
        let text = match label {
            "Mana" => "Mana powers rooms, summons, upgrades, respawns, and Core Smite. The bar is your current capacity.",
            "Gold" => "Gold pays room upgrades and variant swaps. Adventurers bank it when they escape.",
            "Souls" => "Souls come from boss kills and buy permanent Core Powers.",
            label if label.starts_with("Threat") => "Threat rises with adventurer deaths. At the final tier, the realm sends a siege.",
            "CORE UNDER SIEGE" => "The core must survive this siege. Defeat or repel the invading party before its health reaches zero.",
            _ => "This shows your prestige rank and the current dungeon day and hour.",
        };
        macroquad_toolkit::ui::draw_tooltip(text, vec2(rect.x, rect.y + rect.h));
    }
}

fn draw_speed_segments(rect: Rect, speed: i32, paused: bool) -> Option<ControlAction> {
    let pointer = vec2(mouse_position().0, mouse_position().1);
    let released = is_mouse_button_released(MouseButton::Left);
    let mut action = None;
    draw_card(rect, Color::new(0.018, 0.028, 0.045, 0.94), BORDER_MUTED);
    let labels = ["||", "1x", "2x", "4x"];
    let seg_w = rect.w / labels.len() as f32;
    for (idx, label) in labels.iter().enumerate() {
        let seg = Rect::new(rect.x + idx as f32 * seg_w, rect.y, seg_w, rect.h);
        let active = (idx == 0 && paused)
            || (!paused
                && ((idx == 1 && speed == 1)
                    || (idx == 2 && speed == 2)
                    || (idx == 3 && speed >= 4)));
        if active {
            draw_rectangle(seg.x, seg.y, seg.w, seg.h, with_alpha(MANA, 0.12));
        }
        if idx > 0 {
            draw_line(
                seg.x,
                seg.y + 8.0,
                seg.x,
                seg.y + seg.h - 8.0,
                1.0,
                BORDER_MUTED,
            );
        }
        draw_centered_text(label, seg, 15.0, if active { TEXT } else { TEXT_DIM });
        if released && seg.contains(pointer) {
            action = Some(match idx {
                0 => ControlAction::TogglePause,
                1 => ControlAction::SetSpeed(1),
                2 => ControlAction::SetSpeed(2),
                _ => ControlAction::SetSpeed(4),
            });
        }
    }
    action
}

fn draw_header_link(rect: Rect, label: &str) -> bool {
    let pointer = vec2(mouse_position().0, mouse_position().1);
    let hovered = rect.contains(pointer);
    if hovered {
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, with_alpha(SOUL, 0.14));
    }
    draw_centered_text(label, rect, 8.0, if hovered { TEXT } else { TEXT_DIM });
    hovered && is_mouse_button_released(MouseButton::Left)
}

fn draw_brand_mark(center: Vec2, radius: f32) {
    draw_poly(center.x, center.y, 4, radius, 45.0, with_alpha(SOUL, 0.22));
    draw_poly_lines(center.x, center.y, 4, radius, 45.0, 2.0, SOUL);
    draw_poly_lines(
        center.x,
        center.y,
        4,
        radius * 0.66,
        45.0,
        2.0,
        Color::new(1.0, 0.85, 1.0, 0.80),
    );
    draw_line(
        center.x,
        center.y - radius,
        center.x,
        center.y + radius,
        2.0,
        SOUL,
    );
    draw_line(
        center.x - radius,
        center.y,
        center.x + radius,
        center.y,
        2.0,
        SOUL,
    );
}

fn draw_stat_icon(center: Vec2, radius: f32, icon: StatIcon, color: Color) {
    match icon {
        StatIcon::Mana => {
            draw_circle(
                center.x,
                center.y + 3.0,
                radius * 0.55,
                with_alpha(color, 0.22),
            );
            draw_triangle(
                vec2(center.x, center.y - radius),
                vec2(center.x - radius * 0.55, center.y + radius * 0.35),
                vec2(center.x + radius * 0.55, center.y + radius * 0.35),
                color,
            );
        }
        StatIcon::Gold => {
            draw_circle(center.x, center.y, radius, with_alpha(color, 0.18));
            draw_circle_lines(center.x, center.y, radius, 2.0, color);
            draw_text_fit(
                "G",
                center.x - radius * 0.42,
                center.y + radius * 0.46,
                radius * 0.9,
                radius,
                color,
            );
        }
        StatIcon::Soul => {
            draw_poly(center.x, center.y, 4, radius, 45.0, with_alpha(color, 0.20));
            draw_poly_lines(center.x, center.y, 4, radius, 45.0, 2.0, color);
        }
        StatIcon::Time => {
            draw_circle_lines(center.x, center.y, radius, 1.6, color);
            draw_line(
                center.x,
                center.y,
                center.x,
                center.y - radius * 0.62,
                1.5,
                color,
            );
            draw_line(
                center.x,
                center.y,
                center.x + radius * 0.55,
                center.y,
                1.5,
                color,
            );
        }
        StatIcon::Threat => {
            // A warning triangle with an exclamation mark.
            draw_triangle_lines(
                vec2(center.x, center.y - radius),
                vec2(center.x - radius * 0.9, center.y + radius * 0.7),
                vec2(center.x + radius * 0.9, center.y + radius * 0.7),
                2.0,
                color,
            );
            draw_line(
                center.x,
                center.y - radius * 0.36,
                center.x,
                center.y + radius * 0.2,
                2.0,
                color,
            );
            draw_circle(center.x, center.y + radius * 0.46, 1.4, color);
        }
    }
}

fn adventurer_status(state: &GameState) -> (&'static str, Color, &'static str) {
    if state.adventurer_parties.is_empty() {
        return ("SAFE TO REBUILD", EMERALD, "+");
    }

    let core_threat = state.adventurer_parties.iter().any(|party| {
        state.floors.iter().any(|floor| {
            floor.number == party.current_floor
                && floor.rooms.iter().any(|room| {
                    room.position == party.current_room && room.room_type == RoomType::Core
                })
        })
    });
    if core_threat {
        ("CORE UNDER THREAT", DANGER, "!")
    } else if state
        .adventurer_parties
        .iter()
        .any(|party| party.current_room == 0)
    {
        ("ADVENTURERS APPROACHING", WARNING, "!")
    } else {
        ("PARTY INSIDE", WARNING, "!")
    }
}
