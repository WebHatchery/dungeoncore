use super::*;

fn expedition_member(id: u64, level: i32) -> Adventurer {
    build_adventurer(id, format!("Hero {id}"), "Warrior", "Human", level, 1.0)
}

fn veteran_record(id: u64, level: i32, delves: i32) -> HeroRecord {
    HeroRecord {
        id,
        name: format!("Hero {id}"),
        class_name: "Warrior".to_string(),
        race: "Human".to_string(),
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
