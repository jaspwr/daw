pub struct Note {
    pub note: u32,
    pub velocity: u32,
    pub start: u32,
    pub length: u32,
}

pub struct MidiClip {
    pub notes: Vec<Note>,
}
