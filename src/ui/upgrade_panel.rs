use macroquad::prelude::*;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::data::monsters::{get_monster_template, get_species_display_name};
use crate::data::upgrades::get_all_upgrades;
use crate::game_state::{GameState, Room, RoomBattleOrder, RoomType};

use super::theme::*;
use macroquad_toolkit::colors::with_alpha;

mod defenders;
mod orders;
pub(crate) mod previews;

use defenders::{draw_monster_progress_rows, DEFENDER_ROW_H, MAX_DEFENDER_ROWS};
use previews::{room_upgrade_preview, template_trait_summary, template_variant_hint};

#[derive(Debug, Clone)]
pub enum UpgradeAction {
    None,
    Apply(String),
    Remove(crate::game_state::RoomUpgradeType),
    DismissMonster(u64),
    /// Open the drawer's upgrade tab to pick something for this room.
    ArmUpgrades,
    /// Open the monster catalogue to add a defender to this room.
    ArmMonsters,
    /// Place the armed monster onto this defender — upgrading its line or
    /// evicting it, whichever the swap plan says.
    SwapMonster(u64),
    MergeMonster(u64),
    SetBattleOrder(RoomBattleOrder),
    Close,
}

pub fn draw_upgrade_panel(
    state: &GameState,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    defender_scroll: &mut f32,
) -> UpgradeAction {
    let mut action = UpgradeAction::None;
    let rect = Rect::new(x, y, w, h);
    draw_panel(rect, None, SOUL);

    let inner = Rect::new(rect.x + 14.0, rect.y + 14.0, rect.w - 28.0, rect.h - 28.0);
    draw_text_fit(
        "INSPECTOR",
        inner.x,
        inner.y + 21.0,
        inner.w - 40.0,
        18.0,
        TEXT,
    );
    if draw_close_button(Rect::new(inner.x + inner.w - 30.0, inner.y, 30.0, 26.0)) {
        return UpgradeAction::Close;
    }

    let mut y_cursor = inner.y + 46.0;

    if let Some(monster_name) = &state.selected_monster {
        // With a room open too, the armed monster's full stat block would push
        // the defender rows — the drop targets it is aimed at — off the panel.
        // Compact it to identity and price; the rows are what matter now.
        let compact = state.selected_room.is_some();
        y_cursor = draw_selected_monster(state, monster_name, inner, y_cursor, compact);
    }

    if let Some(room) = selected_room(state) {
        y_cursor = draw_selected_room(
            state,
            room,
            inner,
            y_cursor + 10.0,
            defender_scroll,
            &mut action,
        );
        if room.room_type == RoomType::Normal || room.room_type == RoomType::Boss {
            draw_upgrade_choices(state, room, inner, y_cursor + 12.0, &mut action);
        } else {
            draw_hint(
                Rect::new(inner.x, y_cursor + 12.0, inner.w, 54.0),
                match room.room_type {
                    RoomType::Entrance => "Adventurers enter here. Keep the defense deeper in.",
                    RoomType::Core => {
                        "The core must survive. Select combat rooms to build defenses."
                    }
                    RoomType::Normal | RoomType::Boss => "",
                },
                TEXT_MUTED,
            );
        }
    } else if state.selected_monster.is_none() {
        draw_hint(
            Rect::new(inner.x, y_cursor, inner.w, 72.0),
            "Select a room to inspect it, or choose a monster from the drawer.",
            TEXT_MUTED,
        );
    }

    action
}

fn selected_room(state: &GameState) -> Option<&Room> {
    let (floor_num, room_pos) = state.selected_room?;
    state
        .floors
        .iter()
        .find(|floor| floor.number == floor_num)
        .and_then(|floor| floor.rooms.iter().find(|room| room.position == room_pos))
}

fn draw_selected_monster(
    state: &GameState,
    monster_name: &str,
    bounds: Rect,
    y: f32,
    compact: bool,
) -> f32 {
    let rect = Rect::new(bounds.x, y, bounds.w, if compact { 82.0 } else { 136.0 });
    draw_card(rect, with_alpha(SOUL, 0.085), with_alpha(SOUL, 0.25));
    draw_text_fit(
        monster_name,
        rect.x + 12.0,
        rect.y + 25.0,
        rect.w - 24.0,
        18.0,
        TEXT,
    );

    if let Some(template) = get_monster_template(monster_name) {
        draw_text_fit(
            &format!(
                "Tier {} {} defender, {}",
                template.tier,
                get_species_display_name(&template.species),
                template.element.as_deref().unwrap_or("Neutral")
            ),
            rect.x + 12.0,
            rect.y + 50.0,
            rect.w - 24.0,
            12.0,
            TEXT_MUTED,
        );
        let stats_rect = Rect::new(rect.x + 12.0, rect.y + 61.0, rect.w - 24.0, 20.0);
        draw_text_fit(
            &format!(
                "HP {}  ATK {}  DEF {}  Cost {} mana",
                template.hp, template.attack, template.defense, template.base_cost
            ),
            stats_rect.x,
            rect.y + 75.0,
            stats_rect.w,
            12.0,
            if state.mana >= template.base_cost {
                MANA
            } else {
                DANGER
            },
        );
        if is_hovered_rect(stats_rect) {
            crate::ui::draw_tooltip(
                "HP = health. ATK = damage before defense. DEF reduces incoming damage. Cost is mana to summon.",
                vec2(stats_rect.x, stats_rect.y + stats_rect.h + 4.0),
            );
        }
        if !compact {
            draw_text_fit(
                &format!("Traits: {}", template_trait_summary(&template.traits)),
                rect.x + 12.0,
                rect.y + 100.0,
                rect.w - 24.0,
                11.0,
                TEXT_MUTED,
            );
            draw_text_fit(
                &template_variant_hint(state, monster_name),
                rect.x + 12.0,
                rect.y + 122.0,
                rect.w - 24.0,
                11.0,
                SOUL,
            );
        }
    } else {
        draw_text_fit(
            "Monster data unavailable",
            rect.x + 12.0,
            rect.y + 52.0,
            rect.w - 24.0,
            12.0,
            TEXT_MUTED,
        );
    }

    y + rect.h
}

fn draw_selected_room(
    state: &GameState,
    room: &Room,
    bounds: Rect,
    y: f32,
    defender_scroll: &mut f32,
    action: &mut UpgradeAction,
) -> f32 {
    // Card grows with the defender list (up to MAX_DEFENDER_ROWS visible).
    let defender_rows = room.monsters.len().min(MAX_DEFENDER_ROWS);
    let combat_room = room.room_type == RoomType::Normal || room.room_type == RoomType::Boss;
    let order_extra = if combat_room { 72.0 } else { 0.0 };
    let rect = Rect::new(
        bounds.x,
        y,
        bounds.w,
        214.0 + order_extra + defender_rows as f32 * DEFENDER_ROW_H + 10.0,
    );
    let tone = room_color(room);
    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.18),
        with_alpha(tone, 0.26),
    );

    draw_room_badge(
        Rect::new(rect.x + 12.0, rect.y + 16.0, 34.0, 34.0),
        &room.room_type,
        tone,
    );
    draw_text_fit(
        room_name(room),
        rect.x + 56.0,
        rect.y + 27.0,
        rect.w - 68.0,
        18.0,
        TEXT,
    );
    draw_text_fit(
        &format!(
            "F{} · {} · {} resonance",
            room.floor_number,
            crate::data::strata::stratum_for_floor(room.floor_number).name,
            crate::data::strata::stratum_for_floor(room.floor_number).element
        ),
        rect.x + 56.0,
        rect.y + 48.0,
        rect.w - 68.0,
        13.0,
        element_color(&crate::data::strata::stratum_for_floor(room.floor_number).element),
    );
    draw_text_fit(
        room_role(room),
        rect.x + 12.0,
        rect.y + 76.0,
        rect.w - 24.0,
        12.0,
        TEXT_MUTED,
    );

    let alive = room.monsters.iter().filter(|monster| monster.alive).count();
    let adventurers = adventurers_in_room(state, room);
    draw_section_rule(rect.x + 12.0, rect.y + 102.0, rect.w - 24.0, "ROOM STATS");
    // Combat rooms report their slot budget alongside the headcount; the
    // entrance and core hold nothing, so a capacity there would be noise.
    let held = room.monsters.len();
    let defenders = if room.room_type == RoomType::Normal || room.room_type == RoomType::Boss {
        format!(
            "{alive} alive · {held}/{} slots",
            crate::data::constants::room_capacity(room)
        )
    } else {
        format!("{alive}/{held}")
    };
    draw_stat_line(
        rect.x + 12.0,
        rect.y + 130.0,
        rect.w - 24.0,
        "Defenders",
        &defenders,
        if alive > 0 { EMERALD } else { TEXT_DIM },
    );
    let upgrade_names = if room.upgrades.is_empty() {
        "None".to_string()
    } else {
        room.upgrades
            .iter()
            .map(|u| u.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    draw_stat_line(
        rect.x + 12.0,
        rect.y + 153.0,
        rect.w - 24.0,
        "Upgrades",
        &upgrade_names,
        if room.upgrades.is_empty() {
            TEXT_DIM
        } else {
            TREASURE
        },
    );
    draw_stat_line(
        rect.x + 12.0,
        rect.y + 176.0,
        rect.w - 24.0,
        "Threat",
        &adventurers.to_string(),
        if adventurers > 0 { WARNING } else { EMERALD },
    );

    if combat_room {
        orders::draw_battle_orders(
            state,
            room,
            Rect::new(rect.x + 12.0, rect.y + 190.0, rect.w - 24.0, order_extra),
            action,
        );
    }

    if defender_rows > 0 {
        let defenders_y = rect.y + 202.0 + order_extra;
        draw_section_rule(rect.x + 12.0, defenders_y, rect.w - 24.0, "DEFENDERS");
        if let Some(row_action) = draw_monster_progress_rows(
            state,
            room,
            Rect::new(
                rect.x + 12.0,
                defenders_y + 10.0,
                rect.w - 24.0,
                defender_rows as f32 * DEFENDER_ROW_H,
            ),
            defender_scroll,
        ) {
            *action = row_action;
        }
    }

    y + rect.h
}

fn draw_upgrade_choices(
    state: &GameState,
    room: &Room,
    bounds: Rect,
    y: f32,
    action: &mut UpgradeAction,
) {
    let max_h = bounds.y + bounds.h - y;
    if max_h < 78.0 {
        return;
    }

    draw_section_rule(bounds.x, y + 18.0, bounds.w, "ACTIONS");
    let mut row_y = y + 36.0;

    if row_y + 32.0 > bounds.y + bounds.h {
        return;
    }
    if draw_command_button(
        Rect::new(bounds.x, row_y, bounds.w, 32.0),
        "Add defender",
        ButtonTone::Primary,
        state.adventurer_parties.is_empty(),
    ) {
        *action = UpgradeAction::ArmMonsters;
    }
    row_y += 40.0;

    // Adding upgrades remains the drawer's job, keeping one catalogue flow.
    let remaining = get_all_upgrades()
        .into_iter()
        .filter(|t| {
            !room.has_upgrade_type(crate::data::upgrades::parse_upgrade_type(&t.upgrade_type))
        })
        .count();
    let label = if remaining == 0 {
        "Room fully outfitted".to_string()
    } else {
        format!("Add upgrade ({remaining})")
    };
    if row_y + 32.0 > bounds.y + bounds.h {
        return;
    }
    if draw_command_button(
        Rect::new(bounds.x, row_y, bounds.w, 32.0),
        &label,
        ButtonTone::Primary,
        remaining > 0 && state.adventurer_parties.is_empty(),
    ) {
        *action = UpgradeAction::ArmUpgrades;
    }
    row_y += 40.0;

    // Installed upgrades, each with its own remove control.
    for upgrade in &room.upgrades {
        if row_y + 44.0 > bounds.y + bounds.h {
            return;
        }
        draw_hint(
            Rect::new(bounds.x, row_y, bounds.w - 92.0, 40.0),
            &format!("{}: {}", upgrade.name, room_upgrade_preview(upgrade)),
            TREASURE,
        );
        if draw_command_button(
            Rect::new(bounds.x + bounds.w - 84.0, row_y + 4.0, 84.0, 30.0),
            "Remove",
            ButtonTone::Danger,
            state.adventurer_parties.is_empty(),
        ) {
            *action = UpgradeAction::Remove(upgrade.upgrade_type.clone());
        }
        row_y += 48.0;
    }
}

fn draw_hint(rect: Rect, text: &str, color: Color) {
    draw_card(rect, with_alpha(color, 0.055), with_alpha(color, 0.18));
    let lines = macroquad_toolkit::ui::wrap_text(text, rect.w - 20.0, 11.0);
    let mut y = rect.y + 18.0;
    for line in lines.iter().take(3) {
        draw_text_fit(line, rect.x + 10.0, y, rect.w - 20.0, 11.0, color);
        y += 14.0;
    }
}

fn draw_wrapped(text: &str, rect: Rect, size: f32, color: Color) {
    let mut y = rect.y + 14.0;
    for line in macroquad_toolkit::ui::wrap_text(text, rect.w, size)
        .iter()
        .take(4)
    {
        draw_text_fit(line, rect.x, y, rect.w, size, color);
        y += size + 5.0;
    }
}

fn draw_section_rule(x: f32, y: f32, w: f32, label: &str) {
    draw_text_fit(label, x, y, w * 0.36, 11.0, TEXT_DIM);
    draw_line(x + w * 0.36, y - 4.0, x + w, y - 4.0, 1.0, BORDER_MUTED);
}

fn draw_room_badge(rect: Rect, room_type: &RoomType, color: Color) {
    draw_card(rect, with_alpha(color, 0.14), with_alpha(color, 0.42));
    draw_centered_text(room_icon_letter(room_type), rect, 17.0, color);
}

fn room_icon_letter(room_type: &RoomType) -> &'static str {
    match room_type {
        RoomType::Entrance => "E",
        RoomType::Normal => "X",
        RoomType::Boss => "B",
        RoomType::Core => "C",
    }
}

pub fn draw_close_button(rect: Rect) -> bool {
    let hovered = rect.contains(vec2(mouse_position().0, mouse_position().1));
    draw_card(
        rect,
        if hovered {
            with_alpha(SOUL, 0.12)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.05)
        },
        with_alpha(SOUL, 0.18),
    );
    draw_centered_text("X", rect, 13.0, if hovered { SOUL } else { TEXT_DIM });
    was_clicked_rect(rect)
}

fn draw_stat_line(x: f32, baseline_y: f32, w: f32, label: &str, value: &str, color: Color) {
    draw_text_fit(label, x, baseline_y, w * 0.42, 12.0, TEXT_MUTED);
    draw_text_fit_right(value, x + w, baseline_y, w * 0.56, 13.0, color);
}

fn adventurers_in_room(state: &GameState, room: &Room) -> usize {
    state
        .adventurer_parties
        .iter()
        .filter(|party| {
            party.current_floor == room.floor_number && party.current_room == room.position
        })
        .map(|party| party.members.iter().filter(|member| member.alive).count())
        .sum()
}

fn room_name(room: &Room) -> &'static str {
    match room.room_type {
        RoomType::Entrance => "Entrance",
        RoomType::Normal => "Combat Room",
        RoomType::Boss => "Boss Chamber",
        RoomType::Core => "Core",
    }
}

fn room_role(room: &Room) -> &'static str {
    match room.room_type {
        RoomType::Entrance => "Adventurers cross this threshold first.",
        RoomType::Normal => "Primary defense room.",
        RoomType::Boss => "Heavy defense and high risk.",
        RoomType::Core => "The heart of the dungeon.",
    }
}

fn room_color(room: &Room) -> Color {
    match room.room_type {
        RoomType::Entrance => EMERALD,
        RoomType::Normal => MANA,
        RoomType::Boss => WARNING,
        RoomType::Core => SOUL,
    }
}

fn upgrade_color(upgrade_type: &str) -> Color {
    match upgrade_type {
        "trap" => DANGER,
        "treasure" => TREASURE,
        "reinforcement" => EMERALD,
        "evolution" => SOUL,
        "attunement" => ARCANE,
        _ => TEXT_MUTED,
    }
}
