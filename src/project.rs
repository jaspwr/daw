use core::panic;

use crate::{
    midi::{Note, Time},
    track::{self, Track, TrackData, TrackGroup, TrackType},
    ui::{reactive::Reactive, reactive_list::ReactiveListKey},
};

pub struct Project {
    pub meta: ProjectMeta,
    pub tempo: Reactive<f32>,
    pub tracks: TrackGroup,
    pub player_time: Reactive<Time>,
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
        if let Some(action) = self.undo_stack.pop() {
            self.__perform_action(&action, ActionType::Undo);
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            self.__perform_action(&action, ActionType::Redo);
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
            }
        }

        if let Some(inverse) = inverse {
            self.handle_inverse_action(inverse, type_);
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
    ChangeTempo(f32),
    AddMidiNote {
        track_id: u32,
        note: Note,
    },
    RemoveMidiNote {
        track_id: u32,
        note_id: ReactiveListKey,
    },
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
