//! Deterministic depth chapters shared by simulation and presentation.
//!
//! Strata describe the dungeon's ecology. A depth layer describes the rhythm
//! inside each stratum, so floors 1-4, 5-8, and so on still feel like a climb
//! rather than four interchangeable rooms with a new colour wash.

use super::{Adventurer, AdventurerParty, GameState, HeroDrive};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthLayer {
    Threshold,
    Hunt,
    Gauntlet,
    Apex,
}

impl DepthLayer {
    pub fn for_floor(floor: i32) -> Self {
        match (floor.max(1) - 1) % 4 {
            0 => Self::Threshold,
            1 => Self::Hunt,
            2 => Self::Gauntlet,
            _ => Self::Apex,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Threshold => "Threshold",
            Self::Hunt => "Hunt",
            Self::Gauntlet => "Gauntlet",
            Self::Apex => "Apex",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Threshold => "The stone still remembers sunlight; routes are readable.",
            Self::Hunt => "Predators learn the traffic; exposed parties draw extra pressure.",
            Self::Gauntlet => "The deep path narrows into a deliberate test of formation.",
            Self::Apex => "A relic sleeps here, and every living thing guards its secret.",
        }
    }

    pub fn defender_pressure(self) -> f32 {
        match self {
            Self::Threshold => 1.0,
            Self::Hunt => 1.04,
            Self::Gauntlet => 1.09,
            Self::Apex => 1.15,
        }
    }

    pub fn loot_multiplier(self) -> f32 {
        match self {
            Self::Threshold => 1.0,
            Self::Hunt => 1.06,
            Self::Gauntlet => 1.14,
            Self::Apex => 1.24,
        }
    }

    pub fn art_mark(self) -> &'static str {
        match self {
            Self::Threshold => "I",
            Self::Hunt => "II",
            Self::Gauntlet => "III",
            Self::Apex => "IV",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DepthRelic {
    pub id: &'static str,
    pub name: &'static str,
    pub boon: &'static str,
    pub apex_monster: &'static str,
}

pub fn relic_for_floor(floor: i32) -> DepthRelic {
    match crate::data::strata::stratum_for_floor(floor).id.as_str() {
        "rootways" => DepthRelic {
            id: "rootbound_sigil",
            name: "Rootbound Sigil",
            boon: "+50 maximum mana",
            apex_monster: "Rootbound Warcaller",
        },
        "ember_faults" => DepthRelic {
            id: "cinder_crown",
            name: "Cinder Crown",
            boon: "+0.2 mana regeneration",
            apex_monster: "Cinder Titan",
        },
        "drowned_hollows" => DepthRelic {
            id: "tide_lens",
            name: "Tide Lens",
            boon: "Core Smite costs 8 less mana",
            apex_monster: "Tide Maw",
        },
        "crystal_veins" => DepthRelic {
            id: "prism_heart",
            name: "Prism Heart",
            boon: "+5% depth defender pressure",
            apex_monster: "Prism Oracle",
        },
        _ => DepthRelic {
            id: "ossuary_key",
            name: "Ossuary Key",
            boon: "+100 maximum Core health",
            apex_monster: "Ossuary Colossus",
        },
    }
}

/// A party's behaviour is derived from the people inside it. That keeps the
/// party save-compatible while letting returning heroes create recognisable,
/// emergent expedition patterns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExpeditionDoctrine {
    #[default]
    Survey,
    RelicHunt,
    Vengeance,
    Profit,
}

impl ExpeditionDoctrine {
    pub fn label(self) -> &'static str {
        match self {
            Self::Survey => "Survey",
            Self::RelicHunt => "Relic Hunt",
            Self::Vengeance => "Vengeance",
            Self::Profit => "Profit Run",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Survey => "Tests the safest route and retreats with what it learns.",
            Self::RelicHunt => "Pushes deeper, valuing apex chambers over easy treasure.",
            Self::Vengeance => "Hunts dangerous defenders and refuses to be intimidated.",
            Self::Profit => "Follows the richest branch and carries more gold home.",
        }
    }

    pub fn attack_multiplier(self) -> f32 {
        match self {
            Self::Vengeance => 1.08,
            Self::RelicHunt => 1.03,
            Self::Survey | Self::Profit => 1.0,
        }
    }

    pub fn loot_multiplier(self) -> f32 {
        match self {
            Self::Profit => 1.12,
            Self::RelicHunt => 1.05,
            Self::Survey | Self::Vengeance => 1.0,
        }
    }
}

pub fn doctrine_for_members(state: &GameState, members: &[Adventurer]) -> ExpeditionDoctrine {
    let has_rival = members.iter().any(|member| {
        state
            .known_adventurers
            .iter()
            .find(|record| record.id == member.id)
            .is_some_and(|record| record.is_rival())
    });
    if has_rival
        || members
            .iter()
            .any(|member| member.drive == HeroDrive::Glory)
    {
        return ExpeditionDoctrine::Vengeance;
    }
    if members
        .iter()
        .any(|member| member.drive == HeroDrive::Discovery)
    {
        return ExpeditionDoctrine::RelicHunt;
    }
    if members
        .iter()
        .filter(|member| member.drive == HeroDrive::Fortune)
        .count()
        >= 2
    {
        return ExpeditionDoctrine::Profit;
    }
    ExpeditionDoctrine::Survey
}

pub fn doctrine_for_party(state: &GameState, party: &AdventurerParty) -> ExpeditionDoctrine {
    doctrine_for_members(state, &party.members)
}

#[cfg(test)]
mod tests;
