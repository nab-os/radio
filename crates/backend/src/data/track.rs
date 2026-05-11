use radio_core::data::track::{self, Track};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::{Hint, ProbeResult};
use uuid::Uuid;

pub fn load_track(path: PathBuf) -> JoinHandle<(Uuid, Track)> {
    let handle = thread::spawn(move || -> _ {
        let src = std::fs::File::open(path.clone()).expect("failed to open media");

        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let hint = Hint::new();

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let mut probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .expect("unsupported format");

        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks");

        let dec_opts: DecoderOptions = Default::default();

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .expect("unsupported codec");

        let track_id = track.id;

        let sample_rate = track
            .codec_params
            .sample_rate
            .expect("could not get sample rate");
        let channel_count = match track.codec_params.channels {
            Some(channels) => channels.count(),
            None => 1,
        };

        let mut total_frames = 0;
        let mut samples: VecDeque<f32> = VecDeque::new();

        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(Error::ResetRequired) => {
                    unimplemented!();
                }
                Err(_) => {
                    break;
                }
            };

            while !format.metadata().is_latest() {
                format.metadata().pop();
            }

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    let spec = *audio_buf.spec();
                    let frames = audio_buf.frames() as u64;
                    total_frames += frames;
                    let mut sample_buf = SampleBuffer::new(frames, spec);
                    sample_buf.copy_interleaved_ref(audio_buf);

                    samples.extend(sample_buf.samples());
                }
                Err(Error::IoError(_)) => {
                    continue;
                }
                Err(Error::DecodeError(_)) => {
                    continue;
                }
                Err(_) => {
                    break;
                }
            };
        }

        probed.format = format;

        let duration = total_frames as f64 / sample_rate as f64;
        let info = load_track_info(path);
        let tech = track::Tech {
            channel_count,
            sample_rate,
            duration,
        };
        let data = track::Data { samples };
        (Uuid::new_v4(), Track { info, tech, data })
    });

    handle
}

pub fn load_track_info(path: PathBuf) -> track::Info {
    let src = std::fs::File::open(path.clone()).expect("failed to open media");

    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let hint = Hint::new();

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let mut probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");

    let title = extract_tag(&mut probed, StandardTagKey::TrackTitle)
        .unwrap_or("Title not found".to_string());
    let album =
        extract_tag(&mut probed, StandardTagKey::Album).unwrap_or("Album not found".to_string());
    let artist =
        extract_tag(&mut probed, StandardTagKey::Artist).unwrap_or("Artist not found".to_string());

    let cover = extract_cover(&mut probed);

    track::Info {
        path,
        title,
        album,
        artist,
        cover,
    }
}

fn extract_tag(probed: &mut ProbeResult, tag_key: StandardTagKey) -> Option<String> {
    let mut value: Option<String> = None;

    if let Some(metadata) = probed.format.metadata().current() {
        for tag in metadata.tags() {
            if tag.std_key == Some(tag_key) {
                value = Some(tag.value.to_string());
                break;
            }
        }
    }

    if value.is_none() {
        if let Some(metadata) = probed.metadata.get() {
            if let Some(rev) = metadata.current() {
                for tag in rev.tags() {
                    if tag.std_key == Some(tag_key) {
                        value = Some(tag.value.to_string());
                        break;
                    }
                }
            }
        }
    }

    value
}

fn extract_cover(probed: &mut ProbeResult) -> Option<Vec<u8>> {
    let mut cover: Option<Vec<u8>> = None;

    if let Some(metadata) = probed.format.metadata().current() {
        if let Some(visual) = metadata.visuals().first() {
            cover = Some(visual.data.to_vec());
        }
    }
    if let Some(metadata) = probed.metadata.get() {
        if let Some(rev) = metadata.current() {
            if let Some(visual) = rev.visuals().first() {
                cover = Some(visual.data.to_vec());
            }
        }
    }

    cover
}
