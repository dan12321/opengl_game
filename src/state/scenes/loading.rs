use std::sync::{mpsc::{self, Receiver, Sender}, Arc};

use anyhow::Result;
use tracing::debug;

use crate::{audio::{AudioManager, AudioMessage}, render::Renderer, resource::{manager::{DataResRec, ResourceManager}, map::Map}, state::game::Game};

use super::level::SceneState;

pub struct LoadingState {
    pub progress: f32,
    map: Option<Map>,
    map_receiver: DataResRec<Map>,
    pub level: Option<SceneState>,
    audio_send: Sender<AudioMessage>,
}

impl LoadingState {
    pub fn new(resource_manager: &Arc<ResourceManager>, map: String, audio_send: Sender<AudioMessage>) -> Self {
        let (map_sender, map_receiver) = mpsc::channel::<(String, Result<Map>)>();
        resource_manager
            .load_map(map, map_sender);
        Self {
            progress: 0.0,
            map: None,
            map_receiver,
            level: None,
            audio_send,
        }
    }

    pub fn update(&mut self, audio_manager: &AudioManager, renderer: &Renderer) {
        if self.progress < 0.1 {
            self.progress += 0.0001;
        }
        if self.map.is_none() {
            let Ok((_, map)) = self.map_receiver.try_recv() else {
                return;
            };
            let map = map.unwrap();
            let wavs = [map.music.as_str()];
            for wav in wavs {
                self.audio_send.send(AudioMessage::Load(wav.to_string())).unwrap();
            }
            self.map = Some(map);
            debug!("Map Loaded");
            if self.progress < 0.1 {
                self.progress = 0.11;
            }
        }

        let (loading_audio, loaded_audio) = audio_manager.loaded_check();
        let (loading_models, loaded_models) = renderer.loaded_check();
        
        if loading_audio + loading_models == 0 {
            // TODO: Base this on delta time instead of being frame rate dependant
            self.progress += 0.0001;
        } else {
            let loaded_assets = loaded_audio + loaded_models;
            let loading_total = loaded_assets + loading_audio + loading_models;
            let target_progress = 0.1 + 0.9 * (loaded_assets as f32 / loading_total as f32);
            let mut delta = target_progress - self.progress;
            delta = f32::max(delta, 0.0);
            self.progress += delta * delta / 600.0;
        }
        // We want a chance for this to be full
        if self.progress <= 1.2 {
            return;
        }

        self.level = Some(SceneState::new(self.map.take().unwrap(), self.audio_send.clone()));
    }
}


