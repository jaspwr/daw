use std::{cell::RefCell, rc::Rc};

use event_subscriptions::Key;
use global::Globals;
use glow::*;
use midi::*;
use midir::{MidiInput, Ignore};
use piano_roll::e_piano_roll;
use selection::{NoteRef, Selection};
use shortcuts::k;
use top_bar::fb_topbar;
use track::TrackData;
use ui::{
    element::Element,
    frame_buf::FrameBuf,
    gl::RENDER_MODE_SOLID,
    style::{Colour, Style},
    text::{Font, Text},
    *,
};

use crate::{event_subscriptions::{handle_event_subscriptions}, shortcuts::key_from_symbol};

mod event_subscriptions;
mod global;
mod midi;
mod piano_roll;
mod project;
mod selection;
mod top_bar;
mod track;
mod ui;
mod utils;
mod v_scroll_container;
mod shortcuts;

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
            "Hello".to_string(),
            50.,
            &font,
            Colour {
                r: 1.,
                g: 1.,
                b: 1.,
                a: 1.,
            },
            p(10., 10.),
            global_reredner.clone(),
        );

        gl.use_program(Some(element_shader));
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        let midi = MidiClip {
            notes: vec![
                Note {
                    note: 60,
                    velocity: 100,
                    start: 0,
                    length: 100,
                },
                Note {
                    note: 61,
                    velocity: 100,
                    start: 100,
                    length: 100,
                },
                Note {
                    note: 62,
                    velocity: 100,
                    start: 50,
                    length: 100,
                },
                Note {
                    note: 63,
                    velocity: 100,
                    start: 0,
                    length: 100,
                },
            ],
        };

        let (width, height) = window.drawable_size();
        let screen_dims = ComputedDimensions {
            width: width as f32,
            height: height as f32,
        };
        let mut globals = Globals::create(&gl, element_shader, texture_shader, screen_dims, font);
        globals.selection = Selection::MidiNotes(vec![(0, vec![NoteRef { note: 60, start: 0 }])]);

        globals.subscriptions.subscribe_key(Rc::new(RefCell::new(
            |key: &Key, globals: &mut Globals| {
                if *key == k("a")  {
                    println!("HASJKDHKAJSDHAJKSDKHJASDJAHSKDJHASDKHJ  {:?}", key);
                }
            },
        )));

        // globals.subscriptions.subscribe_text_input(Rc::new(RefCell::new(
        //     |text: &String, globals: &mut Globals| {
        //         println!("HASJKDHKAJSDHAJKSDKHJASDJAHSKDJHASDKHJ  {:?}", text);
        //     },
        // )));

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

        let mut style = Style::default();
        style.background_colour.r = 1.;

        let mut resize = true;

        'render: loop {
            for event in events_loop.poll_iter() {
                if let sdl2::event::Event::Quit { .. } = event {
                    break 'render;
                }

                // println!("{:?}", event);

                handle_event_subscriptions(&mut globals, &event);

                if let sdl2::event::Event::Window {
                    timestamp,
                    window_id,
                    win_event,
                } = event
                {
                    if let sdl2::event::WindowEvent::Resized(_, _) = win_event {
                        resize = true;
                    }
                }

                if let sdl2::event::Event::KeyDown {
                    timestamp,
                    window_id,
                    keycode,
                    scancode,
                    keymod,
                    repeat,
                } = event
                {
                    if let Some(keycode) = keycode {
                        if keycode == sdl2::keyboard::Keycode::Escape {

                            globals.loaded_project.tracks[0].mutate(Box::new(|track| {
                                match track.data {
                                    TrackData::Midi(_, ref mut midi_clip) => {
                                        midi_clip.notes.push(Note {
                                            note: 60,
                                            velocity: 100,
                                            start: 0,
                                            length: 100,
                                        });
                                    }
                                    _ => {}
                                }
                            }));
                        }
                    }
                }
            }

            if resize {
                resize = false;
                frame.children_need_rerender.replace(true);
                top_bar.children_need_rerender.replace(true);
            }

            let (width, height) = window.drawable_size();
            gl.viewport(0, 0, width as i32, height as i32);
            gl.uniform_2_f32(
                Some(&globals.element_uniform_locations["window_size"]),
                width as f32,
                height as f32,
            );

            let screen_dims = ComputedDimensions {
                width: width as f32,
                height: height as f32,
            };

            globals.screen_dims = screen_dims;

            gl.clear(glow::COLOR_BUFFER_BIT);
            // root.render(&gl, p(0., 0.), &globals, &screen_dims);

            // frame.render_children(&gl, &globals, &screen_dims);
            frame.render(&gl, ComputedPosition::origin(), &globals, &screen_dims);

            // top_bar.render_children(&gl, &globals, &screen_dims);
            top_bar.render(&gl, ComputedPosition::origin(), &globals, &screen_dims);

            text.render(&gl, ComputedPosition::origin(), &globals, &screen_dims);

            window.gl_swap_window();
        }

        // root.cleanup(&gl);
        frame.cleanup(&gl);
        top_bar.cleanup(&gl);

        gl.delete_program(element_shader);
    }
}

const MAIN_VERTEX_SHADER_SOURCE: &str = include_str!("shaders/main_vert.vert");
const MAIN_FRAGMENT_SHADER_SOURCE: &str = include_str!("shaders/main_frag.frag");
const TEXTURE_FRAGMENT_SHADER_SOURCE: &str = include_str!("shaders/texture_frag.frag");
