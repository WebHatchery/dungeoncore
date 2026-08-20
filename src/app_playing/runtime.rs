use crate::app_support::should_pause_after_frame_gap;
use crate::game_audio::{GameAudio, SoundCue};
use crate::game_state::{self, GameState};
use crate::keybindings::{BindingAction, KeyBindings};
use crate::persistence;
use crate::simulation;
use crate::ui::{self, simulation_active};

use super::session::{PlayingFrameSettings, PlayingSession};

/// Advance the live simulation and consume its renderer-facing feedback.
/// Capture frames bypass this module by setting `simulate` to false, which
/// keeps screenshots deterministic and prevents writes to the save slots.
pub(super) fn advance(
    state: &mut GameState,
    session: &mut PlayingSession,
    settings: PlayingFrameSettings<'_>,
    keybindings: &KeyBindings,
    audio: &GameAudio,
    now: f64,
    frame_seconds: f32,
) {
    if settings.simulate {
        state.visual_time = now as f32;
        ui::set_visual_time(None);
        if !state.paused && should_pause_after_frame_gap(frame_seconds) {
            state.paused = true;
            state.add_log(game_state::LogEntry::system(
                "Dungeon paused after the browser or window was suspended. Tap Resume Dungeon to continue.",
            ));
            session.timing.reset();
        }
    } else {
        state.visual_time += 1.0 / 60.0;
        ui::set_visual_time(Some(state.visual_time));
    }

    if settings.simulate && keybindings.pressed(BindingAction::Pause) {
        state.paused = !state.paused;
        if !state.paused {
            session.timing.reset();
        }
    }

    if simulation_active(settings.simulate, state.paused) {
        state.decay_effects(frame_seconds);
        for party in &mut state.adventurer_parties {
            party.move_anim.tick(frame_seconds);
        }
        state.core_smite_cooldown.tick(frame_seconds);

        let time_interval = 5.0 / state.speed as f64;
        if now - session.timing.last_time_advance > time_interval {
            simulation::advance_time(state);
            session.timing.last_time_advance = now;
        }

        if now - session.timing.last_adventure_tick > 2.0 {
            simulation::spawn_party(state);
            simulation::process_parties(state);
            session.timing.last_adventure_tick = now;
        }

        if now - session.timing.last_save > settings.autosave_interval {
            if let Err(error) = persistence::save_game(settings.save_slot, state) {
                eprintln!("Failed to save: {error}");
            }
            session.timing.last_save = now;
        }
    }

    if settings.simulate {
        audio.update_music(state, settings.music_volume);
        for event in state.take_sound_events() {
            audio.play(SoundCue::from(event), settings.sfx_volume);
        }
    }
}
