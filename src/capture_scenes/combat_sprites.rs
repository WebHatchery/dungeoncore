//! Frozen combat-art proof scene for the headless capture harness.

use macroquad_toolkit::timing::Cooldown;

use crate::game_state::{
    Adventurer, AdventurerParty, EffectAnchor, EffectKind, Equipment, Monster, Stats,
    PARTY_MOVE_SECONDS,
};

use super::{find_combat_room, seed_capture_scene};

pub(super) fn seed(state: &mut crate::game_state::GameState) {
    // Reuse the representative raid, then widen it into a focused art
    // proof: different silhouettes, wounds, a central dust cloud,
    // and another party frozen in transit.
    seed_capture_scene(state, "gameplay");
    if let Some((floor, pos)) = find_combat_room(state) {
        if let Some(room) = state
            .floors
            .iter_mut()
            .find(|floor_data| floor_data.number == floor)
            .and_then(|floor_data| {
                floor_data
                    .rooms
                    .iter_mut()
                    .find(|room| room.position == pos)
            })
        {
            room.monsters.clear();
            room.monsters.push(Monster {
                id: 901,
                type_name: "Goblin".to_string(),
                hp: 20,
                max_hp: 20,
                alive: true,
                is_boss: false,
                scaled_stats: Stats {
                    hp: 20,
                    attack: 5,
                    defense: 2,
                },
                active_traits: Vec::new(),
            });
            room.monsters.push(Monster {
                id: 902,
                type_name: "Skeleton".to_string(),
                hp: 11,
                max_hp: 28,
                alive: true,
                is_boss: false,
                scaled_stats: Stats {
                    hp: 28,
                    attack: 8,
                    defense: 2,
                },
                active_traits: Vec::new(),
            });
            room.monsters.push(Monster {
                id: 904,
                type_name: "Dragon".to_string(),
                hp: 146,
                max_hp: 200,
                alive: true,
                is_boss: false,
                scaled_stats: Stats {
                    hp: 200,
                    attack: 30,
                    defense: 15,
                },
                active_traits: Vec::new(),
            });
            room.monsters.push(Monster {
                id: 903,
                type_name: "Green Slime".to_string(),
                hp: 0,
                max_hp: 26,
                alive: false,
                is_boss: false,
                scaled_stats: Stats {
                    hp: 26,
                    attack: 6,
                    defense: 1,
                },
                active_traits: Vec::new(),
            });
        }
        state.push_effect_at(floor, pos, "", EffectKind::MeleeDust, EffectAnchor::Center);
        state.push_element_effect_at(
            floor,
            pos,
            "",
            EffectKind::HitSpark,
            EffectAnchor::Defenders,
            "Fire",
        );
        state.push_effect_at(
            floor,
            pos,
            "Slain!",
            EffectKind::MonsterDown,
            EffectAnchor::Defenders,
        );
    }
    if let Some(party) = state.adventurer_parties.first_mut() {
        for (member, class) in party
            .members
            .iter_mut()
            .zip(["Cleric", "Ranger", "Paladin"])
        {
            member.class_name = class.to_string();
        }
    }
    let transit_member = Adventurer {
        id: 990,
        name: "Nia".to_string(),
        class_name: "Ranger".to_string(),
        race: "Elf".to_string(),
        drive: crate::game_state::HeroDrive::Discovery,
        resolve: 60,
        level: 3,
        hp: 26,
        max_hp: 40,
        alive: true,
        experience: 0,
        gold: 0,
        equipment: Equipment::default(),
        conditions: Vec::new(),
        scaled_stats: Stats {
            hp: 40,
            attack: 9,
            defense: 3,
        },
    };
    state.adventurer_parties.push(AdventurerParty {
        id: 99,
        members: vec![transit_member],
        current_floor: 1,
        current_room: 2,
        retreating: false,
        casualties: 0,
        loot: 0,
        entry_time: 0,
        target_floor: 1,
        snared_ticks: 0,
        alarmed: false,
        sieging: false,
        prev_room: 1,
        move_anim: Cooldown::new_armed(PARTY_MOVE_SECONDS),
    });
}
