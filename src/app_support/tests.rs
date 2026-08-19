use super::*;

#[test]
fn responsive_drawer_preserves_board_space_without_hiding_open_catalogues() {
    assert_eq!(responsive_drawer_width(false, false, 1280.0), 72.0);
    assert_eq!(responsive_drawer_width(false, true, 1280.0), 326.0);
    assert_eq!(responsive_drawer_width(false, true, 1024.0), 296.0);
    assert_eq!(responsive_drawer_width(true, true, 800.0), 72.0);
}
