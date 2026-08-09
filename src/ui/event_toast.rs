use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

use crate::game_state::{GameState, LogEntry, LogFilter};

use super::theme::*;

/// Draw the compact event strip, with an expanded filtered history on demand.
pub fn draw_event_log(state: &mut GameState, rect: Rect, expanded: &mut bool) {
    draw_panel(rect, None, MANA);

    let toggle = Rect::new(rect.x, rect.y, rect.w, 30.0_f32.min(rect.h));
    let matching_count = state
        .log
        .iter()
        .filter(|entry| state.log_filter.matches(entry))
        .count();
    draw_text_fit(
        &format!(
            "EVENT LOG · {matching_count} {}  [{}]",
            if matching_count == 1 {
                "message"
            } else {
                "messages"
            },
            if *expanded { "CLOSE" } else { "OPEN" }
        ),
        toggle.x + 12.0,
        toggle.y + 20.0,
        230.0,
        13.0,
        MANA,
    );
    if toggle.contains(vec2(mouse_position().0, mouse_position().1))
        && is_mouse_button_released(MouseButton::Left)
    {
        *expanded = !*expanded;
    }
    if !*expanded {
        if let Some(entry) = state
            .log
            .iter()
            .rev()
            .find(|entry| state.log_filter.matches(entry))
        {
            draw_text_fit(
                &entry.message,
                rect.x + 248.0,
                rect.y + 20.0,
                (rect.w - 270.0).max(80.0),
                12.0,
                event_color(entry),
            );
        }
        return;
    }

    let filters = [
        (LogFilter::All, "ALL"),
        (LogFilter::Combat, "COM"),
        (LogFilter::Adventure, "ADV"),
        (LogFilter::Building, "BLD"),
        (LogFilter::System, "SYS"),
    ];
    let filter_w = 34.0;
    let filter_y = rect.y + 7.0;
    for (index, (filter, label)) in filters.iter().enumerate() {
        let filter_rect = Rect::new(
            rect.x + rect.w - 12.0 - filter_w * (filters.len() - index) as f32,
            filter_y,
            filter_w - 3.0,
            17.0,
        );
        let active = state.log_filter == *filter;
        let hovered = filter_rect.contains(vec2(mouse_position().0, mouse_position().1));
        if hovered && is_mouse_button_released(MouseButton::Left) {
            state.log_filter = *filter;
            state.log_scroll = 0;
        }
        draw_card(
            filter_rect,
            if active {
                with_alpha(MANA, 0.22)
            } else {
                PANEL_ALT
            },
            if active {
                with_alpha(MANA, 0.72)
            } else {
                BORDER_MUTED
            },
        );
        draw_centered_text(
            label,
            filter_rect,
            8.0,
            if active { TEXT } else { TEXT_DIM },
        );
    }

    let inner = Rect::new(rect.x + 12.0, rect.y + 36.0, rect.w - 24.0, rect.h - 44.0);
    let line_h = 17.0;
    let max_lines = (inner.h / line_h).floor().max(1.0) as usize;

    let matching: Vec<&LogEntry> = state
        .log
        .iter()
        .filter(|entry| state.log_filter.matches(entry))
        .collect();
    if matching.is_empty() {
        draw_text_fit(
            "No matching events yet.",
            inner.x,
            inner.y + 14.0,
            inner.w,
            12.0,
            TEXT_DIM,
        );
        return;
    }

    if inner.contains(vec2(mouse_position().0, mouse_position().1)) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.0 {
            state.log_scroll = if wheel_y > 0.0 {
                state.log_scroll.saturating_add(1)
            } else {
                state.log_scroll.saturating_sub(1)
            };
        }
    }
    let max_scroll = matching.len().saturating_sub(max_lines);
    state.log_scroll = state.log_scroll.min(max_scroll);
    let end = matching.len() - state.log_scroll;
    let start = end.saturating_sub(max_lines);

    // Oldest of the visible window first, newest at the bottom.
    let entries = &matching[start..end];
    let mut y = inner.y + 13.0;
    for entry in entries {
        let color = event_color(entry);
        draw_text_fit(event_label(entry), inner.x, y, 34.0, 10.0, color);
        draw_text_fit(
            &entry.message,
            inner.x + 40.0,
            y,
            inner.w - 40.0,
            12.0,
            TEXT_MUTED,
        );
        y += line_h;
    }

    if matching.len() > max_lines {
        draw_text_fit_right(
            &format!("{}-{} of {} · scroll", start + 1, end, matching.len()),
            inner.x + inner.w,
            rect.y + 21.0,
            120.0,
            9.0,
            TEXT_DIM,
        );
    }
}

fn event_color(entry: &LogEntry) -> Color {
    match entry.log_type.as_str() {
        "combat" => DANGER,
        "adventure" => WARNING,
        "building" => EMERALD,
        _ => MANA,
    }
}

fn event_label(entry: &LogEntry) -> &'static str {
    match entry.log_type.as_str() {
        "combat" => "COM",
        "adventure" => "ADV",
        "building" => "BLD",
        _ => "SYS",
    }
}
