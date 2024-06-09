// Assets
pub const CUBE_VERT_SHADER: &'static str = "assets/shaders/cube.vert";
pub const TEXTURE_FRAG_SHADER: &'static str = "assets/shaders/texture.frag";
pub const LIGHT_VERT_SHADER: &'static str = "assets/shaders/light.vert";
pub const LIGHT_FRAG_SHADER: &'static str = "assets/shaders/light.frag";
pub const WALL_TEXTURE: &'static str = "assets/textures/scifiwall.jpg";
pub const CONTAINER_TEXTURE: &'static str = "assets/textures/container.png";
pub const CONTAINER_SPECULAR_TEXTURE: &'static str = "assets/textures/container_specular.png";

// Movement
pub const MOVE_SPEED: f32 = 15.0;
pub const CURSOR_MOVEMENT_SCALE: f32 = 360.0;
// TODO: LONGITUDE<=>LATITUDE
// TODO: Debug Min and max seem to be reflected
pub const MIN_CAMERA_LONGITUDE: f32 = -1.2;
pub const MAX_CAMERA_LONGITUDE: f32 = 0.3;
pub const SCROLL_ZOOM_SCALE: f32 = 1.0 / 5.0;
pub const MAX_ZOOM: f32 = 10.0;
pub const MIN_ZOOM: f32 = -5.0;
pub const BEAT_SIZE: f32 = 5.0;
pub const COLUMN_WIDTH: f32 = 1.75;

// PLANE
pub const PLANE_WIDTH: f32 = 5.0;
pub const PLANE_LENGTH: f32 = 100.0;
