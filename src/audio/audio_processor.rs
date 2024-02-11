use std::borrow::{Borrow, BorrowMut};
use std::os::raw::c_void;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Mutex};
use std::{env, fs};

use async_trait::async_trait;
use vst::api::{Event, Events};
use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::host::{Dispatch, Host, HostBuffer, PluginInstance, PluginLoader};
use vst::plugin::Plugin;

use crate::midi::{MidiEvent, Time};

use super::{Buffer, *};

#[async_trait]
pub trait AudioProcessor {
    fn show_gui(&mut self, window_id: *mut c_void) -> Result<(), String> {
        Ok(())
    }
    fn hide_gui(&mut self) {}

    fn process(&mut self, events: Option<&Vec<MidiEvent>>, input: Buffer, t: Time) -> Buffer;

    async fn process_async(
        &mut self,
        events: Option<&Vec<MidiEvent>>,
        input: ThreadSafeBuffer,
        t: Time,
    ) -> ThreadSafeBuffer {
        self.process(events, input.to_buffer(), t).to_thread_safe()
    }

    fn suspend(&mut self) {}
    fn resume(&mut self) {}
    fn change_sample_rate(&mut self, rate: SampleRate);
    fn change_block_size(&mut self, size: BlockSize);
}

pub fn scan_plugin_dir(path: &PathBuf) -> Result<Vec<PluginDescription>, String> {
    let mut plugins = vec![];

    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(plugin) = read_potential_plugin_file(path) {
                    plugins.push(plugin);
                }
            } else if path.is_dir() {
                plugins.extend(
                    scan_plugin_dir(&path)
                        .map_err(|s| s.to_string())?
                        .into_iter(),
                );
            }
        }
    }

    Ok(plugins)
}

fn read_potential_plugin_file(path: PathBuf) -> Option<PluginDescription> {
    let mut file = fs::File::open(&path).ok()?;

    let name = path.file_name()?.to_string_lossy().to_string();

    // TODO: Get plugin metadata

    Some(PluginDescription {
        name,
        path,
        type_: PluginType::Vst2,
        instrument: false,
    })
}

pub struct PluginDescription {
    pub name: String,
    pub path: PathBuf,
    pub type_: PluginType,
    pub instrument: bool,
}

impl PluginDescription {
    pub fn load(&self, a: &Audio) -> Box<dyn AudioProcessor> {
        Box::new(match self.type_ {
            PluginType::Vst2 => load_vst2_plugin(&self.path, &a),
            _ => unimplemented!(),
        })
    }
}

pub enum PluginType {
    Unknown,
    Vst2,
    Vst3,
}

struct Vst2 {
    plugin_instance: PluginInstance,
    state: Vst2State,
    host_buffer: HostBuffer<FrameValue>,
    output: Buffer,
    editor: Option<Box<dyn Editor>>,
}

#[derive(PartialEq, Eq)]
enum Vst2State {
    Suspended,
    Resumed,
}

impl AudioProcessor for Vst2 {
    fn change_sample_rate(&mut self, rate: SampleRate) {
        self.suspend();
        self.plugin_instance.set_sample_rate(rate);
    }

    fn change_block_size(&mut self, size: BlockSize) {
        self.suspend();
        self.plugin_instance.set_block_size(size);
    }

    fn show_gui(&mut self, window_id: *mut c_void) -> Result<(), String> {
        let mut editor = self
            .plugin_instance
            .get_editor()
            .ok_or_else(|| "Plugin has no editor".to_string())?;

        editor.open(window_id as *mut std::ffi::c_void);

        self.editor = Some(editor);

        Ok(())
    }

    fn hide_gui(&mut self) {
        if let Some(editor) = &mut self.editor {
            editor.close();
        }
    }

    fn suspend(&mut self) {
        if self.state == Vst2State::Suspended {
            return;
        }

        self.plugin_instance.suspend();
        self.state = Vst2State::Suspended;
    }

    fn resume(&mut self) {
        if self.state == Vst2State::Resumed {
            return;
        }

        self.plugin_instance.resume();
        self.state = Vst2State::Resumed;
    }

    fn process(&mut self, midi_events: Option<&Vec<MidiEvent>>, input: Buffer, _t: Time) -> Buffer {
        self.resume();
        self.plugin_instance.start_process();

        if let Some(midi_events) = midi_events {
            process_vst2_midi_events_list(&self.plugin_instance, midi_events);
        }

        let inputs_buf = input.data.as_ref().borrow();
        let mut outputs_buf = self.output.data.as_ref().borrow_mut();
        let mut audio_buffer = self.host_buffer.bind(&inputs_buf, &mut outputs_buf);

        self.plugin_instance.process(&mut audio_buffer);

        // self.plugin_instance.stop_process();
        self.output.clone()
    }
}

#[allow(dead_code)]
struct Vst2Host;

impl Host for Vst2Host {
    fn automate(&self, index: i32, value: f32) {
        println!("Parameter {} had its value changed to {}", index, value);
    }
}

fn midi_event_to_vst2_event(midi_event: &MidiEvent) -> *mut Event {
    let mut event = vst::api::MidiEvent {
        event_type: vst::api::EventType::Midi,
        byte_size: 0,
        delta_frames: 0,
        flags: 0,
        note_length: 0,
        note_offset: 0,
        midi_data: [0; 3],
        detune: 0,
        note_off_velocity: 0,
        _reserved1: 0,
        _reserved2: 0,
        _midi_reserved: 0,
    };

    event.midi_data[0] = midi_event.status_byte();

    let note = midi_event.note().unwrap();

    event.midi_data[1] = note.note as u8;
    event.midi_data[2] = note.velocity as u8;

    println!("event.midi_data: {:?}", event.midi_data);

    let event: Event = unsafe { std::mem::transmute(event) };
    let event = Box::into_raw(Box::new(event));

    event
}

#[repr(C)]
pub struct Vst2Events<const L: usize> {
    pub num_events: i32,

    pub _reserved: isize,

    pub events: [*mut Event; L],
}

impl<const L: usize> Vst2Events<L> {
    pub fn new(len: usize) -> Self {
        Self {
            num_events: len as i32,
            _reserved: 0,
            events: [std::ptr::null_mut(); L],
        }
    }
}

fn process_vst2_midi_events_list(plugin_instance: &PluginInstance, midi_events: &Vec<MidiEvent>) {
    let mut events: Vec<*mut Event> = midi_events
        .into_iter()
        .map(|e| midi_event_to_vst2_event(e))
        .collect();

    let num_events = events.len();

    const MAX_EVENTS: usize = 100;

    let mut events_object = Vst2Events::<MAX_EVENTS>::new(num_events);

    if num_events > MAX_EVENTS {
        // Alloc on heap instead
        todo!();
    }

    for (i, event) in events.iter().enumerate() {
        events_object.events[i] = *event;
    }

    plugin_instance.dispatch(
        vst::plugin::OpCode::ProcessEvents,
        0,
        0,
        &events_object as *const _ as *mut _,
        0.0,
    );
}

fn load_vst2_plugin(path: &Path, a: &Audio) -> Vst2 {
    let host = Arc::new(Mutex::new(Vst2Host));

    let mut loader = PluginLoader::load(path, Arc::clone(&host))
        .unwrap_or_else(|e| panic!("Failed to load plugin: {}", e));

    let mut instance = loader.instance().unwrap();

    let info = instance.get_info();

    println!(
        "Loaded '{}':\n\t\
         Vendor: {}\n\t\
         Presets: {}\n\t\
         Parameters: {}\n\t\
         VST ID: {}\n\t\
         Version: {}\n\t\
         Initial Delay: {} samples",
        info.name,
        info.vendor,
        info.presets,
        info.parameters,
        info.unique_id,
        info.version,
        info.initial_delay
    );

    instance.init();

    instance.set_sample_rate(a.sample_rate.get_copy());
    instance.set_block_size(a.block_size.get_copy());

    let output = Buffer::new(2, &a.block_size);

    Vst2 {
        plugin_instance: instance,
        state: Vst2State::Suspended,
        host_buffer: HostBuffer::new(2, 2),
        output,
        editor: None,
    }
}
