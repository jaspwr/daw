use std::collections::HashMap;
use std::rc::Rc;

use glow::*;

use crate::event_subscriptions::Subscriptions;
use crate::project::Project;
use crate::selection::Selection;
use crate::ui::gl::*;
use crate::ui::reactive::Reactive;
use crate::ui::style::*;
use crate::ui::text::Font;
use crate::ui::ComputedDimensions;

pub struct Globals {
    pub loaded_project: Project,
    pub selection: Selection,
    pub viewport: Viewport,
    pub subscriptions: Subscriptions,
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
        let element_uniform_locations = vec![
            "dims",
            "background_col",
            "border_col",
            "border_width",
            "mode",
            "window_size",
        ]
        .into_iter()
        .map(|name| {
            (name, unsafe {
                gl.get_uniform_location(element_shader, name).unwrap()
            })
        })
        .collect();

        let texture_uniform_locations = vec!["samp", "window_size"]
            .into_iter()
            .map(|name| {
                (name, unsafe {
                    gl.get_uniform_location(texture_shader, name).unwrap()
                })
            })
            .collect();

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
            loaded_project: Project::new(),
            subscriptions: Subscriptions::new(),
            viewport: Viewport::default(),
        }
    }
}

#[derive(Default)]
pub struct Viewport {
    pub time_scroll: Reactive<f32>,
    pub h_zoom: Reactive<f32>,
    pub v_zoom: Reactive<f32>,
}
