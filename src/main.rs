//! Dungeon Core - A dungeon management game
#![allow(dead_code)]
//!
//! Migrated from React/TypeScript + PHP to Rust using macroquad.

mod app_actions;
mod app_playing;
mod app_support;
mod capture_scenes;
mod data;
mod game_audio;
mod game_state;
mod keybindings;
mod persistence;
mod readiness;
mod simulation;
mod ui;

use macroquad::miniquad::window::quit;
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::capture;

use app_playing::render_playing_frame;
use app_support::*;
use game_audio::GameAudio;
use keybindings::KeyBindings;
use ui::*;

#[macroquad::main(window_conf)]
async fn main() {
    // Install before loading assets or saves so even startup failures leave a
    // report beside the player's untouched save file on native builds.
    macroquad_toolkit::crash::install_crash_log("dungeon_core");

    let mut assets = AssetManager::new();
    if let Err(error) = assets.load_asset_pack("assets.zip").await {
        eprintln!("Failed to load asset pack; loose asset fallback will be used: {error}");
    }
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
    if let Err(e) = assets
        .load_texture_with_filter(
            ANIMATED_UNIT_SHEET_KEY,
            ANIMATED_UNIT_SHEET_PATH,
            FilterMode::Nearest,
        )
        .await
    {
        eprintln!(
            "Failed to load animated unit sprite sheet; using pose atlas: {}",
            e
        );
    }
    for (key, path, label) in [
        (
            ANIMATED_ADVENTURER_SHEET_KEY,
            ANIMATED_ADVENTURER_SHEET_PATH,
            "animated adventurer",
        ),
        (
            ANIMATED_MONSTER_SHEET_KEY,
            ANIMATED_MONSTER_SHEET_PATH,
            "animated monster",
        ),
        (GIANT_RAT_SPRITE_KEY, GIANT_RAT_SPRITE_PATH, "giant rat"),
        (
            ANIMATED_FULL_MONSTER_SHEET_KEY,
            ANIMATED_FULL_MONSTER_SHEET_PATH,
            "full animated monster",
        ),
    ] {
        if let Err(e) = assets
            .load_texture_with_filter(key, path, FilterMode::Nearest)
            .await
        {
            eprintln!("Failed to load {label} sprite sheet; using fallbacks: {e}");
        }
    }
    let sprites = DungeonSprites::from_assets(&assets);
    let audio = GameAudio::new().await;

    // Screenshot capture harness: when DUNGEON_CORE_CAPTURE_PATH is set, seed a
    // scene, render a fixed number of frames, write a PNG, and exit. No input,
    // no simulation drift, and the player's save file is left untouched.
    if let Some(configs) = capture::CaptureConfig::all_from_env(CAPTURE_PREFIX) {
        for config in configs {
            let scene = capture_scenes::base_scene(&config.scene);
            if matches!(
                scene,
                "title" | "new_game" | "settings" | "save_slots" | "overwrite"
            ) {
                let mut seed_input = String::new();
                let settings = macroquad_toolkit::settings::GameSettings::default();
                let save_states = [
                    persistence::SlotState::Ready {
                        day: 18,
                        difficulty: "Keeper".to_string(),
                        deepest_floor: 4,
                        prestige: 2,
                        dungeon_open: false,
                    },
                    persistence::SlotState::Empty,
                    persistence::SlotState::Corrupt,
                ];
                capture::run_capture_once(&config, |_dt| match scene {
                    "title" => {
                        let _ = draw_title_screen(&assets, true, None);
                    }
                    "new_game" => {
                        let _ = draw_new_game_setup(&assets, &mut seed_input, None);
                    }
                    "settings" => {
                        let _ = draw_title_settings_screen(&assets, &settings, None);
                    }
                    "save_slots" => {
                        let _ = draw_save_slots_screen(&assets, &save_states, None);
                    }
                    "overwrite" => {
                        let _ = draw_slot_overwrite_confirmation(&assets, "Slot 1");
                    }
                    _ => {}
                })
                .await;
                continue;
            }
            let mut cap_state = create_new_game(data::difficulty::Difficulty::default(), 1);
            capture_scenes::seed_capture_scene(&mut cap_state, &config.scene);
            // Most scenes show the Monsters tab; a couple open the tab they exist
            // to show off.
            let mut drawer_tab = match scene {
                "build" => DrawerTab::Build,
                "variants" => DrawerTab::Evolution,
                "traps" => DrawerTab::Traps,
                "journal" => DrawerTab::Heroes,
                _ => DrawerTab::Monsters,
            };
            let mut upgrade_section = UpgradeSection::Traps;
            let mut drawer_open = matches!(
                scene,
                "build" | "variants" | "traps" | "journal" | "placement"
            );
            let mut event_log_expanded = scene == "log";
            let mut species_scroll = 0.0;
            let mut defender_scroll = 0.0;
            let mut heroes_scroll = 0.0;
            let mut show_codex = scene == "codex";
            let mut show_controls = scene == "controls";
            let mut codex_scroll = 0.0;
            // The `coretree` scene boots straight into the core-power tree overlay.
            let mut show_core_tree = scene == "coretree";
            // The `goals` scene boots straight into the milestone overlay.
            let mut show_milestones = scene == "goals";
            let mut milestones_scroll = 0.0;
            let mut t0 = get_time();
            let mut t1 = t0;
            let mut t2 = t0;
            let strip = capture::filmstrip::StripConfig::from_env(CAPTURE_PREFIX);
            if let Some(strip) = strip {
                capture::filmstrip::run_filmstrip(&config, &strip, |dt| {
                    cap_state.visual_time += dt;
                    crate::ui::set_visual_time(Some(cap_state.visual_time));
                    render_playing_frame(
                        &mut cap_state,
                        &mut drawer_tab,
                        &mut upgrade_section,
                        &mut drawer_open,
                        &mut event_log_expanded,
                        &mut species_scroll,
                        &mut defender_scroll,
                        &mut heroes_scroll,
                        &mut show_codex,
                        &mut show_controls,
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
                        &audio,
                        0.0,
                        0.0,
                        &KeyBindings::default(),
                    );
                })
                .await;
            } else {
                capture::run_capture_once(&config, |dt| {
                    cap_state.visual_time += dt;
                    crate::ui::set_visual_time(Some(cap_state.visual_time));
                    render_playing_frame(
                        &mut cap_state,
                        &mut drawer_tab,
                        &mut upgrade_section,
                        &mut drawer_open,
                        &mut event_log_expanded,
                        &mut species_scroll,
                        &mut defender_scroll,
                        &mut heroes_scroll,
                        &mut show_codex,
                        &mut show_controls,
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
                        &audio,
                        0.0,
                        0.0,
                        &KeyBindings::default(),
                    );
                })
                .await;
            }
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
    let mut seed_input = String::new();
    let mut settings = macroquad_toolkit::settings::GameSettings::load("dungeon_core");
    settings.sanitize();
    settings.apply_display();
    let mut keybindings = KeyBindings::load();
    let mut capturing_binding = None;

    // Timing variables
    let mut last_time_advance = get_time();
    let mut last_adventure_tick = get_time();
    let mut last_save = get_time();
    let mut drawer_tab = DrawerTab::Monsters;
    let mut upgrade_section = UpgradeSection::Traps;
    let mut drawer_open = false;
    let mut event_log_expanded = false;
    let mut species_scroll = 0.0;
    let mut defender_scroll = 0.0;
    let mut heroes_scroll = 0.0;
    let mut show_codex = false;
    let mut show_controls = false;
    let mut codex_scroll = 0.0;
    let mut show_core_tree = false;
    let mut show_milestones = false;
    let mut milestones_scroll = 0.0;

    loop {
        if screen != AppScreen::Playing {
            audio.update_title_music(
                settings.effective_music_volume(),
                is_mouse_button_pressed(MouseButton::Left) || get_last_key_pressed().is_some(),
            );
        }
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
                        seed_input.clear();
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
                match draw_new_game_setup(&assets, &mut seed_input, title_notice.as_deref()) {
                    NewGameSetupAction::Start(difficulty, seed) => {
                        state = match seed {
                            Some(seed) => {
                                create_new_game_with_seed(difficulty, settings.default_speed, seed)
                            }
                            None => create_new_game(difficulty, settings.default_speed),
                        };
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
                if action == TitleSettingsAction::OpenKeybindings {
                    title_notice = None;
                    screen = AppScreen::Keybindings;
                    next_frame().await;
                    continue;
                }
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
            AppScreen::Keybindings => {
                match draw_keybindings_screen(&mut keybindings, &mut capturing_binding) {
                    KeybindingsScreenAction::Changed => {
                        if let Err(error) = keybindings.save() {
                            title_notice =
                                Some(format!("Could not save keyboard bindings: {error}"));
                        } else {
                            title_notice = Some("Keyboard bindings saved.".to_string());
                        }
                    }
                    KeybindingsScreenAction::Back => screen = AppScreen::Settings,
                    KeybindingsScreenAction::None => {}
                }
                next_frame().await;
                continue;
            }
            AppScreen::Playing => {}
        }

        state.reduced_motion = !settings.screen_shake;
        render_playing_frame(
            &mut state,
            &mut drawer_tab,
            &mut upgrade_section,
            &mut drawer_open,
            &mut event_log_expanded,
            &mut species_scroll,
            &mut defender_scroll,
            &mut heroes_scroll,
            &mut show_codex,
            &mut show_controls,
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
            &audio,
            settings.effective_sfx_volume(),
            settings.effective_music_volume(),
            &keybindings,
        );

        next_frame().await;
    }
}
