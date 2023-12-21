use std::{rc::Rc, cell::RefCell};

pub fn note_name(note: u8, show_octave: bool) -> String {
    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    let octave = note / 12;
    let note = note % 12;

    if show_octave {
        format!("{}{}", note_names[note as usize], octave)
    } else {
        note_names[note as usize].to_string()
    }
}

// TODO: These should probably have less misleading names.
pub fn malloc<T>(data: T) -> *mut T {
    Box::into_raw(Box::new(data))
}

pub unsafe fn free<T>(ptr: *mut T) {
    drop(Box::from_raw(ptr));
}

pub unsafe fn fetch_ptr<T>(ptr: *mut T) -> Box<T> {
    Box::from_raw(ptr)
}

pub fn leak<T>(box_: Box<T>) -> *mut T {
    Box::into_raw(box_)
}

/// You wouldn't understand, bro... You just wouldn't get it.
pub type RcRefCell<T> = Rc<RefCell<T>>;

pub fn rc_ref_cell<T>(data: T) -> RcRefCell<T> {
    Rc::new(RefCell::new(data))
}
