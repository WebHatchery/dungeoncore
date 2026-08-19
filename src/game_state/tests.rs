use super::*;

fn hero(id: u64, delves: i32, kills: i32) -> HeroRecord {
    HeroRecord {
        id,
        name: "Sable the Bold".to_string(),
        class_name: "Rogue".to_string(),
        race: "Halfling".to_string(),
        drive: HeroDrive::Glory,
        resolve: 65,
        level: 4,
        experience: 0,
        delves,
        kills,
        gold_stolen: 0,
        escapes: 0,
        deepest_floor: 0,
        insights: Vec::new(),
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

#[test]
fn legacy_heroes_gain_safe_drive_and_resolve_defaults() {
    let record = hero(9, 2, 1);
    let mut value = serde_json::to_value(record).unwrap();
    let object = value.as_object_mut().unwrap();
    object.remove("drive");
    object.remove("resolve");
    object.remove("insights");

    let restored: HeroRecord = serde_json::from_value(value).unwrap();
    assert_eq!(restored.drive, HeroDrive::Duty);
    assert_eq!(restored.resolve, 50);
    assert!(restored.insights.is_empty());
}

#[test]
fn escaped_strata_build_bounded_deterministic_hero_wards() {
    let mut record = hero(11, 4, 2);
    assert_eq!(record.learn_stratum(5, 1).unwrap().label(), "Fire I");
    assert_eq!(record.learn_stratum(5, 2).unwrap().label(), "Fire III");
    assert!(record.learn_stratum(5, 1).is_none(), "mastery caps at III");
    assert_eq!(record.learn_stratum(9, 3).unwrap().label(), "Water III");

    let prepared = record.prepared_ward();
    assert_eq!(prepared.label(), "Water III", "later tie wins");
    assert_eq!(prepared.attack_multiplier_against("Water"), 1.12);
    assert_eq!(prepared.damage_multiplier_from("Water"), 0.76);
    assert_eq!(prepared.damage_multiplier_from("Fire"), 1.0);
}

#[test]
fn legacy_live_adventurers_return_without_an_invented_ward() {
    let adventurer = Adventurer {
        id: 7,
        name: "Mara".to_string(),
        class_name: "Warrior".to_string(),
        race: "Human".to_string(),
        drive: HeroDrive::Duty,
        resolve: 50,
        ward: HeroWard::default(),
        level: 2,
        hp: 40,
        max_hp: 40,
        alive: true,
        experience: 0,
        gold: 0,
        equipment: Equipment::default(),
        conditions: Vec::new(),
        scaled_stats: Stats {
            hp: 40,
            attack: 8,
            defense: 3,
        },
    };
    let mut value = serde_json::to_value(adventurer).unwrap();
    value.as_object_mut().unwrap().remove("ward");

    let restored: Adventurer = serde_json::from_value(value).unwrap();
    assert_eq!(restored.ward, HeroWard::default());
}
