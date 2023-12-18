use std::{cell::RefCell, rc::Rc, default};

use crate::global::Globals;

use super::{Size, Position, Dimensions, element::{Element, ElementRef}, style::Style, text::Text, reactive::Reactive};

pub fn e_f32_field (
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

    let container_cpy = container.clone();
    let sub_id = value.subscribe(Box::new(move |new_value| {
        let new_value = new_value.clone();
        container_cpy.borrow_mut().mutate(Box::new(move |element: &mut Element| {
            if let Some(text) = &mut element.text {
                text.text = new_value.to_string();
                text.needs_glyphs_rerender = true;
            }
        }));
    }));

    container.borrow_mut().on_cleanup.push(Box::new(move || {
        value.unsubscribe(sub_id);
    }));

    container
}
