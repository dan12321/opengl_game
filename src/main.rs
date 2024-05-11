mod camera;
mod config;
mod controller;
mod light;
mod model;
mod physics;
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

use controller::{Button, Controller};
use glfw::Context;
use render::Renderer;
use state::GameState;
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
    let mut state = GameState::new();
    let mut controller = Controller::new(&mut glfw, events);
    let renderer = Renderer::new(window_width, window_height);

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
        renderer.render(&state);
    }
}
