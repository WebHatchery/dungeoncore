use macroquad_toolkit::rng::SeededRng;
use macroquad_toolkit::timing::Cooldown;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) mod depth;
mod effects;
mod floor;
pub(crate) mod heroes;
mod model;
mod reputation;
mod rooms;
pub use depth::{
    doctrine_for_members, doctrine_for_party, relic_for_floor, DepthLayer, DepthRelic,
    ExpeditionDoctrine,
};
pub use effects::{EffectAnchor, EffectKind, ElementSound, RoomEffect, SoundEvent};
pub use floor::Floor;
pub use heroes::{
    Adventurer, AdventurerParty, Condition, Equipment, HeroDrive, HeroInsight, HeroRecord,
    HeroStatus, HeroWard,
};
pub use model::{
    ActiveTrait, DungeonStatus, LogEntry, LogFilter, Monster, PendingConfirmation, RaidOutcome,
    RaidSummary, RaidTally, Stats,
};
pub use reputation::{ReputationBand, VisitorQuality, REPUTATION_MAX, REPUTATION_MIN};
pub use rooms::{Room, RoomBattleOrder, RoomType, RoomUpgrade, RoomUpgradeType};

fn default_condition_multiplier() -> f32 {
    1.0
}

/// A ready (zero-duration) cooldown, used as the `#[serde(skip)]` default for
/// transient fields — `Cooldown` has no `Default` impl of its own.
pub(crate) fn ready_cooldown() -> Cooldown {
    Cooldown::new(0.0)
}

pub(crate) fn default_board_zoom() -> f32 {
    1.0
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

fn default_core_hp() -> i32 {
    500
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
    /// Permanent relics recovered from the apex boss of each authored stratum.
    /// Relics are both a long-term reward and a visible record of how deep this
    /// particular dungeon has been pushed.
    #[serde(default)]
    pub depth_relics: Vec<String>,
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
    /// Simulation-originated, cosmetic audio requests; never serialized.
    #[serde(skip)]
    pub sound_events: Vec<SoundEvent>,
    /// Income accumulating over the raid currently in progress.
    #[serde(skip)]
    pub current_raid: Option<RaidTally>,
    /// The most recently concluded raid, shown as a summary card.
    #[serde(skip)]
    pub last_raid_summary: Option<RaidSummary>,
    #[serde(skip)]
    pub pending_confirmation: Option<PendingConfirmation>,
    /// Event-log viewport state is UI-only and deliberately never persisted.
    #[serde(skip)]
    pub log_scroll: usize,
    #[serde(skip)]
    pub log_filter: LogFilter,
    /// Dungeon-board vertical viewport, kept outside saved game state.
    #[serde(skip)]
    pub board_scroll: f32,
    /// Player-controlled camera scale for the dungeon cross-section.
    #[serde(skip, default = "default_board_zoom")]
    pub board_zoom: f32,
    /// Horizontal camera offset in world pixels. Rooms keep their world size
    /// as the dungeon grows; this offset reveals the larger structure.
    #[serde(skip)]
    pub board_pan_x: f32,
    /// Last pointer position used by the board's direct-manipulation camera.
    #[serde(skip)]
    pub board_drag_last: Option<(f32, f32)>,
    /// Prevent a drag release from also selecting the room beneath it.
    #[serde(skip)]
    pub board_dragged: bool,
    /// User preference mirrored from title settings; never part of a save.
    #[serde(skip)]
    pub reduced_motion: bool,
    /// Renderer clock. Interactive play follows wall time; capture runs step
    /// it at a fixed cadence so VFX screenshots are repeatable.
    #[serde(skip)]
    pub visual_time: f32,

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
            depth_relics: Vec::new(),
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
            sound_events: Vec::new(),
            current_raid: None,
            last_raid_summary: None,
            pending_confirmation: None,
            log_scroll: 0,
            log_filter: LogFilter::All,
            board_scroll: 0.0,
            board_zoom: 1.0,
            board_pan_x: 0.0,
            board_drag_last: None,
            board_dragged: false,
            reduced_motion: false,
            visual_time: 0.0,
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
            record.deepest_floor = record.deepest_floor.max(floor);
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
            record.deepest_floor = record.deepest_floor.max(floor);
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

    pub fn has_depth_relic(&self, id: &str) -> bool {
        self.depth_relics.iter().any(|relic| relic == id)
    }

    /// Claim the first apex relic in a stratum. The boss reward is deliberately
    /// permanent and immediately useful, so reaching a new ecological band
    /// changes the next build decisions instead of only changing the backdrop.
    pub fn claim_depth_relic(&mut self, floor: i32) -> Option<DepthRelic> {
        let relic = relic_for_floor(floor);
        if self.has_depth_relic(relic.id) {
            return None;
        }
        self.depth_relics.push(relic.id.to_string());
        match relic.id {
            "rootbound_sigil" => self.max_mana += 50,
            "cinder_crown" => self.mana_regen += 0.2,
            "tide_lens" => {}
            "prism_heart" => self.deep_core_bonus += 0.05,
            "ossuary_key" => {
                self.core_max_hp += 100;
                self.core_hp += 100;
            }
            _ => {}
        }
        Some(relic)
    }

    pub fn depth_pressure(&self, floor: i32) -> f32 {
        let layer = DepthLayer::for_floor(floor);
        let relic_bonus = if self.has_depth_relic("prism_heart") {
            1.05
        } else {
            1.0
        };
        layer.defender_pressure() * relic_bonus
    }

    pub fn depth_loot_multiplier(&self, floor: i32) -> f32 {
        DepthLayer::for_floor(floor).loot_multiplier()
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
mod tests;
