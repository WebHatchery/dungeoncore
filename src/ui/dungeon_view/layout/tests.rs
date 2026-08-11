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
