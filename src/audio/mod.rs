mod audio_file;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use cpal::SupportedStreamConfigRange;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, FromSample, Sample, SizedSample, StreamConfig, SupportedStreamConfig,
};

use tracing::debug;

use self::audio_file::Wav;

pub struct Audio {
    sender: Sender<Action>,
    track_list: Arc<RwLock<Vec<Wav>>>,
}

impl Audio {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let device_name = device.name().unwrap();

        let supported_config: Vec<SupportedStreamConfigRange> = device
            .supported_output_configs()
            .unwrap()
            .into_iter()
            .collect();
        let config = device.default_output_config().unwrap();
        debug!(
            device = device_name,
            configs = format!("{:?}", &supported_config),
            config = format!("{:?}", &config),
            "Output device"
        );

        let wavs = Vec::new();
        let track_list = Arc::new(RwLock::new(wavs));
        let track_list_reader = track_list.clone();
        let (sender, receiver) = mpsc::channel::<Action>();

        thread::spawn(move || {
            let audio_thread = Mixer::new(receiver, track_list_reader, device, config.into());
            audio_thread.run();
        });
        Audio { sender, track_list }
    }

    pub fn add_wav(&mut self, filename: &PathBuf) -> usize {
        let wav = audio_file::Wav::new(filename);
        // TODO: Do this in a non blocking way
        // since reads should only be holding the lock for <1ms and this thread
        // only holds it for a push, waiting for the lock isn't too big of a concern
        let mut track_list = self.track_list.write().unwrap();
        let track_number = track_list.len();
        track_list.push(wav);
        track_number
    }

    pub fn track_action(&self, action: Action) {
        self.sender.send(action).unwrap();
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        self.sender.send(Action::Cleanup).unwrap();
    }
}

struct Mixer {
    device: Device,
    config: SupportedStreamConfig,
    receiver: Receiver<Action>,
    wavs: Arc<RwLock<Vec<Wav>>>,
}

impl Mixer {
    fn new(
        receiver: Receiver<Action>,
        wavs: Arc<RwLock<Vec<Wav>>>,
        device: Device,
        config: SupportedStreamConfig,
    ) -> Self {
        Mixer {
            device,
            config,
            receiver,
            wavs,
        }
    }

    fn run(&self) {
        match self.config.sample_format() {
            cpal::SampleFormat::I8 => self.play::<i8>(),
            cpal::SampleFormat::I16 => self.play::<i16>(),
            cpal::SampleFormat::I32 => self.play::<i32>(),
            cpal::SampleFormat::I64 => self.play::<i64>(),
            cpal::SampleFormat::U8 => self.play::<u8>(),
            cpal::SampleFormat::U16 => self.play::<u16>(),
            cpal::SampleFormat::U32 => self.play::<u32>(),
            cpal::SampleFormat::U64 => self.play::<u64>(),
            cpal::SampleFormat::F32 => self.play::<f32>(),
            cpal::SampleFormat::F64 => self.play::<f64>(),
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        };
    }

    fn play<T>(&self)
    where
        T: SizedSample + FromSample<f32>,
    {
        let config: StreamConfig = self.config.config();
        let sample_rate = config.sample_rate.0 as f64;
        let seconds_per_sample = 1.0 / sample_rate;
        let channels = config.channels as usize;

        let (sender, receiver) = mpsc::channel::<Action>();
        let wavs = self.wavs.clone();
        let mut tracks: HashMap<usize, Track> = HashMap::new();
        let mut next_value =
            move || Self::get_next_audio_value(&wavs, &receiver, &mut tracks, &seconds_per_sample);

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = self
            .device
            .build_output_stream(
                &config.into(),
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut next_value)
                },
                err_fn,
                None,
            )
            .unwrap();

        stream.play().unwrap();

        let mut last_message = self.receiver.recv().unwrap();
        while last_message != Action::Cleanup {
            debug!(
                last_message = format!("{:?}", last_message),
                "last audio message"
            );
            sender.send(last_message).unwrap();
            last_message = self.receiver.recv().unwrap();
        }
    }

    fn get_next_audio_value(
        wavs: &Arc<RwLock<Vec<Wav>>>,
        receiver: &Receiver<Action>,
        tracks: &mut HashMap<usize, Track>,
        seconds_per_sample: &f64,
    ) -> f32 {
        let wav_read = wavs.read().unwrap();
        while let Ok(action) = receiver.try_recv() {
            update_track_state(tracks, &action);
        }
        let mut result = 0.0;
        for i in 0..wav_read.len() {
            if let Some(track) = tracks.get_mut(&i) {
                if track.state == TrackState::Playing || track.state == TrackState::Slow {
                    let samples = &wav_read[i].samples;
                    let track_sample_rate = wav_read[i].sample_rate as f64;
                    let raw_index = track.time * track_sample_rate as f64;
                    let limit = samples.len();
                    let lower_index = raw_index.floor();
                    let upper_index = raw_index.ceil();
                    let dist_to_lower = raw_index - lower_index;
                    if upper_index as usize >= samples.len() {
                        track.state = TrackState::Stopped;
                        track.time = 0.0;
                        continue;
                    }
                    let lower = samples[lower_index as usize % limit];
                    let upper = samples[upper_index as usize % limit];
                    let sample = upper * dist_to_lower + lower * (1.0 - dist_to_lower);
                    result += sample / 32_768.0;

                    let sample_step = if track.state == TrackState::Slow {
                        seconds_per_sample * 0.9
                    } else {
                        *seconds_per_sample
                    };
                    track.time += sample_step;
                }
            }
        }

        // music_wav_second as f32 caused varying rate so use f64 and
        // convert to f32 at the end. If f64 precision still leads to noticeable
        // drift on longer tracks this will need refactoring to not use
        // floats for time calculations
        result as f32
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Play(usize),
    Stop(usize),
    Reset(usize),
    Slow(usize),
    Cleanup,
}

struct Track {
    state: TrackState,
    time: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TrackState {
    Playing,
    Stopped,
    Slow,
}

fn update_track_state(tracks: &mut HashMap<usize, Track>, action: &Action) {
    match action {
        Action::Reset(track) => {
            tracks.insert(
                *track,
                Track {
                    state: TrackState::Stopped,
                    time: 0.0,
                },
            );
        }
        Action::Play(track) => match tracks.get_mut(track) {
            Some(t) => t.state = TrackState::Playing,
            None => {
                tracks.insert(
                    *track,
                    Track {
                        time: 0.0,
                        state: TrackState::Playing,
                    },
                );
            }
        },
        Action::Slow(track) => match tracks.get_mut(track) {
            Some(t) => t.state = TrackState::Slow,
            None => {
                tracks.insert(
                    *track,
                    Track {
                        time: 0.0,
                        state: TrackState::Slow,
                    },
                );
            }
        },
        Action::Stop(track) => match tracks.get_mut(track) {
            Some(t) => t.state = TrackState::Stopped,
            None => {
                tracks.insert(
                    *track,
                    Track {
                        time: 0.0,
                        state: TrackState::Stopped,
                    },
                );
            }
        },
        Action::Cleanup => (),
    }
}
