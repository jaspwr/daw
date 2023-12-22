use core::panic;

use crate::{
    midi::{Note, Time},
    track::{self, Track, TrackData, TrackGroup, TrackType},
    ui::{reactive::Reactive, reactive_list::ReactiveListKey},
    utils::note_name, selection::Selection,
};

pub struct Project {
    pub meta: ProjectMeta,
    pub selection: Reactive<Selection>,
    pub tempo: Reactive<f32>,
    pub tracks: TrackGroup,
    pub player_time: Reactive<Time>,
    pub key_signature: Reactive<KeySignature>,
    pub time_signature: Reactive<TimeSignature>,
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}

pub struct ProjectMeta {
    pub name: String,
    pub description: String,
    pub version: String,
}

impl Project {
    pub fn new() -> Self {
        let mut project = Project {
            meta: ProjectMeta {
                name: "Untitled".to_string(),
                description: "No description".to_string(),
                version: "0.0.1".to_string(),
            },
            selection: Reactive::new(Selection::default()),
            key_signature: Reactive::new(KeySignature::new(0, KeyMode::Major)),
            tempo: Reactive::new(120.),
            tracks: TrackGroup::new(),
            player_time: Reactive::new(0.),
            time_signature: Reactive::new(TimeSignature::common()),
            undo_stack: vec![],
            redo_stack: vec![],
        };

        project.tracks.add_new(TrackType::Midi);

        return project;
    }

    pub fn undo(&mut self) {
        loop {
            if let Some(action) = self.undo_stack.pop() {
                self.__perform_action(&action, ActionType::Undo);
                if !action.undoes_automatically() {
                    return;
                }
            } else {
                break;
            }
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            self.__perform_action(&action, ActionType::Redo);
        }

        println!("{}", self.redo_stack.len());

        loop {
            if let Some(action) = self.redo_stack.last() {
                if !action.undoes_automatically() {
                    return;
                }
            } else {
                return;
            }

            if let Some(action) = self.redo_stack.pop() {
                self.__perform_action(&action, ActionType::Redo);
            }
        }
    }

    pub fn handle_inverse_action(
        &mut self,
        action: Action,
        type_of_action_just_performed: ActionType,
    ) {
        match type_of_action_just_performed {
            ActionType::Normal => {
                self.undo_stack.push(action);
                self.redo_stack.clear();
            }
            ActionType::Undo => self.redo_stack.push(action),
            ActionType::Redo => self.undo_stack.push(action),
        }
    }

    pub fn perform_action(&mut self, action: Action) {
        self.__perform_action(&action, ActionType::Normal);
    }

    fn __perform_action(&mut self, action: &Action, type_: ActionType) {
        let mut inverse: Option<Action> = None;
        match action {
            Action::Group(actions) => {
                for action in actions {
                    // HACK: Hardcoded as normal action so that the inverse actions
                    //       are added to the undo stack.
                    self.__perform_action(action, ActionType::Normal);
                }

                // Collect all the inverse actions that just got pushed to the undo stack
                // and put them in a group.
                let mut inverse_actions = vec![];

                self.undo_stack
                    .drain(self.undo_stack.len() - actions.len()..)
                    .for_each(|action| inverse_actions.push(action));

                inverse = Some(Action::Group(inverse_actions));
            }
            Action::MoveTimeCursor(t) => {
                inverse = Some(Action::MoveTimeCursor(self.player_time.get_copy()));
                self.player_time <<= *t;
            }
            Action::ChangeTempo(new_tempo) => {
                inverse = Some(Action::ChangeTempo(self.tempo.get_copy()));
                self.tempo <<= *new_tempo;
            }
            Action::AddMidiNote { track_id, note } => {
                let note_id = self.tracks[*track_id]
                    .push_note(*note)
                    .expect("Tried to add MIDI note to non MIDI track");
                inverse = Some(Action::RemoveMidiNote {
                    track_id: *track_id,
                    note_id,
                })
            }
            Action::RemoveMidiNote { track_id, note_id } => {
                let note = self.tracks[*track_id]
                    .get_note_from_id(*note_id)
                    .expect("Tried to delete MIDI note that was not found.")
                    .get_copy();
                inverse = Some(Action::AddMidiNote {
                    track_id: *track_id,
                    note,
                });
                self.tracks[*track_id].remove_note(*note_id);
            },
            Action::SetSelection(sel) => {
                inverse = Some(Action::SetSelection(self.selection.get_copy()));
                self.selection <<= sel.clone();
            },
            Action::ModifyMidiNote { track_id, note_id, new_note } => {
                if let Some(note) = &mut self.tracks[*track_id].get_note_from_id(*note_id) {
                    inverse = Some(Action::ModifyMidiNote {
                        track_id: *track_id,
                        note_id: *note_id,
                        new_note: note.get_copy(),
                    });

                    *note <<= *new_note;
                } else {
                    panic!("Tried to modify MIDI note that was not found.");
                }
            }
        }

        if let Some(inverse) = inverse {
            self.handle_inverse_action(inverse, type_);
        } else {
            panic!();
        }
    }
}

#[derive(Clone, Copy)]
enum ActionType {
    Normal,
    Undo,
    Redo,
}

pub enum Action {
    Group(Vec<Action>),
    SetSelection(Selection),
    MoveTimeCursor(Time),
    ChangeTempo(f32),
    AddMidiNote {
        track_id: u32,
        note: Note,
    },
    RemoveMidiNote {
        track_id: u32,
        note_id: ReactiveListKey,
    },
    ModifyMidiNote {
        track_id: u32,
        note_id: ReactiveListKey,
        new_note: Note,
    },
}

impl Action {
    fn undoes_automatically(&self) -> bool {
        match self {
            Action::MoveTimeCursor(_) => true,
            Action::SetSelection(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TimeSignature {
    numerator: u32,
    denominator: u32,
}

impl TimeSignature {
    pub fn common() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
        }
    }

    pub fn beats_per_measure(&self) -> u32 {
        // FIXME: This is probably very wrong but it works for now.
        //        I guess this is quarter notes per bar?
        self.numerator / (self.denominator / 4)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct KeySignature {
    pub root: u32,
    pub mode: KeyMode,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KeyMode {
    Major,
    Minor,
}

impl KeySignature {
    pub fn new(root: u32, mode: KeyMode) -> Self {
        Self { root, mode }
    }

    pub fn name(&self) -> String {
        format!("{} {:?}", note_name(self.root as u8, false), self.mode)
    }

    pub fn from_degree(&self, degree: i32, octave: i32) -> u32 {
        let degree: i32 = degree as i32
            - match self.mode {
                KeyMode::Major => 0,
                KeyMode::Minor => 2,
            };


        let mut octave_offset = 0;
        let mut degree = degree;

        while degree < 1 {
            degree += 7;
            octave_offset -= 1;
        }

        octave_offset += ((degree - 1) / 7) as i32;
        degree = (degree - 1) % 7 + 1;

        let mut note = match degree {
            1 => 0,
            2 => 2,
            3 => 4,
            4 => 5,
            5 => 7,
            6 => 9,
            7 => 11,
            _ => panic!("Invalid degree"),
        };

        note += octave_offset * 12;

        let octave = self.root as i32 + octave * 12;
        (note + octave) as u32
    }

    pub fn to_degree(&self, note: i32) -> Option<Degree> {
        let note = note - self.root as i32;

        let octave = note / 12;


        let degree = match note % 12 {
            0 => 1,
            2 => 2,
            4 => 3,
            5 => 4,
            7 => 5,
            9 => 6,
            11 => 7,
            _ => return None,
        };

        Some(Degree { degree, octave })
    }

}

#[derive(Clone, Copy, Debug)]
pub struct Degree {
    pub degree: i32,
    pub octave: i32,
}
