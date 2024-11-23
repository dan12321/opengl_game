use std::sync::mpsc::Receiver;

use super::config;
use super::config::{MAX_ZOOM, MIN_ZOOM, SCROLL_ZOOM_SCALE};
use glfw::{Action, Glfw, Key, Window, WindowEvent};

pub struct Controller {
    direction_x: f32,
    camera_x: f32,
    camera_y: f32,
    zoom: f32,
    buttons_down: Vec<Button>,
    glfw: Glfw,
    events: Receiver<(f64, WindowEvent)>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Button {
    Restart,
    Quit,
    Pause,
    Level(usize),
}

impl Controller {
    pub fn new(glfw: Glfw, events: Receiver<(f64, WindowEvent)>) -> Self {
        Controller {
            direction_x: 0.0,
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 0.0,
            buttons_down: Vec::new(),
            glfw,
            events,
        }
    }

    pub fn poll_input(&mut self, window: &mut Window) {
        self.glfw.poll_events();
        let mut buttons = Vec::with_capacity(16);
        let mut x_set = false;
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    buttons.push(Button::Quit);
                }
                WindowEvent::Key(Key::Num1, _, Action::Press, _) => {
                    buttons.push(Button::Level(0));
                }
                WindowEvent::Key(Key::Num2, _, Action::Press, _) => {
                    buttons.push(Button::Level(1));
                }
                WindowEvent::Key(Key::R, _, Action::Press, _) => {
                    buttons.push(Button::Restart);
                }
                WindowEvent::Key(Key::P, _, Action::Press, _) => {
                    buttons.push(Button::Pause);
                }
                WindowEvent::Key(Key::Right, _, Action::Press, _) => {
                    self.direction_x = 1.0;
                    x_set = true;
                }
                WindowEvent::Key(Key::Left, _, Action::Press, _) => {
                    self.direction_x = -1.0;
                    x_set = true;
                }
                WindowEvent::CursorPos(x, y) => {
                    let x = x as f32 / config::CURSOR_MOVEMENT_SCALE;
                    let y = y as f32 / config::CURSOR_MOVEMENT_SCALE;
                    let min_cy = config::MIN_CAMERA_LATITUDE;
                    let max_cy = config::MAX_CAMERA_LATITUDE;
                    let cy_min_clamped = if y < min_cy { min_cy } else { y };
                    let cy_clamped = if cy_min_clamped > max_cy {
                        max_cy
                    } else {
                        cy_min_clamped
                    };
                    self.camera_x = x;
                    self.camera_y = cy_clamped;
                    window.set_cursor_pos(
                        (x * config::CURSOR_MOVEMENT_SCALE) as f64,
                        (cy_clamped * config::CURSOR_MOVEMENT_SCALE) as f64,
                    );
                }
                WindowEvent::Scroll(_, y) => {
                    let zoom = self.zoom - (y as f32 * SCROLL_ZOOM_SCALE);
                    let clamp_min = if zoom > MIN_ZOOM { zoom } else { MIN_ZOOM };
                    let clamp = if clamp_min < MAX_ZOOM {
                        clamp_min
                    } else {
                        MAX_ZOOM
                    };
                    self.zoom = clamp as f32;
                }
                _ => (),
            }
        }
        if !x_set {
            self.direction_x = 0.0;
        }
        self.buttons_down = buttons;
    }

    pub fn buttons(&self) -> &Vec<Button> {
        &self.buttons_down
    }

    pub fn direction(&self) -> f32 {
        self.direction_x
    }

    pub fn angle(&self) -> (f32, f32) {
        (self.camera_x, self.camera_y)
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }
}
