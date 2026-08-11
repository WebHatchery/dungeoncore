use super::*;

#[test]
fn milestones_unlock_once_and_stay() {
    let mut s = GameState::new();
    assert_eq!(achieved_count(&s), 0);
    s.raids_completed = 10;
    let n = check_milestones(&mut s);
    // first_raid and veteran_keeper both clear at 10 raids.
    assert!(n >= 2);
    assert!(s.milestones.iter().any(|id| id == "first_raid"));
    assert!(s.milestones.iter().any(|id| id == "veteran_keeper"));
    // Re-checking unlocks nothing new, and a counter reset can't revoke it.
    assert_eq!(check_milestones(&mut s), 0);
    s.raids_completed = 0;
    assert!(s.milestones.iter().any(|id| id == "veteran_keeper"));
}

#[test]
fn every_milestone_condition_is_wired() {
    // A catalog entry with no matching arm in `met` would silently never
    // unlock — guard against that by construction.
    let mut s = GameState::new();
    s.raids_completed = 1_000;
    s.prestige = 1_000;
    s.total_floors = 1_000;
    for _ in 0..50 {
        s.core_powers.push("x".to_string());
        s.unlocked_monsters.push("m".to_string());
    }
    for i in 0..50 {
        s.known_adventurers.push(crate::game_state::HeroRecord {
            id: i,
            name: String::new(),
            class_name: String::new(),
            race: String::new(),
            level: 1,
            experience: 0,
            delves: 0,
            kills: 0,
            gold_stolen: 0,
            status: crate::game_state::HeroStatus::Alive,
            death_floor: 0,
            death_day: 0,
            journal: Vec::new(),
        });
    }
    for m in MILESTONES.iter() {
        assert!(met(&s, m.id), "milestone '{}' has no wired condition", m.id);
    }
}
