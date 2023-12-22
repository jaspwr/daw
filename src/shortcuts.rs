use std::{
    arch::global_asm,
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use sdl2::{
    keyboard::Scancode,
    libc::{creat, glob},
    sys::{KeyCode, SDL_KeyCode},
};

use crate::{
    event_subscriptions::Key,
    global::{self, EditingContext, Globals, PlayingState},
    midi::{Note, Time},
    project::{Action, TimeSignature},
    selection::Selection,
    track::{self, TrackData, TrackId},
    ui::{reactive::Reactive, reactive_list::ReactiveListKey},
};

pub struct ShortcutsBuffer {
    pub keys: Vec<Key>,
    pub amount_modifier: Reactive<Option<i32>>,
}

impl ShortcutsBuffer {
    pub fn new() -> Self {
        Self {
            keys: vec![],
            amount_modifier: Reactive::new(None),
        }
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.amount_modifier <<= None;
    }

    pub fn get_amount(&self) -> i32 {
        match self.amount_modifier.get_copy() {
            Some(amount) => amount,
            None => 1,
        }
    }
}

pub fn universal_shortcuts(globals: &mut Globals) {
    perma_bind(
        globals,
        k("u"),
        Box::new(|globals| {
            globals.loaded_project.undo();
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("^r"),
        Box::new(|globals| {
            globals.loaded_project.redo();
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        BACKSPACE,
        Box::new(|globals| {
            globals.loaded_project.undo();
        }),
    );

    perma_bind(
        globals,
        k("^s"),
        Box::new(|globals| {
            globals.loaded_project.perform_action(Action::AddMidiNote {
                track_id: 0,
                note: Note {
                    start: 0.,
                    length: 100.,
                    note: 61,
                    velocity: 100,
                },
            })
        }),
    );

    perma_bind(
        globals,
        k("^="),
        Box::new(|globals| {
            globals.viewport.h_zoom *= 1.1;
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("^-"),
        Box::new(|globals| {
            globals.viewport.h_zoom /= 1.1;
            globals.shortcuts_buffer.clear();
        }),
    );

    for i in 0..10 {
        perma_bind(
            globals,
            k(&i.to_string()),
            Box::new(move |globals| {
                handle_number_press(globals, i);
            }),
        );
    }

    perma_bind(
        globals,
        k(" "),
        Box::new(|globals| {
            globals.playing_state.play_pause();
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("r"),
        Box::new(|globals| {
            globals.playing_state = PlayingState::Recording;
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(globals, k("A"), Box::new(|globals| {
        globals.loaded_project.selection <<= Selection::None;
    }));

    perma_bind(
        globals,
        k("g"),
        Box::new(|globals| {
            globals
                .loaded_project
                .perform_action(Action::MoveTimeCursor(0.));
        }),
    );

    perma_bind(
        globals,
        LEFT,
        Box::new(|globals| {
            globals.viewport.time_scroll -= 1.;
        }),
    );

    perma_bind(
        globals,
        RIGHT,
        Box::new(|globals| {
            globals.viewport.time_scroll += 1.;
        }),
    );

    perma_bind(
        globals,
        k("h"),
        Box::new(|globals| {
            let num_mod = globals.shortcuts_buffer.get_amount();

            globals.loaded_project.perform_action(Action::MoveTimeCursor(
                globals.loaded_project.player_time.get_copy() - num_mod as Time,
            ));

            let offset: Time = -num_mod as Time;

            offset_selected_notes_in_time(globals, offset);

            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("l"),
        Box::new(|globals| {
            let num_mod = globals.shortcuts_buffer.get_amount();

            globals.loaded_project.perform_action(Action::MoveTimeCursor(
                globals.loaded_project.player_time.get_copy() + num_mod as Time,
            ));

            let offset: Time = num_mod as Time;

            offset_selected_notes_in_time(globals, offset);

            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("j"),
        Box::new(|globals| {
            offset_selected_notes_vertically(globals, -globals.shortcuts_buffer.get_amount(), true);
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("k"),
        Box::new(|globals| {
            offset_selected_notes_vertically(globals, globals.shortcuts_buffer.get_amount(), true);
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("J"),
        Box::new(|globals| {
            offset_selected_notes_vertically(
                globals,
                -globals.shortcuts_buffer.get_amount() * 12,
                false,
            );
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("K"),
        Box::new(|globals| {
            offset_selected_notes_vertically(
                globals,
                globals.shortcuts_buffer.get_amount() * 12,
                false,
            );
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("L"),
        Box::new(|globals| {
            let dt = globals.shortcuts_buffer.get_amount() as Time;
            multiply_selected_notes_length(globals, 2. * dt);
            globals.shortcuts_buffer.clear();
        })
    );

    perma_bind(
        globals,
        k("H"),
        Box::new(|globals| {
            let dt = globals.shortcuts_buffer.get_amount() as Time;
            multiply_selected_notes_length(globals, 0.5 / dt);
            globals.shortcuts_buffer.clear();
        })
    );

    perma_bind(
        globals,
        k("^l"),
        Box::new(|globals| {
            let dt = globals.shortcuts_buffer.get_amount() as Time;
            offset_selected_notes_length(globals, dt);
            globals.shortcuts_buffer.clear();
        })
    );

    perma_bind(
        globals,
        k("^h"),
        Box::new(|globals| {
            let dt = globals.shortcuts_buffer.get_amount() as Time;
            offset_selected_notes_length(globals, -dt);
            globals.shortcuts_buffer.clear();
        })
    );

    perma_bind(
        globals,
        k("o"),
        Box::new(|globals| {
            let bar_length = globals
                .loaded_project
                .time_signature
                .get_copy()
                .beats_per_measure() as Time;

            let num_modifier = globals.shortcuts_buffer.get_amount() as Time;
            globals.loaded_project.player_time += bar_length * num_modifier;
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("O"),
        Box::new(|globals| {
            let bar_length = globals
                .loaded_project
                .time_signature
                .get_copy()
                .beats_per_measure() as Time;

            let num_modifier = globals.shortcuts_buffer.get_amount() as Time;
            globals.loaded_project.player_time -= bar_length * num_modifier;
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("c"),
        Box::new(|globals| {
            if globals.shortcuts_buffer.keys.len() == 0
                && globals.editor_context.get_copy() == EditingContext::PianoRoll
            {
                globals.shortcuts_buffer.keys.push(k("c"));
            }
        }),
    );

    perma_bind(
        globals,
        k("q"),
        Box::new(|globals| {
            globals.shortcuts_buffer.clear();

            perform_actions_on_selected_notes(
                globals,
                Box::new(|globals, note_id, track_id| quantize_note(globals, note_id, track_id)),
            );
        }),
    );

    perma_bind(
        globals,
        k("^j"),
        Box::new(|globals| {
            offset_selected_notes_vertically(
                globals,
                -globals.shortcuts_buffer.get_amount(),
                false,
            );
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("^k"),
        Box::new(|globals| {
            offset_selected_notes_vertically(globals, globals.shortcuts_buffer.get_amount(), false);
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("x"),
        Box::new(|globals| {
            let mut times = globals.shortcuts_buffer.get_amount();
            if times < 2 {
                times = 2;
            }
            split_selected_notes(globals, times);
        }),
    );

    perma_bind(
        globals,
        k("d"),
        Box::new(|globals| {
            delete_selected_notes(globals);
        }),
    );

    perma_bind(globals, k("^p"), Box::new(|globals| {
        globals.editor_context <<= EditingContext::CommandPallet;
    }));

    perma_bind(
        globals,
        k("^a"),
        Box::new(|globals| {
            // FIXME: hardcoded for piano roll and track 0

            globals.shortcuts_buffer.clear();

            let mut selection: HashMap<TrackId, HashSet<ReactiveListKey>> = HashMap::new();

            let track_id: TrackId = 0;
            if let TrackData::Midi(_, notes) = &globals.loaded_project.tracks[track_id].data {
                let notes = notes.notes.copy_of_whole_list();
                let ids: HashSet<ReactiveListKey> = notes.into_iter().map(|(id, _)| id).collect();
                selection.insert(track_id, ids);
            }

            let selection = Selection::MidiNotes(selection);

            globals
                .loaded_project
                .perform_action(Action::SetSelection(selection));
        }),
    );

    perma_bind(
        globals,
        ESCAPE,
        Box::new(|globals| {
            globals.shortcuts_buffer.clear();
        }),
    );
}

fn delete_selected_notes(globals: &mut Globals) {
    perform_actions_on_selected_notes(
        globals,
        Box::new(move |globals, note_id, track_id| {
            vec![Action::RemoveMidiNote { track_id, note_id }]
        }),
    );
    globals.loaded_project.selection <<= Selection::None;
}

fn offset_selected_notes_in_time(globals: &mut Globals, offest: Time) {
    perform_actions_on_selected_notes(
        globals,
        Box::new(move |globals: &mut Globals, note_id, track_id| {
            globals.loaded_project.tracks[track_id]
                .get_note_from_id(note_id)
                .map(|note| {
                    let mut new_note = note.get_copy();

                    new_note.start += offest;

                    vec![Action::ModifyMidiNote {
                        track_id,
                        note_id,
                        new_note,
                    }]
                })
                .unwrap_or(vec![])
        }),
    );
}

fn offset_selected_notes_vertically(globals: &mut Globals, offset: i32, by_scale_degree: bool) {
    if let Selection::MidiNotes(map) = globals.loaded_project.selection.get_copy() {
        let actions = map
            .into_iter()
            .map(|(track_id, notes)| {
                move_notes_vertically(globals, notes, track_id, offset, by_scale_degree)
            })
            .fold(vec![], |mut acc, mut actions| {
                acc.append(&mut actions);
                acc
            });

        let action_group = Action::Group(actions);

        globals.loaded_project.perform_action(action_group);
    }
}

fn offset_selected_notes_length(globals: &mut Globals, time: Time) {
    perform_actions_on_selected_notes(globals, Box::new(move |globals: &mut Globals, note_id, track_id| {
        globals.loaded_project.tracks[track_id].get_note_from_id(note_id)
            .map(|n| {
                let mut new_note = n.get_copy();

                let pre_len = new_note.length;

                new_note.length += time;

                if new_note.length <= 0. {
                    new_note.length = pre_len;
                }

                vec![Action::ModifyMidiNote { track_id, note_id, new_note }]
            })
            .unwrap_or(vec![])
    }));
}

fn multiply_selected_notes_length(globals: &mut Globals, time: Time) {
    perform_actions_on_selected_notes(globals, Box::new(move |globals: &mut Globals, note_id, track_id| {
        globals.loaded_project.tracks[track_id].get_note_from_id(note_id)
            .map(|n| {
                let mut new_note = n.get_copy();

                let pre_len = new_note.length;

                new_note.length *= time;

                if new_note.length <= 0. {
                    new_note.length = pre_len;
                }

                vec![Action::ModifyMidiNote { track_id, note_id, new_note }]
            })
            .unwrap_or(vec![])
    }));
}

fn split_selected_notes(globals: &mut Globals, times: i32) {
    let actions = Box::new(move |globals: &mut Globals, note_id, track_id| {
        split_note(note_id, globals, track_id, times)
    });

    perform_actions_on_selected_notes(globals, actions);
    globals.loaded_project.selection <<= Selection::None;
}

fn quantize_note(
    globals: &mut Globals,
    note_id: ReactiveListKey,
    track_id: TrackId,
) -> Vec<Action> {
    globals.loaded_project.tracks[track_id]
        .get_note_from_id(note_id)
        .map(|note| {
            let mut new_note = note.get_copy();

            new_note.start = new_note.start.round();

            vec![Action::ModifyMidiNote {
                track_id,
                note_id,
                new_note,
            }]
        })
        .unwrap_or(vec![])
}

fn perform_actions_on_selected_notes(
    globals: &mut Globals,
    create_actions: Box<dyn Fn(&mut Globals, ReactiveListKey, TrackId) -> Vec<Action>>,
) {
    if let Selection::MidiNotes(map) = globals.loaded_project.selection.get_copy() {
        let actions = map
            .into_iter()
            .map(|(track_id, notes)| {
                notes
                    .into_iter()
                    .map(|n| create_actions(globals, n, track_id))
                    .fold(vec![], |mut acc, mut actions| {
                        acc.append(&mut actions);
                        acc
                    })
            })
            .fold(vec![], |mut acc, mut actions| {
                acc.append(&mut actions);
                acc
            });

        let action_group = Action::Group(actions);

        globals.loaded_project.perform_action(action_group);
    }
}

fn split_note(
    note_id: ReactiveListKey,
    globals: &mut Globals,
    track_id: u32,
    times: i32,
) -> Vec<Action> {
    let note = globals.loaded_project.tracks[track_id]
        .get_note_from_id(note_id)
        .unwrap()
        .get_copy();

    let length = note.length / times as Time;

    let mut actions = vec![Action::RemoveMidiNote { track_id, note_id }];

    for i in 0..times {
        actions.push(Action::AddMidiNote {
            track_id,
            note: Note {
                start: note.start + i as f64 * length,
                length,
                note: note.note,
                velocity: note.velocity,
            },
        });
    }

    actions
}

fn handle_number_press(globals: &mut Globals, num: i32) {
    if globals.shortcuts_buffer.keys.len() == 0 {
        globals
            .shortcuts_buffer
            .amount_modifier
            .mutate(Box::new(move |a| match a {
                Some(a_current) => {
                    if (*a_current).abs() > 10000 {
                        return;
                    }
                    *a = Some(*a_current * 10 + num);
                }
                None => {
                    *a = Some(num);
                }
            }));
        return;
    }

    if let Some(key) = globals.shortcuts_buffer.keys.first() {
        if key == &k("c") {
            if num == 0 {
                globals.shortcuts_buffer.clear();
                return;
            }

            handle_chord_drawing(globals, num);

            return;
        }
    }
}

fn handle_chord_drawing(globals: &mut Globals, num: i32) {
    // TODO: length modifiers
    let length = globals
        .loaded_project
        .time_signature
        .get_copy()
        .beats_per_measure();

    let mut degrees = vec![1, 3, 5];

    if let Some(num_modifier) = globals.shortcuts_buffer.amount_modifier.get_copy() {
        if num_modifier == 5 {
            degrees = vec![1, 5];
        } else if (num_modifier - 1) % 2 == 0 {
            degrees = (0..=num_modifier)
                .filter(|n| n % 2 == 0)
                .map(|n| n + 1)
                .collect();
        }
    }

    let ks = globals.loaded_project.key_signature.get_copy();
    let chord = degrees
        .into_iter()
        .map(|d| Note {
            note: ks.from_degree(d - 1 + num, 5),
            velocity: 100,
            start: globals.loaded_project.player_time.get_copy(),
            length: length as Time,
        })
        .collect();

    let t = globals.loaded_project.player_time.get_copy();

    add_chord(globals, chord, 0);

    globals
        .loaded_project
        .perform_action(Action::MoveTimeCursor(t + 4.));
}

const LEFT: Key = Key {
    code: SDL_KeyCode::SDLK_LEFT as KeyCode,
    control: false,
    shift: false,
};

const RIGHT: Key = Key {
    code: SDL_KeyCode::SDLK_RIGHT as KeyCode,
    control: false,
    shift: false,
};

const ESCAPE: Key = Key {
    code: SDL_KeyCode::SDLK_ESCAPE as KeyCode,
    control: false,
    shift: false,
};

const BACKSPACE: Key = Key {
    code: SDL_KeyCode::SDLK_BACKSPACE as KeyCode,
    control: false,
    shift: false,
};

pub fn perma_bind(globals: &mut Globals, key: Key, callback: Box<dyn Fn(&mut Globals)>) {
    let callback = Rc::new(callback);
    globals.subscriptions.subscribe_key(Rc::new(RefCell::new(
        move |pressed_key: &Key, globals: &mut Globals| {
            if globals.editor_context.get_copy().grabs_keyboard() {
                return;
            }

            if pressed_key == &key {
                (callback.clone())(globals);
            }
        },
    )));
}

pub fn add_chord(globals: &mut Globals, notes: Vec<Note>, track_id: TrackId) {
    let actions = notes
        .iter()
        .map(|note| Action::AddMidiNote {
            track_id,
            note: *note,
        })
        .collect();
    let action_group = Action::Group(actions);
    globals.loaded_project.perform_action(action_group);
}

pub fn move_notes_vertically(
    globals: &mut Globals,
    note_ids: HashSet<ReactiveListKey>,
    track_id: TrackId,
    offset: i32,
    by_scale_degree: bool,
) -> Vec<Action> {
    let track = &mut globals.loaded_project.tracks[track_id];
    let track_data = &mut track.data;

    let mut actions: Vec<Action> = vec![];

    let ks = globals.loaded_project.key_signature.get_copy();

    if let TrackData::Midi(_, notes) = track_data {
        let notes = &notes.notes;
        for note_id in note_ids {
            let note = notes.get_copy_of_item(note_id).unwrap();
            let note = note.get_copy();

            let new_note = {
                let non_scale_degree = Note {
                    note: (note.note as i32 + offset) as u32,
                    ..note
                };

                if by_scale_degree {
                    if let Some(mut d) = ks.to_degree(note.note as i32) {
                        d.degree += offset;
                        Note {
                            note: ks.from_degree(d.degree, d.octave),
                            ..note
                        }
                    } else {
                        non_scale_degree
                    }
                } else {
                    non_scale_degree
                }
            };

            actions.push(Action::ModifyMidiNote {
                track_id,
                note_id,
                new_note,
            });
            // let note = notes.notes.get_mut(note_id).unwrap();
            // note.note += offset;
        }
    }

    actions
}

pub fn k(symbol: &str) -> Key {
    key_from_symbol(symbol).unwrap()
}

pub fn key_from_symbol(symbol: &str) -> Option<Key> {
    let mut c: char = '_';
    let mut shift = false;
    let mut ctrl = false;

    if symbol.len() == 1 {
        c = symbol.chars().next().unwrap();
    } else if symbol.len() == 2 {
        if symbol.chars().next().unwrap() == '^' {
            c = symbol.chars().nth(1).unwrap();
            ctrl = true;
        } else {
            return None;
        }
    } else {
        return None;
    }

    shift = true;
    // Unshift
    match c {
        '!' => c = '1',
        '@' => c = '2',
        '#' => c = '3',
        '$' => c = '4',
        '%' => c = '5',
        '^' => c = '6',
        '&' => c = '7',
        '*' => c = '8',
        '(' => c = '9',
        ')' => c = '0',
        '_' => c = '-',
        '+' => c = '=',
        '{' => c = '[',
        '}' => c = ']',
        '|' => c = '\\',
        ':' => c = ';',
        '"' => c = '\'',
        '<' => c = ',',
        '>' => c = '.',
        '?' => c = '/',
        '~' => c = '`',
        _ => {
            shift = false;
        }
    }

    if c.is_ascii_uppercase() {
        c = c.to_ascii_lowercase();
        shift = true;
    }

    let c = c as u8;
    if c >= 0x20 && c <= 0x7e {
        return Some(Key {
            code: c as KeyCode,
            control: ctrl,
            shift,
        });
    }

    None
}
