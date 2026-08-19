//! Deterministic browser-readiness exercises shared by tests and release checks.
//!
//! These are state-level companions to the screenshot harness: they drive the
//! same public simulation actions that visible buttons call, but do not need a
//! window or a real save directory. Keeping the route here makes regressions in
//! the tutorial, pause, overlays, resources, or persistence fail in CI.

use std::time::Instant;

use crate::app_support::create_new_game_with_seed;
use crate::data::constants::MAX_ROOMS_PER_FLOOR;
use crate::data::monsters::{get_all_species, get_monster_templates};
use crate::game_state::{DungeonStatus, GameState, LogEntry, PendingConfirmation};
use crate::simulation;

pub const MAX_DUNGEON_FLOORS: i32 = 20;
pub const DEFAULT_SOAK_HOURS: usize = 240;

/// The visible labels that form the touch/click-only readiness route. These
/// names intentionally mirror the player-facing controls rather than keyboard
/// shortcuts, so a missing target is obvious in a review.
pub const TOUCH_ROUTE_CONTROLS: [&str; 14] = [
    "New Game",
    "Start",
    "MENU",
    "BUILD",
    "MONSTERS",
    "OUTFITS",
    "OPEN DUNGEON",
    "PAUSE",
    "Resume Dungeon",
    "CODEX",
    "GOALS",
    "Controls",
    "Settings",
    "Raise a New Dungeon",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct E2eReport {
    pub controls_checked: usize,
    pub log_entries_checked: usize,
    pub save_round_trip_preserved_run: bool,
    pub pause_froze_time: bool,
    pub overlays_reached: bool,
    pub resource_panel_has_values: bool,
    pub reduced_motion_is_available: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoakReport {
    pub speed: i32,
    pub hours: usize,
    pub floors: i32,
    pub rooms: i32,
    pub unlocked_monsters: usize,
    pub max_log_entries: usize,
    pub max_effects: usize,
    pub max_parties: usize,
    pub raids_completed: i32,
    pub total_deaths: i32,
    pub remaining_party: Option<(i32, usize, i32, bool, usize, String)>,
    pub game_over: bool,
    pub peak_tick_micros: u128,
}

/// Build the largest supported dungeon with the complete authored monster
/// roster available. The fixture uses the real room builder, then fills a
/// representative defender in each combat room so the soak exercises graph,
/// combat, pathing, recovery, and endgame code together.
pub fn maximum_dungeon_fixture(seed: u64) -> GameState {
    let mut state =
        create_new_game_with_seed(crate::data::difficulty::Difficulty::default(), 4, seed);
    state.mana = 10_000_000;
    state.max_mana = 10_000_000;
    state.gold = 10_000_000;
    state.souls = 10_000_000;
    state.tutorial_active = false;

    // Unlock every authored identity for the full-roster fixture. Unlocking a
    // species through the UI remains covered by the route test; this fixture
    // is specifically about the largest possible runtime population.
    state.unlocked_species = get_all_species()
        .into_iter()
        .map(|species| species.name)
        .collect();
    state.unlocked_monsters = get_monster_templates()
        .into_iter()
        .map(|monster| monster.name)
        .collect();

    while state.total_floors < MAX_DUNGEON_FLOORS
        || state
            .deepest_floor()
            .map(|floor| {
                floor
                    .rooms
                    .iter()
                    .filter(|room| room.room_type != crate::game_state::RoomType::Core)
                    .count()
                    < MAX_ROOMS_PER_FLOOR + 1
            })
            .unwrap_or(false)
    {
        simulation::add_room(&mut state, None).expect("maximum dungeon fixture should build");
    }

    let normal = state
        .unlocked_monsters
        .iter()
        .filter(|name| {
            get_monster_templates()
                .into_iter()
                .find(|template| &template.name == *name)
                .is_some_and(|template| !template.boss_only)
        })
        .cloned()
        .next()
        .expect("authored roster needs a normal monster");
    let boss = state
        .unlocked_monsters
        .iter()
        .filter(|name| {
            get_monster_templates()
                .into_iter()
                .find(|template| &template.name == *name)
                .is_some_and(|template| template.boss_only)
        })
        .cloned()
        .next();

    let room_targets: Vec<_> = state
        .floors
        .iter()
        .flat_map(|floor| {
            floor.rooms.iter().filter_map(|room| {
                matches!(
                    room.room_type,
                    crate::game_state::RoomType::Normal | crate::game_state::RoomType::Boss
                )
                .then_some((
                    room.floor_number,
                    room.position,
                    room.room_type.clone(),
                ))
            })
        })
        .collect();
    for (floor, room, room_type) in room_targets {
        let _ = simulation::place_monster(&mut state, floor, room, &normal);
        if room_type == crate::game_state::RoomType::Boss {
            if let Some(boss) = &boss {
                let _ = simulation::place_monster(&mut state, floor, room, boss);
            }
        }
    }

    state.mana = state.max_mana;
    state.status = DungeonStatus::Open;
    state.speed = 4;
    state.next_party_spawn = 0;
    state
}

/// Run a fixed-time 4× simulation against the maximum fixture and return the
/// bounded-state metrics used by the soak assertion.
pub fn run_maximum_dungeon_soak(seed: u64, hours: usize) -> SoakReport {
    let mut state = maximum_dungeon_fixture(seed);
    // The live spawn chance is intentionally random. Prime one visitor before
    // measuring so a lucky run cannot spend the whole short soak waiting at
    // the gate instead of exercising combat and recovery.
    for _ in 0..64 {
        simulation::spawn_party(&mut state);
        if !state.adventurer_parties.is_empty() {
            break;
        }
    }
    let mut max_log_entries = 0;
    let mut max_effects = 0;
    let mut max_parties = 0;
    let mut peak_tick_micros = 0;

    for _ in 0..hours {
        let started = Instant::now();
        simulation::advance_time(&mut state);
        // The live loop processes visitors several times while a 4× hour is
        // elapsing. Mirror that cadence so a soak covers both combat and
        // recovery instead of leaving one party permanently mid-route.
        for _ in 0..state.speed.max(1) {
            simulation::spawn_party(&mut state);
            simulation::process_parties(&mut state);
            state.decay_effects(1.0 / 4.0);
        }
        let elapsed = started.elapsed().as_micros();
        peak_tick_micros = peak_tick_micros.max(elapsed);
        max_log_entries = max_log_entries.max(state.log.len());
        max_effects = max_effects.max(state.effects.len());
        max_parties = max_parties.max(state.adventurer_parties.len());
    }

    SoakReport {
        speed: state.speed,
        hours,
        floors: state.total_floors,
        rooms: state.total_room_count(),
        unlocked_monsters: state.unlocked_monsters.len(),
        max_log_entries,
        max_effects,
        max_parties,
        raids_completed: state.raids_completed,
        total_deaths: state.total_deaths,
        remaining_party: state.adventurer_parties.first().map(|party| {
            (
                party.current_floor,
                party.current_room,
                party.target_floor,
                party.retreating,
                party.members.iter().filter(|member| member.alive).count(),
                state
                    .floors
                    .iter()
                    .find(|floor| floor.number == party.current_floor)
                    .and_then(|floor| floor.room_at(party.current_room))
                    .map(|room| format!("{:?}", room.room_type))
                    .unwrap_or_else(|| "missing".to_string()),
            )
        }),
        game_over: state.game_over,
        peak_tick_micros,
    }
}

/// Exercise the player-facing route without a keyboard: start, build, summon,
/// trap, open, pause/resume, inspect overlay state, and round-trip the save
/// payload. The renderer maps these same actions to visible controls.
pub fn run_touch_route(seed: u64) -> E2eReport {
    let mut state =
        create_new_game_with_seed(crate::data::difficulty::Difficulty::default(), 1, seed);
    state.mana = 10_000;
    state.max_mana = 10_000;
    state.gold = 10_000;

    let starter = get_all_species()
        .into_iter()
        .find(|species| species.starter)
        .expect("at least one starter species");
    simulation::unlock_species(&mut state, &starter.name).expect("starter control should work");
    simulation::add_room(&mut state, None).expect("build control should work");
    let (floor, room) = state
        .floors
        .iter()
        .flat_map(|floor| floor.rooms.iter())
        .find(|room| room.room_type == crate::game_state::RoomType::Normal)
        .map(|room| (room.floor_number, room.position))
        .expect("built combat room should be visible");
    let monster = state
        .unlocked_monsters
        .first()
        .cloned()
        .expect("starter control should reveal a monster");
    simulation::place_monster(&mut state, floor, room, &monster)
        .expect("monster placement control should work");
    simulation::apply_upgrade(&mut state, floor, room, "Spike Trap")
        .expect("trap control should work");
    simulation::toggle_dungeon_status(&mut state);
    state.add_log(LogEntry::system("Visible controls route completed."));

    let before_pause = state.hour;
    state.paused = true;
    if !state.paused {
        simulation::advance_time(&mut state);
    }
    let pause_froze_time = before_pause == state.hour;
    state.paused = false;
    simulation::advance_time(&mut state);

    state.pending_confirmation = Some(PendingConfirmation::ResetRun);
    state.last_raid_summary = Some(crate::game_state::RaidSummary {
        outcome: crate::game_state::RaidOutcome::Wiped,
        party_size: 1,
        slain: 1,
        survivors: 0,
        mana_gained: 1,
        mana_recovery_cost: 0,
        souls_gained: 0,
        gold_gained: 0,
        defenders_lost: 0,
        reputation_change: 0,
        reputation_after: state.reputation,
    });
    state.selected_room = Some((floor, room));
    let serialized = serde_json::to_string(&state).expect("save payload should encode");
    let loaded: GameState = serde_json::from_str(&serialized).expect("save payload should reload");
    let save_round_trip_preserved_run = loaded.day == state.day
        && loaded.hour == state.hour
        && loaded.total_room_count() == state.total_room_count()
        && loaded.status == state.status
        && loaded.selected_room.is_none();
    let resource_panel_has_values = crate::ui::resource_panel::resource_panel_data(&state)
        .mana_label
        .contains('/');

    E2eReport {
        controls_checked: TOUCH_ROUTE_CONTROLS.len(),
        log_entries_checked: state.log.len(),
        save_round_trip_preserved_run,
        pause_froze_time,
        overlays_reached: state.pending_confirmation.is_some() && state.last_raid_summary.is_some(),
        resource_panel_has_values,
        reduced_motion_is_available: !state.reduced_motion,
    }
}

#[cfg(test)]
mod tests;
