use crate::{ui::reactive::Reactive, track::{Track, TrackType, TrackGroup}};

pub struct Project {
    pub meta: ProjectMeta,
    pub tempo: Reactive<f32>,
    pub tracks: TrackGroup,
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
        };

        project.tracks.add_new(TrackType::Midi);

        return project;
    }
}
