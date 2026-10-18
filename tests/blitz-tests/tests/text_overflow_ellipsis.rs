//! `text-overflow: ellipsis` ends a line that does not fit its box in "…".
//!
//! NOTE: measures real text, so it needs a usable font.

use anyrender::render_to_buffer;
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_test_harness::Harness;

fn render(html: &str) -> Vec<u8> {
    let mut h = Harness::from_html(html);
    render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| {
            let mut doc = h.base_mut();
            blitz_paint::paint_scene(scene, &mut doc, 1.0, 120, 20, 0, 0);
        },
        120,
        20,
    )
}

#[test]
fn a_long_line_ends_in_an_ellipsis_and_a_short_one_is_left_alone() {
    let page = |style: &str, text: &str| {
        render(&format!(
            "<body style='margin:0;background:#fff;color:#000;font:10px monospace'>\
             <div style='width:60px;white-space:nowrap;overflow:hidden;{style}'>{text}</div></body>"
        ))
    };
    let cut = page("text-overflow:ellipsis", "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let clipped = page("", "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    assert_ne!(cut, clipped, "the long line is only clipped");
    assert_eq!(page("text-overflow:ellipsis", "ABC"), page("", "ABC"));
}

/// Trailing (hanging) whitespace does not overflow a box: lines whose ink
/// fits are painted exactly as without `text-overflow`.
#[test]
fn trailing_whitespace_does_not_trigger_the_ellipsis() {
    let page = |style: &str, width: u32, text: &str| {
        render(&format!(
            "<body style='margin:0;background:#fff;color:#000;font:10px monospace'>\
             <div style='width:{width}px;overflow:hidden;{style}'>{text}</div></body>"
        ))
    };
    for width in 20..=80 {
        let (wrap, plain) = (
            page("text-overflow:ellipsis", width, "AAAA BBBB CCCC"),
            page("", width, "AAAA BBBB CCCC"),
        );
        // A word wider than the box legitimately gets an ellipsis.
        let words_fit = page("text-overflow:ellipsis", width, "AAAA") == page("", width, "AAAA");
        assert!(
            !words_fit || wrap == plain,
            "wrapping box of {width}px got an ellipsis"
        );
    }
    for width in 20..=80 {
        let nowrap = "white-space:nowrap;";
        let ellipsis = format!("{nowrap}text-overflow:ellipsis");
        let ink_fits = page(&ellipsis, width, "ABCDE") == page(nowrap, width, "ABCDE");
        assert!(
            !ink_fits || page(&ellipsis, width, "ABCDE ") == page(nowrap, width, "ABCDE "),
            "nowrap box of {width}px with a trailing space got an ellipsis"
        );
    }
}
