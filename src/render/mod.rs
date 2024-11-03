mod model_renderer;
mod point_light_renderer;
mod model_loader;
mod outline_renderer;

use std::collections::HashMap;
use std::f32::consts::PI;
use std::path::PathBuf;

use model_renderer::ModelRenderer;
use gl;
use gl::types::*;
use image::DynamicImage;
use model_loader::{Material, Mesh, Texture};
use na::Perspective3;
use outline_renderer::OutlineRenderer;
use point_light_renderer::PointLightRenderer;
use tracing::debug;

use crate::config::{OUTLINE_FRAG_SHADER, OUTLINE_VERT_SHADER};
use crate::shader::PointLight;
use crate::state::SceneState;

use super::config::{
    MODEL_VERT_SHADER, LIGHT_FRAG_SHADER, LIGHT_VERT_SHADER, TEXTURE_FRAG_SHADER,
};
use super::shape::{CUBE_INDICES, CUBE_VERTICES};

pub struct Renderer {
    light: PointLightRenderer,
    model: ModelRenderer,
    projection: Perspective3<GLfloat>,
    outline: OutlineRenderer,
}

impl Renderer {
    pub fn new(window_width: u32, window_height: u32) -> (Self, HashMap<String, ModelMeshes>) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::STENCIL_TEST);
        }
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

        let model_vert_shader = PathBuf::from(MODEL_VERT_SHADER);
        let texture_frag_shader = PathBuf::from(TEXTURE_FRAG_SHADER);
        let outline_vert_shader = PathBuf::from(OUTLINE_VERT_SHADER);
        let outline_frag_shader = PathBuf::from(OUTLINE_FRAG_SHADER);

        // TODO: these should be loaded by a resource loader and passed in
        // the renderer should not own model_meshes
        let mut textures: Vec<Texture> = Vec::new();
        let mut materials: Vec<Material> = Vec::new();
        let mut meshes: Vec<Mesh> = Vec::new();
        let mut model_meshes: HashMap<String, ModelMeshes> = HashMap::new();
        let mut backpack = model_loader::Mesh::load(
            &"assets/models/backpack/backpack.obj".into(),
            &mut textures,
            &mut materials,
        );
        let mut plane = model_loader::Mesh::load(
            &"assets/models/plane/plane.obj".into(),
             &mut textures,
             &mut materials
        );
        let mut cube = model_loader::Mesh::load(
            &"assets/models/cube/cube.obj".into(),
             &mut textures,
             &mut materials
        );
        model_meshes.insert("backpack".to_string(), ModelMeshes {
            start: meshes.len(),
            end: meshes.len() + backpack.len(),
        });
        meshes.append(&mut backpack);
        model_meshes.insert("plane".to_string(), ModelMeshes {
            start: meshes.len(),
            end: meshes.len() + plane.len(),
        });
        meshes.append(&mut plane);
        model_meshes.insert("cube".to_string(), ModelMeshes {
            start: meshes.len(),
            end: meshes.len() + cube.len(),
        });
        meshes.append(&mut cube);
        debug!(model = format!("{:?}", &meshes), "loaded model");

        let textures: Vec<DynamicImage> = textures.into_iter()
            .map(|t| t.image)
            .collect();
        
        let outline = OutlineRenderer::new(
            &outline_vert_shader,
            &outline_frag_shader,
            &meshes,
        ).unwrap();

        let model = ModelRenderer::new(
            &model_vert_shader,
            &texture_frag_shader,
            meshes,
            materials,
            &textures,
        )
        .unwrap();

        (Self {
            light,
            model,
            projection,
            outline,
        }, model_meshes)
    }

    pub fn render(&self, state: &SceneState) {
        clear();
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::REPLACE);
            gl::StencilMask(0x00);
        }
        let view = state.camera.transform();
        let light_uniforms: Vec<PointLight> =
            state.point_lights.iter().map(|l| l.as_light_uniforms()).collect();
        self.light
            .draw(&state.point_lights, view, self.projection.as_matrix().clone());

        // Non stenciled models
        self.model.draw(
            &[&[state.player.model.clone()], state.plane.models.as_slice()].concat(),
            &light_uniforms,
            &state.dir_lights,
            &state.camera.position().into(),
            view,
            self.projection.as_matrix().clone(),
        );

        // Stenciled Models
        unsafe {
            gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
            gl::StencilMask(0xFF);
        }
        self.model.draw(
            state.cubes.as_slice(),
            &light_uniforms,
            &state.dir_lights,
            &state.camera.position().into(),
            view,
            self.projection.as_matrix().clone(),
        );
        unsafe {
            gl::StencilFunc(gl::NOTEQUAL, 1, 0xFF);
            gl::StencilMask(0x00);
            gl::Disable(gl::DEPTH_TEST);
        }
        self.outline.draw(
            state.cubes.as_slice(),
            view,
            self.projection.as_matrix().clone());
        unsafe {
            gl::StencilMask(0xFF);
            gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
            gl::Enable(gl::DEPTH_TEST);
        }
    }
}

pub struct ModelMeshes {
    pub start: usize,
    pub end: usize,
}

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
    }
}
