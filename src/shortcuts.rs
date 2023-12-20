use std::{cell::RefCell, rc::Rc};

use sdl2::sys::KeyCode;

use crate::{event_subscriptions::Key, global::Globals, project::Action};

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
            globals.loaded_project.perform_action(Action::ChangeTempo(globals.loaded_project.tempo.get_copy() + 1.))
        }),
    );
}

pub fn perma_bind(globals: &mut Globals, key: Key, callback: Box<dyn Fn(&mut Globals)>) {
    let callback = Rc::new(callback);
    globals.subscriptions.subscribe_key(Rc::new(RefCell::new(
        move |pressed_key: &Key, globals: &mut Globals| {
            let callback = callback.clone();
            if pressed_key == &key {
                callback(globals);
            }
        },
    )));
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
