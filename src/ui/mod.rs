use macroquad::prelude::{get_time, Vec2};
use std::cell::Cell;
use std::cell::RefCell;

thread_local! {
    static VISUAL_TIME: Cell<Option<f32>> = const { Cell::new(None) };
}

/// Use the fixed capture clock when a scene is being photographed, otherwise
/// keep normal interactive animation on wall time.
pub(crate) fn set_visual_time(time: Option<f32>) {
    VISUAL_TIME.with(|clock| clock.set(time));
}

pub(crate) fn visual_time() -> f32 {
    VISUAL_TIME.with(|clock| clock.get().unwrap_or_else(|| get_time() as f32))
}

struct PendingTooltip {
    text: String,
    anchor: Vec2,
}

thread_local! {
    static PENDING_TOOLTIP: RefCell<Option<PendingTooltip>> = const { RefCell::new(None) };
}

/// Queue the current hover hint for the final UI layer.
///
/// Tooltips are requested while panels are being composed, but drawing them
/// immediately lets later panels cover them. Replacing the pending hint keeps
/// the most specific, last-drawn control under the pointer.
pub fn draw_tooltip(text: &str, anchor: Vec2) {
    PENDING_TOOLTIP.with(|pending| {
        *pending.borrow_mut() = Some(PendingTooltip {
            text: text.to_string(),
            anchor,
        });
    });
}

/// Draw and clear the one tooltip collected during the current frame.
pub fn draw_tooltips() {
    let tooltip = PENDING_TOOLTIP.with(|pending| pending.borrow_mut().take());
    if let Some(tooltip) = tooltip {
        macroquad_toolkit::ui::draw_tooltip(&tooltip.text, tooltip.anchor);
    }
}

/// Clear a tooltip left by a frame that ended before its normal final pass.
pub fn clear_tooltips() {
    PENDING_TOOLTIP.with(|pending| {
        pending.borrow_mut().take();
    });
}

pub mod confirmation;
pub mod core_spell_button;
pub mod core_tree;
pub mod dungeon_view;
pub mod event_toast;
pub mod game_log;
pub mod keybindings;
pub mod milestones_overlay;
pub mod overlays;
pub mod resource_panel;
pub mod save_slots;
pub mod settings;
pub mod shell;
pub mod side_drawer;
pub mod species_selector;
pub mod theme;
pub mod title_screen;
pub mod tutorial;
pub mod upgrade_panel;

pub use confirmation::*;
pub use core_spell_button::*;
pub use core_tree::*;
pub use dungeon_view::*;
pub use event_toast::*;
pub use keybindings::*;
pub use milestones_overlay::*;
pub use overlays::*;
pub use save_slots::*;
pub use settings::*;
pub use shell::*;
pub use side_drawer::*;
pub use species_selector::*;
pub use theme::*;
pub use title_screen::*;
pub use upgrade_panel::*;

// Layout constants
pub const PANEL_PADDING: f32 = 10.0;
pub const SIDEBAR_WIDTH: f32 = 250.0;
pub const TOP_BAR_HEIGHT: f32 = 60.0;
pub const LOG_HEIGHT: f32 = 150.0;
