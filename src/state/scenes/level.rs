use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use glfw::{Context, Window};
use na::{vector, Matrix4};
use tracing::debug;

use crate::audio::{AudioManager, AudioMessage, TrackAction};
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
use crate::state::game::Game;

use super::{GameObject, Plane, Player, PointLight, Transform};

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
    paused: bool,
    player_state: PlayerStatus,
    audio_sender: Sender<AudioMessage>,
    pub change_scene: Option<usize>,
}

impl SceneState {
    pub fn new(map: Map, audio_sender: Sender<AudioMessage>) -> Self {
        let camera = Camera::new(8.0, 0.0, -0.82, vector![0.0, 0.0, 0.0]);

        let cubes = Self::starting_cubes(&map);
        let lights = Self::starting_lights();
        

        let mut scene = SceneState {
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
            paused: false,
            change_scene: None,
        };

        scene.play();
        scene
    }

    pub fn update(&mut self, delta_time: &Duration, controller: &Controller) {
        if self.paused {
            self.pause_update(controller);
        } else {
            match self.player_state {
                PlayerStatus::Alive => self.alive_update(delta_time, &controller),
                PlayerStatus::Dead => self.dead_update(delta_time, &controller),
            }
        }
    }

    fn alive_update(&mut self, delta_time: &Duration, controller: &Controller) {
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
                self.death();
                return;
            }
        }

        if controller.buttons().contains(&Button::Pause) {
            self.pause();
            return;
        }
        if let Some(map) = self.map_input(controller) {
            self.load(map);
            return;
        }
    }

    fn dead_update(&mut self, delta_time: &Duration, controller: &Controller) {
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
            self.load(map);
            return;
        }
        if reset {
            self.reset();
            return;
        }
        if controller.buttons().contains(&Button::Pause) {
            self.pause();
            return;
        }
    }

    fn pause_update(&mut self, controller: &Controller) {
        // controller input
        let (camera_lat, camera_long) = controller.angle();
        self.camera.latitude = camera_long;
        self.camera.longitude = camera_lat;
        self.camera.distance = controller.zoom();
        let reset = controller.buttons().contains(&Button::Restart);
        let unpause = controller.buttons().contains(&Button::Pause);

        if let Some(map) = self.map_input(controller) {
            self.load(map);
            return;
        }

        if reset {
            self.reset();
            return;
        }
        if unpause {
            self.play();
            return;
        }
    }

    fn resetting_update(&mut self) {
        self.point_lights = Self::starting_lights();
        self.cubes = Self::starting_cubes(&self.map);
        self.player.model.transform.position.x = 0.0;
        self.player.model.transform.position.z = 0.0;
        self.player.target_lane = 1;
        self.player.current_lane = 1;
        self.player_state = PlayerStatus::Alive;
        self.play();
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

    fn play(&mut self) {
        let action = match self.player_state {
            PlayerStatus::Alive => TrackAction::Play(self.map.music.clone()),
            PlayerStatus::Dead => TrackAction::Slow(self.map.music.clone()),
        };
        let message = AudioMessage::TrackAction(action);
        self.audio_sender.send(message).unwrap();
        self.paused = false;
    }

    fn pause(&mut self) {
        let action = TrackAction::Stop(self.map.music.clone());
        let message = AudioMessage::TrackAction(action);
        self.audio_sender.send(message).unwrap();
        self.paused = true;
    }

    fn death(&mut self) {
        let play_death = TrackAction::Play(DEATH_TRACK.to_string());
        self.audio_sender
            .send(AudioMessage::TrackAction(play_death))
            .unwrap();
        let action = TrackAction::Slow(self.map.music.clone());
        let message = AudioMessage::TrackAction(action);
        self.audio_sender.send(message).unwrap();
        self.player_state = PlayerStatus::Dead;
    }

    fn reset(&mut self) {
        let action = TrackAction::Reset(self.map.music.clone());
        let message = AudioMessage::TrackAction(action);
        self.audio_sender.send(message).unwrap();
        self.resetting_update();
    }

    fn load(&mut self, map: usize) {
        let action = TrackAction::Reset(self.map.music.clone());
        let message = AudioMessage::TrackAction(action);
        self.audio_sender.send(message).unwrap();
        self.change_scene = Some(map);
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

#[derive(Debug)]
pub enum PlayerStatus {
    Alive,
    Dead,
}

