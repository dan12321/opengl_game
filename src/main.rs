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
mod resource;

extern crate glfw;
extern crate image;
extern crate nalgebra as na;
extern crate tracing;
extern crate tracing_subscriber;

use std::{sync::Arc, time::Instant};

use audio::{Audio, AudioEvent};
use controller::{Button, Controller};
use glfw::Context;
use render::Renderer;
use resource::manager::ResourceManager;
use state::{GameState, Status};
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

    // Program Setup
    let (renderer, model_objects) = Renderer::new(window_width, window_height);
    debug!("Renderer loaded");
    let sad_level = "assets/maps/sad";
    let mut state = GameState::new(&sad_level.into(), &model_objects);
    let mut last_status = state.status;

    debug!("Game state loaded");
    let mut controller = Controller::new(&mut glfw, events);
    debug!("Controller loaded");
    let resources = Arc::new(ResourceManager::new());

    let mut audio = Audio::new(resources.clone());
    debug!("Audio loaded");

    let death_track = "test.wav";
    let base_audio_tracks = vec![death_track.to_string()];
    audio.load_wavs(&base_audio_tracks);

    let sad_song = "sad_melodica.wav";
    let songs = vec![sad_song.to_string()];
    audio.load_wavs(&songs);
    let mut audio_events = Vec::new();
    while !audio_events.contains(&AudioEvent::Loaded) {
        audio_events = audio.update();
    }

    let mut last_time = Instant::now();
    audio.track_action(audio::Action::Play(sad_song.to_string()));
    while !window.should_close() {
        window.swap_buffers();

        let current_time = Instant::now();
        let delta_time = current_time.duration_since(last_time);
        last_time = current_time;

        controller.poll_input(&mut window);
        for button in controller.buttons() {
            match button {
                Button::Quit => window.set_should_close(true),
                // Button::Level1 => {
                //     state = GameState::new(&sad_level.into(), &model_objects);
                //     audio.track_action(audio::Action::Stop(sad_song.into()));
                // },
                // Button::Level2 => {
                //     state = GameState::new(&sad_level.into(), &model_objects);
                //     audio.track_action(audio::Action::Play(sad_song.into()));
                //     // let track = upbeat_track;
                //     // last_time = Instant::now();
                //     // current_time = Instant::now();
                //     // delta_time = current_time.duration_since(last_time);
                //     // last_time = current_time;
                //     // current_track = track;
                //     // audio.track_action(audio::Action::Play(track));
                // },
                _ => (),
            }
        }

        state.update(delta_time, &controller);
        if state.status.clone() != last_status {
            match state.status {
                Status::Alive => {
                    audio.track_action(audio::Action::Play(sad_song.into()));
                },
                Status::Dead => {
                    audio.track_action(audio::Action::Slow(sad_song.into()));
                    audio.track_action(audio::Action::Play(death_track.into()));
                },
                Status::Resetting => {
                    audio.track_action(audio::Action::Reset(sad_song.into()));
                },
                Status::Paused(_) => {
                    audio.track_action(audio::Action::Stop(sad_song.into()));
                },
            }
        }
        renderer.render(&state);
        last_status = state.status;
    }
}
