//! A shaped destination for the prestige loop: named prestige ranks and a
//! milestone track. Prestige used to be an uncapped counter with no arc; ranks
//! give it names to climb, and milestones give the run concrete goals — the
//! "reason to say I finished it" a commercial roguelite needs. The milestone
//! ids also seed the eventual Steam achievement set (Tier 5).

use crate::game_state::{GameState, LogEntry};

/// The prestige count at which the dungeon is considered "ascended" — a soft
/// win state the player can reach and keep playing past.
pub const ASCENSION_PRESTIGE: i32 = 10;

/// A named rank for a band of prestige levels, so the counter reads as a climb
/// rather than a bare number.
pub fn prestige_rank(prestige: i32) -> &'static str {
    match prestige {
        0 => "Fledgling Core",
        1..=2 => "Rooted Core",
        3..=4 => "Dread Warren",
        5..=7 => "Abyssal Throne",
        8..=9 => "Nightmare Bastion",
        _ => "Eternal Core",
    }
}

/// One goal on the milestone track. `tier` groups them by difficulty for
/// display (0 = early, higher = later).
pub struct Milestone {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub tier: u8,
}

/// The milestone catalog. Conditions (see [`met`]) read only *monotonic*
/// counters, so an unlocked milestone can never un-unlock.
pub const MILESTONES: [Milestone; 12] = [
    Milestone {
        id: "first_raid",
        name: "First Blood",
        description: "Weather your first adventurer raid.",
        tier: 0,
    },
    Milestone {
        id: "awakened",
        name: "Awakening",
        description: "Awaken your first core power.",
        tier: 0,
    },
    Milestone {
        id: "deep_delver",
        name: "Going Deep",
        description: "Excavate a dungeon three floors deep.",
        tier: 0,
    },
    Milestone {
        id: "menagerie",
        name: "Menagerie",
        description: "Unlock ten kinds of monster.",
        tier: 1,
    },
    Milestone {
        id: "veteran_keeper",
        name: "Veteran Keeper",
        description: "Weather ten raids.",
        tier: 1,
    },
    Milestone {
        id: "first_prestige",
        name: "Ascendant",
        description: "Repel the realm's siege and claim your first prestige.",
        tier: 1,
    },
    Milestone {
        id: "abyssal",
        name: "Abyssal Warren",
        description: "Excavate a dungeon six floors deep.",
        tier: 2,
    },
    Milestone {
        id: "empowered",
        name: "Empowered Heart",
        description: "Awaken six core powers.",
        tier: 2,
    },
    Milestone {
        id: "renowned",
        name: "Infamous",
        description: "Lure twenty-five heroes to their doom.",
        tier: 2,
    },
    Milestone {
        id: "warden",
        name: "Warden of the Deep",
        description: "Weather fifty raids.",
        tier: 3,
    },
    Milestone {
        id: "dread_sovereign",
        name: "Dread Sovereign",
        description: "Reach prestige three.",
        tier: 3,
    },
    Milestone {
        id: "eternal",
        name: "Eternal Core",
        description: "Reach prestige ten — the dungeon is ascended.",
        tier: 4,
    },
];

/// Look up a milestone by id.
pub fn milestone(id: &str) -> Option<&'static Milestone> {
    MILESTONES.iter().find(|m| m.id == id)
}

/// Whether the given milestone's condition is currently satisfied. Reads only
/// monotonic (never-decreasing) counters so unlocks are permanent.
fn met(state: &GameState, id: &str) -> bool {
    match id {
        "first_raid" => state.raids_completed >= 1,
        "veteran_keeper" => state.raids_completed >= 10,
        "warden" => state.raids_completed >= 50,
        "first_prestige" => state.prestige >= 1,
        "dread_sovereign" => state.prestige >= 3,
        "eternal" => state.prestige >= ASCENSION_PRESTIGE,
        "deep_delver" => state.total_floors >= 3,
        "abyssal" => state.total_floors >= 6,
        "awakened" => !state.core_powers.is_empty(),
        "empowered" => state.core_powers.len() >= 6,
        "menagerie" => state.unlocked_monsters.len() >= 10,
        "renowned" => state.known_adventurers.len() >= 25,
        _ => false,
    }
}

/// Count of milestones the player has achieved.
pub fn achieved_count(state: &GameState) -> usize {
    MILESTONES
        .iter()
        .filter(|m| state.milestones.iter().any(|id| id == m.id))
        .count()
}

/// Evaluate every milestone; unlock any newly-met ones and narrate them. Cheap
/// enough to call each hour. Returns the number newly unlocked this call.
pub fn check_milestones(state: &mut GameState) -> usize {
    let mut newly = 0;
    for m in MILESTONES.iter() {
        let already = state.milestones.iter().any(|id| id == m.id);
        if !already && met(state, m.id) {
            state.milestones.push(m.id.to_string());
            newly += 1;
            state.add_log(LogEntry::system(format!(
                "Milestone achieved — {}: {}",
                m.name, m.description
            )));
            if m.id == "eternal" {
                state.add_log(LogEntry::system(
                    "Your dungeon has ASCENDED. The realm will never be rid of it. Play on, or begin anew.",
                ));
            }
        }
    }
    newly
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_climb_with_prestige() {
        assert_eq!(prestige_rank(0), "Fledgling Core");
        assert_eq!(prestige_rank(1), "Rooted Core");
        assert_eq!(prestige_rank(3), "Dread Warren");
        assert_eq!(prestige_rank(6), "Abyssal Throne");
        assert_eq!(prestige_rank(9), "Nightmare Bastion");
        assert_eq!(prestige_rank(10), "Eternal Core");
        assert_eq!(prestige_rank(99), "Eternal Core");
    }

    #[test]
    fn milestones_unlock_once_and_stay() {
        let mut s = GameState::new();
        assert_eq!(achieved_count(&s), 0);
        s.raids_completed = 10;
        let n = check_milestones(&mut s);
        // first_raid and veteran_keeper both clear at 10 raids.
        assert!(n >= 2);
        assert!(s.milestones.iter().any(|id| id == "first_raid"));
        assert!(s.milestones.iter().any(|id| id == "veteran_keeper"));
        // Re-checking unlocks nothing new, and a counter reset can't revoke it.
        assert_eq!(check_milestones(&mut s), 0);
        s.raids_completed = 0;
        assert!(s.milestones.iter().any(|id| id == "veteran_keeper"));
    }

    #[test]
    fn ascension_milestone_gated_on_prestige_ten() {
        let mut s = GameState::new();
        s.prestige = 9;
        check_milestones(&mut s);
        assert!(!s.milestones.iter().any(|id| id == "eternal"));
        s.prestige = ASCENSION_PRESTIGE;
        check_milestones(&mut s);
        assert!(s.milestones.iter().any(|id| id == "eternal"));
    }

    #[test]
    fn every_milestone_condition_is_wired() {
        // A catalog entry with no matching arm in `met` would silently never
        // unlock — guard against that by construction.
        let mut s = GameState::new();
        s.raids_completed = 1_000;
        s.prestige = 1_000;
        s.total_floors = 1_000;
        for _ in 0..50 {
            s.core_powers.push("x".to_string());
            s.unlocked_monsters.push("m".to_string());
        }
        for i in 0..50 {
            s.known_adventurers.push(crate::game_state::HeroRecord {
                id: i,
                name: String::new(),
                class_name: String::new(),
                race: String::new(),
                level: 1,
                experience: 0,
                delves: 0,
                kills: 0,
                gold_stolen: 0,
                status: crate::game_state::HeroStatus::Alive,
                death_floor: 0,
                death_day: 0,
                journal: Vec::new(),
            });
        }
        for m in MILESTONES.iter() {
            assert!(met(&s, m.id), "milestone '{}' has no wired condition", m.id);
        }
    }
}
