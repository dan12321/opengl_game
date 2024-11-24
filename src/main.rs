mod audio;
mod camera;
mod config;
mod controller;
mod physics;
mod render;
mod resource;
mod shader;
mod shape;
mod state;

extern crate glfw;
extern crate image;
extern crate nalgebra as na;
extern crate tracing;
extern crate tracing_subscriber;

use std::time::Instant;

use state::game::Game;
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

    // Program Setup
    let mut game = Game::new();
    debug!("Game Initialized");

    let mut last_time = Instant::now();
    loop {
        let current_time = Instant::now();
        let delta_time = current_time.duration_since(last_time);
        last_time = current_time;

        if !game.update(delta_time) {
            break;
        }
    }
    debug!("Game Closed")
}
