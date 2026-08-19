use super::*;

fn expedition_member(id: u64, level: i32) -> Adventurer {
    build_adventurer(
        id,
        format!("Hero {id}"),
        "Warrior",
        "Human",
        HeroDrive::Duty,
        50,
        level,
        1.0,
    )
}

fn veteran_record(id: u64, level: i32, delves: i32) -> HeroRecord {
    HeroRecord {
        id,
        name: format!("Hero {id}"),
        class_name: "Warrior".to_string(),
        race: "Human".to_string(),
        drive: HeroDrive::Duty,
        resolve: 60,
        level,
        experience: 0,
        delves,
        kills: 0,
        gold_stolen: 0,
        escapes: delves.saturating_sub(1),
        deepest_floor: 2,
        status: HeroStatus::Inside,
        death_floor: 0,
        death_day: 0,
        journal: Vec::new(),
    }
}

#[test]
fn fresh_expeditions_only_test_the_upper_dungeon() {
    let mut state = GameState::new();
    state.total_floors = 10;
    let members = vec![expedition_member(1, 1), expedition_member(2, 2)];

    assert_eq!(expedition_target_floor(&state, &members), 2);
}

#[test]
fn strong_returning_heroes_push_expeditions_deeper() {
    let mut state = GameState::new();
    state.total_floors = 10;
    state.known_adventurers.push(veteran_record(1, 6, 5));
    let members = vec![expedition_member(1, 6), expedition_member(2, 3)];

    assert_eq!(expedition_target_floor(&state, &members), 6);
}

#[test]
fn realm_threat_pushes_parties_toward_the_deep_core() {
    let mut calm = GameState::new();
    calm.total_floors = 10;
    let members = vec![expedition_member(1, 3)];
    let calm_target = expedition_target_floor(&calm, &members);

    calm.total_deaths = 50;
    let desperate_target = expedition_target_floor(&calm, &members);

    assert_eq!(calm_target, 3);
    assert_eq!(desperate_target, 6);
}

#[test]
fn expedition_targets_never_exceed_the_built_dungeon() {
    let mut state = GameState::new();
    state.total_floors = 3;
    state.total_deaths = state.siege_threshold();
    state.prestige = 20;
    state.known_adventurers.push(veteran_record(1, 10, 20));
    let members = vec![expedition_member(1, 10)];

    assert_eq!(expedition_target_floor(&state, &members), 3);
}

#[test]
fn discovery_drives_an_expedition_one_floor_deeper() {
    let mut state = GameState::new();
    state.total_floors = 10;
    let duty = vec![expedition_member(1, 3)];
    let mut discovery = duty.clone();
    discovery[0].drive = HeroDrive::Discovery;

    assert_eq!(
        expedition_target_floor(&state, &discovery),
        expedition_target_floor(&state, &duty) + 1
    );
}

fn settle_survivor(hp: i32, drive: HeroDrive) -> HeroRecord {
    let mut state = GameState::new();
    let mut record = veteran_record(1, 3, 2);
    record.drive = drive;
    record.resolve = 50;
    state.known_adventurers.push(record);

    let mut member = expedition_member(1, 3);
    member.drive = drive;
    member.hp = hp;
    member.max_hp = 100;
    state.adventurer_parties.push(AdventurerParty {
        id: 90,
        members: vec![member],
        current_floor: 3,
        current_room: 0,
        retreating: true,
        casualties: 0,
        loot: 30,
        entry_time: 0,
        target_floor: 3,
        snared_ticks: 0,
        alarmed: false,
        sieging: false,
        prev_room: 0,
        move_anim: Cooldown::new(PARTY_MOVE_SECONDS),
    });

    settle_departing_party(&mut state, 0);
    state.known_adventurers.remove(0)
}

#[test]
fn survival_hardens_confident_heroes_but_grievous_wounds_shake_them() {
    let healthy = settle_survivor(80, HeroDrive::Duty);
    let wounded = settle_survivor(20, HeroDrive::Duty);

    assert_eq!(healthy.resolve, 55);
    assert_eq!(wounded.resolve, 42);
    assert!(healthy.escapes > 0);
    assert_eq!(healthy.deepest_floor, 3);
}

#[test]
fn discovery_heroes_learn_more_from_the_same_escape() {
    let duty = settle_survivor(80, HeroDrive::Duty);
    let discovery = settle_survivor(80, HeroDrive::Discovery);
    assert!(discovery.experience > duty.experience);
}
