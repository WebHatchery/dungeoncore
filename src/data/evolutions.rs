use serde::Deserialize;
use std::collections::HashMap;

/// Evolution path from one monster to another
#[derive(Clone, Debug, Deserialize)]
pub struct EvolutionPath {
    pub from_monster: String,
    pub to_monster: String,
    pub experience_required: i32,
    pub conditions: EvolutionConditions,
}

/// Conditions required for evolution
#[derive(Clone, Debug, Deserialize)]
pub struct EvolutionConditions {
    pub min_floor: i32,
    pub gold_cost: i32,
}

/// Root structure of evolution_trees.json
#[derive(Debug, Deserialize)]
struct EvolutionData {
    evolution_trees: HashMap<String, Vec<EvolutionPath>>,
    starting_monsters: HashMap<String, String>,
}

// Embed JSON at compile time for WASM compatibility
const EVOLUTION_JSON: &str = include_str!("../../assets/evolution_trees.json");

/// Load all evolution trees from embedded JSON
pub fn get_evolution_trees() -> HashMap<String, Vec<EvolutionPath>> {
    let data: EvolutionData =
        serde_json::from_str(EVOLUTION_JSON).expect("Failed to parse evolution_trees.json");
    data.evolution_trees
}

/// Load starting monsters map
pub fn get_starting_monsters() -> HashMap<String, String> {
    let data: EvolutionData =
        serde_json::from_str(EVOLUTION_JSON).expect("Failed to parse evolution_trees.json");
    data.starting_monsters
}

/// Get the first evolution path for a specific monster (if it can evolve).
/// Prefer `get_evolutions_for_monster` — branching monsters have several.
pub fn get_evolution_for_monster(monster_name: &str) -> Option<EvolutionPath> {
    get_evolutions_for_monster(monster_name).into_iter().next()
}

/// All evolution paths available to a monster (branching supported).
pub fn get_evolutions_for_monster(monster_name: &str) -> Vec<EvolutionPath> {
    get_evolution_trees()
        .into_values()
        .flatten()
        .filter(|p| p.from_monster == monster_name)
        .collect()
}
