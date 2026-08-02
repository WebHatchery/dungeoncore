//! Placing a monster onto one that is already standing there.
//!
//! Two things can happen, and which one it is depends entirely on whether the
//! newcomer belongs to the occupant's own line. A Goblin Warrior placed onto a
//! Goblin is that goblin growing up: it keeps its slot and pays only the
//! difference. A Harpy placed onto a Goblin is an eviction: the goblin is
//! retired for the usual half-refund and the harpy is summoned at full price.
//!
//! The upgrade branch is always the cheaper of the two in mana — that is what
//! makes a monster line worth pursuing rather than simply replacing whatever is
//! standing there with the best thing currently affordable.

use crate::data::constants::{get_monster_mana_cost, get_scaled_stats};
use crate::data::evolutions::get_evolutions_for_monster;
use crate::data::monsters::get_monster_template;
use crate::game_state::{ActiveTrait, GameState, LogEntry, Room, RoomType, Stats};

/// Which of the two things a swap would be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwapKind {
    /// The newcomer is the next form of the occupant's own line.
    Upgrade,
    /// An unrelated monster: the occupant is retired to make way.
    Replace,
}

/// What a swap would do and what it would cost, so the UI can say so before the
/// player commits. Mana is the *net* figure — the replace branch has already
/// had the occupant's refund subtracted.
#[derive(Clone, Debug)]
pub struct SwapPlan {
    pub kind: SwapKind,
    pub mana: i32,
    pub gold: i32,
    pub souls: i32,
}

impl SwapPlan {
    /// Short label for the row: what the click will do and what it takes.
    pub fn label(&self) -> String {
        let verb = match self.kind {
            SwapKind::Upgrade => "Upgrade",
            SwapKind::Replace => "Replace",
        };
        let mut cost = format!("{}M", self.mana);
        if self.gold > 0 {
            cost.push_str(&format!(" {}G", self.gold));
        }
        if self.souls > 0 {
            cost.push_str(&format!(" {}S", self.souls));
        }
        format!("{} · {}", verb, cost)
    }
}

/// Mana refunded for retiring a defender — the same half-price rule dismissal
/// uses, so there is only ever one answer to "what is this creature worth back".
fn retirement_refund(room: &Room, type_name: &str, floor_num: i32) -> i32 {
    get_monster_template(type_name)
        .map(|template| {
            let boss_surcharge = room.room_type == RoomType::Boss && !template.boss_only;
            get_monster_mana_cost(template.base_cost, floor_num, boss_surcharge) / 2
        })
        .unwrap_or(0)
}

/// What placing `new_monster` onto the defender `occupant_id` would do.
/// `None` when the room or creature cannot be found, or the newcomer is unknown.
pub fn plan_swap(
    state: &GameState,
    floor_num: i32,
    room_pos: usize,
    occupant_id: u64,
    new_monster: &str,
) -> Option<SwapPlan> {
    let room = state
        .floors
        .iter()
        .find(|f| f.number == floor_num)?
        .rooms
        .iter()
        .find(|r| r.position == room_pos)?;
    let occupant = room.monsters.iter().find(|m| m.id == occupant_id)?;
    let template = get_monster_template(new_monster)?;

    let boss_surcharge = room.room_type == RoomType::Boss && !template.boss_only;
    let new_cost = get_monster_mana_cost(template.base_cost, floor_num, boss_surcharge);

    // The occupant's own line: one step along an evolution path from what is
    // standing here. Anything else is a stranger, however fine a monster.
    let path = get_evolutions_for_monster(&occupant.type_name)
        .into_iter()
        .find(|p| p.to_monster == new_monster);

    Some(match path {
        Some(path) => {
            let old_cost = get_monster_template(&occupant.type_name)
                .map(|t| {
                    let surcharge = room.room_type == RoomType::Boss && !t.boss_only;
                    get_monster_mana_cost(t.base_cost, floor_num, surcharge)
                })
                .unwrap_or(0);
            SwapPlan {
                kind: SwapKind::Upgrade,
                // Only the difference: the creature is already most of the way
                // there. Never free, so growing a line still costs something.
                mana: (new_cost - old_cost).max(1),
                gold: path.conditions.gold_cost,
                souls: template.souls_cost,
            }
        }
        None => SwapPlan {
            kind: SwapKind::Replace,
            mana: new_cost - retirement_refund(room, &occupant.type_name, floor_num),
            gold: 0,
            souls: template.souls_cost,
        },
    })
}

/// Place `new_monster` onto the defender `occupant_id`, upgrading it in place or
/// evicting it, per [`plan_swap`].
pub fn swap_monster(
    state: &mut GameState,
    floor_num: i32,
    room_pos: usize,
    occupant_id: u64,
    new_monster: &str,
) -> Result<(), String> {
    // Restructuring a garrison mid-raid would let the player heal a wounded
    // defender to full by "upgrading" it while it is being hit.
    if !state.adventurer_parties.is_empty() {
        return Err("Cannot restructure defenders while adventurers are in the dungeon!".into());
    }

    let template = get_monster_template(new_monster)
        .ok_or_else(|| format!("Unknown monster: {new_monster}"))?;
    if !state.unlocked_species.contains(&template.species) {
        return Err(format!("Species '{}' is not unlocked!", template.species));
    }
    if !state.unlocked_monsters.contains(&template.name) {
        return Err(format!("{} is not unlocked yet!", template.name));
    }

    let plan = plan_swap(state, floor_num, room_pos, occupant_id, new_monster)
        .ok_or("That defender is no longer there")?;

    // Everything is checked before anything is destroyed — a failed swap must
    // never cost the player the creature that was already standing there.
    let is_boss_room = state
        .floors
        .iter()
        .find(|f| f.number == floor_num)
        .and_then(|f| f.rooms.iter().find(|r| r.position == room_pos))
        .map(|r| r.room_type == RoomType::Boss)
        .ok_or("Room not found")?;
    if template.boss_only && !is_boss_room {
        return Err(format!("{} belongs in a Boss room!", template.name));
    }
    if state.mana < plan.mana {
        return Err(format!("Not enough mana! Need {}.", plan.mana));
    }
    if state.gold < plan.gold {
        return Err(format!("Not enough gold! Need {}.", plan.gold));
    }
    if state.souls < plan.souls {
        return Err(format!("Not enough souls! Need {}.", plan.souls));
    }

    match plan.kind {
        SwapKind::Upgrade => {
            upgrade_in_place(state, floor_num, room_pos, occupant_id, new_monster, &plan)
        }
        SwapKind::Replace => {
            // Retire then summon: the refund lands first, which is exactly why
            // the affordability check above uses the net figure.
            super::remove_monster(state, floor_num, room_pos, occupant_id)?;
            super::place_monster(state, floor_num, room_pos, new_monster)
        }
    }
}

#[cfg(test)]
mod tests;

/// Grow a creature into the next form of its own line: same slot, same id, the
/// new form's stats scaled for the floor it stands on.
fn upgrade_in_place(
    state: &mut GameState,
    floor_num: i32,
    room_pos: usize,
    occupant_id: u64,
    new_monster: &str,
    plan: &SwapPlan,
) -> Result<(), String> {
    let template = get_monster_template(new_monster).ok_or("Unknown monster")?;

    state.mana -= plan.mana;
    state.gold -= plan.gold;
    state.souls -= plan.souls;

    let room = state
        .floors
        .iter_mut()
        .find(|f| f.number == floor_num)
        .and_then(|f| f.rooms.iter_mut().find(|r| r.position == room_pos))
        .ok_or("Room not found")?;
    let monster = room
        .monsters
        .iter_mut()
        .find(|m| m.id == occupant_id)
        .ok_or("Monster not found")?;

    let old_name = monster.type_name.clone();
    let scaled = get_scaled_stats(
        Stats {
            hp: template.hp,
            attack: template.attack,
            defense: template.defense,
        },
        floor_num,
        monster.is_boss,
    );
    monster.type_name = template.name.clone();
    monster.hp = scaled.hp;
    monster.max_hp = scaled.hp;
    monster.alive = true;
    monster.scaled_stats = scaled;
    monster.active_traits = template
        .traits
        .iter()
        .map(|trait_id| ActiveTrait {
            id: trait_id.clone(),
            name: crate::data::traits::get_trait(trait_id)
                .map(|t| t.name)
                .unwrap_or_else(|| trait_id.clone()),
            cooldown_timer: 0,
        })
        .collect();

    state.add_log(LogEntry::building(format!(
        "{} grew into {} on floor {}, room {}.",
        old_name, template.name, floor_num, room_pos
    )));
    Ok(())
}
