//! Screenshot-capture scene seeding. Extracted from `main` (which had grown
//! past the file-size limit); used only by the headless capture harness to boot
//! a representative, frozen scene for a PNG. None of this runs in normal play.

use macroquad_toolkit::timing::Cooldown;

use crate::game_state::{self, GameState, PARTY_MOVE_SECONDS};
use crate::simulation;

mod deep_board;

/// First species flagged as a starter, used to seed capture scenes.
pub fn first_starter_species() -> Option<String> {
    crate::data::monsters::get_all_species()
        .into_iter()
        .find(|species| species.starter)
        .map(|species| species.name)
}

/// First combat-capable room (Normal or Boss) in the dungeon.
pub fn find_combat_room(state: &GameState) -> Option<(i32, usize)> {
    for floor in &state.floors {
        for room in &floor.rooms {
            if room.room_type == game_state::RoomType::Normal
                || room.room_type == game_state::RoomType::Boss
            {
                return Some((room.floor_number, room.position));
            }
        }
    }
    None
}

/// Seed `state` into a representative scene for a screenshot. Scenes:
/// `species` (starter-race modal), `tutorial` (onboarding overlay), and
/// `gameplay` (default: a mid-raid dungeon showing icons, effects, threat, log).
pub fn seed_capture_scene(state: &mut GameState, scene: &str) {
    use crate::game_state::{
        Adventurer, AdventurerParty, DungeonStatus, EffectAnchor, EffectKind, Equipment, LogEntry,
        Monster, Stats,
    };

    state.mana = 999;
    state.max_mana = 999;
    state.gold = 500;

    match scene {
        "deep_board" => deep_board::seed(state),
        "combat_sprites" => {
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
                state.push_effect_at(
                    floor,
                    pos,
                    "",
                    EffectKind::HitSpark,
                    EffectAnchor::Defenders,
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
        "species" => {
            state.unlocked_species.clear();
            state.unlocked_monsters.clear();
        }
        "tutorial" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            // Mid-tutorial: a room and defender are down, now learning elements.
            let _ = simulation::add_room(state, None);
            let monster = state.unlocked_monsters.first().cloned();
            if let (Some(monster), Some((floor, pos))) = (monster, find_combat_room(state)) {
                let _ = simulation::place_monster(state, floor, pos, &monster);
            }
            state.tutorial_active = true;
            state.tutorial_step = 2;
            state.status = DungeonStatus::Closed;
        }
        "placement" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            let _ = simulation::add_room(state, None);
            // Attune the first combat room to Fire so the synergy hint shows.
            if let Some((floor, pos)) = find_combat_room(state) {
                if let Some(f) = state.floors.iter_mut().find(|f| f.number == floor) {
                    if let Some(r) = f.rooms.iter_mut().find(|r| r.position == pos) {
                        r.upgrades.push(game_state::RoomUpgrade {
                            upgrade_type: game_state::RoomUpgradeType::Attunement,
                            name: "Fire Shrine".to_string(),
                            effect: "Fire attunement".to_string(),
                            multiplier: 1.3,
                            element: Some("Fire".to_string()),
                            effect_kind: String::new(),
                            disarmed: false,
                        });
                    }
                }
            }
            state.status = DungeonStatus::Closed;
            // The player is mid-placement with a Fire monster selected.
            state.selected_monster = Some("Ember Wisp".to_string());
        }
        "transit" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            let _ = simulation::add_room(state, None);
            let _ = simulation::add_room(state, None);
            let monster = state.unlocked_monsters.first().cloned();
            if let (Some(monster), Some((floor, pos))) = (monster, find_combat_room(state)) {
                let _ = simulation::place_monster(state, floor, pos, &monster);
            }
            state.status = DungeonStatus::Open;
            state.total_deaths = 14;
            // A party frozen mid-corridor between the entrance and room 1.
            let members = (0..3u64)
                .map(|i| Adventurer {
                    id: 200 + i,
                    name: ["Dain", "Eara", "Fitz"][i as usize].to_string(),
                    class_name: "Ranger".to_string(),
                    race: "Elf".to_string(),
                    level: 2,
                    hp: 34,
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
                })
                .collect();
            state.adventurer_parties.push(AdventurerParty {
                id: 1,
                members,
                current_floor: 1,
                current_room: 1,
                retreating: false,
                casualties: 0,
                loot: 0,
                entry_time: 6,
                target_floor: 1,
                snared_ticks: 0,
                alarmed: false,
                sieging: false,
                prev_room: 0,
                // Half-way through the glide (progress = 1 - 0.3/0.6 = 0.5).
                move_anim: {
                    let mut glide = Cooldown::new_armed(PARTY_MOVE_SECONDS);
                    glide.tick(PARTY_MOVE_SECONDS - 0.3);
                    glide
                },
            });
        }
        "coretree" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            // A few prestiges in: souls to spend and an economy line partly
            // awakened, so the tree shows owned / available / locked states.
            state.prestige = 3;
            state.souls = 30;
            let _ = simulation::endgame::buy_core_power(state, "deep_roots");
            let _ = simulation::endgame::buy_core_power(state, "dread_aura");
            let _ = simulation::endgame::buy_core_power(state, "wellspring");
            let _ = simulation::endgame::buy_core_power(state, "searing_smite");
        }
        "build" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            // Surplus gold and headroom in mana so the Channel-the-Hoard sink is
            // live, and a couple of prestiges' worth of core-power progress.
            state.gold = 640;
            state.mana = 120;
            state.souls = 6;
            state.prestige = 2;
            let _ = simulation::add_room(state, None);
        }
        "variants" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            state.mana = 400;
            let _ = simulation::add_room(state, None);
            // Two lines fielded, each partway to its next variant, so the tab
            // shows pooled progress rather than per-creature XP.
            if let Some((floor, pos)) = find_combat_room(state) {
                let _ = simulation::place_monster(state, floor, pos, "Goblin");
                let _ = simulation::place_monster(state, floor, pos, "Goblin Archer");
            }
            state.add_type_experience("Goblin", 32);
            state.add_type_experience("Goblin Archer", 11);
        }
        "defenders" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            state.mana = 400;
            let _ = simulation::add_room(state, None);
            if let Some((floor, pos)) = find_combat_room(state) {
                let _ = simulation::place_monster(state, floor, pos, "Goblin");
                let _ = simulation::place_monster(state, floor, pos, "Goblin Archer");
                // A third body puts the room over its two-slot budget, which is
                // exactly what a save written before the limit looks like: the
                // rows must render it rather than hide it.
                let _ = simulation::place_monster(state, floor, pos, "Goblin Shaman");
                if let Some(f) = state.floors.iter_mut().find(|f| f.number == floor) {
                    if let Some(r) = f.rooms.iter_mut().find(|r| r.position == pos) {
                        r.monsters.push(game_state::Monster {
                            id: 940,
                            type_name: "Goblin Shaman".to_string(),
                            hp: 6,
                            max_hp: 22,
                            alive: true,
                            is_boss: false,
                            scaled_stats: game_state::Stats {
                                hp: 22,
                                attack: 5,
                                defense: 1,
                            },
                            active_traits: Vec::new(),
                        });
                        // One wounded, one fallen, one whole.
                        if let Some(m) = r.monsters.get_mut(1) {
                            m.hp = m.max_hp / 2;
                        }
                        if let Some(m) = r.monsters.first_mut() {
                            m.alive = false;
                            m.hp = 0;
                        }
                    }
                }
                state.selected_room = Some((floor, pos));
            }
            state.add_type_experience("Goblin", 32);
        }
        "swap" => {
            // A goblin and a slime sharing a room, with an Orc armed: the goblin
            // row offers an upgrade along its own line, the slime row an
            // eviction, and each states its own price.
            for species in ["Goblinoid", "Slime"] {
                let _ = simulation::unlock_species(state, species);
            }
            state.tutorial_active = false;
            state.mana = 400;
            state.gold = 500;
            let _ = simulation::add_room(state, None);
            if let Some((floor, pos)) = find_combat_room(state) {
                let _ = simulation::place_monster(state, floor, pos, "Goblin");
                let _ = simulation::place_monster(state, floor, pos, "Green Slime");
                state.selected_room = Some((floor, pos));
            }
            if !state.unlocked_monsters.iter().any(|m| m == "Orc") {
                state.unlocked_monsters.push("Orc".to_string());
            }
            state.add_type_experience("Goblin", 50);
            state.selected_monster = Some("Orc".to_string());
        }
        "traps" => {
            // One room already trapped, one not, with a trap armed: the empty
            // room lights up to receive it and the trapped one says it already
            // has one of these.
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            state.mana = 400;
            let _ = simulation::add_room(state, None);
            let _ = simulation::add_room(state, None);
            if let Some((floor, pos)) = find_combat_room(state) {
                let _ = simulation::apply_upgrade(state, floor, pos, "Spike Trap");
                state.selected_room = Some((floor, pos));
            }
            state.selected_upgrade = Some("Poison Dart".to_string());
        }
        "journal" => {
            // A rival's page: profile, bounty, and the history the dungeon has
            // watched her accumulate across several delves.
            use crate::game_state::{HeroRecord, HeroStatus};
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            state.day = 14;
            let mut sable = HeroRecord {
                id: 500,
                name: "Sable the Bold".to_string(),
                class_name: "Rogue".to_string(),
                race: "Halfling".to_string(),
                level: 5,
                experience: 30,
                delves: 5,
                kills: 12,
                gold_stolen: 240,
                status: HeroStatus::Alive,
                death_floor: 0,
                death_day: 0,
                journal: Vec::new(),
            };
            for (day, text) in [
                (2, "First delve into the dungeon"),
                (2, "Slew a Goblin on floor 1"),
                (3, "Escaped with 40 gold"),
                (6, "Returned for delve 2"),
                (6, "Slew a Goblin Archer on floor 2"),
                (7, "Escaped with 85 gold"),
                (9, "Reached level 4"),
                (11, "Returned for delve 4"),
                (12, "Slew an Orc on floor 2"),
                (13, "Escaped with 115 gold"),
                (14, "Returned for delve 5"),
            ] {
                sable.remember(day, text);
            }
            state.known_adventurers = vec![sable];
            state.selected_hero = Some(500);
        }
        "rival" => {
            use crate::game_state::{Equipment, HeroRecord, HeroStatus};
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            let _ = simulation::add_room(state, None);
            let monster = state.unlocked_monsters.first().cloned();
            if let (Some(monster), Some((floor, pos))) = (monster, find_combat_room(state)) {
                let _ = simulation::place_monster(state, floor, pos, &monster);
            }
            state.status = DungeonStatus::Open;
            state.total_deaths = 20;
            // A veteran rival (5 delves, 12 kills) leads a fresh recruit into a
            // defended room, so the gold ring + name plate + RIVAL badge show.
            state.known_adventurers = vec![
                HeroRecord {
                    id: 500,
                    name: "Sable the Bold".to_string(),
                    class_name: "Rogue".to_string(),
                    race: "Halfling".to_string(),
                    level: 5,
                    experience: 0,
                    delves: 5,
                    kills: 12,
                    gold_stolen: 240,
                    status: HeroStatus::Inside,
                    death_floor: 0,
                    death_day: 0,
                    journal: Vec::new(),
                },
                HeroRecord {
                    id: 501,
                    name: "Pip".to_string(),
                    class_name: "Warrior".to_string(),
                    race: "Human".to_string(),
                    level: 2,
                    experience: 0,
                    delves: 1,
                    kills: 0,
                    gold_stolen: 0,
                    status: HeroStatus::Inside,
                    death_floor: 0,
                    death_day: 0,
                    journal: Vec::new(),
                },
            ];
            if let Some((floor, pos)) = find_combat_room(state) {
                let mk = |id: u64, name: &str, class: &str, hp: i32| Adventurer {
                    id,
                    name: name.to_string(),
                    class_name: class.to_string(),
                    race: "Human".to_string(),
                    level: 4,
                    hp,
                    max_hp: 50,
                    alive: true,
                    experience: 0,
                    gold: 0,
                    equipment: Equipment::default(),
                    conditions: Vec::new(),
                    scaled_stats: Stats {
                        hp: 50,
                        attack: 10,
                        defense: 4,
                    },
                };
                state.adventurer_parties.push(AdventurerParty {
                    id: 1,
                    members: vec![
                        mk(500, "Sable the Bold", "Rogue", 38),
                        mk(501, "Pip", "Warrior", 44),
                    ],
                    current_floor: floor,
                    current_room: pos,
                    retreating: false,
                    casualties: 0,
                    loot: 60,
                    entry_time: 8,
                    target_floor: 1,
                    snared_ticks: 0,
                    alarmed: false,
                    sieging: false,
                    prev_room: 0,
                    move_anim: Cooldown::new(PARTY_MOVE_SECONDS),
                });
                state.selected_room = Some((floor, pos));
            }
        }
        "goals" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            // A run several prestiges deep with a spread of milestones earned.
            state.prestige = 4;
            state.raids_completed = 18;
            state.total_floors = 4;
            let _ = simulation::add_room(state, None);
            let _ = simulation::endgame::buy_core_power(state, "deep_roots");
            simulation::milestones::check_milestones(state);
        }
        "siege" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            let _ = simulation::add_room(state, None);
            let monster = state.unlocked_monsters.first().cloned();
            if let (Some(monster), Some((floor, pos))) = (monster, find_combat_room(state)) {
                let _ = simulation::place_monster(state, floor, pos, &monster);
            }
            // Peak threat with the dungeon clear musters a real siege party.
            state.total_deaths = 100;
            simulation::endgame::maybe_launch_siege(state);
            state.core_hp = 380;
            state.core_max_hp = 500;
        }
        "summary" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            let _ = simulation::add_room(state, None);
            let monster = state.unlocked_monsters.first().cloned();
            if let (Some(monster), Some((floor, pos))) = (monster, find_combat_room(state)) {
                let _ = simulation::place_monster(state, floor, pos, &monster);
            }
            state.status = DungeonStatus::Open;
            state.total_deaths = 14;
            // A concluded raid, so the post-raid summary card is on screen.
            state.last_raid_summary = Some(game_state::RaidSummary {
                outcome: game_state::RaidOutcome::Wiped,
                party_size: 4,
                slain: 4,
                survivors: 0,
                mana_gained: 60,
                mana_recovery_cost: 15,
                souls_gained: 1,
                gold_gained: 0,
                defenders_lost: 1,
                reputation_change: -12,
                reputation_after: -12,
            });
        }
        _ => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;

            // Build a couple of combat rooms.
            let _ = simulation::add_room(state, None);
            let _ = simulation::add_room(state, None);

            // Place defenders in the first combat room.
            let monster = state.unlocked_monsters.first().cloned();
            if let (Some(monster), Some((floor, pos))) = (monster, find_combat_room(state)) {
                for _ in 0..3 {
                    let _ = simulation::place_monster(state, floor, pos, &monster);
                }
            }

            state.status = DungeonStatus::Open;
            state.total_deaths = 14; // -> "Wary" threat tier

            // Drop an adventuring party into the defended room for a live fight.
            if let Some((floor, pos)) = find_combat_room(state) {
                let members = (0..3u64)
                    .map(|i| Adventurer {
                        id: 100 + i,
                        name: ["Aldric", "Bryn", "Cael"][i as usize].to_string(),
                        class_name: "Warrior".to_string(),
                        race: "Human".to_string(),
                        level: 2,
                        hp: 30,
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
                    })
                    .collect();
                state.adventurer_parties.push(AdventurerParty {
                    id: 1,
                    members,
                    current_floor: floor,
                    current_room: pos,
                    retreating: false,
                    casualties: 1,
                    loot: 40,
                    entry_time: 8,
                    target_floor: 1,
                    snared_ticks: 0,
                    alarmed: false,
                    sieging: false,
                    prev_room: 0,
                    move_anim: Cooldown::new(PARTY_MOVE_SECONDS),
                });

                // Both sides trading blows: defenders take a strong hit on the
                // left, the party takes damage and loses one on the right.
                use game_state::EffectAnchor;
                state.push_effect_at(
                    floor,
                    pos,
                    "Strong hit!",
                    EffectKind::Ability,
                    EffectAnchor::Defenders,
                );
                state.push_effect_at(
                    floor,
                    pos,
                    "-12",
                    EffectKind::Damage,
                    EffectAnchor::Invaders,
                );
                state.push_effect_at(
                    floor,
                    pos,
                    "Slain!",
                    EffectKind::AdventurerDown,
                    EffectAnchor::Invaders,
                );

                // Show the room inspector (defender list + upgrade catalog).
                state.selected_room = Some((floor, pos));
            }

            // Seed the hero ledger so the HEROES tab has content to show.
            use game_state::{HeroRecord, HeroStatus};
            let seed_hero = |id,
                             name: &str,
                             class: &str,
                             race: &str,
                             level,
                             delves,
                             kills,
                             gold,
                             status,
                             df,
                             dd| HeroRecord {
                id,
                name: name.to_string(),
                class_name: class.to_string(),
                race: race.to_string(),
                level,
                experience: 0,
                delves,
                kills,
                gold_stolen: gold,
                status,
                death_floor: df,
                death_day: dd,
                journal: Vec::new(),
            };
            state.known_adventurers = vec![
                seed_hero(
                    100,
                    "Aldric",
                    "Warrior",
                    "Human",
                    2,
                    1,
                    0,
                    0,
                    HeroStatus::Inside,
                    0,
                    0,
                ),
                seed_hero(
                    101,
                    "Bryn",
                    "Warrior",
                    "Dwarf",
                    2,
                    1,
                    0,
                    0,
                    HeroStatus::Inside,
                    0,
                    0,
                ),
                seed_hero(
                    200,
                    "Sable",
                    "Rogue",
                    "Halfling",
                    4,
                    5,
                    12,
                    180,
                    HeroStatus::Alive,
                    0,
                    0,
                ),
                seed_hero(
                    201,
                    "Wren",
                    "Ranger",
                    "Elf",
                    3,
                    3,
                    6,
                    90,
                    HeroStatus::Alive,
                    0,
                    0,
                ),
                seed_hero(
                    300,
                    "Mordred",
                    "Mage",
                    "Human",
                    2,
                    2,
                    3,
                    40,
                    HeroStatus::Dead,
                    2,
                    3,
                ),
            ];

            state.add_log(LogEntry::adventure(
                "New adventurer party enters! (3 members)",
            ));
            state.add_log(LogEntry::combat(
                "Goblin uses Ambush! Dealt 12 damage to 3 adventurers.",
            ));
            state.add_log(LogEntry::combat(
                "Bryn has fallen on floor 1! +20 mana, +10 XP to monsters",
            ));
            state.add_log(LogEntry::building("Spawned defender on floor 1, room 1."));
        }
    }
}
