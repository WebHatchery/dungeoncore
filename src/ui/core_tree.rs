//! The Core Power tree overlay: a branching board of soul-bought permanent
//! upgrades. Roots sit at the top; each lower tier unlocks once its
//! prerequisites are awakened, so repeated prestiges can specialise down the
//! economy, bulwark, or offense lines. Opened with the BUILD tab button or [P].

use macroquad::prelude::*;
use macroquad_toolkit::input::is_hovered_rect;

use crate::game_state::GameState;
use crate::simulation::endgame::{core_power, prereqs_met, CorePower, CORE_POWERS};

use super::theme::*;
use super::upgrade_panel::draw_close_button;
use macroquad_toolkit::colors::with_alpha;

/// Outcome of a frame of the core-power tree overlay.
pub enum CoreTreeResult {
    None,
    Buy(String),
    Close,
}

/// Draw the core-power tree overlay and handle a purchase click.
pub fn draw_core_tree(state: &GameState, sw: f32, sh: f32) -> CoreTreeResult {
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.82));
    let w = (sw - 80.0).min(1040.0);
    let h = (sh - 60.0).min(660.0);
    let x = (sw - w) / 2.0;
    let y = (sh - h) / 2.0;
    let panel = Rect::new(x, y, w, h);
    draw_panel(panel, Some("Core Power Tree"), ARCANE);

    let mut result = CoreTreeResult::None;
    if draw_close_button(Rect::new(x + w - 48.0, y + 8.0, 40.0, 34.0))
        || is_key_pressed(KeyCode::P)
        || is_key_pressed(KeyCode::Escape)
    {
        result = CoreTreeResult::Close;
    }

    draw_text_fit(
        &format!("{} souls banked", state.souls),
        x + 20.0,
        y + 34.0,
        w - 120.0,
        14.0,
        SOUL,
    );
    draw_text_fit(
        "Awaken permanent powers with souls. Lower tiers unlock once their prerequisite is owned.",
        x + 20.0,
        y + 52.0,
        w - 60.0,
        11.0,
        TEXT_DIM,
    );

    let lane_header = Rect::new(x + 20.0, y + 64.0, w - 40.0, 20.0);
    for (label, lane, color) in [
        ("WELLSPRING", 0.5, MANA),
        ("BULWARK", 2.0, EMERALD),
        ("DOMINION", 3.5, SOUL),
    ] {
        let lane_w = lane_header.w / 5.0;
        draw_centered_text(
            label,
            Rect::new(
                lane_header.x + lane * lane_w - lane_w * 0.5,
                lane_header.y,
                lane_w,
                lane_header.h,
            ),
            9.0,
            with_alpha(color, 0.74),
        );
    }
    let content = Rect::new(x + 20.0, y + 84.0, w - 40.0, h - 104.0);
    let max_tier = CORE_POWERS.iter().map(|p| p.tier).max().unwrap_or(0);
    let rows = max_tier as usize + 1;
    let row_h = content.h / rows as f32;
    let node_h = (row_h - 24.0).clamp(56.0, 82.0);
    let gap = 14.0;
    let node_w = (content.w - gap * 4.0) / 5.0;
    let lane_step = node_w + gap;

    // First pass: position every node so connectors can be drawn beneath cards.
    let mut placed: Vec<(&CorePower, Rect)> = Vec::with_capacity(CORE_POWERS.len());
    for power in &CORE_POWERS {
        let ny = content.y + power.tier as f32 * row_h + (row_h - node_h) * 0.5;
        let nx = (content.x + power_lane(power.id) * lane_step)
            .clamp(content.x, content.x + content.w - node_w);
        placed.push((power, Rect::new(nx, ny, node_w, node_h)));
    }

    // Connectors: link each node to its prerequisites.
    for (power, rect) in &placed {
        let owned = state.has_core_power(power.id);
        let available = !owned && prereqs_met(state, power);
        let line_color = if owned {
            with_alpha(EMERALD, 0.5)
        } else if available {
            with_alpha(SOUL, 0.4)
        } else {
            with_alpha(BORDER, 0.5)
        };
        for req in power.requires {
            if let Some((_, req_rect)) = placed.iter().find(|(p, _)| &p.id == req) {
                let from = vec2(req_rect.x + req_rect.w * 0.5, req_rect.y + req_rect.h);
                let to = vec2(rect.x + rect.w * 0.5, rect.y);
                let elbow_y = (from.y + to.y) * 0.5;
                draw_line(from.x, from.y, from.x, elbow_y, 2.0, line_color);
                draw_line(from.x, elbow_y, to.x, elbow_y, 2.0, line_color);
                draw_line(to.x, elbow_y, to.x, to.y, 2.0, line_color);
            }
        }
    }

    // Second pass: draw the node cards and handle the purchase click.
    for (power, rect) in &placed {
        if let Some(id) = draw_node(state, power, *rect) {
            result = CoreTreeResult::Buy(id);
        }
    }

    result
}

/// Draw one power node; returns its id if the player clicked to awaken it.
fn draw_node(state: &GameState, power: &CorePower, rect: Rect) -> Option<String> {
    let owned = state.has_core_power(power.id);
    let unlocked = prereqs_met(state, power);
    let available = !owned && unlocked;
    let affordable = state.souls >= power.cost;

    let accent = if owned {
        EMERALD
    } else if available {
        SOUL
    } else {
        BORDER
    };
    let pointer_hovered = is_hovered_rect(rect);
    let hovered = available && pointer_hovered;
    let clicked = hovered && is_mouse_button_released(MouseButton::Left);

    let fill_alpha = if owned {
        0.14
    } else if available {
        if hovered {
            0.20
        } else {
            0.10
        }
    } else {
        0.04
    };
    draw_card(
        rect,
        with_alpha(accent, fill_alpha),
        with_alpha(accent, if available { 0.55 } else { 0.28 }),
    );

    let title_color = if owned || available { TEXT } else { TEXT_DIM };
    draw_text_fit(
        power.name,
        rect.x + 12.0,
        rect.y + 20.0,
        rect.w - 24.0,
        14.0,
        title_color,
    );
    draw_text_fit(
        power.description,
        rect.x + 12.0,
        rect.y + 38.0,
        rect.w - 24.0,
        9.5,
        if owned || available {
            TEXT_MUTED
        } else {
            TEXT_DIM
        },
    );

    // Status line: OWNED / cost / locked-with-prereq.
    if owned {
        draw_pill(
            Rect::new(rect.x + rect.w - 60.0, rect.y + rect.h - 24.0, 50.0, 16.0),
            "OWNED",
            EMERALD,
        );
    } else if available {
        let cost_color = if affordable { SOUL } else { DANGER };
        draw_text_fit(
            &format!("{} souls", power.cost),
            rect.x + 12.0,
            rect.y + rect.h - 10.0,
            rect.w - 24.0,
            12.0,
            cost_color,
        );
        draw_text_fit(
            if affordable {
                "Tap to awaken"
            } else {
                "Need souls"
            },
            rect.x + rect.w - 108.0,
            rect.y + rect.h - 10.0,
            100.0,
            9.5,
            if affordable { EMERALD } else { TEXT_DIM },
        );
    } else {
        let req_name = power
            .requires
            .first()
            .and_then(|r| core_power(r))
            .map(|p| p.name)
            .unwrap_or("prerequisite");
        draw_text_fit(
            &format!("Requires {}", req_name),
            rect.x + 12.0,
            rect.y + rect.h - 10.0,
            rect.w - 24.0,
            9.5,
            TEXT_DIM,
        );
    }

    if pointer_hovered {
        let status = if owned {
            "Already awakened; this permanent power survives prestige."
        } else if !unlocked {
            "Locked until its prerequisite Core Power is awakened."
        } else if !affordable {
            "Unlocked, but more souls are needed before it can be awakened."
        } else {
            "Tap to spend souls and awaken this permanent power."
        };
        crate::ui::draw_tooltip(
            &format!("{} {}", power.description, status),
            vec2(rect.x, rect.y + rect.h),
        );
    }

    if clicked && affordable {
        Some(power.id.to_string())
    } else {
        None
    }
}

/// Stable conceptual lanes keep the three power families readable even where
/// one capstone has two prerequisites. Fractional lanes center shared roots
/// and capstones between their branches.
fn power_lane(id: &str) -> f32 {
    match id {
        "deep_roots" => 0.5,
        "bulwark_core" => 2.0,
        "dread_aura" => 3.5,
        "wellspring" | "aquifer" | "eternal_wellspring" => 0.0,
        "mana_font" | "grand_reservoir" => 1.0,
        "iron_heart" | "adamant_heart" => 2.0,
        "searing_smite" | "cataclysm" => 3.0,
        "quickening" | "terror_incarnate" => 4.0,
        "worldbreaker" => 3.5,
        _ => 2.0,
    }
}
