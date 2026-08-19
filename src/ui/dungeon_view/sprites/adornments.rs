//! Procedural progression marks layered around monster sprites. The atlas
//! supplies each species silhouette; these element-colored auras and tier
//! crests make evolved forms legible even at the board's smallest scale.

use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AdornmentStyle {
    tier: i32,
    element: ElementMark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ElementMark {
    Fire,
    Water,
    Nature,
    Earth,
    Death,
    Arcane,
    Air,
    Spirit,
    Body,
}

fn adornment_style(name: &str) -> Option<AdornmentStyle> {
    let template = crate::data::monsters::get_monster_template(name)?;
    let element = match template.element.as_deref()? {
        "Fire" => ElementMark::Fire,
        "Water" => ElementMark::Water,
        "Nature" => ElementMark::Nature,
        "Earth" => ElementMark::Earth,
        "Death" => ElementMark::Death,
        "Arcane" => ElementMark::Arcane,
        "Air" => ElementMark::Air,
        "Spirit" => ElementMark::Spirit,
        _ => ElementMark::Body,
    };
    Some(AdornmentStyle {
        tier: template.tier,
        element,
    })
}

fn mark_color(mark: ElementMark, alpha: f32) -> Color {
    let (r, g, b) = match mark {
        ElementMark::Fire => (1.0, 0.31, 0.16),
        ElementMark::Water => (0.22, 0.65, 1.0),
        ElementMark::Nature => (0.30, 0.92, 0.40),
        ElementMark::Earth => (0.82, 0.56, 0.24),
        ElementMark::Death => (0.64, 0.36, 0.88),
        ElementMark::Arcane => (0.80, 0.38, 1.0),
        ElementMark::Air => (0.48, 0.90, 1.0),
        ElementMark::Spirit => (0.90, 0.65, 1.0),
        ElementMark::Body => (0.90, 0.86, 0.77),
    };
    Color::new(r, g, b, alpha)
}

pub(super) fn draw_backdrop(name: &str, center: Vec2, size: f32) {
    let Some(style) = adornment_style(name) else {
        return;
    };
    if style.tier < 3 {
        return;
    }
    let radius = size * (0.42 + style.tier as f32 * 0.025);
    draw_circle(center.x, center.y, radius, mark_color(style.element, 0.075));
    draw_circle_lines(
        center.x,
        center.y,
        radius,
        (size * 0.035).max(0.8),
        mark_color(style.element, 0.48),
    );
    if style.tier >= 4 {
        draw_circle_lines(
            center.x,
            center.y,
            radius + size * 0.09,
            (size * 0.025).max(0.7),
            mark_color(style.element, 0.30),
        );
    }
}

pub(super) fn draw_crest(name: &str, center: Vec2, size: f32) {
    let Some(style) = adornment_style(name) else {
        return;
    };
    if style.tier < 2 {
        return;
    }
    let color = mark_color(style.element, 0.95);
    let gem = center + vec2(0.0, -size * 0.43);
    let half = (size * 0.065).max(1.3);
    draw_triangle(
        gem + vec2(0.0, -half),
        gem + vec2(half, 0.0),
        gem + vec2(0.0, half),
        color,
    );
    draw_triangle(
        gem + vec2(0.0, -half),
        gem + vec2(0.0, half),
        gem + vec2(-half, 0.0),
        color,
    );
    if style.tier >= 3 {
        for side in [-1.0, 1.0] {
            draw_circle(
                gem.x + side * size * 0.13,
                gem.y + size * 0.02,
                (size * 0.022).max(0.7),
                mark_color(style.element, 0.72),
            );
        }
    }
}

#[cfg(test)]
mod tests;
