//! Inline text boxes keep their fractional widths. Rounding every inline
//! width up to whole pixels pushes each following box right by up to a
//! pixel, so rows of labels drift (5 px over a toolbar of ~20 labels).
//!
//! NOTE: measures real text, so it needs a usable font (the `system-fonts`
//! feature); without one the widths are 0 and the test passes vacuously.

use blitz_test_harness::Harness;

#[test]
fn labels_in_a_row_advance_by_their_unrounded_widths() {
    let row: String = (0..10)
        .map(|i| format!("<span id='s{i}'>Mein Layout</span>"))
        .collect();
    let h = Harness::from_html(&format!(
        "<body style='margin:0; font: 13.3px sans-serif'><div style='display:flex'>{row}</div></body>"
    ));
    let xs: Vec<f32> = (0..10)
        .map(|i| h.layout_rect(&format!("#s{i}")).x)
        .collect();
    let steps: Vec<f32> = xs.windows(2).map(|w| w[1] - w[0]).collect();
    if steps.iter().all(|s| *s == 0.0) {
        return; // no font
    }
    // Box edges are rounded, widths are not: with a fractional text width
    // the steps between labels differ (e.g. 70, 71, 70 …); rounded-up
    // widths would make them all equal.
    assert!(
        steps.iter().any(|s| *s != steps[0]),
        "every label advances by the same whole number of pixels: {steps:?}"
    );
}
