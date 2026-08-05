use macroquad_toolkit::rng::SeededRng;
use macroquad_toolkit::timing::Cooldown;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod effects;
mod floor;
pub(crate) mod heroes;
mod reputation;
pub use effects::{EffectAnchor, EffectKind, RoomEffect};
pub use floor::Floor;
pub use heroes::{Adventurer, AdventurerParty, Condition, Equipment, HeroRecord, HeroStatus};
pub use reputation::{ReputationBand, VisitorQuality, REPUTATION_MAX, REPUTATION_MIN};

/// A ready (zero-duration) cooldown, used as the `#[serde(skip)]` default for
/// transient fields — `Cooldown` has no `Default` impl of its own.
pub(crate) fn ready_cooldown() -> Cooldown {
    Cooldown::new(0.0)
}

fn legacy_run_seed() -> u64 {
    0
}

fn legacy_run_rng() -> SeededRng {
    SeededRng::new(legacy_run_seed())
}

/// Cumulative adventurer deaths that push the realm to peak threat (tier 4) and
/// muster a siege. Also the denominator of the HUD's "dread" progress meter.
pub const SIEGE_THREAT_DEATHS: i32 = 100;

/// Seconds a party spends visibly travelling the corridor between two rooms.
/// Comfortably shorter than the 2s combat tick so the glide always completes
/// before the party fights in its new room.
pub const PARTY_MOVE_SECONDS: f32 = 0.6;

/// Combat stats for monsters and adventurers
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
}

/// Room type enumeration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoomType {
    Entrance,
    Normal,
    Boss,
    Core,
}

/// Room upgrade type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoomUpgradeType {
    Trap,
    Treasure,
    Reinforcement,
    Evolution,
    Attunement,
}

/// Room upgrade applied to a room
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomUpgrade {
    pub upgrade_type: RoomUpgradeType,
    pub name: String,
    pub effect: String,
    pub multiplier: f32,
    /// Element this upgrade is keyed to (attunements, elemental traps)
    #[serde(default)]
    pub element: Option<String>,
    /// Trap behavior: "Damage", "Poison", "Burn", "Snare", "Alarm",
    /// "ManaSiphon", "GoldSteal". Empty = legacy flat-damage trap.
    #[serde(default)]
    pub effect_kind: String,
    /// A Rogue sprung this trap; it re-arms between raids (costs mana).
    #[serde(default)]
    pub disarmed: bool,
}

/// Active trait instance on a monster
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveTrait {
    pub id: String,
    pub name: String,
    pub cooldown_timer: i32,
}

/// Monster instance in a room
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Monster {
    pub id: u64,
    pub type_name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub alive: bool,
    pub is_boss: bool,
    pub scaled_stats: Stats,
    #[serde(default)]
    pub active_traits: Vec<ActiveTrait>,
}

/// Room in a dungeon floor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: u64,
    pub room_type: RoomType,
    /// Stable per-floor node key. No longer implies linear order — it is this
    /// room's identity within the floor and the endpoint used by `exits`.
    pub position: usize,
    /// Child rooms this room routes into (directed graph edges within the
    /// floor). The Entrance has >= 1 exit; a fork has >= 2 (up to 3); the Core
    /// sink has none. Empty on pre-graph saves; `GameState::migrate` rebuilds
    /// the linear chain from `position` order.
    #[serde(default)]
    pub exits: Vec<usize>,
    pub floor_number: i32,
    pub monsters: Vec<Monster>,
    /// Installed upgrades — at most one per RoomUpgradeType.
    #[serde(default)]
    pub upgrades: Vec<RoomUpgrade>,
    /// Legacy single-slot field; migrated into `upgrades` on load.
    #[serde(default, skip_serializing)]
    pub upgrade: Option<RoomUpgrade>,
    pub explored: bool,
    pub loot: i32,
}

impl Room {
    pub fn new(id: u64, room_type: RoomType, position: usize, floor_number: i32) -> Self {
        Self {
            id,
            room_type,
            position,
            exits: Vec::new(),
            floor_number,
            monsters: Vec::new(),
            upgrades: Vec::new(),
            upgrade: None,
            explored: false,
            loot: 0,
        }
    }

    /// The installed upgrade of a given type, if any.
    pub fn upgrade_of(&self, upgrade_type: RoomUpgradeType) -> Option<&RoomUpgrade> {
        self.upgrades
            .iter()
            .find(|u| u.upgrade_type == upgrade_type)
    }

    /// Whether the room already holds an upgrade of this type.
    pub fn has_upgrade_type(&self, upgrade_type: RoomUpgradeType) -> bool {
        self.upgrade_of(upgrade_type).is_some()
    }

    /// Get the trap damage multiplier (from trap upgrades)
    pub fn trap_multiplier(&self) -> f32 {
        self.upgrade_of(RoomUpgradeType::Trap)
            .map(|u| u.multiplier)
            .unwrap_or(1.0)
    }

    /// Get the treasure/loot multiplier
    pub fn treasure_multiplier(&self) -> f32 {
        self.upgrade_of(RoomUpgradeType::Treasure)
            .map(|u| u.multiplier)
            .unwrap_or(1.0)
    }

    /// Get monster stat boost from reinforcement
    pub fn reinforcement_multiplier(&self) -> f32 {
        self.upgrade_of(RoomUpgradeType::Reinforcement)
            .map(|u| u.multiplier)
            .unwrap_or(1.0)
    }

    /// Get XP multiplier from evolution upgrade
    pub fn evolution_multiplier(&self) -> f32 {
        self.upgrade_of(RoomUpgradeType::Evolution)
            .map(|u| u.multiplier)
            .unwrap_or(1.0)
    }

    /// Element attunement of this room: (element, stat multiplier for
    /// monsters of that element), if an attunement upgrade is installed.
    pub fn attunement(&self) -> Option<(&str, f32)> {
        self.upgrade_of(RoomUpgradeType::Attunement)
            .and_then(|u| u.element.as_deref().map(|e| (e, u.multiplier)))
    }
}

fn default_core_hp() -> i32 {
    500
}
/// Dungeon operational status
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DungeonStatus {
    Open,
    Closing,
    Closed,
}

/// How a raid ended, from the dungeon's point of view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RaidOutcome {
    /// No adventurer left the dungeon alive.
    Wiped,
    /// Survivors fled or escaped with loot.
    Repelled,
}

/// Running income tally for the active raid (transient). Snapshotted into a
/// [`RaidSummary`] when the party departs, then discarded.
#[derive(Clone, Debug, Default)]
pub struct RaidTally {
    pub mana_gained: i32,
    pub souls_gained: i32,
    pub gold_gained: i32,
    pub defenders_lost: i32,
}

/// The result of a concluded raid, shown to the player as a summary card until
/// dismissed or replaced by the next raid (transient — not persisted).
#[derive(Clone, Debug)]
pub struct RaidSummary {
    pub outcome: RaidOutcome,
    pub party_size: i32,
    pub slain: i32,
    pub survivors: i32,
    pub mana_gained: i32,
    /// Mana paid after the raid to restore dead defenders and sprung traps.
    pub mana_recovery_cost: i32,
    pub souls_gained: i32,
    pub gold_gained: i32,
    pub defenders_lost: i32,
    pub reputation_change: i32,
    pub reputation_after: i32,
}

/// A destructive action awaiting the player's explicit second choice. This is
/// UI state, so it is intentionally not written to a save file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PendingConfirmation {
    ResetRun,
    DismissMonster {
        floor: i32,
        room: usize,
        monster_id: u64,
    },
}

/// Log entry type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub message: String,
    pub log_type: String, // "system", "combat", "adventure", "building"
    pub timestamp: u64,
}

impl LogEntry {
    pub fn new(message: impl Into<String>, log_type: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            log_type: log_type.into(),
            timestamp: 0,
        }
    }

    pub fn system(message: impl Into<String>) -> Self {
        Self::new(message, "system")
    }

    pub fn combat(message: impl Into<String>) -> Self {
        Self::new(message, "combat")
    }

    pub fn adventure(message: impl Into<String>) -> Self {
        Self::new(message, "adventure")
    }

    pub fn building(message: impl Into<String>) -> Self {
        Self::new(message, "building")
    }
}

/// Main game state - mirrors GameState from types/game.ts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    // Resources
    pub mana: i32,
    pub max_mana: i32,
    pub mana_regen: f32,
    pub gold: i32,
    pub souls: i32,

    // Time
    pub day: i32,
    pub hour: i32,
    pub speed: i32,
    /// A deliberate player pause. Persisted so loading a paused dungeon never
    /// advances it before the player has a chance to review the board.
    #[serde(default)]
    pub paused: bool,

    /// The initial seed lets a keeper include a reproducible run identifier in
    /// a bug report. `run_rng` below carries the exact mid-run stream state.
    #[serde(default = "legacy_run_seed")]
    pub run_seed: u64,
    /// All gameplay randomness comes from this saved stream, so loading a run
    /// continues with the same future as the moment it was saved.
    #[serde(default = "legacy_run_rng")]
    pub run_rng: SeededRng,

    // Dungeon
    pub status: DungeonStatus,
    pub floors: Vec<Floor>,
    pub total_floors: i32,
    pub deep_core_bonus: f32,

    // Adventurers
    pub adventurer_parties: Vec<AdventurerParty>,
    /// Earliest spawn time for the next party, in absolute hours
    /// (day * 24 + hour) so it survives the midnight wrap.
    pub next_party_spawn: i32,
    /// Ledger of every hero who has ever entered the dungeon.
    #[serde(default)]
    pub known_adventurers: Vec<HeroRecord>,

    // Reputation / threat
    #[serde(default)]
    pub total_deaths: i32,
    #[serde(default)]
    pub threat_warned: i32,
    #[serde(default)]
    pub raids_completed: i32,
    /// What the realm expects from a visit. Unlike threat, it is earned from
    /// raid outcomes and changes who chooses to enter rather than siege timing.
    #[serde(default)]
    pub reputation: i32,

    // Endgame: the core, sieges, and prestige
    #[serde(default = "default_core_hp")]
    pub core_hp: i32,
    #[serde(default = "default_core_hp")]
    pub core_max_hp: i32,
    /// A tier-4 siege is currently marching / assaulting the core.
    #[serde(default)]
    pub siege_active: bool,
    /// Times the dungeon has repelled the realm's siege.
    #[serde(default)]
    pub prestige: i32,
    /// Permanent soul-bought core powers (ids).
    #[serde(default)]
    pub core_powers: Vec<String>,
    /// Ids of milestones the player has achieved (the goal/achievement track).
    #[serde(default)]
    pub milestones: Vec<String>,
    /// Chosen difficulty for this run (scales invaders, sieges, income, core HP).
    #[serde(default)]
    pub difficulty: crate::data::difficulty::Difficulty,
    /// Recharge on the active Core Smite lever. Transient — a fresh session
    /// always starts ready.
    #[serde(skip, default = "ready_cooldown")]
    pub core_smite_cooldown: Cooldown,
    /// The core has fallen; the run is over (not persisted meaningfully).
    #[serde(default)]
    pub game_over: bool,

    // Onboarding tutorial (only enabled for fresh games)
    #[serde(default)]
    pub tutorial_active: bool,
    #[serde(default)]
    pub tutorial_step: i32,
    /// The player has opened the Codex at least once (drives the tutorial's
    /// "learn the elements" beat). Transient — re-taught each session.
    #[serde(skip)]
    pub tutorial_codex_seen: bool,

    // Monster progression
    pub unlocked_species: Vec<String>,
    pub unlocked_monsters: Vec<String>,
    /// Experience each monster *type* has earned, pooled across every creature
    /// of that type the dungeon has ever fielded. Individual monsters do not
    /// progress; a line does, and crossing a threshold unlocks its next variant.
    #[serde(default)]
    pub monster_type_experience: HashMap<String, i32>,

    // UI state (not persisted)
    #[serde(skip)]
    pub selected_room: Option<(i32, usize)>,
    #[serde(skip)]
    pub selected_monster: Option<String>,
    #[serde(skip)]
    pub selected_upgrade: Option<String>,
    /// Hero whose journal page is open in the HEROES tab, if any.
    #[serde(skip)]
    pub selected_hero: Option<u64>,
    #[serde(skip)]
    pub effects: Vec<RoomEffect>,
    /// Income accumulating over the raid currently in progress.
    #[serde(skip)]
    pub current_raid: Option<RaidTally>,
    /// The most recently concluded raid, shown as a summary card.
    #[serde(skip)]
    pub last_raid_summary: Option<RaidSummary>,
    #[serde(skip)]
    pub pending_confirmation: Option<PendingConfirmation>,

    // Log
    pub log: Vec<LogEntry>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let run_seed = macroquad_toolkit::rng::random_u64();
        // Create initial floor with entrance and core rooms
        let mut floor1 = Floor::new(1, 1, true);
        floor1.rooms.push(Room::new(1, RoomType::Entrance, 0, 1));
        floor1.rooms.push(Room::new(2, RoomType::Core, 1, 1));
        floor1.rebuild_linear_exits();

        Self {
            mana: 100,
            max_mana: 200,
            mana_regen: 1.0,
            gold: 50,
            souls: 0,
            day: 1,
            hour: 6,
            speed: 1,
            paused: false,
            run_seed,
            run_rng: SeededRng::new(run_seed),
            status: DungeonStatus::Closed,
            floors: vec![floor1],
            total_floors: 1,
            deep_core_bonus: 0.1,
            adventurer_parties: Vec::new(),
            // Day 1, hour 8 in absolute hours.
            next_party_spawn: 32,
            known_adventurers: Vec::new(),
            total_deaths: 0,
            threat_warned: 0,
            raids_completed: 0,
            reputation: 0,
            core_hp: 500,
            core_max_hp: 500,
            siege_active: false,
            prestige: 0,
            core_powers: Vec::new(),
            milestones: Vec::new(),
            difficulty: crate::data::difficulty::Difficulty::default(),
            core_smite_cooldown: Cooldown::new(0.0),
            game_over: false,
            tutorial_active: true,
            tutorial_step: 0,
            tutorial_codex_seen: false,
            unlocked_species: vec![],
            unlocked_monsters: vec![],
            monster_type_experience: HashMap::new(),
            selected_room: None,
            selected_monster: None,
            selected_upgrade: None,
            selected_hero: None,
            effects: Vec::new(),
            current_raid: None,
            last_raid_summary: None,
            pending_confirmation: None,
            log: vec![LogEntry::system(
                "Welcome to Dungeon Core! Choose a starter race to awaken your first defenders.",
            )],
        }
    }

    /// Upgrade older saves to the current schema. Called after load.
    pub fn migrate(&mut self) {
        // Single-slot room upgrade → per-type upgrade list.
        for floor in &mut self.floors {
            for room in &mut floor.rooms {
                if let Some(upgrade) = room.upgrade.take() {
                    if !room.has_upgrade_type(upgrade.upgrade_type.clone()) {
                        room.upgrades.push(upgrade);
                    }
                }
            }
        }

        // Linear room queue → graph edges. Pre-graph saves have no `exits`; seed
        // them as a single chain in position order (identical traversal). A
        // floor that already carries edges is left untouched.
        for floor in &mut self.floors {
            let has_edges = floor.rooms.iter().any(|r| !r.exits.is_empty());
            if !has_edges && floor.rooms.len() > 1 {
                floor.rebuild_linear_exits();
            }
        }
    }

    /// Mutable registry record for a hero id, if known.
    pub fn hero_mut(&mut self, id: u64) -> Option<&mut HeroRecord> {
        self.known_adventurers.iter_mut().find(|h| h.id == id)
    }

    /// Credit a monster kill to a hero's ledger, and remember it by name — a
    /// hero's journal is the story of what they did to *this* dungeon.
    pub fn record_hero_kill(&mut self, hero_id: u64, monster: &str, floor: i32) {
        let day = self.day;
        if let Some(record) = self.hero_mut(hero_id) {
            record.kills += 1;
            record.remember(day, format!("Slew a {monster} on floor {floor}"));
        }
    }

    /// Record a hero's death in the ledger. If the fallen hero was a rival, the
    /// dungeon claims a bounty (souls + gold) — the grudge, paid.
    pub fn record_hero_death(&mut self, hero_id: u64, floor: i32) {
        let day = self.day;
        let mut bounty: Option<(String, i32, i32)> = None;
        if let Some(record) = self.hero_mut(hero_id) {
            if record.status != HeroStatus::Dead && record.is_rival() {
                let (souls, gold) = record.bounty();
                bounty = Some((record.name.clone(), souls, gold));
            }
            record.status = HeroStatus::Dead;
            record.death_floor = floor;
            record.death_day = day;
            record.remember(day, format!("Fell on floor {floor}"));
        }
        if let Some((name, souls, gold)) = bounty {
            self.souls += souls;
            self.gold += gold;
            self.add_log(LogEntry::system(format!(
                "BOUNTY CLAIMED — the rival {} falls at last! +{} souls, +{} gold.",
                name, souls, gold
            )));
        }
    }

    /// XP needed to advance from `level` to the next. Levels cap at 10.
    pub fn xp_for_level(level: i32) -> i32 {
        level * 50
    }

    /// Add a log entry, keeping max entries
    pub fn add_log(&mut self, entry: LogEntry) {
        self.log.push(entry);
        if self.log.len() > crate::data::MAX_LOG_ENTRIES {
            self.log.remove(0);
        }
    }

    /// Mutable accumulator for the raid in progress, created on first use.
    pub fn raid_tally(&mut self) -> &mut RaidTally {
        self.current_raid.get_or_insert_with(RaidTally::default)
    }

    /// Whether a permanent core power has been purchased.
    pub fn has_core_power(&self, id: &str) -> bool {
        self.core_powers.iter().any(|p| p == id)
    }

    /// Deaths required to trigger a siege, scaled by difficulty.
    pub fn siege_threshold(&self) -> i32 {
        (SIEGE_THREAT_DEATHS as f32 * self.difficulty.profile().siege_threshold_mult).round() as i32
    }

    /// Mana-income multiplier from difficulty (applied to the presence trickle
    /// and to death income alike).
    /// Experience the whole line of this monster type has pooled.
    pub fn type_experience(&self, type_name: &str) -> i32 {
        self.monster_type_experience
            .get(type_name)
            .copied()
            .unwrap_or(0)
    }

    /// Credit a type's shared pool for work one of its creatures did.
    pub fn add_type_experience(&mut self, type_name: &str, xp: i32) {
        *self
            .monster_type_experience
            .entry(type_name.to_string())
            .or_insert(0) += xp;
    }

    pub fn income_mult(&self) -> f32 {
        self.difficulty.profile().income_mult
    }

    /// Current threat tier (0-4) derived from accumulated adventurer deaths.
    /// The tier-4 (siege) threshold scales with difficulty.
    pub fn threat_tier(&self) -> i32 {
        match self.total_deaths {
            d if d >= self.siege_threshold() => 4,
            d if d >= 50 => 3,
            d if d >= 25 => 2,
            d if d >= 10 => 1,
            _ => 0,
        }
    }

    /// Readable realm standing, separate from the death-driven threat meter.
    pub fn reputation_band(&self) -> ReputationBand {
        reputation::band(self.reputation)
    }

    /// Deterministic adjustments for the next visitor party.
    pub fn visitor_quality(&self) -> VisitorQuality {
        reputation::visitor_quality(self.reputation)
    }

    /// Apply a concluded raid's standing change and keep the value bounded.
    pub fn apply_raid_reputation(
        &mut self,
        floor: i32,
        survivors: i32,
        loot: i32,
        returning_survivors: i32,
    ) -> i32 {
        let change = reputation::raid_change(floor, survivors, loot, returning_survivors);
        self.reputation = (self.reputation + change).clamp(REPUTATION_MIN, REPUTATION_MAX);
        change
    }

    /// Get the deepest floor
    pub fn deepest_floor(&self) -> Option<&Floor> {
        self.floors.iter().find(|f| f.is_deepest)
    }

    /// Get mutable reference to the deepest floor
    pub fn deepest_floor_mut(&mut self) -> Option<&mut Floor> {
        self.floors.iter_mut().find(|f| f.is_deepest)
    }

    /// Count total rooms (excluding entrance and core)
    pub fn total_room_count(&self) -> i32 {
        self.floors
            .iter()
            .flat_map(|f| &f.rooms)
            .filter(|r| r.room_type != RoomType::Core && r.room_type != RoomType::Entrance)
            .count() as i32
    }
}

#[cfg(test)]
mod tests {
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
    fn rival_thresholds() {
        assert!(!hero(1, 1, 0).is_rival());
        assert!(hero(1, 3, 0).is_rival(), "recurring survivor is a rival");
        assert!(hero(1, 1, 5).is_rival(), "prolific slayer is a rival");
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
}
