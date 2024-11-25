pub mod level;
pub mod loading;

use std::{
    sync::{
        mpsc::Sender,
        Arc,
    },
    time::Duration,
};

use level::SceneState;
use loading::LoadingState;
use na::Matrix4;

use crate::{
    audio::{AudioManager, AudioMessage},
    config::{
        BACKPACK_MODEL, CUBE_MODEL, DEATH_TRACK, PLANE_LENGTH, PLANE_MODEL, SAD_MAP, UPBEAT_MAP,
    },
    controller::Controller,
    render::{RenderMessage, Renderer},
    resource::manager::ResourceManager,
    shader,
};

pub struct SceneManager {
    scene: Scene,
    resource_manager: Arc<ResourceManager>,
    maps: Vec<String>,
    audio_send: Sender<AudioMessage>,
}

enum Scene {
    Level(SceneState),
    Loading(LoadingState),
}

impl SceneManager {
    pub fn new(
        resource_manager: Arc<ResourceManager>,
        audio_send: Sender<AudioMessage>,
        render_send: Sender<RenderMessage>,
    ) -> Self {
        let maps = vec![SAD_MAP.to_string(), UPBEAT_MAP.to_string()];

        audio_send
            .send(AudioMessage::Load(DEATH_TRACK.to_string()))
            .unwrap();

        let global_models = [
            CUBE_MODEL.to_string(),
            PLANE_MODEL.to_string(),
            BACKPACK_MODEL.to_string(),
        ];

        for model in global_models {
            render_send.send(RenderMessage::Load(model)).unwrap();
        }

        let loading = LoadingState::new(&resource_manager, maps[0].clone(), audio_send.clone());
        Self {
            resource_manager,
            maps,
            scene: Scene::Loading(loading),
            audio_send,
        }
    }

    pub fn update(
        &mut self,
        delta_time: &Duration,
        controller: &Controller,
        audio_manager: &AudioManager,
        renderer: &Renderer,
    ) {
        match &mut self.scene {
            Scene::Level(l) => {
                l.update(delta_time, controller);
                if let Some(s) = l.change_scene {
                    let loading = LoadingState::new(
                        &self.resource_manager,
                        self.maps[s].clone(),
                        self.audio_send.clone(),
                    );
                    self.scene = Scene::Loading(loading);
                }
            }
            Scene::Loading(l) => {
                l.update(delta_time, audio_manager, renderer);
                if let Some(level) = l.level.take() {
                    self.scene = Scene::Level(level);
                }
            }
        }
    }

    pub fn get_level_state<'a>(&'a self) -> Option<&'a SceneState> {
        match &self.scene {
            Scene::Level(l) => Some(l),
            Scene::Loading(_) => None,
        }
    }

    pub fn get_progress_bar(&self) -> Option<ProgressBar> {
        match &self.scene {
            Scene::Level(_) => None,
            Scene::Loading(l) => Some(ProgressBar {
                transform: Transform {
                    position: (-0.5, -0.5, 0.0).into(),
                    scale: (1.0, 0.2, 1.0).into(),
                    rotation: Matrix4::identity(),
                },
                base_color: (0.0, 0.3, 0.7),
                progress_color: (0.0, 0.7, 1.0),
                progress: f32::min(l.progress, 1.0),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameObject {
    pub transform: Transform,
    pub model: String,
}

#[derive(Copy, Clone, Debug)]
pub struct PointLight {
    pub transform: Transform,
    pub diffuse: (f32, f32, f32),
    pub specular: (f32, f32, f32),
    pub strength: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct ProgressBar {
    pub transform: Transform,
    pub base_color: (f32, f32, f32),
    pub progress_color: (f32, f32, f32),
    pub progress: f32,
}

impl PointLight {
    pub fn as_light_uniforms(&self) -> shader::PointLight {
        let pos = self.transform.position;
        shader::PointLight {
            position: (pos.x, pos.y, pos.z),
            diffuse: self.diffuse,
            specular: self.specular,
            strength: self.strength,
        }
    }
}

#[derive(Debug)]
pub struct Plane {
    pub models: [GameObject; 3],
}

impl Plane {
    fn displace(&mut self, z: f32) {
        for p in &mut self.models {
            p.transform.position.z += z;
            if p.transform.position.z >= PLANE_LENGTH {
                p.transform.position.z -= PLANE_LENGTH * 2.0;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    target_lane: usize,
    current_lane: usize,
    lerp: f32,
    pub model: GameObject,
}

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: XYZ,
    pub scale: XYZ,
    pub rotation: Matrix4<f32>,
}

#[derive(Copy, Clone, Debug)]
pub struct XYZ {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<(f32, f32, f32)> for XYZ {
    fn from(xyz: (f32, f32, f32)) -> Self {
        XYZ {
            x: xyz.0,
            y: xyz.1,
            z: xyz.2,
        }
    }
}

impl Transform {
    pub fn to_matrix4(&self) -> Matrix4<f32> {
        na::Translation3::new(self.position.x, self.position.y, self.position.z).to_homogeneous()
            * self.rotation
            * na::Scale3::new(self.scale.x, self.scale.y, self.scale.z).to_homogeneous()
    }
}
