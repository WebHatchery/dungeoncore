//! Mutations initiated by the side drawer, kept out of the frame renderer.

use crate::game_audio::{GameAudio, SoundCue};
use crate::game_state::{GameState, LogEntry, PendingConfirmation};
use crate::simulation;
use crate::ui::DrawerAction;

pub fn apply_drawer_action(
    action: DrawerAction,
    state: &mut GameState,
    show_core_tree: &mut bool,
    audio: &GameAudio,
    sfx_volume: f32,
) {
    match action {
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
            audio.play(SoundCue::Place, sfx_volume);
            if let Err(error) = simulation::add_room(state, None) {
                state.add_log(LogEntry::system(error));
            }
        }
        DrawerAction::BranchRoom => {
            audio.play(SoundCue::Place, sfx_volume);
            if let Some((floor, room)) = state.selected_room {
                if let Err(error) = simulation::branch_from(state, floor, room) {
                    state.add_log(LogEntry::system(error));
                }
            }
        }
        DrawerAction::UnlockSpecies(species) => {
            if let Err(error) = simulation::unlock_species(state, &species) {
                state.add_log(LogEntry::system(error));
            }
        }
        DrawerAction::OpenCorePowers => *show_core_tree = true,
        DrawerAction::ChannelGold => {
            if let Err(error) = simulation::economy::channel_gold_to_mana(state) {
                state.add_log(LogEntry::system(error));
            }
        }
        DrawerAction::ResetGame => state.pending_confirmation = Some(PendingConfirmation::ResetRun),
        DrawerAction::OpenHero(id) => state.selected_hero = Some(id),
        DrawerAction::CloseHero => state.selected_hero = None,
        DrawerAction::None => {}
    }
}
