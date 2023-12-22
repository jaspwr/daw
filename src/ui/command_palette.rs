use glow::Context;
use sdl2::sys::{KeyCode, SDL_KeyCode};

use crate::{
    global::{Globals, EditingContext},
    ui::{style::Style, Coordinate, Dimensions, Position, Size}, bind_reactives, utils::{RcRefCell, rc_ref_cell},
};

use super::{element::Element, frame_buf::FrameBuf, ComputedDimensions};

pub fn fb_command_palette(
    gl: &Context,
    globals: &mut Globals,
    parent_dims: &ComputedDimensions,
) -> FrameBuf {
    const TOP_GAP: f32 = 30.;
    const WIDTH: f32 = 550.;
    const HEIGHT: f32 = 390.;

    let pos = Position {
        x: Coordinate::FractionOfParentWithOffset(0.5, -WIDTH / 2.),
        y: Coordinate::FractionOfParentWithOffset(1., -HEIGHT - TOP_GAP),
    };

    let dims = Dimensions {
        width: Size::Fixed(WIDTH),
        height: Size::Fixed(HEIGHT),
    };

    let mut frame_buf = FrameBuf::new(gl, None, pos, dims, *parent_dims);
    let needs_rerender = frame_buf.children_need_rerender.clone();
    let frame_bounding_box = frame_buf.bounding_box.clone();

    let container_style = Style {
        background_colour: globals.colour_palette.bg_primary,
        ..Style::default()
    };

    let container = Element::new(
        gl,
        Position::origin(),
        Size::FractionOfParent(1.),
        Size::FractionOfParent(1.),
        Some(container_style),
        None,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        vec![],
    );

    globals.subscriptions.subscribe_key(rc_ref_cell(|key, globals: &mut Globals| {
        if globals.editor_context.get_copy() != EditingContext::CommandPallet {
            return;
        }

        if key.code == SDL_KeyCode::SDLK_ESCAPE as KeyCode {
            globals.editor_context <<= EditingContext::PianoRoll;
        }
    }));

    frame_buf.root_node = Some(container);
    frame_buf
}
