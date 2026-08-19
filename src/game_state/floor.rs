//! A dungeon floor: its rooms and the directed room-graph connecting them
//! (entrance source, core sink), plus the graph-building/validation logic
//! (Phase A of the dungeon-graph work).

use serde::{Deserialize, Serialize};

use super::{Room, RoomType};

/// Floor in the dungeon
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Floor {
    pub id: u64,
    pub number: i32,
    pub rooms: Vec<Room>,
    pub is_deepest: bool,
}

impl Floor {
    pub fn new(id: u64, number: i32, is_deepest: bool) -> Self {
        Self {
            id,
            number,
            rooms: Vec::new(),
            is_deepest,
        }
    }

    /// The room at a given position key, if any.
    pub fn room_at(&self, position: usize) -> Option<&Room> {
        self.rooms.iter().find(|r| r.position == position)
    }

    pub fn room_at_mut(&mut self, position: usize) -> Option<&mut Room> {
        self.rooms.iter_mut().find(|room| room.position == position)
    }

    /// Wire every room's `exits` as a single linear chain in ascending
    /// `position` order (each room points at the next; the last has none). Used
    /// to migrate pre-graph saves and to seed freshly-built linear floors.
    pub fn rebuild_linear_exits(&mut self) {
        let mut positions: Vec<usize> = self.rooms.iter().map(|r| r.position).collect();
        positions.sort_unstable();
        let next: std::collections::HashMap<usize, usize> =
            positions.windows(2).map(|w| (w[0], w[1])).collect();
        for room in &mut self.rooms {
            room.exits = next
                .get(&room.position)
                .map(|&n| vec![n])
                .unwrap_or_default();
        }
    }

    /// Validate the floor graph: a single Entrance source and single Core sink,
    /// every room reachable from the Entrance, and every room able to reach the
    /// Core (no orphans, no dead ends). Returns Err with the first problem.
    pub fn validate_graph(&self) -> Result<(), String> {
        use std::collections::HashSet;
        let entrances: Vec<usize> = self
            .rooms
            .iter()
            .filter(|r| r.room_type == RoomType::Entrance)
            .map(|r| r.position)
            .collect();
        let cores: Vec<usize> = self
            .rooms
            .iter()
            .filter(|r| r.room_type == RoomType::Core)
            .map(|r| r.position)
            .collect();
        if entrances.len() != 1 {
            return Err(format!("floor {} needs exactly one entrance", self.number));
        }
        if cores.len() != 1 {
            return Err(format!("floor {} needs exactly one core", self.number));
        }
        let core = cores[0];

        // Reachability from the entrance (forward BFS over `exits`).
        let mut seen = HashSet::new();
        let mut stack = vec![entrances[0]];
        while let Some(pos) = stack.pop() {
            if !seen.insert(pos) {
                continue;
            }
            if let Some(room) = self.room_at(pos) {
                for &next in &room.exits {
                    if self.room_at(next).is_none() {
                        return Err(format!(
                            "floor {}: room {} exits to missing room {}",
                            self.number, pos, next
                        ));
                    }
                    stack.push(next);
                }
            }
        }
        for room in &self.rooms {
            if !seen.contains(&room.position) {
                return Err(format!(
                    "floor {}: room {} is unreachable from the entrance",
                    self.number, room.position
                ));
            }
            if room.position != core && room.exits.is_empty() {
                return Err(format!(
                    "floor {}: room {} is a dead end (no path to the core)",
                    self.number, room.position
                ));
            }
        }
        Ok(())
    }
}
