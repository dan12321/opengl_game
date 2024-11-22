use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use na::{vector, Matrix4};
use tracing::debug;

use crate::audio::{AudioAction, AudioManager};
use crate::camera::Camera;
use crate::config::{
    self, BACKPACK_MODEL, BEAT_SIZE, COLUMN_WIDTH, CUBE_MODEL, DEATH_TRACK, PLANE_LENGTH,
    PLANE_MODEL, PLANE_WIDTH, SAD_MAP, UPBEAT_MAP,
};
use crate::controller::{Button, Controller};
use crate::physics::AABBColider;
use crate::render::Renderer;
use crate::resource::{manager::ResourceManager, map::Map};
use crate::shader;
use crate::shader::DirLight;

pub struct Game {
    audio_manager: AudioManager,
    renderer: Renderer,
    resource_manager: Arc<ResourceManager>,
    map_loading: Option<Map>,
    progress: f32,
    scene_resources: Option<SceneResources>,
    map_sender: Sender<(String, Result<Map>)>,
    map_receiver: Receiver<(String, Result<Map>)>,
    maps: Vec<String>,
    status: Status,
}

impl Game {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        // Setup managers
        let resource_manager = Arc::new(ResourceManager::new());
        let audio_manager = AudioManager::new(resource_manager.clone());
        let mut renderer = Renderer::new(window_width, window_height, resource_manager.clone());

        let (map_sender, map_receiver) = mpsc::channel();
        let status = Status::Loading;

        // Start loading initial level
        resource_manager.load_map(SAD_MAP.to_string(), map_sender.clone());
        renderer.load_models(vec![
            CUBE_MODEL.to_string(),
            PLANE_MODEL.to_string(),
            BACKPACK_MODEL.to_string(),
        ]);
        Self {
            audio_manager,
            resource_manager,
            map_sender,
            map_receiver,
            status,
            map_loading: None,
            renderer,
            scene_resources: None,
            maps: vec![SAD_MAP.to_string(), UPBEAT_MAP.to_string()],
            progress: 0.0,
        }
    }

    pub fn update(mut self, delta_time: Duration, controller: &Controller, window: &mut glfw::Window) -> Self {
        if controller.buttons().contains(&Button::Quit) {
            // These should "take" the resource but can't with how this is written.
            self.audio_manager.cleanup();
            self.resource_manager.cleanup();
            window.set_should_close(true);
            return self;
        }
        self.status = match self.status {
            Status::Load(map) => self.load_map(map),
            Status::Loading => self.loading_update(),
            Status::Play(scene) => match scene.player_state {
                PlayerStatus::Alive => scene.alive_update(delta_time, controller),
                PlayerStatus::Dead => scene.dead_update(delta_time, controller),
            },
            Status::Paused(scene) => scene.pause_update(controller),
            Status::Resetting(scene) => scene.resetting_update(),
        };
        self.renderer.update(self.status.get_state(), None);
        self
    }

    fn load_map(&mut self, map: usize) -> Status {
        // Clean up resources
        if let Some(mut scene_resources) = self.scene_resources.take() {
            if let Some(music) = scene_resources.music.take() {
                self.audio_manager.unload_wav(&music);
            }
        }

        // Initialise loading of new resources
        self.resource_manager
            .load_map(self.maps[map].clone(), self.map_sender.clone());
        self.progress = 0.0;
        Status::Loading
    }

    fn loading_update(&mut self) -> Status {
        let progress = ProgressBar {
            transform: Transform {
                position: (-0.5, -0.5, 0.0).into(),
                scale: (1.0, 0.2, 1.0).into(),
                rotation: Matrix4::identity(),
            },
            base_color: (0.0, 0.3, 0.7),
            progress_color: (0.0, 0.7, 1.0),
            progress: self.progress,
        };
        self.renderer.update(None, Some(&progress));

        if self.progress < 0.1 {
            self.progress += 0.0001;
        }
        if self.map_loading.is_none() {
            let Ok((_, map)) = self.map_receiver.try_recv() else {
                return Status::Loading;
            };
            let map = map.unwrap();
            // TODO: Move death_track to BaseLoading stage
            let wavs = [DEATH_TRACK, map.music.as_str()];
            self.audio_manager.load_wavs(&wavs);
            self.map_loading = Some(map);
            debug!("Map Loaded");
            if self.progress < 0.1 {
                self.progress = 0.11;
            }
        }

        if self.progress < 0.65 {
            self.progress += 0.00001;
        }

        if !self.audio_manager.loaded_check() {
            return Status::Loading;
        }

        if self.progress < 0.65 {
            self.progress = 0.66;
        }

        if self.progress < 1.0 {
            self.progress += 0.00001;
        }

        if !self.renderer.loaded_check() {
            return Status::Loading;
        }

        self.progress = 1.0;

        let progress = ProgressBar {
            transform: Transform {
                position: (-0.5, -0.5, 0.0).into(),
                scale: (1.0, 0.2, 1.0).into(),
                rotation: Matrix4::identity(),
            },
            base_color: (1.0, 0.0, 0.0),
            progress_color: (0.0, 1.0, 0.0),
            progress: self.progress,
        };
        self.renderer.update(None, Some(&progress));

        let audio_sender = self.audio_manager.get_sender();

        let scene = SceneState::new(self.map_loading.take().unwrap(), audio_sender);
        scene.play()
    }
}

#[derive(Debug)]
pub struct SceneState {
    pub cubes: Vec<GameObject>,
    pub point_lights: Vec<PointLight>,
    pub dir_lights: Vec<DirLight>,
    pub player: Player,
    speed: f32,
    pub plane: Plane,
    pub camera: Camera,
    map: Map,
    player_state: PlayerStatus,
    audio_sender: Sender<AudioAction>,
}

impl SceneState {
    pub fn new(map: Map, audio_sender: Sender<AudioAction>) -> Self {
        let camera = Camera::new(8.0, 0.0, -0.82, vector![0.0, 0.0, 0.0]);

        let cubes = Self::starting_cubes(&map);
        let lights = Self::starting_lights();

        SceneState {
            camera,
            cubes,
            point_lights: lights,
            dir_lights: vec![DirLight {
                direction: (0.0, -0.95, 0.34),
                diffuse: (0.75, 0.95, 1.0),
                specular: (0.6, 0.6, 0.6),
            }],
            player: Player {
                target_lane: 1,
                current_lane: 1,
                lerp: 1.0,
                model: GameObject {
                    transform: Transform {
                        position: (0.0, 0.75, 0.0).into(),
                        scale: (0.75, 0.75, 0.75).into(),
                        rotation: Matrix4::identity(),
                    },
                    model: BACKPACK_MODEL.to_string(),
                },
            },
            speed: BEAT_SIZE * (map.bpm / 60.0) * map.subdivisions,
            plane: Plane {
                models: [
                    GameObject {
                        transform: Transform {
                            position: (0.0, -0.5, 0.0).into(),
                            scale: (1.0, 1.0, 1.0).into(),
                            rotation: Matrix4::identity(),
                        },
                        model: PLANE_MODEL.to_string(),
                    },
                    GameObject {
                        transform: Transform {
                            position: (0.0, -0.5, -PLANE_LENGTH).into(),
                            scale: (1.0, 1.0, 1.0).into(),
                            rotation: Matrix4::identity(),
                        },
                        model: PLANE_MODEL.to_string(),
                    },
                    GameObject {
                        transform: Transform {
                            position: (0.0, -0.5, -PLANE_LENGTH * 2.0).into(),
                            scale: (1.0, 1.0, 1.0).into(),
                            rotation: Matrix4::identity(),
                        },
                        model: PLANE_MODEL.to_string(),
                    },
                ],
            },
            map,
            player_state: PlayerStatus::Alive,
            audio_sender,
        }
    }

    fn alive_update(mut self, delta_time: Duration, controller: &Controller) -> Status {
        // timing properties
        let dt = delta_time.as_secs_f32();
        let displacement = config::MOVE_SPEED * dt;

        // controller input
        let x = controller.direction();

        // lights update
        for light in &mut self.point_lights {
            if light.transform.position.z > 50.0 {
                light.transform.position.z = -90.0;
            }
            light.transform.position.z += self.speed * dt;
        }

        // cubes update
        for cube in &mut self.cubes {
            cube.transform.position.z += self.speed * dt;
        }

        // player update
        if self.player.lerp >= 1.0 {
            // TODO add some leaway so a double tap moves two lanes
            if x > 0.0 && self.player.target_lane < 2 {
                self.player.current_lane = self.player.target_lane;
                self.player.target_lane += 1;
                self.player.lerp = 0.0;
            } else if x < 0.0 && self.player.target_lane > 0 {
                self.player.current_lane = self.player.target_lane;
                self.player.target_lane -= 1;
                self.player.lerp = 0.0;
            }
        }
        self.player.lerp += displacement;
        if self.player.lerp > 1.0 {
            self.player.lerp = 1.0;
        }

        let movable_width = PLANE_WIDTH / 2.0;
        let start = (self.player.current_lane as f32 - 1.0) * movable_width;
        let end = (self.player.target_lane as f32 - 1.0) * movable_width;
        self.player.model.transform.position.x =
            end * self.player.lerp + start * (1.0 - self.player.lerp);
        if self.player.model.transform.position.x > movable_width {
            self.player.model.transform.position.x = movable_width;
        }
        if self.player.model.transform.position.x < -movable_width {
            self.player.model.transform.position.x = -movable_width;
        }

        // plane update
        self.plane.displace(self.speed * dt);

        // Check collisions
        let player_collider = AABBColider {
            position: self.player.model.transform.position,
            scale: self.player.model.transform.scale,
        };
        for cube in &self.cubes {
            let collider = AABBColider {
                position: cube.transform.position,
                scale: cube.transform.scale,
            };
            if player_collider.aabb_colided(&collider) {
                self.player_state = PlayerStatus::Dead;
                return self.death();
            }
        }

        if controller.buttons().contains(&Button::Pause) {
            return self.pause();
        }
        if let Some(map) = self.map_input(controller) {
            return self.load(map);
        }

        Status::Play(self)
    }

    fn dead_update(mut self, delta_time: Duration, controller: &Controller) -> Status {
        // timing properties
        let dt = delta_time.as_secs_f32();
        let speed_ratio = 0.5;
        let displacement = speed_ratio * self.speed * dt;

        // lights update
        for light in &mut self.point_lights {
            if light.transform.position.z > 50.0 {
                light.transform.position.z = -90.0;
            }
            light.transform.position.z += displacement;
        }

        // cubes update
        for cube in &mut self.cubes {
            cube.transform.position.z += displacement;
        }

        // player update
        self.player.model.transform.position.z += displacement;

        // plane update
        self.plane.displace(displacement);

        // controller input
        let reset = controller.buttons().contains(&Button::Restart);
        if let Some(map) = self.map_input(controller) {
            return self.load(map);
        }
        if reset {
            return self.reset();
        }
        if controller.buttons().contains(&Button::Pause) {
            return self.pause();
        }
        Status::Play(self)
    }

    fn pause_update(mut self, controller: &Controller) -> Status {
        // controller input
        let (camera_lat, camera_long) = controller.angle();
        self.camera.latitude = camera_long;
        self.camera.longitude = camera_lat;
        self.camera.distance = controller.zoom();
        let reset = controller.buttons().contains(&Button::Restart);
        let unpause = controller.buttons().contains(&Button::Pause);

        if let Some(map) = self.map_input(controller) {
            return self.load(map);
        }

        if reset {
            return self.reset();
        }
        if unpause {
            return self.play();
        }
        Status::Paused(self)
    }

    fn resetting_update(mut self) -> Status {
        self.point_lights = Self::starting_lights();
        self.cubes = Self::starting_cubes(&self.map);
        self.player.model.transform.position.x = 0.0;
        self.player.model.transform.position.z = 0.0;
        self.player.target_lane = 1;
        self.player.current_lane = 1;
        self.player_state = PlayerStatus::Alive;
        self.play()
    }

    fn starting_lights() -> Vec<PointLight> {
        let mut lights = Vec::with_capacity(64);
        for i in 0..=4 {
            let n = i as f32;
            let x = ((n / 4.0) * PLANE_WIDTH - (PLANE_WIDTH / 2.0)) * 1.75;
            let y = (-n * n + 4.0 * n) + 3.0;
            let light1_transform = Transform {
                position: (x, y, -10.0).into(),
                scale: (0.5, 0.5, 0.5).into(),
                rotation: Matrix4::identity(),
            };
            let light2_transform = Transform {
                position: (x, y, -50.0).into(),
                scale: (0.5, 0.5, 0.5).into(),
                rotation: Matrix4::identity(),
            };
            let light3_transform = Transform {
                position: (x, y, -90.0).into(),
                scale: (0.5, 0.5, 0.5).into(),
                rotation: Matrix4::identity(),
            };
            let light1 = PointLight {
                transform: light1_transform,
                diffuse: (1.0, 1.0, 0.75),
                specular: (1.0, 1.0, 0.75),
                strength: 5.0,
            };
            let light2 = PointLight {
                transform: light2_transform,
                diffuse: (1.0, 1.0, 0.75),
                specular: (1.0, 1.0, 0.75),
                strength: 5.0,
            };
            let light3 = PointLight {
                transform: light3_transform,
                diffuse: (1.0, 1.0, 0.75),
                specular: (1.0, 1.0, 0.75),
                strength: 5.0,
            };
            lights.push(light1);
            lights.push(light2);
            lights.push(light3);
        }

        lights
    }

    fn starting_cubes(map: &Map) -> Vec<GameObject> {
        let mut cubes = Vec::with_capacity(64);
        for i in 0..map.beats.len() {
            let (l, m, r) = map.beats[i];
            let padding = -(map.start_offset + i as f32) * BEAT_SIZE;
            if l {
                cubes.push(GameObject {
                    transform: Transform {
                        position: (-COLUMN_WIDTH, 0.0, padding).into(),
                        scale: (0.75, 0.75, 0.75).into(),
                        rotation: Matrix4::identity(),
                    },
                    // material: BOX_MATERIAL,
                    model: CUBE_MODEL.to_string(),
                });
            }
            if m {
                cubes.push(GameObject {
                    transform: Transform {
                        position: (0.0, 0.0, padding).into(),
                        scale: (0.75, 0.75, 0.75).into(),
                        rotation: Matrix4::identity(),
                    },
                    // material: BOX_MATERIAL,
                    model: CUBE_MODEL.to_string(),
                });
            }
            if r {
                cubes.push(GameObject {
                    transform: Transform {
                        position: (COLUMN_WIDTH, 0.0, padding).into(),
                        scale: (0.75, 0.75, 0.75).into(),
                        rotation: Matrix4::identity(),
                    },
                    // material: BOX_MATERIAL,
                    model: CUBE_MODEL.to_string(),
                });
            }
        }
        cubes
    }

    fn play(self) -> Status {
        let action = match self.player_state {
            PlayerStatus::Alive => AudioAction::Play(self.map.music.clone()),
            PlayerStatus::Dead => AudioAction::Slow(self.map.music.clone()),
        };
        self.audio_sender.send(action).unwrap();
        Status::Play(self)
    }

    fn death(self) -> Status {
        self.audio_sender
            .send(AudioAction::Play(DEATH_TRACK.to_string()))
            .unwrap();
        let action = AudioAction::Slow(self.map.music.clone());
        self.audio_sender.send(action).unwrap();
        Status::Play(self)
    }

    fn pause(self) -> Status {
        let action = AudioAction::Stop(self.map.music.clone());
        self.audio_sender.send(action).unwrap();
        Status::Paused(self)
    }

    fn reset(self) -> Status {
        let action = AudioAction::Reset(self.map.music.clone());
        self.audio_sender.send(action).unwrap();
        Status::Resetting(self)
    }

    fn load(self, map: usize) -> Status {
        let action = AudioAction::Reset(self.map.music.clone());
        self.audio_sender.send(action).unwrap();
        Status::Load(map)
    }

    fn map_input(&self, controller: &Controller) -> Option<usize> {
        for button in controller.buttons() {
            match button {
                Button::Level(i) => return Some(*i),
                _ => (),
            }
        }
        None
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

#[derive(Debug)]
pub enum Status {
    Play(SceneState),
    Resetting(SceneState),
    Paused(SceneState),
    // Storing map as a usize is a bit of a hack to keep ownership happy. This
    // is a sign that how updates is handled needs refactoring.
    Load(usize),
    Loading,
}

impl Status {
    fn get_state(&self) -> Option<&SceneState> {
        match self {
            Self::Play(s) | Self::Resetting(s) | Self::Paused(s) => Some(s),
            Self::Loading => None,
            Self::Load(_) => None,
        }
    }
}

#[derive(Debug)]
pub enum PlayerStatus {
    Alive,
    Dead,
}

struct SceneResources {
    music: Option<String>,
}
