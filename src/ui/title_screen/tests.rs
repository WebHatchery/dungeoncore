use super::*;

#[test]
fn settings_controls_fit_the_standard_and_narrow_browser_panels() {
    for (width, height) in [(1280.0, 720.0), (390.0, 844.0), (480.0, 640.0)] {
        let (panel, rows) = settings_layout(width, height);
        assert!(panel.x >= 0.0 && panel.right() <= width + 0.1);
        assert!(panel.y >= 0.0 && panel.bottom() <= height + 0.1);
        for (index, row) in rows.iter().enumerate() {
            assert!(row.x >= panel.x && row.right() <= panel.right() + 0.1);
            assert!(row.y >= panel.y && row.bottom() <= panel.bottom() + 0.1);
            assert!(row.h >= 28.0, "settings row {index} is too short");
            if let Some(previous) = rows.get(index.wrapping_sub(1)) {
                if index > 0 {
                    assert!(row.y >= previous.bottom());
                }
            }
        }
    }
}
