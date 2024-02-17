use std::sync::mpsc::Receiver;

use super::config::{MAX_ZOOM, MIN_ZOOM, SCROLL_ZOOM_SCALE};
use glfw::{Action, Glfw, Key, WindowEvent};

pub struct Controller<'a> {
    direction_x: f32,
    direction_y: f32,
    camera_x: f32,
    camera_y: f32,
    zoom: f32,
    buttons_down: Vec<Button>,
    glfw: &'a mut Glfw,
    events: Receiver<(f64, WindowEvent)>,
}

pub enum Button {
    Quit,
}

impl<'a> Controller<'a> {
    pub fn new(glfw: &'a mut Glfw, events: Receiver<(f64, WindowEvent)>) -> Self {
        Controller {
            direction_x: 0.0,
            direction_y: 0.0,
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 0.0,
            buttons_down: Vec::new(),
            glfw,
            events,
        }
    }

    pub fn poll_input(&mut self) {
        self.glfw.poll_events();
        let mut buttons = Vec::with_capacity(16);
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    buttons.push(Button::Quit);
                }
                WindowEvent::Key(Key::Up, _, Action::Press, _)
                | WindowEvent::Key(Key::Down, _, Action::Release, _) => {
                    self.direction_y += -1.0;
                }
                WindowEvent::Key(Key::Down, _, Action::Press, _)
                | WindowEvent::Key(Key::Up, _, Action::Release, _) => {
                    self.direction_y -= -1.0;
                }
                WindowEvent::Key(Key::Right, _, Action::Press, _)
                | WindowEvent::Key(Key::Left, _, Action::Release, _) => {
                    self.direction_x += 1.0;
                }
                WindowEvent::Key(Key::Left, _, Action::Press, _)
                | WindowEvent::Key(Key::Right, _, Action::Release, _) => {
                    self.direction_x -= 1.0;
                }
                WindowEvent::CursorPos(x, y) => {
                    self.camera_x = x as f32;
                    self.camera_y = y as f32;
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
        self.buttons_down = buttons;
    }

    pub fn buttons(&self) -> &Vec<Button> {
        &self.buttons_down
    }

    pub fn direction(&self) -> (f32, f32) {
        (self.direction_x, self.direction_y)
    }

    pub fn mouse(&self) -> (f32, f32, f32) {
        (self.camera_x, self.camera_y, self.zoom)
    }
}
