use std::{cell::RefCell, default, rc::Rc};

use crate::global::Globals;

use super::{
    element::{Element, ElementRef},
    reactive::Reactive,
    style::Style,
    text::Text,
    Dimensions, Position, Size,
};

pub fn e_f32_field(
    gl: &glow::Context,
    globals: &Globals,
    pos: Position,
    dims: Dimensions,
    value: Reactive<f32>,
    needs_rerender: Rc<RefCell<bool>>,
) -> ElementRef {
    let style = Style {
        background_colour: globals.colour_palette.bg_primary,
        ..Style::default()
    };

    let current = value.get().borrow().to_string();

    let text = Text::new(
        gl,
        current,
        20.,
        &globals.main_font,
        globals.colour_palette.text_primary,
        Position::origin(),
        needs_rerender.clone(),
    );

    let mut container = Element::new(
        gl,
        pos,
        dims.width,
        dims.height,
        Some(style),
        Some(text),
        needs_rerender.clone(),
        vec![],
    );

    Element::subscribe_mutation_to_reactive(
        &container,
        &value,
        Box::new(move |element: &mut Element, new_value: &f32| {
            if let Some(text) = &mut element.text {
                text.text = new_value.to_string();
                text.needs_glyphs_rerender = true;
            }
        }),
    );

    container
}
