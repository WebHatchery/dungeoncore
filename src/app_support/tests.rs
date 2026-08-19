use super::*;

#[test]
fn responsive_drawer_preserves_board_space_without_hiding_open_catalogues() {
    assert_eq!(responsive_drawer_width(false, false, 1280.0), 72.0);
    assert_eq!(responsive_drawer_width(false, true, 1280.0), 326.0);
    assert_eq!(responsive_drawer_width(false, true, 1024.0), 296.0);
    assert_eq!(responsive_drawer_width(true, true, 800.0), 72.0);
}

#[test]
fn a_suspended_frame_requests_a_pause_but_normal_frames_do_not() {
    assert!(!should_pause_after_frame_gap(1.0 / 60.0));
    assert!(!should_pause_after_frame_gap(SUSPENSION_FRAME_GAP));
    assert!(should_pause_after_frame_gap(0.51));
    assert!(!should_pause_after_frame_gap(f32::NAN));
}
