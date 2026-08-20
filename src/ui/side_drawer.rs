//! The left drawer: a collapsible tab rail plus the active tab's content.
//! Each tab's rendering lives in its own submodule ([`build_tab`],
//! [`monster_tab`], [`traps_tab`], [`evolution_tab`], [`heroes_tab`]); this
//! root owns the public types, the rail, and dispatch.

use macroquad::prelude::*;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};

use crate::game_state::GameState;

use super::theme::*;

mod build_tab;
mod depth_tab;
mod evolution_tab;
mod heroes_tab;
mod monster_tab;
mod traps_tab;

use build_tab::draw_build_tab;
use depth_tab::draw_depth_tab;
use evolution_tab::draw_evolution_tab;
use heroes_tab::draw_heroes_tab;
use macroquad_toolkit::colors::with_alpha;
use monster_tab::draw_monster_tab;
use traps_tab::draw_traps_tab;

pub const DRAWER_OPEN_WIDTH: f32 = 326.0;
pub const DRAWER_COLLAPSED_WIDTH: f32 = 72.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerTab {
    Build,
    Monsters,
    Traps,
    Evolution,
    Heroes,
    Depth,
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
    BranchRoom,
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
    Branch,
    OpenCorePowers,
    ChannelGold,
    Reset,
}

pub fn draw_side_drawer(
    state: &GameState,
    rect: Rect,
    active_tab: &mut DrawerTab,
    open: &mut bool,
    upgrade_section: &mut UpgradeSection,
    monster_scroll: &mut f32,
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
        rect.x + rail_w + 14.0,
        rect.y + 16.0,
        rect.w - rail_w - 28.0,
        rect.h - 32.0,
    );

    match active_tab {
        DrawerTab::Build => match draw_build_tab(state, content) {
            BuildTabAction::Build => action = DrawerAction::BuildRoom,
            BuildTabAction::Branch => action = DrawerAction::BranchRoom,
            BuildTabAction::OpenCorePowers => action = DrawerAction::OpenCorePowers,
            BuildTabAction::ChannelGold => action = DrawerAction::ChannelGold,
            BuildTabAction::Reset => action = DrawerAction::ResetGame,
            BuildTabAction::None => {}
        },
        DrawerTab::Monsters => {
            if let Some(monster) = draw_monster_tab(state, content, monster_scroll) {
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
        DrawerTab::Depth => draw_depth_tab(state, content),
    }

    action
}

fn draw_tab_rail(
    _state: &GameState,
    rect: Rect,
    rail_w: f32,
    active_tab: &mut DrawerTab,
    open: &mut bool,
) -> bool {
    let toggle = Rect::new(rect.x + 8.0, rect.y + 10.0, rail_w - 16.0, 42.0);
    if draw_small_tab(toggle, if *open { "CLOSE" } else { "MENU" }, ARCANE, true) {
        *open = !*open;
    }
    if is_hovered_rect(toggle) {
        crate::ui::draw_tooltip(
            if *open {
                "Collapse the drawer to give the dungeon board more room."
            } else {
                "Expand the drawer to build rooms, place defenders, and inspect progress."
            },
            vec2(toggle.x + toggle.w, toggle.y + toggle.h),
        );
    }

    let mut y = rect.y + 60.0;
    for (tab, icon, label, color) in [
        (DrawerTab::Monsters, "M", "MONSTERS", SOUL),
        (DrawerTab::Traps, "T", "OUTFITS", DANGER),
        (DrawerTab::Build, "B", "BUILD", TREASURE),
        (DrawerTab::Evolution, "V", "VARIANTS", MANA),
        (DrawerTab::Heroes, "H", "HEROES", WARNING),
        (DrawerTab::Depth, "D", "DEPTH", ARCANE),
    ] {
        let tab_rect = Rect::new(rect.x + 6.0, y, rail_w - 12.0, 60.0);
        if draw_rail_tab(tab_rect, icon, label, color, *active_tab == tab) {
            *active_tab = tab;
            *open = true;
        }
        if is_hovered_rect(tab_rect) {
            let help = match tab {
                DrawerTab::Monsters => "Choose defenders to place in a selected room.",
                DrawerTab::Traps => "Choose traps, treasure, buffs, and attunements to install.",
                DrawerTab::Build => "Extend the dungeon, branch routes, and spend permanent souls.",
                DrawerTab::Evolution => "Track shared monster-line XP and unlocked variants.",
                DrawerTab::Heroes => {
                    "Review adventurer records, rival bounties, and dungeon reputation."
                }
                DrawerTab::Depth => {
                    "Read the current depth chapter, recovered apex relics, and the party's live doctrine."
                }
            };
            crate::ui::draw_tooltip(help, vec2(tab_rect.x + tab_rect.w, tab_rect.y));
        }
        y += 66.0;
    }

    false
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
        Rect::new(rect.x, rect.y + 4.0, rect.w, 31.0),
        21.0,
        if active { color } else { TEXT_MUTED },
    );
    draw_centered_text(
        label,
        Rect::new(rect.x + 2.0, rect.y + 37.0, rect.w - 4.0, 15.0),
        8.0,
        if active { TEXT } else { TEXT_DIM },
    );
    was_clicked_rect(rect)
}
