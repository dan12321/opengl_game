mod camera;
mod config;
mod controller;
mod light;
mod model;
mod render;
mod shader;
mod shape;
mod state;
mod physics;

extern crate glfw;
extern crate image;
extern crate nalgebra as na;
extern crate tracing;
extern crate tracing_subscriber;

use std::time::Instant;
use std::{f32::consts::PI, path::PathBuf};

use camera::Camera;
use controller::{Button, Controller};
use gl::types::*;
use glfw::Context;
use light::LightUniform;
use model::Material;
use na::{vector, Matrix4, Perspective3, Rotation3, Translation3};
use physics::AABBColider;
use rand::Rng;
use render::cube_renderer::CubeRenderer;
use render::plane_renderer::PlaneRenderer;
use render::spot_light_renderer::SpotLightRenderer;
use shape::{CUBE_INDICES, CUBE_VERTICES, TEXTURED_CUBE_INDICES, TEXTURED_CUBE_VERTICES, QUAD_VERTICES, QUAD_INDICES};
use state::{Cube, Light, Transform, Plane, GameState};
use tracing::{Level, debug};

fn main() {
    // Log setup
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt().init();
    }

    // Window Setup
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    let window_width = 900;
    let window_height = 900;
    let (mut window, events) = glfw
        .create_window(
            window_width,
            window_height,
            "Hello Window",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_resizable(false);
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.set_raw_mouse_motion(true);
    window.set_scroll_polling(true);

    // OpenGL Setup
    gl::load_with(|s| window.get_proc_address(s));
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    // Program Setup
    let aspect_ratio: GLfloat = window_width as GLfloat / window_height as GLfloat;
    let fovy: GLfloat = PI / 2.0;
    let znear: GLfloat = 0.1;
    let zfar: GLfloat = 100.0;
    let projection: Perspective3<GLfloat> = Perspective3::new(aspect_ratio, fovy, znear, zfar);

    let light_vert_shader = PathBuf::from(config::LIGHT_VERT_SHADER);
    let light_frag_shader = PathBuf::from(config::LIGHT_FRAG_SHADER);
    let light_renderer = SpotLightRenderer::new(
        &light_vert_shader,
        &light_frag_shader,
        &CUBE_VERTICES,
        &CUBE_INDICES,
    )
    .unwrap();

    let cube_vert_shader = PathBuf::from(config::CUBE_VERT_SHADER);
    let texture_frag_shader = PathBuf::from(config::TEXTURE_FRAG_SHADER);
    let texture = image::open(config::WALL_TEXTURE).unwrap();
    let cube_renderer = CubeRenderer::new(
        &cube_vert_shader,
        &texture_frag_shader,
        &TEXTURED_CUBE_VERTICES,
        &TEXTURED_CUBE_INDICES,
        texture,
    )
    .unwrap();

    let plane_vert_shader = PathBuf::from(config::PLANE_VERT_SHADER);
    let texture = image::open(config::WALL_TEXTURE).unwrap();
    let plane_renderer = PlaneRenderer::new(
        &plane_vert_shader,
        &texture_frag_shader,
        &QUAD_VERTICES,
        &QUAD_INDICES,
        texture,
    ).unwrap();

    let mut state = GameState::new();
    let mut controller = Controller::new(&mut glfw, events);

    let mut last_time = Instant::now();
    while !window.should_close() {
        window.swap_buffers();

        let current_time = Instant::now();
        let delta_time = current_time.duration_since(last_time);
        last_time = current_time;

        controller.poll_input(&mut window);
        for button in controller.buttons() {
            match button {
                Button::Quit => window.set_should_close(true),
            }
        }

        state.update(delta_time, &controller);
        
        // render
        clear();
        let view = state.camera.transform();
        let light_uniforms: Vec<LightUniform> = state.lights.iter()
            .map(|l| l.as_light_uniforms())
            .collect();
        //light_uniforms.push(l.as_light_uniforms());
        light_renderer.draw(&state.lights, view, projection.as_matrix().clone());
        cube_renderer.draw(
            &state.cubes,
            &light_uniforms,
            &state.camera.position().into(),
            view,
            projection.as_matrix().clone(),
        );
        cube_renderer.draw(
            &[state.player.cube],
            &light_uniforms,
            &state.camera.position().into(),
            view,
            projection.as_matrix().clone(),
        );

        plane_renderer.draw(
            &[state.plane],
            &light_uniforms,
            &state.camera.position().into(),
            view,
            projection.as_matrix().clone(),
        );
    }
}

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
