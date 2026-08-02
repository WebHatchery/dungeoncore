use crate::game_state::{DungeonStatus, GameState, LogEntry};

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
    state.mana = (state.mana + regen.round() as i32).min(state.max_mana);

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
mod tests {
    use super::*;
    use crate::data::difficulty::Difficulty;
    use crate::game_state::{Adventurer, AdventurerParty, Equipment, Stats, PARTY_MOVE_SECONDS};
    use macroquad_toolkit::timing::Cooldown;

    fn adventurer(id: u64, level: i32, alive: bool) -> Adventurer {
        Adventurer {
            id,
            name: format!("Delver {id}"),
            class_name: "Warrior".to_string(),
            race: "Human".to_string(),
            level,
            hp: if alive { 30 } else { 0 },
            max_hp: 30,
            alive,
            experience: 0,
            gold: 0,
            equipment: Equipment::default(),
            conditions: Vec::new(),
            scaled_stats: Stats {
                hp: 30,
                attack: 8,
                defense: 3,
            },
        }
    }

    fn party_of(members: Vec<Adventurer>) -> AdventurerParty {
        AdventurerParty {
            id: 1,
            members,
            current_floor: 1,
            current_room: 1,
            retreating: false,
            casualties: 0,
            loot: 0,
            entry_time: 0,
            target_floor: 1,
            snared_ticks: 0,
            alarmed: false,
            sieging: false,
            prev_room: 0,
            move_anim: Cooldown::new(PARTY_MOVE_SECONDS),
        }
    }

    #[test]
    fn an_empty_dungeon_draws_nothing_from_intruders() {
        let s = GameState::new();
        assert_eq!(adventurer_presence_regen(&s), 0.0);
    }

    #[test]
    fn a_higher_level_delver_feeds_the_core_more() {
        let mut low = GameState::new();
        low.adventurer_parties
            .push(party_of(vec![adventurer(1, 1, true)]));
        let mut high = GameState::new();
        high.adventurer_parties
            .push(party_of(vec![adventurer(1, 5, true)]));

        assert!(adventurer_presence_regen(&high) > adventurer_presence_regen(&low));
    }

    #[test]
    fn the_dead_feed_nothing() {
        let mut s = GameState::new();
        s.adventurer_parties.push(party_of(vec![
            adventurer(1, 3, true),
            adventurer(2, 3, false),
        ]));
        let with_corpse = adventurer_presence_regen(&s);

        let mut alone = GameState::new();
        alone
            .adventurer_parties
            .push(party_of(vec![adventurer(1, 3, true)]));

        assert_eq!(with_corpse, adventurer_presence_regen(&alone));
    }

    #[test]
    fn presence_income_respects_difficulty() {
        let members = vec![adventurer(1, 4, true), adventurer(2, 2, true)];
        let mut lean = GameState::new();
        lean.difficulty = Difficulty::Overlord;
        lean.adventurer_parties.push(party_of(members.clone()));
        let mut rich = GameState::new();
        rich.difficulty = Difficulty::Apprentice;
        rich.adventurer_parties.push(party_of(members));

        assert!(adventurer_presence_regen(&rich) > adventurer_presence_regen(&lean));
    }

    #[test]
    fn a_party_inside_beats_the_idle_trickle() {
        // The point of the change: a raid in progress must out-earn an empty
        // dungeon by enough to cover respawning defenders and re-arming traps.
        let mut s = GameState::new();
        s.adventurer_parties.push(party_of(vec![
            adventurer(1, 2, true),
            adventurer(2, 3, true),
            adventurer(3, 3, true),
        ]));
        assert!(adventurer_presence_regen(&s) >= 5.0);
    }
}
