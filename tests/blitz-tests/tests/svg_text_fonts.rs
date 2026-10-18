//! Fonts can be registered for SVG text (`@font-face` fonts are registered
//! the same way when they load), and an embedder can drop the system fonts.

use blitz_dom::util::{
    register_svg_font, reset_svg_fonts_without_system_fonts, svg_font_families,
};

#[test]
fn registering_a_font_for_svg_text_is_idempotent_and_ignores_garbage() {
    // Not a font: ignored (fontdb skips data it cannot parse).
    register_svg_font(b"not a font");
    register_svg_font(&[]);
}

#[test]
fn the_svg_font_database_can_be_reset_to_no_system_fonts() {
    reset_svg_fonts_without_system_fonts();
    register_svg_font(b"not a font");
    assert!(svg_font_families().is_empty());
}
