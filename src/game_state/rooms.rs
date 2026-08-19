use super::Monster;
use serde::{Deserialize, Serialize};

/// Standing command given to a room's defenders. Orders trade raw safety for
/// target control and are changed between raids through visible inspector
/// buttons, so the keeper can prepare different rooms for different jobs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomBattleOrder {
    #[default]
    Balanced,
    HoldLine,
    CullWounded,
}

impl RoomBattleOrder {
    pub const ALL: [Self; 3] = [Self::Balanced, Self::HoldLine, Self::CullWounded];

    pub fn label(self) -> &'static str {
        match self {
            Self::Balanced => "Balanced",
            Self::HoldLine => "Hold",
            Self::CullWounded => "Cull",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Balanced => "No modifiers; targets are spread naturally.",
            Self::HoldLine => "-22% damage taken, but -12% defender attack.",
            Self::CullWounded => "Focuses the weakest hero; +12% damage taken.",
        }
    }

    pub fn defender_attack_multiplier(self) -> f32 {
        match self {
            Self::Balanced | Self::CullWounded => 1.0,
            Self::HoldLine => 0.88,
        }
    }

    pub fn defender_damage_taken_multiplier(self) -> f32 {
        match self {
            Self::Balanced => 1.0,
            Self::HoldLine => 0.78,
            Self::CullWounded => 1.12,
        }
    }
}

/// Room type enumeration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoomType {
    Entrance,
    Normal,
    Boss,
    Core,
}

/// Room upgrade type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoomUpgradeType {
    Trap,
    Treasure,
    Reinforcement,
    Evolution,
    Attunement,
}

/// Room upgrade applied to a room.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomUpgrade {
    pub upgrade_type: RoomUpgradeType,
    pub name: String,
    pub effect: String,
    pub multiplier: f32,
    /// Element this upgrade is keyed to (attunements, elemental traps).
    #[serde(default)]
    pub element: Option<String>,
    /// Trap behavior; empty means a legacy flat-damage trap.
    #[serde(default)]
    pub effect_kind: String,
    /// A second simulation rule beyond the upgrade's primary effect. Empty on
    /// legacy saves, but every current catalogue entry supplies one.
    #[serde(default)]
    pub secondary_effect: String,
    #[serde(default)]
    pub secondary_kind: String,
    #[serde(default)]
    pub secondary_value: f32,
    /// A Rogue sprung this trap; it re-arms between raids (costs mana).
    #[serde(default)]
    pub disarmed: bool,
}

/// Room in a dungeon floor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: u64,
    pub room_type: RoomType,
    /// Stable per-floor node key and endpoint used by `exits`.
    pub position: usize,
    /// Child rooms this room routes into. Empty on pre-graph saves; migration
    /// rebuilds the linear chain from `position` order.
    #[serde(default)]
    pub exits: Vec<usize>,
    pub floor_number: i32,
    pub monsters: Vec<Monster>,
    /// Keeper-selected combat behavior. Old saves default to Balanced.
    #[serde(default)]
    pub battle_order: RoomBattleOrder,
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
            battle_order: RoomBattleOrder::Balanced,
            upgrades: Vec::new(),
            upgrade: None,
            explored: false,
            loot: 0,
        }
    }

    pub fn upgrade_of(&self, upgrade_type: RoomUpgradeType) -> Option<&RoomUpgrade> {
        self.upgrades
            .iter()
            .find(|u| u.upgrade_type == upgrade_type)
    }

    pub fn has_upgrade_type(&self, upgrade_type: RoomUpgradeType) -> bool {
        self.upgrade_of(upgrade_type).is_some()
    }

    pub fn trap_multiplier(&self) -> f32 {
        self.upgrade_of(RoomUpgradeType::Trap)
            .map(|u| u.multiplier)
            .unwrap_or(1.0)
    }

    pub fn treasure_multiplier(&self) -> f32 {
        self.upgrade_of(RoomUpgradeType::Treasure)
            .map(|u| u.multiplier)
            .unwrap_or(1.0)
    }

    pub fn reinforcement_multiplier(&self) -> f32 {
        self.upgrade_of(RoomUpgradeType::Reinforcement)
            .map(|u| u.multiplier)
            .unwrap_or(1.0)
    }

    pub fn evolution_multiplier(&self) -> f32 {
        self.upgrade_of(RoomUpgradeType::Evolution)
            .map(|u| u.multiplier)
            .unwrap_or(1.0)
    }

    /// Element attunement of this room: (element, monster stat multiplier).
    pub fn attunement(&self) -> Option<(&str, f32)> {
        self.upgrade_of(RoomUpgradeType::Attunement)
            .and_then(|u| u.element.as_deref().map(|e| (e, u.multiplier)))
    }

    /// Product of secondary room multipliers with the requested behavior.
    pub fn secondary_multiplier(&self, kind: &str) -> f32 {
        self.upgrades
            .iter()
            .filter(|upgrade| upgrade.secondary_kind == kind)
            .map(|upgrade| upgrade.secondary_value)
            .product::<f32>()
            .max(0.0)
    }

    pub fn defender_damage_taken_multiplier(&self) -> f32 {
        neutral_multiplier(self.secondary_multiplier("DefenderDamageReduction"))
            * self.battle_order.defender_damage_taken_multiplier()
    }

    pub fn defender_attack_multiplier(&self) -> f32 {
        self.battle_order.defender_attack_multiplier()
    }

    /// Attack modifier applied to every living adventurer in this room.
    pub fn adventurer_attack_multiplier(&self) -> f32 {
        neutral_multiplier(self.secondary_multiplier("AdventurerAttack"))
    }

    pub fn adventurer_damage_to_monsters_multiplier(&self) -> f32 {
        neutral_multiplier(self.secondary_multiplier("AdventurerDamageToMonsters"))
    }

    /// Matching elemental adventurers receive the same resonance as a shrine.
    pub fn elemental_adventurer_attack_multiplier(&self, element: &str) -> f32 {
        let multiplier = self
            .upgrades
            .iter()
            .filter(|upgrade| {
                upgrade.secondary_kind == "ElementalAdventurerAttack"
                    && upgrade.element.as_deref() == Some(element)
            })
            .map(|upgrade| upgrade.secondary_value)
            .product::<f32>();
        neutral_multiplier(multiplier)
    }

    /// Growth Chamber's additive regeneration rate per combat tick.
    pub fn monster_regeneration_rate(&self) -> f32 {
        self.upgrades
            .iter()
            .filter(|upgrade| upgrade.secondary_kind == "MonsterRegen")
            .map(|upgrade| upgrade.secondary_value)
            .sum()
    }

    pub fn adventurer_kill_mana_multiplier(&self) -> f32 {
        neutral_multiplier(self.secondary_multiplier("KillMana"))
    }
}

fn neutral_multiplier(multiplier: f32) -> f32 {
    if multiplier == 0.0 {
        1.0
    } else {
        multiplier
    }
}
