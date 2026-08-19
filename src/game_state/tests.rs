use super::*;

fn hero(id: u64, delves: i32, kills: i32) -> HeroRecord {
    HeroRecord {
        id,
        name: "Sable the Bold".to_string(),
        class_name: "Rogue".to_string(),
        race: "Halfling".to_string(),
        level: 4,
        experience: 0,
        delves,
        kills,
        gold_stolen: 0,
        escapes: 0,
        deepest_floor: 0,
        status: HeroStatus::Inside,
        death_floor: 0,
        death_day: 0,
        journal: Vec::new(),
    }
}

#[test]
fn a_journal_keeps_only_its_last_pages() {
    // A long campaign must not grow the save without bound.
    let mut h = hero(1, 0, 0);
    for day in 1..=(crate::game_state::heroes::HERO_JOURNAL_LIMIT as i32 + 5) {
        h.remember(day, format!("Delve {day}"));
    }
    assert_eq!(
        h.journal.len(),
        crate::game_state::heroes::HERO_JOURNAL_LIMIT
    );
    // The oldest fell off the front; the newest is still there.
    assert_eq!(h.journal.first().unwrap().text, "Delve 6");
    assert_eq!(h.journal.last().unwrap().text, "Delve 17");
}

#[test]
fn slaying_a_rival_pays_a_bounty() {
    let mut s = GameState::new();
    s.known_adventurers.push(hero(42, 4, 6));
    let souls_before = s.souls;
    let gold_before = s.gold;
    s.record_hero_death(42, 2);
    assert!(s.souls > souls_before, "rival death grants souls");
    assert!(s.gold > gold_before, "rival death grants gold");
    assert_eq!(s.known_adventurers[0].status, HeroStatus::Dead);
}

#[test]
fn slaying_a_nobody_pays_nothing() {
    let mut s = GameState::new();
    s.known_adventurers.push(hero(7, 1, 0));
    let souls_before = s.souls;
    let gold_before = s.gold;
    s.record_hero_death(7, 1);
    assert_eq!(s.souls, souls_before);
    assert_eq!(s.gold, gold_before);
}

// --- Dungeon graph (Phase A) --------------------------------------------

#[test]
fn fresh_floor_is_a_valid_graph() {
    let s = GameState::new();
    assert!(s.floors[0].validate_graph().is_ok());
    // Entrance(0) -> Core(1); the sink has no exits.
    assert_eq!(s.floors[0].room_at(0).unwrap().exits, vec![1]);
    assert!(s.floors[0].room_at(1).unwrap().exits.is_empty());
}

#[test]
fn migrate_rebuilds_linear_exits_for_pre_graph_saves() {
    let mut s = GameState::new();
    // Simulate an old save: strip all edges.
    for f in &mut s.floors {
        for r in &mut f.rooms {
            r.exits.clear();
        }
    }
    s.migrate();
    assert_eq!(s.floors[0].room_at(0).unwrap().exits, vec![1]);
    assert!(s.floors[0].validate_graph().is_ok());
}

#[test]
fn validate_rejects_an_unreachable_dead_end() {
    let mut s = GameState::new();
    // A stray room nothing points to and that goes nowhere.
    s.floors[0]
        .rooms
        .push(Room::new(99, RoomType::Normal, 7, 1));
    assert!(s.floors[0].validate_graph().is_err());
}

#[test]
fn building_extends_the_linear_chain() {
    let mut s = GameState::new();
    s.mana = 1000;
    crate::simulation::add_room(&mut s, None).unwrap();
    let f = &s.floors[0];
    assert!(f.validate_graph().is_ok());
    // Entrance(0) -> Normal(1) -> Core(2).
    assert_eq!(f.room_at(0).unwrap().exits, vec![1]);
    assert_eq!(f.room_at(1).unwrap().exits, vec![2]);
    assert!(f.room_at(2).unwrap().exits.is_empty());
}

#[test]
fn saved_run_rng_continues_the_same_future() {
    let mut original = GameState::new();
    original.run_seed = 0xC0DE_CAFE;
    original.run_rng = SeededRng::new(original.run_seed);
    let _already_drawn = original.run_rng.next_u64();

    let serialized = serde_json::to_string(&original).expect("state serializes");
    let mut restored: GameState = serde_json::from_str(&serialized).expect("state restores");
    assert_eq!(restored.run_seed, original.run_seed);
    assert_eq!(restored.run_rng.next_u64(), original.run_rng.next_u64());
}

#[test]
fn restored_runs_start_with_a_visible_board_zoom() {
    let original = GameState::new();
    let serialized = serde_json::to_string(&original).expect("state serializes");
    let restored: GameState = serde_json::from_str(&serialized).expect("state restores");

    assert_eq!(restored.board_zoom, 1.0);
}

#[test]
fn event_log_keeps_chronological_order_when_it_trims() {
    let mut state = GameState::new();
    state.log.clear();
    let limit = crate::data::MAX_LOG_ENTRIES;
    for entry in 0..=limit {
        state.add_log(LogEntry::system(format!("Event {entry}")));
    }

    assert_eq!(state.log.len(), limit);
    assert_eq!(state.log.first().unwrap().message, "Event 1");
    assert_eq!(state.log.last().unwrap().message, format!("Event {limit}"));
}

#[test]
fn cosmetic_sound_queue_is_bounded_and_drains_once() {
    let mut state = GameState::new();
    for _ in 0..14 {
        state.queue_sound(SoundEvent::Combat);
    }
    state.queue_sound(SoundEvent::Trap);

    let queued = state.take_sound_events();
    assert_eq!(queued.len(), 12);
    assert_eq!(queued.last(), Some(&SoundEvent::Trap));
    assert!(state.take_sound_events().is_empty());
}
