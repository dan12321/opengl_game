use na::Matrix4;

use crate::{model::Material, light::LightUniform};

pub struct GameState {
    cubes: [Cube; 128],
    lights: [Light; 64],
//    box_renderer: Renderer,
//    light_renderer: Renderer,
//    plane_renderer: Renderer,
    player: Cube,
    speed: f32,
    plane: Plane,
}

#[derive(Copy, Clone, Debug)]
pub struct Cube {
    pub transform: Transform,
    pub material: Material,
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
    transform: Transform,
}

#[derive(Copy, Clone, Debug)]
pub struct Plane {
    offset: f32,
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
