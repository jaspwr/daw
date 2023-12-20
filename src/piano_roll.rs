use std::{cell::RefCell, rc::Rc};

use sdl2::libc::glob;

use crate::{
    global::{Globals, Viewport},
    midi::*,
    selection::{NoteRef, Selection},
    track::{TrackData, TrackId},
    ui::*,
    ui::{element::*, scroll_window::e_scroll_window},
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

    let mut notes: Vec<Vec<(&Note, bool)>> = vec![];
    for _ in 0..128 {
        notes.push(vec![]);
    }

    let track = globals.loaded_project.tracks[track_id].get().clone();
    let track_ = track.borrow();

    let midi = match &track_.data {
        TrackData::Midi(_, midi) => midi,
        _ => panic!("Track is not midi"),
    };

    for n in midi.notes.iter() {
        notes[n.note as usize].push((&n, false));
    }

    if let Selection::MidiNotes(selection) = &globals.selection {
        for (track, sel_notes) in selection.iter() {
            if *track == track_id {
                for sel_note in sel_notes.iter() {
                    for note in &mut notes[sel_note.note as usize] {
                        if note.0.start == sel_note.start {
                            note.1 = true;
                        }
                    }
                }
                break;
            }
        }
    }

    let roll = Element::new(
        gl,
        Position::origin(),
        Size::FractionOfParent(1.),
        Size::FractionOfParent(1.),
        Some(style),
        None,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        (0..=127)
            .map(|n| {
                e_note(
                    gl,
                    n,
                    &globals.viewport,
                    globals,
                    &notes[n as usize],
                    needs_rerender.clone(),
                    frame_bounding_box.clone(),
                )
            })
            .collect(),
    );

    let scroll_win = e_scroll_window(
        gl,
        globals,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        true,
        false,
        vec![roll],
    );

    return scroll_win;
}

fn e_note(
    gl: &glow::Context,
    note: i32,
    viewport: &Viewport,
    globals: &Globals,
    notes: &Vec<(&Note, bool)>,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
) -> ElementRef {
    let mut row_style = Style::default();
    row_style.border_width = 1.;
    row_style.border_colour = globals.colour_palette.time_grid;

    let mut key_style = Style::default();

    key_style.border_width = 1.;
    key_style.border_colour = globals.colour_palette.time_grid;
    key_style.padding_left = 5.;
    key_style.padding_top = -1.;

    (key_style.background_colour, row_style.background_colour) = if is_black_key(note) {
        (
            globals.colour_palette.black_key,
            globals.colour_palette.black_key_piano_roll_row,
        )
    } else {
        (
            globals.colour_palette.white_key,
            globals.colour_palette.white_key_piano_roll_row,
        )
    };

    let text_colour = if is_black_key(note) {
        globals.colour_palette.white
    } else {
        globals.colour_palette.black
    };

    let note_height = viewport.v_zoom.get().borrow().clone();

    const KEYBOARD_WIDTH: f32 = 200.;

    let label = Text::new(
        gl,
        note_name(note as u8, true),
        12.,
        &globals.main_font,
        text_colour,
        p(0., 0.),
        needs_rerender.clone(),
    );

    let key = Element::new(
        gl,
        Position::origin(),
        Size::Fixed(KEYBOARD_WIDTH),
        Size::FractionOfParent(1.),
        Some(key_style),
        Some(label),
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        vec![],
    );

    let mut children = vec![key];

    let h_zoom = viewport.h_zoom.get_copy();
    let time_scroll = viewport.time_scroll.get_copy();

    for (n, selected) in notes.iter() {
        let width = n.length as f32 * h_zoom;

        let mut note_style = Style::default();
        note_style.background_colour = globals.colour_palette.white;
        note_style.border_width = 1.;
        note_style.border_colour = globals.colour_palette.time_grid;

        if *selected {
            note_style.border_colour = globals.colour_palette.selected;
            note_style.border_width = 2.;
        }

        let x = KEYBOARD_WIDTH + (n.start as f32 - time_scroll) as f32 * h_zoom;

        let note = Element::new(
            gl,
            p(x, 0.),
            Size::Fixed(width),
            Size::FractionOfParent(1.),
            Some(note_style),
            None,
            needs_rerender.clone(),
            frame_bounding_box.clone(),
            vec![],
        );

        let len = n.length;
        let start = n.start;
        let time_scroll = viewport.time_scroll.clone();
        note.subscribe_mutation_to_reactive(
            &globals.viewport.h_zoom,
            Box::new(move |element: &mut Element, new_value: &f32| {
                let width = len as f32 * new_value;
                element.dimensions.width = Size::Fixed(width);
                let time_scroll = time_scroll.get_copy();
                let x = KEYBOARD_WIDTH + (start as f32 - time_scroll) as f32 * new_value;
                element.position.x = Coordinate::Fixed(x);
            }),
        );

        let h_zoom = viewport.h_zoom.clone();
        let note_cpy = note.clone();
        let start = n.start;
        let sub = globals
            .viewport
            .time_scroll
            .subscribe(Box::new(move |new_value| {
                let new_value = new_value.clone();
                let h_zoom = h_zoom.clone();
                note_cpy.mutate(Box::new(move |element: &mut Element| {
                    let x = KEYBOARD_WIDTH + (start as f32 - new_value) as f32 * h_zoom.get_copy();
                    element.position.x = Coordinate::Fixed(x);
                }));
            }));

        let time_scroll = viewport.time_scroll.clone();
        note.add_cleanup_callback(Box::new(move || {
            time_scroll.unsubscribe(sub);
        }));

        children.push(note);
    }

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

fn is_black_key(note: i32) -> bool {
    let note = note % 12;
    note == 1 || note == 3 || note == 6 || note == 8 || note == 10
}
