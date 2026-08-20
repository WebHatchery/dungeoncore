use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Damage multiplier when the attacker's element beats the defender's.
pub const STRONG_MULT: f32 = 1.5;
/// Damage multiplier when the defender's element beats the attacker's.
pub const WEAK_MULT: f32 = 2.0 / 3.0;

/// Element definition from JSON. The matrix is defined one-directional:
/// only `strong_against` is listed; weakness is derived as its inverse,
/// so a matchup can never be strong both ways.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElementDef {
    pub id: String,
    pub emoji: String,
    pub strong_against: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ElementsData {
    elements: Vec<ElementDef>,
}

// Embed JSON at compile time for WASM compatibility
const ELEMENTS_JSON: &str = macroquad_toolkit::include_json_str!("../../assets/elements.json");

/// Load all element definitions
pub fn get_all_elements() -> Vec<ElementDef> {
    elements_data().elements.clone()
}

fn elements_data() -> &'static ElementsData {
    static CACHE: OnceLock<ElementsData> = OnceLock::new();
    CACHE.get_or_init(|| {
        macroquad_toolkit::data_loader::load_embedded_json_labeled(
            "assets/elements.json",
            ELEMENTS_JSON,
        )
        .expect("Failed to parse assets/elements.json")
    })
}

/// Get one element by id
pub fn get_element(id: &str) -> Option<ElementDef> {
    elements_data()
        .elements
        .iter()
        .find(|e| e.id == id)
        .cloned()
}

/// Attack-damage multiplier for an elemental matchup.
/// Unknown or missing elements fight at neutral effectiveness.
pub fn element_multiplier(attacker: &str, defender: &str) -> f32 {
    let is_strong = |from: &str, against: &str| {
        get_element(from).is_some_and(|e| e.strong_against.iter().any(|s| s == against))
    };

    if is_strong(attacker, defender) {
        STRONG_MULT
    } else if is_strong(defender, attacker) {
        WEAK_MULT
    } else {
        1.0
    }
}
