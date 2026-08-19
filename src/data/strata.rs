//! Authored dungeon-depth bands. Strata turn a tall dungeon into distinct
//! ecological regions: each favors one defender element and raises the reward
//! adventurers carry from increasingly dangerous depths.

use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Clone, Debug, Deserialize)]
pub struct StratumData {
    pub id: String,
    pub name: String,
    pub floor_min: i32,
    pub floor_max: i32,
    pub element: String,
    pub defender_attack_multiplier: f32,
    /// Multiplier on damage received by a matching defender (below 1 guards).
    pub defender_guard_multiplier: f32,
    pub loot_multiplier: f32,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct StrataData {
    strata: Vec<StratumData>,
}

const STRATA_JSON: &str = macroquad_toolkit::include_json_str!("../../assets/strata.json");

fn data() -> &'static StrataData {
    static CACHE: OnceLock<StrataData> = OnceLock::new();
    CACHE.get_or_init(|| {
        macroquad_toolkit::data_loader::load_embedded_json_labeled(
            "assets/strata.json",
            STRATA_JSON,
        )
        .expect("Failed to parse assets/strata.json")
    })
}

pub fn get_all_strata() -> &'static [StratumData] {
    &data().strata
}

pub fn stratum_for_floor(floor: i32) -> &'static StratumData {
    let floor = floor.max(1);
    data()
        .strata
        .iter()
        .find(|stratum| floor >= stratum.floor_min && floor <= stratum.floor_max)
        .or_else(|| data().strata.last())
        .expect("strata catalogue must contain at least one band")
}

impl StratumData {
    pub fn short_label(&self) -> &'static str {
        match self.id.as_str() {
            "rootways" => "RT",
            "ember_faults" => "EM",
            "drowned_hollows" => "WA",
            "crystal_veins" => "AR",
            "grave_below" => "GR",
            _ => "DN",
        }
    }

    pub fn resonates_with(&self, element: &str) -> bool {
        !element.is_empty() && self.element == element
    }

    pub fn attack_multiplier_for(&self, element: &str) -> f32 {
        if self.resonates_with(element) {
            self.defender_attack_multiplier
        } else {
            1.0
        }
    }

    pub fn guard_multiplier_for(&self, element: &str) -> f32 {
        if self.resonates_with(element) {
            self.defender_guard_multiplier
        } else {
            1.0
        }
    }
}
