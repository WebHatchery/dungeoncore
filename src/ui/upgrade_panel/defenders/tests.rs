use super::*;

#[test]
fn touch_pager_reaches_every_deep_room_defender_without_overshooting() {
    let total = 5;
    let mut offset = 0;
    for _ in 0..10 {
        offset = paged_defender_offset(offset, total, 1);
    }
    assert_eq!(offset, 3);
    assert_eq!(offset + MAX_DEFENDER_ROWS, total);

    for _ in 0..10 {
        offset = paged_defender_offset(offset, total, -1);
    }
    assert_eq!(offset, 0);
}
