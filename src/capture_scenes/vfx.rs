//! Deterministic visual-effect fixtures used by the capture harness.

use crate::game_state::{EffectAnchor, EffectKind, GameState};
use crate::simulation;

use super::{find_combat_room, first_starter_species};

/// Seed a VFX-focused scene, returning whether `scene` was handled here.
pub(super) fn seed(state: &mut GameState, scene: &str, narrow: bool) -> bool {
    match scene {
        "prestige_vfx" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            state.prestige = 2;
            simulation::endgame::repel_siege(state);
            if narrow {
                // Keep the Core in the narrow viewport so the prestige burst
                // spawned by `repel_siege` is part of the capture, not the
                // adjacent build preview.
                state.board_pan_x = 650.0;
            }
            true
        }
        "vfx_showcase" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            let _ = simulation::add_room(state, None);
            if let Some((floor, room)) = find_combat_room(state) {
                state.selected_room = Some((floor, room));
                state.push_effect_at(
                    floor,
                    room,
                    "TRAP!",
                    EffectKind::Ability,
                    EffectAnchor::Invaders,
                );
                state.push_element_effect_at(
                    floor,
                    room,
                    "-18",
                    EffectKind::Damage,
                    EffectAnchor::Invaders,
                    "Fire",
                );
                state.push_element_effect_at(
                    floor,
                    room,
                    "",
                    EffectKind::HitSpark,
                    EffectAnchor::Invaders,
                    "Water",
                );
                state.push_effect_at(
                    floor,
                    room,
                    "POISON!",
                    EffectKind::PoisonCloud,
                    EffectAnchor::Invaders,
                );
                state.push_effect_at(
                    floor,
                    room,
                    "Held!",
                    EffectKind::Ability,
                    EffectAnchor::Center,
                );
                state.push_effect_at(
                    floor,
                    room,
                    "Slain!",
                    EffectKind::AdventurerDown,
                    EffectAnchor::Invaders,
                );
                state.push_effect_at(floor, room, "+12g", EffectKind::Loot, EffectAnchor::Center);
                state.push_effect_at(floor, room, "", EffectKind::MeleeDust, EffectAnchor::Center);
            }
            state.total_deaths = state.siege_threshold();
            simulation::endgame::maybe_launch_siege(state);
            simulation::endgame::repel_siege(state);
            if narrow {
                state.board_pan_x = 340.0;
            }
            true
        }
        _ => false,
    }
}
