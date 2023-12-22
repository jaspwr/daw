use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap, ops::ControlFlow, rc::Rc};

use glow::Context;
use midi_control::note;
use sdl2::{keyboard, libc::glob};

use crate::{
    bind_reactives,
    element_creation_queue::{queue_element, CreateElementFn},
    global::{Globals, Viewport},
    midi::*,
    project::TimeSignature,
    selection::Selection,
    track::{TrackData, TrackId},
    ui::{element::*, scroll_window::e_scroll_window},
    ui::{macros::ElementInitDeps, reactive_list::ReactiveListKey, style::*, text::Text},
    ui::{reactive::Reactive, *},
    utils::{note_name, rc_ref_cell, RcRefCell},
};

pub fn e_piano_roll(
    gl: &glow::Context,
    globals: &mut Globals,
    track_id: TrackId,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
) -> ElementRef {
    globals.viewport.h_zoom.set(35.);
    globals.viewport.v_zoom.set(12.);

    let style = Style {
        render_self: false,
        ..Default::default()
    };

    // mark_selected(globals, track_id, &mut notes);

    let element_note_id_map: RcRefCell<HashMap<ReactiveListKey, ElementRef>> =
        rc_ref_cell(HashMap::new());

    let mut roll_children: Vec<ElementRef> = (0..=127)
        .map(|n| {
            e_note(
                gl,
                n,
                globals,
                // &notes[n as usize],
                needs_rerender.clone(),
                frame_bounding_box.clone(),
            )
        })
        .collect();

    let time_grid = e_time_grid(
        gl,
        globals.piano_roll_keyboard_width,
        globals,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
    );

    roll_children.push(time_grid);

    let keyboard_width = globals.piano_roll_keyboard_width;

    let roll = Element::new(
        gl,
        Position::origin(),
        Size::FractionOfParent(1.),
        Size::FractionOfParent(1.),
        Some(style),
        None,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        roll_children,
    );

    {
        // TODO: Refactor

        let needs_rerender = needs_rerender.clone();
        let frame_bounding_box = frame_bounding_box.clone();
        let midi = match &globals.loaded_project.tracks[track_id].data {
            TrackData::Midi(_, midi) => midi,
            _ => panic!("Track is not midi"),
        };
        let roll_cpy = roll.clone();

        let element_note_id_map_cpy = element_note_id_map.clone();

        midi.notes.subscribe_to_push(Box::new(move |key, note| {
            let needs_rerender = needs_rerender.clone();
            let note = note.clone();
            let frame_bounding_box = frame_bounding_box.clone();
            let roll = roll_cpy.clone();

            let element_note_id_map = element_note_id_map_cpy.clone();
            let key = key.clone();

            let create_note_element: CreateElementFn =
                Box::new(move |gl: &Context, globals: &mut Globals| {
                    let note_element = e_midi_note(
                        &note,
                        key.clone(),
                        globals,
                        gl,
                        &needs_rerender,
                        &frame_bounding_box,
                    );

                    element_note_id_map
                        .as_ref()
                        .borrow_mut()
                        .insert(key.clone(), note_element.clone());

                    note_element
                });

            queue_element(create_note_element, roll);
        }));

        let element_note_id_map = element_note_id_map.clone();
        let roll = roll.clone();
        midi.notes.subscribe_to_remove(Box::new(move |note_id, ()| {
            let element_note_id_map = element_note_id_map.clone();
            let roll = roll.clone();
            let note_id = note_id.clone();
            roll.mutate(Box::new(move |element| {
                let element_note_id_map = element_note_id_map.clone();
                let note_element = element_note_id_map
                    .as_ref()
                    .borrow_mut()
                    .remove(&note_id)
                    .unwrap();

                let element_uid = note_element.uid();
                element.children.retain(|c| c.uid() != element_uid);
            }))
        }));
    }

    {
        let midi = match &globals.loaded_project.tracks[track_id].data {
            TrackData::Midi(_, midi) => midi,
            _ => panic!("Track is not midi"),
        };

        let notes_ = midi.notes.copy_of_whole_list();
        let roll = roll.clone();
        for n in notes_.iter() {
            let needs_rerender = needs_rerender.clone();
            let frame_bounding_box = frame_bounding_box.clone();

            let n = n.clone();
            let element_note_id_map = element_note_id_map.clone();
            let create_note_element: CreateElementFn =
                Box::new(move |gl: &Context, globals: &mut Globals| {
                    let element_note_id_map = element_note_id_map.clone();

                    let (note_id, note) = n.clone();

                    let note_element = e_midi_note(
                        &note.clone(),
                        note_id.clone(),
                        globals,
                        gl,
                        &needs_rerender,
                        &frame_bounding_box,
                    );

                    element_note_id_map
                        .as_ref()
                        .borrow_mut()
                        .insert(n.0.clone(), note_element.clone());

                    note_element
                });

            queue_element(create_note_element, roll.clone());
        }
    }

    let player_head = e_player_head(
        gl,
        globals,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        keyboard_width,
    );

    let scroll_win = e_scroll_window(
        gl,
        globals,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        true,
        500.,
        false,
        0.,
        vec![roll, player_head],
    );

    return scroll_win;
}

// fn mark_selected(
//     globals: &mut Globals,
//     track_id: u32,
//     notes: &mut Vec<Vec<(Reactive<Note>, bool)>>,
// ) {
//     if let Selection::MidiNotes(selection) = &globals.loaded_project.selection {
//         for (track, sel_notes) in selection.iter() {
//             if *track == track_id {
//                 flag_selected(sel_notes, notes);
//                 break;
//             }
//         }
//     }
// }

// fn flag_selected(sel_notes: &Vec<NoteRef>, notes: &mut Vec<Vec<(Reactive<Note>, bool)>>) {
//     for sel_note in sel_notes.iter() {
//         for note in &mut *notes[sel_note.note as usize] {
//             let start = note.0.get().borrow().start.clone();
//             if start == sel_note.start {
//                 note.1 = true;
//             }
//         }
//     }
// }

fn e_time_grid(
    gl: &glow::Context,
    base_x: f32,
    globals: &mut Globals,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
) -> ElementRef {
    let style = Style {
        render_self: false,
        ..Default::default()
    };

    let mut children = vec![];

    // let width = {
    //     let bb = frame_bounding_box.borrow().clone().unwrap_or_default();
    //     bb.dimensions().width - base_x
    // };

    let beats_per_division = 1;
    let divisions_count = 500;
    let total_width = {
        let h_zoom = globals.viewport.h_zoom.clone();
        Rc::new(move || {
            time_to_width(
                (divisions_count * beats_per_division) as Time,
                h_zoom.get_copy(),
            )
        })
    };

    for i in 0..divisions_count {
        // let bar = e_bar(
        //     gl,
        //     globals,
        //     needs_rerender.clone(),
        //     frame_bounding_box.clone(),
        //     x(),
        //     1.,
        // );

        let time_scroll = globals.viewport.time_scroll.clone();
        let h_zoom = globals.viewport.h_zoom.clone();

        let deps = ElementInitDeps {
            gl,
            globals,
            needs_rerender: needs_rerender.clone(),
            frame_bounding_box: frame_bounding_box.clone(),
        };

        let bar = e_bar(
            gl,
            globals,
            needs_rerender.clone(),
            frame_bounding_box.clone(),
        );

        bind_reactives! {
            bar {
                [time_scroll, h_zoom] => (|e: &mut Element, ts, hz| {
                    let total_width = time_to_width(
                        (divisions_count * beats_per_division) as Time,
                        hz,
                    );

                    let mut x = x_of_time_no_global_access(i as Time, 0., ts, hz);

                    while x < 0. {
                        x += total_width;
                    }

                    x = x % total_width;

                    e.position.x = Coordinate::Fixed(x);
                }),
            }
        };

        children.push(bar);
    }

    let grid = Element::new(
        gl,
        p(base_x, 0.),
        Size::Fixed(1.),
        Size::FractionOfParent(1.),
        Some(style),
        None,
        needs_rerender.clone(),
        frame_bounding_box,
        children,
    );

    grid
}

fn e_player_head(
    gl: &glow::Context,
    globals: &Globals,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
    x_offset: f32,
) -> ElementRef {
    let style = Style {
        background_colour: globals.colour_palette.player_head,
        ..Default::default()
    };

    let x = x_of_time(
        globals.loaded_project.player_time.get_copy(),
        globals,
        globals.piano_roll_keyboard_width,
    );

    let head = Element::new(
        gl,
        p(x, 0.),
        Size::Fixed(1.),
        Size::Fixed(10000.),
        Some(style),
        None,
        needs_rerender.clone(),
        frame_bounding_box,
        vec![],
    );

    let ts = globals.viewport.time_scroll.clone();
    let pt = globals.loaded_project.player_time.clone();
    let hz = globals.viewport.h_zoom.clone();
    bind_reactives! {
        head {
            [ts, pt, hz] => (|e: &mut Element, ts, pt, hz| {
                let x = x_of_time_no_global_access(pt, x_offset, ts, hz);
                e.position.x = Coordinate::Fixed(x);
            }),
        }
    }

    head
}

fn e_bar(
    gl: &glow::Context,
    globals: &Globals,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
    // x: Coordinate,
    // thickness: f32,
) -> ElementRef {
    let style = Style {
        background_colour: globals.colour_palette.time_grid,
        ..Default::default()
    };

    let thickness = 1.;

    Element::new(
        gl,
        Position::origin(),
        Size::Fixed(thickness),
        Size::Fixed(10000.),
        Some(style),
        None,
        needs_rerender.clone(),
        frame_bounding_box,
        vec![],
    )
}

fn x_of_time(t: Time, globals: &Globals, x_offset: f32) -> f32 {
    let time_scroll = globals.viewport.time_scroll.get_copy();
    let h_zoom = globals.viewport.h_zoom.get_copy();
    x_of_time_no_global_access(t, x_offset, time_scroll, h_zoom)
}

fn x_of_time_no_global_access(t: Time, x_offset: f32, time_scroll: f32, h_zoom: f32) -> f32 {
    x_offset + h_zoom * (t as f32 - time_scroll) as f32
}

fn time_to_width(duration: Time, h_zoom: f32) -> f32 {
    duration as f32 * h_zoom
}

fn width_to_time(width: f32, globals: &Globals) -> Time {
    (width / globals.viewport.h_zoom.get_copy()) as Time
}

fn e_note(
    gl: &glow::Context,
    note: i32,
    globals: &Globals,
    // notes: &Vec<(Reactive<Note>, bool)>,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
) -> ElementRef {
    let key = e_key(globals, note, gl, &needs_rerender, &frame_bounding_box);

    let mut children = vec![key.clone()];

    // es_notes(
    //     notes,
    //     globals,
    //     gl,
    //     &needs_rerender,
    //     &frame_bounding_box,
    //     &mut children,
    // );

    let row_border_width = 1.;

    let mut row_style = Style {
        border_width: row_border_width,
        border_colour: globals.colour_palette.time_grid,
        ..Default::default()
    };

    let mut alt_col = BLACK;

    (row_style.background_colour, alt_col) = if is_black_key(note) {
        (
            globals.colour_palette.black_key_piano_roll_row,
            globals.colour_palette.black_key_piano_roll_row_alt,
        )
    } else {
        (
            globals.colour_palette.white_key_piano_roll_row,
            globals.colour_palette.white_key_piano_roll_row_alt,
        )
    };

    let note_height = globals.viewport.v_zoom.get().borrow().clone();

    let y = note_height * note as f32;

    let key_row = Element::new(
        gl,
        p(0., y),
        Size::FractionOfParent(1.),
        Size::Fixed(note_height),
        Some(row_style),
        None,
        needs_rerender.clone(),
        frame_bounding_box,
        children,
    );

    let vz = globals.viewport.v_zoom.clone();
    let hz = globals.viewport.h_zoom.clone();
    let ts = globals.viewport.time_scroll.clone();
    let t_sig = globals.loaded_project.time_signature.clone();

    let keyboard_width = globals.piano_roll_keyboard_width;
    bind_reactives! {
        key_row {
            [vz] => (|e: &mut Element, vz| { e.dimensions.height = Size::Fixed(vz);}),
            [hz, ts, t_sig] => (move |e: &mut Element, hz, ts, t_sig: TimeSignature| {
                let beats_per_measure = t_sig.beats_per_measure();
                let bar_width = time_to_width(beats_per_measure as Time, hz);

                let x_offset = x_of_time_no_global_access(0., keyboard_width, ts, hz);

                e.style.alt = Some((bar_width, x_offset, alt_col));
            })
        }
    }

    key_row
}

fn e_midi_note(
    n: &Reactive<Note>,
    note_id: ReactiveListKey,
    globals: &Globals,
    gl: &glow::Context,
    needs_rerender: &Rc<RefCell<bool>>,
    frame_bounding_box: &Rc<RefCell<Option<ComputedBoundingBox>>>,
) -> ElementRef {
    let mut note_style = Style::default();
    note_style.background_colour = globals.colour_palette.white;
    note_style.border_width = 1.;
    note_style.border_colour = globals.colour_palette.time_grid;

    let label = Text::new(
        gl,
        String::new(),
        11.,
        &globals.main_font,
        globals.colour_palette.black,
        p(2., 2.),
        needs_rerender.clone(),
    );

    let note = Element::new(
        gl,
        Position::origin(),
        Size::Fixed(1.),
        Size::Fixed(1.),
        Some(note_style),
        Some(label),
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        vec![],
    );

    let keyboard_width = globals.piano_roll_keyboard_width;
    let ts = globals.viewport.time_scroll.clone();
    let hz = globals.viewport.h_zoom.clone();
    let vz = globals.viewport.v_zoom.clone();
    let n = n.clone();
    let selected = globals.loaded_project.selection.clone();
    let selected_colour = globals.colour_palette.selected;
    let note_outline_colour = globals.colour_palette.black;

    bind_reactives! {
        note {
            [n, ts, hz, vz] => (|e: &mut Element, n: Note, ts, hz, vz| {
                let width = n.length as f32 * hz;
                let note_height = vz;
                let x = x_of_time_no_global_access(n.start, keyboard_width, ts, hz);
                let y = note_height * n.note as f32;

                e.position.x = Coordinate::Fixed(x);
                e.dimensions.width = Size::Fixed(width);
                e.position.y = Coordinate::Fixed(y);
                e.dimensions.height = Size::Fixed(note_height);

                let note = n.note;
                e.text_node
                    .as_mut()
                    .unwrap()
                    .mutate(Box::new(move |text| {
                        text.text = note_name(note as u8, true);
                    }));
            }),
            [selected] => (|e: &mut Element, s: Selection| {
                if s.is_note_selected(0, note_id.clone()) {
                    e.style.border_colour = selected_colour;
                    e.style.border_width = 2.;
                } else {
                    e.style.border_colour = note_outline_colour;
                    e.style.border_width = 1.;
                }
            })
        }
    }

    note
}

fn e_key(
    globals: &Globals,
    note: i32,
    gl: &glow::Context,
    needs_rerender: &Rc<RefCell<bool>>,
    frame_bounding_box: &Rc<RefCell<Option<ComputedBoundingBox>>>,
) -> ElementRef {
    let mut key_style = Style::default();

    key_style.border_width = 1.;
    key_style.border_colour = globals.colour_palette.time_grid;
    key_style.padding_left = 5.;
    key_style.padding_top = -1.;

    key_style.background_colour = if is_black_key(note) {
        globals.colour_palette.black_key
    } else {
        globals.colour_palette.white_key
    };

    let text_colour = if is_black_key(note) {
        globals.colour_palette.white
    } else {
        globals.colour_palette.black
    };

    let note_height = globals.viewport.v_zoom.get().borrow().clone();

    let label = Text::new(
        gl,
        note_name(note as u8, true),
        12.,
        &globals.main_font,
        text_colour,
        p(0., 0.),
        needs_rerender.clone(),
    );

    let keyboard_width = globals.piano_roll_keyboard_width;
    let key = Element::new(
        gl,
        Position::origin(),
        Size::Fixed(keyboard_width),
        Size::FractionOfParent(1.),
        Some(key_style),
        Some(label),
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        vec![],
    );
    key
}

fn is_black_key(note: i32) -> bool {
    let note = note % 12;
    note == 1 || note == 3 || note == 6 || note == 8 || note == 10
}
