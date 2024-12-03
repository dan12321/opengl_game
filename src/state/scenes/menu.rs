use std::{sync::{
    mpsc::{self, Sender},
    Arc,
}, time::Duration};

use anyhow::Result;
use na::Matrix4;
use tracing::{debug, field::debug};

use crate::{
    audio::{AudioManager, AudioMessage}, config::{SAD_MAP, UPBEAT_MAP}, controller::Controller, render::Renderer, resource::manager::ResourceManager
};

use super::{loading::LoadingState, Transform, UiElement};

pub struct MenuState {
    left_padding: f32,
    top_padding: f32,
    button_width: f32,
    button_height: f32,
    section_gap: f32,
    button_gap: f32,
    header: String,
    level_buttons: Vec<LevelButton>,
    additional_buttons: Vec<Button>,
    pub loading_scene: Option<String>,
}

impl MenuState {
    pub fn new(
        quit_send: Sender<()>,
    ) -> Self {
        Self {
            left_padding: 0.3,
            top_padding: 0.05,
            button_width: 0.75,
            button_height: 0.1,
            section_gap: 0.15,
            button_gap: 0.01,
            header: "Missed a Beat".to_string(),
            level_buttons: vec![
                LevelButton {
                    name: "Level 1".to_string(),
                    path: SAD_MAP.to_string(),
                    hover_time: 0.0,
                },
                LevelButton {
                    name: "Level 2".to_string(),
                    path: UPBEAT_MAP.to_string(),
                    hover_time: 0.0,
                },
            ],
            additional_buttons: vec![
                Button {
                    name: "Quit".to_string(),
                    hover_time: 0.0,
                    callback: Box::new(move || {
                        quit_send.send(()).unwrap();
                    }),
                },
            ],
            loading_scene: None,
        }
    }

    pub fn update(&mut self, delta_time: &Duration, controller: &Controller) {
        let (x, y) = controller.mouse_pos();
        let clicked = controller.mouse_click();

        if clicked {
            debug!(x=x, y=y, "clicked");
            for i in 0..self.level_buttons.len() {
                debug!(bounds=format!("{:?}", self.get_level_button_bounds(i)), "button_bounds");
            }
        }
        let hover_speed = 5.0;
        let hover_delta = delta_time.as_secs_f32() * hover_speed;

        for i in 0..self.level_buttons.len() {
            if self.coord_in_level_button(i, x, y) {
                let current_hover = self.level_buttons[i].hover_time;
                let new_hover = f32::min(1.0, current_hover + hover_delta);
                self.level_buttons[i].hover_time = new_hover;
                if clicked {
                    self.loading_scene = Some(self.level_buttons[i].path.clone());
                }
            } else {
                self.level_buttons[i].hover_time = 0.0;
            }
        }

        for i in 0..self.additional_buttons.len() {
            if self.coord_in_button(i, x, y) {
                let current_hover = self.additional_buttons[i].hover_time;
                let new_hover = f32::min(1.0, current_hover + hover_delta);
                self.additional_buttons[i].hover_time = new_hover;
                if clicked {
                    let callback = &self.additional_buttons[i].callback;
                    callback();
                }
            } else {
                self.additional_buttons[i].hover_time = 0.0;
            }
        }
    }

    pub fn get_ui_elements(&self) -> Vec<UiElement> {
        let mut result = Vec::new();
        let header_box = self.get_header_bounds();
        let header = UiElement {
            transform: header_box.get_transform(),
            base_color: (0.1, 0.1, 0.1).into(),
            progress_color: (0.0, 0.0, 0.0).into(),
            progress: 0.0,
            merge_color: (0.0, 0.0, 0.0).into(),
            merge_amount: 0.0,
        };
        result.push(header);

        for i in 0..self.level_buttons.len() {
            let level_box = self.get_level_button_bounds(i);
            let level = UiElement {
                transform: level_box.get_transform(),
                base_color: (0.1, 0.1, 0.9).into(),
                progress_color: (0.0, 0.0, 0.0).into(),
                progress: 0.0,
                merge_color: (1.0, 0.1, 0.1).into(),
                merge_amount: self.level_buttons[i].hover_time,
            };
            result.push(level);
        }

        for i in 0..self.additional_buttons.len() {
            let button_box = self.get_button_bounds(i);
            let button = UiElement {
                transform: button_box.get_transform(),
                base_color: (0.1, 0.9, 0.1).into(),
                progress_color: (0.0, 0.0, 0.0).into(),
                progress: 0.0,
                merge_color: (1.0, 0.0, 0.0).into(),
                merge_amount: self.additional_buttons[i].hover_time,
            };
            result.push(button);
        }
        result
    }

    fn coord_in_level_button(&self, index: usize, x: f32, y: f32) -> bool {
        let bounds = self.get_level_button_bounds(index);
        bounds.left < x && x < bounds.right && bounds.bottom < y && y < bounds.top
    }

    fn get_header_bounds(&self) -> BoxBounds {
        let left = 2.0 * (self.left_padding - 0.5);
        let right = left + self.button_width;
        let top = 2.0 * (1.0 - self.top_padding - 0.5);
        let bottom = top - self.button_height;
        BoxBounds {
            left,
            right,
            top,
            bottom,
        }
    }

    fn get_level_button_bounds(&self, index: usize) -> BoxBounds {
        let left = 2.0 * (self.left_padding - 0.5);
        let right = left + self.button_width;
        let top_section = self.top_padding + self.button_height + self.section_gap;
        let button_top = top_section + index as f32 * (self.button_height + self.button_gap);
        let top = 2.0 * (1.0 - button_top - 0.5);
        let bottom = top - self.button_height;
        BoxBounds {
            left,
            right,
            top,
            bottom,
        }
    }

    fn coord_in_button(&self, index: usize, x: f32, y: f32) -> bool {
        let bounds = self.get_button_bounds(index);
        bounds.left < x && x < bounds.right && bounds.bottom < y && y < bounds.top
    }

    fn get_button_bounds(&self, index: usize) -> BoxBounds {
        let left = 2.0 * (self.left_padding - 0.5);
        let right = left + self.button_width;
        let top_section = self.top_padding + self.button_height + 2.0 * self.section_gap + self.level_buttons.len() as f32 * (self.button_height + self.button_gap);
        let button_top = top_section + index as f32 * (self.button_height + self.button_gap);
        let top = 2.0 * (1.0 - button_top - 0.5);
        let bottom = top - self.button_height;
        BoxBounds {
            left,
            right,
            top,
            bottom,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BoxBounds {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

impl BoxBounds {
    fn get_transform(&self) -> Transform {
        let width = self.right - self.left;
        let height = self.top - self.bottom;
        Transform {
            position: (self.left, self.bottom, 0.0).into(),
            scale: (width, height, 1.0).into(),
            rotation: Matrix4::identity(),
        }
    }
}

struct LevelButton {
    name: String,
    path: String,
    hover_time: f32,
}

struct Button {
    name: String,
    callback: Box<dyn Fn()>,
    hover_time: f32,
}

