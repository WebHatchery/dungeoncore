use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Monster template from JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterTemplate {
    pub name: String,
    pub base_cost: i32,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub species: String,
    pub tier: i32,
    pub emoji: String,
    #[serde(default)]
    pub element: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub traits: Vec<String>,
    /// Souls required to summon, on top of mana (Demons)
    #[serde(default)]
    pub souls_cost: i32,
    /// Tier-4 uniques that can only be summoned in Boss rooms
    #[serde(default)]
    pub boss_only: bool,
}

/// Species data from JSON
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeciesData {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub starter: bool,
    pub unlock_cost: i32,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct MonstersData {
    monsters: Vec<MonsterTemplate>,
    species: Vec<SpeciesData>,
}

// Embed JSON at compile time for WASM compatibility
const MONSTERS_JSON: &str = macroquad_toolkit::include_json_str!("../../assets/monsters.json");

/// Parse the embedded monster data once and keep it for the process lifetime.
/// The JSON is immutable at runtime, so re-parsing it on every lookup (as the
/// old code did) was pure waste — costly when called per-unit each frame.
fn monsters_data() -> &'static MonstersData {
    static CACHE: OnceLock<MonstersData> = OnceLock::new();
    CACHE
        .get_or_init(|| serde_json::from_str(MONSTERS_JSON).expect("Failed to parse monsters.json"))
}

/// Load all monster templates from embedded JSON
pub fn get_monster_templates() -> Vec<MonsterTemplate> {
    monsters_data().monsters.clone()
}

/// Find a monster template by name
pub fn get_monster_template(name: &str) -> Option<MonsterTemplate> {
    monsters_data()
        .monsters
        .iter()
        .find(|t| t.name == name)
        .cloned()
}

/// Whether a monster type belongs to the Undead species. Undead have a distinct
/// identity: they rise again for free but can never heal (see the respawn and
/// regeneration logic). Cheap cached lookup.
pub fn is_undead(name: &str) -> bool {
    monsters_data()
        .monsters
        .iter()
        .find(|t| t.name == name)
        .map(|t| t.species == "Undead")
        .unwrap_or(false)
}

/// Mana to reknit a fallen non-undead defender: half its summon cost (min 2).
/// Undead pay nothing — this is only consulted for the living.
pub fn respawn_mana_cost(name: &str) -> i32 {
    monsters_data()
        .monsters
        .iter()
        .find(|t| t.name == name)
        .map(|t| (t.base_cost / 2).max(2))
        .unwrap_or(2)
}

/// The element id of a monster type, if any. Cheap cached lookup safe to call
/// per-unit each frame (no JSON parse, no full-vec clone).
pub fn monster_element_id(name: &str) -> Option<String> {
    monsters_data()
        .monsters
        .iter()
        .find(|t| t.name == name)
        .and_then(|t| t.element.clone())
}

/// Get all species data
pub fn get_all_species() -> Vec<SpeciesData> {
    monsters_data().species.clone()
}

/// Get one species record by internal ID.
pub fn get_species(species_name: &str) -> Option<SpeciesData> {
    get_all_species()
        .into_iter()
        .find(|s| s.name == species_name)
}

/// Get species unlock cost
pub fn get_species_unlock_cost(species_name: &str) -> Option<i32> {
    get_species(species_name).map(|s| s.unlock_cost)
}

/// Human-facing species name; keeps save/internal IDs stable.
pub fn get_species_display_name(species_name: &str) -> String {
    get_all_species()
        .into_iter()
        .find(|s| s.name == species_name)
        .and_then(|s| s.display_name)
        .unwrap_or_else(|| species_name.to_string())
}

/// Starter roster for a race/species. Higher tiers remain progression unlocks.
pub fn get_starter_monsters_for_species(species_name: &str) -> Vec<MonsterTemplate> {
    get_monster_templates()
        .into_iter()
        .filter(|template| template.species == species_name && template.tier == 1)
        .collect()
}

/// Get all unique species names
pub fn get_species_names() -> Vec<String> {
    get_all_species().into_iter().map(|s| s.name).collect()
}
