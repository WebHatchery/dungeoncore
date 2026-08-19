//! Adventurers and their ledger: the individuals who delve, the party that
//! moves through the dungeon as one, and the persistent record each hero
//! accumulates across delves. Unlike defenders, adventurers *are* individuals —
//! they carry their own experience and level between raids.

use super::{ready_cooldown, Stats};
use macroquad_toolkit::timing::Cooldown;
use serde::{Deserialize, Serialize};

/// What keeps a hero returning to the dungeon. A drive is persistent identity,
/// not a temporary combat buff: it shapes route choice, risk tolerance, and
/// the reward that hero values across every delve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeroDrive {
    /// Seeks dangerous defenders and hits harder, but takes greater risks.
    Glory,
    /// Values treasure above a safe route and increases the party's haul.
    Fortune,
    /// Pushes expeditions deeper and learns faster from surviving them.
    Discovery,
    /// Protects companions and holds the line when a party might break.
    #[default]
    Duty,
}

impl HeroDrive {
    pub const ALL: [Self; 4] = [Self::Glory, Self::Fortune, Self::Discovery, Self::Duty];

    pub fn label(self) -> &'static str {
        match self {
            Self::Glory => "Glory",
            Self::Fortune => "Fortune",
            Self::Discovery => "Discovery",
            Self::Duty => "Duty",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Glory => "seeks danger; attacks harder",
            Self::Fortune => "chases treasure; lifts loot",
            Self::Discovery => "delves deeper; learns faster",
            Self::Duty => "steadies party; resists damage",
        }
    }

    pub fn attack_multiplier(self) -> f32 {
        if self == Self::Glory {
            1.10
        } else {
            1.0
        }
    }

    pub fn damage_taken_multiplier(self) -> f32 {
        if self == Self::Duty {
            0.90
        } else {
            1.0
        }
    }
}

fn default_resolve() -> i32 {
    50
}

/// Adventurer equipment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: String,
    pub armor: String,
    pub accessory: String,
}

impl Default for Equipment {
    fn default() -> Self {
        Self {
            weapon: "Rusty Sword".into(),
            armor: "Cloth Robe".into(),
            accessory: "Worn Ring".into(),
        }
    }
}

/// A lingering status effect on an adventurer (poison, burn, …)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Condition {
    pub kind: String,
    /// Combat ticks remaining
    pub ticks: i32,
    /// Damage dealt per tick
    pub power: i32,
    /// Multiplier carried by non-damaging conditions such as Weakened and
    /// Brittle. Legacy conditions default to a neutral multiplier.
    #[serde(default = "super::default_condition_multiplier")]
    pub multiplier: f32,
}

/// Individual adventurer in a party
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Adventurer {
    pub id: u64,
    pub name: String,
    pub class_name: String,
    #[serde(default = "default_race")]
    pub race: String,
    #[serde(default)]
    pub drive: HeroDrive,
    /// Persistent confidence copied from the hero ledger for this delve.
    #[serde(default = "default_resolve")]
    pub resolve: i32,
    /// Best preparation carried from earlier escapes. Fresh and legacy heroes
    /// have no ward; veterans equip the strongest stratum insight they know.
    #[serde(default)]
    pub ward: HeroWard,
    pub level: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub alive: bool,
    pub experience: i32,
    pub gold: i32,
    pub equipment: Equipment,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    pub scaled_stats: Stats,
}

fn default_race() -> String {
    "Human".to_string()
}
/// Standing of a hero in the persistent registry.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum HeroStatus {
    /// Survived a previous raid; available to return.
    Alive,
    /// Currently raiding the dungeon.
    Inside,
    /// Killed within the dungeon.
    Dead,
}

/// How many journal lines a hero keeps. A long campaign must not be able to
/// grow the save without limit, so the oldest entries fall off the front.
pub const HERO_JOURNAL_LIMIT: usize = 12;

/// One line in a hero's journal: what they did, and the day they did it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeroEvent {
    pub day: i32,
    pub text: String,
}

/// Persistent familiarity with one dungeon stratum. Mastery rises only when a
/// hero escapes that band alive, so every ward visible to the player is a
/// consequence of an earlier raid rather than a random modifier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeroInsight {
    pub stratum_id: String,
    pub mastery: u8,
}

/// The single countermeasure a veteran brings into the current delve.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeroWard {
    pub stratum_name: String,
    pub element: String,
    pub mastery: u8,
}

impl HeroWard {
    pub fn label(&self) -> String {
        if self.mastery == 0 || self.element.is_empty() {
            "None".to_string()
        } else {
            format!("{} {}", self.element, roman_rank(self.mastery))
        }
    }

    pub fn attack_multiplier_against(&self, element: &str) -> f32 {
        if self.mastery > 0 && self.element == element {
            1.0 + self.mastery.min(3) as f32 * 0.04
        } else {
            1.0
        }
    }

    pub fn damage_multiplier_from(&self, element: &str) -> f32 {
        if self.mastery > 0 && self.element == element {
            1.0 - self.mastery.min(3) as f32 * 0.08
        } else {
            1.0
        }
    }
}

fn roman_rank(rank: u8) -> &'static str {
    match rank.min(3) {
        1 => "I",
        2 => "II",
        3 => "III",
        _ => "",
    }
}

/// Persistent ledger entry for an adventurer who has entered the dungeon.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeroRecord {
    pub id: u64,
    pub name: String,
    pub class_name: String,
    pub race: String,
    #[serde(default)]
    pub drive: HeroDrive,
    /// Confidence earned or lost through survival. It influences combat and
    /// party nerve when the hero returns; bounded to 20..100 by settlement.
    #[serde(default = "default_resolve")]
    pub resolve: i32,
    pub level: i32,
    pub experience: i32,
    /// Times this hero has entered the dungeon.
    pub delves: i32,
    /// Monsters this hero has slain across all delves.
    pub kills: i32,
    /// Total gold this hero has escaped the dungeon with.
    pub gold_stolen: i32,
    /// Number of delves this hero survived. Kept separately from `delves` so
    /// the ledger can distinguish a frequent visitor from a proven escapee.
    #[serde(default)]
    pub escapes: i32,
    /// Deepest floor this hero has personally reached, whether or not they
    /// returned from it. Old saves begin at the surface.
    #[serde(default)]
    pub deepest_floor: i32,
    /// Strata this hero has escaped and learned to counter. There are only a
    /// handful of authored bands, keeping this collection naturally bounded.
    #[serde(default)]
    pub insights: Vec<HeroInsight>,
    pub status: HeroStatus,
    /// Floor and day of death (only meaningful when status is Dead).
    #[serde(default)]
    pub death_floor: i32,
    #[serde(default)]
    pub death_day: i32,
    /// This hero's own history, newest last. Saves written before the journal
    /// existed simply start empty.
    #[serde(default)]
    pub journal: Vec<HeroEvent>,
}

impl HeroRecord {
    /// A "rival": a recurring survivor (three delves or more) or a prolific
    /// defender-slayer (five kills or more). Rivals are named, marked on the
    /// board, and carry a bounty — the dungeon's grudge made concrete.
    pub fn is_rival(&self) -> bool {
        self.delves >= 3 || self.kills >= 5
    }

    /// Add a line to this hero's history, dropping the oldest once full.
    pub fn remember(&mut self, day: i32, text: impl Into<String>) {
        self.journal.push(HeroEvent {
            day,
            text: text.into(),
        });
        if self.journal.len() > HERO_JOURNAL_LIMIT {
            self.journal.remove(0);
        }
    }

    /// Bounty (souls, gold) for finally slaying this rival, scaled by how much
    /// notoriety they had built raiding the dungeon.
    pub fn bounty(&self) -> (i32, i32) {
        (1 + self.delves / 2, 40 + self.kills * 10)
    }

    /// Strongest known countermeasure, with a later-learned insight winning a
    /// tie. This makes the prepared ward deterministic and visible in saves.
    pub fn prepared_ward(&self) -> HeroWard {
        self.insights
            .iter()
            .enumerate()
            .filter_map(|(index, insight)| {
                let stratum = crate::data::strata::get_stratum(&insight.stratum_id)?;
                Some((index, insight, stratum))
            })
            .max_by_key(|(index, insight, _)| (insight.mastery, *index))
            .map(|(_, insight, stratum)| HeroWard {
                stratum_name: stratum.name.clone(),
                element: stratum.element.clone(),
                mastery: insight.mastery.min(3),
            })
            .unwrap_or_default()
    }

    /// Learn from an escape. Returns the newly strengthened ward so settlement
    /// can write one concise journal line; capped mastery prevents runaway
    /// veteran scaling during very long campaigns.
    pub fn learn_stratum(&mut self, floor: i32, lessons: u8) -> Option<HeroWard> {
        let stratum = crate::data::strata::stratum_for_floor(floor);
        let existing = self
            .insights
            .iter_mut()
            .find(|insight| insight.stratum_id == stratum.id);
        let mastery = match existing {
            Some(insight) => {
                let before = insight.mastery;
                insight.mastery = insight.mastery.saturating_add(lessons).min(3);
                if insight.mastery == before {
                    return None;
                }
                insight.mastery
            }
            None => {
                let mastery = lessons.clamp(1, 3);
                self.insights.push(HeroInsight {
                    stratum_id: stratum.id.clone(),
                    mastery,
                });
                mastery
            }
        };
        Some(HeroWard {
            stratum_name: stratum.name.clone(),
            element: stratum.element.clone(),
            mastery,
        })
    }

    pub fn insight_summary(&self) -> String {
        let labels: Vec<String> = self
            .insights
            .iter()
            .filter_map(|insight| {
                crate::data::strata::get_stratum(&insight.stratum_id).map(|stratum| {
                    format!("{} {}", stratum.short_label(), roman_rank(insight.mastery))
                })
            })
            .collect();
        if labels.is_empty() {
            "None".to_string()
        } else {
            labels.join(" · ")
        }
    }
}

/// Party of adventurers exploring the dungeon
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdventurerParty {
    pub id: u64,
    pub members: Vec<Adventurer>,
    pub current_floor: i32,
    pub current_room: usize,
    pub retreating: bool,
    pub casualties: i32,
    pub loot: i32,
    pub entry_time: i32,
    pub target_floor: i32,
    /// Combat ticks the party is held fast by a snare trap (can't attack)
    #[serde(default)]
    pub snared_ticks: i32,
    /// An alarm trap has alerted the dungeon: monsters fight harder
    #[serde(default)]
    pub alarmed: bool,
    /// Part of the tier-4 siege: marches on the core instead of looting.
    #[serde(default)]
    pub sieging: bool,
    /// Room the party is currently animating out of (only meaningful while
    /// `move_anim` is not ready). Transient — movement is a cosmetic tween.
    #[serde(skip)]
    pub prev_room: usize,
    /// Corridor-travel animation; ready when the party has settled in a room,
    /// armed to [`PARTY_MOVE_SECONDS`] while gliding to the next.
    #[serde(skip, default = "ready_cooldown")]
    pub move_anim: Cooldown,
}
