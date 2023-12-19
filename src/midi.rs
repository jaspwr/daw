#[derive(Clone)]
pub struct Note {
    pub note: u32,
    pub velocity: u32,
    pub start: u32,
    pub length: u32,
}

#[derive(Clone)]
pub struct MidiClip {
    pub notes: Vec<Note>,
}

impl MidiClip {
    pub fn new() -> Self {
        MidiClip { notes: vec![] }
    }
}
