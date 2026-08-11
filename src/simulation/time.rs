use crate::game_state::{DungeonStatus, GameState, LogEntry, SoundEvent};

/// Mana per hour the core draws from the intruders currently inside it.
///
/// Living intruders feed the core the whole time they are in the dungeon, and a
/// higher-level delver carries more to drain. This — not the kill payout — is
/// the dungeon's main income during a raid, so a long deep delve is worth more
/// than a quick wipe, and a dungeon lethal enough to farm kills trades this
/// income away for the threat that eventually musters a siege.
pub fn adventurer_presence_regen(state: &GameState) -> f32 {
    let per_adventurer = crate::data::constants::mana_regen_per_adventurer();
    let per_level = crate::data::constants::mana_regen_per_adventurer_level();
    state
        .adventurer_parties
        .iter()
        .flat_map(|party| party.members.iter())
        .filter(|adventurer| adventurer.alive)
        .map(|adventurer| per_adventurer + adventurer.level as f32 * per_level)
        .sum::<f32>()
        * state.income_mult()
}

/// Advance game time by one hour
pub fn advance_time(state: &mut GameState) {
    state.hour += 1;
    if state.hour >= 24 {
        state.hour = 0;
        state.day += 1;
    }

    // Core-power tree: summed permanent regen bonus (Deep Roots, Wellspring…).
    let core_power_bonus = crate::simulation::endgame::core_mana_regen_bonus(state);

    // Mana regeneration (rounded so fractional bonuses aren't lost)
    let regen = 1.0 + state.deep_core_bonus + adventurer_presence_regen(state) + core_power_bonus;
    state.mana_regen = regen;
    let mana_before = state.mana;
    state.mana = (state.mana + regen.round() as i32).min(state.max_mana);
    if !state.adventurer_parties.is_empty() && state.mana > mana_before {
        state.queue_sound(SoundEvent::Income);
    }

    // Auto-close dungeon when closing and no parties remain
    if state.status == DungeonStatus::Closing && state.adventurer_parties.is_empty() {
        state.status = DungeonStatus::Closed;
        state.add_log(LogEntry::system("Dungeon is now closed."));
    }

    // Process hourly monster traits
    crate::simulation::monsters::process_hourly_traits(state);

    // Unlock evolved forms as defenders gain experience (no auto-transform).
    crate::simulation::monsters::process_evolution_unlocks(state);

    // Escalating warnings when too many adventurers die in the dungeon.
    check_threat_level(state);

    // Unlock any newly-earned milestones on the goal track.
    crate::simulation::milestones::check_milestones(state);

    // At peak fury the realm musters its army for a siege on the core.
    crate::simulation::endgame::maybe_launch_siege(state);
}

/// Emit escalating warnings as the dungeon's death toll rises.
fn check_threat_level(state: &mut GameState) {
    let tier = state.threat_tier();
    if tier > state.threat_warned {
        state.threat_warned = tier;
        let message = match tier {
            1 => "Word spreads: adventurers are dying in your dungeon. The nearby town grows wary.",
            2 => "The Adventurers' Guild has posted warnings about your dungeon's death toll.",
            3 => "The kingdom has taken notice. So many have died that a reckoning is being prepared.",
            _ => "Your dungeon is branded a deathtrap. The realm is mustering an army to destroy your core.",
        };
        state.add_log(LogEntry::system(message));
        state.queue_sound(SoundEvent::Threat);
    }
}

/// Toggle dungeon status between Open and Closed
pub fn toggle_dungeon_status(state: &mut GameState) {
    match state.status {
        DungeonStatus::Open => {
            if state.adventurer_parties.is_empty() {
                state.status = DungeonStatus::Closed;
                state.add_log(LogEntry::system("Dungeon is now closed to adventurers."));
            } else {
                state.status = DungeonStatus::Closing;
                state.add_log(LogEntry::system(
                    "Dungeon is closing... waiting for adventurers to finish.",
                ));
            }
        }
        DungeonStatus::Closed | DungeonStatus::Closing => {
            state.status = DungeonStatus::Open;
            state.add_log(LogEntry::system("Dungeon is now open to adventurers!"));
        }
    }
}

/// Cycle game speed: 1 -> 2 -> 4 -> 1
pub fn toggle_speed(state: &mut GameState) {
    state.speed = match state.speed {
        1 => 2,
        2 => 4,
        _ => 1,
    };
}

#[cfg(test)]
mod tests;
