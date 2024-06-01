mod cube_renderer;
mod plane_renderer;
mod point_light_renderer;

use std::f32::consts::PI;
use std::path::PathBuf;

use cube_renderer::CubeRenderer;
use gl;
use gl::types::*;
use na::Perspective3;
use plane_renderer::PlaneRenderer;
use point_light_renderer::PointLightRenderer;

use crate::shader::PointLight;
use crate::state::GameState;

use super::config::{
    CUBE_VERT_SHADER, LIGHT_FRAG_SHADER, LIGHT_VERT_SHADER, PLANE_VERT_SHADER, TEXTURE_FRAG_SHADER,
    WALL_TEXTURE,
};
use super::shape::{
    CUBE_INDICES, CUBE_VERTICES, QUAD_INDICES, QUAD_VERTICES, TEXTURED_CUBE_INDICES,
    TEXTURED_CUBE_VERTICES,
};

pub struct Renderer {
    light: PointLightRenderer,
    cube: CubeRenderer,
    plane: PlaneRenderer,
    projection: Perspective3<GLfloat>,
}

impl Renderer {
    pub fn new(window_width: u32, window_height: u32) -> Self {
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
        let texture = image::open(WALL_TEXTURE).unwrap();
        let cube = CubeRenderer::new(
            &cube_vert_shader,
            &texture_frag_shader,
            &TEXTURED_CUBE_VERTICES,
            &TEXTURED_CUBE_INDICES,
            texture,
        )
        .unwrap();

        let plane_vert_shader = PathBuf::from(PLANE_VERT_SHADER);
        let texture = image::open(WALL_TEXTURE).unwrap();
        let plane = PlaneRenderer::new(
            &plane_vert_shader,
            &texture_frag_shader,
            &QUAD_VERTICES,
            &QUAD_INDICES,
            texture,
        )
        .unwrap();

        Self {
            light,
            cube,
            plane,
            projection,
        }
    }

    pub fn render(&self, state: &GameState) {
        clear();
        let view = state.camera.transform();
        let light_uniforms: Vec<PointLight> =
            state.point_lights.iter().map(|l| l.as_light_uniforms()).collect();
        self.light
            .draw(&state.point_lights, view, self.projection.as_matrix().clone());
        self.cube.draw(
            &state.cubes,
            &light_uniforms,
            &state.dir_lights,
            &state.camera.position().into(),
            view,
            self.projection.as_matrix().clone(),
        );
        self.cube.draw(
            &[state.player.cube],
            &light_uniforms,
            &state.dir_lights,
            &state.camera.position().into(),
            view,
            self.projection.as_matrix().clone(),
        );

        self.plane.draw(
            &[state.plane],
            &light_uniforms,
            &state.dir_lights,
            &state.camera.position().into(),
            view,
            self.projection.as_matrix().clone(),
        );
    }
}

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
