use crate::game_state::GameState;
use crate::simulation;

/// A depth-tab fixture with every chapter unlocked, so the capture proves the
/// permanent relic ledger and the deeper, stratum-specific defender rewards.
pub(super) fn seed(state: &mut GameState) {
    state.tutorial_active = false;
    let _ = simulation::unlock_species(state, "Goblinoid");
    state.mana = 6_000;
    state.gold = 2_000;
    state.souls = 24;
    state.total_floors = 1;
    // Nine floors are enough to show the first three strata while keeping the
    // screenshot fixture quick to render. The full 17-floor path is covered by
    // the gameplay/readiness soak and is not needed in a drawer-only capture.
    while state.total_floors < 9 {
        let _ = simulation::add_room(state, None);
    }
    state.depth_relics = [
        "rootbound_sigil",
        "cinder_crown",
        "tide_lens",
        "prism_heart",
        "ossuary_key",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    state.status = crate::game_state::DungeonStatus::Closed;
    state.selected_room = None;
}
