use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{
    bind_reactives,
    global::Globals,
    ui::{
        d,
        element::{self, Element},
        frame_buf::FrameBuf,
        input::{e_button, e_f32_field},
        misc_elements::e_text,
        p,
        reactive::Reactive,
        style::{c, Style},
        text::Text,
        BoundingBoxRef, ComputedDimensions, Coordinate, Dimensions, Position, Size,
    },
};
use glow::*;

pub fn fb_topbar(
    gl: &Context,
    globals: &mut Globals,
    parent_dims: &ComputedDimensions,
) -> FrameBuf {
    let pos = Position {
        x: Coordinate::Fixed(0.),
        y: Coordinate::FractionOfParentWithOffset(1., -globals.top_bar_size),
    };
    let dims = Dimensions {
        width: Size::FractionOfParent(1.),
        height: Size::Fixed(globals.top_bar_size),
    };

    let mut frame_buf = FrameBuf::new(gl, None, pos, dims, *parent_dims);
    let needs_rerender = frame_buf.children_need_rerender.clone();
    let frame_bounding_box = frame_buf.bounding_box.clone();

    let container_style = Style {
        background_colour: globals.colour_palette.bg_primary,
        ..Style::default()
    };

    let tempo = globals.loaded_project.tempo.clone();

    let tempo = e_f32_field(
        gl,
        globals,
        p(0., 0.),
        d(40., 100.),
        tempo,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
    );

    let key_mod_element = e_text(
        gl,
        globals,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        p(100., 0.)
    );

    let am = globals.shortcuts_buffer.amount_modifier.clone();
    bind_reactives! {
        key_mod_element {
            [am] => (|e: &mut Element, am: Option<i32>| {
                e.text_node.as_mut().unwrap().mutate(Box::new(move |text| {
                    text.text = am.map(|a| a.to_string()).unwrap_or_else(|| String::new());
                }));
            })
        }
    }

    let container = Element::new(
        gl,
        Position::origin(),
        Size::FractionOfParent(1.),
        Size::FractionOfParent(1.),
        Some(container_style),
        None,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        vec![tempo, key_mod_element],
    );

    frame_buf.root_node = Some(container);
    frame_buf
}
