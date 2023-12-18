use std::collections::HashMap;

use crate::{midi::MidiClip, ui::style::Colour};

pub type TrackId = u32;

struct Instrument {}

pub struct Track {
    uid: TrackId,
    name: String,
    colour: Colour,
    data: TrackData,
}

enum TrackData {
    Midi(Option<Instrument>, Vec<MidiClip>),
    Audio(Vec<f32>),
}

struct Tracks {
    tracks: HashMap<TrackId, Track>,
}