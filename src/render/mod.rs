mod cube_renderer;
mod point_light_renderer;
mod model_loader;

use std::collections::HashMap;
use std::f32::consts::PI;
use std::path::PathBuf;

use cube_renderer::CubeRenderer;
use gl;
use gl::types::*;
use image::DynamicImage;
use model_loader::{Material, Object, Texture};
use na::Perspective3;
use point_light_renderer::PointLightRenderer;
use tracing::debug;

use crate::shader::PointLight;
use crate::state::GameState;

use super::config::{
    CUBE_VERT_SHADER, LIGHT_FRAG_SHADER, LIGHT_VERT_SHADER, TEXTURE_FRAG_SHADER,
};
use super::shape::{CUBE_INDICES, CUBE_VERTICES};

pub struct Renderer {
    light: PointLightRenderer,
    cube: CubeRenderer,
    projection: Perspective3<GLfloat>,
}

impl Renderer {
    pub fn new(window_width: u32, window_height: u32) -> (Self, HashMap<String, ModelObjects>) {
        let aspect_ratio: GLfloat = window_width as GLfloat / window_height as GLfloat;
        let fovy: GLfloat = PI / 2.0;
        let znear: GLfloat = 0.1;
        let zfar: GLfloat = 100.0;
        let projection: Perspective3<GLfloat> = Perspective3::new(aspect_ratio, fovy, znear, zfar);

        let light_vert_shader = PathBuf::from(LIGHT_VERT_SHADER);
        let light_frag_shader = PathBuf::from(LIGHT_FRAG_SHADER);
        let light = PointLightRenderer::new(
            &light_vert_shader,
            &light_frag_shader,
            &CUBE_VERTICES,
            &CUBE_INDICES,
        )
        .unwrap();

        let cube_vert_shader = PathBuf::from(CUBE_VERT_SHADER);
        let texture_frag_shader = PathBuf::from(TEXTURE_FRAG_SHADER);

        // TODO: these should be loaded by a resource loader and passed in
        // the renderer should not own model_objects
        let mut textures: Vec<Texture> = Vec::new();
        let mut materials: Vec<Material> = Vec::new();
        let mut objects: Vec<Object> = Vec::new();
        let mut model_objects: HashMap<String, ModelObjects> = HashMap::new();
        let mut backpack = model_loader::Object::load(
            &"assets/models/backpack/backpack.obj".into(),
            &mut textures,
            &mut materials,
        );
        let mut plane = model_loader::Object::load(
            &"assets/models/plane/plane.obj".into(),
             &mut textures,
             &mut materials
        );
        let mut cube = model_loader::Object::load(
            &"assets/models/cube/cube.obj".into(),
             &mut textures,
             &mut materials
        );
        model_objects.insert("backpack".to_string(), ModelObjects {
            start: objects.len(),
            end: objects.len() + backpack.len(),
        });
        objects.append(&mut backpack);
        model_objects.insert("plane".to_string(), ModelObjects {
            start: objects.len(),
            end: objects.len() + plane.len(),
        });
        objects.append(&mut plane);
        model_objects.insert("cube".to_string(), ModelObjects {
            start: objects.len(),
            end: objects.len() + cube.len(),
        });
        objects.append(&mut cube);
        debug!(model = format!("{:?}", &objects), "loaded model");

        let textures: Vec<DynamicImage> = textures.into_iter()
            .map(|t| t.image)
            .collect();
        let cube = CubeRenderer::new(
            &cube_vert_shader,
            &texture_frag_shader,
            objects,
            materials,
            &textures,
        )
        .unwrap();

        (Self {
            light,
            cube,
            projection,
        }, model_objects)
    }

    pub fn render(&self, state: &GameState) {
        clear();
        let view = state.camera.transform();
        let light_uniforms: Vec<PointLight> =
            state.point_lights.iter().map(|l| l.as_light_uniforms()).collect();
        self.light
            .draw(&state.point_lights, view, self.projection.as_matrix().clone());
        self.cube.draw(
            &[state.cubes.as_slice(), &[state.player.cube.clone(), state.plane.clone()]].concat(),
            &light_uniforms,
            &state.dir_lights,
            &state.camera.position().into(),
            view,
            self.projection.as_matrix().clone(),
        );
    }
}

pub struct ModelObjects {
    pub start: usize,
    pub end: usize,
}

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
