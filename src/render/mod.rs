mod model_renderer;
mod point_light_renderer;
mod progress_renderer;

use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc};

use gl;
use gl::types::*;
use glfw::Window;
use model_renderer::ModelRenderer;
use na::Perspective3;
use point_light_renderer::PointLightRenderer;
use progress_renderer::ProgressRenderer;
use tracing::error;

use crate::config::{PROGRESS_FRAG_SHADER, PROGRESS_VERT_SHADER};
use crate::resource::manager::{DataResRec, DataResSender, ResourceManager};
use crate::resource::model::{Material, Model, Texture};
use crate::shader::PointLight;
use crate::shape::{QUAD_INDICES, QUAD_VERTICES};
use crate::state::scenes::SceneManager;

use super::config::{LIGHT_FRAG_SHADER, LIGHT_VERT_SHADER, MODEL_VERT_SHADER, TEXTURE_FRAG_SHADER};
use super::shape::{CUBE_INDICES, CUBE_VERTICES};

pub struct Renderer {
    light: PointLightRenderer,
    model: ModelRenderer,
    progress: ProgressRenderer,
    resource_manager: Arc<ResourceManager>,
    message_rec: Receiver<RenderMessage>,
    loading_models: HashSet<String>,
    models: HashMap<String, Vec<String>>,
    model_sender: DataResSender<Model>,
    model_rec: DataResRec<Model>,
    loading_material_files: HashSet<String>,
    material_files: HashSet<String>,
    materials: HashMap<String, Material>,
    material_sender: DataResSender<Vec<Material>>,
    material_rec: DataResRec<Vec<Material>>,
    loading_textures: HashSet<String>,
    textures: HashSet<String>,
    texture_sender: DataResSender<Texture>,
    texture_rec: DataResRec<Texture>,
}

impl Renderer {
    pub fn new(resource_manager: Arc<ResourceManager>, message_rec: Receiver<RenderMessage>) -> Self {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }
        let light_vert_shader = PathBuf::from(LIGHT_VERT_SHADER);
        let light_frag_shader = PathBuf::from(LIGHT_FRAG_SHADER);
        let light = PointLightRenderer::new(
            &light_vert_shader,
            &light_frag_shader,
            &CUBE_VERTICES,
            &CUBE_INDICES,
        )
        .unwrap();

        let progress_vert_shader = PathBuf::from(PROGRESS_VERT_SHADER);
        let progress_frag_shader = PathBuf::from(PROGRESS_FRAG_SHADER);
        let progress = ProgressRenderer::new(
            &progress_vert_shader,
            &progress_frag_shader,
            &QUAD_VERTICES,
            &QUAD_INDICES,
        )
        .unwrap();

        let model_vert_shader = PathBuf::from(MODEL_VERT_SHADER);
        let texture_frag_shader = PathBuf::from(TEXTURE_FRAG_SHADER);

        let model = ModelRenderer::new(&model_vert_shader, &texture_frag_shader).unwrap();

        let (model_sender, model_rec) = mpsc::channel();
        let (material_sender, material_rec) = mpsc::channel();
        let (texture_sender, texture_rec) = mpsc::channel();

        Self {
            light,
            model,
            progress,
            resource_manager,
            message_rec,
            loading_models: HashSet::new(),
            models: HashMap::new(),
            loading_material_files: HashSet::new(),
            material_files: HashSet::new(),
            materials: HashMap::new(),
            loading_textures: HashSet::new(),
            textures: HashSet::new(),
            model_sender,
            model_rec,
            material_sender,
            material_rec,
            texture_sender,
            texture_rec,
        }
    }

    pub fn load_model(&mut self, model: String) {
        if self.loading_models.contains(&model) || self.models.contains_key(&model) {
            return;
        }
        self.resource_manager
            .load_model(model.clone(), self.model_sender.clone());
        self.loading_models.insert(model);
    }

    pub fn load_materials(&mut self, materials: Vec<String>) {
        for mat in materials {
            if self.loading_material_files.contains(&mat) || self.materials.contains_key(&mat) {
                continue;
            }
            self.resource_manager
                .load_material(mat.clone(), self.material_sender.clone());
            self.loading_material_files.insert(mat);
        }
    }

    pub fn load_textures(&mut self, textures: Vec<String>) {
        for texture in textures {
            if self.loading_textures.contains(&texture) || self.textures.contains(&texture) {
                continue;
            }
            self.resource_manager
                .load_texture(texture.clone(), self.texture_sender.clone());
            self.loading_textures.insert(texture);
        }
    }

    pub fn loaded_check(&self) -> (usize, usize) {
        let loaded_assets = self.models.len() + self.materials.len() + self.textures.len();
        let loading_assets = self.loading_models.len()
            + self.loading_material_files.len()
            + self.loading_textures.len();
        (loading_assets, loaded_assets)
    }

    pub fn update(&mut self, window: &Window, scene_manager: &SceneManager) {
        // Check Messages
        while let Ok(message) = self.message_rec.try_recv() {
            match message {
                RenderMessage::Load(s) => self.load_model(s),
            }
        }
        // Check loading
        self.loading_update();

        // Render
        self.render(window, scene_manager);
    }

    fn render(&self, window: &Window, scene_manager: &SceneManager) {
        clear();
        let (window_width, window_height) = window.get_size();
        let aspect_ratio: GLfloat = window_width as GLfloat / window_height as GLfloat;
        let fovy: GLfloat = PI / 2.0;
        let znear: GLfloat = 0.1;
        let zfar: GLfloat = 100.0;
        let projection: Perspective3<GLfloat> = Perspective3::new(aspect_ratio, fovy, znear, zfar);

        if let Some(state) = scene_manager.get_level_state() {
            let view = state.camera.transform();
            let light_uniforms: Vec<PointLight> = state
                .point_lights
                .iter()
                .map(|l| l.as_light_uniforms())
                .collect();
            self.light
                .draw(&state.point_lights, view, projection.as_matrix().clone());

            self.model.draw(
                &[
                    &state.cubes[..],
                    &[state.player.model.clone()],
                    state.plane.models.as_slice(),
                ]
                .concat(),
                &light_uniforms,
                &state.dir_lights,
                &state.camera.position().into(),
                view,
                projection.as_matrix().clone(),
                &self.models,
                &self.materials,
            );
        }

        if let Some(progress) = scene_manager.get_progress_bar() {
            self.progress.draw(progress);
        }
    }

    fn loading_update(&mut self) {
        // Models
        if !self.loading_models.is_empty() {
            while let Ok((model_name, res)) = self.model_rec.try_recv() {
                self.loading_models.remove(&model_name);
                let materials = match res {
                    Ok(Model { meshes, materials }) => {
                        let ms: Vec<String> = meshes.iter().map(|m| m.name.to_string()).collect();
                        self.model.load_meshes(meshes);
                        self.models.insert(model_name, ms);
                        materials
                    }
                    Err(e) => {
                        error!(
                            error = e.backtrace().to_string(),
                            model = &model_name,
                            "Failed to load model"
                        );
                        continue;
                    }
                };

                self.load_materials(materials);
            }
        }

        // Materials
        if !self.loading_material_files.is_empty() {
            while let Ok((material_name, res)) = self.material_rec.try_recv() {
                self.loading_material_files.remove(&material_name);
                let textures = match res {
                    Ok(mats) => {
                        let mut textures = Vec::new();
                        for mat in mats {
                            textures.push(mat.diffuse_map.clone());
                            textures.push(mat.specular_map.clone());
                            self.materials.insert(mat.name.clone(), mat);
                        }
                        self.material_files.insert(material_name);
                        textures
                    }
                    Err(e) => {
                        let error: String = e.chain().map(|s| s.to_string()).collect();
                        error!(
                            error = error,
                            material = &material_name,
                            "Failed to load material"
                        );
                        continue;
                    }
                };

                self.load_textures(textures);
            }
        }

        // Textures
        if !self.loading_textures.is_empty() {
            while let Ok((texture_name, res)) = self.texture_rec.try_recv() {
                self.loading_textures.remove(&texture_name);
                match res {
                    Ok(texture) => {
                        self.model.load_texture(texture);
                        self.textures.insert(texture_name);
                    }
                    Err(e) => {
                        error!(
                            error = e.to_string(),
                            texture = &texture_name,
                            "Failed to load Texture"
                        );
                        continue;
                    }
                };
            }
        }
    }
}

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

pub enum RenderMessage {
    Load(String),
}
