mod audio;
mod camera;
mod config;
mod controller;
mod file_utils;
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

use audio::{Audio, AudioBuilder};
use controller::{Button, Controller};
use glfw::Context;
use render::Renderer;
use state::{GameState, Status};
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

    // Program Setup
    let sad_level = "assets/maps/sad";
    let (renderer, model_objects) = Renderer::new(window_width, window_height);
    let mut state = GameState::new(&sad_level.into(), &model_objects);
    let mut last_status = state.status;
    let mut controller = Controller::new(&mut glfw, events);
    let mut audio_builder = AudioBuilder::new();
    let sad_level_song = file_utils::get_level_file(&sad_level.into(), ".wav");
    let death_track = audio_builder.add_wav(&"assets/sounds/test.wav".into());
    let sad_song_track = audio_builder.add_wav(&sad_level_song);
    let upbeat_level = "assets/maps/upbeat";
    let upbeat_level_song = file_utils::get_level_file(&upbeat_level.into(), ".wav");
    let upbeat_track: usize = audio_builder.add_wav(&upbeat_level_song);
    let audio = audio_builder.build();
    let mut current_track = sad_song_track;

    let mut last_time = Instant::now();
    audio.track_action(audio::Action::Play(sad_song_track));
    while !window.should_close() {
        window.swap_buffers();

        let mut current_time = Instant::now();
        let mut delta_time = current_time.duration_since(last_time);
        last_time = current_time;

        controller.poll_input(&mut window);
        for button in controller.buttons() {
            match button {
                Button::Quit => window.set_should_close(true),
                Button::Level1 => {
                    state = GameState::new(&sad_level.into(), &model_objects);
                    audio.track_action(audio::Action::Stop(upbeat_track));
                    current_track = sad_song_track;
                    audio.track_action(audio::Action::Play(sad_song_track));
                },
                Button::Level2 => {
                    state = GameState::new(&upbeat_level.into(), &model_objects);
                    audio.track_action(audio::Action::Stop(sad_song_track));
                    let track = upbeat_track;
                    last_time = Instant::now();
                    current_time = Instant::now();
                    delta_time = current_time.duration_since(last_time);
                    last_time = current_time;
                    current_track = track;
                    audio.track_action(audio::Action::Play(track));
                },
                _ => (),
            }
        }

        state.update(delta_time, &controller);
        if state.status.clone() != last_status {
            match state.status {
                Status::Alive => {
                    audio.track_action(audio::Action::Play(current_track));
                },
                Status::Dead => {
                    audio.track_action(audio::Action::Slow(current_track));
                    audio.track_action(audio::Action::Play(death_track));
                },
                Status::Resetting => {
                    audio.track_action(audio::Action::Reset(current_track));
                },
                Status::Paused(_) => {
                    audio.track_action(audio::Action::Stop(current_track));
                },
            }
        }
        renderer.render(&state);
        last_status = state.status;
    }
}
