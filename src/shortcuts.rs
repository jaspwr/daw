use std::{cell::RefCell, rc::Rc};

use sdl2::{sys::{KeyCode, SDL_KeyCode}, keyboard::Scancode};

use crate::{
    event_subscriptions::Key,
    global::{Globals, PlayingState, self},
    midi::Note,
    project::Action,
    track,
    ui::reactive::Reactive,
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
        }),
    );
    perma_bind(
        globals,
        k("^r"),
        Box::new(|globals| {
            globals.loaded_project.redo();
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

    perma_bind(globals, k("^="), Box::new(|globals| {
        globals.viewport.h_zoom *= 1.1;
    }));

    perma_bind(globals, k("^-"), Box::new(|globals| {
        globals.viewport.h_zoom /= 1.1;
    }));

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
        }),
    );

    perma_bind(
        globals,
        k("r"),
        Box::new(|globals| {
            globals.playing_state = PlayingState::Recording;
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
            for _ in 0..globals.shortcuts_buffer.get_amount() {
                globals.loaded_project.player_time -= 1.;
            }
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(
        globals,
        k("l"),
        Box::new(|globals| {
            for _ in 0..globals.shortcuts_buffer.get_amount() {
                globals.loaded_project.player_time += 1.;
            }
            globals.shortcuts_buffer.clear();
        }),
    );

    perma_bind(globals, ESCAPE, Box::new(|globals| {
        globals.shortcuts_buffer.clear();
    }));
}

fn handle_number_press(globals: &mut Globals, i: i32) {
    if globals.shortcuts_buffer.keys.len() == 0 {
        globals
            .shortcuts_buffer
            .amount_modifier
            .mutate(Box::new(move |a| match a {
                Some(a_current) => {
                    if (*a_current).abs() > 10000 {
                        return;
                    }
                    *a = Some(*a_current * 10 + i);
                }
                None => {
                    *a = Some(i);
                }
            }));
    }
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

pub fn perma_bind(globals: &mut Globals, key: Key, callback: Box<dyn Fn(&mut Globals)>) {
    let callback = Rc::new(callback);
    globals.subscriptions.subscribe_key(Rc::new(RefCell::new(
        move |pressed_key: &Key, globals: &mut Globals| {
            if globals.editor_context.grabs_keyboard() {
                return;
            }

            if pressed_key == &key {
                (callback.clone())(globals);
            }
        },
    )));
}

pub fn add_chord(globals: &mut Globals, notes: Vec<Note>, track_id: u32) {
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
