//! Screenshot-capture scene seeding. Extracted from `main` (which had grown
//! past the file-size limit); used only by the headless capture harness to boot
//! a representative, frozen scene for a PNG. None of this runs in normal play.

use macroquad_toolkit::timing::Cooldown;

use crate::game_state::{self, GameState, PARTY_MOVE_SECONDS};
use crate::simulation;

mod bestiary;
mod combat_sprites;
mod deep_board;
mod depth;
mod heroes;
mod merge;
mod strata;
mod vfx;

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

/// Capture names may carry a viewport suffix while still using the same game
/// fixture. Keeping the suffix out of the fixture match lets one manifest
/// prove both desktop and narrow browser layouts.
pub fn base_scene(scene: &str) -> &str {
    scene.strip_suffix("_narrow").unwrap_or(scene)
}

/// Seed `state` into a representative scene for a screenshot. Scenes:
/// `species` (starter-race modal), `tutorial` (onboarding overlay), and
/// `gameplay` (default: a mid-raid dungeon showing icons, effects, threat, log).
pub fn seed_capture_scene(state: &mut GameState, scene: &str) {
    use crate::game_state::{
        Adventurer, AdventurerParty, DungeonStatus, EffectKind, Equipment, LogEntry, Stats,
    };

    state.mana = 999;
    state.max_mana = 999;
    state.gold = 500;

    let narrow = scene.ends_with("_narrow");
    if vfx::seed(state, base_scene(scene), narrow) {
        return;
    }
    match base_scene(scene) {
        "gameover" => {
            state.game_over = true;
            state.day = 47;
            state.prestige = 3;
            state.total_deaths = 186;
        }
        "deep_board" => deep_board::seed(state),
        "depth" => depth::seed(state),
        "strata" => strata::seed(state),
        "combat_sprites" => combat_sprites::seed(state),
        "bestiary" => bestiary::seed(state),
        "species" => {
            state.unlocked_species.clear();
            state.unlocked_monsters.clear();
        }
        "opening" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            state.status = DungeonStatus::Closed;
        }
        "confirmation" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            state.pending_confirmation = Some(game_state::PendingConfirmation::ResetRun);
        }
        "pause" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            let _ = simulation::add_room(state, None);
            state.paused = true;
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
            let _ = simulation::unlock_species(state, "Elemental");
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
                            secondary_effect: "Matching-element adventurers deal 15% more damage here".to_string(),
                            secondary_kind: "ElementalAdventurerAttack".to_string(),
                            secondary_value: 1.15,
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
                    drive: crate::game_state::HeroDrive::Discovery,
                    resolve: 55,
                    ward: Default::default(),
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
        "branching" => {
            if let Some(species) = first_starter_species() {
                let _ = simulation::unlock_species(state, &species);
            }
            state.tutorial_active = false;
            state.mana = 800;
            let _ = simulation::add_room(state, None);
            let _ = simulation::branch_from(state, 1, 0);
            let _ = simulation::add_room(state, None);
            state.status = DungeonStatus::Closed;
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
                        r.battle_order = game_state::RoomBattleOrder::CullWounded;
                        r.monsters.push(game_state::Monster {
                            id: 940,
                            type_name: "Goblin Shaman".to_string(),
                            hp: 6,
                            max_hp: 22,
                            alive: true,
                            is_boss: false,
                            fusion_rank: 1,
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
        "merge" => merge::seed(state),
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
        "journal" => heroes::seed_journal(state),
        "rival" => heroes::seed_rival(state),
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
            if narrow {
                state.board_pan_x = 340.0;
            }
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
                        drive: crate::game_state::HeroDrive::Duty,
                        resolve: 50,
                        ward: Default::default(),
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
                             level: i32,
                             delves: i32,
                             kills: i32,
                             gold: i32,
                             status,
                             df,
                             dd| HeroRecord {
                id,
                name: name.to_string(),
                class_name: class.to_string(),
                race: race.to_string(),
                drive: match class {
                    "Rogue" => crate::game_state::HeroDrive::Fortune,
                    "Mage" | "Ranger" => crate::game_state::HeroDrive::Discovery,
                    "Warrior" => crate::game_state::HeroDrive::Glory,
                    _ => crate::game_state::HeroDrive::Duty,
                },
                resolve: (50 + delves * 4).min(100),
                level,
                experience: 0,
                delves,
                kills,
                gold_stolen: gold,
                escapes: delves.saturating_sub(1),
                deepest_floor: if df > 0 { df } else { level.max(1) / 2 + 1 },
                insights: Vec::new(),
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
