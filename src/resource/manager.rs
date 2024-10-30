use std::{error::Error, sync::mpsc::Receiver};
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Read;
use std::num::ParseFloatError;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Sender};

use anyhow::{Result, Context};
use tracing::debug;

use super::audio::Wav;

pub struct ResourceManager {
    req_sender: Sender<DataRequest>,
    shutdown_sender: Sender<()>,
    io_thread: JoinHandle<()>,
}

impl ResourceManager {
    pub fn new() -> Self {
        let (req_sender, req_receiver) = mpsc::channel::<DataRequest>();
        let (shutdown_sender, shutdown_receiver) = mpsc::channel::<()>();
        let io_thread = thread::spawn(move || {
            run_io(req_receiver, shutdown_receiver);
        });

        Self {
            req_sender,
            shutdown_sender,
            io_thread,
        }
    }

    // Note: might be a spot for generics, just trait "from_file". Maybe proc_macro
    // if that doesn't work due to needing an enum type for the channel.
    pub fn load_wav(&self, file: String, callback_sender: Sender<(String, Result<Wav>)>) {
        self.req_sender.send(DataRequest::Wav(file, callback_sender)).unwrap();
    }

    pub fn load_map(&self, file: String, callback_sender: Sender<(String, Result<Map>)>) {
        self.req_sender.send(DataRequest::Map(file, callback_sender)).unwrap();
    }
}

fn run_io(rec: Receiver<DataRequest>, shutdown: Receiver<()>) {
    loop {
        if shutdown.try_recv().is_ok() {
            break;
        }
        let Some(req) = rec.try_recv().ok() else {
            continue;
        };

        match req {
            DataRequest::Wav(s, send) => {
                let dir = PathBuf::from(AUDIO_LOCATION);
                let path = dir.join(&s);
                debug!(path = format!("{:?}", &path), "Loading Wav");
                let wav = Wav::new(&path);
                debug!(path = format!("{:?}", &path), "Loaded Wav");
                send.send((s, wav)).unwrap();
            },
            DataRequest::Map(s, send) => {
                let dir = PathBuf::from(MAP_LOCATION);
                let path = dir.join(&s);
                let map = Map::from_file(&path);
                send.send((s, map)).unwrap();
            }
        };
    }
}

enum DataRequest {
    Wav(String, Sender<(String, Result<Wav>)>),
    Map(String, Sender<(String, Result<Map>)>),
}

#[derive(Debug)]
pub struct Map {
    pub version: u64,
    pub bpm: f32,
    pub subdivisions: f32,
    pub start_offset: f32,
    pub music: String,
    pub beats: Vec<(bool, bool, bool)>,
}

impl Map {
    fn from_file(filepath: &PathBuf) -> Result<Self> {
        let mut file = OpenOptions::new().read(true).open(filepath)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let mut lines = buf.lines().filter(|l| !l.starts_with("//"));
        let version: u64 = lines.next()
            .ok_or(MapError::MissingVersion)?
            .trim()
            .parse()
            .context("Parsing Map Version")?;
        let metadata = lines.next().ok_or(MapError::MissingMetadata)?;
        let metadata_parts: Result<Vec<f32>, ParseFloatError> = metadata.split(",")
            .map(|s| s.parse())
            .collect();
        let metadata_parts = metadata_parts.context("Parsing Map Metadata")?;
        if metadata_parts.len() != 3 {
            return Err(MapError::BadMetadata(metadata_parts.len()).into());
        }
        let music = lines.next().ok_or(MapError::MissingSongData)?;
        let beats: Vec<(bool, bool, bool)> = lines.map(|s| parse_map_line(s)).collect();

        Ok(Map {
            version,
            bpm: metadata_parts[0],
            subdivisions: metadata_parts[1],
            start_offset: metadata_parts[2],
            music: music.to_string(),
            beats,
        })
    }
}

fn parse_map_line(s: &str) -> (bool, bool, bool) {
    let mut result = [false; 3];
    for i in 0..3 {
        result[i] = s[i..i + 1] == *"#";
    }
    (result[0], result[1], result[2])
}

#[derive(Debug)]
enum MapError {
    MissingVersion,
    MissingMetadata,
    MissingSongData,
    BadMetadata(usize),
}

impl Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingVersion => write!(f, "Missing Version"),
            Self::MissingMetadata => write!(f, "Missing Metadata"),
            Self::MissingSongData => write!(f, "Missing Song Data"),
            Self::BadMetadata(n) => write!(f, "Found {} metadata arguments", n)
        }
    }
}

impl Error for MapError {}

const AUDIO_LOCATION: &'static str = "assets/sounds";
const MAP_LOCATION: &'static str = "assets/maps";