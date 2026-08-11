use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterTrait {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trait_type: String,  // "Passive", "Active"
    pub target_type: String, // "Self", "Enemy", "EnemyParty", "Ally"
    pub applies_to: String,  // "Hourly", "OnDefense", "OnAttack", "OnCombatStart"
    #[serde(default)]
    pub effect_type: String, // "HealPercent", "DamageFlat", "DamageReductionMult", "AttackBonus"
    #[serde(default)]
    pub scaling_type: String, // "None", "PerAlly", "PerEnemy"
    pub mana_cost: i32,
    pub cooldown: i32,
    pub value: f32,
}

#[derive(Debug, Deserialize)]
struct TraitsData {
    traits: Vec<MonsterTrait>,
}

// Embed JSON for WASM
const TRAITS_JSON: &str = macroquad_toolkit::include_json_str!("../../assets/traits.json");

/// Load all traits
pub fn get_all_traits() -> Vec<MonsterTrait> {
    let data: TraitsData = serde_json::from_str(TRAITS_JSON).expect("Failed to parse traits.json");
    data.traits
}

/// Get a specific trait by ID
pub fn get_trait(id: &str) -> Option<MonsterTrait> {
    get_all_traits().into_iter().find(|t| t.id == id)
}
