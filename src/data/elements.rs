use serde::{Deserialize, Serialize};

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
    let data: ElementsData =
        serde_json::from_str(ELEMENTS_JSON).expect("Failed to parse elements.json");
    data.elements
}

/// Get one element by id
pub fn get_element(id: &str) -> Option<ElementDef> {
    get_all_elements().into_iter().find(|e| e.id == id)
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
