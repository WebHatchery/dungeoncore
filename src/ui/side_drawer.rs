//! The left drawer: a collapsible tab rail plus the active tab's content.
//! Each tab's rendering lives in its own submodule ([`build_tab`],
//! [`monster_tab`], [`traps_tab`], [`evolution_tab`], [`heroes_tab`]); this
//! root owns the public types, the rail, and dispatch.

use macroquad::prelude::*;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::game_state::GameState;

use super::theme::*;

mod build_tab;
mod evolution_tab;
mod heroes_tab;
mod monster_tab;
mod traps_tab;

use build_tab::draw_build_tab;
use evolution_tab::draw_evolution_tab;
use heroes_tab::draw_heroes_tab;
use macroquad_toolkit::colors::with_alpha;
use monster_tab::draw_monster_tab;
use traps_tab::draw_traps_tab;

pub const DRAWER_OPEN_WIDTH: f32 = 274.0;
pub const DRAWER_COLLAPSED_WIDTH: f32 = 64.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerTab {
    Build,
    Monsters,
    Traps,
    Evolution,
    Heroes,
}

/// Sections within the Traps tab, so each upgrade family gets its own list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeSection {
    Traps,
    Loot,
    Buffs,
    Shrines,
}

impl UpgradeSection {
    fn label(self) -> &'static str {
        match self {
            UpgradeSection::Traps => "Traps",
            UpgradeSection::Loot => "Loot",
            UpgradeSection::Buffs => "Buffs",
            UpgradeSection::Shrines => "Shrines",
        }
    }

    /// Which upgrade-template types this section lists.
    fn matches(self, upgrade_type: &str) -> bool {
        match self {
            UpgradeSection::Traps => upgrade_type == "trap",
            UpgradeSection::Loot => upgrade_type == "treasure",
            UpgradeSection::Buffs => upgrade_type == "reinforcement" || upgrade_type == "evolution",
            UpgradeSection::Shrines => upgrade_type == "attunement",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawerAction {
    None,
    BuildRoom,
    SelectMonster(String),
    SelectUpgrade(String),
    UnlockSpecies(String),
    OpenCorePowers,
    ChannelGold,
    ResetGame,
    /// Open a hero's journal page, or close it again.
    OpenHero(u64),
    CloseHero,
}

/// What the Build tab wants to do this frame.
pub enum BuildTabAction {
    None,
    Build,
    OpenCorePowers,
    ChannelGold,
}

pub fn draw_side_drawer(
    state: &GameState,
    rect: Rect,
    active_tab: &mut DrawerTab,
    open: &mut bool,
    upgrade_section: &mut UpgradeSection,
    heroes_scroll: &mut f32,
) -> DrawerAction {
    let mut action = DrawerAction::None;
    draw_panel(rect, None, ARCANE);

    let rail_w = DRAWER_COLLAPSED_WIDTH.min(rect.w);
    if draw_tab_rail(state, rect, rail_w, active_tab, open) {
        action = DrawerAction::ResetGame;
    }

    if !*open || rect.w <= rail_w + 24.0 {
        return action;
    }

    let content = Rect::new(
        rect.x + rail_w + 12.0,
        rect.y + 16.0,
        rect.w - rail_w - 24.0,
        rect.h - 32.0,
    );

    match active_tab {
        DrawerTab::Build => match draw_build_tab(state, content) {
            BuildTabAction::Build => action = DrawerAction::BuildRoom,
            BuildTabAction::OpenCorePowers => action = DrawerAction::OpenCorePowers,
            BuildTabAction::ChannelGold => action = DrawerAction::ChannelGold,
            BuildTabAction::None => {}
        },
        DrawerTab::Monsters => {
            if let Some(monster) = draw_monster_tab(state, content) {
                action = DrawerAction::SelectMonster(monster);
            }
        }
        DrawerTab::Traps => {
            if let Some(upgrade) = draw_traps_tab(state, content, upgrade_section) {
                action = DrawerAction::SelectUpgrade(upgrade);
            }
        }
        DrawerTab::Evolution => {
            action = draw_evolution_tab(state, content);
        }
        DrawerTab::Heroes => match draw_heroes_tab(state, content, heroes_scroll) {
            heroes_tab::HeroesTabAction::Open(id) => action = DrawerAction::OpenHero(id),
            heroes_tab::HeroesTabAction::Close => action = DrawerAction::CloseHero,
            heroes_tab::HeroesTabAction::None => {}
        },
    }

    action
}

fn draw_tab_rail(
    state: &GameState,
    rect: Rect,
    rail_w: f32,
    active_tab: &mut DrawerTab,
    open: &mut bool,
) -> bool {
    let toggle = Rect::new(rect.x + 9.0, rect.y + 12.0, rail_w - 18.0, 34.0);
    if draw_small_tab(toggle, if *open { "<" } else { ">" }, ARCANE, true) {
        *open = !*open;
    }

    let mut y = rect.y + 54.0;
    for (tab, icon, label, color) in [
        (DrawerTab::Monsters, "M", "MONSTERS", SOUL),
        (DrawerTab::Traps, "T", "TRAPS", DANGER),
        (DrawerTab::Build, "B", "BUILD", TREASURE),
        (DrawerTab::Evolution, "V", "VARIANTS", MANA),
        (DrawerTab::Heroes, "H", "HEROES", WARNING),
    ] {
        let tab_rect = Rect::new(rect.x + 7.0, y, rail_w - 14.0, 54.0);
        if draw_rail_tab(tab_rect, icon, label, color, *active_tab == tab) {
            *active_tab = tab;
            *open = true;
        }
        y += 60.0;
    }

    let chip_rect = Rect::new(rect.x + 9.0, rect.y + rect.h - 88.0, rail_w - 18.0, 34.0);
    let color = if state.adventurer_parties.is_empty() {
        EMERALD
    } else {
        WARNING
    };
    draw_card(chip_rect, with_alpha(color, 0.09), with_alpha(color, 0.26));
    draw_centered_text(
        if state.adventurer_parties.is_empty() {
            "Safe"
        } else {
            "Alert"
        },
        chip_rect,
        10.0,
        color,
    );

    // Reset control lives quietly at the bottom of the rail.
    let reset_rect = Rect::new(rect.x + 9.0, rect.y + rect.h - 46.0, rail_w - 18.0, 34.0);
    draw_small_tab(reset_rect, "RESET", DANGER, false)
}

fn draw_section_title(rect: Rect, title: &str, subtitle: &str) {
    draw_text_fit(
        title,
        rect.x + 24.0,
        rect.y + 28.0,
        rect.w - 24.0,
        20.0,
        TEXT,
    );
    draw_poly_lines(rect.x + 10.0, rect.y + 22.0, 6, 8.0, 30.0, 1.5, SOUL);
    draw_text_fit(subtitle, rect.x, rect.y + 54.0, rect.w, 12.0, TEXT_MUTED);
}

fn draw_small_tab(rect: Rect, text: &str, color: Color, active: bool) -> bool {
    let hovered = is_hovered_rect(rect);
    draw_card(
        rect,
        if active {
            with_alpha(color, 0.16)
        } else if hovered {
            with_alpha(color, 0.10)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.10)
        },
        with_alpha(color, if active { 0.42 } else { 0.18 }),
    );
    draw_centered_text(text, rect, 10.0, if active { color } else { TEXT_MUTED });
    was_clicked_rect(rect)
}

fn draw_rail_tab(rect: Rect, icon: &str, label: &str, color: Color, active: bool) -> bool {
    let hovered = is_hovered_rect(rect);
    draw_card(
        rect,
        if active {
            with_alpha(color, 0.14)
        } else if hovered {
            with_alpha(color, 0.08)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.10)
        },
        with_alpha(color, if active { 0.48 } else { 0.16 }),
    );
    draw_centered_text(
        icon,
        Rect::new(rect.x, rect.y + 10.0, rect.w, 24.0),
        22.0,
        if active { color } else { TEXT_DIM },
    );
    draw_centered_text(
        label,
        Rect::new(rect.x + 2.0, rect.y + 43.0, rect.w - 4.0, 20.0),
        8.0,
        if active { TEXT } else { TEXT_MUTED },
    );
    was_clicked_rect(rect)
}
