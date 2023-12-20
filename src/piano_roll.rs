use std::{borrow::BorrowMut, cell::RefCell, ops::ControlFlow, rc::Rc};

use glow::Context;
use sdl2::{keyboard, libc::glob};

use crate::{
    element_creation_queue::{queue_element, CreateElementFn},
    global::{Globals, Viewport},
    midi::*,
    selection::{NoteRef, Selection},
    track::{TrackData, TrackId},
    ui::{element::*, scroll_window::e_scroll_window},
    ui::{reactive::Reactive, *},
    ui::{style::*, text::Text},
    utils::note_name,
};

pub fn e_piano_roll(
    gl: &glow::Context,
    globals: &mut Globals,
    track_id: TrackId,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
) -> ElementRef {
    globals.viewport.h_zoom.set(2.);
    globals.viewport.v_zoom.set(12.);

    let style = Style {
        render_self: false,
        ..Default::default()
    };



    // mark_selected(globals, track_id, &mut notes);



    let mut roll_children = (0..=127)
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
        let needs_rerender = needs_rerender.clone();
        let frame_bounding_box = frame_bounding_box.clone();
        let midi = match &globals.loaded_project.tracks[track_id].data {
            TrackData::Midi(_, midi) => midi,
            _ => panic!("Track is not midi"),
        };
        let roll = roll.clone();

        midi.notes.subscribe_to_push(Box::new(move |key, note| {
            let needs_rerender = needs_rerender.clone();
            let note = note.clone();
            let frame_bounding_box = frame_bounding_box.clone();
            let roll = roll.clone();

            let create_note_element: CreateElementFn =
                Box::new(move |gl: &Context, globals: &mut Globals| {
                    e_midi_note(
                        &note,
                        globals,
                        false,
                        gl,
                        &needs_rerender,
                        &frame_bounding_box,
                    )
                });

            queue_element(create_note_element, roll);
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
            let create_note_element: CreateElementFn =
                Box::new(move |gl: &Context, globals: &mut Globals| {
                    e_midi_note(
                        &n.clone(),
                        globals,
                        false,
                        gl,
                        &needs_rerender,
                        &frame_bounding_box,
                    )
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

    let time_grid = e_time_grid(
        gl,
        keyboard_width,
        globals,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
    );

    let scroll_win = e_scroll_window(
        gl,
        globals,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        true,
        false,
        vec![roll, time_grid, player_head],
    );

    return scroll_win;
}

fn mark_selected(
    globals: &mut Globals,
    track_id: u32,
    notes: &mut Vec<Vec<(Reactive<Note>, bool)>>,
) {
    if let Selection::MidiNotes(selection) = &globals.selection {
        for (track, sel_notes) in selection.iter() {
            if *track == track_id {
                flag_selected(sel_notes, notes);
                break;
            }
        }
    }
}

fn flag_selected(sel_notes: &Vec<NoteRef>, notes: &mut Vec<Vec<(Reactive<Note>, bool)>>) {
    for sel_note in sel_notes.iter() {
        for note in &mut *notes[sel_note.note as usize] {
            let start = note.0.get().borrow().start.clone();
            if start == sel_note.start {
                note.1 = true;
            }
        }
    }
}

fn e_time_grid(
    gl: &glow::Context,
    x: f32,
    globals: &mut Globals,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
) -> ElementRef {
    let style = Style {
        render_self: false,
        ..Default::default()
    };

    let width = {
        let bb = frame_bounding_box.borrow().clone().unwrap_or_default();
        bb.dimensions().width - x
    };

    let mut children = vec![];

    for i in 0..10 {
        let x = x_of_time(i * 50, globals, x);
        children.push(e_bar(
            gl,
            x,
            2.,
            globals,
            needs_rerender.clone(),
            frame_bounding_box.clone(),
        ));
    }

    let grid = Element::new(
        gl,
        p(x, 0.),
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

    let x = x_of_time(globals.loaded_project.player_time.get_copy(), globals, 0.);

    let head = Element::new(
        gl,
        p(x, 0.),
        Size::Fixed(1.),
        Size::FractionOfParent(1.),
        Some(style),
        None,
        needs_rerender.clone(),
        frame_bounding_box,
        vec![],
    );

    let time_scroll = globals.viewport.time_scroll.clone();
    let player_time = globals.loaded_project.player_time.clone();
    head.subscribe_mutation_to_reactive(
        &globals.viewport.h_zoom,
        Box::new(move |element: &mut Element, h_zoom: &f32| {
            let x = x_of_time_no_global_access(
                player_time.get_copy(),
                x_offset,
                time_scroll.get_copy(),
                *h_zoom,
            );
            element.position.x = Coordinate::Fixed(x);
        }),
    );

    let h_zoom = globals.viewport.h_zoom.clone();
    let time_scroll = globals.viewport.time_scroll.clone();
    head.subscribe_mutation_to_reactive(
        &globals.loaded_project.player_time,
        Box::new(move |element: &mut Element, t: &Time| {
            let x =
                x_of_time_no_global_access(*t, x_offset, time_scroll.get_copy(), h_zoom.get_copy());
            element.position.x = Coordinate::Fixed(x);
        }),
    );

    head
}

fn e_bar(
    gl: &glow::Context,
    x: f32,
    thickness: f32,
    globals: &Globals,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
) -> ElementRef {
    let style = Style {
        background_colour: globals.colour_palette.time_grid,
        ..Default::default()
    };

    Element::new(
        gl,
        p(x, 0.),
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

fn width_to_time(width: f32, globals: &Globals) -> Time {
    (width / globals.viewport.h_zoom.get_copy()) as u32
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

    let mut children = vec![key];

    // es_notes(
    //     notes,
    //     globals,
    //     gl,
    //     &needs_rerender,
    //     &frame_bounding_box,
    //     &mut children,
    // );

    let mut row_style = Style {
        border_width: 1.,
        border_colour: globals.colour_palette.time_grid,
        ..Default::default()
    };

    row_style.background_colour = if is_black_key(note) {
        globals.colour_palette.black_key_piano_roll_row
    } else {
        globals.colour_palette.white_key_piano_roll_row
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

    key_row.subscribe_mutation_to_reactive(
        &globals.viewport.v_zoom,
        Box::new(move |element: &mut Element, v_zoom: &f32| {
            let note_height = v_zoom;
            let y = note_height * note as f32;

            element.position.y = Coordinate::Fixed(y);
            element.dimensions.height = Size::Fixed(*note_height);
        }),
    );

    key_row
}

// fn es_notes(
//     notes: &Vec<(Reactive<Note>, bool)>,
//     globals: &Globals,
//     gl: &glow::Context,
//     needs_rerender: &Rc<RefCell<bool>>,
//     frame_bounding_box: &Rc<RefCell<Option<ComputedBoundingBox>>>,
//     children: &mut Vec<ElementRef>,
// ) {
//     let time_scroll = globals.viewport.time_scroll.get_copy();

//     for (n, selected) in notes.iter() {
//         let note = e_midi_note(
//             n,
//             h_zoom,
//             globals,
//             selected,
//             gl,
//             needs_rerender,
//             frame_bounding_box,
//         );

//         children.push(note);
//     }
// }

fn e_midi_note(
    n: &Reactive<Note>,
    globals: &Globals,
    selected: bool,
    gl: &glow::Context,
    needs_rerender: &Rc<RefCell<bool>>,
    frame_bounding_box: &Rc<RefCell<Option<ComputedBoundingBox>>>,
) -> ElementRef {
    let n_init = n.get_copy();
    let h_zoom = globals.viewport.h_zoom.get_copy();
    let width = n_init.length.clone() as f32 * h_zoom;

    let mut note_style = Style::default();
    note_style.background_colour = globals.colour_palette.white;
    note_style.border_width = 1.;
    note_style.border_colour = globals.colour_palette.time_grid;

    if selected {
        note_style.border_colour = globals.colour_palette.selected;
        note_style.border_width = 2.;
    }

    let keyboard_width = globals.piano_roll_keyboard_width;
    let x = x_of_time(n_init.start, globals, keyboard_width);

    let note_height = globals.viewport.v_zoom.get().borrow().clone();
    let y = note_height * n_init.note as f32;

    let note = Element::new(
        gl,
        p(x, y),
        Size::Fixed(width),
        Size::Fixed(note_height),
        Some(note_style),
        None,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        vec![],
    );

    let keyboard_width = globals.piano_roll_keyboard_width;
    {
        let n = n.clone();
        let time_scroll = globals.viewport.time_scroll.clone();
        note.subscribe_mutation_to_reactive(
            &globals.viewport.h_zoom,
            Box::new(move |element: &mut Element, new_value: &f32| {
                let n = n.get_copy();
                let width = n.length as f32 * new_value;
                let time_scroll = time_scroll.get_copy();
                let x = x_of_time_no_global_access(n.start, keyboard_width, time_scroll, h_zoom);

                element.dimensions.width = Size::Fixed(width);
                element.position.x = Coordinate::Fixed(x);
            }),
        );
    }
    {
        let n = n.clone();
        let h_zoom = globals.viewport.h_zoom.clone();
        note.subscribe_mutation_to_reactive(
            &globals.viewport.time_scroll,
            Box::new(move |element: &mut Element, new_value: &f32| {
                let start = n.get_copy().start;
                let x = keyboard_width + (start as f32 - new_value) as f32 * h_zoom.get_copy();
                element.position.x = Coordinate::Fixed(x);
            }),
        );
    }
    {
        let time_scroll = globals.viewport.time_scroll.clone();
        let h_zoom = globals.viewport.h_zoom.clone();
        let v_zoom = globals.viewport.v_zoom.clone();
        note.subscribe_mutation_to_reactive(
            &n,
            Box::new(move |element: &mut Element, n: &Note| {
                let time_scroll = time_scroll.get_copy();
                let h_zoom = h_zoom.get_copy();
                let width = n.length as f32 * h_zoom;
                let x = x_of_time_no_global_access(n.start, keyboard_width, time_scroll, h_zoom);

                let note_height = v_zoom.get_copy();
                let y = note_height * n_init.note as f32;

                element.dimensions.width = Size::Fixed(width);
                element.position.x = Coordinate::Fixed(x);
                element.position.y = Coordinate::Fixed(y);
            }),
        );
    }
    {
        let n = n.clone();
        note.subscribe_mutation_to_reactive(
            &globals.viewport.v_zoom,
            Box::new(move |element: &mut Element, new_value: &f32| {
                let n = n.get_copy();
                let note_height = new_value;
                let y = note_height * n.note as f32;

                element.position.y = Coordinate::Fixed(y);
                element.dimensions.height = Size::Fixed(*note_height);
            }),
        );
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
