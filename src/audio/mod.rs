mod audio_file;

use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, Device, FromSample, Sample, SizedSample, StreamConfig, SupportedStreamConfig
};

use tracing::debug;

use self::audio_file::Wav;

pub struct Audio {
    sender: Sender<i32>,
}

impl Audio {
    pub fn new() -> Self {
        let wav_files = ["assets/sounds/GameSongMono.wav", "assets/sounds/test.wav"];
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let device_name = device.name().unwrap();

        let supported_config: Vec<_> = device.supported_output_configs().unwrap().into_iter().collect();
        let config = device.default_output_config().unwrap();
        debug!(device = device_name, configs = format!("{:?}", &supported_config), config = format!("{:?}", &config), "Output device");

        let mut wavs: Vec<Wav> = Vec::with_capacity(wav_files.len());
        for wav_file in wav_files {
            let wav = audio_file::Wav::new(wav_file.into());
            wavs.push(wav);
        }
        let (sender, receiver) = mpsc::channel();
        sender.send(0).unwrap();

        thread::spawn(move || {
            let audio_thread = Mixer::new(
                receiver,
                wavs,
            device,
            config.into(),
        );
            audio_thread.run();
        });
        Audio {
            sender,
        }
    }

    pub fn collided(&self) {
        self.sender.send(1).unwrap();
    }

    pub fn reset(&self) {
        self.sender.send(2).unwrap();
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        self.sender.send(3).unwrap();
    }
}

struct Mixer {
    device: Device,
    config: SupportedStreamConfig,
    receiver: Receiver<i32>,
    wavs: Vec<Wav>,
}

impl Mixer {
    fn new(receiver: Receiver<i32>, wavs: Vec<Wav>, device: Device, config: SupportedStreamConfig) -> Self {
        Mixer {
            device,
            config,
            receiver,
            wavs,
        }
    }

    fn run(&self) {
        match self.config.sample_format() {
            cpal::SampleFormat::I8 => self.play_sin::<i8>(),
            cpal::SampleFormat::I16 => self.play_sin::<i16>(),
            cpal::SampleFormat::I32 => self.play_sin::<i32>(),
            cpal::SampleFormat::I64 => self.play_sin::<i64>(),
            cpal::SampleFormat::U8 => self.play_sin::<u8>(),
            cpal::SampleFormat::U16 => self.play_sin::<u16>(),
            cpal::SampleFormat::U32 => self.play_sin::<u32>(),
            cpal::SampleFormat::U64 => self.play_sin::<u64>(),
            cpal::SampleFormat::F32 => self.play_sin::<f32>(),
            cpal::SampleFormat::F64 => self.play_sin::<f64>(),
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        };
    }

    fn play_sin<T>(&self)
    where
        T: SizedSample + FromSample<f32>,
    {
        let config: StreamConfig = self.config.config();
        let sample_rate = config.sample_rate.0 as f64;
        let seconds_per_sample = 1.0 / sample_rate;
        let channels = config.channels as usize;

        let (sender, receiver) = mpsc::channel();
        sender.send(0).unwrap();
        let mut last = 0;
        let mut death_wav_position = 0;
        let mut music_wav_second = 0.0;
        let death_wav = self.wavs[1].samples.clone();
        let music_wav = self.wavs[0].samples.clone();
        let music_sample_rate = self.wavs[0].sample_rate as f64;
        debug!(wav = format!("{:?}", &death_wav[0..32]), "play");
        let mut next_value = move || {
            let mut result = 0.0;
            last = receiver.try_recv().unwrap_or(last);
            if last == 2 {
                music_wav_second = 0.0;
                last = 0;
            }

            if last == 1 && death_wav_position < death_wav.len() {
                let sample = death_wav[death_wav_position];
                result += sample / 32_768.0;
                death_wav_position += 1;
            } else if last != 1 {
                death_wav_position = 0;
            }

            let raw_index = music_wav_second * music_sample_rate;
            let limit = music_wav.len();
            let lower_index = raw_index.floor();
            let upper_index = raw_index.ceil();
            let dist_to_lower = raw_index - lower_index;
            let lower = music_wav[lower_index as usize % limit];
            let upper = music_wav[upper_index as usize % limit];
            let music_sample = upper * dist_to_lower + lower * (1.0 - dist_to_lower);
            let music_wav_step = if last == 1 {
                seconds_per_sample * 0.9
            } else {
                seconds_per_sample
            };
            music_wav_second += music_wav_step;
            result += music_sample / 32_768.0;

            // music_wav_second as f32 caused varying rate so use f64 and
            // convert to f32 at the end
            result as f32
        };
    
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    
        let stream = self.device.build_output_stream(
            &config.into(),
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            err_fn,
            None,
        ).unwrap();

        stream.play().unwrap();
    
        let mut last_message = self.receiver.recv().unwrap();
        while last_message != 3 {
            debug!(last_message = last_message, "last audio message");
            if last_message == 1 {
                sender.send(1).unwrap();
            }
            if last_message == 2 {
                sender.send(2).unwrap();
            }
            last_message = self.receiver.recv().unwrap();
        }
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