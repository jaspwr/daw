use std::default;

use crate::{track::TrackId, midi::Time};

#[derive(Default)]
pub enum Selection {
    #[default]
    None,
    MidiNotes(Vec<(TrackId, Vec<NoteRef>)>),
}

pub struct NoteRef {
    pub note: u32,
    pub start: Time,
}