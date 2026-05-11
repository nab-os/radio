use cpal::{
    Device, OutputCallbackInfo, Sample, SampleFormat, SampleRate, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use radio_core::data::track::Track;

#[derive(Clone)]
pub struct Player {
    device: Device,
    config: StreamConfig,
    pub track: Track,
    pub progress: Arc<AtomicU32>, //can store f32
}

impl Player {
    pub fn new(track: Track) -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find output device");
        println!("Output device: {}", device.description().unwrap());

        let config = device
            .supported_output_configs()
            .expect("could not get supported output configs")
            .find(|c| {
                c.sample_format() == SampleFormat::F32
                    && c.channels() == track.tech.channel_count as u16
            })
            .expect("could not get an F32 output config and the right number of channels")
            .try_with_sample_rate(SampleRate::from(track.tech.sample_rate))
            .expect("could not get sample rate")
            .config();
        println!("Output config: {config:?}");

        Player {
            device,
            config,
            track,
            progress: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn play(self: &Self, running: Arc<AtomicBool>) -> JoinHandle<()> {
        let err_fn = |err| eprintln!("stream error: {err}");

        let device = self.device.clone();
        let config = self.config.clone();
        let track = self.track.clone();
        let progress = self.progress.clone();

        thread::spawn(move || {
            let mut samples = track.data.samples.clone();
            let stream = device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &OutputCallbackInfo| write_data(data, &mut samples),
                    err_fn,
                    None,
                )
                .expect("couldn't build output stream");

            let player_start_time = SystemTime::now();
            stream.play().expect("couldn't play");

            while running.load(Ordering::Relaxed) {
                let curr_time = SystemTime::now();
                let time_spent = curr_time
                    .duration_since(player_start_time)
                    .expect("Error in time");

                let p = (100.0 * time_spent.as_secs_f64() / track.tech.duration).min(100.0) as f32;
                progress.store(p.to_bits(), Ordering::Relaxed);

                if p >= 100.0 {
                    break;
                }

                thread::sleep(Duration::from_millis(200));
            }
        })
    }

    pub fn get_progress(&self) -> f32 {
        f32::from_bits(self.progress.load(Ordering::Relaxed))
    }
}

fn write_data(output: &mut [f32], samples: &mut VecDeque<f32>) {
    for sample in output.iter_mut() {
        *sample = samples.pop_front().unwrap_or(f32::EQUILIBRIUM);
    }
}
