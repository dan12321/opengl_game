mod map;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use map::Map;
use na::{vector, Matrix4};

use crate::camera::Camera;
use crate::config::{self, BEAT_SIZE, COLUMN_WIDTH, PLANE_LENGTH, PLANE_WIDTH};
use crate::controller::{Button, Controller};
use crate::render::ModelMeshes;
use crate::{file_utils, shader};
use crate::physics::AABBColider;
use crate::shader::DirLight;

pub struct GameState {
    pub cubes: Vec<Model>,
    pub point_lights: Vec<PointLight>,
    pub dir_lights: Vec<DirLight>,
    pub player: Player,
    speed: f32,
    pub plane: Plane,
    pub camera: Camera,
    map: Map,
    pub status: Status,
    cube_model: Vec<usize>,
}

impl GameState {
    pub fn new(level: &PathBuf, model_objects: &HashMap<String, ModelMeshes>) -> Self {
        let camera = Camera::new(8.0, 0.0, -0.82, vector![0.0, 0.0, 0.0]);

        let full_path = file_utils::get_level_file(level, ".txt");
        let map = Map::from_file(&full_path);
        let cube_model = model_objects.get("cube").unwrap();
        let plane_model = model_objects.get("plane").unwrap();
        let backpack_model = model_objects.get("backpack").unwrap();

        let cubes = Self::starting_cubes(&map, &(cube_model.start..cube_model.end).collect());
        let lights = Self::starting_lights();

        GameState {
            camera,
            cubes,
            point_lights: lights,
            dir_lights: vec![
                DirLight {
                    direction: (0.0, -0.95, 0.34),
                    diffuse: (0.75, 0.95, 1.0),
                    specular: (0.6, 0.6, 0.6),
                }
            ],
            player: Player {
                target_lane: 1,
                current_lane: 1,
                lerp: 1.0,
                model: Model {
                    transform: Transform {
                        position: (0.0, 0.0, 0.0).into(),
                        scale: (0.75, 0.75, 0.75).into(),
                        rotation: Matrix4::identity(),
                    },
                    meshes: (backpack_model.start..backpack_model.end).collect(),
                },
            },
            speed: BEAT_SIZE * (map.bpm / 60.0) * map.subdivisions,
            plane: Plane {
                models: [
                    Model {
                        transform: Transform {
                            position: (0.0, -0.5, 0.0).into(),
                            scale: (1.0, 1.0, 1.0).into(),
                            rotation: Matrix4::identity(),
                        },
                        meshes: (plane_model.start..plane_model.end).collect(),
                    },
                    Model {
                        transform: Transform {
                            position: (0.0, -0.5, -PLANE_LENGTH).into(),
                            scale: (1.0, 1.0, 1.0).into(),
                            rotation: Matrix4::identity(),
                        },
                        meshes: (plane_model.start..plane_model.end).collect(),
                    },
                    Model {
                        transform: Transform {
                            position: (0.0, -0.5, -PLANE_LENGTH * 2.0).into(),
                            scale: (1.0, 1.0, 1.0).into(),
                            rotation: Matrix4::identity(),
                        },
                        meshes: (plane_model.start..plane_model.end).collect(),
                    }
                ]
            },
            map,
            status: Status::Alive,
            cube_model: (cube_model.start..cube_model.end).collect(),
        }
    }

    pub fn update(&mut self, delta_time: Duration, controller: &Controller) {
        self.status = match self.status.to_owned() {
            Status::Alive => self.alive_update(delta_time, controller),
            Status::Dead => self.dead_update(delta_time, controller),
            Status::Paused(ls) => self.pause_update(controller, ls),
            Status::Resetting => self.resetting_update(),
        }
    }

    fn alive_update(&mut self, delta_time: Duration, controller: &Controller) -> Status {
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
                return Status::Dead;
            }
        }

        if controller.buttons().contains(&Button::Pause) {
            return Status::Paused(Status::Alive.into());
        }

        Status::Alive
    }

    fn dead_update(&mut self, delta_time: Duration, controller: &Controller) -> Status {
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
        if reset {
            return Status::Resetting;
        }
        if controller.buttons().contains(&Button::Pause) {
            return Status::Paused(Status::Dead.into());
        }
        Status::Dead
    }

    fn pause_update(&mut self, controller: &Controller, last_status: usize) -> Status {
        // controller input
        let (camera_lat, camera_long) = controller.angle();
        self.camera.latitude = camera_long;
        self.camera.longitude = camera_lat;
        self.camera.distance = controller.zoom();
        let reset = controller.buttons().contains(&Button::Restart);
        let unpause = controller.buttons().contains(&Button::Pause);

        if reset {
            return Status::Resetting;
        } else if unpause {
            return last_status.into();
        }
        Status::Paused(last_status)
    }

    fn resetting_update(&mut self) -> Status {
        self.point_lights = Self::starting_lights();
        self.cubes = Self::starting_cubes(&self.map, &self.cube_model);
        self.player.model.transform.position.x = 0.0;
        self.player.model.transform.position.z = 0.0;
        self.player.target_lane = 1;
        self.player.current_lane = 1;
        Status::Alive
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

    fn starting_cubes(map: &Map, cube_model: &Vec<usize>) -> Vec<Model> {
        let mut cubes = Vec::with_capacity(64);
        for i in 0..map.beats.len() {
            let (l, m, r) = map.beats[i];
            let padding = -(map.start_offset + i as f32) * BEAT_SIZE;
            if l {
                cubes.push(Model {
                    transform: Transform {
                        position: (-COLUMN_WIDTH, 0.0, padding).into(),
                        scale: (0.75, 0.75, 0.75).into(),
                        rotation: Matrix4::identity(),
                    },
                    // material: BOX_MATERIAL,
                    meshes: cube_model.clone(),
                });
            }
            if m {
                cubes.push(Model {
                    transform: Transform {
                        position: (0.0, 0.0, padding).into(),
                        scale: (0.75, 0.75, 0.75).into(),
                        rotation: Matrix4::identity(),
                    },
                    // material: BOX_MATERIAL,
                    meshes: cube_model.clone(),
                });
            }
            if r {
                cubes.push(Model {
                    transform: Transform {
                        position: (COLUMN_WIDTH, 0.0, padding).into(),
                        scale: (0.75, 0.75, 0.75).into(),
                        rotation: Matrix4::identity(),
                    },
                    // material: BOX_MATERIAL,
                    meshes: cube_model.clone(),
                });
            }
        }
        cubes
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    pub transform: Transform,
    pub meshes: Vec<usize>,
}

#[derive(Copy, Clone, Debug)]
pub struct PointLight {
    pub transform: Transform,
    pub diffuse: (f32, f32, f32),
    pub specular: (f32, f32, f32),
    pub strength: f32,
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

pub struct Plane {
    pub models: [Model; 3],
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
    pub model: Model,
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

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Status {
    Alive,
    Dead,
    Resetting,
    // Using a usize to represent last status since other status' don't contain
    // data and it make ownership easier
    Paused(usize),
}

impl From<usize> for Status {
    fn from(value: usize) -> Self {
        match value {
            0 => Status::Alive,
            1 => Status::Dead,
            2 => Status::Resetting,
            _ => panic!("unexpected status value"),
        }
    }
}

impl Into<usize> for Status {
    fn into(self) -> usize {
        match self {
            Self::Alive => 0,
            Self::Dead => 1,
            Self::Resetting => 2,
            Self::Paused(_) => panic!("can't represent paused as usize"),
        }
    }
}