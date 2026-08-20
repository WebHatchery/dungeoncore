use super::*;
use crate::game_state::{Adventurer, AdventurerParty, Stats};

fn party_with(hp: i32, count: usize, floor: i32, room: usize) -> AdventurerParty {
    let members = (0..count as u64)
        .map(|i| Adventurer {
            id: 10 + i,
            name: format!("Hero{i}"),
            class_name: "Warrior".to_string(),
            race: "Human".to_string(),
            drive: crate::game_state::HeroDrive::Duty,
            resolve: 50,
            ward: Default::default(),
            level: 2,
            hp,
            max_hp: hp,
            alive: true,
            experience: 0,
            gold: 0,
            equipment: Default::default(),
            conditions: Vec::new(),
            scaled_stats: Stats {
                hp,
                attack: 5,
                defense: 2,
            },
        })
        .collect();
    AdventurerParty {
        id: 1,
        members,
        current_floor: floor,
        current_room: room,
        retreating: false,
        casualties: 0,
        loot: 0,
        entry_time: 0,
        target_floor: floor,
        snared_ticks: 0,
        alarmed: false,
        sieging: false,
        prev_room: 0,
        move_anim: Cooldown::new(crate::game_state::PARTY_MOVE_SECONDS),
    }
}

#[test]
fn smite_damages_and_costs_mana_and_sets_cooldown() {
    let mut s = GameState::new();
    s.mana = 100;
    s.adventurer_parties.push(party_with(999, 2, 1, 0));
    cast_core_smite(&mut s).unwrap();
    assert!(!s.core_smite_cooldown.is_ready());
    assert_eq!(s.mana, 100 - CORE_SMITE_MANA_COST);
    // Both members took the hit but survived their large HP pool.
    assert!(s.adventurer_parties[0].members.iter().all(|m| m.alive));
    assert!(s.adventurer_parties[0].members[0].hp < 999);
}

#[test]
fn tide_lens_makes_the_core_smite_cheaper() {
    let mut state = GameState::new();
    assert_eq!(smite_mana_cost(&state), CORE_SMITE_MANA_COST);
    state.depth_relics.push("tide_lens".to_string());
    assert_eq!(smite_mana_cost(&state), CORE_SMITE_MANA_COST - 8);
}

#[test]
fn smite_wipe_retreats_party_and_pays_mana() {
    let mut s = GameState::new();
    s.mana = 100;
    s.max_mana = 999;
    s.adventurer_parties.push(party_with(1, 3, 1, 0));
    let deaths_before = s.total_deaths;
    cast_core_smite(&mut s).unwrap();
    assert!(s.adventurer_parties[0].retreating);
    assert!(s.adventurer_parties[0].members.iter().all(|m| !m.alive));
    assert_eq!(s.total_deaths, deaths_before + 3);
}

#[test]
fn smite_blocked_while_recharging_and_without_target() {
    let mut s = GameState::new();
    s.mana = 100;
    // No party present → no target.
    assert!(cast_core_smite(&mut s).is_err());
    // On cooldown → blocked even with a target.
    s.adventurer_parties.push(party_with(999, 1, 1, 0));
    s.core_smite_cooldown = Cooldown::new_armed(5.0);
    assert!(cast_core_smite(&mut s).is_err());
}

#[test]
fn smite_targets_deepest_party() {
    let mut s = GameState::new();
    s.adventurer_parties.push(party_with(999, 1, 1, 0));
    s.adventurer_parties.push(party_with(999, 1, 2, 1));
    assert_eq!(smite_target(&s), Some(1));
}
