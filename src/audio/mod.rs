use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use anyhow::Result;
use cpal::SupportedStreamConfigRange;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, FromSample, Sample, SizedSample, StreamConfig, SupportedStreamConfig,
};

use tracing::{debug, error};

use crate::resource::manager::ResourceManager;

use super::resource::audio::Wav;

pub struct Audio {
    sender: Sender<Action>,
    wavs: Arc<RwLock<HashMap<String, Wav>>>,
    resource_manager: Arc<ResourceManager>,
    resource_rec: Receiver<(String, Result<Wav>)>,
    resource_send: Sender<(String, Result<Wav>)>,
    loading_files: HashSet<String>,
    loaded_files: HashSet<String>,
}

impl Audio {
    pub fn new(resource_manager: Arc<ResourceManager>) -> Self {
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
        let (sender, receiver) = mpsc::channel::<Action>();
        let wavs = Arc::new(RwLock::new(HashMap::new()));
        let audio_thread_wavs = wavs.clone();

        thread::spawn(move || {
            let mut audio_thread = Mixer::new(
                receiver,
                device,
                config.into(),
                audio_thread_wavs,
            );
            audio_thread.run();
        });

        let (resource_send, resource_rec) = mpsc::channel::<(String, Result<Wav>)>();
        let loading_files = std::collections::HashSet::new();
        let loaded_files = std::collections::HashSet::new();

        Audio {
            sender,
            wavs,
            resource_manager,
            resource_rec,
            resource_send,
            loading_files,
            loaded_files,
        }
    }

    pub fn load_wavs(&mut self, wavs: &[String]) {
        debug!("Load Wavs");
        for wav in wavs {
            self.resource_manager.load_wav(wav.clone(), self.resource_send.clone());
            self.loading_files.insert(wav.clone());
        }
    }

    // TODO: Bit flags would probably make more sense than a Vec, think there's
    // a lib for this with nice syntax.
    pub fn update(&mut self) -> Vec<AudioEvent> {
        let mut events = Vec::new();
        if !self.loading_files.is_empty() {
            let mut new_wavs = Vec::with_capacity(self.loading_files.len());
            while let Ok((file, res)) = self.resource_rec.try_recv() {
                debug!(file = file, "Wav loaded Rec");
                self.loading_files.remove(&file);
                match res {
                    Ok(w) => {
                        new_wavs.push((file, w));
                    },
                    Err(e) => error!(err=e.to_string(), "Failed to load wav"),
                }
            }
            if new_wavs.len() != 0 {
                let mut wavs_lock = self.wavs.write().unwrap();
                for (f, w) in new_wavs {
                    wavs_lock.insert(f.clone(), w);
                    self.loaded_files.insert(f);
                }
                if self.loading_files.is_empty() {
                    let keys: Vec<&String> = wavs_lock.keys().collect();
                    debug!(wavs = format!("{:?}", keys), "Loaded All Wavs");
                    events.push(AudioEvent::Loaded);
                }
            }
        }
        events
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
    wavs: Arc<RwLock<HashMap<String, Wav>>>,
}

impl Mixer {
    fn new(
        receiver: Receiver<Action>,
        device: Device,
        config: SupportedStreamConfig,
        wavs: Arc<RwLock<HashMap<String, Wav>>>,
    ) -> Self {
        Mixer {
            device,
            config,
            receiver,
            wavs,
        }
    }

    fn run(&mut self) {
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

    fn play<T>(&mut self)
    where
        T: SizedSample + FromSample<f32>,
    {
        let config: StreamConfig = self.config.config();
        let sample_rate = config.sample_rate.0 as f64;
        let seconds_per_sample = 1.0 / sample_rate;
        let channels = config.channels as usize;

        let (sender, receiver) = mpsc::channel::<Action>();
        let wavs = self.wavs.clone();
        let mut tracks: HashMap<String, Track> = HashMap::new();
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
        wavs: &Arc<RwLock<HashMap<String, Wav>>>,
        receiver: &Receiver<Action>,
        tracks: &mut HashMap<String, Track>,
        seconds_per_sample: &f64,
    ) -> f32 {
        while let Ok(action) = receiver.try_recv() {
            update_track_state(tracks, action);
        }
        let mut result = 0.0;
        for (track_name, track) in tracks {
            if track.state == TrackState::Playing || track.state == TrackState::Slow {
                let w = wavs.read().unwrap();
                let Some(wav) = w.get(track_name) else {
                    error!(track=track_name, "Failed to get track audio");
                    continue;
                };
                let samples = &wav.samples;
                let track_sample_rate = wav.sample_rate as f64;
                let raw_index = track.time * track_sample_rate as f64;
                let index = raw_index as usize;
                if index >= samples.len() {
                    track.state = TrackState::Stopped;
                    track.time = 0.0;
                    continue;
                }
                let sample = samples[index];
                result += sample / 32_768.0;

                let sample_step = if track.state == TrackState::Slow {
                    seconds_per_sample * 0.9
                } else {
                    *seconds_per_sample
                };
                track.time += sample_step;
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

#[derive(Debug, PartialEq)]
pub enum AudioEvent {
    Loaded,
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Play(String),
    Stop(String),
    Reset(String),
    Slow(String),
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

fn update_track_state(tracks: &mut HashMap<String, Track>, action: Action) {
    match action {
        Action::Reset(track) => {
            tracks.insert(
                track,
                Track {
                    state: TrackState::Stopped,
                    time: 0.0,
                },
            );
        }
        Action::Play(track) => match tracks.get_mut(track.as_str()) {
            Some(t) => t.state = TrackState::Playing,
            None => {
                tracks.insert(
                    track,
                    Track {
                        time: 0.0,
                        state: TrackState::Playing,
                    },
                );
            }
        },
        Action::Slow(track) => match tracks.get_mut(track.as_str()) {
            Some(t) => t.state = TrackState::Slow,
            None => {
                tracks.insert(
                    track,
                    Track {
                        time: 0.0,
                        state: TrackState::Slow,
                    },
                );
            }
        },
        Action::Stop(track) => match tracks.get_mut(track.as_str()) {
            Some(t) => t.state = TrackState::Stopped,
            None => {
                tracks.insert(
                    track,
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
