//! Clicking into, dragging over and focusing a text input never panics when
//! its layout is not ready: an editor whose layout is dirty (it is rebuilt by
//! the next `resolve`) places no cursor this time, and a containing block that
//! no longer exists gives no IME cursor area.

use blitz_dom::{DocumentConfig, EventDriver, NodeId, NoopEventHandler};
use blitz_html::{HtmlDocument, HtmlProvider};
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point, PointerCoords,
    PointerDetails, UiEvent,
};
use blitz_traits::shell::{ColorScheme, ShellProvider, Viewport};

use std::sync::{Arc, Mutex};

/// Records the IME requests the document makes of the shell.
#[derive(Default)]
struct ImeRecorder {
    enabled: Mutex<Vec<bool>>,
    cursor_areas: Mutex<Vec<(f32, f32, f32, f32)>>,
}

impl ShellProvider for ImeRecorder {
    fn set_ime_enabled(&self, is_enabled: bool) {
        self.enabled.lock().unwrap().push(is_enabled);
    }
    fn set_ime_cursor_area(&self, x: f32, y: f32, width: f32, height: f32) {
        self.cursor_areas
            .lock()
            .unwrap()
            .push((x, y, width, height));
    }
}

fn pointer_event(x: f32, y: f32, buttons: MouseEventButtons) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons,
        mods: Default::default(),
        details: PointerDetails::default(),
        element: Point::default(),
        active_pointers: Default::default(),
    }
}

fn pointer_down(doc: &mut HtmlDocument, x: f32, y: f32) {
    let mut driver = EventDriver::new(doc, NoopEventHandler);
    driver.handle_ui_event(UiEvent::PointerDown(pointer_event(
        x,
        y,
        MouseEventButtons::Primary,
    )));
}

/// A document with one single-line input at (10, 10), 100 x 24 px, at the
/// app's scale 4/3, laid out once.
fn input_doc(shell: Arc<ImeRecorder>) -> (HtmlDocument, NodeId) {
    let mut doc = HtmlDocument::from_html(
        r#"<html><body style="margin:0">
            <div id="gone"></div>
            <input id="qty" type="text" value="12"
                style="position:absolute; left:10px; top:10px; width:100px; height:24px">
        </body></html>"#,
        DocumentConfig {
            viewport: Some(Viewport::new(200, 100, 4.0 / 3.0, ColorScheme::Light)),
            html_parser_provider: Some(Arc::new(HtmlProvider) as _),
            shell_provider: Some(shell as _),
            ..Default::default()
        },
    );
    doc.resolve(0.0);
    let input = doc.query_selector("#qty").unwrap().unwrap();
    (doc, input)
}

/// Marks the input's editor layout dirty, as a value or width change between
/// two `resolve`s does.
fn dirty_editor_layout(doc: &mut HtmlDocument, input: NodeId) {
    let editor = &mut doc
        .get_node_mut(input)
        .unwrap()
        .element_data_mut()
        .unwrap()
        .text_input_data_mut()
        .unwrap()
        .editor;
    editor.set_width(Some(57.0));
    assert!(editor.try_layout().is_none(), "the editor layout is dirty");
}

#[test]
fn clicking_an_input_whose_editor_layout_is_dirty_focuses_it_without_a_panic() {
    let (mut doc, input) = input_doc(Arc::default());
    dirty_editor_layout(&mut doc, input);

    pointer_down(&mut doc, 20.0, 20.0);

    assert_eq!(doc.get_focussed_node_id(), Some(input));
}

#[test]
fn dragging_over_an_input_whose_editor_layout_is_dirty_does_not_panic() {
    let (mut doc, input) = input_doc(Arc::default());
    pointer_down(&mut doc, 20.0, 20.0);
    dirty_editor_layout(&mut doc, input);

    let mut driver = EventDriver::new(&mut doc, NoopEventHandler);
    driver.handle_ui_event(UiEvent::PointerMove(pointer_event(
        60.0,
        20.0,
        MouseEventButtons::Primary,
    )));

    assert_eq!(doc.get_focussed_node_id(), Some(input));
}

#[test]
fn the_vertical_centering_of_an_input_whose_editor_layout_is_dirty_is_zero() {
    let (mut doc, input) = input_doc(Arc::default());
    dirty_editor_layout(&mut doc, input);

    let offset = doc
        .get_node(input)
        .unwrap()
        .text_input_v_centering_offset(4.0 / 3.0);

    assert_eq!(offset, 0.0);
}

#[test]
fn focusing_an_input_whose_containing_block_is_gone_sets_no_ime_cursor_area() {
    let shell = Arc::new(ImeRecorder::default());
    let (mut doc, input) = input_doc(shell.clone());
    let gone = doc.query_selector("#gone").unwrap().unwrap();
    doc.mutate().remove_and_drop_node(gone);
    assert!(doc.get_node(gone).is_none(), "the node is dropped");
    // A cached containing block that was not updated since the node went.
    doc.get_node(input)
        .unwrap()
        .oof_containing_block
        .set(Some(gone));

    pointer_down(&mut doc, 20.0, 20.0);

    assert_eq!(doc.get_focussed_node_id(), Some(input));
    assert_eq!(*shell.enabled.lock().unwrap(), vec![true]);
    assert!(shell.cursor_areas.lock().unwrap().is_empty());
}

#[test]
fn focusing_an_input_sets_its_ime_cursor_area() {
    let shell = Arc::new(ImeRecorder::default());
    let (mut doc, input) = input_doc(shell.clone());

    pointer_down(&mut doc, 20.0, 20.0);

    assert_eq!(doc.get_focussed_node_id(), Some(input));
    assert_eq!(shell.cursor_areas.lock().unwrap().len(), 1);
}
