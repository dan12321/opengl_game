use std::sync::mpsc::Receiver;
use std::path::PathBuf;
use std::thread;
use std::sync::mpsc::{self, Sender};

use anyhow::Result;
use tracing::debug;

use super::audio::Wav;
use super::map::Map;

#[derive(Debug)]
pub struct ResourceManager {
    req_sender: Sender<DataRequest>,
    // shutdown_sender: Sender<()>,
    // io_thread: JoinHandle<()>,
}

impl ResourceManager {
    pub fn new() -> Self {
        let (req_sender, req_receiver) = mpsc::channel::<DataRequest>();
        let (_shutdown_sender, shutdown_receiver) = mpsc::channel::<()>();
        let _io_thread = thread::spawn(move || {
            run_io(req_receiver, shutdown_receiver);
        });

        Self {
            req_sender,
            // shutdown_sender,
            // io_thread,
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

const AUDIO_LOCATION: &'static str = "assets/sounds";
const MAP_LOCATION: &'static str = "assets/maps";