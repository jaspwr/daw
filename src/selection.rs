use std::{
    collections::{HashMap, HashSet},
    default,
};

use crate::{midi::Time, track::TrackId, ui::reactive_list::ReactiveListKey};

#[derive(Default, Clone)]
pub enum Selection {
    #[default]
    None,
    MidiNotes(HashMap<TrackId, HashSet<ReactiveListKey>>),
}

impl Selection {
    pub fn is_note_selected(&self, track_id: TrackId, note_id: ReactiveListKey) -> bool {
        match self {
            Selection::MidiNotes(track_map) => match track_map.get(&track_id) {
                Some(track) => track.contains(&note_id),
                _ => false,
            },
            _ => false,
        }
    }

    pub fn clear(&mut self) {
        *self = Selection::None;
    }
}
