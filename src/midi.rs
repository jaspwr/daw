use crate::ui::{reactive::Reactive, reactive_list::ReactiveList};


pub type Time = f64;

#[derive(Clone, Copy)]
pub struct Note {
    pub note: u32,
    pub velocity: u32,
    pub start: Time,
    pub length: Time,
}

#[derive(Clone)]
pub struct MidiClip {
    pub notes: ReactiveList<Reactive<Note>>,
}

impl MidiClip {
    pub fn new() -> Self {
        MidiClip { notes: ReactiveList::new() }
    }
}
