use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;

use glfw::{Context, Window};

use super::scenes::SceneManager;
use crate::audio::AudioManager;
use crate::controller::{Button, Controller};
use crate::render::Renderer;
use crate::resource::manager::ResourceManager;

pub struct Game {
    pub audio_manager: AudioManager,
    pub renderer: Renderer,
    resource_manager: Arc<ResourceManager>,
    pub controller: Controller,
    pub scene_manager: SceneManager,
    quit: Receiver<()>,
    pub window: Window,
}

impl Game {
    pub fn new() -> Self {
        // Window Setup
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        let window_width = 900;
        let window_height = 900;
        let (mut window, window_events) = glfw
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
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_cursor_mode(glfw::CursorMode::Normal);
        window.set_raw_mouse_motion(true);
        window.set_scroll_polling(true);

        // OpenGL Setup
        gl::load_with(|s| window.get_proc_address(s));

        // Setup managers
        let controller = Controller::new(glfw, window_events);
        let resource_manager = Arc::new(ResourceManager::new());

        let (audio_send, audio_rec) = mpsc::channel();
        let audio_manager = AudioManager::new(resource_manager.clone(), audio_rec);

        let (render_send, render_rec) = mpsc::channel();
        let renderer = Renderer::new(resource_manager.clone(), render_rec);
        // Initialising this will start loading the base resources
        let (quit_sender, quit_receiver) = mpsc::channel();
        let scene_manager = SceneManager::new(resource_manager.clone(), audio_send, render_send, quit_sender);

        // Start loading base resources
        Self {
            audio_manager,
            resource_manager,
            renderer,
            window,
            controller,
            scene_manager,
            quit: quit_receiver,
        }
    }

    pub fn update(&mut self, delta_time: Duration) -> bool {
        // Process Inputs
        self.controller.poll_input(&mut self.window);

        // Game Logic
        if self.quit.try_recv().is_ok() {
            // These should "take" the resource but can't with how this is written.
            self.audio_manager.cleanup();
            self.resource_manager.cleanup();
            self.window.set_should_close(true);
            return false;
        }
        self.scene_manager.update(
            &delta_time,
            &self.controller,
            &self.audio_manager,
            &self.renderer,
        );

        // Audio
        self.audio_manager.update();

        // Render
        self.renderer.update(&self.window, &self.scene_manager);
        self.window.swap_buffers();
        return true;
    }
}
