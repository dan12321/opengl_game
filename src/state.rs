use std::time::{Instant, Duration};

use na::{vector, Matrix4, Rotation3, Translation3};

use crate::camera::Camera;
use crate::controller::{self, Controller};
use crate::{light::LightUniform, model::Material};
use crate::config::{PLANE_LENGTH, PLANE_WIDTH, self};
use crate::physics::AABBColider;

pub struct GameState {
    pub cubes: Vec<Cube>,
    pub lights: Vec<Light>,
    pub player: Player,
    speed: f32,
    pub plane: Plane,
    pub camera: Camera,
}

impl GameState {
    pub fn new() -> Self {
        let camera = Camera::new(10.0, 0.0, -0.5, vector![0.0, 0.0, 0.0]);

        let mut cubes = Vec::with_capacity(64);
        let positions = [(1.75, 0.0, -10.0), (-1.75, 0.0, -10.0), (0.0, 0.0, -15.0)];
        for position in positions {
            cubes.push(Cube {
                transform: Transform {
                    position: position.into(),
                    scale: (1.0, 1.0, 1.0).into(),
                    rotation: Matrix4::identity(),
                },
                material: PLAYER_MATERIAL,
            });
        }

        let mut lights = Vec::with_capacity(64);
        for i in 0..=4 {
            let n = i as f32;
            let x = ((n / 4.0) * PLANE_WIDTH - (PLANE_WIDTH / 2.0)) * 1.2;
            let y = (-n * n + 4.0 * n) + 1.0;
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
            let light1 = Light {
                transform: light1_transform,
                diffuse: (1.0, 1.0, 1.0),
                specular: (1.0, 1.0, 1.0),
                strength: 50.0,
            };
            let light2 = Light {
                transform: light2_transform,
                diffuse: (1.0, 1.0, 1.0),
                specular: (1.0, 1.0, 1.0),
                strength: 50.0,
            };
            let light3 = Light {
                transform: light3_transform,
                diffuse: (1.0, 1.0, 1.0),
                specular: (1.0, 1.0, 1.0),
                strength: 50.0,
            };
            lights.push(light1);
            lights.push(light2);
            lights.push(light3);
        }

        GameState {
            camera,
            cubes,
            lights,
            player: Player {
                target_lane: 1,
                current_lane: 1,
                lerp: 1.0,
                cube: Cube {
                    transform: Transform {
                        position: (0.0, 0.0, 0.0).into(),
                        scale: (1.0, 1.0, 1.0).into(),
                        rotation: Matrix4::identity(),
                    },
                    material: PLAYER_MATERIAL,
                },
            },
            speed: 1.0,
            plane: Plane {
                transform: Transform {
                    position: (0.0, -0.5, 0.0).into(),
                    scale: (PLANE_WIDTH, PLANE_LENGTH, 1.0).into(),
                    rotation: Rotation3::from_euler_angles(1.570796, 0.0, 0.0).to_homogeneous(),
                },
                material: PLAYER_MATERIAL,
                offset: 0.0,
            },
        }
    }

    pub fn update(&mut self, delta_time: Duration, controller: &Controller) {
        // timing properties
        let dt = delta_time.as_secs_f32();
        self.speed += dt;
        let displacement = config::MOVE_SPEED * dt;
        
        // controller input
        let x = controller.direction();
        let (cx, cy, zoom) = controller.mouse();

        // lights update
        for light in &mut self.lights {
            if light.transform.position.z > 50.0 {
                light.transform.position.z = -90.0;
            }
            light.transform.position.z += self.speed * dt;
        }

        // cubes update
        for cube in &mut self.cubes {
            if cube.transform.position.z > 20.0 {
                cube.transform.position.z = -50.0;
            }
            cube.transform.position.z += self.speed * dt;
        }

        // player update
        if self.player.lerp > 0.999 {
            // TODO add some leaway so a double tap moves two lanes
            if x > 0.0 && self.player.target_lane < 2 {
                self.player.current_lane = self.player.target_lane;
                self.player.target_lane += 1;
                self.player.lerp = 1.0 - self.player.lerp;
            } else if x < 0.0 && self.player.target_lane > 0 {
                self.player.current_lane = self.player.target_lane;
                self.player.target_lane -= 1;
                self.player.lerp = 1.0 - self.player.lerp;
            }
        } else if self.player.lerp < 1.0 {
            self.player.lerp += displacement;
        }
        let start = (self.player.current_lane as f32 - 1.0) * 1.75;
        let end = (self.player.target_lane as f32 - 1.0) * 1.75;
        self.player.cube.transform.position.x =
            end * self.player.lerp + start * (1.0 - self.player.lerp);
        let movable_width = PLANE_WIDTH / 2.0 - 0.5;
        if self.player.cube.transform.position.x > movable_width {
            self.player.cube.transform.position.x = movable_width;
        }
        if self.player.cube.transform.position.x < -movable_width {
            self.player.cube.transform.position.x = -movable_width;
        }

        // Check collisions
        let player_collider = AABBColider {
            position: self.player.cube.transform.position,
            scale: self.player.cube.transform.scale,
        };
        for cube in &self.cubes {
            let collider = AABBColider {
                position: cube.transform.position,
                scale: cube.transform.scale,
            };
            if player_collider.aabb_colided(&collider) {
                self.speed = 1.0;
                self.player.cube.transform.position.x = 0.0;
                self.player.target_lane = 1;
                self.player.current_lane = 1;
            }
        }

        // camera update
        self.camera.latitude = cx as f32 * config::CURSOR_MOVEMENT_SCALE;
        self.camera.longitude = -cy * config::CURSOR_MOVEMENT_SCALE;
        self.camera.distance = self.camera.default_distance + zoom;
        self.camera.target = Translation3::new(
            self.player.cube.transform.position.x,
            self.player.cube.transform.position.y,
            self.player.cube.transform.position.z,
        ).vector;

        // plane update
        self.plane.offset -= self.speed * dt / PLANE_LENGTH;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Cube {
    pub transform: Transform,
    pub material: Material,
}

#[derive(Copy, Clone, Debug)]
pub struct Plane {
    pub transform: Transform,
    pub material: Material,
    pub offset: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct Light {
    pub transform: Transform,
    pub diffuse: (f32, f32, f32),
    pub specular: (f32, f32, f32),
    pub strength: f32,
}

impl Light {
    pub fn as_light_uniforms(&self) -> LightUniform {
        let pos = self.transform.position;
        LightUniform {
            position: (pos.x, pos.y, pos.z),
            diffuse: self.diffuse,
            specular: self.specular,
            strength: self.strength,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Player {
    target_lane: usize,
    current_lane: usize,
    lerp: f32,
    pub cube: Cube,
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

pub static PLAYER_MATERIAL: Material = Material {
    ambient: (0.5, 0.1, 0.1),
    diffuse: (1.0, 0.7, 0.7),
    specular: (1.0, 0.7, 0.7),
    shininess: 128,
};
