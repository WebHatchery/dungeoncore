use super::*;

#[test]
fn new_depth_forms_receive_their_element_and_tier_marks() {
    assert_eq!(
        adornment_style("Storm Drake"),
        Some(AdornmentStyle {
            tier: 2,
            element: ElementMark::Air,
        })
    );
    assert_eq!(
        adornment_style("Death Tide"),
        Some(AdornmentStyle {
            tier: 3,
            element: ElementMark::Water,
        })
    );
    assert_eq!(
        adornment_style("Astral Gel"),
        Some(AdornmentStyle {
            tier: 3,
            element: ElementMark::Arcane,
        })
    );
}

#[test]
fn a_hatchling_and_an_elder_read_as_different_progression_stages() {
    let hatchling = adornment_style("Cave Wyrmling").unwrap();
    let elder = adornment_style("Elder Dragon").unwrap();
    assert_eq!(hatchling.tier, 1);
    assert_eq!(elder.tier, 4);
    assert_ne!(hatchling.element, elder.element);
}
