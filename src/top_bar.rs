use std::{cell::RefCell, rc::Rc};

use crate::{
    global::Globals,
    ui::{
        element::Element, frame_buf::FrameBuf, p, style::Style, text::Text, ComputedDimensions,
        Coordinate, Dimensions, Position, Size,
    },
};
use glow::*;

pub fn fb_topbar(
    gl: &Context,
    globals: &Globals,
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

    let container_style = Style {
        background_colour: globals.colour_palette.bg_primary,
        ..Style::default()
    };

    let label = Text::new(
        gl,
        "PianoRoll".to_string(),
        20.,
        &globals.main_font,
        globals.colour_palette.text_primary,
        p(0., 0.),
        needs_rerender.clone(),
    );

    let mut container = Element::new(
        gl,
        Position::origin(),
        Size::FractionOfParent(1.),
        Size::FractionOfParent(1.),
        Some(container_style),
        Some(label),
        needs_rerender.clone(),
        vec![],
    );

    frame_buf.root_node = Some(container);
    frame_buf
}
