//! Live playing-screen orchestration, shared by interactive and capture runs.

use macroquad::prelude::*;

use crate::app_actions::apply_drawer_action;
use crate::app_support::{
    create_new_game, reset_timers, responsive_drawer_width, should_pause_after_frame_gap,
};
use crate::game_audio::{GameAudio, SoundCue};
use crate::game_state::{self, GameState};
use crate::keybindings::{BindingAction, KeyBindings};
use crate::ui::*;
use crate::{persistence, simulation, tutorial};

/// Render (and, when `simulate` is true, step) one frame of the Playing screen.
/// Shared by the interactive loop and the screenshot capture harness; the
/// capture path passes `simulate = false` so the seeded scene stays frozen and
/// the save file is never touched.
#[allow(clippy::too_many_arguments)]
pub fn render_playing_frame(
    state: &mut GameState,
    drawer_tab: &mut DrawerTab,
    upgrade_section: &mut UpgradeSection,
    drawer_open: &mut bool,
    event_log_expanded: &mut bool,
    species_scroll: &mut f32,
    defender_scroll: &mut f32,
    heroes_scroll: &mut f32,
    show_codex: &mut bool,
    show_controls: &mut bool,
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
    audio: &GameAudio,
    sfx_volume: f32,
    music_volume: f32,
    keybindings: &KeyBindings,
) {
    clear_tooltips();
    let now = get_time();
    let sw = screen_width();
    let sh = screen_height();
    let frame_seconds = get_frame_time();
    if simulate {
        state.visual_time = now as f32;
        crate::ui::set_visual_time(None);
        if !state.paused && should_pause_after_frame_gap(frame_seconds) {
            state.paused = true;
            state.add_log(game_state::LogEntry::system(
                "Dungeon paused after the browser or window was suspended. Tap Resume Dungeon to continue.",
            ));
            reset_timers(last_time_advance, last_adventure_tick, last_save);
        }
    } else {
        state.visual_time += 1.0 / 60.0;
        crate::ui::set_visual_time(Some(state.visual_time));
    }
    draw_game_background(sw, sh);

    // A pause is a first-class simulation state: the UI remains available for
    // inspection, but no time, raids, transient effects, or cooldowns advance.
    if simulate && keybindings.pressed(BindingAction::Pause) {
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

    // Simulations enqueue semantic effects; only a live interactive frame
    // consumes them, keeping capture runs silent and deterministic.
    if simulate {
        audio.update_music(state, music_volume);
        for event in state.take_sound_events() {
            audio.play(event.into(), sfx_volume);
        }
    }

    // Game over: the core has fallen. Offer a fresh dungeon.
    if state.game_over {
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.72));
        if draw_game_over_overlay(state, sw, sh) {
            // A fresh dungeon keeps the fallen run's chosen difficulty.
            *state = create_new_game(state.difficulty, 1);
            let _ = persistence::save_game(save_slot, state);
            reset_timers(last_time_advance, last_adventure_tick, last_save);
        }
        draw_tooltips();
        return;
    }

    // Modal overlay: Species Selection (Prioritize over everything else)
    if state.unlocked_species.is_empty() {
        let modal_w = (sw - 80.0).clamp(620.0, 980.0);
        let modal_h = (sh - 80.0).clamp(520.0, 620.0);
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

        draw_tooltips();
        return;
    }

    let hud_rect = Rect::new(
        OUTER_MARGIN,
        OUTER_MARGIN,
        sw - OUTER_MARGIN * 2.0,
        hud_height(sw),
    );
    match draw_top_hud(state, hud_rect) {
        ControlAction::TogglePause => {
            audio.play(SoundCue::Ui, sfx_volume);
            state.paused = !state.paused;
            if !state.paused {
                reset_timers(last_time_advance, last_adventure_tick, last_save);
            }
        }
        ControlAction::SetSpeed(speed) => {
            audio.play(SoundCue::Ui, sfx_volume);
            state.speed = speed;
            if state.paused {
                state.paused = false;
                reset_timers(last_time_advance, last_adventure_tick, last_save);
            }
        }
        ControlAction::ToggleDungeon => {
            audio.play(SoundCue::Ui, sfx_volume);
            simulation::toggle_dungeon_status(state);
        }
        ControlAction::OpenHelp => {
            audio.play(SoundCue::Ui, sfx_volume);
            *show_controls = true;
        }
        ControlAction::OpenCodex => {
            audio.play(SoundCue::Ui, sfx_volume);
            *show_codex = true;
        }
        ControlAction::OpenGoals => {
            audio.play(SoundCue::Ui, sfx_volume);
            *show_milestones = true;
        }
        _ => {}
    }

    let log_h = if *event_log_expanded {
        LOG_BAR_EXPANDED_HEIGHT
    } else {
        LOG_BAR_COLLAPSED_HEIGHT
    };
    let log_rect = Rect::new(
        OUTER_MARGIN,
        sh - OUTER_MARGIN - log_h,
        sw - OUTER_MARGIN * 2.0,
        log_h,
    );

    let body_top = hud_rect.y + hud_rect.h + PANEL_GAP;
    let body_bottom = log_rect.y - PANEL_GAP;
    let body_h = (body_bottom - body_top).max(220.0);

    let inspector_requested = state.selected_room.is_some() || state.selected_monster.is_some();
    // The catalogue and inspector take turns at the edge of the board. This
    // keeps at least three room widths readable on a 1280px viewport while
    // preserving the player's armed selection as panels change.
    let has_inspector = inspector_requested && !*drawer_open;
    let inline_inspector = has_inspector && sw >= 860.0;
    let drawer_w = responsive_drawer_width(has_inspector, *drawer_open, sw);
    let drawer_rect = Rect::new(OUTER_MARGIN, body_top, drawer_w, body_h);
    let drawer_action = draw_side_drawer(
        state,
        drawer_rect,
        drawer_tab,
        drawer_open,
        upgrade_section,
        species_scroll,
        heroes_scroll,
    );
    let reveal_selected_room = state.selected_room.is_some()
        && matches!(
            &drawer_action,
            DrawerAction::SelectMonster(_) | DrawerAction::SelectUpgrade(_)
        );
    apply_drawer_action(drawer_action, state, show_core_tree, audio, sfx_volume);
    if reveal_selected_room {
        *drawer_open = false;
    }

    let right_panel_w = if inline_inspector {
        (sw * 0.22).clamp(252.0, 286.0)
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

    if simulate && state.selected_monster.is_none() && state.selected_upgrade.is_none() {
        let navigation = if keybindings.pressed(BindingAction::NavigateLeft) {
            Some(RoomNavigation::Left)
        } else if keybindings.pressed(BindingAction::NavigateRight) {
            Some(RoomNavigation::Right)
        } else if keybindings.pressed(BindingAction::NavigateUp) {
            Some(RoomNavigation::Up)
        } else if keybindings.pressed(BindingAction::NavigateDown) {
            Some(RoomNavigation::Down)
        } else {
            None
        };
        if let Some(navigation) = navigation {
            if let Some(selected) = keyboard_room_selection(state, navigation) {
                state.selected_room = Some(selected);
                *defender_scroll = 0.0;
            }
        }
    }

    match draw_dungeon_board(state, dungeon_rect, sprites) {
        DungeonAction::RoomSelected(floor_num, room_pos) => {
            audio.play(SoundCue::Place, sfx_volume);
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
                *drawer_open = false;
                *defender_scroll = 0.0;
            }
        }
        DungeonAction::BuildRoom => {
            audio.play(SoundCue::Place, sfx_volume);
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
    if inline_inspector {
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
            UpgradeAction::ArmMonsters => {
                *drawer_tab = DrawerTab::Monsters;
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
            dungeon_rect.y + 12.0,
            chip_w,
            36.0,
        ),
    );

    // Mid-raid agency: the Core Smite lever. Only shown while invaders are in
    // the dungeon; fires on click or the [Q] hotkey, with the cast surfacing its
    // own feedback (recharging / no mana) via the log.
    if core_spell_visible(state) {
        // Keep the raid lever in the dungeon header beside the status chip;
        // the board below is reserved for the physical cutaway rooms.
        let chip_x = dungeon_rect.x + dungeon_rect.w - chip_w - 24.0;
        let smite_rect = Rect::new(
            (chip_x - CORE_SPELL_BTN_W - 10.0).max(dungeon_rect.x + 12.0),
            dungeon_rect.y + 7.0,
            CORE_SPELL_BTN_W,
            CORE_SPELL_BTN_H,
        );
        let clicked = draw_core_spell_button(state, smite_rect);
        if simulate && (clicked || keybindings.pressed(BindingAction::Smite)) {
            audio.play(SoundCue::Smite, sfx_volume);
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

    draw_event_log(state, log_rect, event_log_expanded);

    // A siege turns the whole screen into an alarm state.
    if state.siege_active {
        draw_siege_overlay(sw, sh, state.reduced_motion);
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
    if !*show_core_tree
        && !*show_codex
        && !*show_milestones
        && !*show_controls
        && keybindings.pressed(BindingAction::CorePowers)
    {
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
    if !*show_milestones
        && !*show_core_tree
        && !*show_codex
        && !*show_controls
        && keybindings.pressed(BindingAction::Goals)
    {
        *show_milestones = true;
        *milestones_scroll = 0.0;
    }
    if *show_milestones && draw_milestones(state, sw, sh, milestones_scroll) {
        *show_milestones = false;
    }

    // Codex overlay: opened with 'C', drawn last so it sits over everything.
    if !*show_codex
        && !*show_core_tree
        && !*show_milestones
        && !*show_controls
        && keybindings.pressed(BindingAction::Codex)
    {
        *show_codex = true;
        *codex_scroll = 0.0;
        state.tutorial_codex_seen = true;
    }
    if *show_codex && draw_codex(state, sw, sh, codex_scroll) {
        *show_codex = false;
    }
    if !*show_controls
        && !*show_codex
        && !*show_core_tree
        && !*show_milestones
        && keybindings.pressed(BindingAction::Help)
    {
        *show_controls = true;
    }
    if *show_controls && draw_controls_reference(sw, sh, keybindings) {
        *show_controls = false;
    }
    draw_tooltips();
}
