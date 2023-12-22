pub fn scan_plugin_dir() -> Vec<PluginDescription> {
    vec![]
}

pub fn load_plugin(path: &str) -> AudioProcessor {
    AudioProcessor {}

}

pub struct PluginDescription {
    pub name: String,
    pub path: String,
    pub instrument: bool,
}

pub struct AudioProcessor {

}