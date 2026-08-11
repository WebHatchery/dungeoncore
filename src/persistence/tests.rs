use super::*;

#[test]
fn slot_names_are_distinct_and_include_the_default() {
    assert!(SAVE_SLOTS.contains(&DEFAULT_SLOT));
    assert_eq!(SAVE_SLOTS.len(), 3);
    assert_ne!(SAVE_SLOTS[0], SAVE_SLOTS[1]);
    assert_ne!(SAVE_SLOTS[1], SAVE_SLOTS[2]);
}

#[test]
fn ready_slot_metadata_describes_a_run_without_persisting_ui_state() {
    let mut state = GameState::new();
    state.day = 14;
    state.total_floors = 4;
    state.prestige = 2;
    state.status = crate::game_state::DungeonStatus::Open;

    let metadata = ready_slot_state(&state);
    assert_eq!(
        metadata,
        SlotState::Ready {
            day: 14,
            difficulty: "Keeper".to_string(),
            deepest_floor: 4,
            prestige: 2,
            dungeon_open: true,
        }
    );
}

#[test]
fn migration_registry_has_named_idempotent_steps() {
    assert!(!SAVE_MIGRATIONS.is_empty());
    assert!(SAVE_MIGRATIONS
        .iter()
        .all(|migration| !migration.name.is_empty()));

    let mut state = GameState::new();
    for room in &mut state.floors[0].rooms {
        room.exits.clear();
    }
    apply_save_migrations(&mut state);
    assert!(state.floors[0].validate_graph().is_ok());
    apply_save_migrations(&mut state);
    assert!(state.floors[0].validate_graph().is_ok());
}

#[test]
fn older_wrapper_payload_decodes_through_the_registry_path() {
    let state = GameState::new();
    let wrapper = serde_json::json!({
        "slot": { "version": "0.0.1" },
        "data": state,
    });
    let migrated = migrate_slot(Some("0.0.1".to_string()), wrapper).unwrap();
    assert_eq!(migrated.day, 1);
}
