use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::game_state::{Equipment, Stats};

/// Equipment types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon,
    Armor,
    Accessory,
}

impl EquipmentType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "weapon" => EquipmentType::Weapon,
            "armor" => EquipmentType::Armor,
            "accessory" => EquipmentType::Accessory,
            _ => panic!("Unknown equipment type: {}", s),
        }
    }
}

/// JSON-loadable equipment template
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquipmentTemplate {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub equipment_type: String,
    pub level: u32,
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub cost: i32,
    pub description: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
struct EquipmentData {
    equipment: Vec<EquipmentTemplate>,
}

// Embed JSON at compile time for WASM compatibility
const EQUIPMENT_JSON: &str = macroquad_toolkit::include_json_str!("../../assets/equipment.json");

/// Load all equipment templates from embedded JSON
pub fn get_all_equipment() -> Vec<EquipmentTemplate> {
    equipment_data().equipment.clone()
}

fn equipment_data() -> &'static EquipmentData {
    static CACHE: OnceLock<EquipmentData> = OnceLock::new();
    CACHE.get_or_init(|| {
        macroquad_toolkit::data_loader::load_embedded_json_labeled(
            "assets/equipment.json",
            EQUIPMENT_JSON,
        )
        .expect("Failed to parse assets/equipment.json")
    })
}

/// Find equipment template by ID
pub fn get_equipment_template(id: u32) -> Option<EquipmentTemplate> {
    get_all_equipment().into_iter().find(|e| e.id == id)
}

/// Find equipment template by name
pub fn get_equipment_template_by_name(name: &str) -> Option<EquipmentTemplate> {
    get_all_equipment().into_iter().find(|e| e.name == name)
}

/// Get equipment of a specific type
pub fn get_equipment_by_type(equipment_type: &str) -> Vec<EquipmentTemplate> {
    get_all_equipment()
        .into_iter()
        .filter(|e| e.equipment_type == equipment_type)
        .collect()
}

/// Get equipment by level
pub fn get_equipment_by_level(level: u32) -> Vec<EquipmentTemplate> {
    get_all_equipment()
        .into_iter()
        .filter(|e| e.level == level)
        .collect()
}

/// Get equipment within a cost range
pub fn get_equipment_by_cost_range(min_cost: i32, max_cost: i32) -> Vec<EquipmentTemplate> {
    get_all_equipment()
        .into_iter()
        .filter(|e| e.cost >= min_cost && e.cost <= max_cost)
        .collect()
}

/// Build a level-appropriate adventurer loadout from the equipment catalog.
pub fn recommended_loadout(class_name: &str, level: i32) -> Equipment {
    let weapon_level = level.max(1) as u32;
    let armor_level = match class_name {
        "Warrior" => level.max(1) as u32,
        "Cleric" => (level - 1).max(1) as u32,
        _ => (level - 2).max(1) as u32,
    };
    let accessory_level = match class_name {
        "Mage" | "Cleric" => level.max(1) as u32,
        _ => (level - 2).max(1) as u32,
    };

    Equipment {
        weapon: best_equipment_name("weapon", weapon_level).unwrap_or_else(|| "Rusty Sword".into()),
        armor: best_equipment_name("armor", armor_level).unwrap_or_else(|| "Cloth Robe".into()),
        accessory: best_equipment_name("accessory", accessory_level)
            .unwrap_or_else(|| "Worn Ring".into()),
    }
}

/// Convert an adventurer's named equipment into bonuses for the current stat model.
pub fn equipment_stat_bonus(equipment: &Equipment, class_name: &str) -> Stats {
    let items = [
        get_equipment_template_by_name(&equipment.weapon),
        get_equipment_template_by_name(&equipment.armor),
        get_equipment_template_by_name(&equipment.accessory),
    ];

    let mut attack = 0;
    let mut defense = 0;
    let mut magic = 0;

    for item in items.into_iter().flatten() {
        attack += item.attack;
        defense += item.defense;
        magic += item.magic;
    }

    let magic_attack = match class_name {
        "Mage" => magic,
        "Cleric" => magic / 2,
        _ => 0,
    };
    let magic_defense = match class_name {
        "Cleric" => magic / 3,
        _ => 0,
    };

    Stats {
        hp: defense * 2 + magic_defense,
        attack: attack + magic_attack,
        defense: defense + magic_defense,
    }
}

fn best_equipment_name(equipment_type: &str, level: u32) -> Option<String> {
    get_equipment_by_type(equipment_type)
        .into_iter()
        .filter(|item| item.level <= level)
        .max_by_key(|item| item.level)
        .map(|item| item.name)
}
