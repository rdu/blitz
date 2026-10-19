//! Layout is rounded to whole device pixels, not CSS pixels. At a
//! fractional scale rounding in CSS pixels put box edges between device
//! pixels: a `1px` border, which Stylo snaps to whole device pixels (0.75
//! CSS px at 4/3), became 1 or 0 CSS px depending on where the box was, so
//! thin borders vanished on some sides (and rounded ones were split into
//! NaN edge paths). Stylo's device pixel is a whole number of app units
//! (`60 / scale`, truncated), so at e.g. 1.75 its `1px` border is slightly
//! less than one device pixel (0.9917): border widths are snapped on their
//! own, not as differences of rounded edges.

use blitz_test_harness::{Harness, HarnessOptions};

const SCALES: [f32; 5] = [1.25, 4.0 / 3.0, 1.5, 1.75, 2.25];
const OFFSETS: [&str; 15] = [
    "0px", "1px", "2px", "3px", "4px", "5px", "6px", "7px", "8px", "0.25px", "0.375px", "0.5px",
    "0.6px", "1.5px", "2.5px",
];

fn at(scale: f32, body: &str) -> Harness {
    Harness::from_html_with(
        &format!("<body style='margin:0'>{body}</body>"),
        HarnessOptions {
            scale,
            ..Default::default()
        },
    )
}

/// `edge` CSS px lies on a device pixel.
fn on_device_pixel(edge: f32, scale: f32) -> bool {
    let d = edge * scale;
    (d - d.round()).abs() < 1e-3
}

#[test]
fn one_px_borders_stay_whole_device_pixels_at_fractional_scales() {
    for scale in SCALES {
        // Stylo snaps `1px` down to whole device pixels, at least one.
        let device_px = scale.floor().max(1.0);
        for off in OFFSETS {
            let h = at(
                scale,
                &format!(
                    "<div style='margin:{off} 0 0 {off}'>\
                     <div id='b' style='width:100px; height:10.3px; border:1px solid black'></div>\
                     <div id='c' style='width:100px; height:7.7px; border:1px solid black'></div>\
                     <div id='d' style='width:100px; height:5px; border:1px solid black'></div>\
                     </div>"
                ),
            );
            let mut prev_bottom: Option<f32> = None;
            for sel in ["#b", "#c", "#d"] {
                let id = h.node(sel);
                let border = h.base().get_node(id).unwrap().final_layout().border;
                for (side, w) in [
                    ("left", border.left),
                    ("right", border.right),
                    ("top", border.top),
                    ("bottom", border.bottom),
                ] {
                    assert!(
                        (w * scale - device_px).abs() < 1e-3,
                        "scale {scale}, offset {off}: {sel} {side} border is {} device px, \
                         not {device_px}",
                        w * scale
                    );
                }
                let r = h.layout_rect(sel);
                for edge in [r.x, r.y, r.x + r.width, r.y + r.height] {
                    assert!(
                        on_device_pixel(edge, scale),
                        "scale {scale}, offset {off}: {sel} has an edge at {edge} CSS px, {} \
                         device px",
                        edge * scale
                    );
                }
                // Stacked boxes neither leave a gap nor overlap.
                if let Some(bottom) = prev_bottom {
                    assert!(
                        ((r.y - bottom) * scale).abs() < 1e-3,
                        "scale {scale}, offset {off}: {sel} starts at {} CSS px, the box above \
                         ends at {bottom}",
                        r.y
                    );
                }
                prev_bottom = Some(r.y + r.height);
            }
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

#[test]
fn boxes_inside_a_border_start_after_it() {
    // A child that fills the content box must not cover a device pixel of
    // its parent's border (its background would hide that side).
    for scale in SCALES {
        for off in OFFSETS {
            let h = at(
                scale,
                &format!(
                    "<div id='b' style='margin:{off} 0 0 {off}; width:100.3px; height:20px; \
                     border:1px solid black'><div id='k' style='height:5.2px'></div></div>"
                ),
            );
            let b = h.layout_rect("#b");
            let k = h.layout_rect("#k");
            let bl = h
                .base()
                .get_node(h.node("#b"))
                .unwrap()
                .final_layout()
                .border;
            for (side, gap) in [
                ("left", k.x - (b.x + bl.left)),
                ("top", k.y - (b.y + bl.top)),
                ("right", (b.x + b.width - bl.right) - (k.x + k.width)),
            ] {
                assert!(
                    (gap * scale).abs() < 1e-3,
                    "scale {scale}, offset {off}: the child is {} device px off the {side} \
                     border's inner edge",
                    gap * scale
                );
            }
        }
    }
}
