//! Serializable and transient domain records used by the game-state aggregate.

use serde::{Deserialize, Serialize};

/// Combat stats for monsters and adventurers.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
}

/// Active trait instance on a monster.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveTrait {
    pub id: String,
    pub name: String,
    pub cooldown_timer: i32,
}

fn default_fusion_rank() -> u8 {
    1
}

/// Monster instance in a room.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Monster {
    pub id: u64,
    pub type_name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub alive: bool,
    pub is_boss: bool,
    /// Rank I-III earned by fusing identical equal-rank defenders. Legacy
    /// creatures and ordinary summons begin at rank I.
    #[serde(default = "default_fusion_rank")]
    pub fusion_rank: u8,
    pub scaled_stats: Stats,
    #[serde(default)]
    pub active_traits: Vec<ActiveTrait>,
}

/// Dungeon operational status.
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

/// Log entry type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub message: String,
    pub log_type: String, // "system", "combat", "adventure", "building"
    pub timestamp: u64,
}

/// A transient filter for the event-log viewport. It intentionally remains out
/// of saves: loading a run should never hide events because of a past UI choice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogFilter {
    #[default]
    All,
    Combat,
    Adventure,
    Building,
    System,
}

impl LogFilter {
    pub fn matches(self, entry: &LogEntry) -> bool {
        match self {
            Self::All => true,
            Self::Combat => entry.log_type == "combat",
            Self::Adventure => entry.log_type == "adventure",
            Self::Building => entry.log_type == "building",
            Self::System => entry.log_type == "system",
        }
    }
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
