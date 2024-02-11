use std::{rc::Rc, sync::{atomic::AtomicU64, Arc, Mutex}};

use crate::{
    midi::{self, Time},
    ui::reactive::Reactive,
    utils::{free, rc_ref_cell, RcRefCell},
};

use self::audio_processor::AudioProcessor;

pub mod audio_processor;
pub mod device;

pub type SampleRate = f32;
pub type BlockSize = i64;
pub type FrameValue = f32;

pub struct Audio {
    pub device: Option<Box<dyn device::Device>>,
    pub output_processor: Option<Box<dyn AudioProcessor>>,
    pub sample_rate: Reactive<SampleRate>,
    pub block_size: Reactive<BlockSize>,
    pub engine_output_buf: Buffer,
}

impl Default for Audio {
    fn default() -> Self {
        Self {
            device: None,
            output_processor: None,
            sample_rate: Reactive::new(44100.0),
            block_size: Reactive::new(512),
            engine_output_buf: Buffer::new(2, &Reactive::new(512)),
        }
    }
}

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Buffer {
    pub uid: u64,
    count: *mut usize,
    pub data: RcRefCell<Vec<Vec<FrameValue>>>,
    pub on_drop: Rc<dyn Fn()>,
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        unsafe {
            *self.count += 1;
        }

        Self {
            uid: self.uid,
            count: self.count,
            data: self.data.clone(),
            on_drop: self.on_drop.clone(),
        }
    }
}

impl Buffer {
    fn new(num_channels: usize, num_frames: &Reactive<BlockSize>) -> Self {
        let uid = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let num_frames = num_frames.clone();

        let data = rc_ref_cell(vec![
            vec![0.0; num_frames.get_copy() as usize];
            num_channels
        ]);

        let sub = {
            let data = data.clone();
            num_frames.subscribe(Box::new(move |new_num_frames| {
                data.swap(&rc_ref_cell(vec![
                    vec![0.0; *new_num_frames as usize];
                    num_channels
                ]));
            }))
        };

        let clean_up = {
            let num_frames = num_frames.clone();
            move || {
                num_frames.unsubscribe(sub);
            }
        };

        Self {
            uid,
            count: Box::into_raw(Box::new(1)),
            data,
            on_drop: Rc::new(clean_up),
        }
    }

    fn new_non_reactive(num_channels: usize, num_frames: usize) -> Self {
        let uid = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        Self {
            uid,
            count: Box::into_raw(Box::new(1)),
            data: rc_ref_cell(vec![vec![0.0; num_frames]; num_channels]),
            on_drop: Rc::new(|| ()),
        }
    }

    pub fn to_thread_safe(&self) -> ThreadSafeBuffer {
        ThreadSafeBuffer {
            data: self.data.borrow().clone(),
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let count = unsafe {
            *self.count -= 1;
            *self.count
        };

        if count == 0 {
            unsafe { free(self.count) };

            (*self.on_drop)();
        }
    }
}

#[derive(Clone)]
pub struct ThreadSafeBuffer {
    pub data: Vec<Vec<FrameValue>>,
}

impl ThreadSafeBuffer {
    fn new(num_channels: usize, num_frames: usize) -> Self {
        Self {
            data: vec![vec![0.0; num_frames]; num_channels],
        }
    }

    pub fn to_buffer(&self) -> Buffer {
        let buf = Buffer::new_non_reactive(self.data.len(), self.data[0].len());
        buf.data.swap(&rc_ref_cell(self.data.clone()));
        buf
    }
}


pub struct ProcessorGroup {
    pub uid: u64,
    pub processors: Vec<Box<dyn AudioProcessor>>,
    pub output: Buffer,
}

impl ProcessorGroup {
    fn new() -> Self {
        let uid = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self {
            uid,
            processors: vec![],
            output: Buffer::new_non_reactive(2, 512)
        }
    }

    fn add_processor(&mut self, mut new_processor: Box<dyn AudioProcessor>, a: &Audio) {
        self.processors.push(new_processor);
    }
}

impl AudioProcessor for ProcessorGroup {
    fn change_sample_rate(&mut self, rate: SampleRate) {
        for processor in &mut self.processors {
            processor.change_sample_rate(rate);
        }
    }

    fn change_block_size(&mut self, size: BlockSize) {
        for processor in &mut self.processors {
            processor.change_block_size(size);
        }
    }

    fn show_gui(&mut self, _window_id: *mut std::ffi::c_void) -> Result<(), String> {
        Ok(())
    }


    fn hide_gui(&mut self) {}

    fn process(&mut self, _midi_events: std::option::Option<&Vec<midi::MidiEvent>>, mut input: Buffer, t: Time) -> Buffer {
        for processor in &mut self.processors {
            input = processor.process(None, input, t);
        }
        input.clone()
    }

}

// pub struct Memo {
//     last_t: Time,
//     output: Buffer,
// }


// // Note: Tracks count as sends...
// pub struct Send {
//     inputs: AddNode,
//     processors: ProcessorGroup,
//     memo: Memo,
// }

// impl Send {
//     async fn process(&mut self, t: Time) -> ThreadSafeBuffer {

//         if self.memo.last_t == t {
//             return self.memo.output.to_thread_safe();
//         }

//         let output = self.processors.process(events, input.to_buffer(), t);

//         self.memo.output = output.clone();
//         self.memo.last_t = t;

//         output.to_thread_safe()
//     }
// }

// pub struct AddNode(Option<Vec<Arc<Mutex<Send>>>>);

// impl AddNode {
//     fn process(rt: &mut Runtime, input: ThreadSafeBuffer, t: Time) -> ThreadSafeBuffer {


//         output
//     }
// }