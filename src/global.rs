use std::collections::HashMap;
use std::default;
use std::rc::Rc;

use glow::*;

use crate::event_subscriptions::Subscriptions;
use crate::project::Project;
use crate::selection::Selection;
use crate::shortcuts::ShortcutsBuffer;
use crate::ui::{gl::*, ComputedPosition};
use crate::ui::reactive::Reactive;
use crate::ui::style::*;
use crate::ui::text::Font;
use crate::ui::ComputedDimensions;

pub struct Globals {
    pub loaded_project: Project,
    pub playing_state: PlayingState,
    pub viewport: Viewport,
    pub shortcuts_buffer: ShortcutsBuffer,
    pub editor_context: Reactive<EditingContext>,
    pub subscriptions: Subscriptions,
    pub element_uniform_locations: HashMap<&'static str, UniformLocation>,
    pub texture_uniform_locations: HashMap<&'static str, UniformLocation>,
    pub colour_palette: ColourPalette,
    pub element_shader: NativeProgram,
    pub texture_shader: NativeProgram,
    pub screen_dims: ComputedDimensions,
    pub main_font: Rc<Font>,
    pub top_bar_size: f32,
    pub piano_roll_keyboard_width: f32,
    pub mouse_pos: ComputedPosition,
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
            "alt_width",
            "alt_col",
            "alt_offset",
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
            playing_state: PlayingState::default(),
            element_uniform_locations,
            texture_uniform_locations,
            shortcuts_buffer: ShortcutsBuffer::new(),
            editor_context: Reactive::new(EditingContext::PianoRoll),
            colour_palette: ColourPalette::default(),
            element_shader,
            texture_shader,
            screen_dims,
            main_font,
            top_bar_size: 25.,
            piano_roll_keyboard_width: 150.,
            loaded_project: Project::new(),
            subscriptions: Subscriptions::new(),
            viewport: Viewport::default(),
            mouse_pos: ComputedPosition::origin(),
        }
    }
}

#[derive(Default)]
pub struct Viewport {
    pub time_scroll: Reactive<f32>,
    pub h_zoom: Reactive<f32>,
    pub v_zoom: Reactive<f32>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum EditingContext {
    PianoRoll,
    InputField(usize),
    CommandPallet
}

#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub enum PlayingState {
    #[default]
    Stopped,
    Playing,
    Recording,
}

impl PlayingState {
    pub fn is_playing(&self) -> bool {
        match self {
            PlayingState::Stopped => false,
            PlayingState::Playing => true,
            PlayingState::Recording => true,
        }
    }

    pub fn play_pause(&mut self) {
        match self {
            PlayingState::Playing => *self = PlayingState::Stopped,
            PlayingState::Stopped => *self = PlayingState::Playing,
            PlayingState::Recording => *self = PlayingState::Stopped,
        }
    }
}


impl EditingContext {
    pub fn grabs_keyboard(&self) -> bool {
        match self {
            EditingContext::InputField(_) => true,
            EditingContext::CommandPallet => true,
            _ => false,
        }
    }
}
