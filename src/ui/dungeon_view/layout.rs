//! Pure layered layout for a floor's directed acyclic room graph.

use std::collections::{HashMap, VecDeque};

use crate::game_state::{Floor, RoomType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RoomLayout {
    pub position: usize,
    pub depth: usize,
    pub lane: i32,
}

/// Assign columns by graph depth and lanes by fork order. A reconverging room
/// retains the first lane that reaches it, keeping the main route stable while
/// parallel paths visibly peel away and return.
pub(super) fn layout_floor(floor: &Floor) -> Vec<RoomLayout> {
    let Some(entrance) = floor
        .rooms
        .iter()
        .find(|room| room.room_type == RoomType::Entrance)
        .map(|room| room.position)
    else {
        return Vec::new();
    };
    let mut depths = HashMap::from([(entrance, 0usize)]);
    let mut lanes = HashMap::from([(entrance, 0i32)]);
    let mut queue = VecDeque::from([entrance]);
    while let Some(position) = queue.pop_front() {
        let Some(room) = floor.room_at(position) else {
            continue;
        };
        let depth = depths[&position];
        let lane = lanes[&position];
        for (index, &exit) in room.exits.iter().enumerate() {
            let next_depth = depth + 1;
            let entry = depths.entry(exit).or_insert(next_depth);
            if next_depth > *entry {
                *entry = next_depth;
            }
            lanes.entry(exit).or_insert_with(|| {
                if room.exits.len() == 1 {
                    lane
                } else {
                    lane + index as i32
                }
            });
            if !queue.contains(&exit) {
                queue.push_back(exit);
            }
        }
    }
    let lane_shift = lanes.values().copied().min().unwrap_or(0);
    let mut layout: Vec<_> = depths
        .into_iter()
        .map(|(position, depth)| RoomLayout {
            position,
            depth,
            lane: lanes[&position] - lane_shift,
        })
        .collect();
    layout.sort_by_key(|node| (node.depth, node.lane, node.position));
    layout
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::{Room, RoomType};

    #[test]
    fn a_diamond_gets_two_lanes_and_a_shared_later_column() {
        let mut floor = Floor::new(1, 1, true);
        floor.rooms = vec![
            Room::new(1, RoomType::Entrance, 0, 1),
            Room::new(2, RoomType::Normal, 1, 1),
            Room::new(3, RoomType::Normal, 2, 1),
            Room::new(4, RoomType::Core, 3, 1),
        ];
        floor.room_at(0).unwrap();
        floor.rooms[0].exits = vec![1, 2];
        floor.rooms[1].exits = vec![3];
        floor.rooms[2].exits = vec![3];
        let layout = layout_floor(&floor);
        let first = layout.iter().find(|node| node.position == 1).unwrap();
        let second = layout.iter().find(|node| node.position == 2).unwrap();
        let core = layout.iter().find(|node| node.position == 3).unwrap();
        assert_eq!(first.depth, second.depth);
        assert_ne!(first.lane, second.lane);
        assert_eq!(core.depth, first.depth + 1);
    }
}
