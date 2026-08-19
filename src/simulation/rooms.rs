use crate::data::constants::{get_room_cost, CORE_ROOM_MANA_BONUS, MAX_ROOMS_PER_FLOOR};
use crate::game_state::{Floor, GameState, LogEntry, Room, RoomType};

/// Add a room to the dungeon
pub fn add_room(state: &mut GameState, target_floor: Option<i32>) -> Result<(), String> {
    // Cannot add rooms while adventurers are in dungeon
    if !state.adventurer_parties.is_empty() {
        return Err("Cannot add rooms while adventurers are in the dungeon!".into());
    }

    // Find target floor
    let floor_num = target_floor.unwrap_or_else(|| {
        state
            .floors
            .iter()
            .find(|f| f.is_deepest)
            .map(|f| f.number)
            .unwrap_or(1)
    });

    let floor_idx = state
        .floors
        .iter()
        .position(|f| f.number == floor_num)
        .ok_or("Floor not found")?;

    // Count non-core rooms on this floor
    let non_core_count = state.floors[floor_idx]
        .rooms
        .iter()
        .filter(|r| r.room_type != RoomType::Core)
        .count();

    // Check if floor is full (entrance + 5 normal/boss rooms)
    if non_core_count > MAX_ROOMS_PER_FLOOR {
        // Create a new floor
        return create_new_floor(state);
    }

    // Calculate cost
    let total_rooms = state.total_room_count();
    let next_pos = non_core_count;
    let is_boss = next_pos == MAX_ROOMS_PER_FLOOR;
    let cost = get_room_cost(total_rooms, is_boss);

    if state.mana < cost {
        return Err(format!("Not enough mana! Need {} mana.", cost));
    }

    state.mana -= cost;

    // Create new room
    let room_type = if is_boss {
        RoomType::Boss
    } else {
        RoomType::Normal
    };
    let has_fork = state.floors[floor_idx]
        .rooms
        .iter()
        .any(|room| room.exits.len() > 1);
    let new_position = if has_fork {
        state.floors[floor_idx]
            .rooms
            .iter()
            .map(|room| room.position)
            .max()
            .unwrap_or(0)
            + 1
    } else {
        next_pos
    };
    let new_room = Room::new(
        state.run_rng.next_u64(),
        room_type.clone(),
        new_position,
        floor_num,
    );

    // A linear floor inserts before (and renumbers) the core as it always did.
    // A branched floor instead inserts a shared room immediately before the
    // core by redirecting every reconverging edge, preserving its diamonds.
    let floor = &mut state.floors[floor_idx];
    if has_fork {
        let core_pos = floor
            .rooms
            .iter()
            .find(|room| room.room_type == RoomType::Core)
            .map(|room| room.position)
            .ok_or("Floor has no core")?;
        floor.rooms.push(new_room);
        for room in &mut floor.rooms {
            for exit in &mut room.exits {
                if *exit == core_pos {
                    *exit = new_position;
                }
            }
        }
        floor
            .rooms
            .iter_mut()
            .find(|room| room.position == new_position)
            .expect("new shared room")
            .exits = vec![core_pos];
    } else if let Some(core_idx) = floor
        .rooms
        .iter()
        .position(|r| r.room_type == RoomType::Core)
    {
        floor.rooms.insert(core_idx, new_room);
        // Update core room position
        floor.rooms[core_idx + 1].position = next_pos + 1;
    } else {
        floor.rooms.push(new_room);
    }
    if !has_fork {
        floor.rebuild_linear_exits();
    }
    floor.validate_graph()?;

    let room_name = if is_boss { "Boss room" } else { "Normal room" };
    state.add_log(LogEntry::building(format!(
        "{} added to floor {} for {} mana.",
        room_name, floor_num, cost
    )));

    Ok(())
}

/// Cost and validate a parallel room growing from `source_pos`. The new room
/// joins the source's existing successor, preserving an Entrance-to-Core route
/// on both sides of the diamond.
pub fn branch_cost(state: &GameState, floor_num: i32, source_pos: usize) -> Result<i32, String> {
    if !state.adventurer_parties.is_empty() {
        return Err("Cannot reshape the dungeon while adventurers are inside!".into());
    }
    let floor = state
        .floors
        .iter()
        .find(|floor| floor.number == floor_num)
        .ok_or("Floor not found")?;
    let source = floor.room_at(source_pos).ok_or("Room not found")?;
    if source.exits.len() != 1 {
        return Err("Choose a room with exactly one route ahead to branch from it.".into());
    }
    if source.room_type == RoomType::Core {
        return Err("The core has no route beyond it to branch.".into());
    }
    if floor
        .rooms
        .iter()
        .filter(|room| room.room_type != RoomType::Core)
        .count()
        > MAX_ROOMS_PER_FLOOR
    {
        return Err("This floor has no room for another branch.".into());
    }
    Ok(get_room_cost(state.total_room_count(), false))
}

/// Create one side of a series-parallel diamond. The source keeps its original
/// route and gains the new one; the newcomer points to the same successor.
pub fn branch_from(state: &mut GameState, floor_num: i32, source_pos: usize) -> Result<(), String> {
    let cost = branch_cost(state, floor_num, source_pos)?;
    if state.mana < cost {
        return Err(format!("Not enough mana! Need {cost} mana."));
    }
    let floor_idx = state
        .floors
        .iter()
        .position(|floor| floor.number == floor_num)
        .ok_or("Floor not found")?;
    let next_pos = state.floors[floor_idx]
        .rooms
        .iter()
        .map(|room| room.position)
        .max()
        .unwrap_or(0)
        + 1;
    let successor = state.floors[floor_idx]
        .room_at(source_pos)
        .and_then(|room| room.exits.first().copied())
        .ok_or("The chosen room has no route to branch.")?;

    state.mana -= cost;
    let floor = &mut state.floors[floor_idx];
    floor.rooms.push(Room::new(
        state.run_rng.next_u64(),
        RoomType::Normal,
        next_pos,
        floor_num,
    ));
    floor
        .rooms
        .last_mut()
        .expect("newly pushed branch room")
        .exits = vec![successor];
    floor
        .rooms
        .iter_mut()
        .find(|room| room.position == source_pos)
        .expect("validated branch source")
        .exits
        .push(next_pos);
    floor.validate_graph()?;

    state.add_log(LogEntry::building(format!(
        "A parallel room branches from room {} on floor {} for {} mana.",
        source_pos, floor_num, cost
    )));
    Ok(())
}

/// Create a new floor in the dungeon
fn create_new_floor(state: &mut GameState) -> Result<(), String> {
    let total_rooms = state.total_room_count();
    let cost = get_room_cost(total_rooms, false);

    if state.mana < cost {
        return Err(format!(
            "Not enough mana! Need {} mana to create new floor.",
            cost
        ));
    }

    state.mana -= cost;
    let new_floor_num = state.total_floors + 1;

    // Remove core from previous deepest floor
    for floor in &mut state.floors {
        if floor.is_deepest {
            floor.is_deepest = false;
            floor.rooms.retain(|r| r.room_type != RoomType::Core);
            // The old Core was the sink for every route. Once it moves to the
            // new floor, discard those stale edge targets so visitors end at
            // the last room and descend instead of walking into a missing
            // room forever. This matters most at the maximum-dungeon soak,
            // where every earlier floor loses its Core in turn.
            let valid_positions: std::collections::HashSet<usize> =
                floor.rooms.iter().map(|room| room.position).collect();
            for room in &mut floor.rooms {
                room.exits.retain(|exit| valid_positions.contains(exit));
            }
        }
    }

    // Create new floor
    let mut new_floor = Floor::new(state.run_rng.next_u64(), new_floor_num, true);
    new_floor.rooms.push(Room::new(
        state.run_rng.next_u64(),
        RoomType::Entrance,
        0,
        new_floor_num,
    ));
    new_floor.rooms.push(Room::new(
        state.run_rng.next_u64(),
        RoomType::Normal,
        1,
        new_floor_num,
    ));
    new_floor.rooms.push(Room::new(
        state.run_rng.next_u64(),
        RoomType::Core,
        2,
        new_floor_num,
    ));
    new_floor.rebuild_linear_exits();

    state.floors.push(new_floor);
    state.total_floors += 1;
    state.deep_core_bonus = state.total_floors as f32 * CORE_ROOM_MANA_BONUS;

    // A deeper core holds more mana — keeps late-tier summons affordable.
    state.max_mana += 50;

    state.add_log(LogEntry::building(format!(
        "New floor {} created for {} mana! Deep core bonus: +{}%, max mana {}",
        new_floor_num,
        cost,
        (state.deep_core_bonus * 100.0) as i32,
        state.max_mana
    )));

    Ok(())
}

#[cfg(test)]
mod tests;
