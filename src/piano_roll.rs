use std::{cell::RefCell, rc::Rc};

use sdl2::libc::glob;

use crate::{
    global::{Globals, Viewport},
    midi::*,
    selection::{NoteRef, Selection},
    track::TrackId,
    ui::element::*,
    ui::*,
    ui::{style::*, text::Text},
    utils::note_name,
};

pub fn e_piano_roll(
    gl: &glow::Context,
    globals: &Globals,
    midi: &MidiClip,
    track_id: TrackId,
    needs_rerender: Rc<RefCell<bool>>,
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

    for n in midi.notes.iter() {
        notes[n.note as usize].push((n, false));
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

    Element::new(
        gl,
        Position::origin(),
        Size::FractionOfParent(1.),
        Size::FractionOfParent(1.),
        Some(style),
        None,
        needs_rerender.clone(),
        (0..=127)
            .map(|n| {
                e_note(
                    gl,
                    n,
                    &globals.viewport,
                    globals,
                    &notes[n as usize],
                    needs_rerender.clone(),
                )
            })
            .collect(),
    )
}

fn e_note(
    gl: &glow::Context,
    note: i32,
    viewport: &Viewport,
    globals: &Globals,
    notes: &Vec<(&Note, bool)>,
    needs_rerender: Rc<RefCell<bool>>,
) -> Rc<RefCell<Element>> {
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

    let note_height = viewport.v_zoom.get().borrow().clone();

    const KEYBOARD_WIDTH: f32 = 200.;

    let label = Text::new(
        gl,
        note_name(note as u8, true),
        12.,
        &globals.main_font,
        globals.colour_palette.black,
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
            vec![],
        );

        let note_cpy = note.clone();
        let len = n.length;
        let start = n.start;
        let time_scroll = viewport.time_scroll.clone();
        let sub = globals
            .viewport
            .h_zoom
            .subscribe(Box::new(move |new_value| {
                let new_value = new_value.clone();
                let time_scroll = time_scroll.clone();
                note_cpy
                    .borrow_mut()
                    .mutate(Box::new(move |element: &mut Element| {
                        let width = len as f32 * new_value;
                        element.dimensions.width = Size::Fixed(width);
                        let time_scroll = time_scroll.get_copy();
                        let x = KEYBOARD_WIDTH + (start as f32 - time_scroll) as f32 * new_value;
                        element.position.x = Coordinate::Fixed(x);
                    }));
            }));

        let h_zoom = viewport.h_zoom.clone();
        note.borrow_mut().on_cleanup.push(Box::new(move || {
            h_zoom.unsubscribe(sub);
        }));

        let h_zoom = viewport.h_zoom.clone();
        let note_cpy = note.clone();
        let start = n.start;
        let sub = globals
            .viewport
            .time_scroll
            .subscribe(Box::new(move |new_value| {
                let new_value = new_value.clone();
                let h_zoom = h_zoom.clone();
                note_cpy
                    .borrow_mut()
                    .mutate(Box::new(move |element: &mut Element| {
                        let x =
                            KEYBOARD_WIDTH + (start as f32 - new_value) as f32 * h_zoom.get_copy();
                        element.position.x = Coordinate::Fixed(x);
                    }));
            }));

        let time_scroll = viewport.time_scroll.clone();
        note.borrow_mut().on_cleanup.push(Box::new(move || {
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
        children,
    );

    let key_row_cpy = key_row.clone();
    let sub_id = viewport.v_zoom.subscribe(Box::new(move |new_value| {
        let new_value = new_value.clone();
        key_row_cpy
            .borrow_mut()
            .mutate(Box::new(move |element: &mut Element| {
                let note_height = new_value;
                let y = note_height * note as f32;

                element.position.y = Coordinate::Fixed(y);
                element.dimensions.height = Size::Fixed(note_height);
            }));
    }));

    let v_zoom = viewport.v_zoom.clone();
    key_row.borrow_mut().on_cleanup.push(Box::new(move || {
        v_zoom.unsubscribe(sub_id);
    }));

    key_row
}

fn is_black_key(note: i32) -> bool {
    let note = note % 12;
    note == 1 || note == 3 || note == 6 || note == 8 || note == 10
}
