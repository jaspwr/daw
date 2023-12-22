use glow::Context;

use crate::{global::Globals, utils::RcRefCell};

use super::{
    element::{Element, ElementRef},
    p,
    style::Style,
    text::Text,
    ComputedBoundingBox, Position, Size,
};

pub fn e_text(
    gl: &Context,
    globals: &Globals,
    needs_rerender: RcRefCell<bool>,
    frame_bounding_box: RcRefCell<Option<ComputedBoundingBox>>,
    position: Position
) -> ElementRef {
    let key_mod_text = Text::new(
        gl,
        String::new(),
        20.,
        &globals.main_font,
        globals.colour_palette.text_primary,
        position,
        needs_rerender.clone(),
    );

    Element::new(
        gl,
        p(100., 0.),
        Size::Fixed(10.),
        Size::Fixed(10.),
        Some(Style {
            render_self: false,
            ..Default::default()
        }),
        Some(key_mod_text),
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        vec![],
    )
}
