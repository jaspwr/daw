use std::{cell::RefCell, default, rc::Rc};

use sdl2::{libc::glob, mouse::MouseButton};

use crate::{
    event_subscriptions::{Key, SubscriptionId},
    global::{Globals, EditingContext}, utils::{RcRefCell, rc_ref_cell},
};

use super::{
    element::{Element, ElementRef},
    reactive::Reactive,
    style::Style,
    text::Text,
    BoundingBoxRef, Dimensions, Position, Size,
};

pub fn e_f32_field(
    gl: &glow::Context,
    globals: &mut Globals,
    pos: Position,
    dims: Dimensions,
    value: Reactive<f32>,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
) -> ElementRef {
    let style = Style {
        background_colour: globals.colour_palette.bg_primary,
        ..Style::default()
    };

    let current = value.get().borrow().to_string();
    let typing_buf = Reactive::new(current.to_string());
    let inserting_at = Reactive::new(0);
    let previous_editor_context = Reactive::new(globals.editor_context.clone());

    let text = Text::new(
        gl,
        current,
        20.,
        &globals.main_font,
        globals.colour_palette.text_primary,
        Position::origin(),
        needs_rerender.clone(),
    );

    let container = Element::new(
        gl,
        pos,
        dims.width,
        dims.height,
        Some(style),
        Some(text),
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        vec![],
    );

    container.subscribe_mutation_to_reactive(
        &value,
        Box::new(move |element: &mut Element, new_value: &f32| {
            if let Some(text) = &mut element.text_node {
                text.text = new_value.to_string();
                text.needs_glyphs_rerender = true;
            }
        }),
    );

    container.subscribe_mutation_to_reactive(
        &typing_buf,
        Box::new(move |element: &mut Element, new_value: &String| {
            if let Some(text) = &mut element.text_node {
                text.text = format!("{}|", new_value.to_string());
                text.needs_glyphs_rerender = true;
            }
        }),
    );

    let typing_buf = typing_buf.clone();
    let value = value.clone();
    let uid = container.uid();

    globals.subscriptions.subscribe_click_in_area(
        container.borrow().bounding_box.clone(),
        Rc::new(RefCell::new(
            move |mb: &MouseButton, globals: &mut Globals| {
                let typing_buf = typing_buf.clone();
                let value = value.clone();
                if mb == &MouseButton::Left {
                    start_input(globals, typing_buf, value, uid, previous_editor_context.clone());
                }
            },
        )),
    );

    container
}

fn start_input(globals: &mut Globals, typing_buf: Reactive<String>, value: Reactive<f32>, uid: usize, previous_editor_context: Reactive<EditingContext>) {
    previous_editor_context.set(globals.editor_context.clone());
    globals.editor_context = EditingContext::InputField(uid);

    typing_buf.set(value.get_copy().to_string());

    let char_sub = {
        let typing_buf = typing_buf.clone();
        globals
            .subscriptions
            .subscribe_text_input(Rc::new(RefCell::new(
                move |text: &String, globals: &mut Globals| {
                    let typing_buf = typing_buf.clone();
                    let text = text.clone();
                    typing_buf.mutate(Box::new(move |v| *v = format!("{}{}", v, text.clone())));
                    // println!("{}", text);
                },
            )))
    };

    let value = value.clone();

    let mut key_sub_id = Reactive::new(0);

    let key_sub_id_cpy = key_sub_id.clone();
    let key_sub = globals.subscriptions.subscribe_key(Rc::new(RefCell::new(
        move |key: &Key, globals: &mut Globals| {
            let mut value = value.clone();
            let typing_buf = typing_buf.clone();
            let key = key.code.clone();
            if key == sdl2::keyboard::Keycode::Backspace as u8 {
                typing_buf.mutate(Box::new(|v| {
                    if v.len() < 1 {
                        return;
                    }

                    *v = v[..v.len() - 1].to_string();
                }));
            }
            if key == sdl2::keyboard::Keycode::Return as u8 {
                globals.editor_context = previous_editor_context.clone().get_copy();
                let new_value_ = typing_buf.get();
                let new_value = new_value_.borrow();

                match new_value.parse::<f32>() {
                    Ok(v) => value <<= v,
                    _ => value <<= value.get_copy(),
                }

                globals.subscriptions.unsubscribe_text_input(char_sub);
                globals.subscriptions.unsubscribe_key(key_sub_id_cpy.clone().get_copy());
            }
        },
    )));

    key_sub_id <<= key_sub;

    // globals.active_input = Some(value.clone());
}

pub fn e_button(
    gl: &glow::Context,
    globals: &mut Globals,
    pos: Position,
    dims: Dimensions,
    text: String,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
    on_click: Box<dyn Fn()>,
) -> ElementRef {
    let style = Style {
        background_colour: globals.colour_palette.bg_primary,
        ..Style::default()
    };

    let text = Text::new(
        gl,
        text,
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
        frame_bounding_box.clone(),
        vec![],
    );

    globals.subscriptions.subscribe_click_in_area(
        container.borrow().bounding_box.clone(),
        Rc::new(RefCell::new(
            move |mb: &MouseButton, globals: &mut Globals| {
                if mb == &MouseButton::Left {
                    on_click();
                }
            },
        )),
    );

    container
}
