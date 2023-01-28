mod camera;
mod model;
mod shader;

extern crate glfw;
extern crate image;
extern crate nalgebra as na;
extern crate tracing;
extern crate tracing_subscriber;

use std::cmp::{max, min};
use std::f32::consts::PI;
use std::time::Instant;
use std::{
    mem,
    ptr,
};
use std::ffi::{c_void};

use camera::Camera;
use gl::types::*;
use glfw::{Action, Context, Key};
use model::{Model, INDICESA, VERTICESA, ModelBuilder};
use na::{Matrix4, Vector3, Rotation3, Projective3, Perspective3, Translation3, vector, Unit};
use rand::Rng;
use shader::Shader;
use tracing::{debug, Level};

const VERTICES: [GLfloat; 12] = [
    0.5,  0.5, 0.0,  // top right
    0.5, -0.5, 0.0,  // bottom right
   -0.5, -0.5, 0.0,  // bottom left
   -0.5,  0.5, 0.0,   // top left 
];


const VERTICESB: [GLfloat; 24] = [
    // Position         Color           Texture
    0.0, -1.0, 0.0,   1.0, 0.0, 0.0,  1.0, 0.0,   // bottom right
   -1.0, -1.0, 0.0,   0.0, 1.0, 0.0,  0.0, 0.0,   // bottom left
   -1.0,  0.0, 0.0,   0.0, 0.0, 1.0,  0.0, 1.0,   // top left 
];

const INDICESB: [GLuint; 3] = [
    0, 1, 2,   // second triangle
];

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
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    let window_width = 900;
    let window_height = 900;
    let (mut window, events) = glfw.create_window(window_width, window_height, "Hello Window", glfw::WindowMode::Windowed)
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

    // Program Setup
    let mut offset = 0.0;
    let offset_uniform = "offset";
    let transformation_uniform = "transformation";
    let projection_uniform = "projection";
    let view_uniform = "view";
    let aspect_ratio: GLfloat = window_width as GLfloat / window_height as GLfloat;
    let fovy: GLfloat = PI / 2.0;
    let znear: GLfloat = 0.1;
    let zfar: GLfloat = 100.0;
    let projection: Perspective3<GLfloat> = Perspective3::new(aspect_ratio, fovy, znear, zfar);
    let mut camera = Camera::new(10.0, 0.0, -0.5, vector![0.0, 0.0, 0.0]);
    let mut view = camera.transform();

    let orange_shader_program = Shader::new("assets/orange_shader.vert", "assets/orange_shader.frag").unwrap();
    let mut model = ModelBuilder::new(VERTICESA.into(), INDICESA.into(), orange_shader_program).init()
        .add_uniform1f(offset_uniform, offset)
        .unwrap()
        .add_uniform_mat4(projection_uniform, projection.as_matrix().clone())
        .unwrap()
        .add_uniform_mat4(view_uniform, view)
        .unwrap();
    let world_space_operation = model.world_space_operation();
    model = model.add_uniform_mat4(transformation_uniform, world_space_operation).unwrap();

    let mut blue_shader_program = Shader::new("assets/blue_shader.vert", "assets/blue_shader.frag").unwrap();

    // Vertex Objects
    let mut vao_blue = 0;
    let mut vbo_blue = 0;
    let mut ebo_blue = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao_blue);
        gl::GenBuffers(1, &mut vbo_blue);
        gl::GenBuffers(1, &mut ebo_blue);
        gl::BindVertexArray(vao_blue);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_blue);
        gl::BufferData(gl::ARRAY_BUFFER, (VERTICESB.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, mem::transmute(&VERTICESB), gl::STATIC_DRAW);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (INDICESB.len() * mem::size_of::<GLuint>()) as GLsizeiptr, mem::transmute(&INDICESB), gl::STATIC_DRAW);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo_blue);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<GLfloat>()) as i32, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<GLfloat>()) as i32, (3 * mem::size_of::<GLfloat>()) as *mut c_void);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<GLfloat>()) as i32, (6 * mem::size_of::<GLfloat>()) as *mut c_void);
        gl::EnableVertexAttribArray(2);
    }

    // Texture
    let mut texture1 = 0;
    let texture_path = "assets/scifiwall.jpg";
    let texture_image = image::open(texture_path).unwrap();
    // Uses native endian. Not sure if this always matches what opengl expects
    let texture_bytes = texture_image.as_bytes();

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::GenTextures(1, &mut texture1);
        gl::BindTexture(gl::TEXTURE_2D, texture1);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, texture_image.width() as i32, texture_image.height() as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, &texture_bytes[0] as *const _ as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    blue_shader_program.use_program();
    blue_shader_program = blue_shader_program.add_uniform1i("aTexture", 0)
        .unwrap()
        .add_uniform1i("bTexture", 1)
        .unwrap()
        .add_uniform1f(offset_uniform, offset)
        .unwrap();

    let mut rng = rand::thread_rng();
    let mut random_offsets = [0.0; 10];
    for i in 0..random_offsets.len() {
        random_offsets[i] = rng.gen_range(0.0..4321.0);
    }
        
    let start = Instant::now();
    while !window.should_close() {
        window.swap_buffers();
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            debug!(event = ?event, "glfw_polled_event");
            let step_size = 0.1;
            let angle_step = 0.1;
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                },
                glfw::WindowEvent::Key(Key::Up, _, Action::Repeat, _) => {
                    let new_offset = if offset + step_size > 1.0 {
                        1.0
                    } else {
                        offset + step_size
                    };
                    offset = new_offset;
                    model.transform = Translation3::new(0.0, 0.0, -step_size) * model.transform;
                },
                glfw::WindowEvent::Key(Key::Down, _, Action::Repeat, _) => {
                    let new_offset = if offset - step_size < -1.0 {
                        -1.0
                    } else {
                        offset - step_size
                    };
                    offset = new_offset;
                    model.transform = Translation3::new(0.0, 0.0, step_size) * model.transform;
                },
                glfw::WindowEvent::Key(Key::Right, _, Action::Repeat, _) => {
                    model.transform = Translation3::new(step_size, 0.0, 0.0) * model.transform;
                },
                glfw::WindowEvent::Key(Key::Left, _, Action::Repeat, _) => {
                    model.transform = Translation3::new(-step_size, 0.0, 0.0) * model.transform;
                },
                glfw::WindowEvent::Key(Key::A, _, Action::Repeat, _) => {
                    let axis = Vector3::z_axis();
                    model.rotation = model.rotation * Rotation3::from_axis_angle(&axis, angle_step).to_homogeneous();
                },
                glfw::WindowEvent::Key(Key::D, _, Action::Repeat, _) => {
                    let axis = Vector3::z_axis();
                    model.rotation = model.rotation * Rotation3::from_axis_angle(&axis, -angle_step).to_homogeneous();
                },
                glfw::WindowEvent::Key(Key::Q, _, Action::Repeat, _) => {
                    let axis = Vector3::y_axis();
                    model.rotation = model.rotation * Rotation3::from_axis_angle(&axis, angle_step).to_homogeneous();
                },
                glfw::WindowEvent::Key(Key::E, _, Action::Repeat, _) => {
                    let axis = Vector3::y_axis();
                    model.rotation = model.rotation * Rotation3::from_axis_angle(&axis, -angle_step).to_homogeneous();
                },
                glfw::WindowEvent::Key(Key::W, _, Action::Repeat, _) => {
                    let axis = Vector3::x_axis();
                    model.rotation = model.rotation * Rotation3::from_axis_angle(&axis, angle_step).to_homogeneous();
                },
                glfw::WindowEvent::Key(Key::S, _, Action::Repeat, _) => {
                    let axis = Vector3::x_axis();
                    model.rotation = model.rotation * Rotation3::from_axis_angle(&axis, -angle_step).to_homogeneous();
                },
                glfw::WindowEvent::CursorPos(x, y) => {
                    let y_min_clamped = if y < -0.3 * 360.0 {-0.3 * 360.0} else {y};
                    let y_clamped = if y_min_clamped > 1.2 * 360.0 {1.2 * 360.0} else {y_min_clamped};
                    camera.latitude = (x / 360.0) as f32;
                    camera.longitude = -(y_clamped / 360.0) as f32;
                    window.set_cursor_pos(x, y_clamped)
                },
                glfw::WindowEvent::Scroll(_, y) => {
                    let zoom = camera.distance - (y as f32 / 5.0);
                    let clamp_min = if zoom > 0.5 {zoom} else {0.5};
                    let clamp = if clamp_min < 10.0 {clamp_min} else {10.0};
                    camera.distance = clamp as f32;
                },
                _ => (),
            }
        }

        clear();
        model.shader.use_program();
        camera.target = model.transform.vector;
        view = camera.transform();
        for i in 0..random_offsets.len() {
            let now = Instant::now().duration_since(start).as_secs_f32();
            let time_offset = if i % 3 == 0 {now} else {0.0};
            let axis: Unit<Vector3<f32>> = Unit::new_normalize(vector![random_offsets[i] % 12.0, random_offsets[i] % 11.0, random_offsets[i] % 3.0]);
            let rotation_offset = Rotation3::from_axis_angle(&axis, (random_offsets[i] + time_offset) % 111 as f32).to_homogeneous();
            let transformation_offset = Translation3::new((random_offsets[i] % 19.0) - 9.5, (random_offsets[i] % 21.0) - 10.5, -random_offsets[i] % 31.0).to_homogeneous();
            model.set_uniform1f(offset_uniform, offset).unwrap();
            if i == 0 {
                model.set_uniform_mat4(transformation_uniform, model.world_space_operation()).unwrap();
            } else {
                model.set_uniform_mat4(transformation_uniform, transformation_offset * rotation_offset).unwrap();
            }
            model.set_uniform_mat4(projection_uniform, projection.as_matrix().clone()).unwrap();
            model.set_uniform_mat4(view_uniform, view).unwrap();
            model.draw();
        }

        // blue_shader_program.use_program();
        // blue_shader_program.set_uniform1f(offset_uniform, offset).unwrap();
        // unsafe {
        //     gl::ActiveTexture(gl::TEXTURE0);
        //     gl::BindTexture(gl::TEXTURE_2D, texture1);
        //     gl::ActiveTexture(gl::TEXTURE1);
        //     gl::BindTexture(gl::TEXTURE_2D, texture2);
        //     gl::BindVertexArray(vao_blue);
        //     gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (INDICESB.len() * mem::size_of::<GLuint>()) as GLsizeiptr, mem::transmute(&INDICESB), gl::STATIC_DRAW);
        //     gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        //     gl::BindVertexArray(0);
        // }
    }
}

fn clear() {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
