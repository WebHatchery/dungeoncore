use super::*;

#[test]
fn visible_touch_route_covers_the_required_browser_actions() {
    for required in [
        "New Game",
        "Start",
        "BUILD",
        "MONSTERS",
        "OUTFITS",
        "OPEN DUNGEON",
        "Resume Dungeon",
        "Settings",
        "Raise a New Dungeon",
    ] {
        assert!(
            TOUCH_ROUTE_CONTROLS.contains(&required),
            "missing visible touch control: {required}"
        );
    }
}

#[test]
fn end_to_end_route_covers_logs_save_pause_overlays_and_resources() {
    let report = run_touch_route(0x00D0_6E00_u64);
    assert_eq!(report.controls_checked, TOUCH_ROUTE_CONTROLS.len());
    assert!(report.log_entries_checked >= 4);
    assert!(report.save_round_trip_preserved_run);
    assert!(report.pause_froze_time);
    assert!(report.overlays_reached);
    assert!(report.resource_panel_has_values);
    assert!(report.reduced_motion_is_available);
}

#[test]
fn full_roster_maximum_dungeon_soak_runs_at_4x_with_bounded_state() {
    let report = run_maximum_dungeon_soak(0x00C0_4E4A_u64, DEFAULT_SOAK_HOURS);
    assert_eq!(report.speed, 4);
    assert_eq!(report.hours, DEFAULT_SOAK_HOURS);
    assert_eq!(report.floors, MAX_DUNGEON_FLOORS);
    assert!(report.rooms >= 20 * MAX_ROOMS_PER_FLOOR as i32);
    assert_eq!(
        report.unlocked_monsters,
        get_monster_templates().len(),
        "the soak must exercise the complete authored roster"
    );
    assert!(report.max_log_entries <= crate::data::MAX_LOG_ENTRIES);
    assert!(report.max_effects <= 48);
    assert!(report.max_parties <= 1);
    assert!(report.raids_completed > 0, "soak report: {report:?}");
    assert!(
        report.peak_tick_micros < 100_000,
        "a simulation tick exceeded the 100ms readiness budget: {}µs",
        report.peak_tick_micros
    );
}
