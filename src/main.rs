use std::{cell::RefCell, rc::Rc};

use element_creation_queue::fulfil_queue;
use event_subscriptions::Key;
use global::{EditingContext, Globals, PlayingState};
use glow::*;
use midi::*;
use midir::{Ignore, MidiInput};
use piano_roll::e_piano_roll;
use sdl2::sys::{SDL_GetPerformanceCounter, SDL_GetPerformanceFrequency};
use selection::Selection;
use shortcuts::{k, universal_shortcuts};
use top_bar::fb_topbar;
use track::TrackData;
use ui::{
    command_palette::fb_command_palette,
    element::Element,
    frame_buf::FrameBuf,
    gl::RENDER_MODE_SOLID,
    reactive::Reactive,
    style::{Colour, Style},
    text::{Font, Text},
    *,
};

use crate::{event_subscriptions::handle_event_subscriptions, shortcuts::key_from_symbol};

mod event_subscriptions;
mod global;
mod midi;
mod plugins;
mod project;
mod selection;
mod shortcuts;
mod track;
mod ui;
mod utils;

fn main() {
    unsafe {
        let (gl, window, mut events_loop, _context) = gl::create_sdl2_context();

        let element_shader =
            gl::create_program(&gl, MAIN_VERTEX_SHADER_SOURCE, MAIN_FRAGMENT_SHADER_SOURCE);
        let texture_shader = gl::create_program(
            &gl,
            MAIN_VERTEX_SHADER_SOURCE,
            TEXTURE_FRAGMENT_SHADER_SOURCE,
        );

        let global_reredner = Rc::new(RefCell::new(false));

        let font = Font::new();
        let mut text = Text::new(
            &gl,
            "Bitch".to_string(),
            50.,
            &font,
            Colour {
                r: 1.,
                g: 1.,
                b: 1.,
                a: 1.,
            },
            Position {
                x: Coordinate::FractionOfParentWithOffset(1., -150.),
                y: Coordinate::Fixed(0.),
            },
            global_reredner.clone(),
        );

        gl.use_program(Some(element_shader));
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        let (width, height) = window.drawable_size();
        let screen_dims = ComputedDimensions {
            width: width as f32,
            height: height as f32,
        };
        let mut globals = Globals::create(&gl, element_shader, texture_shader, screen_dims, font);

        universal_shortcuts(&mut globals);

        let mut midi_in = MidiInput::new("midir reading input").unwrap();
        midi_in.ignore(Ignore::None);
        for port in midi_in.ports() {
            println!("MIDI port: {}", midi_in.port_name(&port).unwrap());
        }

        let mut frame = FrameBuf::new(
            &gl,
            None,
            p(0., 0.),
            Dimensions {
                width: Size::FractionOfParent(1.),
                height: Size::FractionOfParentWithOffset(1., -globals.top_bar_size),
            },
            screen_dims,
        );

        let root = piano_roll::e_piano_roll(
            &gl,
            &mut globals,
            0,
            frame.children_need_rerender.clone(),
            frame.bounding_box.clone(),
        );

        frame.root_node = Some(root);

        gl.uniform_1_i32(
            Some(&globals.element_uniform_locations["mode"]),
            RENDER_MODE_SOLID,
        );

        gl.clear_color(0.1, 0.2, 0.3, 1.0);

        let mut top_bar = fb_topbar(&gl, &mut globals, &screen_dims);
        let mut command_palette = fb_command_palette(&gl, &mut globals, &screen_dims);

        let mut style = Style::default();
        style.background_colour.r = 1.;

        let mut resize = true;

        let mut running = true;

        let mut NOW = SDL_GetPerformanceCounter();
        let mut LAST = 0;

        while running {
            LAST = NOW;
            NOW = SDL_GetPerformanceCounter();
            let delta_t = (NOW - LAST) as f64 / SDL_GetPerformanceFrequency() as f64;

            main_loop(
                &mut events_loop,
                &mut globals,
                &mut resize,
                &gl,
                &mut frame,
                &mut top_bar,
                &mut command_palette,
                &window,
                &mut text,
                &mut running,
                delta_t,
            );
        }

        // root.cleanup(&gl);
        frame.cleanup(&gl);
        top_bar.cleanup(&gl);

        gl.delete_program(element_shader);
    }
}

fn main_loop(
    events_loop: &mut sdl2::EventPump,
    globals: &mut Globals,
    resize: &mut bool,
    gl: &Context,
    frame: &mut FrameBuf,
    top_bar: &mut FrameBuf,
    command_palette: &mut FrameBuf,
    window: &sdl2::video::Window,
    text: &mut Text,
    running: &mut bool,
    delta_t: f64,
) {
    for event in events_loop.poll_iter() {
        if let sdl2::event::Event::Quit { .. } = event {
            *running = false;
            return;
        }

        // println!("{:?}", event);

        handle_event_subscriptions(globals, &event);

        if let sdl2::event::Event::Window {
            timestamp,
            window_id,
            win_event,
        } = event
        {
            if let sdl2::event::WindowEvent::Resized(_, _) = win_event {
                *resize = true;
            }
        }
    }

    fulfil_queue(gl, globals);

    if globals.playing_state.is_playing() {
        // println!("{}", globals.loaded_project.player_time.get_copy());
        let delta_beats: Time =
            delta_t as Time * (globals.loaded_project.tempo.get_copy() as Time / 60.);
        globals.loaded_project.player_time += delta_beats;
    }

    if *resize {
        *resize = false;
        frame.children_need_rerender.replace(true);
        top_bar.children_need_rerender.replace(true);
    }

    let (width, height) = window.drawable_size();
    unsafe {
        gl.viewport(0, 0, width as i32, height as i32);
        gl.uniform_2_f32(
            Some(&globals.element_uniform_locations["window_size"]),
            width as f32,
            height as f32,
        );
    }

    let screen_dims = ComputedDimensions {
        width: width as f32,
        height: height as f32,
    };

    globals.screen_dims = screen_dims;

    unsafe {
        gl.clear(glow::COLOR_BUFFER_BIT);
    }

    frame.render(gl, ComputedPosition::origin(), &*globals, &screen_dims);

    top_bar.render(gl, ComputedPosition::origin(), &*globals, &screen_dims);

    if globals.editor_context.get_copy() == EditingContext::CommandPallet {
        command_palette.render(gl, ComputedPosition::origin(), &*globals, &screen_dims);
    }

    text.render(gl, ComputedPosition::origin(), &*globals, &screen_dims);

    window.gl_swap_window();
}

const MAIN_VERTEX_SHADER_SOURCE: &str = include_str!("shaders/main_vert.vert");
const MAIN_FRAGMENT_SHADER_SOURCE: &str = include_str!("shaders/main_frag.frag");
const TEXTURE_FRAGMENT_SHADER_SOURCE: &str = include_str!("shaders/texture_frag.frag");
