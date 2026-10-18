//! A shrink-to-fit box (flex item, inline-block, absolutely positioned box,
//! button) is sized to its text's max-content width. Inline widths are not
//! rounded to whole pixels, so the width taffy hands back for the relayout can
//! differ from the measured max-content width by float noise (e.g. after
//! `/ scale`, `* scale` and subtracting fractional padding/border). That
//! relayout must not wrap the text onto a second line.
//!
//! Uses an embedded font (DejaVu Sans) with system fonts disabled, so the
//! test measures real text on every machine.

use blitz_dom::{DocumentConfig, build_single_font_ctx};
use blitz_html::HtmlDocument;
use blitz_test_harness::Harness;
use blitz_traits::shell::{ColorScheme, Viewport};

const FONT: &[u8] = include_bytes!("../../../examples/todomvc/assets/DejaVuSans.woff2");

const WORDS: &[&str] = &[
    "Mein Layout",
    "BTCUSDT 1h",
    "Trendlinie zeichnen",
    "64 213,57",
    "Alarm gesetzt",
    "Indikatoren",
    "Fibonacci-Retracement",
    "Kauf Limit 0,5",
];

#[test]
fn shrink_to_fit_labels_stay_on_one_line() {
    let mut wrapped = Vec::new();
    let mut checked = 0;
    for scale in [1.0f32, 1.25, 1.5] {
        for k in 0..40 {
            let size = 11.0 + k as f32 * 0.137;
            let mut body = String::new();
            for (i, w) in WORDS.iter().enumerate() {
                body.push_str(&format!(
                    "<div style='display:flex'><span id='f{i}' style='padding:0 3.3px; border:0.7px solid red'>{w}</span></div>\
                     <div><span id='b{i}' style='display:inline-block; padding-left:1.1px'>{w}</span></div>\
                     <div style='position:relative'><div id='a{i}' style='position:absolute'>{w}</div></div>\
                     <div style='display:flex'><button id='c{i}'>{w}</button></div>"
                ));
            }
            let page = format!(
                "<body style='margin:0; font: {size}px sans-serif; line-height: 20px'>{body}</body>"
            );
            let config = DocumentConfig {
                viewport: Some(Viewport::new(1600, 2000, scale, ColorScheme::Light)),
                font_ctx: Some(build_single_font_ctx(FONT)),
                ..Default::default()
            };
            let mut h = Harness::wrap(HtmlDocument::from_html(&page, config));
            h.pump();
            for (i, w) in WORDS.iter().enumerate() {
                for p in ["f", "b", "a", "c"] {
                    checked += 1;
                    let r = h.layout_rect(&format!("#{p}{i}"));
                    // Not vacuous: the font is embedded, so text has a width.
                    assert!(r.width > 30.0, "{p}{i} has no text width: {r:?}");
                    if r.height > 30.0 {
                        wrapped.push(format!("scale {scale} size {size} {p}{i} '{w}' {r:?}"));
                    }
                }
            }
        }
    }
    assert!(
        wrapped.is_empty(),
        "{} of {checked} labels wrapped:\n{}",
        wrapped.len(),
        wrapped.join("\n")
    );
}
