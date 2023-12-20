use sdl2::sys::KeyCode;

use crate::event_subscriptions::Key;

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
