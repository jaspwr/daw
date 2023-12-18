use std::collections::HashMap;
use std::rc::Rc;

use glow::*;

use crate::selection::Selection;
use crate::ui::ComputedDimensions;
use crate::ui::gl::*;
use crate::ui::style::*;
use crate::ui::text::Font;

pub struct Globals {
    pub selection: Selection,
    pub element_uniform_locations: HashMap<&'static str, UniformLocation>,
    pub texture_uniform_locations: HashMap<&'static str, UniformLocation>,
    pub colour_palette: ColourPalette,
    pub element_shader: NativeProgram,
    pub texture_shader: NativeProgram,
    pub screen_dims: ComputedDimensions,
    pub main_font: Rc<Font>,
    pub top_bar_size: f32,
}

impl Globals {
    pub fn create(
        gl: &Context,
        element_shader: NativeProgram,
        texture_shader: NativeProgram,
        screen_dims: ComputedDimensions,
        main_font: Rc<Font>,
    ) -> Self {
        let element_uniform_locations =
            vec!["dims", "background_col", "border_col", "border_width", "mode", "window_size"]
                .into_iter()
                .map(|name| {
                    (name, unsafe {
                        gl.get_uniform_location(element_shader, name).unwrap()
                    })
                }).collect();

        let texture_uniform_locations =
            vec!["samp", "window_size"]
                .into_iter()
                .map(|name| {
                    (name, unsafe {
                        gl.get_uniform_location(texture_shader, name).unwrap()
                    })
                }).collect();

        Globals {
            selection: Selection::default(),
            element_uniform_locations,
            texture_uniform_locations,
            colour_palette: ColourPalette::default(),
            element_shader,
            texture_shader,
            screen_dims,
            main_font,
            top_bar_size: 25.,
        }
    }
}
