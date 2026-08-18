//! Transient camera controls for the world-space dungeon cutaway.

use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::input::was_clicked_rect;

use crate::game_state::GameState;
use crate::ui::theme::*;

pub(super) fn update(state: &mut GameState, viewport: Rect, max_pan_x: f32, max_pan_y: f32) {
    let pointer = vec2(mouse_position().0, mouse_position().1);
    if is_mouse_button_pressed(MouseButton::Left) {
        state.board_dragged = false;
        state.board_drag_last = viewport.contains(pointer).then_some((pointer.x, pointer.y));
    }
    if is_mouse_button_down(MouseButton::Left) {
        if let Some((last_x, last_y)) = state.board_drag_last {
            let delta = vec2(pointer.x - last_x, pointer.y - last_y);
            if delta.length_squared() > 0.25 {
                state.board_pan_x = (state.board_pan_x - delta.x).clamp(0.0, max_pan_x);
                state.board_scroll = (state.board_scroll - delta.y).clamp(0.0, max_pan_y);
                state.board_dragged = true;
            }
        }
        if state.board_drag_last.is_some() {
            state.board_drag_last = Some((pointer.x, pointer.y));
        }
    } else {
        state.board_drag_last = None;
    }

    if viewport.contains(pointer) {
        let (_, wheel_y) = mouse_wheel();
        state.board_scroll = (state.board_scroll - wheel_y * 52.0).clamp(0.0, max_pan_y);
    }
    state.board_pan_x = state.board_pan_x.clamp(0.0, max_pan_x);
    state.board_scroll = state.board_scroll.clamp(0.0, max_pan_y);
}

pub(super) fn draw_zoom_controls(state: &mut GameState, rect: Rect) {
    let zoom_x = if rect.w >= 560.0 {
        rect.x + 220.0
    } else {
        rect.x + rect.w - 148.0
    };
    let zoom = Rect::new(zoom_x, rect.y + 17.0, 126.0, 24.0);
    let minus = Rect::new(zoom.x, zoom.y, 26.0, zoom.h);
    let reset = Rect::new(zoom.x + 29.0, zoom.y, 66.0, zoom.h);
    let plus = Rect::new(zoom.x + 98.0, zoom.y, 26.0, zoom.h);
    draw_card(zoom, with_alpha(BG_DEEP, 0.90), with_alpha(BORDER, 0.60));
    draw_centered_text("−", minus, 16.0, TEXT);
    draw_centered_text(
        &format!("Zoom {:.0}%", state.board_zoom * 100.0),
        reset,
        9.0,
        TEXT_MUTED,
    );
    draw_centered_text("+", plus, 16.0, TEXT);
    draw_text_fit(
        "Drag to pan",
        zoom.x + zoom.w + 10.0,
        zoom.y + 16.0,
        82.0,
        10.0,
        TEXT_DIM,
    );
    if was_clicked_rect(minus) {
        state.board_zoom = (state.board_zoom - 0.15).max(0.70);
    } else if was_clicked_rect(plus) {
        state.board_zoom = (state.board_zoom + 0.15).min(1.30);
    } else if was_clicked_rect(reset) {
        state.board_zoom = 1.0;
    }
}
