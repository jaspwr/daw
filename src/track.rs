use std::{
    borrow::BorrowMut,
    collections::HashMap,
    ops::{Index, IndexMut},
    sync::atomic::AtomicU32,
};

use crate::{
    midi::MidiClip,
    ui::{reactive::Reactive, style::Colour},
};

static TRACK_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

pub type TrackId = u32;

#[derive(Clone)]
pub struct Instrument {}

#[derive(Clone)]
pub struct Track {
    pub uid: TrackId,
    pub name: String,
    pub colour: Colour,
    pub type_: TrackType,
    pub data: TrackData,
}

impl Track {
    pub fn new(type_: TrackType) -> Self {
        Track {
            uid: TRACK_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            name: "Untitled".to_string(),
            colour: Colour {
                r: 1.,
                g: 1.,
                b: 1.,
                a: 1.,
            },
            type_,
            data: TrackData::new(type_),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum TrackType {
    Midi,
    Audio,
}

#[derive(Clone)]
pub enum TrackData {
    Midi(Option<Instrument>, MidiClip),
    Audio(Vec<f32>),
}

impl TrackData {
    pub fn new(type_: TrackType) -> Self {
        match type_ {
            TrackType::Midi => TrackData::Midi(None, MidiClip::new()),
            TrackType::Audio => TrackData::Audio(vec![]),
        }
    }
}

pub struct TrackGroup {
    pub tracks: HashMap<TrackId, Reactive<Track>>,
}

impl TrackGroup {
    pub fn new() -> Self {
        TrackGroup {
            tracks: HashMap::new(),
        }
    }

    pub fn append(&mut self, track: Track) {
        self.tracks.insert(track.uid, Reactive::new(track));
    }

    pub fn add_new(&mut self, type_: TrackType) {
        self.append(Track::new(type_));
    }

    pub fn delete(&mut self, track_id: TrackId) {
        self.tracks.remove(&track_id);
    }
}

impl Index<TrackId> for TrackGroup {
    type Output = Reactive<Track>;

    fn index<'a>(&'a self, index: TrackId) -> &'a Self::Output {
        self.tracks.get(&index).unwrap()
    }
}

impl IndexMut<TrackId> for TrackGroup {
    fn index_mut<'a>(&'a mut self, index: TrackId) -> &'a mut Self::Output {
        self.tracks.get_mut(&index).unwrap()
    }
}
