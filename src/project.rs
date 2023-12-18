use crate::ui::reactive::Reactive;

pub struct Project {
    pub name: String,
    pub description: String,
    pub version: String,
    pub tempo: Reactive<f32>,
}

impl Project {
    pub fn new() -> Self {
        Project {
            name: "Untitled".to_string(),
            description: "No description".to_string(),
            version: "0.0.1".to_string(),
            tempo: Reactive::new(120.),
        }
    }
}