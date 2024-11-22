use std::sync::mpsc::Receiver;
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};

use anyhow::Result;

use super::audio::Wav;
use super::map::Map;
use super::model::{Material, Model, Texture};

#[derive(Debug)]
pub struct ResourceManager {
    req_sender: Sender<DataReq>,
    shutdown_sender: Sender<()>,
    io_thread: Option<JoinHandle<()>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        let (req_sender, req_receiver) = mpsc::channel::<DataReq>();
        let (shutdown_sender, shutdown_receiver) = mpsc::channel::<()>();
        let io_thread = thread::spawn(move || {
            run_io(req_receiver, shutdown_receiver);
        });

        Self {
            req_sender,
            shutdown_sender,
            io_thread: Some(io_thread),
        }
    }

    // Note: might be a spot for generics, just trait "from_file". Maybe proc_macro
    // if that doesn't work due to needing an enum type for the channel.
    pub fn load_wav(&self, file: String, callback_sender: DataResSender<Wav>) {
        self.req_sender
            .send(DataReq::Wav((file, callback_sender)))
            .unwrap();
    }

    pub fn load_map(&self, file: String, callback_sender: DataResSender<Map>) {
        self.req_sender
            .send(DataReq::Map((file, callback_sender)))
            .unwrap();
    }

    pub fn load_model(&self, file: String, callback_sender: DataResSender<Model>) {
        self.req_sender
            .send(DataReq::Model((file, callback_sender)))
            .unwrap();
    }

    pub fn load_material(&self, file: String, callback_sender: DataResSender<Vec<Material>>) {
        self.req_sender
            .send(DataReq::Material((file, callback_sender)))
            .unwrap();
    }

    pub fn load_texture(&self, file: String, callback_sender: DataResSender<Texture>) {
        self.req_sender
            .send(DataReq::Texture((file, callback_sender)))
            .unwrap();
    }

    pub fn cleanup(&self) {
        // TODO: wait for join. Because this is behind an Arc we can't take ownership
        // of self here. Adding in drop would work but it would be good to keep the
        // shutdown order explicit with resource manager at the end
        self.shutdown_sender.send(());
    }
}

fn run_io(rec: Receiver<DataReq>, shutdown: Receiver<()>) {
    loop {
        if shutdown.try_recv().is_ok() {
            break;
        }
        let Some(req) = rec.try_recv().ok() else {
            continue;
        };

        match req {
            DataReq::Wav((s, send)) => {
                load::<Wav>(AUDIO_LOCATION, s, send);
            }
            DataReq::Map((s, send)) => {
                load::<Map>(MAP_LOCATION, s, send);
            }
            DataReq::Model((s, send)) => {
                load::<Model>(MODEL_LOCATION, s, send);
            }
            DataReq::Material((s, send)) => {
                load::<Material>("", s, send);
            }
            DataReq::Texture((s, send)) => {
                load::<Texture>("", s, send);
            }
        };
    }
}

enum DataReq {
    Wav(DataReqBody<Wav>),
    Map(DataReqBody<Map>),
    Model(DataReqBody<Model>),
    Material(DataReqBody<Vec<Material>>),
    Texture(DataReqBody<Texture>),
}

pub type DataReqBody<T> = (String, DataResSender<T>);

pub type DataResSender<T> = Sender<(String, Result<T>)>;

pub type DataResRec<T> = Receiver<(String, Result<T>)>;

pub trait Loadable {
    type Output;
    fn load(path: &str) -> Result<Self::Output>;
}

fn load<T: Loadable>(
    resource_location: &str,
    resource_name: String,
    sender: DataResSender<T::Output>,
) {
    let path: String = resource_location.to_string() + &resource_name;
    let resource: Result<T::Output> = T::load(&path);
    sender.send((resource_name, resource)).unwrap();
}

const AUDIO_LOCATION: &'static str = "assets/sounds/";
const MAP_LOCATION: &'static str = "assets/maps/";
const MODEL_LOCATION: &'static str = "assets/models/";

