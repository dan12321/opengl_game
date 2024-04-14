mod camera;
mod config;
mod controller;
mod light;
mod model;
mod render;
mod shader;
mod shape;
mod state;

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
use model::{Material, ModelBuilder};
use na::{vector, Matrix4, Perspective3, Rotation3, Translation3};
use rand::Rng;
use render::cube_renderer::CubeRenderer;
use render::plane_renderer::PlaneRenderer;
use render::spot_light_renderer::SpotLightRenderer;
use shader::Shader;
use shape::{CUBE_INDICES, CUBE_VERTICES, TEXTURED_CUBE_INDICES, TEXTURED_CUBE_VERTICES, QUAD_VERTICES, QUAD_INDICES};
use state::{Cube, Light, Transform, Plane};
use tracing::Level;

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
    let transformation_uniform = "transformation";
    let projection_uniform = "projection";
    let view_uniform = "view";
    let camera_position_uniform = "cameraPosition";
    let offset_uniform = "offset";
    let aspect_ratio: GLfloat = window_width as GLfloat / window_height as GLfloat;
    let fovy: GLfloat = PI / 2.0;
    let znear: GLfloat = 0.1;
    let zfar: GLfloat = 100.0;
    let projection: Perspective3<GLfloat> = Perspective3::new(aspect_ratio, fovy, znear, zfar);
    let mut camera = Camera::new(10.0, 0.0, -0.5, vector![0.0, 0.0, 0.0]);
    let mut view = camera.transform();

    let plat_width = 30.0;
    let light_vert_shader = PathBuf::from(config::LIGHT_VERT_SHADER);
    let light_frag_shader = PathBuf::from(config::LIGHT_FRAG_SHADER);
    let light_renderer = SpotLightRenderer::new(
        &light_vert_shader,
        &light_frag_shader,
        &CUBE_VERTICES,
        &CUBE_INDICES,
    )
    .unwrap();

    let mut lights = Vec::with_capacity(64);
    for i in 0..=4 {
        let n = i as f32;
        let x = (n / 4.0) * plat_width - (plat_width / 2.0);
        let z = 2.0 * (-n * n + 4.0 * n);
        let light1_transform = Transform {
            position: (x, z, -10.0).into(),
            scale: (0.5, 0.5, 0.5).into(),
            rotation: Matrix4::identity(),
        };
        let light2_transform = Transform {
            position: (x, z, -50.0).into(),
            scale: (0.5, 0.5, 0.5).into(),
            rotation: Matrix4::identity(),
        };
        let light3_transform = Transform {
            position: (x, z, -90.0).into(),
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
    let mut cube_list = Vec::with_capacity(64);
    let mut player_cube = Cube {
        transform: state::Transform {
            position: (0.0, 0.0, 0.0).into(),
            scale: (1.0, 1.0, 1.0).into(),
            rotation: Matrix4::identity(),
        },
        material: Material::new((0.1, 0.1, 0.1), (1.0, 1.0, 1.0), (0.7, 0.7, 0.7), 128),
    };
    let cube_material = Material::new((0.5, 0.1, 0.1), (1.0, 0.7, 0.7), (1.0, 0.7, 0.7), 128);
    let positions = [(5.0, 0.0, -10.0), (-5.0, 0.0, -10.0), (0.0, 0.0, -15.0)];
    for position in positions {
        cube_list.push(Cube {
            transform: state::Transform {
                position: position.into(),
                scale: (2.0, 1.0, 1.0).into(),
                rotation: Matrix4::identity(),
            },
            material: cube_material,
        });
    }

    let plane_vert_shader = PathBuf::from(config::PLANE_VERT_SHADER);
    let texture = image::open(config::WALL_TEXTURE).unwrap();
    let plane_renderer = PlaneRenderer::new(
        &plane_vert_shader,
        &texture_frag_shader,
        &QUAD_VERTICES,
        &QUAD_INDICES,
        texture,
    ).unwrap();
    let plane_material = Material::new((0.2, 0.2, 0.1), (0.5, 0.5, 0.2), (0.5, 0.5, 0.4), 64);
    let plat_length = 100.0;
    let mut plane = Plane {
        transform: Transform {
            position: (0.0, -0.5, 0.0).into(),
            scale: (plat_width, plat_length, 0.0).into(),
            rotation: Rotation3::from_euler_angles(1.570796, 0.0, 0.0).to_homogeneous()
        },
        material: plane_material,
        offset: 0.0,
    };

    let mut rng = rand::thread_rng();
    let mut random_offsets = [0.0; 10];
    for i in 0..random_offsets.len() {
        random_offsets[i] = rng.gen_range(0.0..4321.0);
    }

    let start = Instant::now();
    let mut last_time = start;

    let mut controller = Controller::new(&mut glfw, events);
    let mut y_pos = 0.0;
    while !window.should_close() {
        window.swap_buffers();
        controller.poll_input();
        let current_time = Instant::now();
        let time_since_start = current_time.duration_since(start).as_secs_f32();
        let time_delta = current_time.duration_since(last_time);
        let move_step_size = config::MOVE_SPEED * time_delta.as_secs_f32();
        let treadmill_speed = time_since_start / 100.0;
        for button in controller.buttons() {
            match button {
                Button::Quit => window.set_should_close(true),
            }
        }
        let (x, _) = controller.direction();
        y_pos -= treadmill_speed;
        player_cube.transform.position.x += x * move_step_size;
        let (cx, cy, zoom) = controller.mouse();
        let min_cy = config::MIN_CAMERA_LONGITUDE / config::CURSOR_MOVEMENT_SCALE;
        let max_cy = config::MAX_CAMERA_LONGITUDE / config::CURSOR_MOVEMENT_SCALE;
        let cy_min_clamped = if cy < min_cy { min_cy } else { cy };
        let cy_clamped = if cy_min_clamped > max_cy {
            max_cy
        } else {
            cy_min_clamped
        };
        camera.latitude = cx as f32 * config::CURSOR_MOVEMENT_SCALE;
        camera.longitude = -cy_clamped * config::CURSOR_MOVEMENT_SCALE;
        window.set_cursor_pos(cx as f64, cy_clamped as f64);
        camera.distance = camera.default_distance + zoom;

        clear();
        camera.target = Translation3::new(
            player_cube.transform.position.x,
            player_cube.transform.position.y,
            player_cube.transform.position.z,
        )
        .vector;
        view = camera.transform();
        let mut lights_to_render = Vec::with_capacity(lights.len());
        let mut light_uniforms = Vec::with_capacity(64);
        for light in &mut lights {
            if light.transform.position.z > y_pos + 50.0 {
                light.transform.position.z = y_pos - 50.0;
            }
            let mut l = light.clone();
            l.transform.position.z -= y_pos;
            lights_to_render.push(l);
            light_uniforms.push(l.as_light_uniforms());
        }
        light_renderer.draw(&lights_to_render, view, projection.as_matrix().clone());
        let camera_position = camera.position();
        let mut cubes_to_render = Vec::with_capacity(cube_list.len());
        for cube in &mut cube_list {
            if cube.transform.position.z > y_pos + 20.0 {
                cube.transform.position.z = y_pos - 50.0;
                cube.transform.position.x = rng.gen_range(0.0..plat_width) - 15.0;
            }
            let mut c = cube.clone();
            c.transform.position.z -= y_pos;
            cubes_to_render.push(c);
        }
        cubes_to_render.push(player_cube);
        cube_renderer.draw(
            &cubes_to_render,
            &light_uniforms,
            &camera_position.into(),
            view,
            projection.as_matrix().clone(),
        );

        plane.offset = y_pos / plat_length;
        plane_renderer.draw(
            &[plane],
            &light_uniforms,
            &camera_position.into(),
            view,
            projection.as_matrix().clone(),
        );
        last_time = current_time;
    }
}

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
