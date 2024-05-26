mod audio_file;

use std::cell::RefCell;
use std::f32::consts::PI;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, RwLock};
use std::time::Duration;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, Device, FromSample, Sample, SizedSample, StreamConfig, SupportedStreamConfig
};

use tracing::debug;

use self::audio_file::Wav;

pub struct Audio {
    sender: Sender<i32>,
    th: JoinHandle<()>,
}

impl Audio {
    pub fn new() -> Self {
        let wav = audio_file::Wav::new("assets/sounds/test.wav".into());
        let (sender, receiver) = mpsc::channel();
        sender.send(0).unwrap();

        let child = thread::spawn(move || {
            let audio_thread = Mixer::new(receiver, wav);
            audio_thread.run();
        });
        Audio {
            sender,
            th: child,
        }
    }

    pub fn collision_effect(&self) {
        self.sender.send(1).unwrap();
        self.sender.send(0).unwrap();
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        self.sender.send(2).unwrap();
    }
}

struct Mixer {
    device: Device,
    config: SupportedStreamConfig,
    receiver: Receiver<i32>,
    wav: Wav,
}

impl Mixer {
    fn new(receiver: Receiver<i32>, wav: Wav) -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let device_name = device.name().unwrap();

        let config = device.default_output_config().unwrap();
        debug!(device = device_name, config = format!("{:?}", &config), "Output device");
    
        Mixer {
            device,
            config: config.into(),
            receiver,
            wav,
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
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;
    
        // Produce a sinusoid of maximum amplitude.
        let mut sample_clock = 0f32;
        let (sender, receiver) = mpsc::channel();
        sender.send(0).unwrap();
        let mut last = 0;
        let mut wav_position = 0;
        let wav = self.wav.samples.clone();
        debug!(wav = format!("{:?}", &wav[0..32]), "play");
        let mut next_value = move || {
            if receiver.try_recv().unwrap_or(last) == 1 {
                last = 1;
                let sample = wav[wav_position];
                wav_position += 1;
                if wav_position >= wav.len() {
                    last = 0;
                    wav_position = 0;
                }
                sample / 32_768.0
                //sample_clock = (sample_clock + 1.0) % sample_rate;
                //(sample_clock * 440.0 * 2.0 * PI / sample_rate).sin()
            } else {
                last = 0;
                wav_position = 0;
                0.0
            }
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
        while last_message != 2 {
            debug!(last_message = last_message, "last audio message");
            if last_message == 1 {
                sender.send(1).unwrap();
                thread::sleep(Duration::from_secs(2));
                sender.send(0).unwrap();
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