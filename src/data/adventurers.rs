use serde::Deserialize;
use std::sync::OnceLock;

/// Adventurer class from JSON
#[derive(Clone, Debug, Deserialize)]
pub struct AdventurerClass {
    pub name: String,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    /// Damage element for combat matchups; empty means neutral.
    #[serde(default)]
    pub element: String,
}

/// Adventurer race: flat stat modifiers applied on top of the class.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct AdventurerRace {
    pub name: String,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    #[serde(default)]
    pub description: String,
}

/// Dialogue quotes
#[derive(Debug, Deserialize)]
pub struct QuotesData {
    pub victory: Vec<String>,
    pub entry: Vec<String>,
    pub exit: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AdventurersData {
    classes: Vec<AdventurerClass>,
    #[serde(default)]
    races: Vec<AdventurerRace>,
    names: Vec<String>,
    quotes: QuotesData,
}

// Embed JSON at compile time for WASM compatibility
const ADVENTURERS_JSON: &str =
    macroquad_toolkit::include_json_str!("../../assets/adventurers.json");

fn data() -> &'static AdventurersData {
    static CACHE: OnceLock<AdventurersData> = OnceLock::new();
    CACHE.get_or_init(|| {
        macroquad_toolkit::data_loader::load_embedded_json_labeled(
            "assets/adventurers.json",
            ADVENTURERS_JSON,
        )
        .expect("Failed to parse assets/adventurers.json")
    })
}

/// Get all adventurer classes
pub fn get_adventurer_classes() -> Vec<AdventurerClass> {
    data().classes.clone()
}

/// Get adventurer class by name
pub fn get_adventurer_class(name: &str) -> Option<AdventurerClass> {
    get_adventurer_classes()
        .into_iter()
        .find(|c| c.name == name)
}

/// Get all adventurer races
pub fn get_all_races() -> Vec<AdventurerRace> {
    data().races.clone()
}

/// Get a race's stat modifiers by name.
pub fn get_race(name: &str) -> Option<AdventurerRace> {
    get_all_races().into_iter().find(|r| r.name == name)
}

/// Get all race names.
pub fn get_race_names() -> Vec<String> {
    get_all_races().into_iter().map(|r| r.name).collect()
}

/// Get all adventurer names
pub fn get_adventurer_names() -> Vec<String> {
    data().names.clone()
}

/// Get victory quotes
pub fn get_victory_quotes() -> Vec<String> {
    data().quotes.victory.clone()
}

/// Get entry quotes
pub fn get_entry_quotes() -> Vec<String> {
    data().quotes.entry.clone()
}

/// Get exit quotes
pub fn get_exit_quotes() -> Vec<String> {
    data().quotes.exit.clone()
}
