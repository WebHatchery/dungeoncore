//! Dungeon Core - A dungeon management game
#![allow(dead_code)]
//!
//! Migrated from React/TypeScript + PHP to Rust using macroquad.

mod capture_scenes;
mod data;
mod game_state;
mod persistence;
mod simulation;
mod ui;

use macroquad::miniquad::window::quit;
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::capture;

use game_state::GameState;
use ui::*;

/// Env-var prefix for the screenshot capture harness (DUNGEON_CORE_CAPTURE_*).
const CAPTURE_PREFIX: &str = "DUNGEON_CORE";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppScreen {
    Title,
    SaveSlots,
    ConfirmSlotOverwrite,
    NewGameSetup,
    Settings,
    Playing,
}

// Helper to check if any modal is open
fn is_modal_open(state: &GameState) -> bool {
    // Add other modals here if needed
    state.unlocked_species.is_empty()
}

fn window_conf() -> Conf {
    capture::capture_window_conf(CAPTURE_PREFIX, "Dungeon Core", 1280, 720)
}

fn create_new_game(difficulty: data::difficulty::Difficulty, default_speed: i32) -> GameState {
    let mut state = GameState::new();
    state.difficulty = difficulty;
    // Scale the starting core so easier runs begin sturdier and harder ones
    // more fragile.
    let mult = difficulty.profile().core_hp_mult;
    state.core_max_hp = ((state.core_max_hp as f32 * mult).round() as i32).max(1);
    state.core_hp = state.core_max_hp;
    state.speed = default_speed.clamp(1, 4);
    state
}

fn reset_timers(last_time_advance: &mut f64, last_adventure_tick: &mut f64, last_save: &mut f64) {
    let now = get_time();
    *last_time_advance = now;
    *last_adventure_tick = now;
    *last_save = now;
}

#[macroquad::main(window_conf)]
async fn main() {
    // Install before loading assets or saves so even startup failures leave a
    // report beside the player's untouched save file on native builds.
    macroquad_toolkit::crash::install_crash_log("dungeon_core");

    let mut assets = AssetManager::new();
    let _ = assets.load_asset_pack("assets.zip").await;
    if let Err(e) = assets
        .load_texture_with_filter(
            TITLE_BACKGROUND_KEY,
            TITLE_BACKGROUND_PATH,
            FilterMode::Linear,
        )
        .await
    {
        eprintln!("Failed to load title background: {}", e);
    }
    if let Err(e) = assets
        .load_texture_with_filter(UNIT_SHEET_KEY, UNIT_SHEET_PATH, FilterMode::Nearest)
        .await
    {
        // The board retains its icon fallback when a loose or packed art asset
        // is absent, which is especially useful for partial browser deploys.
        eprintln!("Failed to load unit sprite sheet; using icons: {}", e);
    }
    let sprites = DungeonSprites::from_assets(&assets);

    // Screenshot capture harness: when DUNGEON_CORE_CAPTURE_PATH is set, seed a
    // scene, render a fixed number of frames, write a PNG, and exit. No input,
    // no simulation drift, and the player's save file is left untouched.
    if let Some(config) = capture::CaptureConfig::from_env(CAPTURE_PREFIX) {
        let mut cap_state = create_new_game(data::difficulty::Difficulty::default(), 1);
        capture_scenes::seed_capture_scene(&mut cap_state, &config.scene);
        // Most scenes show the Monsters tab; a couple open the tab they exist
        // to show off.
        let mut drawer_tab = match config.scene.as_str() {
            "build" => DrawerTab::Build,
            "variants" => DrawerTab::Evolution,
            "traps" => DrawerTab::Traps,
            "journal" => DrawerTab::Heroes,
            _ => DrawerTab::Monsters,
        };
        let mut upgrade_section = UpgradeSection::Traps;
        let mut drawer_open = true;
        let mut species_scroll = 0.0;
        let mut defender_scroll = 0.0;
        let mut heroes_scroll = 0.0;
        let mut show_codex = false;
        let mut codex_scroll = 0.0;
        // The `coretree` scene boots straight into the core-power tree overlay.
        let mut show_core_tree = config.scene == "coretree";
        // The `goals` scene boots straight into the milestone overlay.
        let mut show_milestones = config.scene == "goals";
        let mut milestones_scroll = 0.0;
        let mut t0 = get_time();
        let mut t1 = t0;
        let mut t2 = t0;
        let strip = capture::filmstrip::StripConfig::from_env(CAPTURE_PREFIX);
        if let Some(strip) = strip {
            capture::filmstrip::run_filmstrip(&config, &strip, |_dt| {
                render_playing_frame(
                    &mut cap_state,
                    &mut drawer_tab,
                    &mut upgrade_section,
                    &mut drawer_open,
                    &mut species_scroll,
                    &mut defender_scroll,
                    &mut heroes_scroll,
                    &mut show_codex,
                    &mut codex_scroll,
                    &mut show_core_tree,
                    &mut show_milestones,
                    &mut milestones_scroll,
                    &mut t0,
                    &mut t1,
                    &mut t2,
                    true,
                    30.0,
                    persistence::DEFAULT_SLOT,
                    &sprites,
                );
            })
            .await;
        } else {
            capture::run_capture(&config, |_dt| {
                render_playing_frame(
                    &mut cap_state,
                    &mut drawer_tab,
                    &mut upgrade_section,
                    &mut drawer_open,
                    &mut species_scroll,
                    &mut defender_scroll,
                    &mut heroes_scroll,
                    &mut show_codex,
                    &mut codex_scroll,
                    &mut show_core_tree,
                    &mut show_milestones,
                    &mut milestones_scroll,
                    &mut t0,
                    &mut t1,
                    &mut t2,
                    false,
                    30.0,
                    persistence::DEFAULT_SLOT,
                    &sprites,
                );
            })
            .await;
        }
        return;
    }

    let legacy_notice = match persistence::migrate_legacy_save() {
        Ok(true) => Some("Legacy save moved to Slot 1.".to_string()),
        Ok(false) => None,
        Err(error) => Some(format!("Legacy save was left untouched: {error}")),
    };
    let mut state = create_new_game(data::difficulty::Difficulty::default(), 1);
    let mut active_slot = persistence::DEFAULT_SLOT;
    let mut screen = AppScreen::Title;
    let mut title_notice: Option<String> = legacy_notice;
    let mut settings = macroquad_toolkit::settings::GameSettings::load("dungeon_core");
    settings.sanitize();
    settings.apply_display();

    // Timing variables
    let mut last_time_advance = get_time();
    let mut last_adventure_tick = get_time();
    let mut last_save = get_time();
    let mut drawer_tab = DrawerTab::Monsters;
    let mut upgrade_section = UpgradeSection::Traps;
    let mut drawer_open = true;
    let mut species_scroll = 0.0;
    let mut defender_scroll = 0.0;
    let mut heroes_scroll = 0.0;
    let mut show_codex = false;
    let mut codex_scroll = 0.0;
    let mut show_core_tree = false;
    let mut show_milestones = false;
    let mut milestones_scroll = 0.0;

    loop {
        match screen {
            AppScreen::Title => {
                match draw_title_screen(
                    &assets,
                    persistence::SAVE_SLOTS.iter().any(|slot| {
                        !matches!(persistence::slot_state(slot), persistence::SlotState::Empty)
                    }),
                    title_notice.as_deref(),
                ) {
                    TitleAction::NewGame | TitleAction::LoadGame => {
                        title_notice = None;
                        screen = AppScreen::SaveSlots;
                    }
                    TitleAction::Settings => {
                        title_notice = None;
                        screen = AppScreen::Settings;
                    }
                    TitleAction::Exit => {
                        quit();
                        return;
                    }
                    TitleAction::None => {}
                }
                next_frame().await;
                continue;
            }
            AppScreen::SaveSlots => {
                let states = std::array::from_fn(|index| {
                    persistence::slot_state(persistence::SAVE_SLOTS[index])
                });
                match draw_save_slots_screen(&assets, &states, title_notice.as_deref()) {
                    SaveSlotAction::Load(slot) => match persistence::load_game(slot) {
                        Ok(loaded_state) => {
                            active_slot = slot;
                            state = loaded_state;
                            reset_timers(&mut last_time_advance, &mut last_adventure_tick, &mut last_save);
                            title_notice = None;
                            screen = AppScreen::Playing;
                        }
                        Err(error) => title_notice = Some(format!("Could not load {slot}: {error}")),
                    },
                    SaveSlotAction::New(slot) => {
                        active_slot = slot;
                        if matches!(persistence::slot_state(slot), persistence::SlotState::Ready { .. }) {
                            screen = AppScreen::ConfirmSlotOverwrite;
                        } else {
                            screen = AppScreen::NewGameSetup;
                        }
                    }
                    SaveSlotAction::Recover(slot) => match persistence::recover_corrupt_slot(slot) {
                        Ok(()) => title_notice = Some(format!("{slot} was set aside as a corrupt save. You can start a new run there.")),
                        Err(error) => title_notice = Some(format!("Could not preserve {slot}: {error}")),
                    },
                    SaveSlotAction::Back => {
                        title_notice = None;
                        screen = AppScreen::Title;
                    }
                    SaveSlotAction::None => {}
                }
                next_frame().await;
                continue;
            }
            AppScreen::ConfirmSlotOverwrite => {
                if let Some(confirmed) = draw_slot_overwrite_confirmation(&assets, active_slot) {
                    screen = if confirmed {
                        AppScreen::NewGameSetup
                    } else {
                        AppScreen::SaveSlots
                    };
                }
                next_frame().await;
                continue;
            }
            AppScreen::NewGameSetup => {
                match draw_new_game_setup(&assets, title_notice.as_deref()) {
                    NewGameSetupAction::Start(difficulty) => {
                        state = create_new_game(difficulty, settings.default_speed);
                        if let Err(e) = persistence::save_game(active_slot, &state) {
                            eprintln!("Failed to save new game: {}", e);
                        }
                        reset_timers(
                            &mut last_time_advance,
                            &mut last_adventure_tick,
                            &mut last_save,
                        );
                        title_notice = None;
                        screen = AppScreen::Playing;
                    }
                    NewGameSetupAction::Back => {
                        title_notice = None;
                        screen = AppScreen::Title;
                    }
                    NewGameSetupAction::None => {}
                }
                next_frame().await;
                continue;
            }
            AppScreen::Settings => {
                let action =
                    draw_title_settings_screen(&assets, &settings, title_notice.as_deref());
                let (notice, back) = apply_title_settings_action(&mut settings, action);
                if back {
                    title_notice = None;
                    screen = AppScreen::Title;
                } else if let Some(notice) = notice {
                    title_notice = Some(notice);
                }
                next_frame().await;
                continue;
            }
            AppScreen::Playing => {}
        }

        render_playing_frame(
            &mut state,
            &mut drawer_tab,
            &mut upgrade_section,
            &mut drawer_open,
            &mut species_scroll,
            &mut defender_scroll,
            &mut heroes_scroll,
            &mut show_codex,
            &mut codex_scroll,
            &mut show_core_tree,
            &mut show_milestones,
            &mut milestones_scroll,
            &mut last_time_advance,
            &mut last_adventure_tick,
            &mut last_save,
            true,
            settings.autosave_interval as f64,
            active_slot,
            &sprites,
        );

        next_frame().await;
    }
}

/// Render (and, when `simulate` is true, step) one frame of the Playing screen.
/// Shared by the interactive loop and the screenshot capture harness; the
/// capture path passes `simulate = false` so the seeded scene stays frozen and
/// the save file is never touched.
#[allow(clippy::too_many_arguments)]
fn render_playing_frame(
    state: &mut GameState,
    drawer_tab: &mut DrawerTab,
    upgrade_section: &mut UpgradeSection,
    drawer_open: &mut bool,
    species_scroll: &mut f32,
    defender_scroll: &mut f32,
    heroes_scroll: &mut f32,
    show_codex: &mut bool,
    codex_scroll: &mut f32,
    show_core_tree: &mut bool,
    show_milestones: &mut bool,
    milestones_scroll: &mut f32,
    last_time_advance: &mut f64,
    last_adventure_tick: &mut f64,
    last_save: &mut f64,
    simulate: bool,
    autosave_interval: f64,
    save_slot: &str,
    sprites: &DungeonSprites,
) {
    let now = get_time();
    let sw = screen_width();
    let sh = screen_height();
    draw_game_background(sw, sh);

    // A pause is a first-class simulation state: the UI remains available for
    // inspection, but no time, raids, transient effects, or cooldowns advance.
    if simulate && is_key_pressed(KeyCode::Space) {
        state.paused = !state.paused;
        if !state.paused {
            reset_timers(last_time_advance, last_adventure_tick, last_save);
        }
    }

    if simulation_active(simulate, state.paused) {
        // Age transient combat effects and party-travel animations each frame,
        // and recharge the Core Smite lever in real time.
        state.decay_effects(get_frame_time());
        for party in &mut state.adventurer_parties {
            party.move_anim.tick(get_frame_time());
        }
        state.core_smite_cooldown.tick(get_frame_time());

        // === Time-based Updates ===

        // Advance game time based on speed
        let time_interval = 5.0 / state.speed as f64;
        if now - *last_time_advance > time_interval {
            simulation::advance_time(state);
            *last_time_advance = now;
        }

        // Process adventurer system
        if now - *last_adventure_tick > 2.0 {
            simulation::spawn_party(state);
            simulation::process_parties(state);
            *last_adventure_tick = now;
        }

        // Auto-save every 30 seconds
        if now - *last_save > autosave_interval {
            if let Err(e) = persistence::save_game(save_slot, state) {
                eprintln!("Failed to save: {}", e);
            }
            *last_save = now;
        }
    }

    // Game over: the core has fallen. Offer a fresh dungeon.
    if state.game_over {
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.82));
        if draw_game_over_overlay(state, sw, sh) {
            // A fresh dungeon keeps the fallen run's chosen difficulty.
            *state = create_new_game(state.difficulty, 1);
            let _ = persistence::save_game(save_slot, state);
            reset_timers(last_time_advance, last_adventure_tick, last_save);
        }
        return;
    }

    // Modal overlay: Species Selection (Prioritize over everything else)
    if state.unlocked_species.is_empty() {
        let modal_w = 460.0;
        let modal_h = 540.0;
        let modal_x = (sw - modal_w) / 2.0;
        let modal_y = (sh - modal_h) / 2.0;

        // Draw a semi-transparent background to dim the game
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.7));

        if let Some(selected_species_id) =
            draw_species_selector(state, modal_x, modal_y, modal_w, modal_h, species_scroll)
        {
            // Unlock the selected species
            if let Err(e) = simulation::unlock_species(state, &selected_species_id) {
                eprintln!("Error unlocking species: {}", e);
            } else {
                // Species unlocked successfully - player can now place monsters manually
                state.add_log(crate::game_state::LogEntry::system(format!(
                     "Chosen starter race: {}. Build rooms and place its units to defend your dungeon.",
                     crate::data::monsters::get_species_display_name(&selected_species_id)
                 )));
            }
        }

        return;
    }

    let hud_rect = Rect::new(
        OUTER_MARGIN,
        OUTER_MARGIN,
        sw - OUTER_MARGIN * 2.0,
        HUD_HEIGHT,
    );
    match draw_top_hud(state, hud_rect) {
        ControlAction::TogglePause => {
            state.paused = !state.paused;
            if !state.paused {
                reset_timers(last_time_advance, last_adventure_tick, last_save);
            }
        }
        ControlAction::ToggleSpeed => simulation::toggle_speed(state),
        ControlAction::ToggleDungeon => simulation::toggle_dungeon_status(state),
        _ => {}
    }

    let log_rect = Rect::new(
        OUTER_MARGIN,
        sh - OUTER_MARGIN - LOG_BAR_HEIGHT,
        sw - OUTER_MARGIN * 2.0,
        LOG_BAR_HEIGHT,
    );

    let body_top = hud_rect.y + hud_rect.h + PANEL_GAP;
    let body_bottom = log_rect.y - PANEL_GAP;
    let body_h = (body_bottom - body_top).max(220.0);

    let drawer_w = if *drawer_open {
        SIDE_PANEL_WIDTH.min((sw * 0.22).clamp(250.0, DRAWER_OPEN_WIDTH))
    } else {
        DRAWER_COLLAPSED_WIDTH
    };
    let drawer_rect = Rect::new(OUTER_MARGIN, body_top, drawer_w, body_h);
    match draw_side_drawer(
        state,
        drawer_rect,
        drawer_tab,
        drawer_open,
        upgrade_section,
        heroes_scroll,
    ) {
        DrawerAction::SelectMonster(monster) => {
            if state.selected_monster.as_ref() == Some(&monster) {
                state.selected_monster = None;
            } else {
                state.selected_room = None;
                state.selected_upgrade = None;
                state.selected_monster = Some(monster);
            }
        }
        DrawerAction::SelectUpgrade(upgrade) => {
            if state.selected_upgrade.as_ref() == Some(&upgrade) {
                state.selected_upgrade = None;
            } else {
                state.selected_room = None;
                state.selected_monster = None;
                state.selected_upgrade = Some(upgrade);
            }
        }
        DrawerAction::BuildRoom => {
            if let Err(e) = simulation::add_room(state, None) {
                state.add_log(game_state::LogEntry::system(e));
            }
        }
        DrawerAction::BranchRoom => {
            if let Some((floor, room)) = state.selected_room {
                if let Err(e) = simulation::branch_from(state, floor, room) {
                    state.add_log(game_state::LogEntry::system(e));
                }
            }
        }
        DrawerAction::UnlockSpecies(species) => {
            if let Err(e) = simulation::unlock_species(state, &species) {
                state.add_log(game_state::LogEntry::system(e));
            }
        }
        DrawerAction::OpenCorePowers => *show_core_tree = true,
        DrawerAction::ChannelGold => {
            if let Err(e) = simulation::economy::channel_gold_to_mana(state) {
                state.add_log(game_state::LogEntry::system(e));
            }
        }
        DrawerAction::ResetGame => {
            state.pending_confirmation = Some(game_state::PendingConfirmation::ResetRun);
        }
        DrawerAction::OpenHero(id) => state.selected_hero = Some(id),
        DrawerAction::CloseHero => state.selected_hero = None,
        DrawerAction::None => {}
    }

    let has_inspector = state.selected_room.is_some() || state.selected_monster.is_some();
    let right_panel_w = if has_inspector {
        (sw * 0.21).clamp(270.0, 330.0)
    } else {
        0.0
    };
    let right_gap = if right_panel_w > 0.0 { PANEL_GAP } else { 0.0 };
    let dungeon_x = drawer_rect.x + drawer_rect.w + PANEL_GAP;
    let dungeon_w = sw - dungeon_x - right_panel_w - right_gap - OUTER_MARGIN;
    let dungeon_h = body_h;
    let dungeon_rect = Rect::new(
        dungeon_x,
        body_top,
        dungeon_w.max(320.0),
        dungeon_h.max(220.0),
    );

    match draw_dungeon_board(state, dungeon_rect, sprites) {
        DungeonAction::RoomSelected(floor_num, room_pos) => {
            if let Some(ref monster_name) = state.selected_monster.clone() {
                // Selection stays armed on success so more can be placed with
                // further clicks; it clears on failure (no mana, bad room) or
                // by re-clicking the drawer entry.
                if let Err(e) = simulation::place_monster(state, floor_num, room_pos, monster_name)
                {
                    state.add_log(game_state::LogEntry::system(e));
                    state.selected_monster = None;
                }
            } else if let Some(ref upgrade_name) = state.selected_upgrade.clone() {
                if let Err(e) = simulation::apply_upgrade(state, floor_num, room_pos, upgrade_name)
                {
                    state.add_log(game_state::LogEntry::system(e));
                    state.selected_upgrade = None;
                }
            } else if state.selected_room == Some((floor_num, room_pos)) {
                state.selected_room = None;
                *defender_scroll = 0.0;
            } else {
                state.selected_room = Some((floor_num, room_pos));
                *defender_scroll = 0.0;
            }
        }
        DungeonAction::BuildRoom => {
            if let Err(e) = simulation::add_room(state, None) {
                state.add_log(game_state::LogEntry::system(e));
            }
        }
        DungeonAction::None => {}
    }

    if state.paused {
        if draw_pause_overlay(dungeon_rect) {
            state.paused = false;
            reset_timers(last_time_advance, last_adventure_tick, last_save);
        }
    }

    // Inspector panel (room, monster, and upgrade context)
    if has_inspector {
        let upgrade_panel_w = right_panel_w;
        let upgrade_panel_h = dungeon_h;
        let upgrade_panel_x = sw - upgrade_panel_w - OUTER_MARGIN;
        let upgrade_panel_y = body_top;

        let upgrade_action = draw_upgrade_panel(
            state,
            upgrade_panel_x,
            upgrade_panel_y,
            upgrade_panel_w,
            upgrade_panel_h,
            defender_scroll,
        );
        match upgrade_action {
            UpgradeAction::Apply(name) => {
                if let Some((floor, pos)) = state.selected_room {
                    if let Err(e) = simulation::apply_upgrade(state, floor, pos, &name) {
                        state.add_log(game_state::LogEntry::system(e));
                    }
                }
            }
            UpgradeAction::Remove(upgrade_type) => {
                if let Some((floor, pos)) = state.selected_room {
                    if let Err(e) = simulation::remove_upgrade(state, floor, pos, upgrade_type) {
                        state.add_log(game_state::LogEntry::system(e));
                    }
                }
            }
            UpgradeAction::DismissMonster(monster_id) => {
                if let Some((floor, pos)) = state.selected_room {
                    state.pending_confirmation =
                        Some(game_state::PendingConfirmation::DismissMonster {
                            floor,
                            room: pos,
                            monster_id,
                        });
                }
            }
            UpgradeAction::ArmUpgrades => {
                // Same flow as placing a monster: pick from the drawer, then
                // click the rooms that light up.
                *drawer_tab = DrawerTab::Traps;
                *drawer_open = true;
            }
            UpgradeAction::SwapMonster(monster_id) => {
                // The armed monster goes onto an occupied slot: its own line
                // grows, anything else evicts. Selection clears either way —
                // the slot it was aimed at is no longer what it was.
                if let (Some((floor, pos)), Some(armed)) =
                    (state.selected_room, state.selected_monster.clone())
                {
                    if let Err(e) = simulation::swap_monster(state, floor, pos, monster_id, &armed)
                    {
                        state.add_log(game_state::LogEntry::system(e));
                    }
                    state.selected_monster = None;
                }
            }
            UpgradeAction::Close => {
                state.selected_room = None;
                state.selected_monster = None;
                *defender_scroll = 0.0;
            }
            UpgradeAction::None => {}
        }
    }

    let chip_w = if state.adventurer_parties.is_empty() {
        132.0
    } else {
        184.0
    };
    draw_adventurer_status_chip(
        state,
        Rect::new(
            dungeon_rect.x + dungeon_rect.w - chip_w - 24.0,
            dungeon_rect.y + 24.0,
            chip_w,
            36.0,
        ),
    );

    // Mid-raid agency: the Core Smite lever. Only shown while invaders are in
    // the dungeon; fires on click or the [Q] hotkey, with the cast surfacing its
    // own feedback (recharging / no mana) via the log.
    if core_spell_visible(state) {
        let smite_rect = Rect::new(
            dungeon_rect.x + dungeon_rect.w - CORE_SPELL_BTN_W - 24.0,
            dungeon_rect.y + 24.0 + 36.0 + 10.0,
            CORE_SPELL_BTN_W,
            CORE_SPELL_BTN_H,
        );
        let clicked = draw_core_spell_button(state, smite_rect);
        if simulate && (clicked || is_key_pressed(KeyCode::Q)) {
            if let Err(e) = simulation::core_spell::cast_core_smite(state) {
                state.add_log(game_state::LogEntry::system(e));
            }
        }
    }

    // Post-raid summary card: shows what the last raid cost and earned until
    // the player dismisses it (or the next raid replaces it).
    if let Some(summary) = state.last_raid_summary.clone() {
        if draw_raid_summary(&summary, dungeon_rect) {
            state.last_raid_summary = None;
        }
    }

    if let Some(action) = state.pending_confirmation.clone() {
        match draw_confirmation_overlay(&action, sw, sh) {
            ConfirmationChoice::Confirm => match action {
                game_state::PendingConfirmation::ResetRun => {
                    *state = create_new_game(state.difficulty, 1);
                    let _ = persistence::save_game(save_slot, state);
                    reset_timers(last_time_advance, last_adventure_tick, last_save);
                }
                game_state::PendingConfirmation::DismissMonster {
                    floor,
                    room,
                    monster_id,
                } => {
                    if let Err(e) = simulation::remove_monster(state, floor, room, monster_id) {
                        state.add_log(game_state::LogEntry::system(e));
                    }
                    state.pending_confirmation = None;
                }
            },
            ConfirmationChoice::Cancel => state.pending_confirmation = None,
            ConfirmationChoice::None => {}
        }
    }

    draw_event_log(state, log_rect);

    // A siege turns the whole screen into an alarm state.
    if state.siege_active {
        draw_siege_overlay(sw, sh);
    }

    // Onboarding tutorial: highlight the relevant panel and advance as the
    // player completes each step.
    if tutorial::is_active(state) {
        let anchor_rect = match tutorial::current_anchor(state) {
            Some(tutorial::TutorialAnchor::Drawer) => drawer_rect,
            Some(tutorial::TutorialAnchor::Hud) => hud_rect,
            _ => dungeon_rect,
        };
        if tutorial::draw(state, dungeon_rect, anchor_rect) {
            tutorial::skip(state);
        }
    }
    if simulate {
        tutorial::advance(state);
    }

    // Core Power tree overlay: opened with 'P' or the BUILD-tab button. Drawn
    // before the Codex so 'C'/'P' don't fight over the same frame.
    if !*show_core_tree && !*show_codex && !*show_milestones && is_key_pressed(KeyCode::P) {
        *show_core_tree = true;
    }
    if *show_core_tree {
        match draw_core_tree(state, sw, sh) {
            CoreTreeResult::Buy(id) => {
                if let Err(e) = simulation::endgame::buy_core_power(state, &id) {
                    state.add_log(game_state::LogEntry::system(e));
                }
            }
            CoreTreeResult::Close => *show_core_tree = false,
            CoreTreeResult::None => {}
        }
    }

    // Goals overlay: the milestone track, opened with 'K'.
    if !*show_milestones && !*show_core_tree && !*show_codex && is_key_pressed(KeyCode::K) {
        *show_milestones = true;
        *milestones_scroll = 0.0;
    }
    if *show_milestones && draw_milestones(state, sw, sh, milestones_scroll) {
        *show_milestones = false;
    }

    // Codex overlay: opened with 'C', drawn last so it sits over everything.
    if !*show_codex && !*show_core_tree && !*show_milestones && is_key_pressed(KeyCode::C) {
        *show_codex = true;
        *codex_scroll = 0.0;
        state.tutorial_codex_seen = true;
    }
    if *show_codex && draw_codex(state, sw, sh, codex_scroll) {
        *show_codex = false;
    }
}
