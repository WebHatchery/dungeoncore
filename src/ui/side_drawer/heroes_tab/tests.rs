use super::*;

#[test]
fn touch_roster_pager_reaches_the_last_known_hero_and_returns() {
    let total = 23;
    let visible = 9;
    let mut offset = 0;
    for _ in 0..30 {
        offset = hero_page_offset(offset, total, visible, 1);
    }
    assert_eq!(offset, total - visible);

    for _ in 0..30 {
        offset = hero_page_offset(offset, total, visible, -1);
    }
    assert_eq!(offset, 0);
}
