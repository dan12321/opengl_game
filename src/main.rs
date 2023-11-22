mod camera;
mod config;
mod controller;
mod light;
mod model;
mod shader;
mod shape;

extern crate glfw;
extern crate image;
extern crate nalgebra as na;
extern crate tracing;
extern crate tracing_subscriber;

use std::f32::consts::PI;
use std::time::Instant;

use camera::Camera;
use controller::{Button, Controller};
use gl::types::*;
use glfw::{Action, Context, Key};
use model::ModelBuilder;
use na::{vector, Perspective3, Rotation3, Translation3, Unit, Vector3};
use rand::Rng;
use shader::Shader;
use shape::{TEXTURED_CUBE_INDICES, TEXTURED_CUBE_VERTICES};
use tracing::{debug, Level};

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
    let light_position_uniform = "lightPosition";
    let light_color_uniform = "lightColor";
    let light_strenght_uniform = "lightStrength";
    let ambient_color_uniform = "ambientColor";
    let ambient_color_intensity_uniform = "ambientColorIntensity";
    let camera_position_uniform = "cameraPosition";
    let specular_strength_uniform = "specularStrength";
    let shininess_uniform = "shininess";
    let aspect_ratio: GLfloat = window_width as GLfloat / window_height as GLfloat;
    let fovy: GLfloat = PI / 2.0;
    let znear: GLfloat = 0.1;
    let zfar: GLfloat = 100.0;
    let projection: Perspective3<GLfloat> = Perspective3::new(aspect_ratio, fovy, znear, zfar);
    let mut camera = Camera::new(10.0, 0.0, -0.5, vector![0.0, 0.0, 0.0]);
    let mut view = camera.transform();
    let light_shader = Shader::new(config::LIGHT_VERT_SHADER, config::LIGHT_FRAG_SHADER).unwrap();
    let mut light_model =
        model::ModelBuilder::new(shape::CUBE_VERTICES, shape::CUBE_INDICES, light_shader)
            .add_transform(Translation3::new(0.0, 2.0, 0.0))
            .set_scale(0.3)
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
    let mut light = light::Light::new(light_model, 0.2, 1.0, 0.8, 50.0);
    let texture_shader_program =
        Shader::new(config::TEXTURE_VERT_SHADER, config::TEXTURE_FRAG_SHADER).unwrap();
    let mut model = ModelBuilder::new(
        TEXTURED_CUBE_VERTICES.into(),
        TEXTURED_CUBE_INDICES.into(),
        texture_shader_program,
    )
    .add_texture(String::from(config::WALL_TEXTURE))
    .add_normals()
    .init()
    .add_uniform_mat4(projection_uniform, projection.as_matrix().clone())
    .unwrap()
    .add_uniform_mat4(view_uniform, view)
    .unwrap()
    .add_uniform3f(light_color_uniform, light.color.into())
    .unwrap()
    .add_uniform3f(
        light_position_uniform,
        (
            light.model.transform.x,
            light.model.transform.y,
            light.model.transform.z,
        ),
    )
    .unwrap()
    .add_uniform1f(light_strenght_uniform, light.strength)
    .unwrap()
    .add_uniform3f(ambient_color_uniform, (1.0, 1.0, 1.0))
    .unwrap()
    .add_uniform1f(ambient_color_intensity_uniform, 0.1)
    .unwrap()
    .add_uniform1i(shininess_uniform, 32)
    .unwrap()
    .add_uniform1f(specular_strength_uniform, 0.5)
    .unwrap()
    .add_uniform3f(camera_position_uniform, camera.position())
    .unwrap();
    let world_space_operation = model.world_space_operation();
    model = model
        .add_uniform_mat4(transformation_uniform, world_space_operation)
        .unwrap();

    let mut rng = rand::thread_rng();
    let mut random_offsets = [0.0; 10];
    for i in 0..random_offsets.len() {
        random_offsets[i] = rng.gen_range(0.0..4321.0);
    }

    let start = Instant::now();
    let mut last_time = start;

    let mut controller = Controller::new(&mut glfw, events);

    while !window.should_close() {
        window.swap_buffers();
        controller.poll_input();
        let current_time = Instant::now();
        let time_delta = current_time.duration_since(last_time);
        let move_step_size = config::MOVE_SPEED * time_delta.as_secs_f32();
        for button in controller.buttons() {
            match button {
                Button::Quit => window.set_should_close(true),
            }
        }
        let (x, y) = controller.direction();
        model.transform =
            Translation3::new(x * move_step_size, 0.0, y * move_step_size) * model.transform;
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
        camera.target = model.transform.vector;
        view = camera.transform();
        for i in 0..random_offsets.len() {
            let now = Instant::now().duration_since(start).as_secs_f32();
            let time_offset = if i % 3 == 0 { now } else { 0.0 };
            let axis: Unit<Vector3<f32>> = Unit::new_normalize(vector![
                random_offsets[i] % 12.0,
                random_offsets[i] % 11.0,
                random_offsets[i] % 3.0
            ]);
            let rotation_offset =
                Rotation3::from_axis_angle(&axis, (random_offsets[i] + time_offset) % 111 as f32)
                    .to_homogeneous();
            let transformation_offset = Translation3::new(
                (random_offsets[i] % 19.0) - 9.5,
                (random_offsets[i] % 21.0) - 10.5,
                -random_offsets[i] % 31.0,
            )
            .to_homogeneous();
            if i == 0 {
                model
                    .set_uniform_mat4(transformation_uniform, model.world_space_operation())
                    .unwrap();
            } else {
                model
                    .set_uniform_mat4(
                        transformation_uniform,
                        transformation_offset * rotation_offset,
                    )
                    .unwrap();
            }
            model
                .set_uniform_mat4(projection_uniform, projection.as_matrix().clone())
                .unwrap();
            model.set_uniform_mat4(view_uniform, view).unwrap();
            model
                .set_uniform3f(light_color_uniform, light.color.into())
                .unwrap();
            model
                .set_uniform3f(
                    light_position_uniform,
                    (
                        light.model.transform.x,
                        light.model.transform.y,
                        light.model.transform.z,
                    ),
                )
                .unwrap();
            model
                .set_uniform1f(light_strenght_uniform, light.strength)
                .unwrap();
            model
                .set_uniform3f(camera_position_uniform, camera.position())
                .unwrap();
            model.draw();
        }
        light.model.set_uniform_mat4(view_uniform, view).unwrap();
        light
            .model
            .set_uniform_mat4(transformation_uniform, light.model.world_space_operation())
            .unwrap();
        light
            .model
            .set_uniform3f(color_uniform, light.color.into())
            .unwrap();
        light.model.draw();
        last_time = current_time;
    }
}

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
