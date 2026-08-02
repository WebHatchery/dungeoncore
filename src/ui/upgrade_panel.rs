use macroquad::prelude::*;
use macroquad_toolkit::input::was_clicked_rect;

use crate::data::monsters::{get_monster_template, get_species_display_name};
use crate::data::upgrades::{get_all_upgrades, UpgradeTemplate};
use crate::game_state::{GameState, Monster, Room, RoomType};

use super::theme::*;
use macroquad_toolkit::colors::with_alpha;

mod previews;
use previews::{
    monster_variant_status, room_upgrade_preview, template_trait_summary, template_variant_hint,
    upgrade_preview,
};

#[derive(Debug, Clone)]
pub enum UpgradeAction {
    None,
    Apply(String),
    Remove(crate::game_state::RoomUpgradeType),
    DismissMonster(u64),
    Close,
}

pub fn draw_upgrade_panel(
    state: &GameState,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    upgrade_scroll: &mut f32,
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
        y_cursor = draw_selected_monster(state, monster_name, inner, y_cursor);
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
            draw_upgrade_choices(
                state,
                room,
                inner,
                y_cursor + 12.0,
                upgrade_scroll,
                &mut action,
            );
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

fn draw_selected_monster(state: &GameState, monster_name: &str, bounds: Rect, y: f32) -> f32 {
    let rect = Rect::new(bounds.x, y, bounds.w, 136.0);
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
        draw_text_fit(
            &format!(
                "HP {}  ATK {}  DEF {}  Cost {} mana",
                template.hp, template.attack, template.defense, template.base_cost
            ),
            rect.x + 12.0,
            rect.y + 75.0,
            rect.w - 24.0,
            12.0,
            if state.mana >= template.base_cost {
                MANA
            } else {
                DANGER
            },
        );
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
    let defender_rows = room.monsters.len().clamp(1, MAX_DEFENDER_ROWS);
    let rect = Rect::new(
        bounds.x,
        y,
        bounds.w,
        214.0 + defender_rows as f32 * DEFENDER_ROW_H + 10.0,
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
        &format!("Tier {}", room.floor_number),
        rect.x + 56.0,
        rect.y + 48.0,
        rect.w - 68.0,
        13.0,
        TEXT_MUTED,
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

    draw_section_rule(rect.x + 12.0, rect.y + 202.0, rect.w - 24.0, "DEFENDERS");
    if let Some(monster_id) = draw_monster_progress_rows(
        state,
        room,
        Rect::new(
            rect.x + 12.0,
            rect.y + 212.0,
            rect.w - 24.0,
            defender_rows as f32 * DEFENDER_ROW_H,
        ),
        defender_scroll,
    ) {
        *action = UpgradeAction::DismissMonster(monster_id);
    }

    y + rect.h
}

fn draw_upgrade_choices(
    state: &GameState,
    room: &Room,
    bounds: Rect,
    y: f32,
    upgrade_scroll: &mut f32,
    action: &mut UpgradeAction,
) {
    let max_h = bounds.y + bounds.h - y;
    if max_h < 56.0 {
        return;
    }

    draw_section_rule(bounds.x, y + 18.0, bounds.w, "ACTIONS");
    let mut row_y = y + 36.0;

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

    // Catalog offers only types the room does not hold yet.
    let installed: Vec<_> = room.upgrades.iter().map(|u| &u.upgrade_type).collect();
    let upgrades: Vec<UpgradeTemplate> = get_all_upgrades()
        .into_iter()
        .filter(|t| {
            !installed.contains(&&crate::data::upgrades::parse_upgrade_type(&t.upgrade_type))
        })
        .collect();
    let list_rect = Rect::new(
        bounds.x,
        row_y,
        bounds.w,
        (bounds.y + bounds.h - row_y).max(0.0),
    );
    draw_upgrade_catalog(state, &upgrades, list_rect, upgrade_scroll, action);
}

fn draw_upgrade_catalog(
    state: &GameState,
    upgrades: &[UpgradeTemplate],
    rect: Rect,
    upgrade_scroll: &mut f32,
    action: &mut UpgradeAction,
) {
    if rect.h < 48.0 {
        return;
    }

    let row_h = 58.0;
    let total_h = upgrades.len() as f32 * row_h;
    let max_scroll = (total_h - rect.h).max(0.0);
    let mouse = vec2(mouse_position().0, mouse_position().1);
    if rect.contains(mouse) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.0 {
            *upgrade_scroll = (*upgrade_scroll - wheel_y * row_h).clamp(0.0, max_scroll);
        }
    }
    *upgrade_scroll = (*upgrade_scroll).clamp(0.0, max_scroll);

    draw_card(
        rect,
        Color::new(0.0, 0.0, 0.0, 0.10),
        with_alpha(BORDER, 0.18),
    );

    for (idx, upgrade) in upgrades.iter().enumerate() {
        let row_y = rect.y + idx as f32 * row_h - *upgrade_scroll + 5.0;
        if row_y < rect.y + 4.0 || row_y + row_h - 8.0 > rect.y + rect.h - 12.0 {
            continue;
        }

        let row = Rect::new(rect.x + 6.0, row_y, rect.w - 12.0, row_h - 8.0);
        if draw_upgrade_row(state, upgrade, row) {
            *action = UpgradeAction::Apply(upgrade.name.clone());
        }
    }

    if max_scroll > 0.0 {
        draw_text_fit_right(
            &format!(
                "{} / {}",
                ((*upgrade_scroll / row_h).floor() as usize + 1).min(upgrades.len()),
                upgrades.len()
            ),
            rect.x + rect.w - 8.0,
            rect.y + rect.h - 8.0,
            72.0,
            10.0,
            TEXT_DIM,
        );
    }
}

const DEFENDER_ROW_H: f32 = 46.0;
const MAX_DEFENDER_ROWS: usize = 4;

/// Vertical list of every defender in the room — one card each carrying the
/// creature's condition (health, whether it has fallen), what it hits for, its
/// element and traits, its line's variant progress, and a dismiss control.
/// Wheel-scrolls past MAX_DEFENDER_ROWS.
fn draw_monster_progress_rows(
    state: &GameState,
    room: &Room,
    rect: Rect,
    defender_scroll: &mut f32,
) -> Option<u64> {
    if room.monsters.is_empty() {
        draw_text_fit(
            "No defenders placed.",
            rect.x,
            rect.y + 14.0,
            rect.w,
            11.0,
            TEXT_DIM,
        );
        return None;
    }

    let total = room.monsters.len();
    let visible = total.min(MAX_DEFENDER_ROWS);
    let max_scroll = (total - visible) as f32;
    if total > visible && rect.contains(vec2(mouse_position().0, mouse_position().1)) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.0 {
            *defender_scroll -= wheel_y.signum();
        }
    }
    *defender_scroll = defender_scroll.clamp(0.0, max_scroll);
    let first = *defender_scroll as usize;

    let mut dismissed = None;
    let can_dismiss = state.adventurer_parties.is_empty();
    for (slot, monster) in room.monsters.iter().skip(first).take(visible).enumerate() {
        let row = Rect::new(
            rect.x,
            rect.y + slot as f32 * DEFENDER_ROW_H,
            rect.w,
            DEFENDER_ROW_H - 4.0,
        );
        if draw_defender_row(state, room, monster, row, can_dismiss) {
            dismissed = Some(monster.id);
        }
    }

    if total > visible {
        draw_text_fit_right(
            &format!("{}-{} of {} (scroll)", first + 1, first + visible, total),
            rect.x + rect.w,
            rect.y + rect.h + 10.0,
            rect.w,
            9.0,
            TEXT_DIM,
        );
    }

    dismissed
}

/// One defender's card. Returns true when its dismiss control was clicked.
fn draw_defender_row(
    state: &GameState,
    room: &Room,
    monster: &Monster,
    row: Rect,
    can_dismiss: bool,
) -> bool {
    let element = crate::data::monsters::monster_element_id(&monster.type_name);
    let accent = match &element {
        Some(id) => element_color(id),
        None => EMERALD,
    };
    // A fallen defender is a state the player must be able to act on, not a
    // greyer shade of the same row — it gets the danger tone and says so.
    let tone = if monster.alive { accent } else { DANGER };
    draw_card(row, with_alpha(tone, 0.06), with_alpha(tone, 0.22));

    let name_color = if monster.alive { TEXT } else { TEXT_DIM };
    draw_text_fit(
        &monster.type_name,
        row.x + 8.0,
        row.y + 15.0,
        row.w * 0.50,
        12.0,
        name_color,
    );

    // Offense, right-aligned and clear of the dismiss control.
    draw_text_fit_right(
        &format!(
            "ATK {}  DEF {}",
            monster.scaled_stats.attack, monster.scaled_stats.defense
        ),
        row.x + row.w - 22.0,
        row.y + 15.0,
        row.w * 0.42,
        10.0,
        if monster.alive { TEXT_MUTED } else { TEXT_DIM },
    );

    // Condition: a bar the width of the name column, so a defender crawling
    // back at half health after a respawn is visible at a glance.
    let bar = Rect::new(row.x + 8.0, row.y + 21.0, row.w * 0.50, 5.0);
    draw_rectangle(bar.x, bar.y, bar.w, bar.h, Color::new(0.0, 0.0, 0.0, 0.45));
    if monster.alive && monster.max_hp > 0 {
        let fraction = (monster.hp as f32 / monster.max_hp as f32).clamp(0.0, 1.0);
        let health_tone = if fraction > 0.6 {
            EMERALD
        } else if fraction > 0.3 {
            WARNING
        } else {
            DANGER
        };
        draw_rectangle(bar.x, bar.y, bar.w * fraction, bar.h, health_tone);
    }
    draw_text_fit(
        &if monster.alive {
            format!("{}/{} HP", monster.hp, monster.max_hp)
        } else {
            "Fallen".to_string()
        },
        row.x + 8.0,
        row.y + 38.0,
        row.w * 0.50,
        9.0,
        if monster.alive { TEXT_MUTED } else { DANGER },
    );

    // Element and traits on the right, over the line's variant progress.
    let traits = template_trait_summary(
        &monster
            .active_traits
            .iter()
            .map(|t| t.id.clone())
            .collect::<Vec<_>>(),
    );
    draw_text_fit_right(
        &format!("{} · {}", element.as_deref().unwrap_or("Neutral"), traits),
        row.x + row.w - 8.0,
        row.y + 28.0,
        row.w * 0.48,
        9.0,
        if monster.alive { accent } else { TEXT_DIM },
    );
    let (status, status_color) = monster_variant_status(state, room, monster);
    draw_text_fit_right(
        &status,
        row.x + row.w - 8.0,
        row.y + 40.0,
        row.w * 0.48,
        9.0,
        status_color,
    );

    // Dismiss control: refunds half the summon cost.
    let x_rect = Rect::new(row.x + row.w - 18.0, row.y + 4.0, 16.0, 16.0);
    let hovered = can_dismiss && x_rect.contains(vec2(mouse_position().0, mouse_position().1));
    draw_centered_text(
        "x",
        x_rect,
        12.0,
        if hovered {
            DANGER
        } else if can_dismiss {
            TEXT_MUTED
        } else {
            TEXT_DIM
        },
    );

    can_dismiss && was_clicked_rect(x_rect)
}

fn draw_upgrade_row(state: &GameState, upgrade: &UpgradeTemplate, rect: Rect) -> bool {
    let can_afford = state.mana >= upgrade.mana_cost && state.souls >= upgrade.souls_cost;
    let enabled = can_afford && state.adventurer_parties.is_empty();
    let color = upgrade_color(&upgrade.upgrade_type);
    let hovered = enabled && rect.contains(vec2(mouse_position().0, mouse_position().1));
    draw_card(
        rect,
        if hovered {
            with_alpha(color, 0.13)
        } else {
            with_alpha(color, 0.075)
        },
        with_alpha(color, if enabled { 0.30 } else { 0.12 }),
    );
    draw_text_fit(
        &upgrade.name,
        rect.x + 10.0,
        rect.y + 17.0,
        rect.w - 92.0,
        13.0,
        if enabled { TEXT } else { TEXT_DIM },
    );
    draw_text_fit(
        &upgrade_preview(upgrade),
        rect.x + 10.0,
        rect.y + 34.0,
        rect.w - 92.0,
        10.0,
        TEXT_MUTED,
    );
    let cost_text = if upgrade.souls_cost > 0 {
        format!("{}M {}S", upgrade.mana_cost, upgrade.souls_cost)
    } else {
        format!("{}M", upgrade.mana_cost)
    };
    draw_text_fit_right(
        &cost_text,
        rect.x + rect.w - 10.0,
        rect.y + 16.0,
        78.0,
        10.0,
        if can_afford { MANA } else { TEXT_DIM },
    );
    draw_text_fit_right(
        if enabled { "Apply" } else { "Locked" },
        rect.x + rect.w - 10.0,
        rect.y + 37.0,
        64.0,
        11.0,
        if enabled { EMERALD } else { TEXT_DIM },
    );

    enabled && was_clicked_rect(rect)
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
