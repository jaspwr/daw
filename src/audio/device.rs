use sdl2::{
    audio::{AudioCallback, AudioDevice},
    Sdl,
};

use super::{Audio, BlockSize};

pub trait Device {
    fn get_name(&self) -> String;
    fn open(&mut self) {}
    fn close(&mut self) {}
}

const SDL_CHANNELS: u8 = 2;

impl AudioCallback for SDLAudioDeviceCallback {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // println!("{}", out.len());

        // for (channel_idx, channel) in out.chunks_mut((self.block_size / SDL_CHANNELS as i64) as usize).enumerate() {
        //     // if channel_idx == 0 {
        //     //     continue;
        //     // }

        //     println!("{}", channel_idx);

        //     for (i, sample) in channel.iter_mut().enumerate() {
        //         *sample = (i as f32 / 44100.0 * 440.0 * 2.0 * 3.141592).sin();
        //     }
        // }

        // self.device.process(out);
    }
}

pub struct SDLAudioDeviceCallback {
    // pub device: Box<dyn Device>,
    pub block_size: BlockSize,
}

pub struct SDLAudioDevice {
    pub device: Box<AudioDevice<SDLAudioDeviceCallback>>,
}

impl SDLAudioDevice {
    pub fn new(sdl_context: &Sdl, a: &Audio) -> Self {
        let audio_subsystem = sdl_context.audio().unwrap();

        let block_size = a.block_size.get_copy();
        let block_size = 1024;

        let desired_spec = sdl2::audio::AudioSpecDesired {
            freq: Some(a.sample_rate.get_copy() as i32),
            channels: Some(SDL_CHANNELS),
            samples: Some(block_size as u16),
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                println!("spec: {:?}", spec);
                SDLAudioDeviceCallback {
                    block_size,
                    // device: Box::new(device),
                }
            })
            .unwrap();

        device.resume();

        Self {
            device: Box::new(device),
        }
    }
}

impl Device for SDLAudioDevice {
    fn get_name(&self) -> String {
        "SDL".to_string()
    }

    fn open(&mut self) {
        self.device.resume();
    }
}
