use crate::game_state::{Room, RoomType, Stats};
use serde::Deserialize;

// ===== JSON Data Structures =====

#[derive(Debug, Deserialize)]
struct DungeonConstants {
    max_rooms_per_floor: usize,
    max_log_entries: usize,
    base_room_cost: i32,
    boss_room_extra_cost: i32,
    core_room_mana_bonus: f32,
    /// Slots a boss room gains (or, negative, gives up) against its floor's
    /// capacity — the throne room trades numbers for the boss itself.
    boss_room_capacity_delta: i32,
}

#[derive(Debug, Deserialize)]
struct TimeConstants {
    mana_regen_interval: f32,
    time_advance_interval_ms: f32,
    /// Mana per hour the core draws from each living intruder, before level.
    mana_regen_per_adventurer: f32,
    /// Additional mana per hour per level of that intruder.
    mana_regen_per_adventurer_level: f32,
}

#[derive(Debug, Deserialize)]
struct AdventurerConstants {
    spawn_chance: f32,
    max_party_size: usize,
    min_party_size: usize,
    retreat_threshold: i32,
}

#[derive(Debug, Deserialize)]
struct CombatConstants {
    boss_room_loot_multiplier: f32,
    boss_stat_multiplier: f32,
    level_scaling_formula: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FloorScaling {
    pub floor: i32,
    pub mana_cost_multiplier: f32,
    pub monster_boost: i32,
    pub adventurer_level_min: i32,
    pub adventurer_level_max: i32,
    /// Defenders a room on this floor can hold.
    pub monster_capacity: i32,
}

#[derive(Debug, Deserialize)]
pub struct DeepFloorScaling {
    pub mana_cost_multiplier_increase: f32,
    pub monster_boost_increase: i32,
    /// Extra defender slots per floor beyond the scaling table.
    pub monster_capacity_increase: i32,
    pub adventurer_level_increase: i32,
}

#[derive(Debug, Deserialize)]
struct ConstantsData {
    dungeon: DungeonConstants,
    time: TimeConstants,
    adventurers: AdventurerConstants,
    combat: CombatConstants,
    floor_scaling: Vec<FloorScaling>,
    deep_floor_scaling: DeepFloorScaling,
}

// Embed JSON at compile time
const CONSTANTS_JSON: &str = include_str!("../../assets/constants.json");

fn load_constants() -> ConstantsData {
    serde_json::from_str(CONSTANTS_JSON).expect("Failed to parse constants.json")
}

// ===== Public Accessors =====

pub fn max_rooms_per_floor() -> usize {
    load_constants().dungeon.max_rooms_per_floor
}

pub fn max_log_entries() -> usize {
    load_constants().dungeon.max_log_entries
}

pub fn base_room_cost() -> i32 {
    load_constants().dungeon.base_room_cost
}

pub fn boss_room_extra_cost() -> i32 {
    load_constants().dungeon.boss_room_extra_cost
}

pub fn core_room_mana_bonus() -> f32 {
    load_constants().dungeon.core_room_mana_bonus
}

/// Mana per hour each living intruder feeds the core, before their level.
pub fn mana_regen_per_adventurer() -> f32 {
    load_constants().time.mana_regen_per_adventurer
}

/// Extra mana per hour per level of a living intruder.
pub fn mana_regen_per_adventurer_level() -> f32 {
    load_constants().time.mana_regen_per_adventurer_level
}

pub fn adventurer_spawn_chance() -> f32 {
    load_constants().adventurers.spawn_chance
}

pub fn max_party_size() -> usize {
    load_constants().adventurers.max_party_size
}

pub fn min_party_size() -> usize {
    load_constants().adventurers.min_party_size
}

pub fn retreat_threshold() -> i32 {
    load_constants().adventurers.retreat_threshold
}

pub fn boss_stat_multiplier() -> f32 {
    load_constants().combat.boss_stat_multiplier
}

pub fn level_scaling_formula() -> f32 {
    load_constants().combat.level_scaling_formula
}

pub fn get_floor_scaling(floor: i32) -> Option<FloorScaling> {
    let data = load_constants();
    data.floor_scaling.into_iter().find(|s| s.floor == floor)
}

pub fn get_deep_floor_scaling() -> DeepFloorScaling {
    load_constants().deep_floor_scaling
}

// ===== Calculation Functions =====

/// Calculate room cost based on total rooms built
pub fn get_room_cost(total_room_count: i32, is_boss: bool) -> i32 {
    let base = base_room_cost();
    let boss_extra = boss_room_extra_cost();
    let linear_increase = total_room_count * 5;
    let mut total = base + linear_increase;
    if is_boss {
        total += boss_extra;
    }
    ((total / 5) * 5).max(5)
}

/// Scale monster stats based on floor depth and boss status
pub fn get_scaled_stats(base: Stats, floor: i32, is_boss: bool) -> Stats {
    let floor_mult = 1.0 + ((floor - 1) as f32 * level_scaling_formula());
    let boss_mult = if is_boss { boss_stat_multiplier() } else { 1.0 };
    Stats {
        hp: (base.hp as f32 * floor_mult * boss_mult) as i32,
        attack: (base.attack as f32 * floor_mult * boss_mult) as i32,
        defense: (base.defense as f32 * floor_mult * boss_mult) as i32,
    }
}

/// Calculate monster mana cost based on floor and room type
pub fn get_monster_mana_cost(base_cost: i32, floor_number: i32, is_boss_room: bool) -> i32 {
    let floor_mult = get_floor_scaling(floor_number)
        .map(|s| s.mana_cost_multiplier)
        .unwrap_or(1.0 + (floor_number - 1) as f32 * 0.2);
    let boss_mult = if is_boss_room { 2.0 } else { 1.0 };
    (base_cost as f32 * floor_mult * boss_mult) as i32
}

/// How many defenders a room on this floor can hold. Slots are the scarce
/// resource that makes building a decision: a shallow room fields a couple of
/// bodies, and only depth buys room for more. Floors past the scaling table
/// keep growing at `monster_capacity_increase` per floor. Boss rooms apply
/// `boss_room_capacity_delta` — the throne trades numbers for the boss.
pub fn room_monster_capacity(floor: i32, is_boss: bool) -> usize {
    let data = load_constants();
    let table_capacity = data
        .floor_scaling
        .iter()
        .find(|s| s.floor == floor)
        .map(|s| s.monster_capacity)
        .or_else(|| {
            // Beyond the table: extrapolate from its deepest entry.
            data.floor_scaling
                .iter()
                .max_by_key(|s| s.floor)
                .map(|last| {
                    let beyond = (floor - last.floor).max(0);
                    last.monster_capacity
                        + data.deep_floor_scaling.monster_capacity_increase * beyond
                })
        })
        .unwrap_or(1);

    let delta = if is_boss {
        data.dungeon.boss_room_capacity_delta
    } else {
        0
    };
    (table_capacity + delta).max(1) as usize
}

/// Defender slots in this specific room.
pub fn room_capacity(room: &Room) -> usize {
    room_monster_capacity(room.floor_number, room.room_type == RoomType::Boss)
}

/// Whether the room has no free slot left for another defender. Over-capacity
/// rooms from older saves read as full rather than being trimmed.
pub fn room_is_full(room: &Room) -> bool {
    room.monsters.len() >= room_capacity(room)
}

/// Get adventurer level range for a floor
pub fn get_adventurer_level_range(floor: i32) -> (i32, i32) {
    get_floor_scaling(floor)
        .map(|s| (s.adventurer_level_min, s.adventurer_level_max))
        .unwrap_or((1, 3))
}

// Legacy constant aliases for backwards compatibility
pub const MAX_ROOMS_PER_FLOOR: usize = 5;
pub const MAX_LOG_ENTRIES: usize = 50;
pub const ADVENTURER_SPAWN_CHANCE: f32 = 0.3;
pub const MAX_PARTY_SIZE: usize = 4;
pub const MIN_PARTY_SIZE: usize = 2;
pub const RETREAT_THRESHOLD: i32 = 2;
pub const CORE_ROOM_MANA_BONUS: f32 = 0.1;
