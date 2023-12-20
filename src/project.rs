use core::panic;

use crate::{
    midi::Time,
    track::{Track, TrackGroup, TrackType},
    ui::reactive::Reactive,
};

pub struct Project {
    pub meta: ProjectMeta,
    pub tempo: Reactive<f32>,
    pub tracks: TrackGroup,
    pub player_time: Reactive<Time>,
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
            player_time: Reactive::new(0),
            undo_stack: vec![],
            redo_stack: vec![],
        };

        project.tracks.add_new(TrackType::Midi);

        return project;
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            self.__perform_action(action, ActionType::Undo);
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            self.__perform_action(action, ActionType::Redo);
        }
    }

    pub fn handle_inverse_action(&mut self, action: Action, type_of_action_just_performed: ActionType) {
        match type_of_action_just_performed {
            ActionType::Normal => {
                self.undo_stack.push(action);
                self.redo_stack.clear();
            },
            ActionType::Undo => self.redo_stack.push(action),
            ActionType::Redo => self.undo_stack.push(action),
        }
    }

    pub fn perform_action(&mut self, action: Action) {
        self.__perform_action(action, ActionType::Normal);
    }

    fn __perform_action(&mut self, action: Action, type_: ActionType) {
        let mut inverse: Option<Action> = None;
        match action {
            Action::ChangeTempo(new_tempo) => {
                inverse = Some(Action::ChangeTempo(self.tempo.get_copy()));
                self.tempo <<= new_tempo;
            }
        }

        if let Some(inverse) = inverse {
            self.handle_inverse_action(inverse, type_);
        } else {
            panic!("Missing inverse action");
        }

    }
}

enum ActionType {
    Normal,
    Undo,
    Redo,
}

pub enum Action {
    ChangeTempo(f32),
}
