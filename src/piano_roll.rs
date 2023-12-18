use std::{cell::RefCell, rc::Rc};

use crate::{
    global::Globals,
    midi::*,
    selection::{NoteRef, Selection},
    track::TrackId,
    ui::element::*,
    ui::*,
    ui::{style::*, text::Text},
    utils::note_name,
    Zoom,
};

pub fn e_piano_roll(
    gl: &glow::Context,
    globals: &Globals,
    midi: &MidiClip,
    track_id: TrackId,
    needs_rerender: Rc<RefCell<bool>>,
) -> Element {
    let zoom: Zoom = Zoom {
        h_zoom: 2.,
        v_zoom: 12.,
    };

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
                    &zoom,
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
    zoom: &Zoom,
    globals: &Globals,
    notes: &Vec<(&Note, bool)>,
    needs_rerender: Rc<RefCell<bool>>,
) -> Element {
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

    let note_height = zoom.v_zoom;
    let y = note_height * note as f32;

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

    for (n, selected) in notes.iter() {
        let x = KEYBOARD_WIDTH + n.start as f32 * zoom.h_zoom;
        let width = n.length as f32 * zoom.h_zoom;

        let mut note_style = Style::default();
        note_style.background_colour = globals.colour_palette.white;
        note_style.border_width = 1.;
        note_style.border_colour = globals.colour_palette.time_grid;

        if *selected {
            note_style.border_colour = globals.colour_palette.selected;
            note_style.border_width = 2.;
        }

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

        children.push(note);
    }

    Element::new(
        gl,
        p(0., y),
        Size::FractionOfParent(1.),
        Size::Fixed(note_height),
        Some(row_style),
        None,
        needs_rerender.clone(),
        children,
    )
}

fn is_black_key(note: i32) -> bool {
    let note = note % 12;
    note == 1 || note == 3 || note == 6 || note == 8 || note == 10
}
