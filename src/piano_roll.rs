use crate::{Zoom, ui::*, ui::element::*, ui::style::*, global::Globals, midi::*};

pub fn e_piano_roll(gl: &glow::Context, globals: &Globals, midi: &MidiClip) -> Element {
    let zoom: Zoom = Zoom {
        h_zoom: 2.,
        v_zoom: 12.,
    };

    let mut style = Style::default();
    style.render_self = false;

    let mut notes: Vec<Vec<&Note>> = vec![];
    for _ in 0..128 {
        notes.push(vec![]);
    }

    for n in midi.notes.iter() {
        notes[n.note as usize].push(n);
    }

    Element::new(
        gl,
        0.,
        0.,
        Size::FractionOfParent(1.),
        Size::FractionOfParent(1.),
        Some(style),
        (0..=127)
            .map(|n| e_note(gl, n, &zoom, globals, &notes[n as usize]))
            .collect(),
    )
}

fn e_note(
    gl: &glow::Context,
    note: i32,
    zoom: &Zoom,
    globals: &Globals,
    notes: &Vec<&Note>,
) -> Element {
    let mut row_style = Style::default();
    row_style.border_width = 1.;
    row_style.border_colour = globals.colour_palette.time_grid;

    let mut key_style = Style::default();

    key_style.border_width = 1.;
    key_style.border_colour = globals.colour_palette.time_grid;

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

    let key = Element::new(
        gl,
        0.,
        0.,
        Size::Fixed(KEYBOARD_WIDTH),
        Size::FractionOfParent(1.),
        Some(key_style),
        vec![],
    );

    let mut children = vec![key];

    for n in notes.iter() {
        let x = KEYBOARD_WIDTH + n.start as f32 * zoom.h_zoom;
        let width = n.length as f32 * zoom.h_zoom;

        let mut note_style = Style::default();
        note_style.background_colour = globals.colour_palette.white;
        note_style.border_width = 1.;
        note_style.border_colour = globals.colour_palette.time_grid;

        let note = Element::new(
            gl,
            x,
            0.,
            Size::Fixed(width),
            Size::FractionOfParent(1.),
            Some(note_style),
            vec![],
        );

        children.push(note);
    }

    Element::new(
        gl,
        0.,
        y,
        Size::FractionOfParent(1.),
        Size::Fixed(note_height),
        Some(row_style),
        children,
    )
}

fn is_black_key(note: i32) -> bool {
    let note = note % 12;
    note == 1 || note == 3 || note == 6 || note == 8 || note == 10
}

