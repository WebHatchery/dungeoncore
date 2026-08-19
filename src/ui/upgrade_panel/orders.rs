//! Touch-first standing-order controls embedded in a combat room inspector.

use macroquad::prelude::*;

use crate::game_state::{GameState, Room, RoomBattleOrder};
use crate::ui::theme::*;

use super::{draw_section_rule, UpgradeAction};

pub(super) fn draw_battle_orders(
    state: &GameState,
    room: &Room,
    rect: Rect,
    action: &mut UpgradeAction,
) {
    draw_section_rule(rect.x, rect.y + 12.0, rect.w, "BATTLE ORDER");
    let gap = 6.0;
    let button_w = (rect.w - gap * 2.0) / 3.0;
    let can_change = state.adventurer_parties.is_empty();
    for (index, order) in RoomBattleOrder::ALL.into_iter().enumerate() {
        let selected = room.battle_order == order;
        if draw_command_button(
            Rect::new(
                rect.x + index as f32 * (button_w + gap),
                rect.y + 20.0,
                button_w,
                27.0,
            ),
            order.label(),
            if selected {
                ButtonTone::Primary
            } else {
                ButtonTone::Ghost
            },
            can_change,
        ) {
            *action = UpgradeAction::SetBattleOrder(order);
        }
    }
    draw_text_fit(
        room.battle_order.description(),
        rect.x,
        rect.y + 64.0,
        rect.w,
        10.0,
        TEXT_MUTED,
    );
}
