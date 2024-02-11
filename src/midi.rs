use std::{ops::{Index, IndexMut}, collections::HashMap};

use crate::{ui::{
    reactive::Reactive,
    reactive_list::{ReactiveList, ReactiveListKey},
}, audio::{SampleRate, BlockSize}};

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
        MidiClip {
            notes: ReactiveList::new(),
        }
    }

    pub fn get_note(&self, key: ReactiveListKey) -> Option<Reactive<Note>> {
        self.notes.get_copy_of_item(key)
    }
}

#[derive(Clone)]
pub struct MidiEvent {
    pub time: Time,
    pub data: MidiEventData,
}

#[derive(Clone)]
pub enum MidiEventData {
    NoteOn { note: NoteEvent },
    NoteOff { note: NoteEvent },
}

#[derive(Clone, Copy)]
pub struct NoteEvent {
    pub note: u32,
    pub velocity: u32,
}

impl MidiEvent {
    pub fn status_byte(&self) -> u8 {
        match self.data {
            MidiEventData::NoteOn { note: _ } => 0x90,
            MidiEventData::NoteOff { note: _ } => 0x80,
        }
    }

    pub fn note(&self) -> Option<NoteEvent> {
        match self.data {
            MidiEventData::NoteOn { note } => Some(note),
            MidiEventData::NoteOff { note } => Some(note),
        }
    }
}

impl Note {
    pub fn to_midi_events(&self) -> Vec<MidiEvent> {
        vec![
            MidiEvent {
                time: self.start,
                data: MidiEventData::NoteOn {
                    note: NoteEvent {
                        note: self.note,
                        velocity: self.velocity,
                    },
                },
            },
            MidiEvent {
                time: self.start + self.length,
                data: MidiEventData::NoteOff {
                    note: NoteEvent {
                        note: self.note,
                        velocity: 0,
                    },
                },
            },
        ]
    }
}

pub type BlockId = usize;
pub type EventId = usize;

struct MidiEventsBlockList {
    pub blocks: HashMap<BlockId, MidiEventsBlock>,
    event_locations: HashMap<EventId, BlockId>,
    id_counter: EventId,
}

fn get_block_id(time: Time) -> BlockId {
    unimplemented!()
}

impl MidiEventsBlockList {
    pub fn new() -> Self {
        MidiEventsBlockList {
            blocks: HashMap::new(),
            event_locations: HashMap::new(),
            id_counter: 0,
        }
    }

    pub fn insert_event(&mut self, event: MidiEvent) -> EventId {
        let block_id = get_block_id(event.time);
        let event_id = self.id();
        self.blocks
            .entry(block_id)
            .or_insert_with(HashMap::new)
            .insert(event_id, event);
        self.event_locations.insert(event_id, block_id);
        event_id
    }

    pub fn delete_event(&mut self, event_id: EventId) {
        let block_id = self.event_locations[&event_id];
        self.blocks.get_mut(&block_id).unwrap().remove(&event_id);
        self.event_locations.remove(&event_id);
    }

    fn id(&mut self) -> EventId {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }

}

pub type MidiEventsBlock = HashMap<EventId, MidiEvent>;