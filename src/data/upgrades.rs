use crate::game_state::{RoomUpgrade, RoomUpgradeType};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// JSON-loadable upgrade template
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeTemplate {
    #[serde(rename = "type")]
    pub upgrade_type: String,
    pub name: String,
    pub effect: String,
    pub multiplier: f32,
    pub mana_cost: i32,
    pub souls_cost: i32,
    /// Element this upgrade is keyed to (attunements, elemental traps)
    #[serde(default)]
    pub element: Option<String>,
    /// Trap behavior kind; see RoomUpgrade::effect_kind
    #[serde(default)]
    pub effect_kind: String,
    /// Second, simulation-facing room rule. Every current catalogue entry
    /// declares one; defaults keep older authored data and saves readable.
    #[serde(default)]
    pub secondary_effect: String,
    #[serde(default)]
    pub secondary_kind: String,
    #[serde(default)]
    pub secondary_value: f32,
}

#[derive(Debug, Deserialize)]
struct UpgradesData {
    upgrades: Vec<UpgradeTemplate>,
}

// Embed JSON at compile time for WASM compatibility
const UPGRADES_JSON: &str = macroquad_toolkit::include_json_str!("../../assets/upgrades.json");

/// Load all upgrade templates from embedded JSON
pub fn get_all_upgrades() -> Vec<UpgradeTemplate> {
    upgrades_data().upgrades.clone()
}

fn upgrades_data() -> &'static UpgradesData {
    static CACHE: OnceLock<UpgradesData> = OnceLock::new();
    CACHE.get_or_init(|| {
        macroquad_toolkit::data_loader::load_embedded_json_labeled(
            "assets/upgrades.json",
            UPGRADES_JSON,
        )
        .expect("Failed to parse assets/upgrades.json")
    })
}

/// Find upgrade template by name
pub fn get_upgrade_template(name: &str) -> Option<UpgradeTemplate> {
    upgrades_data()
        .upgrades
        .iter()
        .find(|u| u.name == name)
        .cloned()
}

/// Get upgrades of a specific type
pub fn get_upgrades_by_type(upgrade_type: &str) -> Vec<UpgradeTemplate> {
    upgrades_data()
        .upgrades
        .iter()
        .filter(|u| u.upgrade_type == upgrade_type)
        .cloned()
        .collect()
}

/// Convert string type to RoomUpgradeType enum
pub fn parse_upgrade_type(type_str: &str) -> RoomUpgradeType {
    match type_str {
        "trap" => RoomUpgradeType::Trap,
        "treasure" => RoomUpgradeType::Treasure,
        "reinforcement" => RoomUpgradeType::Reinforcement,
        "evolution" => RoomUpgradeType::Evolution,
        "attunement" => RoomUpgradeType::Attunement,
        _ => RoomUpgradeType::Trap, // Default fallback
    }
}

/// Convert upgrade template to room upgrade
impl UpgradeTemplate {
    pub fn to_room_upgrade(&self) -> RoomUpgrade {
        RoomUpgrade {
            upgrade_type: parse_upgrade_type(&self.upgrade_type),
            name: self.name.clone(),
            effect: self.effect.clone(),
            multiplier: self.multiplier,
            element: self.element.clone(),
            effect_kind: self.effect_kind.clone(),
            secondary_effect: self.secondary_effect.clone(),
            secondary_kind: self.secondary_kind.clone(),
            secondary_value: self.secondary_value,
            disarmed: false,
        }
    }
}
