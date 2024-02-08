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

use std::io::Read;
use std::time::Instant;
use std::collections::VecDeque;
use std::{f32::consts::PI, path::PathBuf};

use camera::Camera;
use controller::{Button, Controller};
use gl::types::*;
use glfw::Context;
use model::{Material, ModelBuilder};
use na::{vector, Matrix4, Perspective3, Rotation3, Translation3, Unit, Vector3};
use rand::Rng;
use render::box_renderer::CubeRenderer;
use shader::Shader;
use shape::{TEXTURED_CUBE_INDICES, TEXTURED_CUBE_VERTICES};
use state::Cube;
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
    let transformation_uniform = "transformation";
    let projection_uniform = "projection";
    let view_uniform = "view";
    let color_uniform = "color";
    let camera_position_uniform = "cameraPosition";
    let offset_uniform = "offset";
    let aspect_ratio: GLfloat = window_width as GLfloat / window_height as GLfloat;
    let fovy: GLfloat = PI / 2.0;
    let znear: GLfloat = 0.1;
    let zfar: GLfloat = 100.0;
    let projection: Perspective3<GLfloat> = Perspective3::new(aspect_ratio, fovy, znear, zfar);
    let mut camera = Camera::new(10.0, 0.0, -0.5, vector![0.0, 0.0, 0.0]);
    let mut view = camera.transform();

    let light_shader = Shader::new(config::LIGHT_VERT_SHADER, config::LIGHT_FRAG_SHADER).unwrap();
    let light_start_position = (0.0, 2.0, 0.0);
    let mut light_model =
        model::ModelBuilder::new(shape::CUBE_VERTICES, shape::CUBE_INDICES, light_shader)
            .add_transform(Translation3::new(
                light_start_position.0,
                light_start_position.1,
                light_start_position.2,
            ))
            .set_scale((0.3, 0.3, 0.3))
            .init()
            .add_uniform3f(color_uniform, (0.0, 1.0, 0.0))
            .unwrap()
            .add_uniform_mat4(projection_uniform, projection.as_matrix().clone())
            .unwrap()
            .add_uniform_mat4(view_uniform, view)
            .unwrap();
    let light_wso = light_model.world_space_operation();
    light_model = light_model
        .add_uniform_mat4(transformation_uniform, light_wso)
        .unwrap();
    let mut light = light::Light::new(light_model, (0.2, 1.0, 0.8), (0.2, 1.0, 0.8), 50.0);

    let light2_shader = Shader::new(config::LIGHT_VERT_SHADER, config::LIGHT_FRAG_SHADER).unwrap();
    let light2_start_position = (0.0, 0.0, -10.0);
    let mut light2_model =
        model::ModelBuilder::new(shape::CUBE_VERTICES, shape::CUBE_INDICES, light2_shader)
            .add_transform(Translation3::new(
                light2_start_position.0,
                light2_start_position.1,
                light2_start_position.2,
            ))
            .set_scale((0.3, 0.3, 0.3))
            .init()
            .add_uniform3f(color_uniform, (1.0, 0.0, 1.0))
            .unwrap()
            .add_uniform_mat4(projection_uniform, projection.as_matrix().clone())
            .unwrap()
            .add_uniform_mat4(view_uniform, view)
            .unwrap();
    let light2_wso = light2_model.world_space_operation();
    light2_model = light2_model
        .add_uniform_mat4(transformation_uniform, light2_wso)
        .unwrap();
    let mut light2 = light::Light::new(light2_model, (1.0, 0.0, 1.0), (1.0, 0.0, 1.0), 80.0);

    let cube_vert_shader = PathBuf::from(config::CUBE_VERT_SHADER);
    let texture_frag_shader = PathBuf::from(config::TEXTURE_FRAG_SHADER);
    let texture = image::open(config::WALL_TEXTURE).unwrap();
    let cube_renderer = CubeRenderer::new(
        &cube_vert_shader,
        &texture_frag_shader,
        &TEXTURED_CUBE_VERTICES,
        &TEXTURED_CUBE_INDICES,
        texture,
    ).unwrap();
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
    let positions = [
        (5.0, 0.0, -10.0),
        (-5.0, 0.0, -10.0),
        (0.0, 0.0, -15.0),
    ];
    for position in positions {
        cube_list.push(Cube {
            transform: state::Transform {
                position: position.into(),
                scale: (1.0, 1.0, 1.0).into(),
                rotation: Matrix4::identity(),
            },
            material: cube_material,
        });
    }

    let plane_shader_program =
        Shader::new(config::PLANE_VERT_SHADER, config::TEXTURE_FRAG_SHADER).unwrap();
    let plane_material = Material::new((0.2, 0.2, 0.1), (0.5, 0.5, 0.2), (0.5, 0.5, 0.4), 64);
    let mut plane = ModelBuilder::new(
        shape::QUAD_VERTICES.into(),
        shape::QUAD_INDICES.into(),
        plane_shader_program,
    )
    .add_texture(String::from(config::WALL_TEXTURE))
    .set_scale((30.0, 100.0, 0.0))
    .set_rotation(Rotation3::from_euler_angles(1.570796, 0.0, 0.0).to_homogeneous())
    .add_transform(Translation3::new(0.0, -0.5, 0.0))
    .init()
    .add_uniform_mat4(projection_uniform, projection.as_matrix().clone())
    .unwrap()
    .add_uniform_mat4(view_uniform, view)
    .unwrap()
    .add_light(light.as_light_uniforms())
    .unwrap()
    .add_light(light2.as_light_uniforms())
    .unwrap()
    .set_material(plane_material)
    .unwrap()
    .add_uniform3f(camera_position_uniform, camera.position())
    .unwrap()
    .add_uniform1f(offset_uniform, 0.0)
    .unwrap();
    let plane_space = plane.world_space_operation();
    plane = plane
        .add_uniform_mat4(transformation_uniform, plane_space)
        .unwrap();

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
        let treadmil_speed = time_since_start / 100.0;
        for button in controller.buttons() {
            match button {
                Button::Quit => window.set_should_close(true),
            }
        }
        let (x, y) = controller.direction();
        y_pos -= treadmil_speed;
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

        light.model.transform = Translation3::new(
            light_start_position.0 + (time_since_start.sin() * 4.0),
            light_start_position.1,
            light_start_position.2 - (time_since_start.cos() * 8.0 + 8.0) - y_pos,
        );

        light2.model.transform.z = -y_pos;

        light.diffuse.0 = time_since_start.sin();
        light.specular.0 = time_since_start.sin();

        clear();
        camera.target = Translation3::new(
            player_cube.transform.position.x,
            player_cube.transform.position.y,
            player_cube.transform.position.z,
        ).vector;
        view = camera.transform();
        let lights = vec![light.as_light_uniforms(), light2.as_light_uniforms()];
        let camera_position = camera.position();
        let mut cubes_to_render = Vec::with_capacity(cube_list.len());
        for cube in &mut cube_list {

            if cube.transform.position.z > y_pos + 20.0 {
                cube.transform.position.z = y_pos - 50.0;
            }
            let mut c = cube.clone();
            c.transform.position.z -= y_pos;
            cubes_to_render.push(c);
        }
        cubes_to_render.push(player_cube);
        cube_renderer.draw(&cubes_to_render, &lights, &camera_position.into(), view, projection.as_matrix().clone());
        plane.set_uniform_mat4(view_uniform, view).unwrap();
        plane.set_light(0, light.as_light_uniforms());
        plane.set_light(1, light2.as_light_uniforms());
        plane
            .set_uniform3f(camera_position_uniform, camera.position())
            .unwrap();
        plane.set_uniform1f(offset_uniform, y_pos / plane.scale.1);
        plane.draw();
        light.model.set_uniform_mat4(view_uniform, view).unwrap();
        light
            .model
            .set_uniform_mat4(transformation_uniform, light.model.world_space_operation())
            .unwrap();
        light
            .model
            .set_uniform3f(color_uniform, light.specular.into())
            .unwrap();
        light.model.draw();
        light2.model.set_uniform_mat4(view_uniform, view).unwrap();
        light2
            .model
            .set_uniform_mat4(transformation_uniform, light2.model.world_space_operation())
            .unwrap();
        light2
            .model
            .set_uniform3f(color_uniform, light2.specular.into())
            .unwrap();
        light2.model.draw();
        last_time = current_time;
    }
}

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
