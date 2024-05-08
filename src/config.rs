// Assets
pub const CUBE_VERT_SHADER: &'static str = "assets/shaders/cube.vert";
pub const PLANE_VERT_SHADER: &'static str = "assets/shaders/plane.vert";
pub const TEXTURE_FRAG_SHADER: &'static str = "assets/shaders/texture.frag";
pub const LIGHT_VERT_SHADER: &'static str = "assets/shaders/light.vert";
pub const LIGHT_FRAG_SHADER: &'static str = "assets/shaders/light.frag";
pub const WALL_TEXTURE: &'static str = "assets/textures/scifiwall.jpg";

// Movement
pub const MOVE_SPEED: f32 = 6.0;
pub const ROTATION_SPEED: f32 = 5.0;
pub const CURSOR_MOVEMENT_SCALE: f32 = 1.0 / 360.0;
pub const MIN_CAMERA_LONGITUDE: f32 = -0.3;
pub const MAX_CAMERA_LONGITUDE: f32 = 1.2;
pub const SCROLL_ZOOM_SCALE: f32 = 1.0 / 5.0;
pub const MAX_ZOOM: f32 = 10.0;
pub const MIN_ZOOM: f32 = -5.0;

// PLANE
pub const PLANE_WIDTH: f32 = 5.0;
pub const PLANE_LENGTH: f32 = 100.0;
