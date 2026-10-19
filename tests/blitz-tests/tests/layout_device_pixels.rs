//! Layout is rounded to whole device pixels, not CSS pixels. At a
//! fractional scale (4/3 here) rounding in CSS pixels put box edges
//! between device pixels: a `1px` border, which Stylo snaps to one device
//! pixel (0.75 CSS px), became 1 or 0 CSS px depending on where the box
//! was, so thin borders vanished on some sides (and rounded ones were
//! split into NaN edge paths).

use blitz_test_harness::{Harness, HarnessOptions};

fn at(scale: f32, body: &str) -> Harness {
    Harness::from_html_with(
        &format!("<body style='margin:0'>{body}</body>"),
        HarnessOptions {
            scale,
            ..Default::default()
        },
    )
}

#[test]
fn one_px_borders_stay_one_device_pixel_at_a_fractional_scale() {
    let scale = 4.0 / 3.0;
    let device = 1.0 / scale;
    for top in ["0px", "0.25px", "0.375px", "0.5px", "0.6px", "1px", "1.5px"] {
        let h = at(
            scale,
            &format!(
                "<div id='b' style='margin-top:{top}; width:100px; height:50px; border:1px solid black'></div>"
            ),
        );
        let id = h.node("#b");
        let border = h.base().get_node(id).unwrap().final_layout().border;
        for (side, w) in [
            ("left", border.left),
            ("right", border.right),
            ("top", border.top),
            ("bottom", border.bottom),
        ] {
            assert!(
                (w - device).abs() < 1e-4,
                "margin-top {top}: {side} border {w} CSS px, one device pixel is {device}"
            );
        }
        // The box's edges lie on device pixels.
        let r = h.layout_rect("#b");
        for edge in [r.x, r.y, r.x + r.width, r.y + r.height] {
            let d = edge * scale;
            assert!(
                (d - d.round()).abs() < 1e-3,
                "margin-top {top}: an edge at {edge} CSS px is {d} device px"
            );
        }
    }
}

#[test]
fn at_scale_one_layout_is_rounded_to_whole_pixels_as_before() {
    let h = at(
        1.0,
        "<div id='b' style='margin-top:0.6px; width:10.4px; height:5.5px'></div>",
    );
    let r = h.layout_rect("#b");
    // Edges at round(0.6) = 1 and round(6.1) = 6.
    assert_eq!((r.y, r.width, r.height), (1.0, 10.0, 5.0));
}
