use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::input::is_hovered_rect;
use macroquad_toolkit::ui::{
    draw_surface, draw_text_centered_in_box, truncate_text_to_width, SurfaceStyle,
};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub const BG: Color = Color::new(0.010, 0.014, 0.022, 1.0);
pub const BG_DEEP: Color = Color::new(0.004, 0.006, 0.011, 1.0);
pub const PANEL: Color = Color::new(0.024, 0.031, 0.047, 1.0);
pub const PANEL_ALT: Color = Color::new(0.035, 0.043, 0.064, 1.0);
pub const PANEL_HEADER: Color = Color::new(0.050, 0.053, 0.076, 1.0);
pub const CARD: Color = Color::new(0.026, 0.033, 0.050, 0.86);
pub const BORDER: Color = Color::new(0.240, 0.270, 0.340, 0.48);
pub const BORDER_MUTED: Color = Color::new(0.140, 0.165, 0.220, 0.36);
pub const TEXT: Color = Color::new(0.890, 0.870, 0.810, 1.0);
pub const TEXT_MUTED: Color = Color::new(0.560, 0.585, 0.650, 1.0);
pub const TEXT_DIM: Color = Color::new(0.350, 0.380, 0.450, 1.0);
pub const MANA: Color = Color::new(0.300, 0.650, 1.000, 1.0);
pub const TREASURE: Color = Color::new(1.000, 0.680, 0.180, 1.0);
pub const SOUL: Color = Color::new(0.720, 0.360, 1.000, 1.0);
pub const ARCANE: Color = Color::new(0.610, 0.300, 1.000, 1.0);
pub const EMERALD: Color = Color::new(0.270, 0.860, 0.430, 1.0);
pub const DANGER: Color = Color::new(0.940, 0.250, 0.300, 1.0);
pub const WARNING: Color = Color::new(1.000, 0.560, 0.140, 1.0);

#[derive(Clone, Copy)]
pub enum ButtonTone {
    Primary,
    Danger,
    Ghost,
    Arcane,
}

/// Distinct hue for each combat element, so same-element units read alike and
/// different ones read apart. Paired with a unit's letter glyph, this stays
/// legible without relying on colour alone. Unknown/neutral falls back to green.
pub fn element_color(element: &str) -> Color {
    match element {
        "Fire" => Color::new(1.00, 0.42, 0.24, 1.0),
        "Water" => Color::new(0.32, 0.64, 1.00, 1.0),
        "Nature" => Color::new(0.42, 0.84, 0.40, 1.0),
        "Earth" => Color::new(0.80, 0.62, 0.36, 1.0),
        "Air" => Color::new(0.66, 0.90, 0.96, 1.0),
        "Spirit" => Color::new(1.00, 0.90, 0.56, 1.0),
        "Death" => Color::new(0.64, 0.44, 0.80, 1.0),
        "Arcane" => Color::new(0.72, 0.40, 1.00, 1.0),
        "Body" => Color::new(0.92, 0.72, 0.62, 1.0),
        _ => EMERALD,
    }
}

/// A compact, colour-independent identifier for a combat element. These are
/// intentionally distinct even where element names begin alike (Air/Arcane),
/// so small board badges remain unambiguous.
pub fn element_marker(element: &str) -> &'static str {
    match element {
        "Fire" => "FI",
        "Water" => "WA",
        "Nature" => "NA",
        "Earth" => "EA",
        "Air" => "AI",
        "Spirit" => "SP",
        "Death" => "DE",
        "Arcane" => "AR",
        "Body" => "BO",
        _ => "NE",
    }
}

/// A small element badge for sprites and other compact board tokens. Its
/// outlined lozenge and text label carry the distinction without hue alone.
pub fn draw_element_badge(center: Vec2, element: &str) {
    let color = element_color(element);
    let rect = Rect::new(center.x - 8.0, center.y - 5.5, 16.0, 11.0);
    draw_card(rect, with_alpha(BG_DEEP, 0.88), with_alpha(color, 0.92));
    draw_centered_text(element_marker(element), rect, 7.0, TEXT);
}

pub fn draw_game_background(sw: f32, sh: f32) {
    clear_background(BG_DEEP);
    draw_rectangle(0.0, 0.0, sw, sh, BG);

    draw_rectangle(0.0, 0.0, sw, 100.0, Color::new(0.0, 0.0, 0.0, 0.34));
    draw_rectangle(0.0, sh - 116.0, sw, 116.0, Color::new(0.0, 0.0, 0.0, 0.30));
    draw_line(0.0, 88.0, sw, 88.0, 1.0, with_alpha(TREASURE, 0.18));

    let grid_color = Color::new(0.24, 0.30, 0.40, 0.014);
    let mut x = 0.0;
    while x <= sw {
        draw_line(x, 0.0, x, sh, 1.0, grid_color);
        x += 72.0;
    }
    let mut y = 0.0;
    while y <= sh {
        draw_line(0.0, y, sw, y, 1.0, grid_color);
        y += 72.0;
    }
}

pub fn draw_panel(rect: Rect, title: Option<&str>, accent: Color) {
    let style = SurfaceStyle::new(PANEL)
        .with_shadow(vec2(4.0, 5.0), Color::new(0.0, 0.0, 0.0, 0.40))
        .with_border(1.0, with_alpha(BORDER, 0.42))
        .with_inner_border(1.0, 1.0, with_alpha(accent, 0.09));
    draw_surface(rect, &style);

    if let Some(title) = title {
        draw_rectangle(rect.x, rect.y, rect.w, 30.0_f32.min(rect.h), PANEL_HEADER);
        draw_line(
            rect.x,
            rect.y + 30.0,
            rect.x + rect.w,
            rect.y + 30.0,
            1.0,
            BORDER_MUTED,
        );
        draw_text_fit(
            title,
            rect.x + 12.0,
            rect.y + 20.0,
            rect.w - 24.0,
            16.0,
            accent,
        );
    }
}

pub fn draw_card(rect: Rect, fill: Color, border: Color) {
    let style = SurfaceStyle::new(fill)
        .with_border(1.0, border)
        .with_inner_border(1.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.028));
    draw_surface(rect, &style);
}

pub fn draw_text_fit(text: &str, x: f32, baseline_y: f32, max_width: f32, size: f32, color: Color) {
    let clipped = truncate_text_to_width(text, max_width.max(8.0), size);
    draw_ui_text(&clipped, x, baseline_y, size, color);
}

pub fn draw_text_fit_right(
    text: &str,
    right_x: f32,
    baseline_y: f32,
    max_width: f32,
    size: f32,
    color: Color,
) {
    let clipped = truncate_text_to_width(text, max_width.max(8.0), size);
    let dims = measure_ui_text(&clipped, None, size as u16, 1.0);
    draw_ui_text(&clipped, right_x - dims.width, baseline_y, size, color);
}

pub fn draw_centered_text(text: &str, rect: Rect, size: f32, color: Color) {
    draw_text_centered_in_box(text, rect.x, rect.y, rect.w, rect.h, size, color);
}

pub fn draw_pill(rect: Rect, text: &str, color: Color) {
    draw_card(rect, with_alpha(color, 0.14), with_alpha(color, 0.58));
    draw_centered_text(
        text,
        rect,
        12.0,
        Color::new(color.r.max(0.78), color.g.max(0.78), color.b.max(0.78), 1.0),
    );
}

pub fn draw_icon_disc(center: Vec2, radius: f32, color: Color, label: &str) {
    draw_circle(center.x, center.y, radius, with_alpha(color, 0.22));
    draw_circle_lines(center.x, center.y, radius, 1.5, with_alpha(color, 0.72));
    draw_centered_text(
        label,
        Rect::new(
            center.x - radius,
            center.y - radius + 1.0,
            radius * 2.0,
            radius * 2.0,
        ),
        radius * 0.88,
        color,
    );
}

pub fn draw_command_button(rect: Rect, text: &str, tone: ButtonTone, enabled: bool) -> bool {
    let hovered = enabled && is_hovered_rect(rect);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
    let clicked = hovered && is_mouse_button_released(MouseButton::Left);

    let (base, border, text_color) = match tone {
        ButtonTone::Primary => (
            Color::new(0.13, 0.58, 0.34, 1.0),
            EMERALD,
            Color::new(0.92, 1.0, 0.95, 1.0),
        ),
        ButtonTone::Danger => (
            Color::new(0.58, 0.12, 0.13, 1.0),
            DANGER,
            Color::new(1.0, 0.93, 0.93, 1.0),
        ),
        ButtonTone::Ghost => (Color::new(0.105, 0.130, 0.185, 1.0), BORDER, TEXT),
        ButtonTone::Arcane => (
            Color::new(0.34, 0.14, 0.56, 1.0),
            SOUL,
            Color::new(0.96, 0.91, 1.0, 1.0),
        ),
    };

    let mut fill = if pressed {
        Color::new(base.r * 0.72, base.g * 0.72, base.b * 0.72, base.a)
    } else if hovered {
        Color::new(
            (base.r * 1.22).min(1.0),
            (base.g * 1.22).min(1.0),
            (base.b * 1.22).min(1.0),
            base.a,
        )
    } else {
        base
    };

    if !enabled {
        fill = Color::new(0.075, 0.085, 0.110, 1.0);
    }

    draw_card(
        rect,
        fill,
        if enabled {
            border
        } else {
            with_alpha(BORDER, 0.36)
        },
    );
    let y_offset = if pressed { 1.5 } else { 0.0 };
    draw_centered_text(
        text,
        Rect::new(rect.x + 6.0, rect.y + y_offset, rect.w - 12.0, rect.h),
        15.0,
        if enabled { text_color } else { TEXT_DIM },
    );

    clicked
}

pub fn draw_bar(rect: Rect, value: f32, max: f32, color: Color, label: Option<&str>) {
    let ratio = if max <= 0.0 {
        0.0
    } else {
        (value / max).clamp(0.0, 1.0)
    };
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.018, 0.024, 0.040, 0.82),
    );
    draw_rectangle(rect.x, rect.y, rect.w * ratio, rect.h, color);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BORDER_MUTED);
    if let Some(label) = label {
        draw_centered_text(label, rect, 11.0, TEXT);
    }
}
